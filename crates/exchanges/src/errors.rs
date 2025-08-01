//! Exchange-specific error types
//!
//! High-performance architecture with comprehensive error handling
//! and performance-optimized error propagation.

use thiserror::Error;

/// Result type for exchange operations
pub type Result<T> = std::result::Result<T, ExchangeError>;

/// Exchange operation errors
#[derive(Error, Debug, Clone)]
pub enum ExchangeError {
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("HTTP error {0}: {1}")]
    HttpError(u16, String),
    
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    #[error("Authentication failed")]
    AuthenticationFailed,
    
    #[error("Missing credentials: {0}")]
    MissingCredentials(String),
    
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("Signing error: {0}")]
    SigningError(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Insufficient balance")]
    InsufficientBalance,
    
    #[error("Invalid symbol: {0}")]
    InvalidSymbol(String),
    
    #[error("Invalid order: {0}")]
    InvalidOrder(String),
    
    #[error("Order not found: {0}")]
    OrderNotFound(String),
    
    #[error("Exchange not supported: {0}")]
    ExchangeNotSupported(String),
    
    #[error("Feature not supported: {0}")]
    FeatureNotSupported(String),
    
    #[error("Unsupported method: {0}")]
    UnsupportedMethod(String),
    
    #[error("Unsupported stream: {0}")]
    UnsupportedStream(String),
    
    #[error("Client not initialized: {0}")]
    ClientNotInitialized(String),
    
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Market closed")]
    MarketClosed,
    
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),
    
    #[error("Price precision error: {0}")]
    PricePrecisionError(String),
    
    #[error("Quantity precision error: {0}")]
    QuantityPrecisionError(String),
    
    #[error("Fixed point error: {0}")]
    FixedPointError(String),
}

impl From<sriquant_core::fixed::FixedError> for ExchangeError {
    fn from(err: sriquant_core::fixed::FixedError) -> Self {
        Self::FixedPointError(err.to_string())
    }
}

impl From<serde_json::Error> for ExchangeError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError(err.to_string())
    }
}

impl From<url::ParseError> for ExchangeError {
    fn from(err: url::ParseError) -> Self {
        Self::InvalidUrl(err.to_string())
    }
}

// Using monoio-native HTTP client for all network operations

/// Exchange-specific error codes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCode {
    // Generic codes
    Unknown = 0,
    Success = 1,
    
    // Network codes (1000-1099)
    NetworkTimeout = 1001,
    ConnectionLost = 1002,
    DnsFailure = 1003,
    
    // Authentication codes (1100-1199)
    InvalidApiKey = 1101,
    InvalidSignature = 1102,
    TimestampExpired = 1103,
    PermissionDenied = 1104,
    
    // Trading codes (1200-1299)
    InsufficientBalance = 1201,
    InvalidSymbol = 1202,
    InvalidQuantity = 1203,
    InvalidPrice = 1204,
    OrderNotFound = 1205,
    MarketClosed = 1206,
    
    // Rate limiting codes (1300-1399)
    RateLimitExceeded = 1301,
    WeightLimitExceeded = 1302,
    
    // System codes (1400-1499)
    SystemMaintenance = 1401,
    SystemOverload = 1402,
}

impl From<ErrorCode> for u16 {
    fn from(code: ErrorCode) -> u16 {
        code as u16
    }
}

impl From<u16> for ErrorCode {
    fn from(code: u16) -> Self {
        match code {
            1 => ErrorCode::Success,
            1001 => ErrorCode::NetworkTimeout,
            1002 => ErrorCode::ConnectionLost,
            1003 => ErrorCode::DnsFailure,
            1101 => ErrorCode::InvalidApiKey,
            1102 => ErrorCode::InvalidSignature,
            1103 => ErrorCode::TimestampExpired,
            1104 => ErrorCode::PermissionDenied,
            1201 => ErrorCode::InsufficientBalance,
            1202 => ErrorCode::InvalidSymbol,
            1203 => ErrorCode::InvalidQuantity,
            1204 => ErrorCode::InvalidPrice,
            1205 => ErrorCode::OrderNotFound,
            1206 => ErrorCode::MarketClosed,
            1301 => ErrorCode::RateLimitExceeded,
            1302 => ErrorCode::WeightLimitExceeded,
            1401 => ErrorCode::SystemMaintenance,
            1402 => ErrorCode::SystemOverload,
            _ => ErrorCode::Unknown,
        }
    }
}