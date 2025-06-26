//! # CPU Frequency Management Module
//! 
//! This module provides comprehensive CPU frequency scaling (CPU frequency management)
//! functionality for the kernel scheduler. It enables dynamic voltage and frequency
//! scaling (DVFS) to optimize power consumption and performance based on workload demands.
//!
//! ## Features
//! - Dynamic frequency scaling with validation
//! - Governor-based frequency management policies
//! - Thermal throttling protection
//! - Performance monitoring and statistics
//! - Safe frequency transitions with hardware limits
//! - Multi-core frequency coordination
//!
//! ## Supported Governors
//! - **Performance**: Maximum frequency for high performance
//! - **Powersave**: Minimum frequency for power efficiency
//! - **Ondemand**: Dynamic scaling based on CPU load
//! - **Conservative**: Gradual frequency adjustments
//! - **Userspace**: Manual frequency control
//!
//! ## Usage
//! ```rust
//! use crate::kernel::scheduler::cpufreq;
//! 
//! // Initialize CPU frequency management
//! cpufreq::init()?;
//! 
//! // Set performance governor for high-performance tasks
//! cpufreq::set_governor(Governor::Performance)?;
//! 
//! // Get current frequency
//! let freq = cpufreq::get_current_frequency()?;
//! println!("Current CPU frequency: {} MHz", freq);
//! 
//! // Set specific frequency (if supported by current governor)
//! cpufreq::set_frequency(2400000)?; // 2.4 GHz
//! ```

use crate::kernel::scheduler::cpufreq::cpufreq_impl::{
    CpuFreq, CpuFreqImpl, CpuFreqImplTrait, CpuFreqImplError, 
    CpuFreqImplResult, CpuFreqImplConfig
};
use crate::kernel::log::{kernel_info, kernel_warn, kernel_error, kernel_debug};
use crate::kernel::time::get_current_time_us;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use alloc::vec::Vec;
use alloc::string::String;

pub mod cpufreq_impl;

/// Global initialization flag
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Last frequency change timestamp for rate limiting
static LAST_FREQ_CHANGE: AtomicU64 = AtomicU64::new(0);

/// Frequency validation limits (in Hz)
const MIN_SAFE_FREQUENCY: u64 = 400_000_000;  // 400 MHz
const MAX_SAFE_FREQUENCY: u64 = 5_000_000_000; // 5 GHz
const FREQ_CHANGE_MIN_INTERVAL_US: u64 = 10_000; // 10ms minimum between changes

/// Thermal throttling thresholds
const THERMAL_THROTTLE_TEMP: u64 = 85; // 85°C
const THERMAL_CRITICAL_TEMP: u64 = 95; // 95°C

/// CPU frequency governors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Governor {
    /// Maximum performance - highest available frequency
    Performance,
    /// Power saving - lowest available frequency
    Powersave,
    /// Dynamic scaling based on CPU utilization
    Ondemand,
    /// Conservative scaling with gradual adjustments
    Conservative,
    /// Manual frequency control
    Userspace,
}

impl Governor {
    /// Returns the governor name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Governor::Performance => "performance",
            Governor::Powersave => "powersave",
            Governor::Ondemand => "ondemand",
            Governor::Conservative => "conservative",
            Governor::Userspace => "userspace",
        }
    }
}

/// CPU frequency statistics and monitoring data
#[derive(Debug, Clone)]
pub struct CpuFreqStats {
    /// Current frequency in Hz
    pub current_frequency: u64,
    /// Minimum frequency in Hz
    pub min_frequency: u64,
    /// Maximum frequency in Hz
    pub max_frequency: u64,
    /// Average frequency over last measurement period
    pub average_frequency: u64,
    /// Current governor
    pub current_governor: Governor,
    /// Number of frequency transitions
    pub transition_count: u64,
    /// Time spent at each frequency level (frequency_hz, time_us)
    pub frequency_time: Vec<(u64, u64)>,
    /// Current CPU temperature (if available)
    pub temperature: Option<u64>,
    /// Thermal throttling status
    pub thermal_throttled: bool,
    /// Power consumption estimate (in mW, if available)
    pub power_consumption: Option<u64>,
}

