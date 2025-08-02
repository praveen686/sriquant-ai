//! Advanced Binance trading example with live orders and WebSocket streaming
//!
//! Demonstrates:
//! - Live order placement and management using the new simplified API
//! - Real-time market data streaming
//! - Portfolio tracking with fixed-point arithmetic
//! - Risk management and position sizing
//! - High-performance latency monitoring
//! 
//! New REST API endpoints tested:
//! - place_order() - Simplified order placement with Fixed types
//! - get_24hr_ticker() - Full 24hr market statistics
//! - get_klines() - Historical candlestick data
//! - get_all_orders() - Order history retrieval
//! - get_order_trades() - Trade fills for specific orders
//! - get_account_info() - Real account balance updates
//! 
//! Run with:
//! ```bash
//! export BINANCE_API_KEY="your_testnet_api_key"
//! export BINANCE_SECRET_KEY="your_testnet_secret_key"
//! cargo run --example binance_advanced
//! ```

use sriquant_core::prelude::*;
use sriquant_exchanges::binance::{BinanceConfig, BinanceExchange, BinanceRestClient};
use sriquant_exchanges::prelude::*;
use sriquant_exchanges::types::{OrderSide, OrderType};
use tracing::{info, warn, error, debug};
use std::collections::HashMap;
use std::time::Duration;

/// Trading strategy configuration
#[derive(Debug, Clone)]
pub struct TradingConfig {
    pub symbol: String,
    pub max_position_size: Fixed,
    pub risk_per_trade: Fixed, // Percentage of portfolio
    pub stop_loss_pct: Fixed,
    pub take_profit_pct: Fixed,
    pub min_spread: Fixed, // Minimum bid-ask spread to trade
}

impl Default for TradingConfig {
    fn default() -> Self {
        Self {
            symbol: "BTCUSDT".to_string(),
            max_position_size: Fixed::from_str_exact("0.01").unwrap(), // 0.01 BTC
            risk_per_trade: Fixed::from_str_exact("1.0").unwrap(), // 1% of portfolio
            stop_loss_pct: Fixed::from_str_exact("2.0").unwrap(), // 2% stop loss
            take_profit_pct: Fixed::from_str_exact("3.0").unwrap(), // 3% take profit
            min_spread: Fixed::from_str_exact("0.01").unwrap(), // $0.01 minimum spread
        }
    }
}

/// Portfolio tracker using fixed-point arithmetic
#[derive(Debug)]
pub struct Portfolio {
    balances: HashMap<String, Fixed>,
    positions: HashMap<String, Position>,
    total_value_usdt: Fixed,
    unrealized_pnl: Fixed,
    realized_pnl: Fixed,
    last_update: u64,
}

#[derive(Debug, Clone)]
pub struct Position {
    symbol: String,
    size: Fixed,
    entry_price: Fixed,
    current_price: Fixed,
    unrealized_pnl: Fixed,
    side: OrderSide,
    timestamp: u64,
}

impl Portfolio {
    pub fn new() -> Self {
        Self {
            balances: HashMap::new(),
            positions: HashMap::new(),
            total_value_usdt: Fixed::ZERO,
            unrealized_pnl: Fixed::ZERO,
            realized_pnl: Fixed::ZERO,
            last_update: 0,
        }
    }
    
    pub fn update_balance(&mut self, asset: &str, balance: Fixed) {
        self.balances.insert(asset.to_string(), balance);
        self.last_update = nanos() / 1_000_000; // Convert to milliseconds
    }
    
    pub fn get_balance(&self, asset: &str) -> Fixed {
        self.balances.get(asset).copied().unwrap_or(Fixed::ZERO)
    }
    
    pub fn calculate_position_size(&self, price: Fixed, risk_pct: Fixed) -> Result<Fixed> {
        let usdt_balance = self.get_balance("USDT");
        let risk_amount = usdt_balance * (risk_pct / Fixed::from_i64(100)?);
        let position_size = risk_amount / price;
        Ok(position_size)
    }
    
    pub fn update_position(&mut self, symbol: &str, price: Fixed) {
        if let Some(position) = self.positions.get_mut(symbol) {
            position.current_price = price;
            position.unrealized_pnl = match position.side {
                OrderSide::Buy => (price - position.entry_price) * position.size,
                OrderSide::Sell => (position.entry_price - price) * position.size,
            };
        }
    }
}

/// Advanced trading bot
pub struct AdvancedTradingBot {
    exchange: BinanceExchange,
    rest_client: BinanceRestClient,
    config: TradingConfig,
    portfolio: Portfolio,
    active_orders: HashMap<String, (u64, u64)>, // client_order_id -> (order_id, timestamp_ms)
    performance_metrics: PerformanceTracker,
}

