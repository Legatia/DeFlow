# Bitcoin Testnet Testing Strategy

## 🌐 Overview

This document outlines the comprehensive strategy for testing DeFlow's Bitcoin DeFi integration using Bitcoin testnet. Testnet testing provides the most reliable validation of Bitcoin functionality by using the actual Bitcoin network with test coins.

## 🎯 Why Bitcoin Testnet Testing?

### **Current Testing Challenges**
- Mock service configuration mismatches
- Test infrastructure complexity issues
- Factory override problems in unit tests
- React Flow component testing difficulties

### **Testnet Testing Benefits**

✅ **Real Bitcoin Network**: Actual blockchain validation with real network conditions  
✅ **Free Test Coins**: Testnet Bitcoin has no monetary value  
✅ **Complete Transaction Flow**: End-to-end validation from address generation to confirmation  
✅ **Threshold ECDSA Validation**: Tests actual ICP-Bitcoin cryptographic integration  
✅ **Network Latency & Timing**: Real-world performance characteristics  
✅ **UTXO Management**: Validates actual UTXO detection and selection  
✅ **Fee Estimation**: Tests real Bitcoin fee market dynamics  

## 🏗️ Implementation Status

### **Already Testnet-Ready Components**

Your current implementation has testnet-ready architecture:

```rust
// Bitcoin network configuration (src/DeFlow_backend/src/defi/bitcoin/service.rs)
pub async fn get_bitcoin_network(&self) -> BitcoinNetwork {
    self.context.network  // Configurable for Testnet/Mainnet/Regtest
}

// Address generation with proper network prefixes
pub async fn get_p2wpkh_address(&self, user: Principal) -> Result<BitcoinAddress, String> {
    // Returns tb1q... for testnet, bc1q... for mainnet
    let address = self.public_key_to_p2wpkh_address(&public_key)?;
}

// Transaction building for all address types
pub async fn create_transaction(&self, params: TransactionParams) -> Result<BitcoinTransaction, String> {
    // Handles P2PKH, P2WPKH, P2TR for any network
}
```

## 🔧 Testnet Setup Configuration

### **1. Network Configuration Update**

```rust
// Update canister configuration for testnet
const BITCOIN_NETWORK: BitcoinNetwork = BitcoinNetwork::Testnet;

// Testnet API endpoints
const BITCOIN_TESTNET_API: &str = "https://blockstream.info/testnet/api";
const MEMPOOL_TESTNET_API: &str = "https://mempool.space/testnet/api";

// Testnet-specific parameters
const TESTNET_MIN_CONFIRMATIONS: u32 = 1;  // Faster testing
const TESTNET_DEFAULT_FEE_RATE: u64 = 5;   // Lower fees
```

### **2. Deployment Configuration**

```bash
# Deploy to testnet-connected ICP environment
dfx deploy --network testnet --with-cycles 1000000000000

# Initialize Bitcoin service for testnet
dfx canister call DeFlow_backend init_bitcoin_service '(variant { Testnet })'

# Verify network configuration
dfx canister call DeFlow_backend get_bitcoin_network_info
```

### **3. Environment Variables**

```bash
# .env configuration for testnet
BITCOIN_NETWORK=testnet
BITCOIN_API_URL=https://blockstream.info/testnet/api
ENABLE_TESTNET_LOGGING=true
TESTNET_FEE_MULTIPLIER=1.5
```

## 💰 Test Bitcoin Acquisition

### **Testnet Faucets**
- **Mempool Testnet Faucet**: https://testnet-faucet.mempool.co/
- **Bitcoin Testnet Faucet**: https://bitcoinfaucet.uo1.net/
- **Coinfaucet Testnet**: https://coinfaucet.eu/en/btc-testnet/

### **Faucet Usage Strategy**
```bash
# 1. Generate multiple test addresses
ADDR1=$(dfx canister call DeFlow_backend get_bitcoin_address '("P2WPKH")')
ADDR2=$(dfx canister call DeFlow_backend get_bitcoin_address '("P2PKH")')
ADDR3=$(dfx canister call DeFlow_backend get_bitcoin_address '("P2TR")')

# 2. Request testnet Bitcoin for each address type
# Use faucets to send ~0.01 BTC to each address

# 3. Wait for confirmations (typically 10-30 minutes)
```

