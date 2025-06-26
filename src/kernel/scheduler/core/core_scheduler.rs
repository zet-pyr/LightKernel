//! # Core Scheduler Implementation
//! 
//! This module provides the main scheduling infrastructure that coordinates
//! all scheduler subsystems across multiple CPU cores. It implements a modern,
//! hierarchical scheduler with support for multiple scheduling policies,
//! load balancing, power management, and real-time guarantees.
//!
//! ## Architecture Overview
//! 
//! The core scheduler orchestrates multiple specialized schedulers:
//! - **Fair Scheduler (CFS)**: For normal tasks with proportional fairness
//! - **Real-Time Scheduler**: For FIFO and Round-Robin RT tasks
//! - **Deadline Scheduler**: For tasks with strict timing requirements
//! - **Idle Scheduler**: For background and idle tasks
//! - **Power Management**: CPU frequency and idle state management
//! - **Load Balancing**: Cross-CPU task migration and balancing
//!
//! ## Key Features
//! 
//! - Multi-core aware scheduling with NUMA topology support
//! - Hierarchical load tracking with Per-Entity Load Tracking (PELT)
//! - Advanced power management with DVFS integration
//! - Real-time scheduling with priority inheritance
//! - Deadline scheduling with bandwidth isolation
//! - Automatic load balancing and task migration
//! - Comprehensive debugging and statistics
//! - Memory barrier coordination for SMP safety
//!
//! ## Usage
//! 
//! ```rust
//! use crate::kernel::scheduler::CoreScheduler;
//! 
//! // Initialize the scheduler
//! let scheduler = CoreScheduler::new();
//! scheduler.init()?;
//! 
//! // Main scheduling loop (called from timer interrupt)
//! loop {
//!     scheduler.schedule()?;
//! }
//! ```

use crate::kernel::scheduler::core::CoreScheduler;
use crate::kernel::scheduler::clock::*;
use crate::kernel::scheduler::autogroup::*;
use crate::kernel::scheduler::completion::*;
use crate::kernel::scheduler::cpufreq::*;
use crate::kernel::scheduler::cpuidle::*;
use crate::kernel::scheduler::deadline::*;
use crate::kernel::scheduler::debug::*;
use crate::kernel::scheduler::domains::*;
use crate::kernel::scheduler::fair::*;
use crate::kernel::scheduler::idle::*;
use crate::kernel::scheduler::isolation::*;
use crate::kernel::scheduler::loadavg::*;
use crate::kernel::scheduler::membarrier::*;
use crate::kernel::scheduler::migration::*;
use crate::kernel::scheduler::features::*;
use crate::kernel::scheduler::rt::*;
use crate::kernel::scheduler::stats::*;
use crate::kernel::scheduler::stop_task::*;
use crate::kernel::scheduler::swait::*;
use crate::kernel::scheduler::wait::*;
use crate::kernel::scheduler::pelt::*;
use crate::kernel::scheduler::preempt::*;
use crate::kernel::scheduler::topology::*;

use crate::kernel::task::{Task, TaskId, TaskPriority, TaskState};
use crate::kernel::cpu::{CpuId, CpuMask};
use crate::kernel::time::{Timestamp, Duration};
use crate::kernel::error::{KernelResult, SchedulerError};
use crate::kernel::sync::{SpinLock, RwLock, Mutex};
use crate::kernel::log::{kernel_info, kernel_warn, kernel_error, kernel_debug};
use crate::kernel::memory::percpu::PerCpu;
use crate::arch::context::Context;
use crate::arch::cpu::current_cpu_id;

use alloc::vec::Vec;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicU32, Ordering};
use core::time::Duration as CoreDuration;

/// Core scheduler state with enhanced state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SchedulerState {
    /// Scheduler not initialized
    Uninitialized = 0,
    /// Currently initializing
    Initializing = 1,
    /// Normal operation
    Running = 2,
    /// Temporarily suspended
    Suspended = 3,
    /// Gracefully stopping
    Stopping = 4,
    /// Completely stopped
    Stopped = 5,
    /// Error state requiring restart
    Error = 6,
}

