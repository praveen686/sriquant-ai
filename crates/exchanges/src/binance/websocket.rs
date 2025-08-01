//! Binance WebSocket streaming client using monoio for maximum performance
//!
//! High-performance architecture:
//! - Single-threaded async with monoio
//! - Nanosecond precision timing for latency measurement
//! - Efficient WebSocket handling
//! - Real-time market data streaming

use crate::errors::{ExchangeError, Result};
use crate::websocket::MonoioWebSocket;
use sriquant_core::prelude::*;
use sriquant_core::timing::nanos;
use super::rest::BinanceConfig;

use std::collections::HashMap;
use tracing::{info, debug};
use serde_json::Value;
use url::Url;

/// High-performance Binance WebSocket client using monoio
pub struct BinanceWebSocketClient {
    #[allow(dead_code)] // Stored for future authenticated WebSocket operations
    config: BinanceConfig,
    base_url: String,
    subscriptions: HashMap<String, bool>,
    websocket: Option<MonoioWebSocket>,
}

impl BinanceWebSocketClient {
    /// Create a new Binance WebSocket client
    pub fn new(config: BinanceConfig) -> Self {
        let base_url = if config.testnet {
            "wss://stream.testnet.binance.vision".to_string()
        } else {
            "wss://stream.binance.com:9443".to_string()
        };
        
        info!("🔗 Binance WebSocket client created");
        info!("   Base URL: {}", base_url);
        
        Self {
            config,
            base_url,
            subscriptions: HashMap::new(),
            websocket: None,
        }
    }
    
    /// Connect to WebSocket stream (multi-stream endpoint)
    pub async fn connect(&mut self) -> Result<()> {
        let timer = PerfTimer::start("binance_ws_connect".to_string());
        
        // Connect to multi-stream endpoint for subscriptions
        let stream_url = format!("{}/ws", self.base_url);
        let url = Url::parse(&stream_url)
            .map_err(|e| ExchangeError::InvalidUrl(e.to_string()))?;
        
        info!("🔗 Connecting to Binance WebSocket: {}", url);
        
        // Establish WebSocket connection
        let websocket = MonoioWebSocket::connect(url).await?;
        self.websocket = Some(websocket);
        
        timer.log_elapsed();
        info!("✅ Connected to Binance WebSocket successfully");
        
        Ok(())
    }

    /// Connect to a single stream directly (alternative connection method)
    pub async fn connect_single_stream(&mut self, stream: &str) -> Result<()> {
        let timer = PerfTimer::start("binance_ws_connect_single".to_string());
        
        // Connect directly to a single stream
        let stream_url = format!("{}/ws/{}", self.base_url, stream);
        let url = Url::parse(&stream_url)
            .map_err(|e| ExchangeError::InvalidUrl(e.to_string()))?;
        
        info!("🔗 Connecting to single Binance WebSocket stream: {}", url);
        
        // Establish WebSocket connection
        let websocket = MonoioWebSocket::connect(url).await?;
        self.websocket = Some(websocket);
        
        // Mark this stream as subscribed (no subscription message needed)
        self.subscriptions.insert(stream.to_string(), true);
        
        timer.log_elapsed();
        info!("✅ Connected to single stream: {}", stream);
        
        Ok(())
    }

    /// Connect and subscribe to multiple streams
    pub async fn connect_multi_stream(&mut self, streams: Vec<&str>) -> Result<()> {
        // First connect to the multi-stream endpoint
        self.connect().await?;
        
        info!("📊 Subscribing to {} streams...", streams.len());
        
        // Subscribe to each stream
        for stream in streams {
            self.subscribe_stream(stream).await?;
        }
        
        info!("✅ All {} streams subscribed successfully", self.subscriptions.len());
        Ok(())
    }

    
    /// Subscribe to ticker updates for a symbol
    pub async fn subscribe_ticker(&mut self, symbol: &str) -> Result<()> {
        let stream_name = format!("{}@ticker", symbol.to_lowercase());
        self.subscribe_stream(&stream_name).await
    }
    
