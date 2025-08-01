//! Binance authentication and request signing
//!
//! High-performance architecture:
//! - High-performance HMAC-SHA256 signing
//! - Nanosecond precision timestamps
//! - Secure credential handling

use crate::errors::{ExchangeError, Result};
use sriquant_core::prelude::*;

use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;
use tracing::debug;
use url::Url;

type HmacSha256 = Hmac<Sha256>;

/// Binance API credentials
#[derive(Debug, Clone)]
pub struct BinanceCredentials {
    pub api_key: String,
    pub secret_key: String,
}

impl BinanceCredentials {
    /// Create new credentials
    pub fn new(api_key: String, secret_key: String) -> Self {
        Self {
            api_key,
            secret_key,
        }
    }
    
    /// Load credentials from environment variables
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("BINANCE_API_KEY")
            .map_err(|_| ExchangeError::MissingCredentials("BINANCE_API_KEY".to_string()))?;
        let secret_key = std::env::var("BINANCE_SECRET_KEY")
            .map_err(|_| ExchangeError::MissingCredentials("BINANCE_SECRET_KEY".to_string()))?;
        
        Ok(Self::new(api_key, secret_key))
    }
    
    /// Check if credentials are valid (non-empty)
    pub fn is_valid(&self) -> bool {
        !self.api_key.is_empty() && !self.secret_key.is_empty()
    }
}

/// Binance request signer
pub struct BinanceSigner {
    credentials: BinanceCredentials,
}

impl BinanceSigner {
    /// Create new signer with credentials
    pub fn new(credentials: BinanceCredentials) -> Result<Self> {
        if !credentials.is_valid() {
            return Err(ExchangeError::InvalidCredentials);
        }
        
        Ok(Self { credentials })
    }
    
    /// Sign a request with HMAC-SHA256
    pub fn sign_request(
        &self,
        method: &str,
        endpoint: &str,
        params: &HashMap<String, String>,
    ) -> Result<SignedRequest> {
        let timer = PerfTimer::start("binance_sign_request".to_string());
        
        // Add timestamp with nanosecond precision
        let mut signed_params = params.clone();
        let timestamp = get_timestamp_ms();
        signed_params.insert("timestamp".to_string(), timestamp.to_string());
        
        // Create query string
        let query_string = self.build_query_string(&signed_params);
        
        // Create signature
        let signature = self.create_signature(&query_string)?;
        signed_params.insert("signature".to_string(), signature);
        
        // Build final query string with signature
        let final_query = self.build_query_string(&signed_params);
        
        let signed_request = SignedRequest {
            method: method.to_string(),
            endpoint: endpoint.to_string(),
            query_string: final_query,
            headers: self.build_headers(),
            timestamp,
        };
        
        timer.log_elapsed();
        debug!("🔐 Signed request: {} {}", method, endpoint);
        
        Ok(signed_request)
    }
    
    /// Create HMAC-SHA256 signature
    fn create_signature(&self, payload: &str) -> Result<String> {
        let mut mac = HmacSha256::new_from_slice(self.credentials.secret_key.as_bytes())
            .map_err(|e| ExchangeError::SigningError(format!("HMAC setup failed: {e}")))?;
        
        mac.update(payload.as_bytes());
        let signature = mac.finalize().into_bytes();
        
        Ok(hex::encode(signature))
    }
    
    /// Build query string from parameters
    fn build_query_string(&self, params: &HashMap<String, String>) -> String {
        let mut pairs: Vec<_> = params.iter().collect();
        pairs.sort_by_key(|(k, _)| *k); // Sort by key for consistent ordering
        
        pairs
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&")
    }
    
