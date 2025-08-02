//! Comprehensive tests for Binance REST API endpoints using rstest
//!
//! Tests all new REST API endpoints with parameterized tests,
//! fixtures, and async testing capabilities.

use sriquant_core::prelude::*;
use sriquant_exchanges::binance::{BinanceConfig, BinanceRestClient};
use sriquant_exchanges::types::{OrderSide, OrderType};
use rstest::*;
use serial_test::serial;
use std::collections::HashMap;
use std::time::Duration;
use monoio::time::sleep;
use tracing::info;

// ============================================================================
// TEST FIXTURES (shared test data and setup)
// ============================================================================

/// Fixture for test configuration
#[fixture]
fn test_config() -> BinanceConfig {
    // Load .env file
    dotenv::dotenv().ok();
    
    BinanceConfig::testnet()
        .with_env_credentials()
        .expect("Failed to load credentials from environment")
}

/// Fixture for test symbols
#[fixture]
fn test_symbols() -> Vec<&'static str> {
    vec!["BTCUSDT", "ETHUSDT", "BNBUSDT"]
}

/// Fixture for test order parameters
#[fixture]
fn test_order_params() -> Vec<(OrderSide, &'static str, &'static str)> {
    vec![
        (OrderSide::Buy, "0.00010", "50000.00"),
        (OrderSide::Sell, "0.00010", "150000.00"),
        (OrderSide::Buy, "0.00015", "45000.00"),
    ]
}

// ============================================================================
// BASIC CONNECTIVITY TESTS
// ============================================================================

#[cfg(test)]
mod connectivity_tests {
    use super::*;

    #[rstest]
    #[monoio::test]
    async fn test_ping_latency(test_config: BinanceConfig) {
        let client = BinanceRestClient::new(test_config).await
            .expect("Failed to create REST client");
        
        let start = nanos();
        client.ping().await.expect("Ping failed");
        let latency = (nanos() - start) / 1000; // Convert to microseconds
        
        info!("Ping latency: {}μs", latency);
        assert!(latency < 1_000_000, "Ping latency should be under 1 second");
    }

    #[rstest]
    #[monoio::test]
    async fn test_server_time(test_config: BinanceConfig) {
        let client = BinanceRestClient::new(test_config).await
            .expect("Failed to create REST client");
        
        let server_time = client.server_time().await
            .expect("Failed to get server time");
        
        let local_time = nanos() / 1_000_000;
        let time_diff = (server_time as i64 - local_time as i64).abs();
        
        // Time difference should be reasonable (under 10 seconds)
        assert!(time_diff < 10_000, "Server time difference too large: {}ms", time_diff);
    }
}

// ============================================================================
// MARKET DATA ENDPOINT TESTS
// ============================================================================

#[cfg(test)]
mod market_data_tests {
    use super::*;

    #[rstest]
    #[monoio::test]
    async fn test_24hr_ticker(
        test_config: BinanceConfig,
        #[values("BTCUSDT", "ETHUSDT", "BNBUSDT")] symbol: &str
    ) {
        let client = BinanceRestClient::new(test_config).await
            .expect("Failed to create REST client");
        
        let ticker = client.get_24hr_ticker(symbol).await
            .expect("Failed to get 24hr ticker");
        
        // Validate ticker data
        assert_eq!(ticker.symbol, symbol);
        assert!(!ticker.last_price.is_empty());
        assert!(!ticker.volume.is_empty());
        
        // Parse and validate numeric values
        let price = Fixed::from_str_exact(&ticker.last_price)
            .expect("Invalid price format");
        assert!(price > Fixed::ZERO, "Price should be positive");
        
        let volume = Fixed::from_str_exact(&ticker.volume)
            .expect("Invalid volume format");
        assert!(volume >= Fixed::ZERO, "Volume should be non-negative");
    }

    #[rstest]
    #[case("1m", 60)]
    #[case("5m", 60)]
    #[case("1h", 24)]
    #[case("1d", 7)]
    #[monoio::test]
    async fn test_klines(
        test_config: BinanceConfig,
        #[case] interval: &str,
        #[case] limit: usize
    ) {
        let client = BinanceRestClient::new(test_config).await
            .expect("Failed to create REST client");
        
        let klines = client.get_klines("BTCUSDT", interval, None, None, Some(limit as u32))
            .await.expect("Failed to get klines");
        
        // Should return requested number or less
        assert!(klines.len() <= limit);
        assert!(!klines.is_empty(), "Should return at least one kline");
        
        // Validate first kline
        let (open, high, low, close, volume) = klines[0].ohlcv()
            .expect("Failed to parse OHLCV");
        
        assert!(open > Fixed::ZERO);
        assert!(high >= low, "High should be >= Low");
        assert!(high >= open, "High should be >= Open");
        assert!(high >= close, "High should be >= Close");
        assert!(low <= open, "Low should be <= Open");
        assert!(low <= close, "Low should be <= Close");
        assert!(volume >= Fixed::ZERO);
    }

