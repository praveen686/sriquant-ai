//! Simple connectivity test for SriQuant.ai
use sriquant_core::prelude::*;
use sriquant_exchanges::binance::{BinanceConfig, BinanceExchange};
use tracing::{info, error};

#[monoio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    init_logging();
    
    info!("🚀 Testing SriQuant.ai Binance connectivity...");
    
    let config = BinanceConfig::testnet()
        .with_env_credentials()?;
    
    let mut exchange = BinanceExchange::new(config).await?;
    exchange.init_rest().await?;
    
    // Test basic connectivity
    let latency = exchange.ping().await?;
    info!("✅ Ping successful: {}μs", latency);
    
    // Get server time
    let server_time = exchange.server_time().await?;
    info!("⏰ Server time: {}", server_time);
    
    // Get exchange info
    let exchange_info = exchange.exchange_info().await?;
    info!("📊 Exchange has {} symbols", exchange_info.symbols.len());
    
    info!("✅ All tests passed!");
    Ok(())
}