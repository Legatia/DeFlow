#!/bin/bash

# DeFlow Yield Optimization Demo Runner
# Usage: ./run_demo.sh [csv_file] [output_dir]

echo "🎯 DeFlow Yield Optimization Demo Runner"
echo "========================================"

# Set default paths
CSV_FILE="${1:-./demo_yield_data_90_days.csv}"
OUTPUT_DIR="${2:-./demo_result}"

# Check if CSV file exists
if [ ! -f "$CSV_FILE" ]; then
    echo "❌ CSV file not found: $CSV_FILE"
    echo ""
    echo "Available CSV files:"
    ls -la *.csv 2>/dev/null || echo "   No CSV files found in current directory"
    echo ""
    echo "Usage: $0 [csv_file] [output_dir]"
    echo "Example: $0 demo_yield_data_90_days.csv ./demo_result"
    exit 1
fi

# Create output directory if it doesn't exist
mkdir -p "$OUTPUT_DIR"

echo "📊 Input CSV: $CSV_FILE"
echo "📁 Output Directory: $OUTPUT_DIR"
echo ""

# Run the calculation
echo "🚀 Running DeFlow yield optimization calculation..."
node run_demo_calculation.cjs "$CSV_FILE" "$OUTPUT_DIR"

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Demo calculation completed successfully!"
    echo ""
    echo "📁 Results available in: $OUTPUT_DIR"
    echo "   📊 optimization_results.csv - Data for analysis"
    echo "   📄 optimization_results.json - Detailed JSON"
    echo "   📋 summary_report.md - Human-readable summary"
    echo ""
    echo "💡 Quick view of results:"
    echo "   cat $OUTPUT_DIR/summary_report.md"
else
    echo ""
    echo "❌ Demo calculation failed!"
    exit 1
fi