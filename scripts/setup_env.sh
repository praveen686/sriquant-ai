#!/bin/bash
# SriQuant.ai Environment Setup Script

set -e

echo "🚀 SriQuant.ai Environment Setup"
echo "=================================="
echo

# Check if .env already exists
if [ -f ".env" ]; then
    echo "⚠️  .env file already exists!"
    read -p "Do you want to overwrite it? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Setup cancelled."
        exit 0
    fi
fi

# Copy template
echo "📋 Creating .env from template..."
cp .env.example .env
echo "✅ .env file created"
echo

# Interactive setup
echo "🔧 Interactive Configuration Setup"
echo "==================================="
echo

# Binance API Key
echo "Enter your Binance API credentials:"
echo "(Get them from: https://testnet.binance.vision/ for testnet)"
echo
read -p "Binance API Key: " api_key
read -s -p "Binance Secret Key: " secret_key
echo
echo

# Environment
echo "Select environment:"
echo "1) Testnet (recommended for development)"
echo "2) Production (live trading - use with caution!)"
read -p "Choose (1/2): " env_choice

if [ "$env_choice" = "2" ]; then
    use_testnet="false"
    echo "⚠️  WARNING: You selected PRODUCTION environment!"
    echo "   Make sure you understand the risks and start with small amounts."
else
    use_testnet="true"
    echo "✅ Using testnet (safe for development)"
fi
echo

# Update .env file
echo "💾 Updating .env file..."

# Use sed to replace the placeholder values
sed -i "s/your_binance_api_key_here/$api_key/" .env
sed -i "s/your_binance_secret_key_here/$secret_key/" .env
sed -i "s/BINANCE_TESTNET=true/BINANCE_TESTNET=$use_testnet/" .env

echo "✅ .env file configured"
echo

# Security reminder
echo "🔒 SECURITY REMINDERS:"
echo "======================"
echo "✅ Your .env file is excluded from git (see .gitignore)"
echo "✅ Never share your API keys with anyone"
echo "✅ Never commit .env files to version control"
echo "⚠️  Use IP restrictions on your Binance API keys"
echo "⚠️  Enable only necessary permissions"
echo


# Test credentials
echo "🧪 Testing your credentials..."
echo "=============================="
echo

if command -v cargo &> /dev/null; then
    echo "Running credential test..."
    cargo run --example test_credentials
else
    echo "⚠️  Cargo not found. Please install Rust to test credentials."
    echo "   Visit: https://rustup.rs/"
fi

echo
echo "🎉 Setup complete!"
echo
echo "Next steps:"
echo "1. Test your setup: cargo run --example test_credentials"
echo "2. Read the documentation: setup_credentials.md"
echo "3. Start with paper trading mode first"
echo
echo "Happy trading! 📈"