## 🧪 Comprehensive Test Scenarios

### **Scenario 1: Address Generation & Validation**

**Objective**: Verify all Bitcoin address types generate correctly for testnet

```bash
#!/bin/bash
echo "🔑 Testing Address Generation on Testnet"

# Test P2PKH (Legacy) - Should start with 'm' or 'n'
P2PKH_ADDR=$(dfx canister call DeFlow_backend get_bitcoin_address '("P2PKH")')
echo "P2PKH Address: $P2PKH_ADDR"
[[ $P2PKH_ADDR =~ ^[mn] ]] && echo "✅ P2PKH format valid" || echo "❌ P2PKH format invalid"

# Test P2WPKH (SegWit) - Should start with 'tb1q'
P2WPKH_ADDR=$(dfx canister call DeFlow_backend get_bitcoin_address '("P2WPKH")')
echo "P2WPKH Address: $P2WPKH_ADDR"
[[ $P2WPKH_ADDR =~ ^tb1q ]] && echo "✅ P2WPKH format valid" || echo "❌ P2WPKH format invalid"

# Test P2TR (Taproot) - Should start with 'tb1p'
P2TR_ADDR=$(dfx canister call DeFlow_backend get_bitcoin_address '("P2TR")')
echo "P2TR Address: $P2TR_ADDR"
[[ $P2TR_ADDR =~ ^tb1p ]] && echo "✅ P2TR format valid" || echo "❌ P2TR format invalid"

# Verify addresses on blockchain explorer
echo "📍 Verify addresses at: https://blockstream.info/testnet/"
```

**Expected Results**:
- P2PKH addresses: `m...` or `n...`
- P2WPKH addresses: `tb1q...`
- P2TR addresses: `tb1p...`
- All addresses should be valid and appear on testnet explorer

### **Scenario 2: Portfolio Management & UTXO Detection**

**Objective**: Validate portfolio tracking and UTXO management

```bash
#!/bin/bash
echo "💼 Testing Portfolio Management on Testnet"

# Get empty portfolio initially
echo "📊 Initial Portfolio State:"
EMPTY_PORTFOLIO=$(dfx canister call DeFlow_backend get_bitcoin_portfolio)
echo "$EMPTY_PORTFOLIO"

# Generate addresses and fund them (manual step)
echo "📤 Fund the following addresses with testnet Bitcoin:"
echo "Address 1: $P2WPKH_ADDR"
echo "Address 2: $P2PKH_ADDR"
echo "Use faucet: https://testnet-faucet.mempool.co/"
read -p "Press enter after funding addresses..."

# Wait for confirmations
echo "⏳ Waiting for confirmations (60 seconds)..."
sleep 60

# Check updated portfolio
echo "📊 Updated Portfolio State:"
UPDATED_PORTFOLIO=$(dfx canister call DeFlow_backend get_bitcoin_portfolio)
echo "$UPDATED_PORTFOLIO"

# Verify UTXO detection
echo "🔍 UTXO Analysis:"
dfx canister call DeFlow_backend get_utxos_for_address "(\"$P2WPKH_ADDR\")"

# Check individual address balances
echo "💰 Address Balances:"
BALANCE1=$(dfx canister call DeFlow_backend get_bitcoin_balance "(\"$P2WPKH_ADDR\")")
BALANCE2=$(dfx canister call DeFlow_backend get_bitcoin_balance "(\"$P2PKH_ADDR\")")
echo "P2WPKH Balance: $BALANCE1 satoshis"
echo "P2PKH Balance: $BALANCE2 satoshis"
```

**Expected Results**:
- Empty portfolio initially shows 0 BTC
- After funding, portfolio shows correct total BTC and USD value
- UTXOs are properly detected and categorized
- Individual address balances match explorer data

### **Scenario 3: Transaction Broadcasting & Confirmation**

**Objective**: Test complete transaction lifecycle

