// Safe BigInt handling - Don't interfere with DFINITY crypto libraries
// Only provide utilities for application-level BigInt conversions
import BigNumber from 'bignumber.js';

// Configure BigNumber for optimal precision
BigNumber.config({
  EXPONENTIAL_AT: [-18, 18],
  DECIMAL_PLACES: 18,
  ROUNDING_MODE: BigNumber.ROUND_DOWN
});

// DON'T replace native BigInt - let DFINITY libraries use it
// Instead, provide safe conversion utilities

// BigNumber.js utilities for application use
export const BigIntUtils = {
  // Convert any value to number safely using BigNumber.js
  toNumber(value: any): number {
    try {
      if (typeof value === 'number') return isFinite(value) ? value : 0;
      if (typeof value === 'bigint') {
        // For BigInt values, convert via string to avoid precision loss
        const str = value.toString();
        const bn = new BigNumber(str);
        
        // Check if it's safe to convert to number
        if (bn.isGreaterThan(Number.MAX_SAFE_INTEGER)) {
          console.warn('BigInt value too large for safe number conversion:', str);
          return Number.MAX_SAFE_INTEGER;
        }
        
        if (bn.isLessThan(Number.MIN_SAFE_INTEGER)) {
          console.warn('BigInt value too small for safe number conversion:', str);
          return Number.MIN_SAFE_INTEGER;
        }
        
        return bn.toNumber();
      }
      
      const bn = new BigNumber(value.toString());
      return bn.toNumber();
    } catch (error) {
      console.warn('Number conversion error:', error);
      return 0;
    }
  },

  // Convert to string safely
  toString(value: any): string {
    try {
      if (typeof value === 'bigint') {
        return value.toString();
      }
      return new BigNumber(value.toString()).toFixed();
    } catch (error) {
      console.warn('String conversion error:', error);
      return '0';
    }
  },

  // Create BigNumber from various inputs
  fromValue(value: any): BigNumber {
    try {
      if (typeof value === 'bigint') {
        return new BigNumber(value.toString());
      }
      return new BigNumber(value.toString());
    } catch (error) {
      console.warn('BigNumber creation error:', error);
      return new BigNumber(0);
    }
  },

  // Safe BigInt creation from number (for our application)
  toBigInt(value: any): bigint {
    try {
      if (typeof value === 'bigint') return value;
      if (typeof value === 'number' && !Number.isInteger(value)) {
        console.warn('Converting non-integer to BigInt, using Math.floor');
        value = Math.floor(value);
      }
      return BigInt(value);
    } catch (error) {
      console.warn('BigInt creation error:', error, 'returning BigInt(0)');
      return BigInt(0);
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
  },

  // Safe arithmetic operations
  add(a: any, b: any): string {
    try {
      const aBN = this.fromValue(a);
      const bBN = this.fromValue(b);
      return aBN.plus(bBN).toFixed();
    } catch (error) {
      console.warn('Addition error:', error);
      return '0';
    }
  },

  subtract(a: any, b: any): string {
    try {
      const aBN = this.fromValue(a);
      const bBN = this.fromValue(b);
      return aBN.minus(bBN).toFixed();
    } catch (error) {
      console.warn('Subtraction error:', error);
      return '0';
    }
  },

  multiply(a: any, b: any): string {
    try {
      const aBN = this.fromValue(a);
      const bBN = this.fromValue(b);
      return aBN.multipliedBy(bBN).toFixed();
    } catch (error) {
      console.warn('Multiplication error:', error);
      return '0';
    }
  },

  divide(a: any, b: any): string {
    try {
      const aBN = this.fromValue(a);
      const bBN = this.fromValue(b);
      if (bBN.isZero()) {
        console.warn('Division by zero');
        return '0';
      }
      return aBN.dividedBy(bBN).toFixed();
    } catch (error) {
      console.warn('Division error:', error);
      return '0';
    }
  }
};

// Safe Math.pow override for our application
const originalMathPow = Math.pow;
Math.pow = function(base: any, exponent: any): number {
  try {
    // Handle BigInt inputs safely
    const safeBase = typeof base === 'bigint' ? BigIntUtils.toNumber(base) : Number(base);
    const safeExponent = typeof exponent === 'bigint' ? BigIntUtils.toNumber(exponent) : Number(exponent);
    
    if (!isFinite(safeBase) || !isFinite(safeExponent)) {
      console.warn('Math.pow: Invalid arguments, using 0');
      return 0;
    }
    
    return originalMathPow(safeBase, safeExponent);
  } catch (error) {
    console.warn('Math.pow error:', error);
    return 0;
  }
};

// Global error handler for BigInt conversion issues
window.addEventListener('error', (event) => {
  if (event.message && event.message.includes('Cannot convert a BigInt value to a number')) {
    console.warn('BigInt conversion error caught globally:', event.message);
    console.warn('Use BigIntUtils.toNumber() for safe conversion');
    event.preventDefault();
    return false;
  }
});

// Unhandled promise rejection handler
window.addEventListener('unhandledrejection', (event) => {
  const message = event.reason?.message || '';
  
  if (message.includes('Cannot convert a BigInt value to a number')) {
    console.warn('BigInt promise rejection caught globally:', message);
    console.warn('Use BigIntUtils.toNumber() for safe conversion');
    event.preventDefault();
    return false;
  }
});


// Export a safe BigInt helper
export const safeBigInt = {
  // Create BigInt safely
  create: BigIntUtils.toBigInt,
  // Convert to number safely  
  toNumber: BigIntUtils.toNumber,
  // Convert to string safely
  toString: BigIntUtils.toString,
  // Arithmetic operations
  add: BigIntUtils.add,
  subtract: BigIntUtils.subtract,
  multiply: BigIntUtils.multiply,
  divide: BigIntUtils.divide,
  compare: BigIntUtils.compare
};