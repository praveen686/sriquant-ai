//! # SriQuant.ai Exchange Integrations
//! 
//! High-performance exchange integrations for algorithmic trading.
//! Currently focuses on Binance with plans to expand to other exchanges.
//!
//! ## Architecture
//!
//! - **monoio-based HTTP client** - Single-threaded async for maximum performance
//! - **Precision timing** - Track order latency with nanosecond precision
//! - **Fixed-point arithmetic** - Exact decimal calculations
//! - **Unified interface** - Consistent API across all exchanges
//! - **WebSocket streaming** - Real-time market data and order updates

pub mod binance;
pub mod traits;
pub mod types;
pub mod errors;
pub mod http;
pub mod websocket;

// Re-export main types
pub use binance::BinanceExchange;
pub use traits::{Exchange, StreamingExchange};
pub use types::*;
pub use errors::{ExchangeError, Result};
pub use http::MonoioHttpsClient;
pub use websocket::MonoioWebSocket;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::binance::BinanceExchange;
    pub use crate::traits::{Exchange, StreamingExchange};
    pub use crate::types::*;
    pub use crate::errors::{ExchangeError, Result};
    pub use crate::http::MonoioHttpsClient;
    pub use crate::websocket::MonoioWebSocket;
    pub use sriquant_core::prelude::*;
}