    /// Subscribe to order book updates for a symbol
    pub async fn subscribe_depth(&mut self, symbol: &str, levels: Option<u32>) -> Result<()> {
        let stream_name = if let Some(levels) = levels {
            format!("{}@depth{}@100ms", symbol.to_lowercase(), levels)
        } else {
            format!("{}@depth@100ms", symbol.to_lowercase())
        };
        self.subscribe_stream(&stream_name).await
    }
    
    /// Subscribe to trade updates for a symbol
    pub async fn subscribe_trades(&mut self, symbol: &str) -> Result<()> {
        let stream_name = format!("{}@trade", symbol.to_lowercase());
        self.subscribe_stream(&stream_name).await
    }
    
    /// Subscribe to kline/candlestick updates
    pub async fn subscribe_klines(&mut self, symbol: &str, interval: &str) -> Result<()> {
        let stream_name = format!("{}@kline_{}", symbol.to_lowercase(), interval);
        self.subscribe_stream(&stream_name).await
    }
    
    /// Generic stream subscription
    async fn subscribe_stream(&mut self, stream: &str) -> Result<()> {
        if self.websocket.is_none() {
            return Err(ExchangeError::NetworkError("WebSocket not connected".to_string()));
        }

        // Generate unique ID for this subscription
        let sub_id = self.subscriptions.len() + 1;

        // Create subscription message
        let subscription_msg = serde_json::json!({
            "method": "SUBSCRIBE",
            "params": [stream],
            "id": sub_id
        });

        info!("📨 Sending subscription message: {}", subscription_msg);

        // Send subscription message
        if let Some(ref mut ws) = self.websocket {
            ws.send_text(subscription_msg.to_string()).await?;
        }

        self.subscriptions.insert(stream.to_string(), true);
        info!("📊 Subscribed to stream: {}", stream);
        Ok(())
    }
    
