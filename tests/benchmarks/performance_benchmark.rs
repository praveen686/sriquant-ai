//! Performance benchmarking suite for SriQuant.ai
//!
//! Measures and compares performance of key components:
//! - Fixed-point arithmetic vs floating-point
//! - ID generation throughput
//! - Timing precision and overhead
//! - Memory allocation patterns
//! - Network latency simulation

use sriquant_core::prelude::*;
use sriquant_exchanges::prelude::*;
use std::time::Instant;
use std::collections::HashMap;
use tracing::{info, warn};

/// Benchmark result statistics
#[derive(Debug, Clone)]
pub struct BenchmarkStats {
    pub name: String,
    pub iterations: u64,
    pub total_time_nanos: u64,
    pub avg_time_nanos: u64,
    pub min_time_nanos: u64,
    pub max_time_nanos: u64,
    pub p50_nanos: u64,
    pub p95_nanos: u64,
    pub p99_nanos: u64,
    pub throughput_ops_per_sec: f64,
}

impl BenchmarkStats {
    pub fn from_samples(name: String, samples: Vec<u64>) -> Self {
        if samples.is_empty() {
            return Self::empty(name);
        }
        
        let mut sorted_samples = samples.clone();
        sorted_samples.sort();
        
        let iterations = samples.len() as u64;
        let total_time_nanos: u64 = samples.iter().sum();
        let avg_time_nanos = total_time_nanos / iterations;
        let min_time_nanos = *sorted_samples.first().unwrap();
        let max_time_nanos = *sorted_samples.last().unwrap();
        
        let p50_nanos = sorted_samples[sorted_samples.len() / 2];
        let p95_nanos = sorted_samples[(sorted_samples.len() * 95) / 100];
        let p99_nanos = sorted_samples[(sorted_samples.len() * 99) / 100];
        
        let throughput_ops_per_sec = if avg_time_nanos > 0 {
            1_000_000_000.0 / avg_time_nanos as f64
        } else {
            0.0
        };
        
        Self {
            name,
            iterations,
            total_time_nanos,
            avg_time_nanos,
            min_time_nanos,
            max_time_nanos,
            p50_nanos,
            p95_nanos,
            p99_nanos,
            throughput_ops_per_sec,
        }
    }
    
    fn empty(name: String) -> Self {
        Self {
            name,
            iterations: 0,
            total_time_nanos: 0,
            avg_time_nanos: 0,
            min_time_nanos: 0,
            max_time_nanos: 0,
            p50_nanos: 0,
            p95_nanos: 0,
            p99_nanos: 0,
            throughput_ops_per_sec: 0.0,
        }
    }
    
    pub fn print_summary(&self) {
        info!("📊 Benchmark: {}", self.name);
        info!("   Iterations: {}", self.iterations);
        info!("   Avg Time: {:.2}ns", self.avg_time_nanos);
        info!("   Min Time: {}ns", self.min_time_nanos);
        info!("   Max Time: {}ns", self.max_time_nanos);
        info!("   P50: {}ns", self.p50_nanos);
        info!("   P95: {}ns", self.p95_nanos);
        info!("   P99: {}ns", self.p99_nanos);
        info!("   Throughput: {:.0} ops/sec", self.throughput_ops_per_sec);
        
        if self.avg_time_nanos < 1000 {
            info!("   ✅ Excellent performance (sub-microsecond)");
        } else if self.avg_time_nanos < 10000 {
            info!("   ✅ Good performance (< 10μs)");
        } else if self.avg_time_nanos < 100000 {
            info!("   ⚠️  Moderate performance (< 100μs)");
        } else {
            info!("   ❌ Poor performance (> 100μs)");
        }
    }
}

/// Comprehensive benchmark suite
pub struct PerformanceBenchmark {
    results: HashMap<String, BenchmarkStats>,
}

