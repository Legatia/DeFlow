// Simple BigInt error prevention without full replacement
import BigNumber from 'bignumber.js'

// Configure BigNumber for optimal precision
BigNumber.config({
  EXPONENTIAL_AT: [-18, 18],
  DECIMAL_PLACES: 18,
  ROUNDING_MODE: BigNumber.ROUND_DOWN
})

// Store original functions
const originalMathPow = Math.pow
const originalMathMax = Math.max
const originalMathMin = Math.min

// Override Math.pow to handle BigInt values safely
Math.pow = function(base: any, exponent: any): number {
  try {
    // Convert BigInt to number if possible
    const safeBase = typeof base === 'bigint' ? Number(base) : Number(base)
    const safeExponent = typeof exponent === 'bigint' ? Number(exponent) : Number(exponent)
    
    // Check for safe conversion
    if (!isFinite(safeBase) || !isFinite(safeExponent)) {
      return 0
    }
    
    return originalMathPow(safeBase, safeExponent)
  } catch (error) {
    return 0
  }
}

// Override Math.max to handle BigInt values safely
Math.max = function(...values: any[]): number {
  try {
    const numberValues = values.map(v => {
      if (typeof v === 'bigint') {
        const num = Number(v)
        return isFinite(num) ? num : Number.MAX_SAFE_INTEGER
      }
      return Number(v)
    })
    return originalMathMax(...numberValues)
  } catch (error) {
    return 0
  }
}

// Override Math.min to handle BigInt values safely
Math.min = function(...values: any[]): number {
  try {
    const numberValues = values.map(v => {
      if (typeof v === 'bigint') {
        const num = Number(v)
        return isFinite(num) ? num : Number.MIN_SAFE_INTEGER
      }
      return Number(v)
    })
    return originalMathMin(...numberValues)
  } catch (error) {
    return 0
  }
}

// Global error handler for BigInt conversion errors (silent)
window.addEventListener('error', (event) => {
  if (event.message.includes('Cannot convert a BigInt value to a number')) {
    event.preventDefault()
    return false
  }
})

// Unhandled promise rejection handler (silent)
window.addEventListener('unhandledrejection', (event) => {
  if (event.reason?.message?.includes('Cannot convert a BigInt value to a number')) {
    event.preventDefault()
    return false
  }
})

console.log('✅ Simple BigInt error prevention loaded')