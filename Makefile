# SriQuant.ai Development Makefile
# High-performance quantitative trading library

.PHONY: help build test bench clean doc install run-basic run-advanced run-benchmark format lint audit deps

# Default target
help:
	@echo "SriQuant.ai - High-Performance Quantitative Trading Library"
	@echo "==========================================================="
	@echo ""
	@echo "Available targets:"
	@echo "  build          - Build all crates in release mode"
	@echo "  test           - Run all tests"
	@echo "  bench          - Run performance benchmarks"  
	@echo "  clean          - Clean build artifacts"
	@echo "  doc            - Generate and open documentation"
	@echo "  install        - Install development dependencies"
	@echo ""
	@echo "Examples:"
	@echo "  run-basic      - Run basic Binance connectivity example"
	@echo "  run-advanced   - Run advanced trading bot example"
	@echo "  run-benchmark  - Run performance benchmark suite"
	@echo ""
	@echo "Development:"
	@echo "  format         - Format code with rustfmt"
	@echo "  lint           - Run clippy lints"
	@echo "  audit          - Check for security vulnerabilities"
	@echo "  deps           - Check dependency status"
	@echo ""
	@echo "Performance Tips:"
	@echo "  - Use 'sudo' for CPU core binding"
	@echo "  - Set CPU governor to 'performance'"
	@echo "  - Disable CPU frequency scaling"

# Build targets
build:
	@echo "🔨 Building SriQuant.ai (Release Mode)..."
	@echo "   Using Rust Edition 2024 with high-performance optimizations"
	cargo build --release --workspace
	@echo "✅ Build completed"

build-dev:
	@echo "🔨 Building SriQuant.ai (Development Mode)..."
	cargo build --workspace
	@echo "✅ Development build completed"

# Test targets  
test:
	@echo "🧪 Running test suite..."
	cargo test --workspace
	@echo "✅ All tests passed"

test-core:
	@echo "🧪 Testing core runtime..."
	cargo test --package sriquant-core

test-exchanges:
	@echo "🧪 Testing exchange integrations..."
	cargo test --package sriquant-exchanges

# Benchmark targets
bench: build
	@echo "📊 Running performance benchmarks..."
	@echo "   Measuring against high-performance targets"
	cargo run --release --bin performance_benchmark
	@echo "✅ Benchmarks completed"

bench-timing:
	@echo "⏱️ Running timing precision benchmarks..."
	cargo run --release --bin performance_benchmark -- --timing-only

# Clean targets
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	@echo "✅ Clean completed"

clean-all: clean
	@echo "🧹 Deep cleaning (including deps)..."
	rm -rf target/
	rm -f Cargo.lock
	@echo "✅ Deep clean completed"

# Documentation
doc:
	@echo "📚 Generating documentation..."
	cargo doc --workspace --no-deps --open
	@echo "✅ Documentation opened in browser"

doc-private:
	@echo "📚 Generating documentation (including private items)..."
	cargo doc --workspace --no-deps --document-private-items --open

# Installation and setup
install:
	@echo "⚙️ Installing development dependencies..."
	rustup update
	rustup component add rustfmt clippy
	cargo install cargo-audit cargo-outdated cargo-tree
	@echo "✅ Development environment ready"

setup-env:
	@echo "⚙️ Setting up environment with interactive script..."
	@chmod +x scripts/setup_env.sh
	@./scripts/setup_env.sh

# Example runners
run-basic: build
	@echo "🚀 Running basic Binance example..."
	@echo "   Testing connectivity and basic operations"
	cargo run --release --bin binance_basic

run-advanced: build setup-env
	@echo "🚀 Running advanced trading bot..."
	@echo "   ⚠️  Make sure BINANCE_TESTNET=true in .env"
	@echo "   ⚠️  This will simulate trading operations"
	cargo run --release --bin binance_advanced_trading

run-benchmark: build
	@echo "📊 Running comprehensive performance benchmark..."
	@echo "   Measuring all core components against performance targets"
	sudo cargo run --release --bin performance_benchmark

# Development tools
format:
	@echo "🎨 Formatting code..."
	cargo fmt --all
	@echo "✅ Code formatted"

format-check:
	@echo "🎨 Checking code formatting..."
	cargo fmt --all -- --check

lint:
	@echo "📝 Running clippy lints..."
	cargo clippy --workspace --all-targets -- -D warnings
	@echo "✅ No lint issues found"

lint-fix:
	@echo "📝 Applying automatic lint fixes..."
	cargo clippy --workspace --all-targets --fix --allow-dirty -- -D warnings

audit:
	@echo "🔒 Auditing dependencies for security vulnerabilities..."
	cargo audit
	@echo "✅ Security audit completed"

deps:
	@echo "📦 Checking dependency status..."
	cargo outdated
	cargo tree --duplicates

# Performance optimization helpers
set-performance-governor:
	@echo "⚡ Setting CPU governor to performance mode..."
	@echo "   This requires sudo privileges"
	sudo cpupower frequency-set -g performance
	@cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor
	@echo "✅ CPU governor set to performance mode"

check-cpu-info:
	@echo "💻 CPU Information:"
	@echo "   Model: $$(cat /proc/cpuinfo | grep 'model name' | head -1 | cut -d: -f2 | xargs)"
	@echo "   Cores: $$(nproc)"
	@echo "   Governor: $$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo 'N/A')"
	@echo "   Max Freq: $$(cat /sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq 2>/dev/null || echo 'N/A') kHz"

# Development workflow
dev-setup: install setup-env
	@echo "🔧 Development environment setup complete!"
	@echo ""
	@echo "Next steps:"
	@echo "1. Edit .env with your Binance API credentials"
	@echo "2. Run 'make run-basic' to test connectivity"
	@echo "3. Run 'make run-benchmark' to measure performance"
	@echo "4. Run 'make test' to ensure everything works"

dev-check: format-check lint test
	@echo "✅ All development checks passed!"

# Release preparation
pre-release: clean build test bench audit
	@echo "🚀 Pre-release checks completed successfully!"
	@echo "   Ready for release"

# Continuous Integration targets
ci-test: format-check lint test
	@echo "✅ CI test pipeline completed"

ci-bench: build bench
	@echo "✅ CI benchmark pipeline completed"

# Help for performance optimization
perf-help:
	@echo "🚀 Performance Optimization Guide"
	@echo "================================="
	@echo ""
	@echo "For maximum performance:"
	@echo ""
	@echo "1. CPU Configuration:"
	@echo "   make set-performance-governor  # Set CPU to performance mode"
	@echo "   echo 0 | sudo tee /sys/devices/system/cpu/cpufreq/boost  # Disable turbo boost for consistency"
	@echo ""
	@echo "2. System Configuration:"
	@echo "   echo 1 | sudo tee /proc/sys/kernel/sched_rt_runtime_us  # Allow RT scheduling"
	@echo "   ulimit -r unlimited  # Allow real-time priority"
	@echo ""  
	@echo "3. Running with optimal settings:"
	@echo "   sudo nice -n -20 make run-benchmark  # Run with highest priority"
	@echo ""
	@echo "4. Monitoring:"
	@echo "   htop -C  # Monitor CPU usage by core"
	@echo "   perf stat make run-benchmark  # Detailed performance counters"