/// Thermal throttling information
#[derive(Debug, Clone)]
pub struct ThermalInfo {
    /// Current temperature in Celsius
    pub temperature: u64,
    /// Is thermal throttling active
    pub throttled: bool,
    /// Throttled frequency limit
    pub throttle_frequency: Option<u64>,
    /// Time spent throttled (in microseconds)
    pub throttle_time: u64,
}

/// Initializes the CPU frequency management module with enhanced configuration
/// 
/// Sets up the CPU frequency scaling system with comprehensive error handling,
/// thermal monitoring, and performance optimization features.
///
/// # Arguments
/// * `config` - Optional custom configuration, uses default if None
///
/// # Returns
/// - `Ok(())` if initialization succeeds
/// - `Err(CpuFreqImplError)` if initialization fails
///
/// # Examples
/// ```rust
/// // Initialize with default configuration
/// cpufreq::init()?;
/// 
/// // Initialize with custom configuration
/// let config = CpuFreqImplConfig {
///     enable_thermal_management: true,
///     default_governor: Governor::Ondemand,
///     ..Default::default()
/// };
/// cpufreq::init_with_config(config)?;
/// ```
pub fn init() -> CpuFreqImplResult<()> {
    init_with_config(CpuFreqImplConfig::default())
}

/// Initializes with custom configuration
pub fn init_with_config(config: CpuFreqImplConfig) -> CpuFreqImplResult<()> {
    if INITIALIZED.load(Ordering::Acquire) {
        kernel_warn!("CPU frequency management already initialized");
        return Ok(());
    }

    kernel_info!("Initializing CPU frequency management...");
    
    let cpufreq_impl = CpuFreqImpl::new(config)
        .map_err(|e| {
            kernel_error!("Failed to create CPU frequency implementation: {:?}", e);
            e
        })?;
    
    CpuFreq::set_impl(cpufreq_impl);
    INITIALIZED.store(true, Ordering::Release);
    
    // Log initialization details
    if let Ok(freqs) = get_available_frequencies() {
        kernel_info!("Available frequencies: {:?} MHz", 
                    freqs.iter().map(|f| f / 1_000_000).collect::<Vec<_>>());
    }
    
    if let Ok(current) = get_current_frequency() {
        kernel_info!("Current frequency: {} MHz", current / 1_000_000);
    }
    
    kernel_info!("CPU frequency management initialized successfully");
    Ok(())
}

/// Returns the current CPU frequency with enhanced error handling
///
/// # Returns
/// - `Ok(frequency)` with the current frequency in Hz
/// - `Err(CpuFreqImplError)` if the operation fails
///
/// # Examples
/// ```rust
/// match cpufreq::get_current_frequency() {
///     Ok(freq) => println!("Current frequency: {} GHz", freq as f64 / 1e9),
///     Err(e) => eprintln!("Error getting frequency: {:?}", e),
/// }
/// ```
pub fn get_current_frequency() -> CpuFreqImplResult<u64> {
    ensure_initialized()?;
    
    CpuFreq::get_impl().get_current_frequency()
        .map_err(|e| {
            kernel_warn!("Failed to get current frequency: {:?}", e);
            e
        })
}

