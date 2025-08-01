//! Exchange traits defining common interfaces
//!
//! High-performance architecture with async traits
//! and high-performance abstractions.

use crate::errors::Result;
use crate::types::*;
use async_trait::async_trait;
use std::collections::HashMap;
use sriquant_core::Fixed;

/// Core exchange interface
#[async_trait]
pub trait Exchange: Send + Sync {
    /// Get exchange name
    fn name(&self) -> &str;
    
    /// Test connectivity and measure latency
    async fn ping(&self) -> Result<u64>;
    
    /// Get server time
    async fn server_time(&self) -> Result<u64>;
    
    /// Get exchange information
    async fn exchange_info(&self) -> Result<HashMap<String, Symbol>>;
    
    /// Get account information
    async fn account_info(&self) -> Result<AccountInfo>;
    
    /// Get account balances
    async fn balances(&self) -> Result<Vec<Balance>>;
    
    /// Get ticker for a symbol
    async fn ticker(&self, symbol: &str) -> Result<Ticker>;
    
    /// Get order book for a symbol
    async fn order_book(&self, symbol: &str, limit: Option<u32>) -> Result<OrderBook>;
    
    /// Get recent trades for a symbol
    async fn recent_trades(&self, symbol: &str, limit: Option<u32>) -> Result<Vec<Trade>>;
    
    /// Get klines/candlesticks for a symbol
    async fn klines(
        &self,
        symbol: &str,
        interval: &str,
        start_time: Option<u64>,
        end_time: Option<u64>,
        limit: Option<u32>,
    ) -> Result<Vec<Kline>>;
}

/// Trading interface for exchanges that support trading
#[async_trait]
pub trait TradingExchange: Exchange {
    /// Place a new order
    async fn place_order(&self, request: OrderRequest) -> Result<OrderResponse>;
    
    /// Cancel an order
    async fn cancel_order(&self, symbol: &str, order_id: &str) -> Result<OrderResponse>;
    
    /// Cancel all orders for a symbol
    async fn cancel_all_orders(&self, symbol: &str) -> Result<Vec<OrderResponse>>;
    
    /// Get order status
    async fn get_order(&self, symbol: &str, order_id: &str) -> Result<OrderResponse>;
    
    /// Get open orders
    async fn open_orders(&self, symbol: Option<&str>) -> Result<Vec<OrderResponse>>;
    
    /// Get order history
    async fn order_history(
        &self,
        symbol: &str,
        start_time: Option<u64>,
        end_time: Option<u64>,
        limit: Option<u32>,
    ) -> Result<Vec<OrderResponse>>;
    
    /// Get trade history
    async fn trade_history(
        &self,
        symbol: &str,
        start_time: Option<u64>,
        end_time: Option<u64>,
        limit: Option<u32>,
    ) -> Result<Vec<Trade>>;
}

/// Streaming interface for real-time market data
#[async_trait]
pub trait StreamingExchange: Send + Sync {
    /// Connect to WebSocket streams
    async fn connect(&mut self) -> Result<()>;
    
    /// Disconnect from WebSocket streams
    async fn disconnect(&mut self) -> Result<()>;
    
    /// Subscribe to ticker updates
    async fn subscribe_ticker(&mut self, symbol: &str) -> Result<()>;
    
    /// Subscribe to trade updates
    async fn subscribe_trades(&mut self, symbol: &str) -> Result<()>;
    
    /// Subscribe to order book updates
    async fn subscribe_order_book(&mut self, symbol: &str, levels: Option<u32>) -> Result<()>;
    
    /// Subscribe to kline updates
    async fn subscribe_klines(&mut self, symbol: &str, interval: &str) -> Result<()>;
    
    /// Unsubscribe from a stream
    async fn unsubscribe(&mut self, stream: &str) -> Result<()>;
    
    /// Get next market data event
    async fn next_event(&mut self) -> Result<Option<MarketData>>;
    