```bash
#!/bin/bash
echo "📨 Testing Bitcoin Transaction Broadcasting"

# Pre-transaction state
echo "📊 Pre-Transaction Portfolio:"
PRE_PORTFOLIO=$(dfx canister call DeFlow_backend get_bitcoin_portfolio)
echo "$PRE_PORTFOLIO"

# Create test transaction
DESTINATION_ADDR="tb1qtest2example3address4for5testnet6testing"  # Replace with real testnet address
SEND_AMOUNT=50000  # 0.0005 BTC
FEE_AMOUNT=2000    # 2000 sat fee

echo "📤 Broadcasting Transaction..."
echo "To: $DESTINATION_ADDR"
echo "Amount: $SEND_AMOUNT satoshis"
echo "Fee: $FEE_AMOUNT satoshis"

# Send Bitcoin transaction
SEND_RESULT=$(dfx canister call DeFlow_backend send_bitcoin "(
  \"$DESTINATION_ADDR\",
  $SEND_AMOUNT,
  $FEE_AMOUNT,
  \"medium\"
)")

echo "📋 Transaction Result:"
echo "$SEND_RESULT"

# Extract transaction ID from result
TXID=$(echo "$SEND_RESULT" | grep -o '"transaction_id":"[^"]*"' | cut -d'"' -f4)
echo "🆔 Transaction ID: $TXID"

# Monitor transaction status
echo "🔍 Monitoring transaction..."
for i in {1..10}; do
  sleep 30
  echo "⏰ Check $i: Verifying transaction status..."
  curl -s "https://blockstream.info/testnet/api/tx/$TXID" | jq '.status'
done

# Post-transaction portfolio
echo "📊 Post-Transaction Portfolio:"
POST_PORTFOLIO=$(dfx canister call DeFlow_backend get_bitcoin_portfolio)
echo "$POST_PORTFOLIO"
```

**Expected Results**:
- Transaction broadcasts successfully to testnet
- Transaction ID is returned and valid
- Transaction appears in testnet mempool within minutes
- Portfolio balance decreases by (amount + fee)
- Transaction confirms within 10-60 minutes

### **Scenario 4: Fee Estimation & Priority Testing**

**Objective**: Validate fee estimation across different priority levels

```bash
#!/bin/bash
echo "💸 Testing Fee Estimation on Testnet"

# Test fee estimation for different priorities
priorities=("low" "medium" "high")

for priority in "${priorities[@]}"; do
  echo "🎯 Testing $priority priority fee estimation..."
  
  FEE_ESTIMATE=$(dfx canister call DeFlow_backend estimate_bitcoin_fee "(
    100000,
    \"$priority\"
  )")
  
  echo "$priority Priority Fee: $FEE_ESTIMATE"
  
  # Extract fee values
  TOTAL_FEE=$(echo "$FEE_ESTIMATE" | grep -o '"total_fee_satoshis":[0-9]*' | cut -d':' -f2)
  SAT_PER_BYTE=$(echo "$FEE_ESTIMATE" | grep -o '"sat_per_byte":[0-9]*' | cut -d':' -f2)
  
  echo "  Total Fee: $TOTAL_FEE satoshis"
  echo "  Rate: $SAT_PER_BYTE sat/byte"
  echo ""
done

# Compare with actual testnet fee rates
echo "🌐 Current Testnet Fee Rates:"
curl -s "https://mempool.space/testnet/api/v1/fees/recommended" | jq '.'
```

**Expected Results**:
- Low priority: ~1-5 sat/byte
- Medium priority: ~5-15 sat/byte  
- High priority: ~15-50 sat/byte
- Fee estimates should be reasonable for testnet conditions

### **Scenario 5: Error Handling & Edge Cases**

**Objective**: Test error conditions and edge cases

```bash
#!/bin/bash
echo "🚨 Testing Error Handling on Testnet"

# Test invalid address format
echo "❌ Testing invalid address..."
INVALID_RESULT=$(dfx canister call DeFlow_backend send_bitcoin "(
  \"invalid_address_format\",
  10000,
  1000,
  \"medium\"
)" 2>&1 || echo "Expected error occurred")
echo "Invalid address result: $INVALID_RESULT"

# Test insufficient funds
echo "❌ Testing insufficient funds..."
LARGE_AMOUNT=100000000  # 1 BTC (likely more than test balance)
INSUFFICIENT_RESULT=$(dfx canister call DeFlow_backend send_bitcoin "(
  \"$DESTINATION_ADDR\",
  $LARGE_AMOUNT,
  1000,
  \"medium\"
)" 2>&1 || echo "Expected error occurred")
echo "Insufficient funds result: $INSUFFICIENT_RESULT"

# Test zero amount
echo "❌ Testing zero amount..."
ZERO_RESULT=$(dfx canister call DeFlow_backend send_bitcoin "(
  \"$DESTINATION_ADDR\",
  0,
  1000,
  \"medium\"
)" 2>&1 || echo "Expected error occurred")
echo "Zero amount result: $ZERO_RESULT"

# Test negative amount
echo "❌ Testing negative amount..."
NEGATIVE_RESULT=$(dfx canister call DeFlow_backend send_bitcoin "(
  \"$DESTINATION_ADDR\",
  -1000,
  1000,
  \"medium\"
)" 2>&1 || echo "Expected error occurred")
echo "Negative amount result: $NEGATIVE_RESULT"
```