/// Sets the CPU frequency with comprehensive validation and safety checks
///
/// # Arguments
/// * `frequency` - Target frequency in Hz
///
/// # Returns
/// - `Ok(())` if the frequency was set successfully
/// - `Err(CpuFreqImplError)` if the operation fails or frequency is invalid
///
/// # Safety
/// - Validates frequency is within safe operating limits
/// - Checks thermal conditions before frequency changes
/// - Enforces minimum time between frequency changes
/// - Verifies frequency is available on the hardware
///
/// # Examples
/// ```rust
/// // Set to 2.4 GHz
/// cpufreq::set_frequency(2_400_000_000)?;
/// 
/// // Set to maximum available frequency
/// let max_freq = cpufreq::get_max_frequency()?;
/// cpufreq::set_frequency(max_freq)?;
/// ```
pub fn set_frequency(frequency: u64) -> CpuFreqImplResult<()> {
    ensure_initialized()?;
    
    // Rate limiting check
    let current_time = get_current_time_us();
    let last_change = LAST_FREQ_CHANGE.load(Ordering::Acquire);
    if current_time - last_change < FREQ_CHANGE_MIN_INTERVAL_US {
        kernel_debug!("Frequency change rate limited");
        return Err(CpuFreqImplError::RateLimited);
    }
    
    // Validate frequency range
    if frequency < MIN_SAFE_FREQUENCY || frequency > MAX_SAFE_FREQUENCY {
        kernel_warn!("Frequency {} Hz outside safe range ({}-{} Hz)", 
                    frequency, MIN_SAFE_FREQUENCY, MAX_SAFE_FREQUENCY);
        return Err(CpuFreqImplError::InvalidFrequency);
    }
    
    // Check if frequency is available
    let available_freqs = get_available_frequencies()?;
    if !available_freqs.contains(&frequency) {
        kernel_warn!("Frequency {} Hz not available on this system", frequency);
        return Err(CpuFreqImplError::UnsupportedFrequency);
    }
    
    // Thermal protection check
    if let Ok(thermal_info) = get_thermal_info() {
        if thermal_info.temperature > THERMAL_CRITICAL_TEMP {
            kernel_error!("CPU temperature too high ({} °C), rejecting frequency increase", 
                         thermal_info.temperature);
            return Err(CpuFreqImplError::ThermalThrottled);
        }
        
        if thermal_info.temperature > THERMAL_THROTTLE_TEMP {
            let current_freq = get_current_frequency()?;
            if frequency > current_freq {
                kernel_warn!("CPU temperature high ({} °C), limiting frequency increase", 
                           thermal_info.temperature);
                // Allow only conservative increases
                let max_allowed = current_freq + (current_freq / 10); // 10% increase max
                if frequency > max_allowed {
                    return Err(CpuFreqImplError::ThermalThrottled);
                }
            }
        }
    }
    
    // Perform the frequency change
    CpuFreq::get_impl().set_frequency(frequency)
        .map_err(|e| {
            kernel_error!("Failed to set frequency to {} Hz: {:?}", frequency, e);
            e
        })?;
    
    LAST_FREQ_CHANGE.store(current_time, Ordering::Release);
    kernel_info!("CPU frequency set to {} MHz", frequency / 1_000_000);
    Ok(())
}

/// Returns the list of available CPU frequencies
///
/// # Returns
/// - `Ok(Vec<u64>)` containing available frequencies in Hz
/// - `Err(CpuFreqImplError)` if the operation fails
///
/// # Examples
/// ```rust
/// let frequencies = cpufreq::get_available_frequencies()?;
/// for freq in frequencies {
///     println!("Available: {} GHz", freq as f64 / 1e9);
/// }
/// ```
pub fn get_available_frequencies() -> CpuFreqImplResult<Vec<u64>> {
    ensure_initialized()?;
    
    CpuFreq::get_impl().get_available_frequencies()
        .map_err(|e| {
            kernel_warn!("Failed to get available frequencies: {:?}", e);
            e
        })
}

/// Gets the minimum available frequency
///
/// # Returns
/// - `Ok(frequency)` with the minimum frequency in Hz
/// - `Err(CpuFreqImplError)` if the operation fails
pub fn get_min_frequency() -> CpuFreqImplResult<u64> {
    let frequencies = get_available_frequencies()?;
    frequencies.iter().min().copied()
        .ok_or(CpuFreqImplError::NoFrequenciesAvailable)
}

/// Gets the maximum available frequency
///
/// # Returns
/// - `Ok(frequency)` with the maximum frequency in Hz
/// - `Err(CpuFreqImplError)` if the operation fails
pub fn get_max_frequency() -> CpuFreqImplResult<u64> {
    let frequencies = get_available_frequencies()?;
    frequencies.iter().max().copied()
        .ok_or(CpuFreqImplError::NoFrequenciesAvailable)
}

/// Restores the default CPU frequency with enhanced safety
///
/// # Returns
/// - `Ok(())` if the default frequency was restored successfully
/// - `Err(CpuFreqImplError)` if the operation fails
///
/// # Examples
/// ```rust
/// // Restore to safe default after high-performance task
/// cpufreq::restore_default_frequency()?;
/// ```
pub fn restore_default_frequency() -> CpuFreqImplResult<()> {
    ensure_initialized()?;
    
    let default_freq = CpuFreq::get_impl().get_default_frequency()
        .map_err(|e| {
            kernel_error!("Failed to get default frequency: {:?}", e);
            e
        })?;
    
    set_frequency(default_freq)?;
    kernel_info!("CPU frequency restored to default: {} MHz", default_freq / 1_000_000);
    Ok(())
}

