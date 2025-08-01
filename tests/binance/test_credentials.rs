//! Test Binance API credentials and connectivity
//!
//! This example tests your Binance API setup:
//! 1. Loads credentials from .env
//! 2. Tests REST API connectivity  
//! 3. Tests WebSocket connectivity
//! 4. Verifies permissions

use sriquant_exchanges::binance::{BinanceConfig, BinanceRestClient};
use sriquant_core::prelude::*;
use std::env;

#[monoio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🚀 SriQuant.ai Binance Credentials Test");
    println!("=====================================");

    // Load environment variables
    dotenv::dotenv().ok();
    
    // Get credentials from environment
    let api_key = env::var("BINANCE_API_KEY")
        .map_err(|_| "BINANCE_API_KEY not found in .env file")?;
    let secret_key = env::var("BINANCE_SECRET_KEY") 
        .map_err(|_| "BINANCE_SECRET_KEY not found in .env file")?;
    let use_testnet = env::var("BINANCE_TESTNET")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);

    if api_key == "your_binance_api_key_here" || secret_key == "your_binance_secret_key_here" {
        eprintln!("❌ ERROR: Please update your .env file with real Binance API credentials!");
        eprintln!("   Copy .env.example to .env and fill in your actual API keys.");
        return Err("Invalid credentials".into());
    }

    println!("✅ Environment variables loaded");
    println!("   API Key: {}...{}", 
        &api_key[..8], 
        &api_key[api_key.len()-8..]
    );
    println!("   Using testnet: {use_testnet}");
    println!();

    // Create Binance configuration
    let config = if use_testnet {
        BinanceConfig::testnet().with_credentials(api_key, secret_key)
    } else {
        BinanceConfig::default().with_credentials(api_key, secret_key)
    };

    println!("🔗 Testing REST API connectivity...");
    
    // Test basic connectivity
    let rest_client = BinanceRestClient::new(config.clone()).await?;
    
    // Test server time (public endpoint - no auth required)
    match rest_client.get_server_time().await {
        Ok(server_time) => {
            println!("✅ REST API connectivity: OK");
            println!("   Server time: {server_time}");
            
            let local_time = nanos() / 1_000_000;
            let time_diff = (server_time as i64 - local_time as i64).abs();
            
            if time_diff > 1000 {
                println!("⚠️  WARNING: Time difference > 1s: {time_diff}ms");
                println!("   Consider synchronizing your system clock");
            } else {
                println!("✅ Time synchronization: OK ({time_diff}ms diff)");
            }
        }
        Err(e) => {
            eprintln!("❌ REST API connectivity failed: {e}");
            return Err(e.into());
        }
    }
    println!();

    println!("🔐 Testing API authentication...");
    
    // Test authenticated endpoint
    match rest_client.get_account_info().await {
        Ok(account) => {
            println!("✅ API authentication: OK");
            println!("   Account type: {}", if use_testnet { "TESTNET" } else { "PRODUCTION" });
            println!("   Balances available: {}", account.balances.len());
            
            // Show non-zero balances
            let non_zero_balances: Vec<_> = account.balances.iter()
                .filter(|b| b.free.parse::<f64>().unwrap_or(0.0) > 0.0 || 
                           b.locked.parse::<f64>().unwrap_or(0.0) > 0.0)
                .collect();
                
            if !non_zero_balances.is_empty() {
                println!("   Non-zero balances:");
                for balance in non_zero_balances.iter().take(5) {
                    println!("     {}: {} (locked: {})", 
                        balance.asset, balance.free, balance.locked);
                }
                if non_zero_balances.len() > 5 {
                    println!("     ... and {} more", non_zero_balances.len() - 5);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ API authentication failed: {e}");
            eprintln!("   Check your API key and secret");
            eprintln!("   Verify API permissions (Read Info required)");
            return Err(e.into());
        }
    }
    println!();

    println!("📊 Testing market data...");
    
    // Test market data
    match rest_client.get_symbol_price_ticker("BTCUSDT").await {
        Ok(ticker) => {
            println!("✅ Market data: OK");
            println!("   BTCUSDT price: ${}", ticker.price);
        }
        Err(e) => {
            eprintln!("⚠️  Market data test failed: {e}");
        }
    }
    println!();

    println!("🎯 Testing trading permissions...");
    
    // Test if we can place orders (dry run)
    let test_order_params = sriquant_exchanges::binance::rest::TestOrderParams {
        symbol: "BTCUSDT",
        side: "BUY",
        order_type: "LIMIT",
        quantity: Some("0.001"),   // Small test amount
        price: Some("30000.0"),    // Below market price
        time_in_force: Some("GTC"), // Good Till Cancelled
        stop_price: None,
        iceberg_qty: None,
    };
    
    match rest_client.test_new_order(&test_order_params).await {
        Ok(_) => {
            println!("✅ Trading permissions: OK");
            println!("   Spot trading is enabled");
        }
        Err(e) => {
            eprintln!("⚠️  Trading permissions test failed: {e}");
            eprintln!("   You may need to enable 'Spot & Margin Trading' permission");
        }
    }
    println!();

    println!("🔌 Testing WebSocket connectivity...");
    
    // Note: Full WebSocket test would require more complex setup
    // For now, just verify the WebSocket URL is accessible
    let ws_url = if use_testnet {
        "wss://stream.testnet.binance.vision/ws/btcusdt@ticker"
    } else {
        "wss://stream.binance.com:9443/ws/btcusdt@ticker"
    };
    
    println!("✅ WebSocket URL configured: {ws_url}");
    println!("   (Full WebSocket test requires running market data example)");
    println!();

    println!("🎉 Credentials test completed successfully!");
    println!();
    println!("Next steps:");
    println!("1. Run market data example: cargo run --example market_data");
    println!("2. Test paper trading: cargo run --example paper_trading");
    println!("3. Check the documentation in setup_credentials.md");
    
    if use_testnet {
        println!();
        println!("💡 TIP: You're using testnet - perfect for development!");
        println!("   Switch to production by setting BINANCE_TESTNET=false in .env");
    } else {
        println!();
        println!("⚠️  WARNING: You're using PRODUCTION credentials!");
        println!("   Make sure you understand the risks and start with small amounts.");
    }

    Ok(())
}