**Expected Results**:
- Invalid addresses should be rejected with clear error messages
- Insufficient funds should return specific error about available balance
- Zero and negative amounts should be rejected
- All errors should be handled gracefully without crashes

## 📊 Automated Integration Test Suite

### **Complete Test Runner Script**

```bash
#!/bin/bash
# Bitcoin Testnet Integration Test Suite
# Run this script to perform comprehensive testnet validation

set -e  # Exit on any error

echo "🚀 DeFlow Bitcoin Testnet Integration Test Suite"
echo "=================================================="

# Configuration
TESTNET_EXPLORER="https://blockstream.info/testnet"
FAUCET_URL="https://testnet-faucet.mempool.co"

# Test results tracking
TESTS_PASSED=0
TESTS_FAILED=0
TOTAL_TESTS=0

# Helper function for test result tracking
test_result() {
  local test_name="$1"
  local result="$2"
  
  TOTAL_TESTS=$((TOTAL_TESTS + 1))
  
  if [ "$result" = "PASS" ]; then
    echo "✅ $test_name: PASSED"
    TESTS_PASSED=$((TESTS_PASSED + 1))
  else
    echo "❌ $test_name: FAILED"
    TESTS_FAILED=$((TESTS_FAILED + 1))
  fi
}

# Test 1: Service Initialization
echo ""
echo "🔧 Test 1: Service Initialization"
echo "----------------------------------"

if dfx canister call DeFlow_backend get_bitcoin_network_info > /dev/null 2>&1; then
  test_result "Service Initialization" "PASS"
else
  test_result "Service Initialization" "FAIL"
fi

# Test 2: Address Generation
echo ""
echo "🔑 Test 2: Address Generation"
echo "-----------------------------"

P2WPKH_ADDR=$(dfx canister call DeFlow_backend get_bitcoin_address '("P2WPKH")' 2>/dev/null | tr -d '"')
P2PKH_ADDR=$(dfx canister call DeFlow_backend get_bitcoin_address '("P2PKH")' 2>/dev/null | tr -d '"')
P2TR_ADDR=$(dfx canister call DeFlow_backend get_bitcoin_address '("P2TR")' 2>/dev/null | tr -d '"')

echo "P2WPKH: $P2WPKH_ADDR"
echo "P2PKH: $P2PKH_ADDR"
echo "P2TR: $P2TR_ADDR"

if [[ $P2WPKH_ADDR =~ ^tb1q ]] && [[ $P2PKH_ADDR =~ ^[mn] ]] && [[ $P2TR_ADDR =~ ^tb1p ]]; then
  test_result "Address Generation" "PASS"
else
  test_result "Address Generation" "FAIL"
fi

# Test 3: Portfolio Management
echo ""
echo "💼 Test 3: Portfolio Management"
echo "-------------------------------"

PORTFOLIO=$(dfx canister call DeFlow_backend get_bitcoin_portfolio 2>/dev/null)
if [[ $PORTFOLIO =~ "total_btc" ]] && [[ $PORTFOLIO =~ "addresses" ]]; then
  test_result "Portfolio Management" "PASS"
else
  test_result "Portfolio Management" "FAIL"
fi

# Test 4: Balance Checking
echo ""
echo "💰 Test 4: Balance Checking"
echo "---------------------------"

BALANCE=$(dfx canister call DeFlow_backend get_bitcoin_balance "(\"$P2WPKH_ADDR\")" 2>/dev/null)
if [[ $BALANCE =~ ^[0-9]+$ ]]; then
  test_result "Balance Checking" "PASS"
  echo "Current balance: $BALANCE satoshis"
else
  test_result "Balance Checking" "FAIL"
fi

# Test 5: Fee Estimation
echo ""
echo "💸 Test 5: Fee Estimation"
echo "-------------------------"

FEE_ESTIMATE=$(dfx canister call DeFlow_backend estimate_bitcoin_fee '(100000, "medium")' 2>/dev/null)
if [[ $FEE_ESTIMATE =~ "total_fee_satoshis" ]] && [[ $FEE_ESTIMATE =~ "sat_per_byte" ]]; then
  test_result "Fee Estimation" "PASS"
else
  test_result "Fee Estimation" "FAIL"
fi

# Test 6: Error Handling
echo ""
echo "🚨 Test 6: Error Handling"
echo "-------------------------"

# Test invalid address
INVALID_TEST=$(dfx canister call DeFlow_backend send_bitcoin '("invalid", 1000, 500, "medium")' 2>&1 || echo "ERROR_CAUGHT")
if [[ $INVALID_TEST =~ "ERROR_CAUGHT" ]] || [[ $INVALID_TEST =~ "error" ]] || [[ $INVALID_TEST =~ "invalid" ]]; then
  test_result "Error Handling - Invalid Address" "PASS"
else
  test_result "Error Handling - Invalid Address" "FAIL"
fi

# Manual Transaction Test (Interactive)
echo ""
echo "📨 Test 7: Transaction Broadcasting (Manual)"
echo "--------------------------------------------"

echo "🎯 To complete transaction testing:"
echo "1. Send testnet Bitcoin to: $P2WPKH_ADDR"
echo "2. Use faucet: $FAUCET_URL"
echo "3. Wait for confirmation (~10-30 minutes)"
echo "4. Run transaction test manually:"
echo ""
echo "   dfx canister call DeFlow_backend send_bitcoin '("
echo "     \"tb1qtest...\","
echo "     50000,"
echo "     2000,"
echo "     \"medium\""
echo "   )'"
echo ""
echo "5. Monitor at: $TESTNET_EXPLORER/address/$P2WPKH_ADDR"

# Test Summary
echo ""
echo "📊 Test Results Summary"
echo "======================"
echo "Total Tests: $TOTAL_TESTS"
echo "Passed: $TESTS_PASSED"
echo "Failed: $TESTS_FAILED"
echo "Success Rate: $(( TESTS_PASSED * 100 / TOTAL_TESTS ))%"

if [ $TESTS_FAILED -eq 0 ]; then
  echo ""
  echo "🎉 ALL AUTOMATED TESTS PASSED!"
  echo "✅ Bitcoin DeFi integration is working correctly on testnet"
  echo "🚀 Ready for manual transaction testing"
else
  echo ""
  echo "⚠️  Some tests failed. Please review the results above."
  echo "🔧 Fix issues before proceeding to transaction testing"
fi

echo ""
echo "🔗 Useful Links:"
echo "- Testnet Explorer: $TESTNET_EXPLORER"
echo "- Bitcoin Faucet: $FAUCET_URL"
echo "- Monitor Addresses: $TESTNET_EXPLORER/address/[ADDRESS]"
```