/// Sets the CPU frequency governor
///
/// # Arguments
/// * `governor` - The governor to set
///
/// # Returns
/// - `Ok(())` if the governor was set successfully
/// - `Err(CpuFreqImplError)` if the operation fails
///
/// # Examples
/// ```rust
/// // Set performance governor for demanding tasks
/// cpufreq::set_governor(Governor::Performance)?;
/// 
/// // Set powersave governor for battery optimization
/// cpufreq::set_governor(Governor::Powersave)?;
/// ```
pub fn set_governor(governor: Governor) -> CpuFreqImplResult<()> {
    ensure_initialized()?;
    
    CpuFreq::get_impl().set_governor(governor)
        .map_err(|e| {
            kernel_error!("Failed to set governor to {}: {:?}", governor.as_str(), e);
            e
        })?;
    
    kernel_info!("CPU frequency governor set to: {}", governor.as_str());
    Ok(())
}

/// Gets the current CPU frequency governor
///
/// # Returns
/// - `Ok(Governor)` with the current governor
/// - `Err(CpuFreqImplError)` if the operation fails
pub fn get_current_governor() -> CpuFreqImplResult<Governor> {
    ensure_initialized()?;
    
    CpuFreq::get_impl().get_current_governor()
        .map_err(|e| {
            kernel_warn!("Failed to get current governor: {:?}", e);
            e
        })
}

/// Gets comprehensive CPU frequency statistics
///
/// # Returns
/// - `Ok(CpuFreqStats)` with detailed statistics
/// - `Err(CpuFreqImplError)` if the operation fails
///
/// # Examples
/// ```rust
/// let stats = cpufreq::get_frequency_stats()?;
/// println!("Current: {} MHz, Average: {} MHz", 
///          stats.current_frequency / 1_000_000,
///          stats.average_frequency / 1_000_000);
/// ```
pub fn get_frequency_stats() -> CpuFreqImplResult<CpuFreqStats> {
    ensure_initialized()?;
    
    CpuFreq::get_impl().get_frequency_stats()
        .map_err(|e| {
            kernel_warn!("Failed to get frequency statistics: {:?}", e);
            e
        })
}

/// Gets thermal information and throttling status
///
/// # Returns
/// - `Ok(ThermalInfo)` with thermal data
/// - `Err(CpuFreqImplError)` if thermal monitoring is not available
pub fn get_thermal_info() -> CpuFreqImplResult<ThermalInfo> {
    ensure_initialized()?;
    
    CpuFreq::get_impl().get_thermal_info()
        .map_err(|e| {
            kernel_debug!("Failed to get thermal info: {:?}", e);
            e
        })
}

/// Resets frequency statistics counters
///
/// # Returns
/// - `Ok(())` if statistics were reset successfully
/// - `Err(CpuFreqImplError)` if the operation fails
pub fn reset_frequency_stats() -> CpuFreqImplResult<()> {
    ensure_initialized()?;
    
    CpuFreq::get_impl().reset_frequency_stats()
        .map_err(|e| {
            kernel_error!("Failed to reset frequency statistics: {:?}", e);
            e
        })?;
    
    kernel_info!("CPU frequency statistics reset");
    Ok(())
}

/// Performs intelligent frequency scaling based on current load
///
/// # Arguments
/// * `cpu_load` - Current CPU load percentage (0-100)
/// * `target_latency` - Target response latency in microseconds
///
/// # Returns
/// - `Ok(new_frequency)` with the selected frequency
/// - `Err(CpuFreqImplError)` if scaling fails
pub fn scale_frequency_intelligent(cpu_load: u32, target_latency: u64) -> CpuFreqImplResult<u64> {
    ensure_initialized()?;
    
    if cpu_load > 100 {
        return Err(CpuFreqImplError::InvalidParameter);
    }
    
    let available_freqs = get_available_frequencies()?;
    let current_freq = get_current_frequency()?;
    
    // Intelligent scaling algorithm
    let target_freq = if cpu_load > 80 {
        // High load: scale to maximum
        *available_freqs.iter().max().unwrap()
    } else if cpu_load < 20 {
        // Low load: scale down for power saving
        *available_freqs.iter().min().unwrap()
    } else {
        // Medium load: proportional scaling
        let min_freq = *available_freqs.iter().min().unwrap();
        let max_freq = *available_freqs.iter().max().unwrap();
        let scale_factor = cpu_load as f64 / 100.0;
        let target = min_freq as f64 + (max_freq - min_freq) as f64 * scale_factor;
        
        // Find closest available frequency
        available_freqs.iter()
            .min_by_key(|&&freq| ((freq as f64 - target).abs() as u64))
            .copied().unwrap()
    };
    
    // Apply latency constraints
    let latency_adjusted_freq = if target_latency < 1000 { // < 1ms
        // Very low latency required, prefer higher frequencies
        available_freqs.iter()
            .filter(|&&f| f >= target_freq)
            .min().copied()
            .unwrap_or(target_freq)
    } else {
        target_freq
    };
    
    if latency_adjusted_freq != current_freq {
        set_frequency(latency_adjusted_freq)?;
    }
    
    Ok(latency_adjusted_freq)
}

