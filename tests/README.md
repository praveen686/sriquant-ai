# SriQuant.ai Test Suite

This directory contains comprehensive tests for the SriQuant.ai trading library, using Rust's native testing frameworks and the rstest library for parameterized testing.

## Test Structure

```
tests/
├── src/
│   ├── lib.rs                 # Test module aggregator
│   ├── unit_tests.rs         # Unit tests demonstrating Rust testing features
│   └── binance_rest_tests.rs # Comprehensive Binance REST API tests
├── examples/                  # Runnable example applications
├── binance/                   # Exchange-specific test utilities
└── benchmarks/               # Performance benchmarks
```

## Running Tests

### All Tests
```bash
cargo test
```

### Specific Test Module
```bash
# Run all Binance REST API tests
cargo test --package sriquant-tests --lib binance_rest_tests

# Run connectivity tests only
cargo test --package sriquant-tests --lib connectivity_tests

# Run market data tests
cargo test --package sriquant-tests --lib market_data_tests
```

### With Output
```bash
cargo test -- --nocapture
```

### Sequential Execution (for tests that share resources)
```bash
cargo test -- --test-threads=1
```

## Test Categories

### 1. Unit Tests (`unit_tests.rs`)
- Basic assertions and test patterns
- Parameterized tests using rstest
- Property-based testing with proptest
- Async tests with monoio
- Performance benchmarks

### 2. Binance REST API Tests (`binance_rest_tests.rs`)

#### Connectivity Tests
- `test_ping_latency` - Tests basic connectivity and measures latency
- `test_server_time` - Validates server time synchronization

#### Market Data Tests
- `test_24hr_ticker` - Tests ticker data retrieval for multiple symbols
- `test_klines` - Tests historical candlestick data with different intervals
- `test_order_book` - Tests order book depth at various levels

#### Trading Tests
- `test_place_order` - Tests the simplified order placement API
- `test_get_all_orders` - Tests order history retrieval
- `test_account_info` - Tests account balance retrieval

#### Integration Tests
- `test_order_lifecycle` - Complete order flow: place, query, cancel
- `test_portfolio_workflow` - Portfolio management and position sizing

#### Performance Tests
- `test_sequential_requests` - Measures API call performance
- `test_rate_limiting` - Tests rate limit handling

#### Error Handling Tests
- `test_invalid_symbol` - Tests error handling for invalid symbols
- `test_invalid_order_params` - Tests validation of order parameters
- `test_query_non_existent_order` - Tests handling of missing orders

## Test Features

### Parameterized Testing with rstest
```rust
#[rstest]
#[case("1m", 60)]
#[case("5m", 60)]
#[case("1h", 24)]
async fn test_klines(
    test_config: BinanceConfig,
    #[case] interval: &str,
    #[case] limit: usize
) {
    // Test runs for each case
}
```

### Test Fixtures
```rust
#[fixture]
fn test_config() -> BinanceConfig {
    dotenv::dotenv().ok();
    BinanceConfig::testnet()
        .with_env_credentials()
        .expect("Failed to load credentials")
}
```

### Sequential Test Execution
```rust
#[serial]
#[monoio::test(enable_timer = true)]
async fn test_place_order() {
    // Runs sequentially to avoid conflicts
}
```

## Environment Setup

Create a `.env` file with your Binance testnet credentials:
```bash
BINANCE_API_KEY=your_testnet_api_key
BINANCE_SECRET_KEY=your_testnet_secret_key
BINANCE_TESTNET=true
```

## Test Results Summary

All 27 tests pass successfully, covering:
- ✅ REST API connectivity
- ✅ Market data endpoints
- ✅ Order placement and management
- ✅ Account information retrieval
- ✅ Error handling
- ✅ Performance characteristics

The test suite validates:
1. All new simplified REST API endpoints work correctly
2. Proper decimal precision handling with Fixed types
3. Correct order lifecycle management with timing delays
4. Graceful error handling for edge cases
5. Performance within acceptable limits

## Running Examples

In addition to tests, runnable examples are provided:
```bash
# Basic connectivity test
cargo run --example binance_basic

# Advanced trading bot demo
cargo run --example binance_advanced

# WebSocket streaming
cargo run --example binance_websocket

# User stream monitoring
cargo run --example binance_user_stream
```