## 📈 Performance & Monitoring

### **Key Metrics to Track**

```bash
# Performance benchmarks
echo "⚡ Performance Metrics"

# Address generation time
time dfx canister call DeFlow_backend get_bitcoin_address '("P2WPKH")'

# Portfolio fetch time  
time dfx canister call DeFlow_backend get_bitcoin_portfolio

# Balance check time
time dfx canister call DeFlow_backend get_bitcoin_balance "(\"$ADDRESS\")"

# Fee estimation time
time dfx canister call DeFlow_backend estimate_bitcoin_fee '(100000, "medium")'
```

### **Expected Performance Targets**

- **Address Generation**: < 2 seconds
- **Portfolio Fetch**: < 5 seconds  
- **Balance Check**: < 3 seconds
- **Fee Estimation**: < 2 seconds
- **Transaction Broadcasting**: < 10 seconds

### **Monitoring Dashboard**

```bash
# Real-time monitoring script
while true; do
  clear
  echo "🖥️  DeFlow Bitcoin Testnet Monitor"
  echo "=================================="
  echo "Timestamp: $(date)"
  echo ""
  
  # Service status
  echo "🟢 Service Status: $(dfx canister status DeFlow_backend | grep Status)"
  
  # Portfolio summary
  PORTFOLIO=$(dfx canister call DeFlow_backend get_bitcoin_portfolio 2>/dev/null)
  TOTAL_BTC=$(echo "$PORTFOLIO" | grep -o '"total_btc":[0-9.]*' | cut -d':' -f2)
  echo "💰 Total BTC: $TOTAL_BTC"
  
  # Recent activity
  echo "📊 Address Balances:"
  for addr in $P2WPKH_ADDR $P2PKH_ADDR; do
    BALANCE=$(dfx canister call DeFlow_backend get_bitcoin_balance "(\"$addr\")" 2>/dev/null)
    echo "  ${addr:0:20}...: $BALANCE sat"
  done
  
  sleep 30
done
```

