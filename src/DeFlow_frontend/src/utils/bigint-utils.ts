// Proper BigInt handling using bignumber.js as recommended by ICP
import BigNumber from 'bignumber.js'

// Configure BigNumber for ICP usage
BigNumber.config({
  EXPONENTIAL_AT: [-18, 18],
  DECIMAL_PLACES: 18,
  ROUNDING_MODE: BigNumber.ROUND_DOWN
})

export class BigIntUtils {
  /**
   * Apply decimals to a raw number (typically for token amounts)
   * This is the recommended approach from ICP community
   */
  static applyDecimals(rawNumber: bigint | string | number, decimals: number = 8): string {
    return new BigNumber(rawNumber.toString())
      .dividedBy(10 ** decimals)
      .toString()
  }

  /**
   * Remove decimals from a display number to get raw amount
   */
  static removeDecimals(displayNumber: string | number, decimals: number = 8): string {
    return new BigNumber(displayNumber.toString())
      .multipliedBy(10 ** decimals)
      .toFixed(0) // Remove decimal places for integer result
  }

  /**
   * Safe conversion from BigInt to string (for display)
   */
  static toString(value: bigint | string | number): string {
    return new BigNumber(value.toString()).toString()
  }

  /**
   * Safe conversion from BigInt to number (only for small values)
   */
  static toNumber(value: bigint | string | number): number {
    const bn = new BigNumber(value.toString())
    
    // Check if it's safe to convert to number
    if (bn.isGreaterThan(Number.MAX_SAFE_INTEGER)) {
      console.warn('BigInt too large for safe number conversion, using MAX_SAFE_INTEGER')
      return Number.MAX_SAFE_INTEGER
    }
    
    if (bn.isLessThan(Number.MIN_SAFE_INTEGER)) {
      console.warn('BigInt too small for safe number conversion, using MIN_SAFE_INTEGER')
      return Number.MIN_SAFE_INTEGER
    }
    
    return bn.toNumber()
  }

  /**
   * Create BigInt safely from various inputs
   */
  static toBigInt(value: string | number | bigint): bigint {
    try {
      if (typeof value === 'bigint') return value
      const bn = new BigNumber(value.toString())
      return BigInt(bn.toFixed(0))
    } catch (error) {
      console.warn('Failed to convert to BigInt:', error)
      return BigInt(0)
    }
  }

  /**
   * ICP timestamp handling (nanoseconds)
   */
  static timestampToDate(nanos: bigint | string | number): Date {
    try {
      // Convert nanoseconds to milliseconds
      const millis = new BigNumber(nanos.toString())
        .dividedBy(1_000_000)
        .toNumber()
      
      return new Date(millis)
    } catch (error) {
      console.warn('Failed to convert timestamp to date:', error)
      return new Date()
    }
  }

  /**
   * Convert Date to ICP timestamp (nanoseconds)
   */
  static dateToTimestamp(date: Date = new Date()): bigint {
    try {
      const millis = date.getTime()
      const nanos = new BigNumber(millis).multipliedBy(1_000_000)
      return BigInt(nanos.toFixed(0))
    } catch (error) {
      console.warn('Failed to convert date to timestamp:', error)
      return BigInt(Date.now() * 1_000_000)
    }
  }

  /**
   * Format large numbers for display with commas
   */
  static formatForDisplay(value: bigint | string | number, decimals: number = 8): string {
    try {
      const formatted = this.applyDecimals(value, decimals)
      const bn = new BigNumber(formatted)
      
      // Add commas for thousands
      return bn.toFormat()
    } catch (error) {
      console.warn('Failed to format for display:', error)
      return '0'
    }
  }

  /**
   * Compare two BigInt values safely
   */
  static compare(a: bigint | string | number, b: bigint | string | number): number {
    const aBN = new BigNumber(a.toString())
    const bBN = new BigNumber(b.toString())
    return aBN.comparedTo(bBN) || 0
  }

  /**
   * Add two BigInt values safely
   */
  static add(a: bigint | string | number, b: bigint | string | number): bigint {
    const result = new BigNumber(a.toString()).plus(b.toString())
    return BigInt(result.toFixed(0))
  }

  /**
   * Subtract two BigInt values safely
   */
  static subtract(a: bigint | string | number, b: bigint | string | number): bigint {
    const result = new BigNumber(a.toString()).minus(b.toString())
    return BigInt(result.toFixed(0))
  }

  /**
   * Check if a value is zero
   */
  static isZero(value: bigint | string | number): boolean {
    return new BigNumber(value.toString()).isZero()
  }

  /**
   * Check if a value is positive
   */
  static isPositive(value: bigint | string | number): boolean {
    return new BigNumber(value.toString()).isPositive()
  }

  /**
   * Get the maximum of two values
   */
  static max(a: bigint | string | number, b: bigint | string | number): bigint {
    const aBN = new BigNumber(a.toString())
    const bBN = new BigNumber(b.toString())
    const max = BigNumber.maximum(aBN, bBN)
    return BigInt(max.toFixed(0) || '0')
  }

  /**
   * Get the minimum of two values
   */
  static min(a: bigint | string | number, b: bigint | string | number): bigint {
    const aBN = new BigNumber(a.toString())
    const bBN = new BigNumber(b.toString())
    const min = BigNumber.minimum(aBN, bBN)
    return BigInt(min.toFixed(0) || '0')
  }
}