impl SchedulerState {
    /// Check if scheduler can accept new tasks
    pub fn can_schedule(&self) -> bool {
        matches!(self, SchedulerState::Running)
    }
    
    /// Check if scheduler is in a stable state
    pub fn is_stable(&self) -> bool {
        matches!(self, SchedulerState::Running | SchedulerState::Stopped)
    }
}

/// Enhanced scheduling policy types with additional metadata
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum SchedPolicy {
    /// CFS (Completely Fair Scheduler) - default for normal tasks
    Normal = 0,
    /// FIFO real-time scheduling
    Fifo = 1,
    /// Round-robin real-time scheduling  
    RoundRobin = 2,
    /// Batch processing (lower priority than normal)
    Batch = 3,
    /// Idle tasks (lowest priority)
    Idle = 4,
    /// Deadline scheduling with timing guarantees
    Deadline = 5,
    /// Interactive tasks (higher responsiveness)
    Interactive = 6,
    /// Background tasks (very low priority)
    Background = 7,
}

impl SchedPolicy {
    /// Get the base priority class for this policy
    pub fn priority_class(&self) -> u32 {
        match self {
            SchedPolicy::Fifo | SchedPolicy::RoundRobin => 100,     // RT priority
            SchedPolicy::Deadline => 90,                            // Deadline priority
            SchedPolicy::Interactive => 80,                         // Interactive priority
            SchedPolicy::Normal => 50,                              // Normal priority
            SchedPolicy::Batch => 30,                               // Batch priority
            SchedPolicy::Background => 10,                          // Background priority
            SchedPolicy::Idle => 0,                                 // Idle priority
        }
    }
    
    /// Check if this is a real-time policy
    pub fn is_realtime(&self) -> bool {
        matches!(self, SchedPolicy::Fifo | SchedPolicy::RoundRobin | SchedPolicy::Deadline)
    }
    
    /// Get the scheduler responsible for this policy
    pub fn scheduler_name(&self) -> &'static str {
        match self {
            SchedPolicy::Normal | SchedPolicy::Interactive => "CFS",
            SchedPolicy::Fifo | SchedPolicy::RoundRobin => "RT",
            SchedPolicy::Deadline => "DL",
            SchedPolicy::Batch | SchedPolicy::Background => "BATCH",
            SchedPolicy::Idle => "IDLE",
        }
    }
}

/// Comprehensive scheduler statistics with performance metrics
#[derive(Debug, Default)]
pub struct SchedulerStats {
    /// Total context switches across all CPUs
    pub context_switches: AtomicU64,
    /// Number of preemptions (involuntary context switches)
    pub preemptions: AtomicU64,
    /// Task migrations between CPUs
    pub migrations: AtomicU64,
    /// Load balancing operations
    pub load_balance_calls: AtomicU64,
    /// Scheduler timer ticks processed
    pub scheduler_ticks: AtomicU64,
    /// Failed scheduling attempts
    pub schedule_failures: AtomicU64,
    /// Tasks created
    pub tasks_created: AtomicU64,
    /// Tasks destroyed
    pub tasks_destroyed: AtomicU64,
    /// RT throttling events
    pub rt_throttled: AtomicU64,
    /// Deadline misses
    pub deadline_misses: AtomicU64,
    /// CPU idle time (microseconds)
    pub cpu_idle_time: AtomicU64,
    /// Average scheduling latency (nanoseconds)
    pub avg_schedule_latency: AtomicU64,
    /// Peak scheduling latency (nanoseconds)
    pub peak_schedule_latency: AtomicU64,
    /// System load (fixed point, multiplied by 1000)
    pub system_load: AtomicU32,
}

impl SchedulerStats {
    /// Get context switches per second
    pub fn context_switches_per_sec(&self, uptime_secs: u64) -> f64 {
        if uptime_secs == 0 { return 0.0; }
        self.context_switches.load(Ordering::Relaxed) as f64 / uptime_secs as f64
    }
    
