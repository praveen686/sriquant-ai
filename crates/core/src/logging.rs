//! Unified logging integration
//!
//! Integrates ftlog for standardized logging across Rust components,
//! offering simple configuration for enhanced debugging and monitoring.

use tracing::Level;
#[cfg(not(feature = "ftlog"))]
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize unified logging system ()
pub fn init_logging() {
    INIT.call_once(|| {
        // Try to use ftlog if available, fall back to tracing
        #[cfg(feature = "ftlog")]
        {
            init_ftlog();
        }
        
        #[cfg(not(feature = "ftlog"))]
        {
            init_tracing();
        }
    });
}

/// Initialize ftlog (when available)
#[cfg(feature = "ftlog")]
fn init_ftlog() {
    use ftlog;
    ftlog::builder()
        .max_log_level(ftlog::LevelFilter::Debug)
        .bounded(100000, false) // 100k buffer, non-blocking
        .utc() // Use UTC timestamps
        .build()
        .expect("Failed to initialize ftlog");
        
    tracing::info!("📝 Initialized ftlog unified logging");
}

/// Initialize tracing (fallback)
#[cfg(not(feature = "ftlog"))]
fn init_tracing() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .finish();
        
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");
        
    tracing::info!("📝 Initialized tracing logging (ftlog not available)");
}

/// High-performance log levels
pub enum LogLevel {
    Trace,
    Debug, 
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

/// Configure logging level at runtime
pub fn set_log_level(_level: LogLevel) {
    // This would require more complex subscriber management
    // For now, use environment variable RUST_LOG
    tracing::warn!("Log level changes require restart. Use RUST_LOG environment variable.");
}

/// Performance-optimized logging macros
#[macro_export]
macro_rules! log_latency {
    ($operation:expr, $duration_micros:expr) => {
        if $duration_micros < 1000 {
            tracing::debug!("⚡ {} completed in {}μs", $operation, $duration_micros);
        } else {
            tracing::info!("⚡ {} completed in {:.3}ms", $operation, $duration_micros as f64 / 1000.0);
        }
    };
}

#[macro_export]
macro_rules! log_trade {
    ($side:expr, $symbol:expr, $quantity:expr, $price:expr) => {
        tracing::info!("💰 TRADE: {} {} {} @ {}", $side, $symbol, $quantity, $price);
    };
}

#[macro_export]
macro_rules! log_order {
    ($action:expr, $order_id:expr, $symbol:expr) => {
        tracing::info!("📋 ORDER {}: {} ({})", $action, $order_id, $symbol);
    };
}

#[macro_export]
macro_rules! log_error {
    ($operation:expr, $error:expr) => {
        tracing::error!("❌ {} failed: {}", $operation, $error);
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_logging_init() {
        // Should not panic
        init_logging();
        init_logging(); // Should be safe to call multiple times
    }
    
    #[test] 
    fn test_log_macros() {
        init_logging();
        
        log_latency!("test_operation", 500);
        log_trade!("BUY", "BTCUSDT", "1.0", "50000.00");
        log_order!("PLACED", "12345", "ETHUSDT");
        log_error!("order_placement", "insufficient balance");
    }
}