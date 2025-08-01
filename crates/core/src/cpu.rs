//! CPU binding utilities for high-performance trading
//! 
//! High-performance architecture for CPU core binding to achieve
//! maximum single-core performance for trading threads.

use tracing::{info, warn};

/// Bind current thread to specific CPU core(s)
/// 
/// This is high-performance functionality.
/// Binding to a dedicated CPU core reduces context switching and 
/// improves latency consistency for trading workloads.
pub fn bind_to_cpu_set(cpu_core: usize) -> Result<(), String> {
    #[cfg(feature = "cpu-binding")]
    {
        
        // Get available CPU cores
        let core_ids = core_affinity::get_core_ids()
            .ok_or_else(|| "Failed to get CPU core IDs".to_string())?;
        
        if cpu_core >= core_ids.len() {
            return Err(format!(
                "CPU core {} not available (max: {})", 
                cpu_core, 
                core_ids.len() - 1
            ));
        }
        
        let core_id = core_ids[cpu_core];
        
        if core_affinity::set_for_current(core_id) {
            info!("✅ Successfully bound to CPU core {}", cpu_core);
            Ok(())
        } else {
            Err(format!("Failed to bind to CPU core {cpu_core}"))
        }
    }
    
    #[cfg(not(feature = "cpu-binding"))]
    {
        warn!("CPU binding disabled (compile with --features cpu-binding)");
        Ok(())
    }
}

/// Get current CPU core binding
pub fn get_current_cpu() -> Option<usize> {
    #[cfg(feature = "cpu-binding")]
    {
        // This is platform-specific and would need proper implementation
        // For now, return None
        None
    }
    
    #[cfg(not(feature = "cpu-binding"))]
    {
        None
    }
}

/// Get number of available CPU cores
pub fn get_cpu_count() -> usize {
    #[cfg(feature = "cpu-binding")]
    {
        core_affinity::get_core_ids()
            .map(|cores| cores.len())
            .unwrap_or(1)
    }
    
    #[cfg(not(feature = "cpu-binding"))]
    {
        num_cpus::get()
    }
}

/// Set CPU governor to performance mode (Linux only)
/// 
/// This helps achieve consistent latency by preventing CPU frequency scaling.
/// Requires root privileges or appropriate permissions.
pub fn set_performance_governor() -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        use std::path::Path;
        
        let cpu_count = get_cpu_count();
        let mut errors = Vec::new();
        
        for i in 0..cpu_count {
            let governor_path = format!("/sys/devices/system/cpu/cpu{i}/cpufreq/scaling_governor");
            
            if Path::new(&governor_path).exists() {
                if let Err(e) = fs::write(&governor_path, "performance") {
                    errors.push(format!("CPU {i}: {e}"));
                }
            }
        }
        
        if errors.is_empty() {
            info!("🚀 Set CPU governor to performance mode for {} cores", cpu_count);
            Ok(())
        } else {
            warn!("Failed to set performance governor for some CPUs: {:?}", errors);
            Err(format!("Errors setting governor: {errors:?}"))
        }
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        warn!("CPU governor setting only supported on Linux");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_cpu_count() {
        let count = get_cpu_count();
        assert!(count > 0);
        assert!(count <= 256); // Reasonable upper bound
    }
    
    #[test]
    fn test_bind_to_invalid_cpu() {
        let cpu_count = get_cpu_count();
        let result = bind_to_cpu_set(cpu_count + 10);
        assert!(result.is_err());
    }
}