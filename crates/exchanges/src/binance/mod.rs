//! Binance exchange integration with high-performance architecture
//!
//! High-performance Binance client using monoio for single-threaded async performance,
//! precision timing, and exact decimal arithmetic.

pub mod rest;
pub mod auth;
pub mod types;
pub mod websocket;
pub mod user_stream;
pub mod connection;

use crate::errors::{ExchangeError, Result};
use sriquant_core::{PerfTimer, nanos};
use tracing::info;

// Re-export types from submodules
pub use rest::{BinanceConfig, ExchangeInfo, SymbolInfo, BinanceRestClient};
pub use auth::{BinanceCredentials, BinanceSigner};
pub use types::*;
pub use websocket::BinanceWebSocketClient;
pub use user_stream::{BinanceUserStreamClient, UserDataEvent, AccountUpdateEvent, BalanceUpdateEvent, OrderUpdateEvent, BalanceInfo, TradeSide};
pub use connection::ConnectionManager;


/// High-performance Binance exchange client
/// 
/// High-performance architecture:
/// - Single-threaded async with monoio
/// - Nanosecond precision timing  
/// - Fixed-point arithmetic
/// - CPU core binding
pub struct BinanceExchange {
    config: BinanceConfig,
    rest_client: Option<BinanceRestClient>,
    #[allow(dead_code)] // Will be used when authenticated endpoints are implemented
    signer: Option<BinanceSigner>,
    websocket_client: Option<BinanceWebSocketClient>,
}

impl BinanceExchange {
    /// Create a new Binance exchange client
    pub async fn new(config: BinanceConfig) -> Result<Self> {
        info!("🚀 Initializing Binance exchange");
        info!("   Base URL: {}", config.base_url);
        info!("   WebSocket: {}", config.ws_url);
        info!("   Testnet: {}", config.testnet);
        info!("   Timing: {}", config.enable_timing);
        info!("   CPU Core: {:?}", config.cpu_core);
        
        // Initialize signer if credentials are available
        let signer = if !config.api_key.is_empty() && !config.api_secret.is_empty() {
            let credentials = BinanceCredentials::new(config.api_key.clone(), config.api_secret.clone());
            Some(BinanceSigner::new(credentials)?)
        } else {
            None
        };
        
        Ok(Self {
            config,
            rest_client: None,
            signer,
            websocket_client: None,
        })
    }
    
    /// Initialize REST client
    pub async fn init_rest(&mut self) -> Result<()> {
        let client = BinanceRestClient::new(self.config.clone()).await?;
        self.rest_client = Some(client);
        info!("✅ Binance REST client initialized");
        Ok(())
    }
    
    /// Initialize WebSocket streaming
    pub async fn init_websocket(&mut self) -> Result<()> {
        info!("🌐 Initializing Binance WebSocket");
        let mut ws_client = BinanceWebSocketClient::new(self.config.clone());
        ws_client.connect().await?;
        self.websocket_client = Some(ws_client);
        info!("✅ Binance WebSocket client initialized and connected");
        Ok(())
    }
    
    /// Get exchange information
    pub async fn exchange_info(&self) -> Result<ExchangeInfo> {
        let timer = PerfTimer::start("binance_exchange_info");
        let client = self.rest_client.as_ref()
            .ok_or_else(|| ExchangeError::ClientNotInitialized("REST client not initialized".to_string()))?;
        let info = client.exchange_info().await?;
        timer.log_elapsed();
        Ok(info)
    }
    
    /// Test connectivity and measure latency
    pub async fn ping(&self) -> Result<u64> {
        let start = nanos();
        let client = self.rest_client.as_ref()
            .ok_or_else(|| ExchangeError::ClientNotInitialized("REST client not initialized".to_string()))?;
        client.ping().await?;
        let latency_nanos = nanos() - start;
        
        let latency_micros = latency_nanos / 1000;
        info!("🏓 Binance ping: {}μs", latency_micros);
        
        Ok(latency_micros)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_binance_config_creation() {
        let config = BinanceConfig::default();
        assert_eq!(config.base_url, "https://api.binance.com");
        assert!(!config.testnet);
        assert_eq!(config.timeout_ms, 5000);
    }
    
    #[test]
    fn test_testnet_config() {
        let config = BinanceConfig::testnet();
        assert!(config.testnet);
        assert!(config.base_url.contains("testnet"));
    }
    
    #[test]
    fn test_config_builder() {
        let config = BinanceConfig::default()
            .with_credentials("key".to_string(), "secret".to_string())
            .with_timing(false)
            .with_cpu_core(Some(2));
            
        assert_eq!(config.api_key, "key");
        assert_eq!(config.api_secret, "secret");
        assert!(!config.enable_timing);
        assert_eq!(config.cpu_core, Some(2));
    }
}