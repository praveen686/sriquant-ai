# Binance API Integration Guide

This guide covers the complete Binance integration in SriQuant.ai, including REST API, WebSocket market data streams, and user data streams.

## Table of Contents
- [Overview](#overview)
- [REST API](#rest-api)
- [WebSocket Market Data](#websocket-market-data)
- [User Data Streams](#user-data-streams)
- [Examples](#examples)
- [Best Practices](#best-practices)

## Overview

SriQuant.ai provides high-performance Binance integration with:
- **REST API**: Trading, account management, and market data
- **WebSocket**: Real-time market data streaming
- **User Streams**: Real-time account and order updates
- **Testnet Support**: Full testnet environment for testing

### Architecture

```
sriquant-ai/crates/exchanges/src/binance/
├── mod.rs          # Module exports and main exchange client
├── rest.rs         # REST API implementation
├── websocket.rs    # Market data WebSocket client
├── user_stream.rs  # User data stream WebSocket client
├── auth.rs         # HMAC-SHA256 authentication
├── types.rs        # Binance-specific types
└── connection.rs   # Connection management
```

## REST API

### Configuration

```rust
use sriquant_exchanges::binance::{BinanceConfig, BinanceRestClient};

// Production configuration
let config = BinanceConfig::default()
    .with_credentials(api_key, api_secret);

// Testnet configuration
let config = BinanceConfig::testnet()
    .with_env_credentials()?; // Loads from BINANCE_API_KEY and BINANCE_SECRET_KEY

// Create REST client
let client = BinanceRestClient::new(config).await?;
```

### Available Endpoints

#### Public Endpoints (No Authentication Required)
- `ping()` - Test connectivity
- `server_time()` - Get server time
- `exchange_info()` - Get exchange information
- `ticker_24hr(symbol)` - Get 24hr ticker statistics
- `order_book(symbol, limit)` - Get order book
- `recent_trades(symbol, limit)` - Get recent trades
- `get_symbol_price_ticker(symbol)` - Get current price

#### Private Endpoints (Authentication Required)
- `get_account_info()` - Get account information
- `new_order(params)` - Place a new order (low-level)
- `place_order(symbol, side, type, qty, price)` - Place a new order (simplified)
- `test_new_order(params)` - Test order placement (no execution)
- `cancel_order(symbol, order_id)` - Cancel an order
- `query_order(symbol, order_id)` - Query order status
- `open_orders(symbol)` - Get open orders
- `my_trades(symbol, limit)` - Get account trades
- `get_all_orders(symbol, limit, start_time, end_time)` - Get all orders history
- `get_order_trades(symbol, order_id)` - Get trades for a specific order
- `get_24hr_ticker(symbol)` - Get 24hr ticker statistics (alias for ticker_24hr)
- `get_klines(symbol, interval, start_time, end_time, limit)` - Get historical candlestick data

#### User Stream Management
- `create_listen_key()` - Create a listen key for user streams
- `keepalive_listen_key(listen_key)` - Extend listen key validity
- `close_listen_key(listen_key)` - Close user stream

### Order Placement Examples

#### Using the simplified API
```rust
use sriquant_exchanges::binance::{BinanceRestClient, BinanceConfig};
use sriquant_exchanges::types::{OrderSide, OrderType};
use sriquant_core::prelude::Fixed;

// Place a limit order
let order = client.place_order(
    "BTCUSDT",
    OrderSide::Buy,
    OrderType::Limit,
    Fixed::from_str_exact("0.001")?,     // 0.001 BTC
    Some(Fixed::from_str_exact("50000.00")?), // $50,000 per BTC
).await?;
println!("Order placed: ID={}", order.order_id);

// Place a market order
let order = client.place_order(
    "BTCUSDT",
    OrderSide::Sell,
    OrderType::Market,
    Fixed::from_str_exact("0.001")?,
    None, // No price needed for market orders
).await?;
```

#### Using the low-level API
```rust
use sriquant_exchanges::binance::rest::TestOrderParams;

// Create order parameters
let order_params = TestOrderParams {
    symbol: "BTCUSDT",
    side: "BUY",
    order_type: "LIMIT",
    quantity: Some("0.001"),
    price: Some("50000.00"),
    time_in_force: Some("GTC"),
    stop_price: None,
    iceberg_qty: None,
};

// Place order
let order = client.new_order(&order_params).await?;
println!("Order placed: ID={}", order.order_id);
```

### Order History and Trades

```rust
// Get all orders for the last 24 hours
let start_time = nanos() / 1_000_000 - 24 * 60 * 60 * 1000;
let orders = client.get_all_orders("BTCUSDT", Some(100), Some(start_time), None).await?;
for order in orders {
    println!("Order {}: {} {} @ {} - Status: {}", 
        order.order_id, order.side, order.orig_qty, order.price, order.status);
}

// Get trades for a specific order
let trades = client.get_order_trades("BTCUSDT", order_id).await?;
for trade in trades {
    println!("Trade {}: {} @ {} - Fee: {} {}", 
        trade.id, trade.qty, trade.price, trade.commission, trade.commission_asset);
}
```

### Market Data

```rust
// Get 24hr ticker statistics
let ticker = client.get_24hr_ticker("BTCUSDT").await?;
println!("BTC Price: {} Change: {}% Volume: {} BTC",
    ticker.last_price, ticker.price_change_percent, ticker.volume);
println!("High: {} Low: {} VWAP: {}",
    ticker.high_price, ticker.low_price, ticker.weighted_avg_price);

// Get historical candlestick data
let klines = client.get_klines("BTCUSDT", "1h", None, None, Some(24)).await?;
for kline in klines {
    let (open, high, low, close, volume) = kline.ohlcv()?;
    println!("Time: {} O:{} H:{} L:{} C:{} V:{}", 
        kline.open_time, open, high, low, close, volume);
}
```

## WebSocket Market Data

### Market Data Client

```rust
use sriquant_exchanges::binance::BinanceWebSocketClient;

// Create WebSocket client
let mut ws_client = BinanceWebSocketClient::new(config);

// Connect to multiple streams
let streams = vec![
    "btcusdt@ticker",      // 24hr ticker
    "btcusdt@depth5@100ms", // Order book (5 levels, 100ms updates)
    "btcusdt@trade",       // Real-time trades
    "btcusdt@kline_1m"     // 1-minute candlesticks
];
ws_client.connect_multi_stream(streams).await?;

// Receive messages
loop {
    match ws_client.receive_message().await {
        Ok(MarketDataEvent::Ticker(ticker)) => {
            println!("Price: {} Change: {}%", ticker.price, ticker.price_change);
        },
        Ok(MarketDataEvent::Depth(depth)) => {
            println!("Best bid: {:?} Best ask: {:?}", 
                depth.bids.first(), depth.asks.first());
        },
        Ok(MarketDataEvent::Trade(trade)) => {
            println!("Trade: {} {} @ {}", 
                trade.side, trade.quantity, trade.price);
        },
        Ok(MarketDataEvent::Kline(kline)) => {
            println!("Candle: O:{} H:{} L:{} C:{} V:{}", 
                kline.open, kline.high, kline.low, kline.close, kline.volume);
        },
        Err(e) => {
            eprintln!("WebSocket error: {}", e);
            break;
        }
    }
}
```

### Available Market Data Streams

- **Ticker**: `symbol@ticker` - 24hr rolling window statistics
- **Order Book**: `symbol@depth` or `symbol@depth5@100ms` - Order book updates
- **Trades**: `symbol@trade` - Individual trade updates
- **Klines**: `symbol@kline_interval` - Candlestick data (1m, 5m, 15m, etc.)

## User Data Streams

User data streams provide real-time updates for:
- Account balance changes
- Order status updates (NEW, PARTIALLY_FILLED, FILLED, CANCELED, etc.)
- Trade executions

### Setting Up User Streams

```rust
use sriquant_exchanges::binance::{BinanceUserStreamClient, UserDataEvent};

// 1. Create listen key using REST client
let rest_client = BinanceRestClient::new(config.clone()).await?;
let listen_key = rest_client.create_listen_key().await?;

// 2. Connect to user stream
let mut user_stream = BinanceUserStreamClient::new(config);
user_stream.connect(&listen_key).await?;

// 3. Process events
loop {
    match user_stream.receive_event().await {
        Ok(UserDataEvent::OrderUpdate(order)) => {
            println!("Order Update:");
            println!("  Symbol: {}", order.symbol);
            println!("  Order ID: {}", order.order_id);
            println!("  Status: {}", order.order_status);
            println!("  Filled: {} / {}", 
                order.cumulative_filled_quantity, 
                order.order_quantity);
        },
        Ok(UserDataEvent::AccountUpdate(account)) => {
            println!("Account Update:");
            for balance in &account.balances {
                if balance.free > Fixed::ZERO || balance.locked > Fixed::ZERO {
                    println!("  {}: Free={} Locked={}", 
                        balance.asset, balance.free, balance.locked);
                }
            }
        },
        Ok(UserDataEvent::BalanceUpdate(balance)) => {
            println!("Balance Update: {} {}", 
                balance.asset, balance.balance_delta);
        },
        Err(e) => {
            eprintln!("User stream error: {}", e);
            break;
        }
    }
}

// 4. Keep alive (every 30 minutes)
rest_client.keepalive_listen_key(&listen_key).await?;

// 5. Close when done
rest_client.close_listen_key(&listen_key).await?;
```

### User Stream Event Types

#### OrderUpdate
Triggered when an order is:
- Created (NEW)
- Partially filled (PARTIALLY_FILLED)
- Completely filled (FILLED)
- Canceled (CANCELED)
- Rejected (REJECTED)
- Expired (EXPIRED)

Fields include:
- `order_id`: Exchange order ID
- `client_order_id`: Your custom order ID
- `symbol`: Trading pair
- `side`: BUY or SELL
- `order_type`: LIMIT, MARKET, etc.
- `order_status`: Current status
- `order_price`: Order price
- `order_quantity`: Original quantity
- `cumulative_filled_quantity`: Amount filled
- `last_executed_price`: Price of last fill
- `commission_amount`: Trading fee
- `transaction_time`: Event timestamp

#### AccountUpdate
Provides a snapshot of all account balances when any balance changes.

#### BalanceUpdate
Shows the change in a specific asset balance.

## Examples

### Complete Trading Example

See `tests/examples/user_stream_with_orders.rs` for a complete example that:
1. Connects to user stream
2. Places orders
3. Shows real-time updates
4. Cancels orders
5. Demonstrates proper cleanup

Run with:
```bash
cargo run --example user_stream_with_orders
```

### Available Examples

- `binance_basic.rs` - Basic connectivity test
- `binance_rest.rs` - REST API operations
- `binance_websocket.rs` - Market data streaming
- `binance_user_stream.rs` - User data stream monitoring
- `place_simple_order.rs` - Simple order placement
- `user_stream_with_orders.rs` - Combined demo

### Demo Script

A convenience script is provided to demonstrate user streams:
```bash
./scripts/run_user_stream_demo.sh
```

This script:
1. Starts the user stream monitor
2. Waits for connection
3. Places test orders to trigger events

## Best Practices

### Connection Management
- Use connection pooling for REST API
- Implement automatic reconnection for WebSockets
- Keep listen keys alive every 30 minutes
- Close listen keys when shutting down

### Error Handling
- Handle rate limits gracefully
- Implement exponential backoff for retries
- Log all errors with context
- Monitor connection health

### Performance Optimization
- Use monoio runtime for single-threaded async
- Bind to dedicated CPU cores for trading
- Minimize allocations in hot paths
- Use fixed-point arithmetic for prices

### Security
- Never log API credentials
- Use environment variables for keys
- Rotate API keys regularly
- Use IP whitelisting on Binance

### Testing
- Always test on testnet first
- Use small quantities for initial tests
- Monitor all order states
- Implement proper cleanup

## Troubleshooting

### Common Issues

1. **"Listen key not found"**
   - Listen keys expire after 60 minutes
   - Call keepalive every 30 minutes
   - Recreate if expired

2. **"Invalid API key/secret"**
   - Check environment variables
   - Ensure testnet keys for testnet
   - Verify key permissions

3. **"Timestamp for request is outside recvWindow"**
   - System time may be off
   - Sync with NTP server
   - Increase recv_window if needed

4. **WebSocket disconnections**
   - Normal after 24 hours
   - Implement auto-reconnection
   - Monitor connection health

### Debug Tips

Enable debug logging:
```rust
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

Monitor performance:
```rust
let timer = PerfTimer::start("operation");
// ... operation ...
timer.log_elapsed();
```

## API Limits

### REST API
- Weight limits vary by endpoint
- 1200 requests per minute (testnet may differ)
- Order limits: 10 orders/second, 100,000 orders/day

### WebSocket
- 5 connections per IP
- 300 connections per attempt every 5 minutes
- Combined streams limited to 1024 per connection

### User Streams
- Listen key valid for 60 minutes
- Can be extended with keepalive
- Maximum 1 user stream per listen key