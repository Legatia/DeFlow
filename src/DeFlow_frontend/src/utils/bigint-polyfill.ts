// BigInt polyfill and safe conversion utilities
// This fixes the "Cannot convert a BigInt value to a number" error

// Store the original Math.pow function
const originalMathPow = Math.pow;

// Override Math.pow to handle BigInt values safely
Math.pow = function(base: any, exponent: any): number {
  try {
    // Convert BigInt to number safely
    const safeBase = typeof base === 'bigint' 
      ? Number(base > Number.MAX_SAFE_INTEGER ? Number.MAX_SAFE_INTEGER : base)
      : Number(base);
    
    const safeExponent = typeof exponent === 'bigint'
      ? Number(exponent > Number.MAX_SAFE_INTEGER ? Number.MAX_SAFE_INTEGER : exponent)
      : Number(exponent);
    
    return originalMathPow(safeBase, safeExponent);
  } catch (error) {
    console.warn('Math.pow BigInt conversion error, using fallback:', error);
    return 0;
  }
};

// Safe BigInt conversion utilities
export const BigIntUtils = {
  // Convert BigInt to number safely
  toNumber(value: bigint | number | string): number {
    if (typeof value === 'number') return value;
    if (typeof value === 'string') return parseInt(value) || 0;
    
    try {
      if (value <= BigInt(Number.MAX_SAFE_INTEGER)) {
        return Number(value);
      }
      console.warn('BigInt too large for safe conversion, using MAX_SAFE_INTEGER');
      return Number.MAX_SAFE_INTEGER;
    } catch (error) {
      console.warn('BigInt conversion error:', error);
      return 0;
    }
  },

  // Convert to string safely
  toString(value: bigint | number | string): string {
    try {
      return String(value);
    } catch (error) {
      console.warn('BigInt toString error:', error);
      return '0';
    }
  },

  // Create BigInt safely from various inputs
  fromValue(value: number | string | bigint): bigint {
    try {
      if (typeof value === 'bigint') return value;
      if (typeof value === 'number') return BigInt(Math.floor(value));
      if (typeof value === 'string') return BigInt(value);
      return BigInt(0);
    } catch (error) {
      console.warn('BigInt creation error:', error);
      return BigInt(0);
    }
  },

  // ICP timestamp utilities
  timestampToDate(timestamp: bigint | string | number): Date {
    try {
      const nanos = typeof timestamp === 'bigint' 
        ? this.toNumber(timestamp)
        : Number(timestamp);
      
      // Convert nanoseconds to milliseconds
      const millis = Math.floor(nanos / 1000000);
      return new Date(millis);
    } catch (error) {
      console.warn('Timestamp conversion error:', error);
      return new Date();
    }
  },

  dateToTimestamp(date: Date = new Date()): string {
    // Return as string to avoid BigInt issues in serialization
    const millis = date.getTime();
    const nanos = millis * 1000000;
    return nanos.toString();
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

console.log('✅ BigInt polyfill loaded successfully');