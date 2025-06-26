/// File for the PSI (Pressure Stall Information) scheduler module.
/// This module is responsible for managing the PSI metrics
/// and providing insights into system pressure for scheduling decisions.
/// This file is part of the kernel's scheduler subsystem.

use std::time::{Duration, Instant};
use std::collections::HashMap;

// Import PSI-related modules
use crate::kernel::scheduler::psi::metrics::PSIMetrics;
use crate::kernel::scheduler::psi::pressure::Pressure;
use crate::kernel::scheduler::psi::pressure_type::PressureType;
use crate::kernel::scheduler::psi::pressure_state::PressureState;
use crate::kernel::scheduler::psi::pressure_tracker::{
    PressureTracker, PressureTrackerState, PressureTrackerType,
    PressureTrackerConfig, PressureTrackerMetrics
};

/// PSI pressure thresholds for different severity levels
#[derive(Debug, Clone, Copy)]
pub struct PSIThresholds {
    pub low: f64,
    pub medium: f64,
    pub high: f64,
    pub critical: f64,
}

impl Default for PSIThresholds {
    fn default() -> Self {
        Self {
            low: 10.0,      // 10% pressure
            medium: 30.0,   // 30% pressure
            high: 60.0,     // 60% pressure
            critical: 90.0, // 90% pressure
        }
    }
}

/// PSI severity levels based on pressure measurements
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PSISeverity {
    None,
    Low,
    Medium,
    High,
    Critical,
}

/// PSI configuration for the scheduler
#[derive(Debug, Clone)]
pub struct PSIConfig {
    pub enabled: bool,
    pub update_interval: Duration,
    pub thresholds: PSIThresholds,
    pub history_size: usize,
}

impl Default for PSIConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            update_interval: Duration::from_millis(100), // 100ms update interval
            thresholds: PSIThresholds::default(),
            history_size: 60, // Keep 60 measurements (6 seconds at 100ms intervals)
        }
    }
}

/// PSI history entry for tracking pressure over time
#[derive(Debug, Clone)]
pub struct PSIHistoryEntry {
    pub timestamp: Instant,
    pub cpu_pressure: f64,
    pub memory_pressure: f64,
    pub io_pressure: f64,
    pub severity: PSISeverity,
}

/// Main PSI scheduler structure
#[derive(Debug)]
pub struct PSIScheduler {
    config: PSIConfig,
    metrics: PSIMetrics,
    pressure_tracker: PressureTracker,
    history: Vec<PSIHistoryEntry>,
    last_update: Instant,
    pressure_events: HashMap<PressureType, u64>,
}

impl PSIScheduler {
    /// Create a new PSI scheduler with default configuration
    pub fn new() -> Self {
        Self::with_config(PSIConfig::default())
    }

    /// Create a new PSI scheduler with custom configuration
    pub fn with_config(config: PSIConfig) -> Self {
        Self {
            config,
            metrics: PSIMetrics::new(),
            pressure_tracker: PressureTracker::new(),
            history: Vec::new(),
            last_update: Instant::now(),
            pressure_events: HashMap::new(),
        }
    }

    /// Update PSI metrics and perform pressure analysis
    pub fn update_metrics(&mut self) {
        let now = Instant::now();
        
        // Check if enough time has passed since last update
        if now.duration_since(self.last_update) < self.config.update_interval {
            return;
        }

        // Update the pressure tracker
        self.pressure_tracker.update();
        
        // Get current pressure measurements
        let cpu_pressure = self.pressure_tracker.get_cpu_pressure();
        let memory_pressure = self.pressure_tracker.get_memory_pressure();
        let io_pressure = self.pressure_tracker.get_io_pressure();

        // Determine severity level
        let max_pressure = cpu_pressure.max(memory_pressure).max(io_pressure);
        let severity = self.calculate_severity(max_pressure);

        // Update metrics
        self.metrics.update_with_pressures(cpu_pressure, memory_pressure, io_pressure);

        // Record pressure events
        self.record_pressure_events(severity);

        // Add to history
        let entry = PSIHistoryEntry {
            timestamp: now,
            cpu_pressure,
            memory_pressure,
            io_pressure,
            severity,
        };
        
        self.add_history_entry(entry);
        self.last_update = now;
    }

