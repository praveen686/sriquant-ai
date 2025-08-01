//! High-performance async runtime based on monoio
//! 
//! High-performance architecture:
//! - Single-threaded async for maximum performance
//! - CPU binding for dedicated cores
//! - Optimized for trading workloads

use monoio::{RuntimeBuilder, IoUringDriver};
use tracing::{info, warn};
use crate::cpu::bind_to_cpu_set;

/// High-performance trading runtime configuration
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// CPU core to bind to (None for no binding)
    pub cpu_core: Option<usize>,
    /// Thread name
    pub thread_name: String,
    /// Enable timing optimizations
    pub enable_timing: bool,
    /// Runtime thread stack size
    pub stack_size: Option<usize>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            cpu_core: Some(0), // Bind to first core by default
            thread_name: "sriquant-main".to_string(),
            enable_timing: true,
            stack_size: Some(2 * 1024 * 1024), // 2MB stack
        }
    }
}

/// SriQuant.ai high-performance runtime
/// 
/// Based on monoio for single-threaded async performance,
/// following high-performance principles.
pub struct SriQuantRuntime {
    config: RuntimeConfig,
}

impl SriQuantRuntime {
    /// Create a new SriQuant runtime with default configuration
    pub fn new() -> Self {
        Self::with_config(RuntimeConfig::default())
    }
    
    /// Create a new SriQuant runtime with custom configuration
    pub fn with_config(config: RuntimeConfig) -> Self {
        // Bind to CPU core if specified ()
        if let Some(cpu_core) = config.cpu_core {
            if let Err(e) = bind_to_cpu_set(cpu_core) {
                warn!("Failed to bind to CPU core {}: {}", cpu_core, e);
            } else {
                info!("🔗 Bound to CPU core {}", cpu_core);
            }
        }
        
        info!("🚀 SriQuant runtime initialized");
        info!("   Thread: {}", config.thread_name);
        info!("   CPU Core: {:?}", config.cpu_core);
        info!("   Timing: {}", config.enable_timing);
        
        Self { config }
    }
    
    /// Run a future on the SriQuant runtime
    pub fn block_on<F>(&mut self, future: F) -> F::Output
    where
        F: std::future::Future,
    {
        let mut runtime = RuntimeBuilder::<IoUringDriver>::new().build().expect("Failed to create runtime");
        runtime.block_on(future)
    }
    
    /// Start the runtime and run until completion
    pub fn start<F, Fut>(self, f: F) -> Fut::Output
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future,
    {
        info!("▶️  Starting SriQuant runtime");
        let mut runtime = RuntimeBuilder::<IoUringDriver>::new().build().expect("Failed to create runtime");
        let result = runtime.block_on(f());
        info!("⏹️  SriQuant runtime stopped");
        result
    }
    
    /// Get runtime configuration
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }
}

impl Default for SriQuantRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to create and run a SriQuant runtime
pub fn run_sriquant<F, Fut>(f: F) -> Fut::Output
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future,
{
    let runtime = SriQuantRuntime::new();
    runtime.start(f)
}

/// Convenience function to run with specific CPU binding
pub fn run_sriquant_on_cpu<F, Fut>(cpu_core: usize, f: F) -> Fut::Output
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future,
{
    let config = RuntimeConfig {
        cpu_core: Some(cpu_core),
        ..Default::default()
    };
    let runtime = SriQuantRuntime::with_config(config);
    runtime.start(f)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_runtime_creation() {
        let runtime = SriQuantRuntime::new();
        assert_eq!(runtime.config().thread_name, "sriquant-main");
        assert_eq!(runtime.config().cpu_core, Some(0));
    }
    
    #[test]
    fn test_runtime_with_config() {
        let config = RuntimeConfig {
            cpu_core: Some(2),
            thread_name: "test-runtime".to_string(),
            enable_timing: false,
            stack_size: None,
        };
        
        let runtime = SriQuantRuntime::with_config(config);
        assert_eq!(runtime.config().cpu_core, Some(2));
        assert_eq!(runtime.config().thread_name, "test-runtime");
        assert!(!runtime.config().enable_timing);
    }
}