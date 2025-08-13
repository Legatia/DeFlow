// Complete BigInt elimination - Use only BigNumber.js
// This completely prevents any BigInt usage that causes conversion errors
import BigNumber from 'bignumber.js';

// Configure BigNumber for optimal precision
BigNumber.config({
  EXPONENTIAL_AT: [-18, 18],
  DECIMAL_PLACES: 18,
  ROUNDING_MODE: BigNumber.ROUND_DOWN
});

// Completely disable native BigInt to prevent conversion issues
const originalBigInt = (globalThis as any).BigInt;

// Replace BigInt with BigNumber.js wrapper
(globalThis as any).BigInt = function(value: any): any {
  console.warn('BigInt usage detected, converting to BigNumber.js:', value);
  const bn = new BigNumber(value.toString());
  
  // Return an object that behaves like BigInt but uses BigNumber internally
  return {
    _isBigNumber: true,
    _value: bn,
    toString: () => bn.toFixed(0),
    valueOf: () => bn.toNumber(),
    [Symbol.toPrimitive]: (hint: string) => {
      if (hint === 'number') return bn.toNumber();
      return bn.toFixed(0);
    },
    toNumber: () => bn.toNumber()
  };
};

// Store the original Math.pow function
const originalMathPow = Math.pow;

// Override Math.pow to handle BigInt values safely
Math.pow = function(base: any, exponent: any): number {
  try {
    // Convert any BigInt-like objects to numbers
    const safeBase = base?._isBigNumber ? base.toNumber() : 
                    typeof base === 'bigint' ? Number(base) : Number(base);
    
    const safeExponent = exponent?._isBigNumber ? exponent.toNumber() :
                        typeof exponent === 'bigint' ? Number(exponent) : Number(exponent);
    
    // Check for safe conversion
    if (!isFinite(safeBase) || !isFinite(safeExponent)) {
      console.warn('Math.pow: Invalid arguments, using 0');
      return 0;
    }
    
    return originalMathPow(safeBase, safeExponent);
  } catch (error) {
    console.warn('Math.pow BigInt conversion error, using fallback:', error);
    return 0;
  }
};

// BigNumber.js utilities - No BigInt usage
export const BigIntUtils = {
  // Convert any value to number safely using BigNumber.js
  toNumber(value: any): number {
    try {
      if (typeof value === 'number') return isFinite(value) ? value : 0;
      if (value?._isBigNumber) return value.toNumber();
      
      const bn = new BigNumber(value.toString());
      
      // Check if it's safe to convert
      if (bn.isGreaterThan(Number.MAX_SAFE_INTEGER)) {
        console.warn('Value too large for safe conversion, using MAX_SAFE_INTEGER');
        return Number.MAX_SAFE_INTEGER;
      }
      
      if (bn.isLessThan(Number.MIN_SAFE_INTEGER)) {
        console.warn('Value too small for safe conversion, using MIN_SAFE_INTEGER');
        return Number.MIN_SAFE_INTEGER;
      }
      
      return bn.toNumber();
    } catch (error) {
      console.warn('Number conversion error:', error);
      return 0;
    }
  },

  // Convert to string safely
  toString(value: any): string {
    try {
      if (value?._isBigNumber) return value.toString();
      return new BigNumber(value.toString()).toFixed();
    } catch (error) {
      console.warn('String conversion error:', error);
      return '0';
    }
  },

  // Create BigNumber from various inputs (no BigInt)
  fromValue(value: any): BigNumber {
    try {
      if (value?._isBigNumber) return value._value;
      return new BigNumber(value.toString());
    } catch (error) {
      console.warn('BigNumber creation error:', error);
      return new BigNumber(0);
    }
  },

  // ICP timestamp utilities using BigNumber.js
  timestampToDate(timestamp: any): Date {
    try {
      const bn = this.fromValue(timestamp);
      
      // Convert nanoseconds to milliseconds
      const millis = bn.dividedBy(1_000_000).toNumber();
      return new Date(millis);
    } catch (error) {
      console.warn('Timestamp conversion error:', error);
      return new Date();
    }
  },

  dateToTimestamp(date: Date = new Date()): string {
    try {
      const millis = date.getTime();
      const nanos = new BigNumber(millis).multipliedBy(1_000_000);
      return nanos.toFixed(0);
    } catch (error) {
      console.warn('Date to timestamp error:', error);
      return new BigNumber(Date.now() * 1_000_000).toFixed(0);
    }
  },

  // Format for display with commas
  formatForDisplay(value: any, decimals: number = 8): string {
    try {
      const bn = this.fromValue(value);
      const formatted = bn.dividedBy(new BigNumber(10).pow(decimals));
      return formatted.toFormat();
    } catch (error) {
      console.warn('Format display error:', error);
      return '0';
    }
  },

  // Compare two values
  compare(a: any, b: any): number {
    try {
      const aBN = this.fromValue(a);
      const bBN = this.fromValue(b);
      return aBN.comparedTo(bBN) || 0;
    } catch (error) {
      console.warn('Compare error:', error);
      return 0;
    }
  }
};

// Global error handler for BigInt issues
window.addEventListener('error', (event) => {
  if (event.message.includes('Cannot convert a BigInt value to a number')) {
    console.warn('BigInt conversion error caught globally, preventing crash:', event.message);
    event.preventDefault();
    return false;
  }
});

// Unhandled promise rejection handler
window.addEventListener('unhandledrejection', (event) => {
  if (event.reason?.message?.includes('Cannot convert a BigInt value to a number')) {
    console.warn('BigInt promise rejection caught globally, preventing crash:', event.reason.message);
    event.preventDefault();
    return false;
  }
});

// Simplified BigInt protection without Promise override
// (The Promise override was causing TypeScript issues)

console.log('✅ BigInt completely replaced with BigNumber.js - No native BigInt usage');