    /// Get connection status
    fn connection_status(&self) -> ConnectionStatus;
    
    /// Get active subscriptions
    fn subscriptions(&self) -> Vec<Subscription>;
}

/// Advanced trading features
#[async_trait]
pub trait AdvancedTradingExchange: TradingExchange {
    /// Place multiple orders atomically
    async fn place_batch_orders(&self, requests: Vec<OrderRequest>) -> Result<Vec<OrderResponse>>;
    
    /// Modify an existing order
    async fn modify_order(
        &self,
        symbol: &str,
        order_id: &str,
        quantity: Option<Fixed>,
        price: Option<Fixed>,
    ) -> Result<OrderResponse>;
    
    /// Get order fills/executions
    async fn order_fills(&self, symbol: &str, order_id: &str) -> Result<Vec<Trade>>;
    
    /// Set position mode (for futures)
    async fn set_position_mode(&self, dual_side: bool) -> Result<()>;
    
    /// Get position information (for futures)
    async fn positions(&self, symbol: Option<&str>) -> Result<Vec<Position>>;
    
    /// Set leverage (for futures)
    async fn set_leverage(&self, symbol: &str, leverage: u32) -> Result<()>;
}

/// Risk management interface
#[async_trait]
pub trait RiskManagement: Send + Sync {
    /// Check if order is within risk limits
    async fn validate_order(&self, request: &OrderRequest) -> Result<bool>;
    
    /// Get maximum order size for symbol
    async fn max_order_size(&self, symbol: &str, side: OrderSide) -> Result<Fixed>;
    
    /// Get available balance for trading
    async fn available_balance(&self, asset: &str) -> Result<Fixed>;
    
    /// Calculate required margin for order
    async fn required_margin(&self, request: &OrderRequest) -> Result<Fixed>;
    
    /// Get current exposure
    async fn exposure(&self, symbol: Option<&str>) -> Result<HashMap<String, Fixed>>;
}

/// Performance monitoring interface
pub trait PerformanceMonitoring: Send + Sync {
    /// Get latency statistics
    fn latency_stats(&self) -> PerformanceMetrics;
    
    /// Get error rate
    fn error_rate(&self) -> f64;
    
    /// Get request rate
    fn request_rate(&self) -> f64;
    
    /// Reset performance counters
    fn reset_stats(&mut self);
}

/// Position information (for futures trading)
#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub side: PositionSide,
    pub size: Fixed,
    pub entry_price: Fixed,
    pub mark_price: Fixed,
    pub unrealized_pnl: Fixed,
    pub leverage: u32,
    pub margin: Fixed,
    pub maintenance_margin: Fixed,
    pub update_time: u64,
}

/// Position side (for futures trading)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionSide {
    Long,
    Short,
    Both, // For hedge mode
}

impl std::fmt::Display for PositionSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PositionSide::Long => write!(f, "LONG"),
            PositionSide::Short => write!(f, "SHORT"),
            PositionSide::Both => write!(f, "BOTH"),
        }
    }
}

/// Exchange factory for creating exchange instances
pub trait ExchangeFactory: Send + Sync {
    /// Create a new exchange instance
    fn create_exchange(&self, config: &str) -> Result<Box<dyn Exchange>>;
    
    /// Create a trading exchange instance
    fn create_trading_exchange(&self, config: &str) -> Result<Box<dyn TradingExchange>>;
    
    /// Create a streaming exchange instance
    fn create_streaming_exchange(&self, config: &str) -> Result<Box<dyn StreamingExchange>>;
    
    /// List supported exchanges
    fn supported_exchanges(&self) -> Vec<String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_position_side_display() {
        assert_eq!(PositionSide::Long.to_string(), "LONG");
        assert_eq!(PositionSide::Short.to_string(), "SHORT");
        assert_eq!(PositionSide::Both.to_string(), "BOTH");
    }
}