    /// Get system load as percentage
    pub fn system_load_percent(&self) -> f64 {
        self.system_load.load(Ordering::Relaxed) as f64 / 10.0
    }
    
    /// Reset statistics counters
    pub fn reset(&self) {
        self.context_switches.store(0, Ordering::Relaxed);
        self.preemptions.store(0, Ordering::Relaxed);
        self.migrations.store(0, Ordering::Relaxed);
        self.load_balance_calls.store(0, Ordering::Relaxed);
        self.schedule_failures.store(0, Ordering::Relaxed);
        self.rt_throttled.store(0, Ordering::Relaxed);
        self.deadline_misses.store(0, Ordering::Relaxed);
        self.avg_schedule_latency.store(0, Ordering::Relaxed);
        self.peak_schedule_latency.store(0, Ordering::Relaxed);
    }
}

/// Per-CPU scheduler data for efficient SMP scaling
#[derive(Debug, Default)]
pub struct PerCpuSchedulerData {
    /// CPU-specific runqueue statistics
    pub runqueue_size: AtomicU32,
    /// Last scheduling decision timestamp
    pub last_schedule_time: AtomicU64,
    /// CPU utilization (0-1000 for 0-100.0%)
    pub cpu_utilization: AtomicU32,
    /// Current task running on this CPU
    pub current_task: Mutex<Option<TaskId>>,
    /// Next task to run (pre-selected)
    pub next_task: Mutex<Option<TaskId>>,
    /// CPU frequency scaling factor
    pub freq_scale: AtomicU32,
    /// Idle state information
    pub idle_state: AtomicU32,
    /// Local scheduling statistics
    pub local_stats: SchedulerStats,
}

/// Scheduling decision result
#[derive(Debug, Clone)]
pub enum ScheduleResult {
    /// Continue running current task
    KeepCurrent,
    /// Switch to new task
    SwitchTo(TaskId),
    /// CPU should go idle
    GoIdle,
    /// Reschedule immediately (high priority task arrived)
    RescheduleImmediate,
}

/// Load balancing configuration
#[derive(Debug, Clone)]
pub struct LoadBalanceConfig {
    /// Enable aggressive load balancing
    pub aggressive_balance: bool,
    /// Minimum imbalance threshold (0-100%)
    pub imbalance_threshold: u32,
    /// Maximum tasks to migrate per balance operation
    pub max_migrations_per_balance: u32,
    /// Load balance interval in ticks
    pub balance_interval: u64,
    /// Enable NUMA-aware balancing
    pub numa_aware: bool,
}

impl Default for LoadBalanceConfig {
    fn default() -> Self {
        Self {
            aggressive_balance: false,
            imbalance_threshold: 25,
            max_migrations_per_balance: 4,
            balance_interval: 100,
            numa_aware: true,
        }
    }
}

/// Enhanced scheduler configuration
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Enable preemption
    pub preemption_enabled: bool,
    /// Scheduling tick frequency (Hz)
    pub tick_frequency: u32,
    /// Default time slice for RR tasks (microseconds)
    pub default_timeslice: u64,
    /// Load balancing configuration
    pub load_balance: LoadBalanceConfig,
    /// Enable power-aware scheduling
    pub power_aware: bool,
    /// Maximum RT bandwidth (percent of CPU time)
    pub rt_bandwidth_percent: u32,
    /// Enable scheduler debugging
    pub debug_enabled: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            preemption_enabled: true,
            tick_frequency: 1000, // 1000 Hz
            default_timeslice: 10_000, // 10ms
            load_balance: LoadBalanceConfig::default(),
            power_aware: true,
            rt_bandwidth_percent: 95,
            debug_enabled: false,
        }
    }
}

