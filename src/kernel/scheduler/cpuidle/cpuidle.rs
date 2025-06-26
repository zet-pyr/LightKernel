//! # CPU Idle States Management Module
//! 
//! This module provides a high-level interface for managing CPU idle states
//! in the kernel scheduler. It handles CPU power management by controlling
//! different idle states to optimize power consumption and performance.
//!
//! ## Features
//! - Initialize and configure CPU idle state management
//! - Query current and available idle states
//! - Set specific idle states with validation
//! - Restore default configurations
//! - Runtime support detection
//!
//! ## Usage
//! ```rust
//! use crate::kernel::scheduler::cpuidle;
//! 
//! // Initialize the CPU idle management
//! cpuidle::init()?;
//! 
//! // Check if idle state management is supported
//! if cpuidle::is_supported() {
//!     // Get available states
//!     let states = cpuidle::get_available_idle_states()?;
//!     
//!     // Set a specific idle state
//!     cpuidle::set_idle_state(states[0])?;
//! }
//! ```

use crate::kernel::scheduler::cpuidle::cpuidle_impl::{
    CpuIdle, CpuIdleImpl, CpuIdleImplTrait, CpuIdleImplError, 
    CpuIdleImplResult, CpuIdleImplConfig
};
use crate::kernel::log::{kernel_info, kernel_warn, kernel_error};
use core::sync::atomic::{AtomicBool, Ordering};

pub mod cpuidle_impl;

/// Global flag to track initialization status
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// CPU idle state validation limits
const MIN_IDLE_STATE: u64 = 0;
const MAX_IDLE_STATE: u64 = 7; // Typical maximum for most architectures

/// Initializes the CPU idle states management module with enhanced error handling
/// 
/// This function sets up the CPU idle state management system with default
/// configuration. It should be called once during kernel initialization.
///
/// # Returns
/// - `Ok(())` if initialization succeeds
/// - `Err(CpuIdleImplError)` if initialization fails
///
/// # Examples
/// ```rust
/// match cpuidle::init() {
///     Ok(()) => kernel_info!("CPU idle management initialized successfully"),
///     Err(e) => kernel_error!("Failed to initialize CPU idle management: {:?}", e),
/// }
/// ```
pub fn init() -> CpuIdleImplResult<()> {
    if INITIALIZED.load(Ordering::Acquire) {
        kernel_warn!("CPU idle management already initialized");
        return Ok(());
    }

    kernel_info!("Initializing CPU idle states management...");
    
    let config = CpuIdleImplConfig::default();
    let cpuidle_impl = CpuIdleImpl::new(config)
        .map_err(|e| {
            kernel_error!("Failed to create CPU idle implementation: {:?}", e);
            e
        })?;
    
    CpuIdle::set_impl(cpuidle_impl);
    INITIALIZED.store(true, Ordering::Release);
    
    kernel_info!("CPU idle states management initialized successfully");
    Ok(())
}

/// Returns the current CPU idle state
///
/// # Returns
/// - `Ok(state)` with the current idle state ID
/// - `Err(CpuIdleImplError)` if the operation fails or module not initialized
///
/// # Examples
/// ```rust
/// match cpuidle::get_current_idle_state() {
///     Ok(state) => println!("Current idle state: {}", state),
///     Err(e) => eprintln!("Error getting idle state: {:?}", e),
/// }
/// ```
pub fn get_current_idle_state() -> CpuIdleImplResult<u64> {
    ensure_initialized()?;
    
    CpuIdle::get_impl().get_current_idle_state()
        .map_err(|e| {
            kernel_warn!("Failed to get current idle state: {:?}", e);
            e
        })
}

/// Sets the CPU idle state with validation
///
/// # Arguments
/// * `state` - The idle state ID to set
///
/// # Returns
/// - `Ok(())` if the state was set successfully
/// - `Err(CpuIdleImplError)` if the operation fails or state is invalid
///
/// # Examples
/// ```rust
/// if let Err(e) = cpuidle::set_idle_state(2) {
///     kernel_error!("Failed to set idle state: {:?}", e);
/// }
/// ```
pub fn set_idle_state(state: u64) -> CpuIdleImplResult<()> {
    ensure_initialized()?;
    
    // Validate state range
    if state < MIN_IDLE_STATE || state > MAX_IDLE_STATE {
        kernel_warn!("Invalid idle state {} (valid range: {}-{})", 
                    state, MIN_IDLE_STATE, MAX_IDLE_STATE);
        return Err(CpuIdleImplError::InvalidState);
    }
    
    // Check if state is available
    let available_states = get_available_idle_states()?;
    if !available_states.contains(&state) {
        kernel_warn!("Idle state {} is not available on this system", state);
        return Err(CpuIdleImplError::UnsupportedState);
    }
    
    CpuIdle::get_impl().set_idle_state(state)
        .map_err(|e| {
            kernel_error!("Failed to set idle state {}: {:?}", state, e);
            e
        })?;
    
    kernel_info!("CPU idle state set to: {}", state);
    Ok(())
}

/// Returns the list of available CPU idle states
///
/// # Returns
/// - `Ok(Vec<u64>)` containing available idle state IDs
/// - `Err(CpuIdleImplError)` if the operation fails
///
/// # Examples
/// ```rust
/// match cpuidle::get_available_idle_states() {
///     Ok(states) => {
///         println!("Available idle states: {:?}", states);
///         for state in states {
///             println!("  State {}: {}", state, cpuidle::get_idle_state_name(state)?);
///         }
///     },
///     Err(e) => eprintln!("Error: {:?}", e),
/// }
/// ```
pub fn get_available_idle_states() -> CpuIdleImplResult<Vec<u64>> {
    ensure_initialized()?;
    
    CpuIdle::get_impl().get_available_idle_states()
        .map_err(|e| {
            kernel_warn!("Failed to get available idle states: {:?}", e);
            e
        })
}

