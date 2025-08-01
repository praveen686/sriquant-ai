//! Binance REST API client using monoio for maximum performance
//!
//! High-performance architecture:
//! - Single-threaded async with monoio
//! - Nanosecond precision timing for latency measurement
//! - Efficient connection reuse
//! - Fixed-point arithmetic for price calculations

use crate::errors::{ExchangeError, Result};
use crate::http::MonoioHttpsClient;
use crate::binance::auth::BinanceAuth;
use sriquant_core::prelude::*;

use tracing::{debug, info};
use serde_json::Value;
use url::Url;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parameters for test order request
#[derive(Debug, Clone)]
pub struct TestOrderParams<'a> {
    pub symbol: &'a str,
    pub side: &'a str,
    pub order_type: &'a str,
    pub quantity: Option<&'a str>,
    pub price: Option<&'a str>,
    pub time_in_force: Option<&'a str>,
    pub stop_price: Option<&'a str>,
    pub iceberg_qty: Option<&'a str>,
}

/// Binance exchange configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceConfig {
    pub api_key: String,
    pub api_secret: String,
    pub base_url: String,
    pub ws_url: String,
    pub testnet: bool,
    pub timeout_ms: u64,
    pub enable_timing: bool,
    pub cpu_core: Option<usize>,
}

impl Default for BinanceConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_secret: String::new(),
            base_url: "https://api.binance.com".to_string(),
            ws_url: "wss://stream.binance.com:9443".to_string(),
            testnet: false,
            timeout_ms: 5000,
            enable_timing: true,
            cpu_core: Some(0),
        }
    }
}

impl BinanceConfig {
    pub fn testnet() -> Self {
        Self {
            base_url: "https://testnet.binance.vision".to_string(),
            ws_url: "wss://testnet.binance.vision".to_string(),
            testnet: true,
            ..Default::default()
        }
    }
    
    pub fn with_credentials(mut self, api_key: String, api_secret: String) -> Self {
        self.api_key = api_key;
        self.api_secret = api_secret;
        self
    }
    
    pub fn with_timing(mut self, enable: bool) -> Self {
        self.enable_timing = enable;
        self
    }
    
    pub fn with_cpu_core(mut self, core: Option<usize>) -> Self {
        self.cpu_core = core;
        self
    }
    
    pub fn with_env_credentials(mut self) -> crate::errors::Result<Self> {
        use crate::errors::ExchangeError;
        
        let api_key = std::env::var("BINANCE_API_KEY")
            .map_err(|_| ExchangeError::MissingCredentials("BINANCE_API_KEY".to_string()))?;
        let api_secret = std::env::var("BINANCE_SECRET_KEY")
            .map_err(|_| ExchangeError::MissingCredentials("BINANCE_SECRET_KEY".to_string()))?;
        
        self.api_key = api_key;
        self.api_secret = api_secret;
        Ok(self)
    }
}

/// Exchange information from Binance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeInfo {
    pub timezone: String,
    pub server_time: u64,
    pub symbols: Vec<SymbolInfo>,
}

/// Symbol information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInfo {
    pub symbol: String,
    pub status: String,
    #[serde(rename = "baseAsset")]
    pub base_asset: String,
    #[serde(rename = "quoteAsset")]
    pub quote_asset: String,
    pub filters: Vec<serde_json::Value>,
}

/// High-performance Binance REST client using monoio
pub struct BinanceRestClient {
    #[allow(dead_code)] // Will be used for authenticated requests
    config: BinanceConfig,
    base_url: Url,
    https_client: MonoioHttpsClient,
    // Connection pool for reuse (simplified for now)
    // In production, you'd want a proper connection pool
}

impl BinanceRestClient {
    /// Create a new Binance REST client
    pub async fn new(config: BinanceConfig) -> Result<Self> {
        let base_url = Url::parse(&config.base_url)
            .map_err(|e| ExchangeError::InvalidUrl(e.to_string()))?;
        
        info!("🔗 Binance REST client created");
        info!("   Base URL: {}", base_url);
        
        let https_client = MonoioHttpsClient::new()?;
        
        Ok(Self {
            config,
            base_url,
            https_client,
        })
    }
    
    /// Test connectivity (ping endpoint)
    pub async fn ping(&self) -> Result<()> {
        let endpoint = "/api/v3/ping";
        let _response = self.get_request(endpoint, None).await?;
        Ok(())
    }
    