/// Core scheduler structure with all subsystems
pub struct CoreScheduler {
    // Core scheduling components
    clock: ClockScheduler,
    autogroup: AutoGroupScheduler,
    completion: CompletionScheduler,
    cpufreq: CpuFreqScheduler,
    cpuidle: CpuIdleScheduler,
    deadline: DeadlineScheduler,
    debug: DebugScheduler,
    domains: DomainsScheduler,
    fair: FairScheduler,
    idle: IdleScheduler,
    isolation: IsolationScheduler,
    loadavg: LoadAvgScheduler,
    membarrier: MembarrierScheduler,
    migration: MigrationScheduler,
    features: FeaturesScheduler,
    rt: RtScheduler,
    stats: StatsScheduler,
    stop_task: StopTaskScheduler,
    swait: SwaitScheduler,
    wait: WaitScheduler,
    pelt: PeltScheduler,
    preempt: PreemptScheduler,
    topology: TopologyScheduler,
    
    // Enhanced scheduler state
    state: AtomicU64,
    config: RwLock<SchedulerConfig>,
    global_stats: SchedulerStats,
    per_cpu_data: PerCpu<PerCpuSchedulerData>,
    tick_counter: AtomicU64,
    last_balance_time: AtomicU64,
    emergency_stop: AtomicBool,
    init_timestamp: AtomicU64,
}

impl CoreScheduler {
    /// Create a new core scheduler instance with configuration
    pub fn new() -> Self {
        Self::with_config(SchedulerConfig::default())
    }
    
    /// Create scheduler with custom configuration
    pub fn with_config(config: SchedulerConfig) -> Self {
        kernel_info!("Creating core scheduler with config: {:?}", config);
        
        CoreScheduler {
            // Core scheduling components
            clock: ClockScheduler::new(),
            autogroup: AutoGroupScheduler::new(),
            completion: CompletionScheduler::new(),
            cpufreq: CpuFreqScheduler::new(),
            cpuidle: CpuIdleScheduler::new(),
            deadline: DeadlineScheduler::with_config(config.rt_bandwidth_percent),
            debug: DebugScheduler::new(),
            domains: DomainsScheduler::new(),
            fair: FairScheduler::with_timeslice(config.default_timeslice),
            idle: IdleScheduler::new(),
            isolation: IsolationScheduler::new(),
            loadavg: LoadAvgScheduler::new(),
            membarrier: MembarrierScheduler::new(),
            migration: MigrationScheduler::with_config(config.load_balance.clone()),
            features: FeaturesScheduler::new(),
            rt: RtScheduler::with_bandwidth(config.rt_bandwidth_percent),
            stats: StatsScheduler::new(),
            stop_task: StopTaskScheduler::new(),
            swait: SwaitScheduler::new(),
            wait: WaitScheduler::new(),
            pelt: PeltScheduler::new(),
            preempt: PreemptScheduler::with_enabled(config.preemption_enabled),
            topology: TopologyScheduler::new(),
            
            // Enhanced scheduler state
            state: AtomicU64::new(SchedulerState::Uninitialized as u64),
            config: RwLock::new(config),
            global_stats: SchedulerStats::default(),
            per_cpu_data: PerCpu::new(PerCpuSchedulerData::default()),
            tick_counter: AtomicU64::new(0),
            last_balance_time: AtomicU64::new(0),
            emergency_stop: AtomicBool::new(false),
            init_timestamp: AtomicU64::new(0),
        }
    }

    /// Initialize the scheduler with comprehensive error handling and validation
    pub fn init(&self) -> KernelResult<()> {
        let start_time = Timestamp::now();
        self.init_timestamp.store(start_time.as_nanos(), Ordering::Release);
        
        kernel_info!("Initializing core scheduler...");
        self.set_state(SchedulerState::Initializing);

        // Validate system requirements
        self.validate_system_requirements()?;

        // Initialize components in dependency order
        self.init_core_infrastructure()?;
        self.init_cpu_management()?;
        self.init_scheduling_policies()?;
        self.init_synchronization()?;
        self.init_load_tracking()?;
        self.init_advanced_features()?;
        self.init_debugging()?;

        // Initialize per-CPU data structures
        self.init_per_cpu_data()?;
        
        // Perform system validation
        self.validate_scheduler_state()?;

        // Set state to running
        self.set_state(SchedulerState::Running);
        
        let init_time = Timestamp::now().as_nanos() - start_time.as_nanos();
        kernel_info!("Core scheduler initialized successfully in {} μs", init_time / 1000);
        
        Ok(())
    }