/// Restores the default CPU idle state
///
/// This function resets the CPU idle state to the system default,
/// which is typically the most balanced state for power and performance.
///
/// # Returns
/// - `Ok(())` if the default state was restored successfully
/// - `Err(CpuIdleImplError)` if the operation fails
///
/// # Examples
/// ```rust
/// if let Err(e) = cpuidle::restore_default_idle_state() {
///     kernel_error!("Failed to restore default idle state: {:?}", e);
/// }
/// ```
pub fn restore_default_idle_state() -> CpuIdleImplResult<()> {
    ensure_initialized()?;
    
    let default_state = CpuIdle::get_impl().get_default_idle_state()
        .map_err(|e| {
            kernel_error!("Failed to get default idle state: {:?}", e);
            e
        })?;
    
    CpuIdle::get_impl().set_idle_state(default_state)
        .map_err(|e| {
            kernel_error!("Failed to restore default idle state {}: {:?}", default_state, e);
            e
        })?;
    
    kernel_info!("CPU idle state restored to default: {}", default_state);
    Ok(())
}

/// Checks if CPU idle state management is supported on this system
///
/// # Returns
/// - `true` if idle state management is supported and initialized
/// - `false` if not supported or not initialized
///
/// # Examples
/// ```rust
/// if cpuidle::is_supported() {
///     println!("CPU idle state management is available");
/// } else {
///     println!("CPU idle state management is not supported");
/// }
/// ```
pub fn is_supported() -> bool {
    if !INITIALIZED.load(Ordering::Acquire) {
        return false;
    }
    
    CpuIdle::get_impl().is_supported().unwrap_or_else(|e| {
        kernel_warn!("Error checking idle state support: {:?}", e);
        false
    })
}

/// Returns the name/description of an idle state
///
/// # Arguments
/// * `state` - The idle state ID
///
/// # Returns
/// - `Ok(String)` with the state description
/// - `Err(CpuIdleImplError)` if the state is invalid or operation fails
pub fn get_idle_state_name(state: u64) -> CpuIdleImplResult<String> {
    ensure_initialized()?;
    
    CpuIdle::get_impl().get_idle_state_name(state)
        .map_err(|e| {
            kernel_warn!("Failed to get name for idle state {}: {:?}", state, e);
            e
        })
}

/// Gets detailed statistics about CPU idle state usage
///
/// # Returns
/// - `Ok(CpuIdleStats)` with usage statistics
/// - `Err(CpuIdleImplError)` if the operation fails
pub fn get_idle_statistics() -> CpuIdleImplResult<CpuIdleStats> {
    ensure_initialized()?;
    
    CpuIdle::get_impl().get_statistics()
        .map_err(|e| {
            kernel_warn!("Failed to get idle statistics: {:?}", e);
            e
        })
}

/// Resets CPU idle state statistics counters
///
/// # Returns
/// - `Ok(())` if statistics were reset successfully
/// - `Err(CpuIdleImplError)` if the operation fails
pub fn reset_idle_statistics() -> CpuIdleImplResult<()> {
    ensure_initialized()?;
    
    CpuIdle::get_impl().reset_statistics()
        .map_err(|e| {
            kernel_error!("Failed to reset idle statistics: {:?}", e);
            e
        })?;
    
    kernel_info!("CPU idle statistics reset");
    Ok(())
}

/// Performs a safe shutdown of the CPU idle management system
///
/// This should be called during kernel shutdown to ensure proper cleanup.
///
/// # Returns
/// - `Ok(())` if shutdown was successful
/// - `Err(CpuIdleImplError)` if cleanup failed
pub fn shutdown() -> CpuIdleImplResult<()> {
    if !INITIALIZED.load(Ordering::Acquire) {
        return Ok(());
    }
    
    kernel_info!("Shutting down CPU idle states management...");
    
    // Restore default state before shutdown
    if let Err(e) = restore_default_idle_state() {
        kernel_warn!("Failed to restore default state during shutdown: {:?}", e);
    }
    
    CpuIdle::get_impl().shutdown()
        .map_err(|e| {
            kernel_error!("Failed to shutdown CPU idle management: {:?}", e);
            e
        })?;
    
    INITIALIZED.store(false, Ordering::Release);
    kernel_info!("CPU idle states management shutdown complete");
    Ok(())
}

/// Ensures the module is initialized before performing operations
///
/// # Returns
/// - `Ok(())` if initialized
/// - `Err(CpuIdleImplError::NotInitialized)` if not initialized
#[inline]
fn ensure_initialized() -> CpuIdleImplResult<()> {
    if !INITIALIZED.load(Ordering::Acquire) {
        kernel_error!("CPU idle management not initialized");
        return Err(CpuIdleImplError::NotInitialized);
    }
    Ok(())
}

/// CPU idle state usage statistics
#[derive(Debug, Clone)]
pub struct CpuIdleStats {
    /// Total time spent in each idle state (in microseconds)
    pub state_usage_time: Vec<(u64, u64)>, // (state_id, time_us)
    /// Number of entries into each idle state
    pub state_entry_count: Vec<(u64, u64)>, // (state_id, count)
    /// Average time spent per entry for each state
    pub average_residency: Vec<(u64, u64)>, // (state_id, avg_time_us)
    /// Current active state
    pub current_state: u64,
    /// Total idle time across all states
    pub total_idle_time: u64,
}