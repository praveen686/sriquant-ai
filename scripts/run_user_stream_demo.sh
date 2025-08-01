#!/bin/bash

# Script to demonstrate Binance user stream with live order updates
# This runs the user stream monitor and places test orders

echo "🚀 Starting Binance User Stream Demo..."
echo "This will:"
echo "  1. Start the user stream monitor"
echo "  2. Wait for it to connect"
echo "  3. Place test orders to trigger events"
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Start user stream in background
echo -e "${BLUE}Starting user stream monitor...${NC}"
cargo run --example binance_user_stream &
USER_STREAM_PID=$!

# Wait for user stream to connect (usually takes 2-3 seconds)
echo -e "${YELLOW}Waiting for user stream to connect...${NC}"
sleep 4

# Check if user stream is still running
if ! ps -p $USER_STREAM_PID > /dev/null; then
    echo -e "${RED}User stream failed to start!${NC}"
    exit 1
fi

echo -e "${GREEN}✅ User stream connected!${NC}"
echo ""

# Place orders to trigger events
echo -e "${BLUE}Placing test orders to trigger user stream events...${NC}"
echo "Watch the user stream output above for real-time updates!"
echo ""

# Run the order placement
cargo run --example place_simple_order

echo ""
echo -e "${YELLOW}Demo complete! The user stream will continue running.${NC}"
echo "Press Ctrl+C to stop the user stream monitor."
echo ""

# Wait for user stream (it will run until Ctrl+C)
wait $USER_STREAM_PID