    /// Main scheduler entry point with enhanced error handling and metrics
    pub fn schedule(&self) -> KernelResult<()> {
        let schedule_start = Timestamp::now();
        
        // Quick state check
        if !self.is_running() {
            return Err(SchedulerError::NotRunning.into());
        }
        
        // Check for emergency stop
        if self.emergency_stop.load(Ordering::Acquire) {
            return self.emergency_shutdown();
        }

        // Increment tick counter
        let current_tick = self.tick_counter.fetch_add(1, Ordering::Relaxed);
        self.global_stats.scheduler_ticks.fetch_add(1, Ordering::Relaxed);

        // Update scheduler subsystems
        self.update_scheduler_subsystems(current_tick)?;
        
        // Perform load balancing if needed
        self.maybe_load_balance(current_tick)?;
        
        // Main scheduling decision
        let schedule_result = self.make_scheduling_decision()?;
        
        // Execute scheduling decision
        self.execute_schedule_result(schedule_result)?;
        
        // Update scheduling latency metrics
        let schedule_time = Timestamp::now().as_nanos() - schedule_start.as_nanos();
        self.update_latency_stats(schedule_time);
        
        Ok(())
    }

    /// Enhanced scheduling decision with policy-aware selection
    fn make_scheduling_decision(&self) -> KernelResult<ScheduleResult> {
        let current_cpu = current_cpu_id();
        let current_task = self.get_current_task(current_cpu);
        
        // Check for stop tasks first (highest priority)
        if let Some(stop_task) = self.stop_task.pick_next_task(current_cpu)? {
            return Ok(ScheduleResult::SwitchTo(stop_task.id()));
        }
        
        // Handle real-time tasks (second highest priority)
        if let Some(rt_task) = self.rt.pick_next_task(current_cpu)? {
            // Check if we need to preempt current task
            if let Some(current) = current_task {
                if self.should_preempt_for_rt(&current, &rt_task)? {
                    self.global_stats.preemptions.fetch_add(1, Ordering::Relaxed);
                    return Ok(ScheduleResult::SwitchTo(rt_task.id()));
                }
            } else {
                return Ok(ScheduleResult::SwitchTo(rt_task.id()));
            }
        }
        
        // Handle deadline tasks (third priority)
        if let Some(dl_task) = self.deadline.pick_next_task(current_cpu)? {
            if let Some(current) = current_task {
                if self.should_preempt_for_deadline(&current, &dl_task)? {
                    self.global_stats.preemptions.fetch_add(1, Ordering::Relaxed);
                    return Ok(ScheduleResult::SwitchTo(dl_task.id()));
                }
            } else {
                return Ok(ScheduleResult::SwitchTo(dl_task.id()));
            }
        }
        
        // Handle fair (CFS) tasks
        if let Some(fair_task) = self.fair.pick_next_task(current_cpu)? {
            // Check if current task should be preempted
            if let Some(current) = current_task {
                if self.should_preempt_for_fair(&current, &fair_task)? {
                    return Ok(ScheduleResult::SwitchTo(fair_task.id()));
                } else {
                    return Ok(ScheduleResult::KeepCurrent);
                }
            } else {
                return Ok(ScheduleResult::SwitchTo(fair_task.id()));
            }
        }
        
        // No runnable tasks - check if current task can continue
        if let Some(current) = current_task {
            if current.state() == TaskState::Running {
                return Ok(ScheduleResult::KeepCurrent);
            }
        }
        
        // Fall back to idle
        Ok(ScheduleResult::GoIdle)
    }