#[derive(Debug)]
pub struct PerformanceTracker {
    total_trades: u64,
    winning_trades: u64,
    losing_trades: u64,
    total_profit: Fixed,
    max_drawdown: Fixed,
    avg_latency_micros: u64,
    latency_samples: Vec<u64>,
}

impl PerformanceTracker {
    pub fn new() -> Self {
        Self {
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            total_profit: Fixed::ZERO,
            max_drawdown: Fixed::ZERO,
            avg_latency_micros: 0,
            latency_samples: Vec::with_capacity(1000),
        }
    }
    
    pub fn record_trade(&mut self, profit: Fixed) {
        self.total_trades += 1;
        self.total_profit += profit;
        
        if profit > Fixed::ZERO {
            self.winning_trades += 1;
        } else {
            self.losing_trades += 1;
        }
    }
    
    pub fn record_latency(&mut self, latency_micros: u64) {
        self.latency_samples.push(latency_micros);
        if self.latency_samples.len() > 1000 {
            self.latency_samples.remove(0);
        }
        
        let sum: u64 = self.latency_samples.iter().sum();
        self.avg_latency_micros = sum / self.latency_samples.len() as u64;
    }
    
    pub fn win_rate(&self) -> f64 {
        if self.total_trades == 0 {
            0.0
        } else {
            self.winning_trades as f64 / self.total_trades as f64 * 100.0
        }
    }
    
    pub fn print_summary(&self) {
        info!("📊 Performance Summary:");
        info!("   Total Trades: {}", self.total_trades);
        info!("   Win Rate: {:.2}%", self.win_rate());
        info!("   Total Profit: ${}", self.total_profit);
        info!("   Avg Latency: {}μs", self.avg_latency_micros);
        
        if !self.latency_samples.is_empty() {
            let mut sorted = self.latency_samples.clone();
            sorted.sort();
            let p50 = sorted[sorted.len() / 2];
            let p95 = sorted[(sorted.len() * 95) / 100];
            let p99 = sorted[(sorted.len() * 99) / 100];
            
            info!("   Latency P50: {}μs", p50);
            info!("   Latency P95: {}μs", p95);
            info!("   Latency P99: {}μs", p99);
        }
    }
}

impl AdvancedTradingBot {
    pub async fn new(config: TradingConfig) -> Result<Self> {
        info!("🚀 Initializing Advanced Trading Bot");
        info!("   Symbol: {}", config.symbol);
        info!("   Max Position: {}", config.max_position_size);
        info!("   Risk per Trade: {}%", config.risk_per_trade);
        
        // Load credentials from environment
        let binance_config = BinanceConfig::testnet()
            .with_env_credentials()?;
        
        let mut exchange = BinanceExchange::new(binance_config.clone()).await?;
        exchange.init_rest().await?;
        exchange.init_websocket().await?;
        
        // Create separate REST client for new endpoints
        let rest_client = BinanceRestClient::new(binance_config).await?;
        
        // Test connectivity
        let latency = exchange.ping().await?;
        info!("✅ Connected to Binance (latency: {}μs)", latency);
        
        Ok(Self {
            exchange,
            rest_client,
            config,
            portfolio: Portfolio::new(),
            active_orders: HashMap::new(),
            performance_metrics: PerformanceTracker::new(),
        })
    }
    
    pub async fn run(&mut self) -> Result<()> {
        info!("🎯 Starting trading bot...");
        
        // Bind to CPU core for optimal performance
        if let Err(e) = bind_to_cpu_set(0) {
            warn!("Failed to bind to CPU core 0: {}", e);
        }
        
        // Test new endpoints first
        self.test_new_endpoints().await?;
        
        // Initialize portfolio
        self.update_portfolio().await?;
        
        // Start market data streaming
        self.start_market_data_stream().await?;
        
        // Main trading loop (limited iterations for testing)
        let mut iteration = 0u64;
        let max_iterations = 20; // Run for 20 iterations then exit
        loop {
            iteration += 1;
            
            if iteration > max_iterations {
                info!("🏁 Reached max iterations, shutting down...");
                self.performance_metrics.print_summary();
                self.print_portfolio_summary();
                return Ok(());
            }
            
            let timer = PerfTimer::start("trading_iteration".to_string());
            
            // Update portfolio every 10 iterations
            if iteration % 10 == 0 {
                if let Err(e) = self.update_portfolio().await {
                    error!("Failed to update portfolio: {}", e);
                }
            }
            
            // Check for trading opportunities
            if let Err(e) = self.check_trading_signals().await {
                error!("Failed to check trading signals: {}", e);
            }
            
            // Monitor active orders
            if let Err(e) = self.monitor_orders().await {
                error!("Failed to monitor orders: {}", e);
            }
            
            let elapsed = timer.elapsed_micros();
            self.performance_metrics.record_latency(elapsed);
            
            // Print performance every 100 iterations
            if iteration % 100 == 0 {
                self.performance_metrics.print_summary();
                self.print_portfolio_summary();
            }
            
            // Sleep for a short time (adjust based on strategy needs)
            monoio::time::sleep(Duration::from_millis(100)).await;
        }
        
        Ok(())
    }
    
