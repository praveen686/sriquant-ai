# 🔐 SriQuant.ai Credentials Setup Guide

This guide helps you securely configure your Binance API credentials for SriQuant.ai.

## 🚀 Quick Start

### Option 1: Automated Setup (Recommended)
```bash
./setup_env.sh
```

### Option 2: Manual Setup
```bash
cp .env.example .env
# Edit .env with your credentials
nano .env
```

## 📋 Prerequisites

1. **Rust Installation**: [Install Rust](https://rustup.rs/) if not already installed
2. **Binance Account**: Create accounts on both:
   - [Binance Testnet](https://testnet.binance.vision/) (for development)
   - [Binance Production](https://www.binance.com/) (for live trading)

## 🔑 Getting API Keys

### For Development (Testnet) - START HERE!

1. **Go to Binance Testnet**: https://testnet.binance.vision/
2. **Login with GitHub**: Click "Login with GitHub"
3. **Generate API Key**:
   - Click "Generate HMAC_SHA256 Key"
   - Copy both API Key and Secret Key
   - Enable "Spot & Margin Trading" if you want to test orders

### For Production (Live Trading) - USE WITH CAUTION!

1. **Go to Binance**: https://www.binance.com/en/my/settings/api-management
2. **Create API Key**:
   - Enter a label (e.g., "SriQuant.ai")
   - Complete security verification
3. **Configure Permissions** (Be Restrictive!):
   - ✅ **Read Info**: Always required
   - ✅ **Spot & Margin Trading**: Only if you need trading
   - ❌ **Futures**: Only if specifically needed
   - ❌ **Enable Withdrawals**: **NEVER ENABLE**
4. **IP Restriction** (Highly Recommended):
   - Add your server/development machine IP
   - Enable "Restrict access to trusted IPs only"

## ⚙️ Configuration

### Required Environment Variables

```bash
# Basic Configuration
BINANCE_API_KEY=your_actual_api_key_here
BINANCE_SECRET_KEY=your_actual_secret_key_here
BINANCE_TESTNET=true  # false for production

# Logging
RUST_LOG=info

# Basic Risk Management
MAX_POSITION_SIZE=0.01
RISK_PER_TRADE=1.0
```

### Advanced Configuration

See `.env.example` for all available options including:
- Rate limiting settings
- WebSocket configuration
- Risk management parameters
- Feature flags
- Emergency controls

## 🧪 Testing Your Setup

### 1. Test Credentials
```bash
cargo run --example test_credentials
```

This will verify:
- ✅ Environment variables loaded
- ✅ REST API connectivity
- ✅ API authentication
- ✅ Market data access
- ✅ Trading permissions
- ✅ WebSocket connectivity

### 2. Test Market Data
```bash
cargo run --example market_data
```

### 3. Paper Trading Test
```bash
cargo run --example paper_trading
```

## 🔒 Security Best Practices

### Critical Security Rules:

1. **🚫 NEVER commit .env files to git**
   - `.env` is already in `.gitignore`
   - Double-check before any git commits

2. **🔐 Use minimum required permissions**
   - Start with "Read Info" only
   - Add "Spot Trading" only if needed
   - Never enable "Enable Withdrawals"

3. **🌐 Restrict API access by IP**
   - Add your specific IP addresses
   - Use VPN or fixed IP for consistency

4. **🔄 Rotate keys regularly**
   - Change API keys monthly
   - Delete old/unused keys

5. **📱 Enable 2FA on your Binance account**
   - Use authenticator app (not SMS)
   - Keep backup codes secure

### File Security:
```bash
# Set proper permissions on .env file
chmod 600 .env

# Verify it's not tracked by git
git status --ignored
```

## 🚨 Emergency Procedures

### If API Keys Are Compromised:

1. **Immediately disable the API key** in Binance dashboard
2. **Check recent API activity** for unauthorized access
3. **Generate new API keys** with different restrictions
4. **Review account for any unauthorized trades**
5. **Update your .env file** with new credentials

### If Trading Bot Malfunctions:

1. **Emergency stop**: Set `EMERGENCY_STOP_ON_RISK_BREACH=true`
2. **Cancel all orders** via Binance dashboard
3. **Disable API key** temporarily
4. **Review logs** to understand the issue

## 📊 Monitoring & Alerts

### Built-in Safeguards:

- **Daily loss limits**: `MAX_DAILY_LOSS_LIMIT=1000.0`
- **Position size limits**: `MAX_POSITION_SIZE=0.01`
- **Rate limiting**: Automatic API rate limiting
- **Connection monitoring**: Auto-reconnect on disconnection

### Recommended Monitoring:

1. **Account balance alerts**
2. **Unusual trading activity**
3. **API rate limit warnings**
4. **Connection status monitoring**

## 🔧 Troubleshooting

### Common Issues:

| Error | Cause | Solution |
|-------|-------|----------|
| "Invalid API Key" | Wrong API key | Double-check key in .env |
| "Signature Invalid" | Wrong secret or time sync | Check secret & sync system time |
| "IP Not Allowed" | IP restriction | Add your IP to whitelist |
| "Insufficient Balance" | Not enough funds | Check account balance |
| "Rate Limit Exceeded" | Too many requests | Reduce trading frequency |
| "Permission Denied" | Missing permissions | Enable required permissions |

### Debug Mode:
```bash
RUST_LOG=debug cargo run --example test_credentials
```

### Check System Time:
```bash
# Linux/Mac
ntpdate -s time.nist.gov

# Or check manually
curl -s https://api.binance.com/api/v3/time
date +%s000
```

## 📈 Production Checklist

Before going live:

- [ ] ✅ Tested thoroughly on testnet
- [ ] ✅ API keys have minimum required permissions
- [ ] ✅ IP restrictions configured
- [ ] ✅ Risk limits configured conservatively
- [ ] ✅ Emergency procedures documented
- [ ] ✅ Monitoring alerts set up
- [ ] ✅ Started with small position sizes
- [ ] ✅ Reviewed all configuration parameters
- [ ] ✅ Backup API keys generated and stored securely

## 📞 Support

### Getting Help:

1. **Check logs**: `RUST_LOG=debug cargo run`
2. **Test connectivity**: `ping api.binance.com`
3. **Verify credentials**: Use test_credentials example
4. **Review documentation**: This file and `setup_credentials.md`

### Resources:

- [Binance API Documentation](https://binance-docs.github.io/apidocs/)
- [Binance API Status](https://binance.statuspage.io/)
- [SriQuant.ai Examples](./examples/)

---

## ⚖️ Legal Disclaimer

**Important**: This software is for educational and research purposes. Trading involves risk of financial loss. The authors are not responsible for any trading losses. Always test thoroughly before using real money. Comply with local financial regulations.

**Start small, test thoroughly, trade responsibly!** 📊🔒