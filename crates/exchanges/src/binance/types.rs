//! Binance-specific types and data structures
//!
//! High-performance architecture with fixed-point arithmetic
//! and precise timing for all financial calculations.

use sriquant_core::prelude::*;
use serde::{Deserialize, Serialize};

/// Binance order status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinanceOrderStatus {
    #[serde(rename = "NEW")]
    New,
    #[serde(rename = "PARTIALLY_FILLED")]
    PartiallyFilled,
    #[serde(rename = "FILLED")]
    Filled,
    #[serde(rename = "CANCELED")]
    Canceled,
    #[serde(rename = "PENDING_CANCEL")]
    PendingCancel,
    #[serde(rename = "REJECTED")]
    Rejected,
    #[serde(rename = "EXPIRED")]
    Expired,
}

/// Binance order type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinanceOrderType {
    #[serde(rename = "LIMIT")]
    Limit,
    #[serde(rename = "MARKET")]
    Market,
    #[serde(rename = "STOP_LOSS")]
    StopLoss,
    #[serde(rename = "STOP_LOSS_LIMIT")]
    StopLossLimit,
    #[serde(rename = "TAKE_PROFIT")]
    TakeProfit,
    #[serde(rename = "TAKE_PROFIT_LIMIT")]
    TakeProfitLimit,
    #[serde(rename = "LIMIT_MAKER")]
    LimitMaker,
}

/// Binance order side
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinanceOrderSide {
    #[serde(rename = "BUY")]
    Buy,
    #[serde(rename = "SELL")]
    Sell,
}

/// Binance time in force
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinanceTimeInForce {
    #[serde(rename = "GTC")]
    GoodTillCanceled,
    #[serde(rename = "IOC")]
    ImmediateOrCancel,
    #[serde(rename = "FOK")]
    FillOrKill,
}

/// Binance account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceAccountInfo {
    #[serde(rename = "makerCommission")]
    pub maker_commission: u32,
    #[serde(rename = "takerCommission")]
    pub taker_commission: u32,
    #[serde(rename = "buyerCommission")]
    pub buyer_commission: u32,
    #[serde(rename = "sellerCommission")]
    pub seller_commission: u32,
    #[serde(rename = "canTrade")]
    pub can_trade: bool,
    #[serde(rename = "canWithdraw")]
    pub can_withdraw: bool,
    #[serde(rename = "canDeposit")]
    pub can_deposit: bool,
    #[serde(rename = "updateTime")]
    pub update_time: u64,
    #[serde(rename = "accountType")]
    pub account_type: String,
    pub balances: Vec<BinanceBalance>,
    pub permissions: Vec<String>,
}

/// Binance account balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceBalance {
    pub asset: String,
    #[serde(rename = "free")]
    pub free: String,
    #[serde(rename = "locked")]
    pub locked: String,
}

impl BinanceBalance {
    /// Get free balance as Fixed
    pub fn free_amount(&self) -> Result<Fixed, crate::errors::ExchangeError> {
        Fixed::from_str_exact(&self.free)
            .map_err(|_| crate::errors::ExchangeError::InvalidResponse("Invalid free balance".to_string()))
    }
    
    /// Get locked balance as Fixed
    pub fn locked_amount(&self) -> Result<Fixed, crate::errors::ExchangeError> {
        Fixed::from_str_exact(&self.locked)
            .map_err(|_| crate::errors::ExchangeError::InvalidResponse("Invalid locked balance".to_string()))
    }
    
    /// Get total balance (free + locked) as Fixed
    pub fn total_amount(&self) -> Result<Fixed, crate::errors::ExchangeError> {
        let free = self.free_amount()?;
        let locked = self.locked_amount()?;
        Ok(free + locked)
    }
}

/// Binance order response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceOrderResponse {
    pub symbol: String,
    #[serde(rename = "orderId")]
    pub order_id: u64,
    #[serde(rename = "orderListId")]
    pub order_list_id: i64,
    #[serde(rename = "clientOrderId")]
    pub client_order_id: String,
    #[serde(rename = "transactTime")]
    pub transact_time: u64,
    pub price: String,
    #[serde(rename = "origQty")]
    pub orig_qty: String,
    #[serde(rename = "executedQty")]
    pub executed_qty: String,
    #[serde(rename = "cummulativeQuoteQty")]
    pub cummulative_quote_qty: String,
    pub status: BinanceOrderStatus,
    #[serde(rename = "timeInForce")]
    pub time_in_force: BinanceTimeInForce,
    #[serde(rename = "type")]
    pub order_type: BinanceOrderType,
    pub side: BinanceOrderSide,
    pub fills: Vec<BinanceFill>,
}

