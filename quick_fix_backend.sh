#!/bin/bash

# Quick fix for backend compilation errors
# Replace problematic ic_cdk::println! statements with comments

cd /Users/zhang/Desktop/ICP/DeFlow/src/DeFlow_backend

# Fix common pattern where println was incorrectly placed in match arms
find . -name "*.rs" -exec sed -i '' 's/ic_cdk::println!("\([^"]*\)" =>/"\1" =>/g' {} \;

# Fix common pattern where println was incorrectly placed in vec! declarations  
find . -name "*.rs" -exec sed -i '' 's/ic_cdk::println!("\([^"]*\)\.to_string(),/"\1".to_string(),/g' {} \;

# Fix double println issues
find . -name "*.rs" -exec sed -i '' 's/ic_cdk::println!(\s*ic_cdk::println!(/ic_cdk::println!(/g' {} \;

# Remove trailing println issues  
find . -name "*.rs" -exec sed -i '' 's/ic_cdk::println!([^)]*);$/\/\/ Logging temporarily disabled/g' {} \;

echo "Quick fixes applied. Running cargo check..."
cargo check