    /// Build HTTP headers for authenticated requests
    fn build_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("X-MBX-APIKEY".to_string(), self.credentials.api_key.clone());
        headers.insert("Content-Type".to_string(), "application/x-www-form-urlencoded".to_string());
        headers
    }
    
    /// Sign WebSocket authentication
    pub fn sign_websocket_auth(&self) -> Result<WebSocketAuth> {
        let timestamp = get_timestamp_ms();
        let payload = format!("timestamp={timestamp}");
        let signature = self.create_signature(&payload)?;
        
        Ok(WebSocketAuth {
            api_key: self.credentials.api_key.clone(),
            timestamp,
            signature,
        })
    }
    
    /// Validate request signature (for testing)
    pub fn validate_signature(&self, payload: &str, signature: &str) -> bool {
        match self.create_signature(payload) {
            Ok(expected_sig) => expected_sig == signature,
            Err(_) => false,
        }
    }
}

/// Signed request with all necessary components
#[derive(Debug, Clone)]
pub struct SignedRequest {
    pub method: String,
    pub endpoint: String,
    pub query_string: String,
    pub headers: HashMap<String, String>,
    pub timestamp: u64,
}

impl SignedRequest {
    /// Build full URL with query parameters
    pub fn build_url(&self, base_url: &str) -> Result<String> {
        let mut url = Url::parse(base_url)
            .map_err(|e| ExchangeError::InvalidUrl(e.to_string()))?;
        
        url.set_path(&self.endpoint);
        
        if !self.query_string.is_empty() {
            url.set_query(Some(&self.query_string));
        }
        
        Ok(url.to_string())
    }
    
    /// Get age of request in milliseconds
    pub fn age_ms(&self) -> u64 {
        let current_time = get_timestamp_ms();
        current_time.saturating_sub(self.timestamp)
    }
    
    /// Check if request is expired (older than 5 seconds)
    pub fn is_expired(&self) -> bool {
        self.age_ms() > 5000
    }
}

/// WebSocket authentication data
#[derive(Debug, Clone)]
pub struct WebSocketAuth {
    pub api_key: String,
    pub timestamp: u64,
    pub signature: String,
}

/// Get current timestamp in milliseconds using high-precision timing
fn get_timestamp_ms() -> u64 {
    // Use our high-precision timer
    let nanos = nanos();
    nanos / 1_000_000 // Convert to milliseconds
}

/// Request signing parameters for different Binance endpoints
pub struct BinanceEndpoints;

impl BinanceEndpoints {
    /// Parameters for placing a new order
    pub fn new_order(
        symbol: &str,
        side: &str,
        order_type: &str,
        quantity: &str,
        price: Option<&str>,
        time_in_force: Option<&str>,
    ) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        params.insert("side".to_string(), side.to_string());
        params.insert("type".to_string(), order_type.to_string());
        params.insert("quantity".to_string(), quantity.to_string());
        
        if let Some(price) = price {
            params.insert("price".to_string(), price.to_string());
        }
        
        if let Some(tif) = time_in_force {
            params.insert("timeInForce".to_string(), tif.to_string());
        }
        
        params
    }
    
    /// Parameters for canceling an order
    pub fn cancel_order(symbol: &str, order_id: Option<u64>, orig_client_order_id: Option<&str>) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        
        if let Some(id) = order_id {
            params.insert("orderId".to_string(), id.to_string());
        }
        
        if let Some(client_id) = orig_client_order_id {
            params.insert("origClientOrderId".to_string(), client_id.to_string());
        }
        
        params
    }
    
    /// Parameters for querying account information
    pub fn account_info() -> HashMap<String, String> {
        HashMap::new() // Only timestamp and signature required
    }
    
    /// Parameters for querying open orders
    pub fn open_orders(symbol: Option<&str>) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if let Some(symbol) = symbol {
            params.insert("symbol".to_string(), symbol.to_string());
        }
        
        params
    }
    
    /// Parameters for querying order history
    pub fn order_history(symbol: &str, limit: Option<u32>) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        
        if let Some(limit) = limit {
            params.insert("limit".to_string(), limit.to_string());
        }
        
        params
    }
}

/// Simple authentication helper for REST client
pub struct BinanceAuth {
    secret_key: String,
}