    /// Execute the scheduling decision with comprehensive error handling
    fn execute_schedule_result(&self, result: ScheduleResult) -> KernelResult<()> {
        match result {
            ScheduleResult::KeepCurrent => {
                // Nothing to do, continue current task
                Ok(())
            }
            ScheduleResult::SwitchTo(task_id) => {
                let task = Task::get_by_id(task_id)
                    .ok_or(SchedulerError::TaskNotFound)?;
                self.switch_to_task(&task)
            }
            ScheduleResult::GoIdle => {
                let current_cpu = current_cpu_id();
                let idle_task = self.idle.get_idle_task(current_cpu)?;
                self.switch_to_task(&idle_task)
            }
            ScheduleResult::RescheduleImmediate => {
                // Trigger immediate reschedule
                self.preempt.request_reschedule()?;
                Ok(())
            }
        }
    }

    /// Enhanced task switching with comprehensive state management
    fn switch_to_task(&self, new_task: &Task) -> KernelResult<()> {
        let switch_start = Timestamp::now();
        let current_cpu = current_cpu_id();
        
        // Get current task (if any)
        let current_task = Task::current();
        
        // Validate the switch is legal
        self.validate_task_switch(current_task.as_ref(), new_task)?;
        
        // Update statistics
        self.global_stats.context_switches.fetch_add(1, Ordering::Relaxed);
        
        // Handle preemption logic
        if let Some(current) = current_task.as_ref() {
            self.preempt.handle_task_preemption(current)?;
        }
        
        // Notify schedulers about the switch
        self.notify_task_switch(current_task.as_ref(), new_task)?;
        
        // Perform the actual context switch
        self.perform_context_switch(current_task.as_ref(), new_task)?;
        
        // Update per-CPU data
        self.update_per_cpu_current_task(current_cpu, new_task.id())?;
        
        // Update task accounting
        new_task.on_cpu_switch(current_cpu)?;
        new_task.set_last_run(Timestamp::now());
        
        // Update switch latency
        let switch_time = Timestamp::now().as_nanos() - switch_start.as_nanos();
        self.update_switch_latency(switch_time);
        
        kernel_debug!("Task switch: {} -> {} on CPU {}", 
                     current_task.map(|t| t.id().as_u64()).unwrap_or(0),
                     new_task.id().as_u64(), 
                     current_cpu.as_u32());
        
        Ok(())
    }

    /// Enhanced task wake up with policy-aware handling
    pub fn wake_up_task(&self, task: &Task) -> KernelResult<()> {
        if !self.is_running() {
            return Err(SchedulerError::NotRunning.into());
        }
        
        kernel_debug!("Waking up task {} with policy {:?}", 
                     task.id().as_u64(), task.sched_policy());
        
        // Update task state
        task.set_state(TaskState::Runnable);
        task.set_wake_time(Timestamp::now());
        
        // Enqueue in appropriate scheduler
        match task.sched_policy() {
            SchedPolicy::Normal | SchedPolicy::Interactive => {
                self.fair.enqueue_task(task)?;
            }
            SchedPolicy::Batch | SchedPolicy::Background => {
                self.fair.enqueue_task_batch(task)?;
            }
            SchedPolicy::Fifo | SchedPolicy::RoundRobin => {
                self.rt.enqueue_task(task)?;
                // RT tasks may need immediate preemption
                if self.rt.should_preempt_current(task)? {
                    self.preempt.request_reschedule()?;
                }
            }
            SchedPolicy::Deadline => {
                self.deadline.enqueue_task(task)?;
                // Deadline tasks may need immediate preemption
                if self.deadline.should_preempt_current(task)? {
                    self.preempt.request_reschedule()?;
                }
            }
            SchedPolicy::Idle => {
                self.idle.enqueue_task(task)?;
            }
        }
        
        // Update statistics
        self.update_wakeup_stats(task);
        
        Ok(())
    }