/// Binance order fill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceFill {
    pub price: String,
    pub qty: String,
    pub commission: String,
    #[serde(rename = "commissionAsset")]
    pub commission_asset: String,
    #[serde(rename = "tradeId")]
    pub trade_id: u64,
}

impl BinanceFill {
    /// Get fill price as Fixed
    pub fn price_amount(&self) -> Result<Fixed, crate::errors::ExchangeError> {
        Fixed::from_str_exact(&self.price)
            .map_err(|_| crate::errors::ExchangeError::InvalidResponse("Invalid fill price".to_string()))
    }
    
    /// Get fill quantity as Fixed
    pub fn quantity_amount(&self) -> Result<Fixed, crate::errors::ExchangeError> {
        Fixed::from_str_exact(&self.qty)
            .map_err(|_| crate::errors::ExchangeError::InvalidResponse("Invalid fill quantity".to_string()))
    }
    
    /// Get commission as Fixed
    pub fn commission_amount(&self) -> Result<Fixed, crate::errors::ExchangeError> {
        Fixed::from_str_exact(&self.commission)
            .map_err(|_| crate::errors::ExchangeError::InvalidResponse("Invalid commission".to_string()))
    }
}

/// Binance order query response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceOrderQuery {
    pub symbol: String,
    #[serde(rename = "orderId")]
    pub order_id: u64,
    #[serde(rename = "orderListId")]
    pub order_list_id: i64,
    #[serde(rename = "clientOrderId")]
    pub client_order_id: String,
    pub price: String,
    #[serde(rename = "origQty")]
    pub orig_qty: String,
    #[serde(rename = "executedQty")]
    pub executed_qty: String,
    #[serde(rename = "cummulativeQuoteQty")]
    pub cummulative_quote_qty: String,
    pub status: BinanceOrderStatus,
    #[serde(rename = "timeInForce")]
    pub time_in_force: BinanceTimeInForce,
    #[serde(rename = "type")]
    pub order_type: BinanceOrderType,
    pub side: BinanceOrderSide,
    #[serde(rename = "stopPrice")]
    pub stop_price: String,
    #[serde(rename = "icebergQty")]
    pub iceberg_qty: String,
    pub time: u64,
    #[serde(rename = "updateTime")]
    pub update_time: u64,
    #[serde(rename = "isWorking")]
    pub is_working: bool,
    #[serde(rename = "origQuoteOrderQty")]
    pub orig_quote_order_qty: String,
}

/// Binance kline/candlestick data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceKline {
    #[serde(rename = "0")]
    pub open_time: u64,
    #[serde(rename = "1")]
    pub open: String,
    #[serde(rename = "2")]
    pub high: String,
    #[serde(rename = "3")]
    pub low: String,
    #[serde(rename = "4")]
    pub close: String,
    #[serde(rename = "5")]
    pub volume: String,
    #[serde(rename = "6")]
    pub close_time: u64,
    #[serde(rename = "7")]
    pub quote_asset_volume: String,
    #[serde(rename = "8")]
    pub number_of_trades: u32,
    #[serde(rename = "9")]
    pub taker_buy_base_asset_volume: String,
    #[serde(rename = "10")]
    pub taker_buy_quote_asset_volume: String,
    #[serde(rename = "11")]
    pub ignore: String,
}