    /// Get server time
    pub async fn server_time(&self) -> Result<u64> {
        let endpoint = "/api/v3/time";
        let response = self.get_request(endpoint, None).await?;
        
        let server_time: u64 = response["serverTime"]
            .as_u64()
            .ok_or_else(|| ExchangeError::InvalidResponse("Missing serverTime".to_string()))?;
            
        Ok(server_time)
    }
    
    /// Alias for server_time() for compatibility
    pub async fn get_server_time(&self) -> Result<u64> {
        self.server_time().await
    }
    
    /// Get exchange information
    pub async fn exchange_info(&self) -> Result<ExchangeInfo> {
        let endpoint = "/api/v3/exchangeInfo";
        let response = self.get_request(endpoint, None).await?;
        
        serde_json::from_value(response)
            .map_err(|e| ExchangeError::SerializationError(e.to_string()))
    }
    
    /// Get ticker information for a symbol
    pub async fn ticker_24hr(&self, symbol: &str) -> Result<Ticker24hr> {
        let endpoint = "/api/v3/ticker/24hr";
        let params = vec![("symbol", symbol)];
        let response = self.get_request(endpoint, Some(params)).await?;
        
        serde_json::from_value(response)
            .map_err(|e| ExchangeError::SerializationError(e.to_string()))
    }
    
    /// Get order book for a symbol
    pub async fn order_book(&self, symbol: &str, limit: Option<u32>) -> Result<OrderBookResponse> {
        let endpoint = "/api/v3/depth";
        let mut params = vec![("symbol", symbol)];
        
        let limit_str;
        if let Some(limit) = limit {
            limit_str = limit.to_string();
            params.push(("limit", &limit_str));
        }
        
        let response = self.get_request(endpoint, Some(params)).await?;
        
        serde_json::from_value(response)
            .map_err(|e| ExchangeError::SerializationError(e.to_string()))
    }
    
    /// Get recent trades for a symbol
    pub async fn recent_trades(&self, symbol: &str, limit: Option<u32>) -> Result<Vec<TradeResponse>> {
        let endpoint = "/api/v3/trades";
        let mut params = vec![("symbol", symbol)];
        
        let limit_str;
        if let Some(limit) = limit {
            limit_str = limit.to_string();
            params.push(("limit", &limit_str));
        }
        
        let response = self.get_request(endpoint, Some(params)).await?;
        
        serde_json::from_value(response)
            .map_err(|e| ExchangeError::SerializationError(e.to_string()))
    }
    
    /// Get account information (requires authentication)
    pub async fn get_account_info(&self) -> Result<AccountInfo> {
        let endpoint = "/api/v3/account";
        let response = self.signed_request(endpoint, "GET", None).await?;
        
        serde_json::from_value(response)
            .map_err(|e| ExchangeError::SerializationError(e.to_string()))
    }
    
    /// Get symbol price ticker
    pub async fn get_symbol_price_ticker(&self, symbol: &str) -> Result<PriceTicker> {
        let endpoint = "/api/v3/ticker/price";
        let params = vec![("symbol", symbol)];
        let response = self.get_request(endpoint, Some(params)).await?;
        
        serde_json::from_value(response)
            .map_err(|e| ExchangeError::SerializationError(e.to_string()))
    }
    
    /// Test new order (validates order without placing)
    pub async fn test_new_order(&self, order_params: &TestOrderParams<'_>) -> Result<()> {
        let endpoint = "/api/v3/order/test";
        
        let mut params = HashMap::new();
        params.insert("symbol", order_params.symbol);
        params.insert("side", order_params.side);
        params.insert("type", order_params.order_type);
        
        if let Some(q) = order_params.quantity {
            params.insert("quantity", q);
        }
        if let Some(p) = order_params.price {
            params.insert("price", p);
        }
        if let Some(tif) = order_params.time_in_force {
            params.insert("timeInForce", tif);
        }
        if let Some(sp) = order_params.stop_price {
            params.insert("stopPrice", sp);
        }
        if let Some(iq) = order_params.iceberg_qty {
            params.insert("icebergQty", iq);
        }
        
        let _response = self.signed_request(endpoint, "POST", Some(params)).await?;
        Ok(())
    }

