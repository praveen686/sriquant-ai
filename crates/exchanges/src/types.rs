//! Common exchange types and data structures
//!
//! High-performance architecture with fixed-point arithmetic
//! for all financial calculations.

use sriquant_core::prelude::*;
use serde::{Deserialize, Serialize};

/// Generic order side
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl std::fmt::Display for OrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderSide::Buy => write!(f, "BUY"),
            OrderSide::Sell => write!(f, "SELL"),
        }
    }
}

/// Generic order type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    StopLossLimit,
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderType::Market => write!(f, "MARKET"),
            OrderType::Limit => write!(f, "LIMIT"),
            OrderType::StopLoss => write!(f, "STOP_LOSS"),
            OrderType::StopLossLimit => write!(f, "STOP_LOSS_LIMIT"),
        }
    }
}

/// Generic order status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
    Expired,
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderStatus::New => write!(f, "NEW"),
            OrderStatus::PartiallyFilled => write!(f, "PARTIALLY_FILLED"),
            OrderStatus::Filled => write!(f, "FILLED"),
            OrderStatus::Canceled => write!(f, "CANCELED"),
            OrderStatus::Rejected => write!(f, "REJECTED"),
            OrderStatus::Expired => write!(f, "EXPIRED"),
        }
    }
}

/// Generic time in force
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeInForce {
    GoodTillCanceled,
    ImmediateOrCancel,
    FillOrKill,
}

impl std::fmt::Display for TimeInForce {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeInForce::GoodTillCanceled => write!(f, "GTC"),
            TimeInForce::ImmediateOrCancel => write!(f, "IOC"),
            TimeInForce::FillOrKill => write!(f, "FOK"),
        }
    }
}

/// Generic symbol information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub symbol: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub status: String,
    pub min_quantity: Fixed,
    pub max_quantity: Fixed,
    pub quantity_precision: u32,
    pub min_price: Fixed,
    pub max_price: Fixed,
    pub price_precision: u32,
    pub min_notional: Fixed,
}

/// Generic order request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: Fixed,
    pub price: Option<Fixed>,
    pub stop_price: Option<Fixed>,
    pub time_in_force: Option<TimeInForce>,
    pub client_order_id: Option<String>,
}

/// Generic order response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub client_order_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: Fixed,
    pub price: Option<Fixed>,
    pub stop_price: Option<Fixed>,
    pub status: OrderStatus,
    pub filled_quantity: Fixed,
    pub average_price: Option<Fixed>,
    pub time_in_force: Option<TimeInForce>,
    pub timestamp: u64,
    pub update_time: u64,
}

/// Generic balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub asset: String,
    pub free: Fixed,
    pub locked: Fixed,
}

impl Balance {
    /// Get total balance (free + locked)
    pub fn total(&self) -> Fixed {
        self.free + self.locked
    }
}

/// Generic account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub account_type: String,
    pub can_trade: bool,
    pub can_withdraw: bool,
    pub can_deposit: bool,
    pub balances: Vec<Balance>,
    pub update_time: u64,
}

/// Generic ticker information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub symbol: String,
    pub price: Fixed,
    pub price_change: Fixed,
    pub price_change_percent: Fixed,
    pub high: Fixed,
    pub low: Fixed,
    pub volume: Fixed,
    pub quote_volume: Fixed,
    pub timestamp: u64,
}

/// Generic trade information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub symbol: String,
    pub price: Fixed,
    pub quantity: Fixed,
    pub side: OrderSide,
    pub timestamp: u64,
    pub is_buyer_maker: bool,
}

/// Generic order book level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookLevel {
    pub price: Fixed,
    pub quantity: Fixed,
}

/// Generic order book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub symbol: String,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
    pub timestamp: u64,
    pub update_id: u64,
}

impl OrderBook {
    /// Get best bid price
    pub fn best_bid(&self) -> Option<Fixed> {
        self.bids.first().map(|level| level.price)
    }
    
    /// Get best ask price
    pub fn best_ask(&self) -> Option<Fixed> {
        self.asks.first().map(|level| level.price)
    }
    
    /// Get bid-ask spread
    pub fn spread(&self) -> Option<Fixed> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }
    
    /// Get mid price
    pub fn mid_price(&self) -> Option<Fixed> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid + ask) / Fixed::from_i64(2).unwrap()),
            _ => None,
        }
    }
}

/// Generic kline/candlestick data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kline {
    pub symbol: String,
    pub interval: String,
    pub open_time: u64,
    pub close_time: u64,
    pub open: Fixed,
    pub high: Fixed,
    pub low: Fixed,
    pub close: Fixed,
    pub volume: Fixed,
    pub quote_volume: Fixed,
    pub number_of_trades: u32,
    pub is_closed: bool,
}

/// Generic market data event
#[derive(Debug, Clone)]
pub enum MarketData {
    Ticker(Ticker),
    Trade(Trade),
    OrderBook(OrderBook),
    Kline(Kline),
}

/// Connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Error,
}

/// Subscription status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubscriptionStatus {
    Unsubscribed,
    Subscribing,
    Subscribed,
    Unsubscribing,
    Error,
}

/// Stream subscription
#[derive(Debug, Clone)]
pub struct Subscription {
    pub stream: String,
    pub symbol: String,
    pub status: SubscriptionStatus,
    pub last_update: u64,
}

/// Performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub latency_nanos: u64,
    pub requests_per_second: f64,
    pub success_rate: f64,
    pub error_count: u64,
    pub last_update: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_order_side_display() {
        assert_eq!(OrderSide::Buy.to_string(), "BUY");
        assert_eq!(OrderSide::Sell.to_string(), "SELL");
    }
    
    #[test]
    fn test_order_type_display() {
        assert_eq!(OrderType::Market.to_string(), "MARKET");
        assert_eq!(OrderType::Limit.to_string(), "LIMIT");
    }
    
    #[test]
    fn test_balance_total() {
        let balance = Balance {
            asset: "BTC".to_string(),
            free: Fixed::from_str_exact("1.0").unwrap(),
            locked: Fixed::from_str_exact("0.5").unwrap(),
        };
        
        assert_eq!(balance.total().to_string(), "1.5");
    }
    
    #[test]
    fn test_order_book_calculations() {
        let order_book = OrderBook {
            symbol: "BTCUSDT".to_string(),
            bids: vec![
                OrderBookLevel {
                    price: Fixed::from_str_exact("50000.0").unwrap(),
                    quantity: Fixed::from_str_exact("1.0").unwrap(),
                },
            ],
            asks: vec![
                OrderBookLevel {
                    price: Fixed::from_str_exact("50001.0").unwrap(),
                    quantity: Fixed::from_str_exact("1.0").unwrap(),
                },
            ],
            timestamp: 1234567890,
            update_id: 123,
        };
        
        assert_eq!(order_book.best_bid().unwrap().to_string(), "50000.0");
        assert_eq!(order_book.best_ask().unwrap().to_string(), "50001.0");
        assert_eq!(order_book.spread().unwrap().to_string(), "1.0");
        // Mid price should be (50000 + 50001) / 2 = 50000.5
        assert_eq!(order_book.mid_price().unwrap().to_string(), "50000.5");
    }
}