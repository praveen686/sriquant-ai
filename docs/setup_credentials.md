# Setting Up Binance Credentials for SriQuant.ai

## Security First! 🔒

**CRITICAL**: Never commit actual API keys to version control. The `.env` file is already in `.gitignore` to prevent accidental commits.

## Step 1: Get Your Binance API Keys

### For Testnet (Recommended for Development):
1. Go to [Binance Testnet](https://testnet.binance.vision/)
2. Log in with your GitHub account
3. Generate API Key and Secret
4. **Enable Spot & Margin Trading** permissions

### For Production (Live Trading):
1. Go to [Binance API Management](https://www.binance.com/en/my/settings/api-management)
2. Create a new API key
3. **Enable only necessary permissions**:
   - ✅ Read Info
   - ✅ Spot & Margin Trading (if needed)
   - ❌ Futures Trading (unless required)
   - ❌ Enable Withdrawals (NOT RECOMMENDED)
4. **Restrict API Access**:
   - Add your server IP to IP whitelist
   - Enable "Restrict access to trusted IPs only"

## Step 2: Create Your .env File

Copy the example and fill in your credentials:

```bash
# In the sriquant-ai directory
cp .env.example .env
```

Then edit `.env` with your actual credentials:

```bash
# Example .env file (use your real values)
BINANCE_API_KEY=your_actual_api_key_here
BINANCE_SECRET_KEY=your_actual_secret_key_here
BINANCE_TESTNET=true  # Set to false for production

# ... other settings
```

## Step 3: Verify Your Setup

Test your credentials with:

```bash
cargo run --example basic_trading
```

## Security Best Practices ⚠️

### DO:
- ✅ Use testnet for development
- ✅ Restrict API keys to specific IPs
- ✅ Enable only required permissions
- ✅ Rotate API keys regularly
- ✅ Monitor API usage
- ✅ Use separate keys for different environments

### DON'T:
- ❌ Never commit .env files
- ❌ Never share API keys in chat/email
- ❌ Never enable withdrawal permissions
- ❌ Never use production keys in development
- ❌ Never hardcode keys in source code

## Environment Variables Reference

| Variable | Description | Example |
|----------|-------------|---------|
| `BINANCE_API_KEY` | Your Binance API key | `abc123...` |
| `BINANCE_SECRET_KEY` | Your Binance secret key | `def456...` |
| `BINANCE_TESTNET` | Use testnet (true/false) | `true` |
| `RUST_LOG` | Logging level | `info` |
| `MAX_POSITION_SIZE` | Max position size | `0.01` |
| `RISK_PER_TRADE` | Risk per trade (%) | `1.0` |

## Troubleshooting

### Common Issues:

1. **"Invalid API Key"**
   - Check if API key is correct
   - Verify IP restrictions
   - Ensure API key has required permissions

2. **"Signature Invalid"**
   - Check if secret key is correct
   - Verify system time is synchronized
   - Check if using correct testnet/production URLs

3. **"Rate Limit Exceeded"**
   - Reduce `ORDERS_PER_SECOND` in .env
   - Check `API_CALLS_PER_MINUTE` setting

### Getting Help:

1. Check the logs: `RUST_LOG=debug cargo run`
2. Verify connectivity: `ping api.binance.com`
3. Test with minimal example first

## Next Steps

Once credentials are set up:

1. Run basic connectivity test
2. Try paper trading mode
3. Test with small amounts
4. Enable advanced features gradually

Remember: **Start with testnet, test thoroughly, then move to production with small amounts!**