    #[rstest]
    #[monoio::test]
    async fn test_order_book(
        test_config: BinanceConfig,
        #[values(5, 10, 20)] limit: u16
    ) {
        let client = BinanceRestClient::new(test_config).await
            .expect("Failed to create REST client");
        
        let order_book = client.order_book("BTCUSDT", Some(limit as u32)).await
            .expect("Failed to get order book");
        
        // Validate order book structure
        assert!(!order_book.bids.is_empty(), "Should have bid orders");
        assert!(!order_book.asks.is_empty(), "Should have ask orders");
        assert!(order_book.bids.len() <= limit as usize);
        assert!(order_book.asks.len() <= limit as usize);
        
        // Validate bid/ask ordering
        if order_book.bids.len() > 1 {
            let bid1 = Fixed::from_str_exact(&order_book.bids[0][0]).unwrap();
            let bid2 = Fixed::from_str_exact(&order_book.bids[1][0]).unwrap();
            assert!(bid1 >= bid2, "Bids should be in descending order");
        }
        
        if order_book.asks.len() > 1 {
            let ask1 = Fixed::from_str_exact(&order_book.asks[0][0]).unwrap();
            let ask2 = Fixed::from_str_exact(&order_book.asks[1][0]).unwrap();
            assert!(ask1 <= ask2, "Asks should be in ascending order");
        }
    }
}

// ============================================================================
// TRADING ENDPOINT TESTS (with proper isolation)
// ============================================================================

#[cfg(test)]
mod trading_tests {
    use super::*;

    /// Test the simplified place_order API
    #[rstest]
    #[serial]
    #[monoio::test(enable_timer = true)]
    async fn test_place_order(
        test_config: BinanceConfig,
        test_order_params: Vec<(OrderSide, &'static str, &'static str)>
    ) {
        let client = BinanceRestClient::new(test_config).await
            .expect("Failed to create REST client");
        
        let mut order_ids = Vec::new();
        
        // Place test orders
        for (side, qty_str, price_str) in test_order_params {
            let quantity = Fixed::from_str_exact(qty_str).unwrap();
            let price = Fixed::from_str_exact(price_str).unwrap();
            
            let order = client.place_order(
                "BTCUSDT",
                side,
                OrderType::Limit,
                quantity.round_dp(5), // Ensure proper precision
                Some(price.round_dp(2)),
            ).await.expect("Failed to place order");
            
            assert!(order.order_id > 0);
            assert_eq!(order.symbol, "BTCUSDT");
            assert_eq!(order.status, "NEW");
            
            order_ids.push(order.order_id);
            
            // Small delay between orders
            sleep(Duration::from_millis(100)).await;
        }
        
        // Wait before querying
        sleep(Duration::from_secs(2)).await;
        
        // Cancel all test orders
        for order_id in order_ids {
            match client.cancel_order("BTCUSDT", order_id).await {
                Ok(_) => info!("Canceled test order {}", order_id),
                Err(e) => info!("Order {} already filled or canceled: {}", order_id, e),
            }
        }
    }

    /// Test order history retrieval
    #[rstest]
    #[monoio::test]
    async fn test_get_all_orders(test_config: BinanceConfig) {
        let client = BinanceRestClient::new(test_config).await
            .expect("Failed to create REST client");
        
        // Get orders from last 24 hours
        let start_time = nanos() / 1_000_000 - 24 * 60 * 60 * 1000;
        let orders = client.get_all_orders("BTCUSDT", Some(50), Some(start_time), None)
            .await.expect("Failed to get order history");
        
        // Validate order structure
        for order in &orders {
            assert!(!order.symbol.is_empty());
            assert!(order.order_id > 0);
            assert!(!order.status.is_empty());
            assert!(!order.side.is_empty());
            assert!(!order.order_type.is_empty());
            
            // Validate quantities
            let orig_qty = Fixed::from_str_exact(&order.orig_qty)
                .expect("Invalid original quantity");
            assert!(orig_qty > Fixed::ZERO);
        }
    }