impl BinanceKline {
    /// Get OHLCV as Fixed values
    pub fn ohlcv(&self) -> Result<(Fixed, Fixed, Fixed, Fixed, Fixed), crate::errors::ExchangeError> {
        let open = Fixed::from_str_exact(&self.open)
            .map_err(|_| crate::errors::ExchangeError::InvalidResponse("Invalid open price".to_string()))?;
        let high = Fixed::from_str_exact(&self.high)
            .map_err(|_| crate::errors::ExchangeError::InvalidResponse("Invalid high price".to_string()))?;
        let low = Fixed::from_str_exact(&self.low)
            .map_err(|_| crate::errors::ExchangeError::InvalidResponse("Invalid low price".to_string()))?;
        let close = Fixed::from_str_exact(&self.close)
            .map_err(|_| crate::errors::ExchangeError::InvalidResponse("Invalid close price".to_string()))?;
        let volume = Fixed::from_str_exact(&self.volume)
            .map_err(|_| crate::errors::ExchangeError::InvalidResponse("Invalid volume".to_string()))?;
        
        Ok((open, high, low, close, volume))
    }
}

/// Binance error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceError {
    pub code: i32,
    pub msg: String,
}

/// Convert Binance order side to our generic type
impl From<BinanceOrderSide> for crate::types::OrderSide {
    fn from(side: BinanceOrderSide) -> Self {
        match side {
            BinanceOrderSide::Buy => crate::types::OrderSide::Buy,
            BinanceOrderSide::Sell => crate::types::OrderSide::Sell,
        }
    }
}

/// Convert our generic order side to Binance type
impl From<crate::types::OrderSide> for BinanceOrderSide {
    fn from(side: crate::types::OrderSide) -> Self {
        match side {
            crate::types::OrderSide::Buy => BinanceOrderSide::Buy,
            crate::types::OrderSide::Sell => BinanceOrderSide::Sell,
        }
    }
}

/// Convert Binance order type to our generic type
impl From<BinanceOrderType> for crate::types::OrderType {
    fn from(order_type: BinanceOrderType) -> Self {
        match order_type {
            BinanceOrderType::Limit => crate::types::OrderType::Limit,
            BinanceOrderType::Market => crate::types::OrderType::Market,
            BinanceOrderType::StopLoss => crate::types::OrderType::StopLoss,
            BinanceOrderType::StopLossLimit => crate::types::OrderType::StopLossLimit,
            BinanceOrderType::LimitMaker => crate::types::OrderType::Limit, // Map to Limit
            _ => crate::types::OrderType::Market, // Default fallback
        }
    }
}

/// Convert our generic order type to Binance type
impl From<crate::types::OrderType> for BinanceOrderType {
    fn from(order_type: crate::types::OrderType) -> Self {
        match order_type {
            crate::types::OrderType::Limit => BinanceOrderType::Limit,
            crate::types::OrderType::Market => BinanceOrderType::Market,
            crate::types::OrderType::StopLoss => BinanceOrderType::StopLoss,
            crate::types::OrderType::StopLossLimit => BinanceOrderType::StopLossLimit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_balance_conversions() {
        let balance = BinanceBalance {
            asset: "BTC".to_string(),
            free: "1.23456789".to_string(),
            locked: "0.10000000".to_string(),
        };
        
        let free = balance.free_amount().unwrap();
        let locked = balance.locked_amount().unwrap();
        let total = balance.total_amount().unwrap();
        
        assert_eq!(free.to_string(), "1.23456789");
        assert_eq!(locked.to_string(), "0.10000000");
        assert_eq!(total.to_string(), "1.33456789");
    }
    
    #[test]
    fn test_fill_conversions() {
        let fill = BinanceFill {
            price: "50000.12345".to_string(),
            qty: "0.01".to_string(),
            commission: "0.001".to_string(),
            commission_asset: "BNB".to_string(),
            trade_id: 12345,
        };
        
        let price = fill.price_amount().unwrap();
        let qty = fill.quantity_amount().unwrap();
        let commission = fill.commission_amount().unwrap();
        
        assert_eq!(price.to_string(), "50000.12345");
        assert_eq!(qty.to_string(), "0.01");
        assert_eq!(commission.to_string(), "0.001");
    }
    
    #[test]
    fn test_side_conversions() {
        let binance_buy = BinanceOrderSide::Buy;
        let generic_buy: crate::types::OrderSide = binance_buy.into();
        assert_eq!(generic_buy, crate::types::OrderSide::Buy);
        
        let generic_sell = crate::types::OrderSide::Sell;
        let binance_sell: BinanceOrderSide = generic_sell.into();
        assert_eq!(binance_sell, BinanceOrderSide::Sell);
    }
}