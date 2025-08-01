//! Binance User Data Stream WebSocket client
//!
//! Specialized WebSocket client for handling user data streams including:
//! - Account updates
//! - Balance updates  
//! - Order updates
//! - Trade executions

use crate::errors::{ExchangeError, Result};
use crate::websocket::MonoioWebSocket;
use sriquant_core::prelude::*;
use super::rest::BinanceConfig;

use tracing::{info, debug};
use serde_json::Value;
use url::Url;

/// Binance User Stream WebSocket client
pub struct BinanceUserStreamClient {
    #[allow(dead_code)]
    config: BinanceConfig,
    base_url: String,
    websocket: Option<MonoioWebSocket>,
    listen_key: String,
}

impl BinanceUserStreamClient {
    /// Create a new user stream client
    pub fn new(config: BinanceConfig) -> Self {
        let base_url = if config.testnet {
            "wss://stream.testnet.binance.vision".to_string()
        } else {
            "wss://stream.binance.com:9443".to_string()
        };
        
        info!("🔗 Binance User Stream client created");
        info!("   Base URL: {}", base_url);
        
        Self {
            config,
            base_url,
            websocket: None,
            listen_key: String::new(),
        }
    }
    
    /// Connect to user data stream
    pub async fn connect(&mut self, listen_key: &str) -> Result<()> {
        let timer = PerfTimer::start("binance_user_stream_connect".to_string());
        
        // Store listen key
        self.listen_key = listen_key.to_string();
        
        // Connect to user data stream endpoint
        let stream_url = format!("{}/ws/{}", self.base_url, listen_key);
        let url = Url::parse(&stream_url)
            .map_err(|e| ExchangeError::InvalidUrl(e.to_string()))?;
        
        info!("🔗 Connecting to Binance user data stream: {}", url);
        
        // Establish WebSocket connection
        let websocket = MonoioWebSocket::connect(url).await?;
        self.websocket = Some(websocket);
        
        timer.log_elapsed();
        info!("✅ Connected to user data stream");
        
        Ok(())
    }
    
    /// Receive and process next user data event
    pub async fn receive_event(&mut self) -> Result<UserDataEvent> {
        loop {
            let message = if let Some(ref mut ws) = self.websocket {
                let timer = PerfTimer::start("binance_user_stream_receive".to_string());
                let msg = ws.receive_text().await?;
                timer.log_elapsed();
                msg
            } else {
                return Err(ExchangeError::NetworkError("User stream not connected".to_string()));
            };
            
            debug!("Received user data message: {}", message);
            
            match self.process_message(&message) {
                Ok(event) => return Ok(event),
                Err(e) => {
                    debug!("Error processing message: {}", e);
                    continue;
                }
            }
        }
    }
    
    /// Process incoming user data message
    fn process_message(&self, message: &str) -> Result<UserDataEvent> {
        let timer = PerfTimer::start("binance_user_stream_process".to_string());
        
        let json: Value = serde_json::from_str(message)
            .map_err(|e| ExchangeError::SerializationError(e.to_string()))?;
        
        let event = if let Some(event_type) = json["e"].as_str() {
            match event_type {
                "outboundAccountPosition" => self.parse_account_update(&json)?,
                "balanceUpdate" => self.parse_balance_update(&json)?,
                "executionReport" => self.parse_order_update(&json)?,
                _ => return Err(ExchangeError::UnsupportedStream(format!("Unknown user event type: {}", event_type)))
            }
        } else {
            return Err(ExchangeError::InvalidResponse("No event type in user data message".to_string()));
        };
        
        timer.log_elapsed();
        Ok(event)
    }
    
    /// Parse account update event
    fn parse_account_update(&self, data: &Value) -> Result<UserDataEvent> {
        let mut balances = Vec::new();
        
        if let Some(balance_array) = data["B"].as_array() {
            for balance in balance_array {
                let asset = balance["a"].as_str().unwrap_or("").to_string();
                let free = Fixed::from_str_exact(balance["f"].as_str().unwrap_or("0"))
                    .map_err(|_| ExchangeError::InvalidResponse("Invalid free balance".to_string()))?;
                let locked = Fixed::from_str_exact(balance["l"].as_str().unwrap_or("0"))
                    .map_err(|_| ExchangeError::InvalidResponse("Invalid locked balance".to_string()))?;
                
                balances.push(BalanceInfo { asset, free, locked });
            }
        }
        
        let account_update = AccountUpdateEvent {
            event_time: data["E"].as_u64().unwrap_or(0),
            last_account_update: data["u"].as_u64().unwrap_or(0),
            balances,
        };
        
        Ok(UserDataEvent::AccountUpdate(account_update))
    }