impl PerformanceBenchmark {
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
        }
    }
    
    /// Run all benchmarks
    pub async fn run_all(&mut self) {
        info!("🚀 Starting SriQuant.ai Performance Benchmark Suite");
        info!("   Following performance targets");
        
        // Bind to CPU core for consistent results
        if let Err(e) = bind_to_cpu_set(0) {
            warn!("Failed to bind to CPU core 0: {}", e);
        }
        
        self.benchmark_timing_precision().await;
        self.benchmark_fixed_point_arithmetic().await;
        self.benchmark_id_generation().await;
        self.benchmark_memory_allocation().await;
        self.benchmark_serialization().await;
        self.benchmark_hash_operations().await;
        
        self.print_summary();
    }
    
    /// Benchmark timing precision and overhead
    async fn benchmark_timing_precision(&mut self) {
        const ITERATIONS: usize = 100_000;
        let mut samples = Vec::with_capacity(ITERATIONS);
        
        info!("⏱️  Benchmarking timing precision...");
        
        for _ in 0..ITERATIONS {
            let start = nanos();
            let end = nanos();
            samples.push(end - start);
        }
        
        let stats = BenchmarkStats::from_samples("Timing Precision".to_string(), samples);
        stats.print_summary();
        self.results.insert("timing_precision".to_string(), stats);
        
        // Also benchmark PerfTimer overhead
        let mut timer_samples = Vec::with_capacity(ITERATIONS);
        for _ in 0..ITERATIONS {
            let start = nanos();
            let timer = PerfTimer::start("test".to_string());
            let _elapsed = timer.elapsed_nanos();
            let end = nanos();
            timer_samples.push(end - start);
        }
        
        let timer_stats = BenchmarkStats::from_samples("PerfTimer Overhead".to_string(), timer_samples);
        timer_stats.print_summary();
        self.results.insert("perf_timer_overhead".to_string(), timer_stats);
    }
    
    /// Benchmark fixed-point arithmetic performance
    async fn benchmark_fixed_point_arithmetic(&mut self) {
        const ITERATIONS: usize = 50_000;
        info!("🔢 Benchmarking fixed-point arithmetic...");
        
        let a = Fixed::from_str_exact("123.456789").unwrap();
        let b = Fixed::from_str_exact("987.654321").unwrap();
        
        // Addition benchmark
        let mut add_samples = Vec::with_capacity(ITERATIONS);
        for _ in 0..ITERATIONS {
            let start = nanos();
            let _result = a + b;
            let end = nanos();
            add_samples.push(end - start);
        }
        
        let add_stats = BenchmarkStats::from_samples("Fixed Addition".to_string(), add_samples);
        add_stats.print_summary();
        self.results.insert("fixed_addition".to_string(), add_stats);
        
        // Multiplication benchmark
        let mut mul_samples = Vec::with_capacity(ITERATIONS);
        for _ in 0..ITERATIONS {
            let start = nanos();
            let _result = a * b;
            let end = nanos();
            mul_samples.push(end - start);
        }
        
        let mul_stats = BenchmarkStats::from_samples("Fixed Multiplication".to_string(), mul_samples);
        mul_stats.print_summary();
        self.results.insert("fixed_multiplication".to_string(), mul_stats);
        
        // Division benchmark
        let mut div_samples = Vec::with_capacity(ITERATIONS);
        for _ in 0..ITERATIONS {
            let start = nanos();
            let _result = a / b;
            let end = nanos();
            div_samples.push(end - start);
        }
        
        let div_stats = BenchmarkStats::from_samples("Fixed Division".to_string(), div_samples);
        div_stats.print_summary();
        self.results.insert("fixed_division".to_string(), div_stats);
        
        // Compare with f64 arithmetic
        let a_f64 = 123.456789f64;
        let b_f64 = 987.654321f64;
        
        let mut f64_add_samples = Vec::with_capacity(ITERATIONS);
        for _ in 0..ITERATIONS {
            let start = nanos();
            let _result = a_f64 + b_f64;
            let end = nanos();
            f64_add_samples.push(end - start);
        }
        
        let f64_add_stats = BenchmarkStats::from_samples("F64 Addition".to_string(), f64_add_samples);
        f64_add_stats.print_summary();
        self.results.insert("f64_addition".to_string(), f64_add_stats);
    }
    
    /// Benchmark ID generation performance
    async fn benchmark_id_generation(&mut self) {
        const ITERATIONS: usize = 10_000;
        info!("🆔 Benchmarking ID generation...");
        
        // nanoid benchmark
        let mut nanoid_samples = Vec::with_capacity(ITERATIONS);
        for _ in 0..ITERATIONS {
            let start = nanos();
            let _id = generate_id();
            let end = nanos();
            nanoid_samples.push(end - start);
        }
        
        let nanoid_stats = BenchmarkStats::from_samples("Nanoid Generation".to_string(), nanoid_samples);
        nanoid_stats.print_summary();
        self.results.insert("nanoid_generation".to_string(), nanoid_stats);
        
        // Sequential ID benchmark
        let mut seq_samples = Vec::with_capacity(ITERATIONS);
        for _ in 0..ITERATIONS {
            let start = nanos();
            let _id = idgen_next_id();
            let end = nanos();
            seq_samples.push(end - start);
        }
        
        let seq_stats = BenchmarkStats::from_samples("Sequential ID".to_string(), seq_samples);
        seq_stats.print_summary();
        self.results.insert("sequential_id".to_string(), seq_stats);
        
        // OrderId creation benchmark
        let mut order_id_samples = Vec::with_capacity(ITERATIONS);
        for _ in 0..ITERATIONS {
            let start = nanos();
            let _id = OrderId::new();
            let end = nanos();
            order_id_samples.push(end - start);
        }
        
        let order_id_stats = BenchmarkStats::from_samples("OrderId Creation".to_string(), order_id_samples);
        order_id_stats.print_summary();
        self.results.insert("order_id_creation".to_string(), order_id_stats);
    }
    
    /// Benchmark memory allocation patterns
    async fn benchmark_memory_allocation(&mut self) {
        const ITERATIONS: usize = 10_000;
        info!("💾 Benchmarking memory allocation...");
        
        // Vec allocation benchmark
        let mut vec_samples = Vec::with_capacity(ITERATIONS);
        for _ in 0..ITERATIONS {
            let start = nanos();
            let _vec: Vec<u64> = Vec::with_capacity(100);
            let end = nanos();
            vec_samples.push(end - start);
        }
        
        let vec_stats = BenchmarkStats::from_samples("Vec Allocation".to_string(), vec_samples);
        vec_stats.print_summary();
        self.results.insert("vec_allocation".to_string(), vec_stats);
        
        // HashMap allocation benchmark
        let mut map_samples = Vec::with_capacity(ITERATIONS);
        for _ in 0..ITERATIONS {
            let start = nanos();
            let _map: HashMap<String, u64> = HashMap::with_capacity(50);
            let end = nanos();
            map_samples.push(end - start);
        }
        
        let map_stats = BenchmarkStats::from_samples("HashMap Allocation".to_string(), map_samples);
        map_stats.print_summary();
        self.results.insert("hashmap_allocation".to_string(), map_stats);
        
        // String allocation benchmark
        let mut string_samples = Vec::with_capacity(ITERATIONS);
        for _ in 0..ITERATIONS {
            let start = nanos();
            let _s = String::with_capacity(256);
            let end = nanos();
            string_samples.push(end - start);
        }
        
        let string_stats = BenchmarkStats::from_samples("String Allocation".to_string(), string_samples);
        string_stats.print_summary();
        self.results.insert("string_allocation".to_string(), string_stats);
    }
    
    /// Benchmark serialization performance
    async fn benchmark_serialization(&mut self) {
        const ITERATIONS: usize = 5_000;
        info!("📦 Benchmarking serialization...");
        
        // Create test data
        let order = OrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            quantity: Fixed::from_str_exact("0.001").unwrap(),
            price: Some(Fixed::from_str_exact("50000.0").unwrap()),
            stop_price: None,
            time_in_force: Some(TimeInForce::GoodTillCanceled),
            client_order_id: Some("test_order_123".to_string()),
        };
        
        // JSON serialization benchmark
        let mut serialize_samples = Vec::with_capacity(ITERATIONS);
        for _ in 0..ITERATIONS {
            let start = nanos();
            let _json = serde_json::to_string(&order).unwrap();
            let end = nanos();
            serialize_samples.push(end - start);
        }
        
        let serialize_stats = BenchmarkStats::from_samples("JSON Serialization".to_string(), serialize_samples);
        serialize_stats.print_summary();
        self.results.insert("json_serialization".to_string(), serialize_stats);
        
        // JSON deserialization benchmark
        let json_data = serde_json::to_string(&order).unwrap();
        let mut deserialize_samples = Vec::with_capacity(ITERATIONS);
        for _ in 0..ITERATIONS {
            let start = nanos();
            let _order: OrderRequest = serde_json::from_str(&json_data).unwrap();
            let end = nanos();
            deserialize_samples.push(end - start);
        }
        
        let deserialize_stats = BenchmarkStats::from_samples("JSON Deserialization".to_string(), deserialize_samples);
        deserialize_stats.print_summary();
        self.results.insert("json_deserialization".to_string(), deserialize_stats);
    }
    
    /// Benchmark hash operations
    async fn benchmark_hash_operations(&mut self) {
        const ITERATIONS: usize = 20_000;
        info!("🔐 Benchmarking hash operations...");
        
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let data = "BTCUSDT_12345_1234567890";
        
        // Hash calculation benchmark
        let mut hash_samples = Vec::with_capacity(ITERATIONS);
        for _ in 0..ITERATIONS {
            let start = nanos();
            let mut hasher = DefaultHasher::new();
            data.hash(&mut hasher);
            let _hash = hasher.finish();
            let end = nanos();
            hash_samples.push(end - start);
        }
        
        let hash_stats = BenchmarkStats::from_samples("Hash Calculation".to_string(), hash_samples);
        hash_stats.print_summary();
        self.results.insert("hash_calculation".to_string(), hash_stats);
        
        // HashMap lookup benchmark
        let mut map = HashMap::new();
        for i in 0..1000 {
            map.insert(format!("key_{}", i), i);
        }
        
        let mut lookup_samples = Vec::with_capacity(ITERATIONS);
        for i in 0..ITERATIONS {
            let key = format!("key_{}", i % 1000);
            let start = nanos();
            let _value = map.get(&key);
            let end = nanos();
            lookup_samples.push(end - start);
        }
        
        let lookup_stats = BenchmarkStats::from_samples("HashMap Lookup".to_string(), lookup_samples);
        lookup_stats.print_summary();
        self.results.insert("hashmap_lookup".to_string(), lookup_stats);
    }
    
    /// Print comprehensive benchmark summary
    pub fn print_summary(&self) {
        info!("🏁 Performance Benchmark Summary");
        info!("================================");
        
        // Performance targets based on high-performance architecture
        let targets = [
            ("timing_precision", 10), // 10ns max for timing calls
            ("fixed_addition", 100),   // 100ns max for fixed-point add
            ("nanoid_generation", 1000), // 1μs max for ID generation
        ];
        
        let mut passed = 0;
        let mut failed = 0;
        
        for (benchmark, target_ns) in &targets {
            if let Some(stats) = self.results.get(*benchmark) {
                if stats.avg_time_nanos <= *target_ns {
                    info!("✅ {}: {}ns (target: {}ns)", stats.name, stats.avg_time_nanos, target_ns);
                    passed += 1;
                } else {
                    info!("❌ {}: {}ns (target: {}ns)", stats.name, stats.avg_time_nanos, target_ns);
                    failed += 1;
                }
            }
        }
        
        info!("Summary: {}/{} targets met", passed, passed + failed);
        
        // Find fastest and slowest operations
        if let Some((_fastest_name, fastest_stats)) = self.results.iter()
            .min_by_key(|(_, stats)| stats.avg_time_nanos) {
            info!("🚀 Fastest: {} ({:.2}ns avg)", fastest_stats.name, fastest_stats.avg_time_nanos);
        }
        
        if let Some((_slowest_name, slowest_stats)) = self.results.iter()
            .max_by_key(|(_, stats)| stats.avg_time_nanos) {
            info!("🐌 Slowest: {} ({:.2}ns avg)", slowest_stats.name, slowest_stats.avg_time_nanos);
        }
        
        // Calculate overall throughput metrics
        let total_ops: f64 = self.results.values()
            .map(|s| s.throughput_ops_per_sec)
            .sum();
        
        info!("💪 Total Combined Throughput: {:.0} ops/sec", total_ops);
    }
}

#[monoio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logging();
    
    info!("🎯 SriQuant.ai Performance Benchmark Suite");
    info!("   Measuring performance against performance targets");
    
    let mut benchmark = PerformanceBenchmark::new();
    benchmark.run_all().await;
    
    info!("✅ Benchmark suite completed");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_benchmark_stats() {
        let samples = vec![100, 200, 150, 300, 250];
        let stats = BenchmarkStats::from_samples("Test".to_string(), samples);
        
        assert_eq!(stats.iterations, 5);
        assert_eq!(stats.min_time_nanos, 100);
        assert_eq!(stats.max_time_nanos, 300);
        assert_eq!(stats.avg_time_nanos, 200);
        assert_eq!(stats.p50_nanos, 200);
    }
    
    #[test]
    fn test_empty_benchmark_stats() {
        let stats = BenchmarkStats::from_samples("Empty".to_string(), vec![]);
        assert_eq!(stats.iterations, 0);
        assert_eq!(stats.throughput_ops_per_sec, 0.0);
    }
    
    #[monoio::test]
    async fn test_benchmark_creation() {
        let benchmark = PerformanceBenchmark::new();
        assert!(benchmark.results.is_empty());
    }
}