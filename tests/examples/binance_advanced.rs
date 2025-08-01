//! Advanced Binance trading example with live orders and WebSocket streaming
//!
//! Demonstrates:
//! - Live order placement and management
//! - Real-time market data streaming
//! - Portfolio tracking with fixed-point arithmetic
//! - Risk management and position sizing
//! - High-performance latency monitoring

use sriquant_core::prelude::*;
use sriquant_exchanges::binance::{BinanceConfig, BinanceExchange};
use sriquant_exchanges::prelude::*;
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
    config: TradingConfig,
    portfolio: Portfolio,
    active_orders: HashMap<String, OrderResponse>,
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
        
        let mut exchange = BinanceExchange::new(binance_config).await?;
        exchange.init_rest().await?;
        exchange.init_websocket().await?;
        
        // Test connectivity
        let latency = exchange.ping().await?;
        info!("✅ Connected to Binance (latency: {}μs)", latency);
        
        Ok(Self {
            exchange,
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
        
        // Initialize portfolio
        self.update_portfolio().await?;
        
        // Start market data streaming
        self.start_market_data_stream().await?;
        
        // Main trading loop
        let mut iteration = 0u64;
        loop {
            iteration += 1;
            
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
    }
    
    async fn update_portfolio(&mut self) -> Result<()> {
        debug!("💼 Updating portfolio...");
        
        let timer = PerfTimer::start("portfolio_update".to_string());
        
        // This would normally fetch account info from the exchange
        // For demonstration, we'll simulate some balances
        self.portfolio.update_balance("USDT", Fixed::from_str_exact("1000.0")?);
        self.portfolio.update_balance("BTC", Fixed::from_str_exact("0.02")?);
        
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
        // This is where trading logic would go
        // For demonstration, we'll show the structure
        
        let current_price = Fixed::from_str_exact("50000.0")?; // Simulated BTC price
        
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
    
    async fn place_limit_orders(&mut self, mid_price: Fixed) -> Result<()> {
        let spread = Fixed::from_str_exact("10.0")?; // $10 spread
        
        if spread < self.config.min_spread {
            debug!("Spread too small: ${}", spread);
            return Ok(());
        }
        
        let position_size = self.portfolio.calculate_position_size(
            mid_price, 
            self.config.risk_per_trade
        )?;
        
        // Limit position size
        let final_size = if position_size > self.config.max_position_size {
            self.config.max_position_size
        } else {
            position_size
        };
        
        // Place buy order below market
        let buy_price = mid_price - (spread / Fixed::from_i64(2)?);
        self.place_order(OrderSide::Buy, buy_price, final_size).await?;
        
        // Place sell order above market
        let sell_price = mid_price + (spread / Fixed::from_i64(2)?);
        self.place_order(OrderSide::Sell, sell_price, final_size).await?;
        
        Ok(())
    }
    
    async fn place_order(&mut self, side: OrderSide, price: Fixed, quantity: Fixed) -> Result<()> {
        let timer = PerfTimer::start("place_order".to_string());
        
        info!("📋 Placing {} order: {} {} @ ${}", 
            side, quantity, self.config.symbol, price);
        
        // In a real implementation, this would place the order via the exchange API
        // For demonstration, we'll simulate the order placement
        
        let order_id = generate_id_with_prefix("ORD");
        let order_response = OrderResponse {
            order_id: order_id.clone(),
            client_order_id: order_id.clone(),
            symbol: self.config.symbol.clone(),
            side,
            order_type: OrderType::Limit,
            quantity,
            price: Some(price),
            stop_price: None,
            status: OrderStatus::New,
            filled_quantity: Fixed::ZERO,
            average_price: None,
            time_in_force: Some(TimeInForce::GoodTillCanceled),
            timestamp: nanos() / 1_000_000,
            update_time: nanos() / 1_000_000,
        };
        
        self.active_orders.insert(order_id, order_response);
        
        let elapsed = timer.elapsed_micros();
        self.performance_metrics.record_latency(elapsed);
        
        info!("✅ Order placed successfully (latency: {}μs)", elapsed);
        
        Ok(())
    }
    
    async fn monitor_orders(&mut self) -> Result<()> {
        // Check status of active orders
        let mut completed_orders = Vec::new();
        
        for (order_id, order) in &self.active_orders {
            // In a real implementation, you would query the exchange for order status
            // For demonstration, we'll simulate some orders being filled
            
            if order.timestamp < (nanos() / 1_000_000) - 10000 { // 10 seconds old
                debug!("🎯 Simulating order fill for {}", order_id);
                completed_orders.push(order_id.clone());
                
                // Calculate profit/loss
                let profit = Fixed::from_str_exact("5.0")?; // Simulated $5 profit
                self.performance_metrics.record_trade(profit);
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

#[monoio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logging();
    
    info!("🚀 SriQuant.ai Advanced Binance Trading Bot");
    info!("   Following high-performance design");
    
    // Create trading configuration
    let config = TradingConfig {
        symbol: "BTCUSDT".to_string(),
        max_position_size: Fixed::from_str_exact("0.001")?, // 0.001 BTC for testing
        risk_per_trade: Fixed::from_str_exact("0.5")?, // 0.5% risk per trade
        stop_loss_pct: Fixed::from_str_exact("1.0")?,
        take_profit_pct: Fixed::from_str_exact("2.0")?,
        min_spread: Fixed::from_str_exact("5.0")?, // $5 minimum spread
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