impl BinanceAuth {
    /// Create new auth helper
    pub fn new(_api_key: &str, secret_key: &str) -> Self {
        Self {
            secret_key: secret_key.to_string(),
        }
    }
    
    /// Sign a message with HMAC-SHA256
    pub fn sign(&self, message: &str) -> String {
        use hmac::Mac;
        
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(message.as_bytes());
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }
    
    /// Build query string from parameters
    pub fn build_query_string(&self, params: &HashMap<&str, &str>) -> String {
        let mut pairs: Vec<_> = params.iter().collect();
        pairs.sort_by_key(|(k, _)| *k);
        
        pairs
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&")
    }
}

/// Security utilities
pub struct BinanceSecurity;

impl BinanceSecurity {
    /// Validate API key format (basic check)
    pub fn is_valid_api_key(key: &str) -> bool {
        key.len() >= 64 && key.chars().all(|c| c.is_ascii_alphanumeric())
    }
    
    /// Validate secret key format (basic check)
    pub fn is_valid_secret_key(key: &str) -> bool {
        key.len() >= 64 && key.chars().all(|c| c.is_ascii_alphanumeric())
    }
    
    /// Generate client order ID
    pub fn generate_client_order_id() -> String {
        let id = generate_id_with_prefix("SRI");
        id.replace("-", "").chars().take(36).collect() // Binance limit
    }
    
    /// Check if timestamp is within acceptable range
    pub fn is_timestamp_valid(timestamp: u64) -> bool {
        let current = get_timestamp_ms();
        let diff = current.saturating_sub(timestamp);
        diff <= 60000 // Within 1 minute
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_credentials_creation() {
        let creds = BinanceCredentials::new(
            "test_api_key".to_string(),
            "test_secret_key".to_string(),
        );
        
        assert!(creds.is_valid());
        assert_eq!(creds.api_key, "test_api_key");
        assert_eq!(creds.secret_key, "test_secret_key");
    }
    
    #[test]
    fn test_empty_credentials() {
        let creds = BinanceCredentials::new("".to_string(), "".to_string());
        assert!(!creds.is_valid());
    }
    
    #[test]
    fn test_signature_creation() {
        let creds = BinanceCredentials::new(
            "test_api_key".to_string(),
            "test_secret_key".to_string(),
        );
        
        let signer = BinanceSigner::new(creds).unwrap();
        let signature = signer.create_signature("symbol=BTCUSDT&side=BUY&type=LIMIT&quantity=1&price=50000&timestamp=1234567890").unwrap();
        
        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 64); // SHA256 hex is 64 chars
    }
    
    #[test]
    fn test_signature_validation() {
        let creds = BinanceCredentials::new(
            "test_api_key".to_string(),
            "test_secret_key".to_string(),
        );
        
        let signer = BinanceSigner::new(creds).unwrap();
        let payload = "symbol=BTCUSDT&side=BUY";
        let signature = signer.create_signature(payload).unwrap();
        
        assert!(signer.validate_signature(payload, &signature));
        assert!(!signer.validate_signature(payload, "invalid_signature"));
    }
    
    #[test]
    fn test_client_order_id_generation() {
        let order_id = BinanceSecurity::generate_client_order_id();
        assert!(!order_id.is_empty());
        assert!(order_id.len() <= 36);
        assert!(order_id.starts_with("SRI"));
    }
    
    #[test]
    fn test_endpoint_parameters() {
        let params = BinanceEndpoints::new_order(
            "BTCUSDT",
            "BUY",
            "LIMIT",
            "1.0",
            Some("50000.0"),
            Some("GTC"),
        );
        
        assert_eq!(params.get("symbol").unwrap(), "BTCUSDT");
        assert_eq!(params.get("side").unwrap(), "BUY");
        assert_eq!(params.get("type").unwrap(), "LIMIT");
        assert_eq!(params.get("quantity").unwrap(), "1.0");
        assert_eq!(params.get("price").unwrap(), "50000.0");
        assert_eq!(params.get("timeInForce").unwrap(), "GTC");
    }
}