//! Simple order placement on Binance testnet for user stream testing
//!
//! Minimal example that just places orders without fetching full account info

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
    
    info!("🚀 Starting Simple Binance Order Placement");
    
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
    let client = BinanceRestClient::new(config).await?;
    info!("✅ REST client initialized");
    
    // Get current price
    info!("\n💱 Getting BTCUSDT price...");
    let ticker = client.get_symbol_price_ticker("BTCUSDT").await?;
    let current_price = Fixed::from_str_exact(&ticker.price).unwrap_or(Fixed::ZERO);
    info!("📈 Current price: ${}", current_price);
    
    // Calculate a buy price 10% below market (to ensure it doesn't execute immediately)
    let buy_price = (current_price * Fixed::from_f64(0.90).unwrap()).round_dp(2);
    info!("🎯 Buy order price: ${} (10% below market)", buy_price);
    
    // Place a limit buy order
    info!("\n📝 Placing LIMIT BUY order...");
    let buy_price_str = buy_price.to_string();
    let order_params = TestOrderParams {
        symbol: "BTCUSDT",
        side: "BUY",
        order_type: "LIMIT",
        quantity: Some("0.001"),  // Small amount
        price: Some(&buy_price_str),
        time_in_force: Some("GTC"),
        stop_price: None,
        iceberg_qty: None,
    };
    
    match client.new_order(&order_params).await {
        Ok(order) => {
            info!("✅ Order placed successfully!");
            info!("   Order ID: {}", order.order_id);
            info!("   Client Order ID: {}", order.client_order_id);
            info!("   Status: {:?}", order.status);
            info!("   Symbol: {}", order.symbol);
            info!("   Side: {}", order.side);
            info!("   Price: ${}", order.price);
            info!("   Quantity: {}", order.orig_qty);
            info!("   Time: {}", order.transact_time);
            
            // Wait a moment then cancel it
            info!("\n⏳ Waiting 5 seconds before canceling...");
            monoio::time::sleep(std::time::Duration::from_secs(5)).await;
            
            info!("\n❌ Canceling the order...");
            match client.cancel_order("BTCUSDT", order.order_id).await {
                Ok(canceled) => {
                    info!("✅ Order canceled!");
                    info!("   Status: {:?}", canceled.status);
                }
                Err(e) => {
                    error!("❌ Failed to cancel: {}", e);
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to place order: {}", e);
            error!("   Make sure you have USDT balance in your testnet account");
        }
    }
    
    info!("\n✅ Done! Check your user stream for events.");
    
    Ok(())
}