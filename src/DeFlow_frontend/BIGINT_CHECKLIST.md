# BigInt Avoidance Checklist

## 🏛️ ICP Community Standards

**Official Guidance**: "The ICP community recommends using bignumber.js for all math involving large numbers to avoid JavaScript limitations with BigInt conversion."

**Key Community Requirements:**
- ✅ Use BigNumber.js for token amounts and large values from canisters
- ✅ Never mix BigInt and number types in operations  
- ✅ Use explicit conversion with `.toString()` when necessary
- ✅ Keep calculations in BigNumber throughout for consistency

## 🚀 Quick Start for New Developers

### 1. Essential Setup (Required for Every New File)

**When creating ANY new service/component that might use BigInt:**

```typescript
// ✅ STEP 1: Import polyfill FIRST (before any other imports)
import '../utils/bigint-polyfill'

// ✅ STEP 2: Import BigIntUtils for safe operations
import { BigIntUtils } from '../utils/bigint-utils'

// ✅ STEP 3: Then your other imports
import { Actor, HttpAgent } from '@dfinity/agent'
import React from 'react'
```

### 2. Pre-Development Checklist

- [ ] ✅ Polyfill imported first in file
- [ ] ✅ BigIntUtils imported for numeric operations
- [ ] ✅ Console shows "BigInt completely replaced with BigNumber.js"
- [ ] ✅ No direct BigInt() constructor usage planned
- [ ] ✅ All canister responses will be converted using BigIntUtils

### 3. Development Checklist

**For ICP Canister Integration:**
- [ ] ✅ Use `BigIntUtils.dateToTimestamp()` for timestamps
- [ ] ✅ Use `BigIntUtils.timestampToDate()` for timestamp conversion
- [ ] ✅ Convert canister responses with `BigIntUtils.toString()`
- [ ] ✅ Use `BigIntUtils.toNumber()` for display values
- [ ] ✅ Handle token amounts with `applyDecimals()`/`removeDecimals()`

**For UI Components:**
- [ ] ✅ Convert BigInt to string before passing to React props
- [ ] ✅ Use BigIntUtils formatting for display values
- [ ] ✅ Handle form inputs with safe number conversion
- [ ] ✅ JSON serialization uses converted values

**For Mathematical Operations:**
- [ ] ✅ Use `BigIntUtils.compare()` for comparisons
- [ ] ✅ Use `BigIntUtils.add()`/`subtract()` for arithmetic
- [ ] ✅ Use `BigIntUtils.max()`/`min()` for range operations
- [ ] ✅ Avoid direct Math.pow with BigInt values

### 4. Testing Checklist

**Browser Console Validation:**
- [ ] ✅ `console.log(BigInt(123))` shows wrapped object, not native BigInt
- [ ] ✅ No "Cannot convert BigInt to number" errors
- [ ] ✅ `BigIntUtils.formatForDisplay('1234567890', 8)` works correctly
- [ ] ✅ Frontend loads without BigInt-related crashes

**Functional Testing:**
- [ ] ✅ Canister calls return properly formatted data
- [ ] ✅ UI displays large numbers correctly
- [ ] ✅ Form submissions handle large values
- [ ] ✅ JSON operations work with all data structures

### 5. Common Patterns Reference

**✅ CORRECT Patterns (ICP Community Compliant):**
```typescript
// ICP Community Recommended Token Pattern
import BigNumber from "bignumber.js";
const display = new BigNumber(rawAmount).dividedBy(10 ** decimals).toString();

// DeFlow BigIntUtils (Implements Community Pattern)
const display = BigIntUtils.applyDecimals(rawAmount, 8)  // Uses BigNumber internally
const raw = BigIntUtils.removeDecimals(displayAmount, 8)

// Safe comparisons (no type mixing)
if (BigIntUtils.compare(valueA, valueB) > 0) { /* ... */ }

// Explicit conversions (community guidance)
const str = BigIntUtils.toString(bigIntValue)  // Always safe
const num = BigIntUtils.toNumber(bigIntValue)  // Only for safe range

// BigNumber methods for math (not native Math)
const result = BigNumber(base).pow(exponent)  // Instead of Math.pow

// Timestamps (ICP nanosecond handling)
const timestamp = BigIntUtils.dateToTimestamp()
const date = BigIntUtils.timestampToDate(canisterTimestamp)
```