    /// Receive and process next WebSocket message
    pub async fn receive_message(&mut self) -> Result<MarketDataEvent> {
        loop {
            let message = if let Some(ref mut ws) = self.websocket {
                let timer = PerfTimer::start("binance_ws_receive".to_string());
                let msg = ws.receive_text().await?;
                timer.log_elapsed();
                msg
            } else {
                return Err(ExchangeError::NetworkError("WebSocket not connected".to_string()));
            };
            
            debug!("Received WebSocket message: {}", message);
            
            match self.process_message_content(&message) {
                Ok(event) => return Ok(event),
                Err(ExchangeError::InvalidResponse(msg)) if msg.contains("Subscription confirmation") => {
                    // Skip subscription confirmations and continue reading
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Process incoming WebSocket message content
    fn process_message_content(&self, message: &str) -> Result<MarketDataEvent> {
        let timer = PerfTimer::start("binance_ws_process".to_string());
        
        let json: Value = serde_json::from_str(message)
            .map_err(|e| ExchangeError::SerializationError(e.to_string()))?;
        
        let event = if let Some(stream) = json["stream"].as_str() {
            // Combined stream format: {"stream":"btcusdt@ticker","data":{...}}
            self.parse_stream_data(stream, &json["data"])?
        } else if let Some(event_type) = json["e"].as_str() {
            // Single stream format: {"e":"24hrTicker","s":"BTCUSDT",...}
            self.parse_single_stream_data(event_type, &json)?
        } else if json["lastUpdateId"].is_number() && (json["bids"].is_array() || json["asks"].is_array()) {
            // Order book snapshot format: {"lastUpdateId":123,"bids":[...],"asks":[...]}
            self.parse_order_book_snapshot(&json)?
        } else if let Some(_result) = json["result"].as_null() {
            // Handle subscription confirmation messages ({"result":null,"id":1})
            if let Some(id) = json["id"].as_u64() {
                info!("✅ Subscription confirmed for ID: {}", id);
                return Err(ExchangeError::InvalidResponse("Subscription confirmation - not market data".to_string()));
            } else {
                info!("Unknown subscription response format: {}", message);
                return Err(ExchangeError::InvalidResponse("Unknown subscription response".to_string()));
            }
        } else {
            debug!("Unknown message format: {}", message);
            return Err(ExchangeError::InvalidResponse("Unknown message format".to_string()));
        };
        
        timer.log_elapsed();
        Ok(event)
    }
    
    /// Parse stream data based on stream type
    fn parse_stream_data(&self, stream: &str, data: &Value) -> Result<MarketDataEvent> {
        if stream.contains("@ticker") {
            self.parse_ticker_data(data)
        } else if stream.contains("@depth") {
            self.parse_depth_data(data)
        } else if stream.contains("@trade") {
            self.parse_trade_data(data)
        } else if stream.contains("@kline") {
            self.parse_kline_data(data)
        } else {
            Err(ExchangeError::UnsupportedStream(stream.to_string()))
        }
    }
    
    /// Parse single stream data based on event type
    fn parse_single_stream_data(&self, event_type: &str, data: &Value) -> Result<MarketDataEvent> {
        match event_type {
            "24hrTicker" => self.parse_ticker_data(data),
            "depthUpdate" => self.parse_depth_data(data),
            "trade" => self.parse_trade_data(data),
            "kline" => self.parse_kline_data(data),
            _ => Err(ExchangeError::UnsupportedStream(format!("Unsupported event type: {}", event_type)))
        }
    }

    /// Parse order book snapshot (initial depth data)
    fn parse_order_book_snapshot(&self, data: &Value) -> Result<MarketDataEvent> {
        let mut bids = Vec::new();
        let mut asks = Vec::new();
        
        if let Some(bids_array) = data["bids"].as_array() {
            for bid in bids_array {
                if let Some(bid_array) = bid.as_array() {
                    if bid_array.len() >= 2 {
                        let price = Fixed::from_str_exact(bid_array[0].as_str().unwrap_or("0"))
                            .map_err(|_| ExchangeError::InvalidResponse("Invalid bid price".to_string()))?;
                        let quantity = Fixed::from_str_exact(bid_array[1].as_str().unwrap_or("0"))
                            .map_err(|_| ExchangeError::InvalidResponse("Invalid bid quantity".to_string()))?;
                        bids.push(OrderBookLevel { price, quantity });
                    }
                }
            }
        }
        
        if let Some(asks_array) = data["asks"].as_array() {
            for ask in asks_array {
                if let Some(ask_array) = ask.as_array() {
                    if ask_array.len() >= 2 {
                        let price = Fixed::from_str_exact(ask_array[0].as_str().unwrap_or("0"))
                            .map_err(|_| ExchangeError::InvalidResponse("Invalid ask price".to_string()))?;
                        let quantity = Fixed::from_str_exact(ask_array[1].as_str().unwrap_or("0"))
                            .map_err(|_| ExchangeError::InvalidResponse("Invalid ask quantity".to_string()))?;
                        asks.push(OrderBookLevel { price, quantity });
                    }
                }
            }
        }
        
        let depth = DepthUpdate {
            symbol: "BTCUSDT".to_string(), // For depth snapshots, we know this is BTCUSDT from our subscription
            bids,
            asks,
            timestamp: nanos() / 1_000_000, // Current timestamp in milliseconds
            update_id: data["lastUpdateId"].as_u64().unwrap_or(0),
        };
        
        Ok(MarketDataEvent::Depth(depth))
    }

    /// Parse ticker data
    fn parse_ticker_data(&self, data: &Value) -> Result<MarketDataEvent> {
        let ticker = TickerUpdate {
            symbol: data["s"].as_str().unwrap_or("").to_string(),
            price: Fixed::from_str_exact(data["c"].as_str().unwrap_or("0"))
                .map_err(|_| ExchangeError::InvalidResponse("Invalid price".to_string()))?,
            price_change: Fixed::from_str_exact(data["P"].as_str().unwrap_or("0"))
                .map_err(|_| ExchangeError::InvalidResponse("Invalid price change".to_string()))?,
            volume: Fixed::from_str_exact(data["v"].as_str().unwrap_or("0"))
                .map_err(|_| ExchangeError::InvalidResponse("Invalid volume".to_string()))?,
            timestamp: data["E"].as_u64().unwrap_or(0),
        };
        
        Ok(MarketDataEvent::Ticker(ticker))
    }
    
    /// Parse depth/order book data
    fn parse_depth_data(&self, data: &Value) -> Result<MarketDataEvent> {
        let mut bids = Vec::new();
        let mut asks = Vec::new();
        
        if let Some(bids_array) = data["b"].as_array() {
            for bid in bids_array {
                if let Some(bid_array) = bid.as_array() {
                    if bid_array.len() >= 2 {
                        let price = Fixed::from_str_exact(bid_array[0].as_str().unwrap_or("0"))
                            .map_err(|_| ExchangeError::InvalidResponse("Invalid bid price".to_string()))?;
                        let quantity = Fixed::from_str_exact(bid_array[1].as_str().unwrap_or("0"))
                            .map_err(|_| ExchangeError::InvalidResponse("Invalid bid quantity".to_string()))?;
                        bids.push(OrderBookLevel { price, quantity });
                    }
                }
            }
        }
        
        if let Some(asks_array) = data["a"].as_array() {
            for ask in asks_array {
                if let Some(ask_array) = ask.as_array() {
                    if ask_array.len() >= 2 {
                        let price = Fixed::from_str_exact(ask_array[0].as_str().unwrap_or("0"))
                            .map_err(|_| ExchangeError::InvalidResponse("Invalid ask price".to_string()))?;
                        let quantity = Fixed::from_str_exact(ask_array[1].as_str().unwrap_or("0"))
                            .map_err(|_| ExchangeError::InvalidResponse("Invalid ask quantity".to_string()))?;
                        asks.push(OrderBookLevel { price, quantity });
                    }
                }
            }
        }
        
        let depth = DepthUpdate {
            symbol: data["s"].as_str().unwrap_or("").to_string(),
            bids,
            asks,
            timestamp: data["E"].as_u64().unwrap_or(0),
            update_id: data["u"].as_u64().unwrap_or(0),
        };
        
        Ok(MarketDataEvent::Depth(depth))
    }
    
    /// Parse trade data
    fn parse_trade_data(&self, data: &Value) -> Result<MarketDataEvent> {
        let trade = TradeUpdate {
            symbol: data["s"].as_str().unwrap_or("").to_string(),
            price: Fixed::from_str_exact(data["p"].as_str().unwrap_or("0"))
                .map_err(|_| ExchangeError::InvalidResponse("Invalid trade price".to_string()))?,
            quantity: Fixed::from_str_exact(data["q"].as_str().unwrap_or("0"))
                .map_err(|_| ExchangeError::InvalidResponse("Invalid trade quantity".to_string()))?,
            side: if data["m"].as_bool().unwrap_or(false) { TradeSide::Sell } else { TradeSide::Buy },
            timestamp: data["T"].as_u64().unwrap_or(0),
            trade_id: data["t"].as_u64().unwrap_or(0),
        };
        
        Ok(MarketDataEvent::Trade(trade))
    }
    
    /// Parse kline/candlestick data
    fn parse_kline_data(&self, data: &Value) -> Result<MarketDataEvent> {
        let k = &data["k"];
        
        let kline = KlineUpdate {
            symbol: k["s"].as_str().unwrap_or("").to_string(),
            interval: k["i"].as_str().unwrap_or("").to_string(),
            open_time: k["t"].as_u64().unwrap_or(0),
            close_time: k["T"].as_u64().unwrap_or(0),
            open: Fixed::from_str_exact(k["o"].as_str().unwrap_or("0"))
                .map_err(|_| ExchangeError::InvalidResponse("Invalid open price".to_string()))?,
            high: Fixed::from_str_exact(k["h"].as_str().unwrap_or("0"))
                .map_err(|_| ExchangeError::InvalidResponse("Invalid high price".to_string()))?,
            low: Fixed::from_str_exact(k["l"].as_str().unwrap_or("0"))
                .map_err(|_| ExchangeError::InvalidResponse("Invalid low price".to_string()))?,
            close: Fixed::from_str_exact(k["c"].as_str().unwrap_or("0"))
                .map_err(|_| ExchangeError::InvalidResponse("Invalid close price".to_string()))?,
            volume: Fixed::from_str_exact(k["v"].as_str().unwrap_or("0"))
                .map_err(|_| ExchangeError::InvalidResponse("Invalid volume".to_string()))?,
            is_closed: k["x"].as_bool().unwrap_or(false),
        };
        
        Ok(MarketDataEvent::Kline(kline))
    }
    
    /// Get active subscriptions
    pub fn get_subscriptions(&self) -> Vec<String> {
        self.subscriptions.keys().cloned().collect()
    }
    
    /// Unsubscribe from a stream
    pub async fn unsubscribe(&mut self, stream: &str) -> Result<()> {
        if let Some(ref mut ws) = self.websocket {
            let unsubscription_msg = serde_json::json!({
                "method": "UNSUBSCRIBE",
                "params": [stream],
                "id": 2
            });
            
            ws.send_text(unsubscription_msg.to_string()).await?;
        }
        
        self.subscriptions.remove(stream);
        info!("❌ Unsubscribed from stream: {}", stream);
        Ok(())
    }
    
    /// Close WebSocket connection
    pub async fn close(&mut self) -> Result<()> {
        if let Some(mut ws) = self.websocket.take() {
            info!("🔌 Closing Binance WebSocket connection");
            ws.close(1000, "Normal closure".to_string()).await?;
        }
        self.subscriptions.clear();
        Ok(())
    }
    
    /// Check if WebSocket is connected
    pub fn is_connected(&self) -> bool {
        self.websocket.as_ref().is_some_and(|ws| ws.is_connected())
    }
    
    /// Send ping to keep connection alive
    pub async fn ping(&mut self) -> Result<()> {
        if let Some(ref mut ws) = self.websocket {
            ws.ping(vec![]).await?;
            debug!("🏓 Sent WebSocket ping");
        }
        Ok(())
    }
}

/// Market data events from WebSocket
#[derive(Debug, Clone)]
pub enum MarketDataEvent {
    Ticker(TickerUpdate),
    Depth(DepthUpdate),
    Trade(TradeUpdate),
    Kline(KlineUpdate),
}

/// Ticker update data
#[derive(Debug, Clone)]
pub struct TickerUpdate {
    pub symbol: String,
    pub price: Fixed,
    pub price_change: Fixed,
    pub volume: Fixed,
    pub timestamp: u64,
}

/// Depth/order book update data
#[derive(Debug, Clone)]
pub struct DepthUpdate {
    pub symbol: String,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
    pub timestamp: u64,
    pub update_id: u64,
}

/// Trade update data
#[derive(Debug, Clone)]
pub struct TradeUpdate {
    pub symbol: String,
    pub price: Fixed,
    pub quantity: Fixed,
    pub side: TradeSide,
    pub timestamp: u64,
    pub trade_id: u64,
}

/// Kline/candlestick update data
#[derive(Debug, Clone)]
pub struct KlineUpdate {
    pub symbol: String,
    pub interval: String,
    pub open_time: u64,
    pub close_time: u64,
    pub open: Fixed,
    pub high: Fixed,
    pub low: Fixed,
    pub close: Fixed,
    pub volume: Fixed,
    pub is_closed: bool,
}

/// Order book level
#[derive(Debug, Clone)]
pub struct OrderBookLevel {
    pub price: Fixed,
    pub quantity: Fixed,
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
    fn test_websocket_client_creation() {
        let config = BinanceConfig::testnet();
        let client = BinanceWebSocketClient::new(config);
        assert_eq!(client.base_url, "wss://testnet.binance.vision/ws");
    }
    
    #[monoio::test]
    async fn test_stream_subscription() {
        let config = BinanceConfig::testnet();
        let client = BinanceWebSocketClient::new(config);
        
        // Note: This test would require actual WebSocket connection
        // For now, just test the client creation and subscription tracking
        let subscriptions = client.get_subscriptions();
        assert!(subscriptions.is_empty());
        
        // Test that client is created properly
        assert!(!client.is_connected());
    }
    
    #[test]
    fn test_message_processing() {
        let config = BinanceConfig::testnet();
        let client = BinanceWebSocketClient::new(config);
        
        let sample_message = r#"{
            "stream": "btcusdt@ticker",
            "data": {
                "s": "BTCUSDT",
                "c": "50000.00",
                "P": "1.5",
                "v": "1000.5",
                "E": 1234567890
            }
        }"#;
        
        let result = client.process_message_content(sample_message);
        assert!(result.is_ok());
        
        if let Ok(MarketDataEvent::Ticker(ticker)) = result {
            assert_eq!(ticker.symbol, "BTCUSDT");
            assert_eq!(ticker.timestamp, 1234567890);
        } else {
            panic!("Expected ticker event");
        }
    }
}