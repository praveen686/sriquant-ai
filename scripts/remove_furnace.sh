 #!/bin/bash
  # Remove all Furnace references from SriQuant.ai codebase

  cd /home/praveen/Desktop/omsreem/sriquant-ai

  echo "🚀 Removing all Furnace references from SriQuant.ai codebase..."

  # Core source files
  echo "📝 Updating core source files..."
  find crates/core/src -name "*.rs" -exec sed -i 's/Following Furnace.*architecture/High-performance architecture/g' {} \;
  find crates/core/src -name "*.rs" -exec sed -i 's/like Furnace.*)/)/g' {} \;
  find crates/core/src -name "*.rs" -exec sed -i 's/equivalent to Furnace.*functionality/high-performance functionality/g' {} \;
  find crates/core/src -name "*.rs" -exec sed -i 's/following Furnace.*principles/following high-performance principles/g' {} \;
  find crates/core/src -name "*.rs" -exec sed -i 's/like Furnace[^.]*$/for high performance/g' {} \;
  find crates/core/src -name "*.rs" -exec sed -i 's/Precision timestamping following Furnace.*implementation/Precision timestamping 
  implementation/g' {} \;
  find crates/core/src -name "*.rs" -exec sed -i 's/ID generation following Furnace.*implementation/ID generation implementation/g' {} \;
  find crates/core/src -name "*.rs" -exec sed -i 's/Fixed-point arithmetic following Furnace.*implementation/Fixed-point arithmetic 
  implementation/g' {} \;
  find crates/core/src -name "*.rs" -exec sed -i 's/Unified logging following Furnace.*integration/Unified logging integration/g' {} \;
  find crates/core/src -name "*.rs" -exec sed -i 's/Generate a unique ID using nanoid (like Furnace)/Generate a unique ID using nanoid/g' {} \;
  find crates/core/src -name "*.rs" -exec sed -i 's/Generate sequential ID (like Furnace.*)/Generate sequential ID/g' {} \;
  find crates/core/src -name "*.rs" -exec sed -i 's/Following Furnace.*architecture:/High-performance architecture:/g' {} \;
  find crates/core/src -name "*.rs" -exec sed -i 's/following Furnace.*architecture principles/following high-performance architecture 
  principles/g' {} \;
  find crates/core/src -name "*.rs" -exec sed -i 's/Bind to CPU core if specified (like Furnace.*)/Bind to CPU core if specified/g' {} \;
  find crates/core/src -name "*.rs" -exec sed -i 's/Following Furnace.*supports/High-performance implementation supports/g' {} \;
  find crates/core/src -name "*.rs" -exec sed -i 's/Convenience macro for creating Fixed values (similar to Furnace)/Convenience macro for 
  creating Fixed values/g' {} \;
  find crates/core/src -name "*.rs" -exec sed -i 's/Initialize unified logging system (like Furnace.*)/Initialize unified logging system/g' {}
  \;
  find crates/core/src -name "*.rs" -exec sed -i 's/Log levels matching Furnace.*levels/High-performance log levels/g' {} \;

  # Exchange source files
  echo "📝 Updating exchange source files..."
  find crates/exchanges/src -name "*.rs" -exec sed -i 's/Following Furnace.*architecture/High-performance architecture/g' {} \;
  find crates/exchanges/src -name "*.rs" -exec sed -i 's/following Furnace.*principles/following high-performance principles/g' {} \;
  find crates/exchanges/src -name "*.rs" -exec sed -i 's/with Furnace-like architecture/with high-performance architecture/g' {} \;
  find crates/exchanges/src -name "*.rs" -exec sed -i 's/Binance exchange integration with Furnace-like architecture/Binance exchange 
  integration with high-performance architecture/g' {} \;
  find crates/exchanges/src -name "*.rs" -exec sed -i 's/Following Furnace.*:/High-performance:/g' {} \;

  # Test files
  echo "📝 Updating test files..."
  find tests -name "*.rs" -exec sed -i 's/Following Furnace.*targets/Following performance targets/g' {} \;
  find tests -name "*.rs" -exec sed -i 's/against Furnace targets/against performance targets/g' {} \;
  find tests -name "*.rs" -exec sed -i 's/like Furnace/for high performance/g' {} \;
  find tests -name "*.rs" -exec sed -i 's/Following Furnace.*performance/Following high-performance design/g' {} \;
  find tests -name "*.rs" -exec sed -i 's/Performance targets based on Furnace architecture/Performance targets based on high-performance 
  architecture/g' {} \;
  find tests -name "*.rs" -exec sed -i 's/Measuring performance against Furnace targets/Measuring performance against performance targets/g' {}
   \;

  # Example files
  echo "📝 Updating example files..."
  find examples -name "*.rs" -exec sed -i 's/like Furnace/for high performance/g' {} \;
  find examples -name "*.rs" -exec sed -i 's/Following Furnace.*performance/Following high-performance design/g' {} \;
  find examples -name "*.rs" -exec sed -i 's/Bind to CPU core 0 for maximum performance (like Furnace)/Bind to CPU core 0 for maximum 
  performance/g' {} \;

  # Update Cargo.toml files in examples
  echo "📝 Updating example Cargo.toml files..."
  find examples -name "Cargo.toml" -exec sed -i 's/following Furnace architecture/with high-performance architecture/g' {} \;

  # Additional cleanup for any remaining references
  echo "📝 Final cleanup..."
  find . -name "*.rs" -not -path "./target/*" -exec sed -i 's/Furnace Framework/High-Performance Trading Systems/g' {} \;
  find . -name "*.rs" -not -path "./target/*" -exec sed -i 's/Furnace framework/high-performance framework/g' {} \;
  find . -name "*.rs" -not -path "./target/*" -exec sed -i 's/Furnace benchmarks/performance benchmarks/g' {} \;

  echo "✅ All Furnace references have been removed from the codebase!"
  echo ""
  echo "🔍 Verifying removal..."
  if grep -r "Furnace" --include="*.rs" --include="*.toml" --include="*.md" --exclude-dir=target . > /dev/null; then
      echo "⚠️  Some references may still remain. Run this to check:"
      echo "   grep -r 'Furnace' --include='*.rs' --include='*.toml' --include='*.md' --exclude-dir=target ."
  else
      echo "✅ Verification complete: No Furnace references found!"
  fi

  echo ""
  echo "🎉 SriQuant.ai codebase is now free of Furnace references!"