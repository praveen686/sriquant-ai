//! # SriQuant.ai Core
//! 
//! High-performance core runtime and types with optimized architecture.
//! 
//! ## Architecture Principles
//! 
//! 1. **Single-threaded async with monoio** - Maximum single-core performance
//! 2. **CPU binding** - Dedicated CPU cores for trading threads
//! 3. **Nanosecond precision timing** - 7ns latency, 0.3ns precision
//! 4. **Fixed-point arithmetic** - Exact decimal calculations
//! 5. **Lock-free communication** - Ringbuf for inter-thread messaging
//! 6. **Unified logging** - ftlog for consistent logging
//! 7. **Efficient ID generation** - nanoid for unique identifiers

pub mod runtime;
pub mod timing;
pub mod fixed;
pub mod logging;
pub mod id_gen;
pub mod cpu;

// Re-export commonly used items
pub use runtime::SriQuantRuntime;
pub use timing::{nanos, PerfTimer, Timestamp};
pub use fixed::Fixed;
pub use logging::init_logging;
pub use id_gen::{generate_id, OrderId, TradeId};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::runtime::SriQuantRuntime;
    pub use crate::timing::{nanos, PerfTimer, Timestamp};
    pub use crate::fixed::Fixed;
    pub use crate::id_gen::{generate_id, OrderId, TradeId, generate_id_with_prefix, idgen_next_id};
    pub use crate::logging::init_logging;
    pub use crate::cpu::{bind_to_cpu_set, get_cpu_count};
    
    // Common external types
    pub use monoio;
    pub use serde::{Deserialize, Serialize};
    pub use chrono::{DateTime, Utc};
}