    /// Parse balance update event
    fn parse_balance_update(&self, data: &Value) -> Result<UserDataEvent> {
        let balance_update = BalanceUpdateEvent {
            event_time: data["E"].as_u64().unwrap_or(0),
            asset: data["a"].as_str().unwrap_or("").to_string(),
            balance_delta: Fixed::from_str_exact(data["d"].as_str().unwrap_or("0"))
                .map_err(|_| ExchangeError::InvalidResponse("Invalid balance delta".to_string()))?,
            clear_time: data["T"].as_u64().unwrap_or(0),
        };
        
        Ok(UserDataEvent::BalanceUpdate(balance_update))
    }

    /// Parse order update event (execution report)
    fn parse_order_update(&self, data: &Value) -> Result<UserDataEvent> {
        let side = match data["S"].as_str().unwrap_or("") {
            "BUY" => TradeSide::Buy,
            "SELL" => TradeSide::Sell,
            _ => TradeSide::Buy, // default
        };
        
        let order_update = OrderUpdateEvent {
            event_time: data["E"].as_u64().unwrap_or(0),
            symbol: data["s"].as_str().unwrap_or("").to_string(),
            client_order_id: data["c"].as_str().unwrap_or("").to_string(),
            side,
            order_type: data["o"].as_str().unwrap_or("").to_string(),
            time_in_force: data["f"].as_str().unwrap_or("").to_string(),
            order_quantity: Fixed::from_str_exact(data["q"].as_str().unwrap_or("0"))
                .map_err(|_| ExchangeError::InvalidResponse("Invalid order quantity".to_string()))?,
            order_price: Fixed::from_str_exact(data["p"].as_str().unwrap_or("0"))
                .map_err(|_| ExchangeError::InvalidResponse("Invalid order price".to_string()))?,
            stop_price: Fixed::from_str_exact(data["P"].as_str().unwrap_or("0"))
                .map_err(|_| ExchangeError::InvalidResponse("Invalid stop price".to_string()))?,
            iceberg_quantity: Fixed::from_str_exact(data["F"].as_str().unwrap_or("0"))
                .map_err(|_| ExchangeError::InvalidResponse("Invalid iceberg quantity".to_string()))?,
            order_list_id: data["g"].as_i64().unwrap_or(-1),
            original_client_order_id: data["C"].as_str().unwrap_or("").to_string(),
            execution_type: data["x"].as_str().unwrap_or("").to_string(),
            order_status: data["X"].as_str().unwrap_or("").to_string(),
            order_reject_reason: data["r"].as_str().unwrap_or("").to_string(),
            order_id: data["i"].as_u64().unwrap_or(0),
            last_executed_quantity: Fixed::from_str_exact(data["l"].as_str().unwrap_or("0"))
                .map_err(|_| ExchangeError::InvalidResponse("Invalid last executed quantity".to_string()))?,
            cumulative_filled_quantity: Fixed::from_str_exact(data["z"].as_str().unwrap_or("0"))
                .map_err(|_| ExchangeError::InvalidResponse("Invalid cumulative filled quantity".to_string()))?,
            last_executed_price: Fixed::from_str_exact(data["L"].as_str().unwrap_or("0"))
                .map_err(|_| ExchangeError::InvalidResponse("Invalid last executed price".to_string()))?,
            commission_amount: Fixed::from_str_exact(data["n"].as_str().unwrap_or("0"))
                .map_err(|_| ExchangeError::InvalidResponse("Invalid commission amount".to_string()))?,
            commission_asset: data["N"].as_str().unwrap_or("").to_string(),
            transaction_time: data["T"].as_u64().unwrap_or(0),
            trade_id: data["t"].as_u64().unwrap_or(0),
            is_order_on_book: data["w"].as_bool().unwrap_or(false),
            is_trade_maker_side: data["m"].as_bool().unwrap_or(false),
            order_creation_time: data["O"].as_u64().unwrap_or(0),
            cumulative_quote_asset_transacted_quantity: Fixed::from_str_exact(data["Z"].as_str().unwrap_or("0"))
                .map_err(|_| ExchangeError::InvalidResponse("Invalid cumulative quote quantity".to_string()))?,
            last_quote_asset_transacted_quantity: Fixed::from_str_exact(data["Y"].as_str().unwrap_or("0"))
                .map_err(|_| ExchangeError::InvalidResponse("Invalid last quote quantity".to_string()))?,
            quote_order_quantity: Fixed::from_str_exact(data["Q"].as_str().unwrap_or("0"))
                .map_err(|_| ExchangeError::InvalidResponse("Invalid quote order quantity".to_string()))?,
        };
        
        Ok(UserDataEvent::OrderUpdate(order_update))
    }
    
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.websocket.as_ref().is_some_and(|ws| ws.is_connected())
    }
    
    /// Send ping to keep connection alive
    pub async fn ping(&mut self) -> Result<()> {
        if let Some(ref mut ws) = self.websocket {
            ws.ping(vec![]).await?;
            debug!("🏓 Sent user stream ping");
        }
        Ok(())
    }
    
    /// Close the connection
    pub async fn close(&mut self) -> Result<()> {
        if let Some(mut ws) = self.websocket.take() {
            info!("🔌 Closing user stream connection");
            ws.close(1000, "Normal closure".to_string()).await?;
        }
        Ok(())
    }
    
    /// Get the current listen key
    pub fn get_listen_key(&self) -> &str {
        &self.listen_key
    }
}

