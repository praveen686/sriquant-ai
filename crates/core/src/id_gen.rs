//! ID generation implementation
//!
//! Incorporates nanoid and idgen_next_id functions for efficient and unique 
//! identifier generation, essential for transaction tracking.

use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Global counter for sequential ID generation
static GLOBAL_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Order ID type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrderId(String);

/// Trade ID type  
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TradeId(String);

/// Request ID type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RequestId(String);

/// Session ID type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(String);

impl Default for OrderId {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderId {
    /// Create a new order ID
    pub fn new() -> Self {
        Self(generate_id_with_prefix("ORD"))
    }
    
    /// Create from string
    pub fn from_string(id: String) -> Self {
        Self(id)
    }
    
    /// Get as string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for TradeId {
    fn default() -> Self {
        Self::new()
    }
}

impl TradeId {
    /// Create a new trade ID
    pub fn new() -> Self {
        Self(generate_id_with_prefix("TRD"))
    }
    
    /// Create from string
    pub fn from_string(id: String) -> Self {
        Self(id)
    }
    
    /// Get as string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestId {
    /// Create a new request ID
    pub fn new() -> Self {
        Self(generate_id_with_prefix("REQ"))
    }
    
    /// Create from string
    pub fn from_string(id: String) -> Self {
        Self(id)
    }
    
    /// Get as string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionId {
    /// Create a new session ID
    pub fn new() -> Self {
        Self(generate_id_with_prefix("SES"))
    }
    
    /// Create from string
    pub fn from_string(id: String) -> Self {
        Self(id)
    }
    
    /// Get as string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Display implementations
impl Display for OrderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for TradeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Generate a unique ID using nanoid ()
pub fn generate_id() -> String {
    nanoid!(12) // 12 character nanoid
}

/// Generate a unique ID with custom length
pub fn generate_id_with_length(length: usize) -> String {
    nanoid!(length)
}

/// Generate a unique ID with prefix and timestamp
pub fn generate_id_with_prefix(prefix: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let short_id = nanoid!(8);
    format!("{prefix}-{timestamp}-{short_id}")
}

/// Generate sequential ID ()
pub fn idgen_next_id() -> u64 {
    GLOBAL_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Generate timestamped sequential ID
pub fn generate_timestamped_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let counter = idgen_next_id();
    format!("{timestamp:016x}{counter:08x}")
}

/// Generate ID for specific exchange
pub fn generate_exchange_id(exchange: &str) -> String {
    let id = generate_id();
    format!("{}_{}", exchange.to_uppercase(), id)
}

/// ID generator configuration
#[derive(Debug, Clone)]
pub struct IdConfig {
    pub prefix: Option<String>,
    pub length: usize,
    pub include_timestamp: bool,
    pub use_counter: bool,
}

impl Default for IdConfig {
    fn default() -> Self {
        Self {
            prefix: None,
            length: 12,
            include_timestamp: false,
            use_counter: false,
        }
    }
}

/// Configurable ID generator
pub struct IdGenerator {
    config: IdConfig,
}

impl IdGenerator {
    /// Create new ID generator with config
    pub fn new(config: IdConfig) -> Self {
        Self { config }
    }
    
    /// Generate ID with current configuration  
    pub fn generate(&self) -> String {
        let mut parts = Vec::new();
        
        // Add prefix if configured
        if let Some(ref prefix) = self.config.prefix {
            parts.push(prefix.clone());
        }
        
        // Add timestamp if configured
        if self.config.include_timestamp {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            parts.push(timestamp.to_string());
        }
        
        // Add counter if configured
        if self.config.use_counter {
            let counter = idgen_next_id();
            parts.push(format!("{counter:08x}"));
        }
        
        // Add nanoid
        let length = self.config.length;
        parts.push(nanoid::nanoid!(length, &nanoid::alphabet::SAFE));
        
        parts.join("-")
    }
}

impl Default for IdGenerator {
    fn default() -> Self {
        Self::new(IdConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    
    #[test]
    fn test_generate_id() {
        let id1 = generate_id();
        let id2 = generate_id();
        
        assert_eq!(id1.len(), 12);
        assert_eq!(id2.len(), 12);
        assert_ne!(id1, id2);
    }
    
    #[test]
    fn test_generate_id_with_length() {
        let id = generate_id_with_length(20);
        assert_eq!(id.len(), 20);
    }
    
    #[test]
    fn test_generate_id_with_prefix() {
        let id = generate_id_with_prefix("TEST");
        assert!(id.starts_with("TEST-"));
    }
    
    #[test]
    fn test_idgen_next_id() {
        let id1 = idgen_next_id();
        let id2 = idgen_next_id();
        
        assert_eq!(id2, id1 + 1);
    }
    
    #[test]
    fn test_order_id() {
        let order_id = OrderId::new();
        assert!(order_id.as_str().starts_with("ORD-"));
    }
    
    #[test]
    fn test_trade_id() {
        let trade_id = TradeId::new();
        assert!(trade_id.as_str().starts_with("TRD-"));
    }
    
    #[test]
    fn test_id_uniqueness() {
        let mut ids = HashSet::new();
        
        // Generate 1000 IDs and ensure they're all unique
        for _ in 0..1000 {
            let id = generate_id();
            assert!(ids.insert(id), "Duplicate ID generated");
        }
    }
    
    #[test]
    fn test_id_generator() {
        let config = IdConfig {
            prefix: Some("CUSTOM".to_string()),
            length: 8,
            include_timestamp: true,
            use_counter: true,
        };
        
        let generator = IdGenerator::new(config);
        let id = generator.generate();
        
        assert!(id.starts_with("CUSTOM-"));
        assert!(id.split('-').count() >= 3); // prefix, timestamp, counter, nanoid
    }
}