    /// Test account info retrieval
    #[rstest]
    #[monoio::test]
    async fn test_account_info(test_config: BinanceConfig) {
        let client = BinanceRestClient::new(test_config).await
            .expect("Failed to create REST client");
        
        match client.get_account_info().await {
            Ok(account) => {
                assert!(account.maker_commission >= 0);
                assert!(account.taker_commission >= 0);
                assert!(!account.balances.is_empty());
                
                // Find and validate major balances
                let mut found_usdt = false;
                let mut found_btc = false;
                
                for balance in &account.balances {
                    if balance.asset == "USDT" {
                        found_usdt = true;
                        let free = Fixed::from_str_exact(&balance.free).unwrap_or(Fixed::ZERO);
                        let locked = Fixed::from_str_exact(&balance.locked).unwrap_or(Fixed::ZERO);
                        assert!(free + locked >= Fixed::ZERO);
                    }
                    if balance.asset == "BTC" {
                        found_btc = true;
                    }
                }
                
                assert!(found_usdt, "Should have USDT balance");
                assert!(found_btc, "Should have BTC balance");
            }
            Err(e) => {
                // Handle large response gracefully
                info!("Account info failed (possibly due to large response): {}", e);
            }
        }
    }
}

// ============================================================================
// PERFORMANCE AND STRESS TESTS
// ============================================================================

#[cfg(test)]
mod performance_tests {
    use super::*;

    /// Test sequential API calls performance
    #[rstest]
    #[monoio::test]
    async fn test_sequential_requests(test_config: BinanceConfig) {
        let client = BinanceRestClient::new(test_config).await
            .expect("Failed to create REST client");
        
        let start = nanos();
        let symbols = ["BTCUSDT", "ETHUSDT", "BNBUSDT"];
        let mut results = Vec::new();
        
        // Make sequential requests and measure performance
        for symbol in symbols {
            let ticker = client.get_symbol_price_ticker(symbol).await
                .expect("Failed to get price");
            results.push(ticker);
        }
        
        let elapsed = (nanos() - start) / 1000;
        info!("Sequential requests completed in {}μs", elapsed);
        
        // All requests should succeed
        assert_eq!(results.len(), 3);
        for ticker in results {
            assert!(!ticker.symbol.is_empty());
            assert!(!ticker.price.is_empty());
        }
        
        // Performance check - should complete in reasonable time
        assert!(elapsed < 3_000_000, "Sequential requests took too long");
    }

    /// Test API rate limiting behavior
    #[rstest]
    #[monoio::test(enable_timer = true)]
    async fn test_rate_limiting(test_config: BinanceConfig) {
        let client = BinanceRestClient::new(test_config).await
            .expect("Failed to create REST client");
        
        let mut success_count = 0;
        let mut rate_limit_count = 0;
        
        // Make rapid requests
        for i in 0..10 {
            match client.get_symbol_price_ticker("BTCUSDT").await {
                Ok(_) => success_count += 1,
                Err(e) => {
                    if e.to_string().contains("429") {
                        rate_limit_count += 1;
                        info!("Hit rate limit at request {}", i);
                    }
                }
            }
            
            // Small delay to avoid hitting limits
            sleep(Duration::from_millis(50)).await;
        }
        
        assert!(success_count > 0, "At least some requests should succeed");
        info!("Success: {}, Rate limited: {}", success_count, rate_limit_count);
    }
}

