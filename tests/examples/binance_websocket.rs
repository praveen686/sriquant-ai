//! Binance WebSocket streaming test
//! 
//! Demonstrates real-time market data streaming with SriQuant.ai

use sriquant_core::prelude::*;
use sriquant_exchanges::binance::{BinanceConfig, BinanceWebSocketClient};
use sriquant_exchanges::binance::websocket::{MarketDataEvent, TradeSide};
use tracing::{info, error};
use std::time::Duration;

#[monoio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logging();
    
    info!("🚀 Testing SriQuant.ai Binance WebSocket Streaming");
    
    // Create testnet WebSocket configuration
    let config = BinanceConfig::testnet();
    let mut ws_client = BinanceWebSocketClient::new(config);
    
    // Connect to WebSocket
    info!("🔌 Connecting to Binance WebSocket...");
    ws_client.connect().await?;
    info!("✅ WebSocket connected successfully");
    
    // Subscribe to real-time ticker data
    info!("📊 Subscribing to BTCUSDT ticker...");
    ws_client.subscribe_ticker("btcusdt").await?;
    
    // Subscribe to depth/order book updates
    info!("📋 Subscribing to BTCUSDT depth (5 levels)...");
    ws_client.subscribe_depth("btcusdt", Some(5)).await?;
    
    // Subscribe to trade stream
    info!("💰 Subscribing to BTCUSDT trades...");
    ws_client.subscribe_trades("btcusdt").await?;
    
    info!("🎯 Starting real-time data stream (will run for 30 seconds)...");
    info!("   Watch for live price updates, trades, and order book changes");
    
    // Stream real-time data for 30 seconds
    let start_time = nanos();
    let duration_ns = 30_000_000_000u64; // 30 seconds in nanoseconds
    let mut message_count = 0;
    
    while (nanos() - start_time) < duration_ns {
        match ws_client.receive_message().await {
            Ok(event) => {
                message_count += 1;
                
                // Log different types of market data
                match event {
                    MarketDataEvent::Ticker(ticker) => {
                        info!("🏷️  TICKER: {} = ${} (24h change: ${})", 
                            ticker.symbol, 
                            ticker.price, 
                            ticker.price_change
                        );
                    },
                    MarketDataEvent::Depth(depth) => {
                        let best_bid = depth.bids.get(0).map(|b| b.price.to_string()).unwrap_or("N/A".to_string());
                        let best_ask = depth.asks.get(0).map(|a| a.price.to_string()).unwrap_or("N/A".to_string());
                        info!("📊 DEPTH: {} - Best Bid: ${} | Best Ask: ${}", 
                            depth.symbol,
                            best_bid,
                            best_ask
                        );
                    },
                    MarketDataEvent::Trade(trade) => {
                        let side_str = match trade.side {
                            TradeSide::Buy => "BUY",
                            TradeSide::Sell => "SELL",
                        };
                        info!("💰 TRADE: {} {} @ ${} ({})", 
                            side_str,
                            trade.quantity, 
                            trade.price,
                            trade.symbol
                        );
                    },
                    MarketDataEvent::Kline(kline) => {
                        info!("📈 KLINE: {} - O: ${} H: ${} L: ${} C: ${}", 
                            kline.symbol, 
                            kline.open, 
                            kline.high, 
                            kline.low, 
                            kline.close
                        );
                    },
                    _ => {
                        info!("📨 Other market data event received");
                    }
                }
                
                // Add small delay to prevent flooding
                if message_count % 10 == 0 {
                    monoio::time::sleep(Duration::from_millis(100)).await;
                }
            },
            Err(e) => {
                error!("❌ WebSocket error: {}", e);
                break;
            }
        }
    }
    
    let elapsed_ms = (nanos() - start_time) / 1_000_000;
    info!("⚡ Streaming Statistics:");
    info!("   • Duration: {}ms", elapsed_ms);
    info!("   • Messages received: {}", message_count);
    info!("   • Average rate: {:.1} msg/sec", 
        message_count as f64 / (elapsed_ms as f64 / 1000.0)
    );
    
    // Clean disconnect
    info!("🔌 Disconnecting WebSocket...");
    ws_client.close().await?;
    info!("✅ WebSocket test completed successfully");
    
    Ok(())
}