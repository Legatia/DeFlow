# DeFlow Yield Optimization - Demo Package

## Quick Demo Scripts

### Instant Results (Pre-calculated)
```bash
# View 365-day realistic wave-riding results
cat demo_package/365_day_realistic/summary_report.md

# View conservative strategy results  
cat demo_package/90_day_conservative/summary_report.md

# View aggressive strategy results
cat demo_package/90_day_aggressive/summary_report.md
```

### Live Demo Commands
```bash
# Generate fresh 365-day data and run
python3 generate_365_day_demo_data.py && ./run_demo.sh demo_yield_data_365_days.csv ./live_demo

# Quick 90-day conservative run
./run_demo.sh demo_yield_data_conservative.csv ./live_conservative

# Show protocol switching over time
head -50 demo_package/365_day_realistic/optimization_results.csv
```

## Key Demo Talking Points

### Wave-Riding Strategy Benefits
- **Lazy Style**: Stays in high-yield positions until they fade
- **Smart Switching**: Only moves when significantly profitable
- **Multi-Protocol**: Diversifies across Pendle, Curve, Uniswap V3, Balancer
- **Real Yield Focus**: Captures actual token rewards, not just APY numbers

### Performance Highlights
- **12.70% Annual Returns** on realistic 365-day simulation
- **Outperforms static strategies** despite transaction costs
- **Captures APY spikes** up to 40% during market events
- **L2-focused approach** avoids expensive ETH gas fees

### Protocol Distribution Examples
- **Pendle**: 43.6% (during high real yield periods)
- **Curve**: 21.9% (volatility and arbitrage opportunities)
- **Balancer**: 20.0% (new feature launches)
- **Uniswap V3**: 12.6% (meme seasons and volume spikes)

## Demo Flow Suggestions

1. **Start with Problem**: Show static single-pool limitations
2. **Introduce Wave-Riding**: Explain sticky threshold concept
3. **Show Results**: Display 365-day performance metrics
4. **Highlight Switching**: Show protocol distribution over time
5. **Compare Strategies**: Conservative vs Aggressive scenarios
6. **Emphasize Real Yield**: Focus on Pendle mechanics and actual returns

