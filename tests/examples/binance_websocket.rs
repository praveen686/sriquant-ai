//! Binance WebSocket streaming test
//! 
//! Demonstrates real-time market data streaming with SriQuant.ai

use sriquant_core::prelude::*;
use sriquant_exchanges::binance::{BinanceConfig, BinanceWebSocketClient};
use sriquant_exchanges::binance::websocket::{MarketDataEvent, TradeSide};
use tracing::{info, error};

#[monoio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    // Simple logging setup to avoid complex initialization
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("🚀 Testing SriQuant.ai Binance WebSocket Streaming");
    
    // Create testnet WebSocket configuration with environment credentials
    let config_result = BinanceConfig::testnet().with_env_credentials();
    
    let (config, has_credentials) = match config_result {
        Ok(config) => {
            info!("🔑 API credentials loaded from environment");
            (config, true)
        }
        Err(_) => {
            info!("🌐 No API credentials in environment - using testnet config");
            (BinanceConfig::testnet(), false)
        }
    };
    
    if has_credentials {
        info!("🔑 API credentials found - you can use REST API for account operations");
        info!("📊 Testing market data streams...");
    } else {
        info!("🌐 No API credentials - testing public market data streams only");
    }
    
    // Connect to multiple public streams
    let mut ws_client = BinanceWebSocketClient::new(config);
    let streams = vec![
        "btcusdt@ticker",
        "ethusdt@ticker", 
        "btcusdt@depth5@100ms",
        "btcusdt@trade"
    ];
    ws_client.connect_multi_stream(streams).await?;
    info!("✅ WebSocket connected successfully with multiple streams");
    
    // Test market data streams
    test_market_data_streams(&mut ws_client, 30).await?;
    
    Ok(())
}

async fn test_market_data_streams(ws_client: &mut BinanceWebSocketClient, duration_seconds: u64) -> Result<(), Box<dyn std::error::Error>> {
    info!("🎯 Starting market data streams (will run for {} seconds)...", duration_seconds);
    info!("   Watch for live price updates, trades, and order book changes");
    
    // Stream real-time data 
    let start_time = nanos();
    let duration_ns = duration_seconds * 1_000_000_000u64;
    let mut message_count = 0;
    
    while (nanos() - start_time) < duration_ns {
        match ws_client.receive_message().await {
            Ok(event) => {
                message_count += 1;
                
                // Log different types of market data with enhanced formatting
                match event {
                    MarketDataEvent::Ticker(ticker) => {
                        let change_emoji = if ticker.price_change >= Fixed::ZERO { "📈" } else { "📉" };
                        info!("{} TICKER: {} = ${} (24h: {}${:.3})", 
                            change_emoji,
                            ticker.symbol, 
                            ticker.price, 
                            if ticker.price_change >= Fixed::ZERO { "+" } else { "" },
                            ticker.price_change
                        );
                    },
                    MarketDataEvent::Depth(depth) => {
                        let best_bid = depth.bids.get(0).map(|b| b.price.to_string()).unwrap_or("N/A".to_string());
                        let best_ask = depth.asks.get(0).map(|a| a.price.to_string()).unwrap_or("N/A".to_string());
                        let spread = if let (Some(bid), Some(ask)) = (depth.bids.get(0), depth.asks.get(0)) {
                            format!("${:.2}", ask.price - bid.price)
                        } else {
                            "N/A".to_string()
                        };
                        info!("📊 DEPTH: {} - Bid: ${} | Ask: ${} | Spread: {}", 
                            depth.symbol,
                            best_bid,
                            best_ask,
                            spread
                        );
                    },
                    MarketDataEvent::Trade(trade) => {
                        let side_emoji = match trade.side {
                            TradeSide::Buy => "🟢",
                            TradeSide::Sell => "🔴",
                        };
                        let side_str = match trade.side {
                            TradeSide::Buy => "BUY",
                            TradeSide::Sell => "SELL",
                        };
                        info!("{} TRADE: {} {} {} @ ${} | ID: {}", 
                            side_emoji,
                            trade.symbol,
                            side_str,
                            trade.quantity, 
                            trade.price,
                            trade.trade_id
                        );
                    },
                    MarketDataEvent::Kline(kline) => {
                        let status = if kline.is_closed { "CLOSED" } else { "LIVE" };
                        info!("📈 KLINE: {} ({}) - O:${} H:${} L:${} C:${} V:{}", 
                            kline.symbol, 
                            status,
                            kline.open, 
                            kline.high, 
                            kline.low, 
                            kline.close,
                            kline.volume
                        );
                    }
                }
                
                // Add small delay to prevent flooding (using simple loop delay)
                if message_count % 10 == 0 {
                    for _ in 0..1000000 { /* Simple CPU delay */ }
                }
            },
            Err(e) => {
                error!("❌ WebSocket error: {}", e);
                break;
            }
        }
    }
    
    let elapsed_ms = (nanos() - start_time) / 1_000_000;
    info!("⚡ Market Data Statistics:");
    info!("   • Duration: {}ms", elapsed_ms);
    info!("   • Messages received: {}", message_count);
    if elapsed_ms > 0 {
        info!("   • Average rate: {:.1} msg/sec", 
            message_count as f64 / (elapsed_ms as f64 / 1000.0)
        );
    }
    
    // Clean disconnect
    info!("🔌 Disconnecting WebSocket...");
    ws_client.close().await?;
    info!("✅ Market data test completed successfully");
    
    Ok(())
}