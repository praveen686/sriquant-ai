//! Place test orders on Binance testnet to trigger user stream events
//!
//! This script places various types of orders to demonstrate user stream functionality

use sriquant_core::prelude::*;
use sriquant_exchanges::binance::{BinanceConfig, BinanceRestClient};
use sriquant_exchanges::binance::rest::TestOrderParams;
use tracing::{info, error};

#[monoio::main(enable_timer = true)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Setup logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("🚀 Starting Binance Testnet Order Placement Script");
    
    // Load configuration
    let config = match BinanceConfig::testnet().with_env_credentials() {
        Ok(config) => {
            info!("✅ API credentials loaded from environment");
            config
        }
        Err(e) => {
            error!("❌ Failed to load API credentials: {}", e);
            return Err(e.into());
        }
    };
    
    // Create REST client
    let client = BinanceRestClient::new(config).await?;
    info!("✅ REST client initialized");
    
    // Get account info first
    info!("\n📊 Fetching account information...");
    let account = client.get_account_info().await?;
    info!("✅ Account info retrieved");
    
    // Show USDT balance
    for balance in &account.balances {
        if balance.asset == "USDT" {
            let free = Fixed::from_str_exact(&balance.free).unwrap_or(Fixed::ZERO);
            let locked = Fixed::from_str_exact(&balance.locked).unwrap_or(Fixed::ZERO);
            if free > Fixed::ZERO || locked > Fixed::ZERO {
                info!("💰 USDT Balance: Free={} Locked={}", free, locked);
            }
        }
    }
    
    // Get current BTC price
    info!("\n💱 Getting current BTCUSDT price...");
    let ticker = client.get_symbol_price_ticker("BTCUSDT").await?;
    let current_price = Fixed::from_str_exact(&ticker.price).unwrap_or(Fixed::ZERO);
    info!("📈 Current BTCUSDT price: ${}", current_price);
    
    // Calculate order prices - round to 2 decimal places
    let buy_price = (current_price * Fixed::from_f64(0.95).unwrap()).round_dp(2); // 5% below market
    let sell_price = (current_price * Fixed::from_f64(1.05).unwrap()).round_dp(2); // 5% above market
    
    info!("\n🎯 Order Strategy:");
    info!("   Buy Price: ${} (5% below market)", buy_price);
    info!("   Sell Price: ${} (5% above market)", sell_price);
    
    // Place orders with delay between them
    info!("\n📝 Placing test orders...");
    
    // 1. Place a limit buy order
    info!("\n1️⃣ Placing LIMIT BUY order...");
    let buy_price_str = buy_price.to_string();
    let buy_order_params = TestOrderParams {
        symbol: "BTCUSDT",
        side: "BUY",
        order_type: "LIMIT",
        quantity: Some("0.001"),
        price: Some(&buy_price_str),
        time_in_force: Some("GTC"),
        stop_price: None,
        iceberg_qty: None,
    };
    match client.new_order(&buy_order_params).await {
        Ok(order) => {
            info!("✅ Buy order placed successfully!");
            info!("   Order ID: {}", order.order_id);
            info!("   Status: {:?}", order.status);
            info!("   Price: ${}", order.price);
            info!("   Quantity: {}", order.orig_qty);
        }
        Err(e) => {
            error!("❌ Failed to place buy order: {}", e);
        }
    }
    
    // Wait a bit
    monoio::time::sleep(std::time::Duration::from_secs(2)).await;
    
    // 2. Place a limit sell order
    info!("\n2️⃣ Placing LIMIT SELL order...");
    let sell_price_str = sell_price.to_string();
    let sell_order_params = TestOrderParams {
        symbol: "BTCUSDT",
        side: "SELL",
        order_type: "LIMIT",
        quantity: Some("0.001"),
        price: Some(&sell_price_str),
        time_in_force: Some("GTC"),
        stop_price: None,
        iceberg_qty: None,
    };
    match client.new_order(&sell_order_params).await {
        Ok(order) => {
            info!("✅ Sell order placed successfully!");
            info!("   Order ID: {}", order.order_id);
            info!("   Status: {:?}", order.status);
            info!("   Price: ${}", order.price);
            info!("   Quantity: {}", order.orig_qty);
        }
        Err(e) => {
            error!("❌ Failed to place sell order: {}", e);
        }
    }
    
    // Wait a bit
    monoio::time::sleep(std::time::Duration::from_secs(2)).await;
    
    // 3. Place a market buy order (small amount)
    info!("\n3️⃣ Placing MARKET BUY order...");
    let market_order_params = TestOrderParams {
        symbol: "BTCUSDT",
        side: "BUY",
        order_type: "MARKET",
        quantity: Some("0.0001"),
        price: None,
        time_in_force: None,
        stop_price: None,
        iceberg_qty: None,
    };
    match client.new_order(&market_order_params).await {
        Ok(order) => {
            info!("✅ Market buy order executed!");
            info!("   Order ID: {}", order.order_id);
            info!("   Status: {:?}", order.status);
            info!("   Executed Qty: {}", order.executed_qty);
            info!("   Cumulative Quote: {}", order.cumulative_quote_qty);
        }
        Err(e) => {
            error!("❌ Failed to place market order: {}", e);
        }
    }
    
    // Wait a bit
    monoio::time::sleep(std::time::Duration::from_secs(3)).await;
    
    // 4. Get open orders
    info!("\n📋 Fetching open orders...");
    match client.open_orders(Some("BTCUSDT")).await {
        Ok(orders) => {
            info!("📊 Found {} open orders", orders.len());
            for order in &orders {
                info!("   • {} {} {} @ ${} (ID: {})", 
                    order.symbol, order.side, order.orig_qty, order.price, order.order_id);
            }
            
            // 5. Cancel one order if exists
            if let Some(order_to_cancel) = orders.first() {
                info!("\n❌ Canceling order ID: {}", order_to_cancel.order_id);
                match client.cancel_order("BTCUSDT", order_to_cancel.order_id).await {
                    Ok(canceled) => {
                        info!("✅ Order canceled successfully!");
                        info!("   Status: {:?}", canceled.status);
                    }
                    Err(e) => {
                        error!("❌ Failed to cancel order: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to get open orders: {}", e);
        }
    }
    
    info!("\n✅ Test order placement completed!");
    info!("💡 Check your user stream example to see the real-time updates!");
    
    Ok(())
}