    /// Intelligent load balancing with NUMA awareness
    pub fn load_balance(&self) -> KernelResult<()> {
        if !self.is_running() {
            return Ok(());
        }

        let balance_start = Timestamp::now();
        self.global_stats.load_balance_calls.fetch_add(1, Ordering::Relaxed);
        
        kernel_debug!("Starting load balance operation");
        
        // Get load balancing configuration
        let config = self.config.read().load_balance.clone();
        
        // Check if enough time has passed since last balance
        let current_time = balance_start.as_nanos();
        let last_balance = self.last_balance_time.load(Ordering::Acquire);
        let balance_interval_ns = config.balance_interval * 1_000_000; // Convert to nanoseconds
        
        if current_time - last_balance < balance_interval_ns {
            return Ok(()); // Too soon for another balance
        }
        
        // Perform the load balancing
        let migrations = self.migration.balance_load_intelligent(&config)?;
        
        // Update statistics
        self.global_stats.migrations.fetch_add(migrations as u64, Ordering::Relaxed);
        self.last_balance_time.store(current_time, Ordering::Release);
        
        let balance_time = Timestamp::now().as_nanos() - balance_start.as_nanos();
        kernel_debug!("Load balance completed: {} migrations in {} μs", 
                     migrations, balance_time / 1000);
        
        Ok(())
    }

    /// Migrate task with comprehensive validation and state management
    pub fn migrate_task(&self, task: &Task, target_cpu: CpuId) -> KernelResult<()> {
        // Validate migration is possible
        if !task.can_migrate_to(target_cpu)? {
            return Err(SchedulerError::MigrationNotAllowed.into());
        }
        
        // Check CPU affinity
        if !task.cpu_affinity().contains(target_cpu) {
            return Err(SchedulerError::AffinityViolation.into());
        }
        
        kernel_debug!("Migrating task {} from CPU {} to CPU {}", 
                     task.id().as_u64(), task.current_cpu().as_u32(), target_cpu.as_u32());
        
        // Perform migration
        let result = self.migration.migrate_task_safe(task, target_cpu);
        
        if result.is_ok() {
            self.global_stats.migrations.fetch_add(1, Ordering::Relaxed);
        }
        
        result
    }

    /// Enhanced scheduler debugging with detailed information
    pub fn debug_info(&self) -> KernelResult<()> {
        if !self.config.read().debug_enabled {
            return Ok(());
        }
        
        kernel_info!("=== Scheduler Debug Information ===");
        kernel_info!("State: {:?}", self.get_state());
        kernel_info!("Uptime: {} ticks", self.uptime_ticks());
        
        // Global statistics
        let stats = &self.global_stats;
        kernel_info!("Context switches: {}", stats.context_switches.load(Ordering::Relaxed));
        kernel_info!("Preemptions: {}", stats.preemptions.load(Ordering::Relaxed));
        kernel_info!("Migrations: {}", stats.migrations.load(Ordering::Relaxed));
        kernel_info!("Load balance calls: {}", stats.load_balance_calls.load(Ordering::Relaxed));
        kernel_info!("Schedule failures: {}", stats.schedule_failures.load(Ordering::Relaxed));
        kernel_info!("RT throttled: {}", stats.rt_throttled.load(Ordering::Relaxed));
        kernel_info!("Deadline misses: {}", stats.deadline_misses.load(Ordering::Relaxed));
        kernel_info!("Avg schedule latency: {} ns", stats.avg_schedule_latency.load(Ordering::Relaxed));
        kernel_info!("Peak schedule latency: {} ns", stats.peak_schedule_latency.load(Ordering::Relaxed));
        kernel_info!("System load: {:.1}%", stats.system_load_percent());
        
        // Per-CPU information
        self.debug_per_cpu_info()?;
        
        // Scheduler-specific debug info
        self.debug.print_scheduler_info()?;
        self.fair.print_fair_info()?;
        self.rt.print_rt_info()?;
        self.deadline.print_deadline_info()?;
        self.idle.print_idle_info()?;
        kernel_info!("=== End of Scheduler Debug Information ===");
        Ok(())
    }
}