    /// Place a new order
    pub async fn new_order(&self, order_params: &TestOrderParams<'_>) -> Result<NewOrderResponse> {
        let endpoint = "/api/v3/order";
        
        let mut params = HashMap::new();
        params.insert("symbol", order_params.symbol);
        params.insert("side", order_params.side);
        params.insert("type", order_params.order_type);
        
        if let Some(q) = order_params.quantity {
            params.insert("quantity", q);
        }
        if let Some(p) = order_params.price {
            params.insert("price", p);
        }
        if let Some(tif) = order_params.time_in_force {
            params.insert("timeInForce", tif);
        }
        if let Some(sp) = order_params.stop_price {
            params.insert("stopPrice", sp);
        }
        if let Some(iq) = order_params.iceberg_qty {
            params.insert("icebergQty", iq);
        }
        
        let response = self.signed_request(endpoint, "POST", Some(params)).await?;
        
        serde_json::from_value(response)
            .map_err(|e| ExchangeError::SerializationError(e.to_string()))
    }

    /// Cancel an existing order
    pub async fn cancel_order(&self, symbol: &str, order_id: u64) -> Result<CancelOrderResponse> {
        let endpoint = "/api/v3/order";
        
        let order_id_str = order_id.to_string();
        let mut params = HashMap::new();
        params.insert("symbol", symbol);
        params.insert("orderId", &order_id_str);
        
        let response = self.signed_request(endpoint, "DELETE", Some(params)).await?;
        
        serde_json::from_value(response)
            .map_err(|e| ExchangeError::SerializationError(e.to_string()))
    }

    /// Query order status
    pub async fn query_order(&self, symbol: &str, order_id: u64) -> Result<QueryOrderResponse> {
        let endpoint = "/api/v3/order";
        
        let order_id_str = order_id.to_string();
        let mut params = HashMap::new();
        params.insert("symbol", symbol);
        params.insert("orderId", &order_id_str);
        
        let response = self.signed_request(endpoint, "GET", Some(params)).await?;
        
        serde_json::from_value(response)
            .map_err(|e| ExchangeError::SerializationError(e.to_string()))
    }

    /// Get all open orders for a symbol
    pub async fn open_orders(&self, symbol: Option<&str>) -> Result<Vec<QueryOrderResponse>> {
        let endpoint = "/api/v3/openOrders";
        
        let mut params = HashMap::new();
        if let Some(s) = symbol {
            params.insert("symbol", s);
        }
        
        let response = self.signed_request(endpoint, "GET", Some(params)).await?;
        
        serde_json::from_value(response)
            .map_err(|e| ExchangeError::SerializationError(e.to_string()))
    }

    /// Get trade history for a symbol
    pub async fn my_trades(&self, symbol: &str, limit: Option<u32>) -> Result<Vec<MyTradeResponse>> {
        let endpoint = "/api/v3/myTrades";
        
        let mut params = HashMap::new();
        params.insert("symbol", symbol);
        
        let limit_str = limit.map(|l| l.to_string());
        if let Some(ref l) = limit_str {
            params.insert("limit", l);
        }
        
        let response = self.signed_request(endpoint, "GET", Some(params)).await?;
        
        serde_json::from_value(response)
            .map_err(|e| ExchangeError::SerializationError(e.to_string()))
    }
    
    /// Make a GET request with timing measurement
    async fn get_request(
        &self,
        endpoint: &str,
        params: Option<Vec<(&str, &str)>>,
    ) -> Result<Value> {
        let timer = PerfTimer::start(format!("binance_get_{endpoint}"));
        
        // Build URL
        let mut url = self.base_url.clone();
        url.set_path(endpoint);
        
        if let Some(params) = params {
            let mut query_pairs = url.query_pairs_mut();
            for (key, value) in params {
                query_pairs.append_pair(key, value);
            }
        }
        
        debug!("📡 GET {}", url);
        
        // For now, use a simplified HTTP client
        // In production, you'd want a proper monoio-based HTTP client
        let response = self.make_http_request(url.as_str(), "GET", None).await?;
        
        timer.log_elapsed();
        
        debug!("Response: {}", response);
        
        serde_json::from_str(&response)
            .map_err(|e| ExchangeError::SerializationError(format!("{e}: {response}")))
    }
    
