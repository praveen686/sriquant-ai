//! Combined user stream and order placement demo
//! 
//! This example starts a user stream and then places orders to demonstrate real-time updates

use sriquant_core::prelude::*;
use sriquant_exchanges::binance::{BinanceConfig, BinanceRestClient, BinanceUserStreamClient, UserDataEvent, TradeSide};
use sriquant_exchanges::binance::rest::TestOrderParams;
use tracing::{info, error};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use monoio::time::sleep;
use std::time::Duration;

#[monoio::main(enable_timer = true)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Setup logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("🚀 Starting Binance User Stream Demo with Live Orders");
    
    // Load configuration
    let config = match BinanceConfig::testnet().with_env_credentials() {
        Ok(config) => {
            info!("✅ API credentials loaded");
            config
        }
        Err(e) => {
            error!("❌ Failed to load API credentials: {}", e);
            return Err(e.into());
        }
    };
    
    // Create REST client
    let rest_client = Arc::new(BinanceRestClient::new(config.clone()).await?);
    info!("✅ REST client initialized");
    
    // Create listen key
    info!("🔑 Creating listen key for user stream...");
    let listen_key = rest_client.create_listen_key().await?;
    info!("✅ Listen key created");
    
    // Flag to track if we should continue
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    let listen_key_clone = listen_key.clone();
    
    // Start user stream monitor in background
    let _user_stream_handle = monoio::spawn(async move {
        let mut ws_client = BinanceUserStreamClient::new(config.clone());
        
        match ws_client.connect(&listen_key_clone).await {
            Ok(_) => {
                info!("✅ User stream connected!");
                info!("📊 Monitoring for events...\n");
            }
            Err(e) => {
                error!("❌ Failed to connect user stream: {}", e);
                return;
            }
        }
        
        // Monitor events
        while running_clone.load(Ordering::Relaxed) {
            match ws_client.receive_event().await {
                Ok(event) => {
                    match event {
                        UserDataEvent::OrderUpdate(order) => {
                            let side_emoji = match order.side {
                                TradeSide::Buy => "🟢",
                                TradeSide::Sell => "🔴",
                            };
                            
                            info!("════════════════════════════════════════");
                            info!("{} ORDER UPDATE RECEIVED!", side_emoji);
                            info!("   Time: {}", order.event_time);
                            info!("   Symbol: {}", order.symbol);
                            info!("   Order ID: {}", order.order_id);
                            info!("   Client Order ID: {}", order.client_order_id);
                            info!("   Side: {:?} | Type: {}", order.side, order.order_type);
                            info!("   Price: {} | Quantity: {}", order.order_price, order.order_quantity);
                            info!("   Status: {} | Execution: {}", order.order_status, order.execution_type);
                            info!("   Filled: {} / {}", order.cumulative_filled_quantity, order.order_quantity);
                            info!("════════════════════════════════════════\n");
                        },
                        UserDataEvent::AccountUpdate(account) => {
                            info!("👤 Account update: {} balances", account.balances.len());
                        },
                        UserDataEvent::BalanceUpdate(balance) => {
                            info!("💰 Balance update: {} {}", balance.asset, balance.balance_delta);
                        }
                    }
                },
                Err(e) => {
                    error!("User stream error: {}", e);
                    break;
                }
            }
        }
        
        info!("User stream monitor stopped");
    });
    
    // Wait for stream to connect
    info!("⏳ Waiting for user stream to connect...");
    sleep(Duration::from_secs(3)).await;
    
    // Now place some orders
    info!("\n🎯 Starting order placement sequence...\n");
    
    // Get current price
    let ticker = rest_client.get_symbol_price_ticker("BTCUSDT").await?;
    let current_price = Fixed::from_str_exact(&ticker.price).unwrap_or(Fixed::ZERO);
    info!("📈 Current BTCUSDT price: ${}", current_price);
    
    // Order 1: Buy order 10% below market
    info!("\n1️⃣ Placing BUY order (10% below market)...");
    let buy_price = (current_price * Fixed::from_f64(0.90).unwrap()).round_dp(2);
    let buy_price_str = buy_price.to_string();
    
    let buy_params = TestOrderParams {
        symbol: "BTCUSDT",
        side: "BUY",
        order_type: "LIMIT",
        quantity: Some("0.001"),
        price: Some(&buy_price_str),
        time_in_force: Some("GTC"),
        stop_price: None,
        iceberg_qty: None,
    };
    
    match rest_client.new_order(&buy_params).await {
        Ok(order) => {
            info!("✅ Buy order placed! ID: {}", order.order_id);
            
            // Wait to see the NEW event
            sleep(Duration::from_secs(2)).await;
            
            // Cancel it
            info!("🚫 Canceling buy order...");
            match rest_client.cancel_order("BTCUSDT", order.order_id).await {
                Ok(_) => info!("✅ Buy order canceled!"),
                Err(e) => error!("Failed to cancel: {}", e),
            }
        }
        Err(e) => error!("Failed to place buy order: {}", e),
    }
    
    // Wait a bit
    sleep(Duration::from_secs(3)).await;
    
    // Order 2: Sell order 10% above market
    info!("\n2️⃣ Placing SELL order (10% above market)...");
    let sell_price = (current_price * Fixed::from_f64(1.10).unwrap()).round_dp(2);
    let sell_price_str = sell_price.to_string();
    
    let sell_params = TestOrderParams {
        symbol: "BTCUSDT",
        side: "SELL",
        order_type: "LIMIT",
        quantity: Some("0.001"),
        price: Some(&sell_price_str),
        time_in_force: Some("GTC"),
        stop_price: None,
        iceberg_qty: None,
    };
    
    match rest_client.new_order(&sell_params).await {
        Ok(order) => {
            info!("✅ Sell order placed! ID: {}", order.order_id);
            
            // Let it sit for a moment
            sleep(Duration::from_secs(3)).await;
            
            // Cancel it
            info!("🚫 Canceling sell order...");
            match rest_client.cancel_order("BTCUSDT", order.order_id).await {
                Ok(_) => info!("✅ Sell order canceled!"),
                Err(e) => error!("Failed to cancel: {}", e),
            }
        }
        Err(e) => error!("Failed to place sell order: {}", e),
    }
    
    // Wait to see final events
    sleep(Duration::from_secs(2)).await;
    
    info!("\n✅ Order sequence complete!");
    info!("📊 Check the events above to see real-time updates");
    info!("⏳ Keeping stream open for 10 more seconds...\n");
    
    // Keep running for a bit more
    sleep(Duration::from_secs(10)).await;
    
    // Cleanup
    info!("🛑 Shutting down...");
    running.store(false, Ordering::Relaxed);
    
    // Close listen key
    rest_client.close_listen_key(&listen_key).await?;
    
    info!("✅ Demo complete!");
    
    Ok(())
}