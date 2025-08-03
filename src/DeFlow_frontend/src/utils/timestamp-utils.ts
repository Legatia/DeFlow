// Simple timestamp utilities using BigNumber.js - NO BigInt!
import BigNumber from 'bignumber.js'

// Configure BigNumber for ICP usage
BigNumber.config({
  EXPONENTIAL_AT: [-18, 18],
  DECIMAL_PLACES: 0, // We want integers for timestamps
  ROUNDING_MODE: BigNumber.ROUND_DOWN
})

export class TimestampUtils {
  /**
   * Convert JavaScript Date to ICP timestamp string (nanoseconds)
   */
  static dateToICPTimestamp(date: Date = new Date()): string {
    const milliseconds = date.getTime()
    const nanoseconds = new BigNumber(milliseconds).multipliedBy(1_000_000)
    return nanoseconds.toString()
  }

  /**
   * Convert ICP timestamp string (nanoseconds) to JavaScript Date
   */
  static icpTimestampToDate(timestamp: string): Date {
    try {
      const nanoseconds = new BigNumber(timestamp)
      const milliseconds = nanoseconds.dividedBy(1_000_000)
      return new Date(milliseconds.toNumber())
    } catch (error) {
      console.warn('Failed to convert timestamp to date:', error)
      return new Date()
    }
  }

  /**
   * Format ICP timestamp as human-readable string
   */
  static formatICPTimestamp(timestamp: string): string {
    return this.icpTimestampToDate(timestamp).toISOString()
  }

  /**
   * Get current timestamp as ICP format string
   */
  static now(): string {
    return this.dateToICPTimestamp(new Date())
  }

  /**
   * Format timestamp for display
   */
  static formatForDisplay(timestamp: string): string {
    try {
      return this.icpTimestampToDate(timestamp).toLocaleString()
    } catch (error) {
      console.warn('Failed to format timestamp for display:', error)
      return 'Invalid date'
    }
  }

  /**
   * Calculate duration between two timestamps
   */
  static calculateDuration(startTimestamp: string, endTimestamp?: string): number {
    try {
      const start = this.icpTimestampToDate(startTimestamp)
      const end = endTimestamp ? this.icpTimestampToDate(endTimestamp) : new Date()
      return end.getTime() - start.getTime()
    } catch (error) {
      console.warn('Failed to calculate duration:', error)
      return 0
    }
  }

  /**
   * Format duration in human-readable format
   */
  static formatDuration(durationMs: number): string {
    if (durationMs < 1000) return `${durationMs}ms`
    if (durationMs < 60000) return `${Math.round(durationMs / 1000)}s`
    if (durationMs < 3600000) return `${Math.round(durationMs / 60000)}m`
    return `${Math.round(durationMs / 3600000)}h`
  }

  /**
   * Compare two timestamps
   */
  static compare(timestamp1: string, timestamp2: string): number {
    try {
      const ts1 = new BigNumber(timestamp1)
      const ts2 = new BigNumber(timestamp2)
      return ts1.comparedTo(ts2) || 0
    } catch (error) {
      console.warn('Failed to compare timestamps:', error)
      return 0
    }
  }

  /**
   * Check if timestamp is valid
   */
  static isValid(timestamp: string): boolean {
    try {
      const bn = new BigNumber(timestamp)
      return bn.isFinite() && bn.isPositive()
    } catch (error) {
      return false
    }
  }
}

console.log('✅ Timestamp utilities loaded (BigNumber.js, no BigInt)')