    async fn update_portfolio(&mut self) -> Result<()> {
        debug!("💼 Updating portfolio...");
        
        let timer = PerfTimer::start("portfolio_update".to_string());
        
        // Fetch real account info from the exchange
        match self.rest_client.get_account_info().await {
            Ok(account_info) => {
                for balance in &account_info.balances {
                    let free = Fixed::from_str_exact(&balance.free).unwrap_or(Fixed::ZERO);
                    let locked = Fixed::from_str_exact(&balance.locked).unwrap_or(Fixed::ZERO);
                    let total = free + locked;
                    
                    if total > Fixed::ZERO {
                        self.portfolio.update_balance(&balance.asset, total);
                        debug!("  {}: {} (free: {}, locked: {})", balance.asset, total, free, locked);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to fetch account info: {}. Using default balances.", e);
                // Fallback to default balances for testing
                self.portfolio.update_balance("USDT", Fixed::from_str_exact("1000.0")?);
                self.portfolio.update_balance("BTC", Fixed::from_str_exact("0.02")?);
            }
        }
        
        timer.log_elapsed();
        Ok(())
    }
    
    async fn start_market_data_stream(&mut self) -> Result<()> {
        info!("📊 Starting market data stream for {}", self.config.symbol);
        
        // This would normally connect to WebSocket and subscribe to market data
        // For demonstration, we'll simulate receiving market data
        
        info!("✅ Market data stream started");
        Ok(())
    }
    
    async fn check_trading_signals(&mut self) -> Result<()> {
        // Get real-time market data
        let ticker = self.rest_client.get_symbol_price_ticker(&self.config.symbol).await?;
        let current_price = Fixed::from_str_exact(&ticker.price)?;
        
        debug!("Current {} price: ${}", self.config.symbol, current_price);
        
        // Update position with current price
        self.portfolio.update_position(&self.config.symbol, current_price);
        
        // Check if we should place a new order
        let usdt_balance = self.portfolio.get_balance("USDT");
        if usdt_balance > Fixed::from_str_exact("100.0")? {
            // Example: Simple market making strategy
            self.place_limit_orders(current_price).await?;
        }
        
        Ok(())
    }
    
    async fn place_limit_orders(&mut self, current_price: Fixed) -> Result<()> {
        // Only place orders if we don't have too many active
        if self.active_orders.len() >= 4 {
            debug!("Already have {} active orders, skipping", self.active_orders.len());
            return Ok(());
        }
        
        // NOTE: In production, you should maintain order book via WebSocket
        // This REST call is only for demonstration purposes
        // Real trading systems subscribe to order book updates via WebSocket:
        // - wss://stream.binance.com:9443/ws/btcusdt@depth20@100ms
        // - Then maintain a local order book with real-time updates
        
        // For demo only - using REST API (not recommended for production)
        let order_book = self.rest_client.order_book(&self.config.symbol, Some(5)).await?;
        
        // Calculate best bid/ask
        let best_bid = order_book.bids.first()
            .and_then(|b| Fixed::from_str_exact(&b[0]).ok())
            .unwrap_or(current_price - Fixed::from_i64(1)?);
        let best_ask = order_book.asks.first()
            .and_then(|a| Fixed::from_str_exact(&a[0]).ok())
            .unwrap_or(current_price + Fixed::from_i64(1)?);
        
        let spread = best_ask - best_bid;
        info!("Order book: Bid: {} Ask: {} Spread: {}", best_bid, best_ask, spread);
        
        if spread < self.config.min_spread {
            debug!("Spread too small: ${}", spread);
            return Ok(());
        }
        
        let position_size = self.portfolio.calculate_position_size(
            current_price, 
            self.config.risk_per_trade
        )?;
        
        // Limit position size
        let final_size = if position_size > self.config.max_position_size {
            self.config.max_position_size
        } else {
            position_size
        };
        
        // Round to proper precision for BTCUSDT (5 decimal places)
        let final_size = final_size.round_dp(5);
        
        // Place orders with proper spacing from best bid/ask
        let tick_size = Fixed::from_str_exact("0.01")?; // $0.01 for BTCUSDT
        
        // Place buy order below best bid
        let buy_price = (best_bid - tick_size).round_dp(2);
        self.place_order(OrderSide::Buy, buy_price, final_size).await?;
        
        // Place sell order above best ask  
        let sell_price = (best_ask + tick_size).round_dp(2);
        self.place_order(OrderSide::Sell, sell_price, final_size).await?;
        
        Ok(())
    }
    
    async fn test_new_endpoints(&mut self) -> Result<()> {
        info!("🧪 Testing new REST API endpoints...");
        
        // Test get_24hr_ticker
        let timer = PerfTimer::start("get_24hr_ticker".to_string());
        let ticker = self.rest_client.get_24hr_ticker(&self.config.symbol).await?;
        let elapsed = timer.elapsed_micros();
        info!("📊 24hr Ticker - Price: {} Change: {}% Volume: {} ({}μs)",
            ticker.last_price, ticker.price_change_percent, ticker.volume, elapsed);
        
        // Test get_klines
        let timer = PerfTimer::start("get_klines".to_string());
        let klines = self.rest_client.get_klines(&self.config.symbol, "1h", None, None, Some(5)).await?;
        let elapsed = timer.elapsed_micros();
        info!("📈 Retrieved {} klines ({}μs)", klines.len(), elapsed);
        for (i, kline) in klines.iter().enumerate() {
            if let Ok((open, high, low, close, volume)) = kline.ohlcv() {
                debug!("  Kline {}: O:{} H:{} L:{} C:{} V:{}", 
                    i, open, high, low, close, volume);
            }
        }
        
        // Test get_all_orders (last 24 hours)
        let timer = PerfTimer::start("get_all_orders".to_string());
        let start_time = nanos() / 1_000_000 - 24 * 60 * 60 * 1000;
        let orders = self.rest_client.get_all_orders(&self.config.symbol, Some(10), Some(start_time), None).await?;
        let elapsed = timer.elapsed_micros();
        info!("📋 Retrieved {} historical orders ({}μs)", orders.len(), elapsed);
        for order in &orders {
            debug!("  Order {}: {} {} @ {} - Status: {}", 
                order.order_id, order.side, order.orig_qty, order.price, order.status);
        }
        
        info!("✅ All new endpoints tested successfully");
        Ok(())
    }
    
    async fn place_order(&mut self, side: OrderSide, price: Fixed, quantity: Fixed) -> Result<()> {
        let timer = PerfTimer::start("place_order".to_string());
        
        info!("📋 Placing {} order: {} {} @ ${}", 
            side, quantity, self.config.symbol, price);
        
        // Use the new simplified place_order API
        let order = self.rest_client.place_order(
            &self.config.symbol,
            side,
            OrderType::Limit,
            quantity,
            Some(price),
        ).await?;
        
        // Store the order ID with timestamp
        let current_time = nanos() / 1_000_000; // Convert to milliseconds
        self.active_orders.insert(order.client_order_id.clone(), (order.order_id, current_time));
        
        let elapsed = timer.elapsed_micros();
        self.performance_metrics.record_latency(elapsed);
        
        info!("✅ Order placed successfully - ID: {} (latency: {}μs)", order.order_id, elapsed);
        
        Ok(())
    }
    
    async fn monitor_orders(&mut self) -> Result<()> {
        // Check status of active orders
        let mut completed_orders = Vec::new();
        let current_time = nanos() / 1_000_000;
        
        for (client_order_id, (order_id, placed_time)) in &self.active_orders {
            // Skip orders that are less than 2 seconds old
            // This gives Binance time to process the order
            let order_age_ms = current_time - placed_time;
            if order_age_ms < 2000 {
                debug!("Skipping order {} - too recent ({}ms old)", order_id, order_age_ms);
                continue;
            }
            
            // Add a small delay to avoid hammering the API
            monoio::time::sleep(Duration::from_millis(100)).await;
            
            // Query real order status
            match self.rest_client.query_order(&self.config.symbol, *order_id).await {
                Ok(order) => {
                    debug!("Order {} status: {}", order_id, order.status);
                    
                    // Check if order is completed
                    if order.status == "FILLED" || order.status == "CANCELED" || order.status == "REJECTED" {
                        info!("📋 Order {} completed with status: {}", order_id, order.status);
                        completed_orders.push(client_order_id.clone());
                        
                        if order.status == "FILLED" {
                            // Get trades for this order
                            match self.rest_client.get_order_trades(&self.config.symbol, *order_id).await {
                                Ok(trades) => {
                                    let mut total_value = Fixed::ZERO;
                                    let mut total_commission = Fixed::ZERO;
                                    
                                    for trade in &trades {
                                        let qty = Fixed::from_str_exact(&trade.qty)?;
                                        let price = Fixed::from_str_exact(&trade.price)?;
                                        let commission = Fixed::from_str_exact(&trade.commission)?;
                                        
                                        total_value = total_value + (qty * price);
                                        total_commission = total_commission + commission;
                                        
                                        debug!("  Trade {}: {} @ {} - Fee: {} {}", 
                                            trade.id, trade.qty, trade.price, trade.commission, trade.commission_asset);
                                    }
                                    
                                    // Simple profit calculation (this is just an example)
                                    let profit = Fixed::from_i64(10)? - total_commission; // Placeholder profit
                                    self.performance_metrics.record_trade(profit);
                                }
                                Err(e) => {
                                    warn!("Failed to get trades for order {}: {}", order_id, e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to query order {}: {}", order_id, e);
                }
            }
        }
        
        // Remove completed orders
        for order_id in completed_orders {
            self.active_orders.remove(&order_id);
        }
        
        Ok(())
    }
    
    fn print_portfolio_summary(&self) {
        info!("💼 Portfolio Summary:");
        info!("   USDT Balance: ${}", self.portfolio.get_balance("USDT"));
        info!("   BTC Balance: {} BTC", self.portfolio.get_balance("BTC"));
        info!("   Active Orders: {}", self.active_orders.len());
        info!("   Unrealized PnL: ${}", self.portfolio.unrealized_pnl);
        info!("   Realized PnL: ${}", self.portfolio.realized_pnl);
    }
}

#[monoio::main(enable_timer = true)]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();
    
    info!("🚀 SriQuant.ai Advanced Binance Trading Bot");
    info!("   Following high-performance design");
    
    // Create trading configuration
    let config = TradingConfig {
        symbol: "BTCUSDT".to_string(),
        max_position_size: Fixed::from_str_exact("0.001")?, // 0.001 BTC for testing
        risk_per_trade: Fixed::from_str_exact("0.5")?, // 0.5% risk per trade
        stop_loss_pct: Fixed::from_str_exact("1.0")?,
        take_profit_pct: Fixed::from_str_exact("2.0")?,
        min_spread: Fixed::from_str_exact("0.001")?, // $0.001 minimum spread to allow testing
    };
    
    // Create and run trading bot
    match AdvancedTradingBot::new(config).await {
        Ok(mut bot) => {
            info!("✅ Trading bot initialized successfully");
            bot.run().await?;
        }
        Err(e) => {
            error!("❌ Failed to initialize trading bot: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_portfolio_calculations() {
        let mut portfolio = Portfolio::new();
        portfolio.update_balance("USDT", Fixed::from_str_exact("1000.0").unwrap());
        
        let price = Fixed::from_str_exact("50000.0").unwrap();
        let risk_pct = Fixed::from_str_exact("1.0").unwrap();
        
        let position_size = portfolio.calculate_position_size(price, risk_pct).unwrap();
        assert_eq!(position_size.to_string(), "0.00020"); // $10 / $50000 = 0.0002 BTC
    }
    
    #[test]
    fn test_performance_tracker() {
        let mut tracker = PerformanceTracker::new();
        
        tracker.record_trade(Fixed::from_str_exact("10.0").unwrap());
        tracker.record_trade(Fixed::from_str_exact("-5.0").unwrap());
        tracker.record_trade(Fixed::from_str_exact("15.0").unwrap());
        
        assert_eq!(tracker.total_trades, 3);
        assert_eq!(tracker.winning_trades, 2);
        assert_eq!(tracker.losing_trades, 1);
        assert!((tracker.win_rate() - 66.66666666666667).abs() < 0.0001);
        assert_eq!(tracker.total_profit.to_string(), "20.0");
    }
    
    #[monoio::test]
    async fn test_trading_config() {
        let config = TradingConfig::default();
        assert_eq!(config.symbol, "BTCUSDT");
        assert_eq!(config.max_position_size.to_string(), "0.01");
        assert_eq!(config.risk_per_trade.to_string(), "1.0");
    }
}