    /// Make a signed request (for authenticated endpoints)
    async fn signed_request(
        &self,
        endpoint: &str,
        method: &str,
        params: Option<HashMap<&str, &str>>,
    ) -> Result<Value> {
        let timer = PerfTimer::start(format!("binance_signed_{endpoint}"));
        
        // Create auth helper
        let auth = BinanceAuth::new(&self.config.api_key, &self.config.api_secret);
        
        // Build URL with signature
        let mut url = self.base_url.clone();
        url.set_path(endpoint);
        
        // Prepare query parameters
        let mut query_params = HashMap::new();
        if let Some(p) = params {
            query_params.extend(p);
        }
        
        // Add timestamp and recvWindow
        let timestamp = nanos() / 1_000_000; // Convert to milliseconds
        let timestamp_str = timestamp.to_string();
        let recv_window = "5000".to_string();
        query_params.insert("timestamp", &timestamp_str);
        query_params.insert("recvWindow", &recv_window);
        
        // Create signature
        let query_string = auth.build_query_string(&query_params);
        let signature = auth.sign(&query_string);
        
        
        // Add signature to URL
        url.set_query(Some(&format!("{query_string}&signature={signature}")));
        
        debug!("📡 {} {} (signed)", method, url);
        
        // Make request with API key header
        let mut headers = HashMap::new();
        headers.insert("X-MBX-APIKEY", self.config.api_key.as_str());
        
        let response = self.make_http_request_with_headers(
            url.as_str(),
            method,
            None,
            headers
        ).await?;
        
        timer.log_elapsed();
        
        serde_json::from_str(&response)
            .map_err(|e| ExchangeError::SerializationError(e.to_string()))
    }
    
    /// Make HTTP request using monoio-native HTTPS client
    async fn make_http_request(
        &self,
        url: &str,
        method: &str,
        body: Option<&str>,
    ) -> Result<String> {
        self.make_http_request_with_headers(url, method, body, HashMap::new()).await
    }
    
    /// Make HTTP request with custom headers
    async fn make_http_request_with_headers(
        &self,
        url: &str,
        method: &str,
        body: Option<&str>,
        headers: HashMap<&str, &str>,
    ) -> Result<String> {
        let response = self.https_client.request_with_headers(method, url, body, &headers).await?;
        
        if response.status != 200 {
            return Err(ExchangeError::HttpError(
                response.status,
                format!("HTTP {}: {}", response.status, response.body),
            ));
        }
        
        Ok(response.body)
    }
    
}

/// 24-hour ticker statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Ticker24hr {
    pub symbol: String,
    #[serde(rename = "priceChange")]
    pub price_change: String,
    #[serde(rename = "priceChangePercent")]
    pub price_change_percent: String,
    #[serde(rename = "weightedAvgPrice")]
    pub weighted_avg_price: String,
    #[serde(rename = "prevClosePrice")]
    pub prev_close_price: String,
    #[serde(rename = "lastPrice")]
    pub last_price: String,
    #[serde(rename = "lastQty")]
    pub last_qty: String,
    #[serde(rename = "bidPrice")]
    pub bid_price: String,
    #[serde(rename = "bidQty")]
    pub bid_qty: String,
    #[serde(rename = "askPrice")]
    pub ask_price: String,
    #[serde(rename = "askQty")]
    pub ask_qty: String,
    #[serde(rename = "openPrice")]
    pub open_price: String,
    #[serde(rename = "highPrice")]
    pub high_price: String,
    #[serde(rename = "lowPrice")]
    pub low_price: String,
    pub volume: String,
    #[serde(rename = "quoteVolume")]
    pub quote_volume: String,
    #[serde(rename = "openTime")]
    pub open_time: u64,
    #[serde(rename = "closeTime")]
    pub close_time: u64,
    #[serde(rename = "firstId")]
    pub first_id: u64,
    #[serde(rename = "lastId")]
    pub last_id: u64,
    pub count: u64,
}

/// Order book response from Binance
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OrderBookResponse {
    #[serde(rename = "lastUpdateId")]
    pub last_update_id: u64,
    pub bids: Vec<[String; 2]>, // [price, quantity]
    pub asks: Vec<[String; 2]>, // [price, quantity]
}

/// Trade response from Binance
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TradeResponse {
    pub id: u64,
    pub price: String,
    pub qty: String,
    #[serde(rename = "quoteQty")]
    pub quote_qty: String,
    pub time: u64,
    #[serde(rename = "isBuyerMaker")]
    pub is_buyer_maker: bool,
    #[serde(rename = "isBestMatch")]
    pub is_best_match: bool,
}

