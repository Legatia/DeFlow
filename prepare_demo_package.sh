#!/bin/bash

# DeFlow Demo Package Generator
# Pre-generates all demo scenarios for smooth presentations

echo "🎯 DeFlow Demo Package Generator"
echo "==============================="
echo "Pre-generating all demo scenarios for instant presentation..."
echo ""

# Create demo package directory
DEMO_PKG_DIR="./demo_package"
mkdir -p "$DEMO_PKG_DIR"

echo "📁 Creating demo package in: $DEMO_PKG_DIR"
echo ""

# Generate different dataset scenarios if they don't exist
echo "📊 Step 1: Generating dataset scenarios..."

if [ ! -f "demo_yield_data_365_days.csv" ]; then
    echo "   Generating 365-day realistic dataset..."
    python3 generate_365_day_demo_data.py
else
    echo "   ✅ 365-day dataset already exists"
fi

if [ ! -f "demo_yield_data_conservative.csv" ]; then
    echo "   Generating conservative scenario..."
    python3 generate_demo_data.py conservative
else
    echo "   ✅ Conservative dataset already exists"
fi

if [ ! -f "demo_yield_data_aggressive.csv" ]; then
    echo "   Generating aggressive scenario..."
    python3 generate_demo_data.py aggressive
else
    echo "   ✅ Aggressive dataset already exists"
fi

echo ""

# Run optimization scenarios
echo "🚀 Step 2: Running optimization scenarios..."

scenarios=(
    "demo_yield_data_365_days.csv:365_day_realistic:Full Year Realistic"
    "demo_yield_data_conservative.csv:90_day_conservative:Conservative 90-Day"
    "demo_yield_data_aggressive.csv:90_day_aggressive:Aggressive 90-Day"
)

for scenario in "${scenarios[@]}"; do
    IFS=':' read -r csv_file output_dir description <<< "$scenario"
    
    echo "   📈 Running: $description"
    
    if [ -f "$csv_file" ]; then
        ./run_demo.sh "$csv_file" "$DEMO_PKG_DIR/$output_dir" > /dev/null 2>&1
        
        if [ $? -eq 0 ]; then
            echo "      ✅ Completed: $description"
            
            # Extract key metrics for quick reference
            final_value=$(grep "Final Portfolio Value" "$DEMO_PKG_DIR/$output_dir/summary_report.md" | grep -o '\$[0-9,]*\.[0-9]*')
            total_return=$(grep "Total Return" "$DEMO_PKG_DIR/$output_dir/summary_report.md" | grep -o '[0-9]*\.[0-9]*%')
            vs_single_pool=$(grep "vs Best Single Pool" "$DEMO_PKG_DIR/$output_dir/summary_report.md" | grep -o '\+\$[0-9]*\.[0-9]*')
            
            echo "         💰 Final Value: $final_value ($total_return)"
            echo "         🏆 vs Single Pool: $vs_single_pool"
        else
            echo "      ❌ Failed: $description"
        fi
    else
        echo "      ⚠️  CSV not found: $csv_file"
    fi
    echo ""
done

# Create presentation summary
echo "📋 Step 3: Creating presentation summary..."

cat > "$DEMO_PKG_DIR/PRESENTATION_SUMMARY.md" << 'EOF'
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

EOF

# Create quick demo launcher
cat > "$DEMO_PKG_DIR/quick_demo.sh" << 'EOF'
#!/bin/bash

echo "🎯 DeFlow Quick Demo Launcher"
echo "============================"
echo ""
echo "Select demo scenario:"
echo "1) 365-Day Realistic Results (Pre-calculated)"
echo "2) Conservative Strategy Results" 
echo "3) Aggressive Strategy Results"
echo "4) Live Fresh Calculation"
echo "5) Protocol Switching Timeline"
echo ""
read -p "Choice (1-5): " choice

case $choice in
    1)
        echo "📊 365-Day Realistic Wave-Riding Results:"
        echo "========================================"
        cat 365_day_realistic/summary_report.md
        ;;
    2)
        echo "📊 Conservative Strategy Results:"
        echo "==============================="
        cat 90_day_conservative/summary_report.md
        ;;
    3)
        echo "📊 Aggressive Strategy Results:"
        echo "============================="
        cat 90_day_aggressive/summary_report.md
        ;;
    4)
        echo "🚀 Running Fresh Live Calculation..."
        cd .. && ./run_demo.sh demo_yield_data_365_days.csv ./demo_package/live_fresh
        ;;
    5)
        echo "📈 Protocol Switching Timeline (Last 30 days):"
        echo "=============================================="
        echo "Day,Protocol,Chain,APY,Portfolio Value"
        tail -30 365_day_realistic/optimization_results.csv | cut -d',' -f1,3,4,5,6
        ;;
    *)
        echo "Invalid choice"
        ;;
esac
EOF

chmod +x "$DEMO_PKG_DIR/quick_demo.sh"

# Create performance comparison table
echo "📊 Step 4: Creating performance comparison..."

cat > "$DEMO_PKG_DIR/PERFORMANCE_COMPARISON.md" << 'EOF'
# DeFlow Performance Comparison

| Scenario | Duration | Final Value | Return % | vs Single Pool | Protocols Used |
|----------|----------|-------------|----------|----------------|----------------|
EOF

# Add data to comparison table
for scenario_dir in "$DEMO_PKG_DIR"/*/; do
    if [ -f "$scenario_dir/summary_report.md" ]; then
        scenario_name=$(basename "$scenario_dir")
        final_value=$(grep "Final Portfolio Value" "$scenario_dir/summary_report.md" | grep -o '\$[0-9,]*\.[0-9]*' | head -1)
        return_pct=$(grep "Total Return" "$scenario_dir/summary_report.md" | grep -o '[0-9]*\.[0-9]*%' | head -1)
        vs_pool=$(grep "vs Best Single Pool" "$scenario_dir/summary_report.md" | grep -o '\+\$[0-9]*\.[0-9]*' | head -1)
        protocols=$(grep -A 10 "Protocol Distribution" "$scenario_dir/summary_report.md" | grep '\*\*' | wc -l)
        
        duration="365 days"
        if [[ $scenario_name == *"90_day"* ]]; then
            duration="90 days"
        fi
        
        echo "| $scenario_name | $duration | $final_value | $return_pct | $vs_pool | $protocols |" >> "$DEMO_PKG_DIR/PERFORMANCE_COMPARISON.md"
    fi
done

echo ""
echo "✅ Demo package generation complete!"
echo ""
echo "📁 Demo Package Contents:"
echo "   📊 365_day_realistic/ - Full year wave-riding results"
echo "   📊 90_day_conservative/ - Conservative strategy results"  
echo "   📊 90_day_aggressive/ - Aggressive strategy results"
echo "   📋 PRESENTATION_SUMMARY.md - Key talking points and commands"
echo "   📊 PERFORMANCE_COMPARISON.md - Side-by-side comparison table"
echo "   🚀 quick_demo.sh - Interactive demo launcher"
echo ""
echo "🎯 Ready for presentation! Use:"
echo "   cd $DEMO_PKG_DIR && ./quick_demo.sh"
echo ""
echo "💡 Or view any results instantly:"
echo "   cat $DEMO_PKG_DIR/365_day_realistic/summary_report.md"