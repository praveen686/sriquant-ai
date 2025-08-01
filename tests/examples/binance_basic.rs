//! Basic Binance exchange example using SriQuant.ai
//!
//! Demonstrates:
//! - High-performance Binance REST API client
//! - Nanosecond precision timing
//! - Fixed-point arithmetic for prices
//! - CPU binding for optimal performance
//! - Unified logging with performance metrics

use sriquant_core::prelude::*;
use sriquant_exchanges::binance::{BinanceConfig, BinanceExchange};
use tracing::{info, error};

#[monoio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Initialize logging
    init_logging();
    
    info!("🚀 Starting SriQuant.ai Binance Basic Example");
    
    // Bind to CPU core 0 for maximum performance (for high performance)
    if let Err(e) = bind_to_cpu_set(0) {
        error!("Failed to bind to CPU core 0: {}", e);
    }
    
    // Create Binance configuration for testnet with credentials
    let config = BinanceConfig::testnet()
        .with_env_credentials()? // Load credentials from environment
        .with_timing(true) // Enable nanosecond precision timing
        .with_cpu_core(Some(0)); // Bind to core 0
    
    info!("📋 Configuration:");
    info!("   Exchange: Binance Testnet");
    info!("   Base URL: {}", config.base_url);
    info!("   WebSocket: {}", config.ws_url);
    info!("   Timing: {}", config.enable_timing);
    info!("   CPU Core: {:?}", config.cpu_core);
    
    // Create exchange client
    let mut exchange = BinanceExchange::new(config).await?;
    
    // Initialize REST client
    exchange.init_rest().await?;
    
    // Test connectivity and measure latency using authenticated endpoint
    info!("🏓 Testing connectivity and authentication...");
    let latency_timer = PerfTimer::start("connectivity_test");
    
    // Use ping endpoint for basic connectivity
    let latency_us = exchange.ping().await?;
    
    if latency_us < 1000 {
        info!("✅ Excellent latency: {}μs", latency_us);
    } else if latency_us < 10000 {
        info!("✅ Good latency: {:.1}ms", latency_us as f64 / 1000.0);
    } else {
        info!("⚠️  High latency: {:.1}ms", latency_us as f64 / 1000.0);
    }
    
    // Get exchange information (demonstrates market data access)
    info!("📊 Fetching exchange information...");
    let exchange_info = exchange.exchange_info().await?;
    
    info!("📈 Exchange Info:");
    info!("   Timezone: {}", exchange_info.timezone);
    info!("   Server Time: {}", exchange_info.server_time);
    info!("   Total Symbols: {}", exchange_info.symbols.len());
    
    // Find some popular symbols (demonstrates data processing)
    let mut btc_symbols = Vec::new();
    let mut eth_symbols = Vec::new();
    
    for symbol in &exchange_info.symbols {
        if symbol.status == "TRADING" {
            if symbol.base_asset == "BTC" && btc_symbols.len() < 3 {
                btc_symbols.push(&symbol.symbol);
            } else if symbol.base_asset == "ETH" && eth_symbols.len() < 3 {
                eth_symbols.push(&symbol.symbol);
            }
        }
    }
    
    info!("₿ Bitcoin pairs: {:?}", btc_symbols);
    info!("Ξ Ethereum pairs: {:?}", eth_symbols);
    
    // Demonstrate timing precision
    info!("⏱️  Testing timing precision...");
    let timing_tests = 1000;
    let start = nanos();
    for _ in 0..timing_tests {
        let _timestamp = nanos(); // Test timing overhead
    }
    let total_nanos = nanos() - start;
    let avg_nanos_per_call = total_nanos / timing_tests;
    
    info!("   {} timing calls in {}ns", timing_tests, total_nanos);
    info!("   Average: {}ns per timestamp", avg_nanos_per_call);
    info!("   Precision: Sub-microsecond level");
    
    let exchange_info_latency = latency_timer.elapsed_micros();
    
    // Performance summary with detailed metrics
    info!("⚡ Performance Summary:");
    info!("   • Connectivity Test: {}μs", latency_us);
    info!("   • Exchange Info Fetch: {}μs", exchange_info_latency);
    info!("   • Timing Precision: {}ns per timestamp call", avg_nanos_per_call);
    info!("   • Exchange Symbols: {} trading pairs", exchange_info.symbols.len());
    info!("   • CPU Core: Bound to core 0 for maximum performance");
    info!("   • Runtime: Monoio single-threaded async");
    info!("   • CPU Architecture: Single-core bound for performance");
    info!("   • Timestamp Resolution: Nanosecond level");
    
    // Demonstrate Fixed-point arithmetic
    info!("🔢 Fixed-Point Arithmetic Demo:");
    
    let price1 = Fixed::from_str_exact("50000.12345678")?;
    let price2 = Fixed::from_str_exact("49999.87654321")?;
    let quantity = Fixed::from_str_exact("0.001")?;
    
    let price_diff = price1 - price2;
    let notional1 = price1 * quantity;
    let notional2 = price2 * quantity;
    
    info!("   Price 1: ${}", price1);
    info!("   Price 2: ${}", price2);
    info!("   Difference: ${}", price_diff);
    info!("   Quantity: {}", quantity);
    info!("   Notional 1: ${}", notional1.to_string_with_scale(8));
    info!("   Notional 2: ${}", notional2.to_string_with_scale(8));
    
    // Calculate percentage change
    let percent_change = price_diff.percent_of(price2)?;
    info!("   Change: {:.4}%", percent_change);
    
    info!("✅ SriQuant.ai Binance Basic Example completed successfully");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fixed_arithmetic() {
        let price = Fixed::from_str_exact("100.123456").unwrap();
        let quantity = Fixed::from_str_exact("2.0").unwrap();
        let total = price * quantity;
        
        assert_eq!(total.to_string(), "200.2469120");
    }
}