/// User data events
#[derive(Debug, Clone)]
pub enum UserDataEvent {
    AccountUpdate(AccountUpdateEvent),
    BalanceUpdate(BalanceUpdateEvent),
    OrderUpdate(OrderUpdateEvent),
}

/// Account update event
#[derive(Debug, Clone)]
pub struct AccountUpdateEvent {
    pub event_time: u64,
    pub last_account_update: u64,
    pub balances: Vec<BalanceInfo>,
}

/// Balance information
#[derive(Debug, Clone)]
pub struct BalanceInfo {
    pub asset: String,
    pub free: Fixed,
    pub locked: Fixed,
}

/// Balance update event
#[derive(Debug, Clone)]
pub struct BalanceUpdateEvent {
    pub event_time: u64,
    pub asset: String,
    pub balance_delta: Fixed,
    pub clear_time: u64,
}

/// Order update event
#[derive(Debug, Clone)]
pub struct OrderUpdateEvent {
    pub event_time: u64,
    pub symbol: String,
    pub client_order_id: String,
    pub side: TradeSide,
    pub order_type: String,
    pub time_in_force: String,
    pub order_quantity: Fixed,
    pub order_price: Fixed,
    pub stop_price: Fixed,
    pub iceberg_quantity: Fixed,
    pub order_list_id: i64,
    pub original_client_order_id: String,
    pub execution_type: String,
    pub order_status: String,
    pub order_reject_reason: String,
    pub order_id: u64,
    pub last_executed_quantity: Fixed,
    pub cumulative_filled_quantity: Fixed,
    pub last_executed_price: Fixed,
    pub commission_amount: Fixed,
    pub commission_asset: String,
    pub transaction_time: u64,
    pub trade_id: u64,
    pub is_order_on_book: bool,
    pub is_trade_maker_side: bool,
    pub order_creation_time: u64,
    pub cumulative_quote_asset_transacted_quantity: Fixed,
    pub last_quote_asset_transacted_quantity: Fixed,
    pub quote_order_quantity: Fixed,
}

/// Trade side
#[derive(Debug, Clone)]
pub enum TradeSide {
    Buy,
    Sell,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_user_stream_client_creation() {
        let config = BinanceConfig::testnet();
        let client = BinanceUserStreamClient::new(config);
        assert_eq!(client.base_url, "wss://stream.testnet.binance.vision");
        assert!(!client.is_connected());
    }
}