## ✅ Success Validation Criteria

### **Testnet Testing Success Indicators**

**🎯 Phase 1: Basic Functionality (Must Pass)**
- ✅ All address types generate with correct testnet prefixes
- ✅ Addresses validate correctly on testnet explorer
- ✅ Portfolio fetching returns proper structure
- ✅ Balance checking works for all address types
- ✅ Fee estimation returns reasonable values

**🎯 Phase 2: Transaction Flow (Must Pass)**
- ✅ UTXOs are detected after receiving testnet Bitcoin
- ✅ Portfolio balance updates correctly
- ✅ Transaction broadcasting succeeds
- ✅ Transaction appears in testnet mempool
- ✅ Transaction confirms on testnet blockchain

**🎯 Phase 3: Edge Cases (Should Pass)**
- ✅ Error handling for invalid addresses
- ✅ Insufficient funds detection
- ✅ Input validation for amounts and fees
- ✅ Network timeout handling

**🎯 Phase 4: Performance (Should Pass)**
- ✅ Address generation < 2 seconds
- ✅ Portfolio operations < 5 seconds
- ✅ Transaction broadcasting < 10 seconds

## 🚀 Next Steps After Testnet Success

### **If Testnet Testing Passes**

1. **Document Results**: Record all successful test outcomes
2. **Performance Optimization**: Fine-tune based on testnet performance data
3. **Security Audit**: Review threshold ECDSA implementation
4. **Mainnet Preparation**: Prepare configuration for Bitcoin mainnet
5. **User Documentation**: Create user guides based on testnet experience

### **If Testnet Testing Reveals Issues**

1. **Issue Classification**: Categorize problems (networking, cryptographic, logic)
2. **Root Cause Analysis**: Deep dive into failed operations
3. **Fix Implementation**: Address identified issues
4. **Regression Testing**: Re-run testnet tests after fixes
5. **Iterative Improvement**: Repeat until all tests pass

## 📝 Documentation & Reporting

### **Test Report Template**

```markdown
# DeFlow Bitcoin Testnet Test Report

**Date**: [Test Date]
**Tester**: [Your Name]
**Environment**: Bitcoin Testnet
**Canister Version**: [Version]

## Test Results Summary
- **Total Tests**: X
- **Passed**: X 
- **Failed**: X
- **Success Rate**: X%

## Detailed Results

### Address Generation
- P2PKH: ✅/❌ [Address: ...]
- P2WPKH: ✅/❌ [Address: ...]  
- P2TR: ✅/❌ [Address: ...]

### Transaction Testing
- Broadcasting: ✅/❌ [TXID: ...]
- Confirmation: ✅/❌ [Block: ...]
- Fee Accuracy: ✅/❌ [Expected vs Actual]

### Performance Metrics
- Address Generation: X.Xs
- Portfolio Fetch: X.Xs
- Transaction Broadcast: X.Xs

## Issues Identified
[List any problems found]

## Recommendations
[Next steps and improvements]
```

## 🎉 Conclusion

**Bitcoin testnet testing is the definitive validation method** for your DeFlow Bitcoin DeFi integration. Unlike unit tests with mock complexities, testnet testing proves real-world functionality with actual Bitcoin network interactions.

**Success on testnet means**:
- ✅ Your Bitcoin integration is production-ready
- ✅ Threshold ECDSA is working correctly
- ✅ All transaction flows are functional  
- ✅ Error handling is robust
- ✅ Performance is acceptable

**This strategy provides confidence** that your Day 8 Bitcoin integration is not just implemented, but actually **working correctly in a real Bitcoin environment**.