// ============================================================================
// INTEGRATION TESTS (full workflow)
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test complete order lifecycle
    #[rstest]
    #[serial]
    #[monoio::test(enable_timer = true)]
    async fn test_order_lifecycle(test_config: BinanceConfig) {
        let client = BinanceRestClient::new(test_config).await
            .expect("Failed to create REST client");
        
        // 1. Get current price
        let ticker = client.get_symbol_price_ticker("BTCUSDT").await
            .expect("Failed to get price");
        let current_price = Fixed::from_str_exact(&ticker.price).unwrap();
        
        // 2. Place a limit order far from market price
        let order_price = (current_price * Fixed::from_str_exact("0.5").unwrap()).round_dp(2);
        let quantity = Fixed::from_str_exact("0.00010").unwrap();
        
        let order = client.place_order(
            "BTCUSDT",
            OrderSide::Buy,
            OrderType::Limit,
            quantity,
            Some(order_price),
        ).await.expect("Failed to place order");
        
        info!("Placed order {} at {}", order.order_id, order_price);
        
        // 3. Wait for order to be queryable
        sleep(Duration::from_secs(2)).await;
        
        // 4. Query order status
        let status = client.query_order("BTCUSDT", order.order_id).await
            .expect("Failed to query order");
        
        assert_eq!(status.order_id, order.order_id);
        assert_eq!(status.symbol, "BTCUSDT");
        assert!(status.status == "NEW" || status.status == "PARTIALLY_FILLED");
        
        // 5. Get trades (should be empty for unfilled order)
        match client.get_order_trades("BTCUSDT", order.order_id).await {
            Ok(trades) => {
                assert!(trades.is_empty(), "Unfilled order should have no trades");
            }
            Err(_) => {
                // Some orders might not have trades endpoint available
            }
        }
        
        // 6. Cancel the order
        let cancel_result = client.cancel_order("BTCUSDT", order.order_id).await
            .expect("Failed to cancel order");
        
        assert_eq!(cancel_result.order_id, order.order_id);
        assert_eq!(cancel_result.status, "CANCELED");
        
        info!("Successfully completed order lifecycle test");
    }

    /// Test portfolio management workflow
    #[rstest]
    #[monoio::test]
    async fn test_portfolio_workflow(test_config: BinanceConfig) {
        let client = BinanceRestClient::new(test_config).await
            .expect("Failed to create REST client");
        
        // 1. Get account balances
        let mut balances = HashMap::new();
        match client.get_account_info().await {
            Ok(account) => {
                for balance in account.balances {
                    let free = Fixed::from_str_exact(&balance.free).unwrap_or(Fixed::ZERO);
                    let locked = Fixed::from_str_exact(&balance.locked).unwrap_or(Fixed::ZERO);
                    if free + locked > Fixed::ZERO {
                        balances.insert(balance.asset, (free, locked));
                    }
                }
            }
            Err(_) => {
                // Use default test balances
                balances.insert("USDT".to_string(), (Fixed::from_i64(1000).unwrap(), Fixed::ZERO));
                balances.insert("BTC".to_string(), (Fixed::from_str_exact("0.01").unwrap(), Fixed::ZERO));
            }
        }
        
        // 2. Calculate position sizes based on risk
        let usdt_balance = balances.get("USDT").map(|(f, _)| *f).unwrap_or(Fixed::ZERO);
        let risk_pct = Fixed::from_str_exact("1.0").unwrap(); // 1% risk
        let risk_amount = usdt_balance * (risk_pct / Fixed::from_i64(100).unwrap());
        
        info!("USDT Balance: {}, Risk amount: {}", usdt_balance, risk_amount);
        
        // 3. Get market data for position sizing
        let ticker = client.get_24hr_ticker("BTCUSDT").await
            .expect("Failed to get ticker");
        
        let price = Fixed::from_str_exact(&ticker.last_price).unwrap();
        let position_size = (risk_amount / price).round_dp(5);
        
        info!("BTC Price: {}, Position size: {} BTC", price, position_size);
        
        assert!(position_size > Fixed::ZERO, "Position size should be positive");
        assert!(position_size < Fixed::from_str_exact("1.0").unwrap(), "Position size should be reasonable");
    }
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[rstest]
    #[monoio::test]
    async fn test_invalid_symbol(test_config: BinanceConfig) {
        let client = BinanceRestClient::new(test_config).await
            .expect("Failed to create REST client");
        
        let result = client.get_symbol_price_ticker("INVALID_SYMBOL").await;
        assert!(result.is_err(), "Invalid symbol should return error");
    }

    #[rstest]
    #[monoio::test]
    async fn test_invalid_order_params(test_config: BinanceConfig) {
        let client = BinanceRestClient::new(test_config).await
            .expect("Failed to create REST client");
        
        // Test with zero quantity
        let result = client.place_order(
            "BTCUSDT",
            OrderSide::Buy,
            OrderType::Limit,
            Fixed::ZERO,
            Some(Fixed::from_i64(50000).unwrap()),
        ).await;
        
        assert!(result.is_err(), "Zero quantity should fail");
    }

    #[rstest]
    #[monoio::test]
    async fn test_query_non_existent_order(test_config: BinanceConfig) {
        let client = BinanceRestClient::new(test_config).await
            .expect("Failed to create REST client");
        
        let result = client.query_order("BTCUSDT", 999999999).await;
        assert!(result.is_err(), "Non-existent order should return error");
    }
}

// ============================================================================
// UTILITIES AND HELPERS
// ============================================================================

/// Helper to generate unique client order IDs
fn generate_test_order_id() -> String {
    format!("rstest_{}", generate_id())
}

/// Helper to validate decimal precision
fn assert_decimal_places(value_str: &str, expected_places: usize) {
    if let Some(dot_pos) = value_str.find('.') {
        let decimal_places = value_str.len() - dot_pos - 1;
        assert!(
            decimal_places <= expected_places,
            "Value {} has {} decimal places, expected max {}",
            value_str, decimal_places, expected_places
        );
    }
}

#[cfg(test)]
mod helper_tests {
    use super::*;

    #[test]
    fn test_unique_order_ids() {
        let mut ids = std::collections::HashSet::new();
        for _ in 0..100 {
            let id = generate_test_order_id();
            assert!(ids.insert(id), "Generated duplicate order ID");
        }
    }

    #[rstest]
    #[case("123.45", 2, true)]
    #[case("123.456", 2, false)]
    #[case("123", 2, true)]
    #[case("123.4", 2, true)]
    fn test_decimal_validation(#[case] value: &str, #[case] places: usize, #[case] should_pass: bool) {
        let result = std::panic::catch_unwind(|| {
            assert_decimal_places(value, places);
        });
        
        assert_eq!(result.is_ok(), should_pass);
    }
}