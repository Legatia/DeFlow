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