**❌ WRONG Patterns (Violate ICP Community Guidelines):**
```typescript
// Type mixing (causes JavaScript crashes)
const result = BigInt(123) + Number(456)  // NEVER mix types

// Direct BigInt usage 
const timestamp = BigInt(Date.now() * 1_000_000)  // Use BigNumber.js instead

// Math operations with BigInt (expects number types)
const result = Math.pow(BigInt(2), BigInt(8))  // Use BigNumber methods

// Implicit conversions (precision loss)
const num = Number(bigIntValue)  // Use explicit .toString() then Number()

// JSON serialization with BigInt
JSON.stringify({ value: bigIntValue })  // Convert with BigIntUtils.toString first

// React props with BigInt
<Component value={bigIntValue} />  // Convert with BigIntUtils.toString first

// Direct arithmetic (type mixing)
const sum = bigIntA + bigIntB  // Use BigIntUtils.add for consistency
```

## 🔧 Debugging Guide

### Issue: "BigInt is not defined" or "BigInt is not a function"
**Cause**: Polyfill not loaded before BigInt usage
**Fix**: Ensure `import '../utils/bigint-polyfill'` is the FIRST import

### Issue: "Cannot convert BigInt to number"
**Cause**: Native BigInt bypassed polyfill system
**Fix**: Check import order, use BigIntUtils.toNumber()

### Issue: JSON serialization errors
**Cause**: BigInt values in objects being serialized
**Fix**: Use BigIntUtils.toString() before serialization

### Issue: Math operations failing
**Cause**: BigInt values in Math functions
**Fix**: Use BigIntUtils methods for arithmetic

## 📊 Performance Guidelines

### Acceptable Usage Patterns:
- ✅ **Small numbers (< 1M)**: Direct number conversion with BigIntUtils.toNumber()
- ✅ **Display values**: String conversion with BigIntUtils.toString()
- ✅ **Token amounts**: Decimal conversion with applyDecimals()/removeDecimals()
- ✅ **Timestamps**: ICP nanosecond conversion with timestamp utilities

### Optimization Tips:
- 🔄 **Cache converted values** instead of converting repeatedly
- 🔄 **Use string storage** for large numbers in state
- 🔄 **Convert once at API boundaries** rather than throughout code
- 🔄 **Prefer BigIntUtils methods** over direct BigNumber.js usage

## 🎯 Production Deployment Checklist

**Pre-deployment Validation:**
- [ ] ✅ `npm run build` succeeds without BigInt TypeScript errors
- [ ] ✅ `npm run dev` starts without console BigInt errors
- [ ] ✅ All canister integration tests pass
- [ ] ✅ UI components render large numbers correctly
- [ ] ✅ No performance regressions from BigNumber.js usage

**Post-deployment Monitoring:**
- [ ] ✅ Browser console shows no BigInt conversion errors
- [ ] ✅ Application loads without crashes
- [ ] ✅ Canister communication works correctly
- [ ] ✅ User can perform all numeric operations

## 🚨 Emergency BigInt Issue Detection

**Add this to any component suspected of BigInt issues:**
```typescript
useEffect(() => {
  const originalError = console.error;
  console.error = (...args) => {
    if (args.some(arg => typeof arg === 'string' && arg.includes('BigInt'))) {
      console.log('🚨 BigInt issue detected in component:', args);
      // Add component name and state info for debugging
    }
    originalError.apply(console, args);
  };
  
  return () => {
    console.error = originalError;
  };
}, []);
```

---

## ✅ Final Verification

**Your implementation is correct when:**
1. Console shows: "✅ BigInt completely replaced with BigNumber.js"
2. No BigInt-related errors in browser console
3. All numeric operations work correctly with large values
4. Frontend loads and functions without crashes
5. Canister integration works seamlessly

**Remember**: When in doubt, ALWAYS use BigIntUtils methods instead of direct BigInt operations!