    /// Calculate PSI severity based on pressure value
    fn calculate_severity(&self, pressure: f64) -> PSISeverity {
        let thresholds = &self.config.thresholds;
        
        if pressure >= thresholds.critical {
            PSISeverity::Critical
        } else if pressure >= thresholds.high {
            PSISeverity::High
        } else if pressure >= thresholds.medium {
            PSISeverity::Medium
        } else if pressure >= thresholds.low {
            PSISeverity::Low
        } else {
            PSISeverity::None
        }
    }

    /// Record pressure events for statistics
    fn record_pressure_events(&mut self, severity: PSISeverity) {
        match severity {
            PSISeverity::Critical => {
                *self.pressure_events.entry(PressureType::Critical).or_insert(0) += 1;
            }
            PSISeverity::High => {
                *self.pressure_events.entry(PressureType::High).or_insert(0) += 1;
            }
            PSISeverity::Medium => {
                *self.pressure_events.entry(PressureType::Medium).or_insert(0) += 1;
            }
            _ => {} // Don't record low or none events
        }
    }

    /// Add entry to history, maintaining size limit
    fn add_history_entry(&mut self, entry: PSIHistoryEntry) {
        self.history.push(entry);
        
        // Keep history within configured size
        if self.history.len() > self.config.history_size {
            self.history.remove(0);
        }
    }

    /// Get current PSI metrics
    pub fn get_metrics(&self) -> &PSIMetrics {
        &self.metrics
    }

    /// Get current pressure severity
    pub fn get_current_severity(&self) -> PSISeverity {
        self.history.last()
            .map(|entry| entry.severity)
            .unwrap_or(PSISeverity::None)
    }

    /// Get average pressure over the last N entries
    pub fn get_average_pressure(&self, entries: usize) -> (f64, f64, f64) {
        let count = entries.min(self.history.len());
        if count == 0 {
            return (0.0, 0.0, 0.0);
        }

        let start_idx = self.history.len() - count;
        let recent_entries = &self.history[start_idx..];

        let (cpu_sum, mem_sum, io_sum) = recent_entries.iter().fold(
            (0.0, 0.0, 0.0),
            |(cpu_acc, mem_acc, io_acc), entry| {
                (
                    cpu_acc + entry.cpu_pressure,
                    mem_acc + entry.memory_pressure,
                    io_acc + entry.io_pressure,
                )
            },
        );

        let count_f64 = count as f64;
        (cpu_sum / count_f64, mem_sum / count_f64, io_sum / count_f64)
    }

    /// Check if system is under pressure
    pub fn is_under_pressure(&self) -> bool {
        matches!(
            self.get_current_severity(),
            PSISeverity::Medium | PSISeverity::High | PSISeverity::Critical
        )
    }

    /// Check if system is under critical pressure
    pub fn is_critical_pressure(&self) -> bool {
        matches!(self.get_current_severity(), PSISeverity::Critical)
    }

    /// Get pressure event statistics
    pub fn get_pressure_events(&self) -> &HashMap<PressureType, u64> {
        &self.pressure_events
    }

    /// Reset all PSI metrics and history
    pub fn reset(&mut self) {
        self.metrics.reset();
        self.pressure_tracker.reset();
        self.history.clear();
        self.pressure_events.clear();
        self.last_update = Instant::now();
    }

    /// Get PSI configuration
    pub fn get_config(&self) -> &PSIConfig {
        &self.config
    }

    /// Update PSI configuration
    pub fn update_config(&mut self, config: PSIConfig) {
        self.config = config;
    }