/// Checks if CPU frequency management is supported on this system
///
/// # Returns
/// - `true` if frequency management is supported and initialized
/// - `false` if not supported or not initialized
///
/// # Examples
/// ```rust
/// if cpufreq::is_supported() {
///     println!("CPU frequency scaling is available");
/// } else {
///     println!("CPU frequency scaling is not supported");
/// }
/// ```
pub fn is_supported() -> bool {
    if !INITIALIZED.load(Ordering::Acquire) {
        return false;
    }
    
    CpuFreq::get_impl().is_supported().unwrap_or_else(|e| {
        kernel_warn!("Error checking frequency scaling support: {:?}", e);
        false
    })
}

/// Performs safe shutdown of CPU frequency management
///
/// # Returns
/// - `Ok(())` if shutdown was successful
/// - `Err(CpuFreqImplError)` if cleanup failed
pub fn shutdown() -> CpuFreqImplResult<()> {
    if !INITIALIZED.load(Ordering::Acquire) {
        return Ok(());
    }
    
    kernel_info!("Shutting down CPU frequency management...");
    
    // Restore safe default frequency
    if let Err(e) = restore_default_frequency() {
        kernel_warn!("Failed to restore default frequency during shutdown: {:?}", e);
    }
    
    // Reset to conservative governor
    if let Err(e) = set_governor(Governor::Conservative) {
        kernel_warn!("Failed to set conservative governor during shutdown: {:?}", e);
    }
    
    CpuFreq::get_impl().shutdown()
        .map_err(|e| {
            kernel_error!("Failed to shutdown CPU frequency management: {:?}", e);
            e
        })?;
    
    INITIALIZED.store(false, Ordering::Release);
    kernel_info!("CPU frequency management shutdown complete");
    Ok(())
}

/// Ensures the module is initialized before performing operations
#[inline]
fn ensure_initialized() -> CpuFreqImplResult<()> {
    if !INITIALIZED.load(Ordering::Acquire) {
        kernel_error!("CPU frequency management not initialized");
        return Err(CpuFreqImplError::NotInitialized);
    }
    Ok(())
}

/// Convenience function to set performance mode
///
/// # Examples
/// ```rust
/// // Enable high performance mode
/// cpufreq::set_performance_mode()?;
/// ```
pub fn set_performance_mode() -> CpuFreqImplResult<()> {
    set_governor(Governor::Performance)?;
    let max_freq = get_max_frequency()?;
    set_frequency(max_freq)?;
    kernel_info!("Performance mode enabled");
    Ok(())
}

/// Convenience function to set power saving mode
///
/// # Examples
/// ```rust
/// // Enable power saving mode
/// cpufreq::set_powersave_mode()?;
/// ```
pub fn set_powersave_mode() -> CpuFreqImplResult<()> {
    set_governor(Governor::Powersave)?;
    let min_freq = get_min_frequency()?;
    set_frequency(min_freq)?;
    kernel_info!("Power saving mode enabled");
    Ok(())
}

/// Convenience function to set balanced mode
///
/// # Examples
/// ```rust
/// // Enable balanced mode
/// cpufreq::set_balanced_mode()?;
/// ```
pub fn set_balanced_mode() -> CpuFreqImplResult<()> {
    set_governor(Governor::Ondemand)?;
    restore_default_frequency()?;
    kernel_info!("Balanced mode enabled");
    Ok(())
}