/// Account information response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    #[serde(rename = "makerCommission")]
    pub maker_commission: u32,
    #[serde(rename = "takerCommission")]
    pub taker_commission: u32,
    #[serde(rename = "buyerCommission")]
    pub buyer_commission: u32,
    #[serde(rename = "sellerCommission")]
    pub seller_commission: u32,
    #[serde(rename = "canTrade")]
    pub can_trade: bool,
    #[serde(rename = "canWithdraw")]
    pub can_withdraw: bool,
    #[serde(rename = "canDeposit")]
    pub can_deposit: bool,
    #[serde(rename = "updateTime")]
    pub update_time: u64,
    #[serde(rename = "accountType")]
    pub account_type: String,
    pub balances: Vec<Balance>,
    pub permissions: Vec<String>,
}

/// Balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub asset: String,
    pub free: String,
    pub locked: String,
}

/// Price ticker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceTicker {
    pub symbol: String,
    pub price: String,
}

/// New order response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewOrderResponse {
    pub symbol: String,
    #[serde(rename = "orderId")]
    pub order_id: u64,
    #[serde(rename = "orderListId")]
    pub order_list_id: i32,
    #[serde(rename = "clientOrderId")]
    pub client_order_id: String,
    #[serde(rename = "transactTime")]
    pub transact_time: u64,
    pub price: String,
    #[serde(rename = "origQty")]
    pub orig_qty: String,
    #[serde(rename = "executedQty")]
    pub executed_qty: String,
    #[serde(rename = "cummulativeQuoteQty")]
    pub cumulative_quote_qty: String,
    pub status: String,
    #[serde(rename = "timeInForce")]
    pub time_in_force: String,
    #[serde(rename = "type")]
    pub order_type: String,
    pub side: String,
}

/// Cancel order response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrderResponse {
    pub symbol: String,
    #[serde(rename = "origClientOrderId")]
    pub orig_client_order_id: String,
    #[serde(rename = "orderId")]
    pub order_id: u64,
    #[serde(rename = "orderListId")]
    pub order_list_id: i32,
    #[serde(rename = "clientOrderId")]
    pub client_order_id: String,
    pub price: String,
    #[serde(rename = "origQty")]
    pub orig_qty: String,
    #[serde(rename = "executedQty")]
    pub executed_qty: String,
    #[serde(rename = "cummulativeQuoteQty")]
    pub cumulative_quote_qty: String,
    pub status: String,
    #[serde(rename = "timeInForce")]
    pub time_in_force: String,
    #[serde(rename = "type")]
    pub order_type: String,
    pub side: String,
}

/// Query order response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryOrderResponse {
    pub symbol: String,
    #[serde(rename = "orderId")]
    pub order_id: u64,
    #[serde(rename = "orderListId")]
    pub order_list_id: i32,
    #[serde(rename = "clientOrderId")]
    pub client_order_id: String,
    pub price: String,
    #[serde(rename = "origQty")]
    pub orig_qty: String,
    #[serde(rename = "executedQty")]
    pub executed_qty: String,
    #[serde(rename = "cummulativeQuoteQty")]
    pub cumulative_quote_qty: String,
    pub status: String,
    #[serde(rename = "timeInForce")]
    pub time_in_force: String,
    #[serde(rename = "type")]
    pub order_type: String,
    pub side: String,
    #[serde(rename = "stopPrice")]
    pub stop_price: String,
    #[serde(rename = "icebergQty")]
    pub iceberg_qty: String,
    pub time: u64,
    #[serde(rename = "updateTime")]
    pub update_time: u64,
    #[serde(rename = "isWorking")]
    pub is_working: bool,
    #[serde(rename = "origQuoteOrderQty")]
    pub orig_quote_order_qty: String,
}

/// My trades response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyTradeResponse {
    pub symbol: String,
    pub id: u64,
    #[serde(rename = "orderId")]
    pub order_id: u64,
    #[serde(rename = "orderListId")]
    pub order_list_id: i32,
    pub price: String,
    pub qty: String,
    #[serde(rename = "quoteQty")]
    pub quote_qty: String,
    pub commission: String,
    #[serde(rename = "commissionAsset")]
    pub commission_asset: String,
    pub time: u64,
    #[serde(rename = "isBuyer")]
    pub is_buyer: bool,
    #[serde(rename = "isMaker")]
    pub is_maker: bool,
    #[serde(rename = "isBestMatch")]
    pub is_best_match: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[monoio::test]
    async fn test_rest_client_creation() {
        let config = BinanceConfig::testnet();
        let client = BinanceRestClient::new(config).await;
        assert!(client.is_ok());
    }
}