    /// Enable or disable PSI monitoring
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    /// Print detailed PSI metrics for debugging
    pub fn print_detailed_metrics(&self) {
        println!("=== PSI Scheduler Metrics ===");
        println!("Enabled: {}", self.config.enabled);
        println!("Current Severity: {:?}", self.get_current_severity());
        
        if let Some(last_entry) = self.history.last() {
            println!("Latest Pressures:");
            println!("  CPU: {:.2}%", last_entry.cpu_pressure);
            println!("  Memory: {:.2}%", last_entry.memory_pressure);
            println!("  I/O: {:.2}%", last_entry.io_pressure);
        }

        // Show averages
        let (avg_cpu, avg_mem, avg_io) = self.get_average_pressure(10);
        println!("10-Sample Averages:");
        println!("  CPU: {:.2}%", avg_cpu);
        println!("  Memory: {:.2}%", avg_mem);
        println!("  I/O: {:.2}%", avg_io);

        // Show pressure events
        println!("Pressure Events:");
        for (pressure_type, count) in &self.pressure_events {
            println!("  {:?}: {}", pressure_type, count);
        }

        println!("History Size: {}/{}", self.history.len(), self.config.history_size);
        println!("==============================");
    }

    /// Get scheduling hint based on current PSI state
    pub fn get_scheduling_hint(&self) -> SchedulingHint {
        match self.get_current_severity() {
            PSISeverity::Critical => SchedulingHint::ReduceLoad,
            PSISeverity::High => SchedulingHint::LimitNewTasks,
            PSISeverity::Medium => SchedulingHint::PreferLightTasks,
            PSISeverity::Low => SchedulingHint::Normal,
            PSISeverity::None => SchedulingHint::Normal,
        }
    }
}

/// Scheduling hints based on PSI pressure levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulingHint {
    Normal,           // Normal scheduling
    PreferLightTasks, // Prefer lighter tasks
    LimitNewTasks,    // Limit creation of new tasks
    ReduceLoad,       // Actively reduce system load
}

impl Default for PSIScheduler {
    fn default() -> Self {
        Self::new()
    }
}

// Enhanced PSIMetrics implementation
impl PSIMetrics {
    /// Create new PSI metrics instance
    pub fn new() -> Self {
        Self {
            cpu_pressure: 0.0,
            memory_pressure: 0.0,
            io_pressure: 0.0,
            last_updated: Instant::now(),
        }
    }

    /// Update metrics with specific pressure values
    pub fn update_with_pressures(&mut self, cpu: f64, memory: f64, io: f64) {
        self.cpu_pressure = cpu;
        self.memory_pressure = memory;
        self.io_pressure = io;
        self.last_updated = Instant::now();
    }

    /// Get the maximum pressure across all types
    pub fn get_max_pressure(&self) -> f64 {
        self.cpu_pressure.max(self.memory_pressure).max(self.io_pressure)
    }

    /// Check if any pressure exceeds threshold
    pub fn exceeds_threshold(&self, threshold: f64) -> bool {
        self.get_max_pressure() > threshold
    }

    /// Reset all metrics to zero
    pub fn reset(&mut self) {
        self.cpu_pressure = 0.0;
        self.memory_pressure = 0.0;
        self.io_pressure = 0.0;
        self.last_updated = Instant::now();
    }

    /// Get age of metrics
    pub fn get_age(&self) -> Duration {
        Instant::now().duration_since(self.last_updated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_psi_scheduler_creation() {
        let psi = PSIScheduler::new();
        assert_eq!(psi.get_current_severity(), PSISeverity::None);
        assert!(!psi.is_under_pressure());
    }

    #[test]
    fn test_severity_calculation() {
        let psi = PSIScheduler::new();
        assert_eq!(psi.calculate_severity(5.0), PSISeverity::None);
        assert_eq!(psi.calculate_severity(15.0), PSISeverity::Low);
        assert_eq!(psi.calculate_severity(45.0), PSISeverity::Medium);
        assert_eq!(psi.calculate_severity(75.0), PSISeverity::High);
        assert_eq!(psi.calculate_severity(95.0), PSISeverity::Critical);
    }

    #[test]
    fn test_scheduling_hints() {
        let psi = PSIScheduler::new();
        // Test would require mocking pressure values
        assert_eq!(psi.get_scheduling_hint(), SchedulingHint::Normal);
    }
}