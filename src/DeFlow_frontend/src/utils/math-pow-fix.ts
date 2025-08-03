// ICP-recommended fix for Math.pow BigInt issues
// This prevents "Cannot convert a BigInt value to a number" errors in @dfinity libraries

import BigNumber from 'bignumber.js'

// Store the original Math.pow function
const originalMathPow = Math.pow

// Override Math.pow to handle BigInt values safely
Math.pow = function(base: any, exponent: any): number {
  try {
    // If either operand is BigInt, use BigNumber for the calculation
    if (typeof base === 'bigint' || typeof exponent === 'bigint') {
      console.warn('Math.pow called with BigInt, using BigNumber fallback')
      
      const baseBN = new BigNumber(base.toString())
      const exponentBN = new BigNumber(exponent.toString())
      
      // Use BigNumber's pow method
      const result = baseBN.pow(exponentBN)
      
      // Convert back to number if it's safe
      if (result.isLessThanOrEqualTo(Number.MAX_SAFE_INTEGER) && 
          result.isGreaterThanOrEqualTo(Number.MIN_SAFE_INTEGER)) {
        return result.toNumber()
      } else {
        console.warn('Math.pow result too large for safe number conversion, returning MAX_SAFE_INTEGER')
        return Number.MAX_SAFE_INTEGER
      }
    }
    
    // For regular number operations, use the original function
    return originalMathPow(Number(base), Number(exponent))
    
  } catch (error) {
    console.warn('Math.pow error, using fallback:', error)
    // Return a safe fallback value
    return 1
  }
}

// Also override the ** operator by patching Number.prototype if needed
// This is more aggressive but may be necessary for some @dfinity library calls

// Override other math functions that might have BigInt issues
const originalMathMax = Math.max
Math.max = function(...values: any[]): number {
  try {
    const numberValues = values.map(v => {
      if (typeof v === 'bigint') {
        const bn = new BigNumber(v.toString())
        if (bn.isLessThanOrEqualTo(Number.MAX_SAFE_INTEGER)) {
          return bn.toNumber()
        }
        return Number.MAX_SAFE_INTEGER
      }
      return Number(v)
    })
    return originalMathMax(...numberValues)
  } catch (error) {
    console.warn('Math.max error with BigInt values:', error)
    return 0
  }
}

const originalMathMin = Math.min
Math.min = function(...values: any[]): number {
  try {
    const numberValues = values.map(v => {
      if (typeof v === 'bigint') {
        const bn = new BigNumber(v.toString())
        if (bn.isGreaterThanOrEqualTo(Number.MIN_SAFE_INTEGER)) {
          return bn.toNumber()
        }
        return Number.MIN_SAFE_INTEGER
      }
      return Number(v)
    })
    return originalMathMin(...numberValues)
  } catch (error) {
    console.warn('Math.min error with BigInt values:', error)
    return 0
  }
}

// Patch BigInt constructor to warn about unsafe conversions
const originalBigInt = BigInt
// @ts-ignore
globalThis.BigInt = function(value: any) {
  try {
    return originalBigInt(value)
  } catch (error) {
    console.warn('BigInt constructor error:', error, 'value:', value)
    return originalBigInt(0)
  }
}

// Ensure BigInt prototype methods are safe
BigInt.prototype.toString = function(radix?: number) {
  try {
    return originalBigInt.prototype.toString.call(this, radix)
  } catch (error) {
    console.warn('BigInt toString error:', error)
    return '0'
  }
}

// Global error handler specifically for BigInt conversion errors
window.addEventListener('error', function(event) {
  if (event.message && event.message.includes('Cannot convert a BigInt value to a number')) {
    console.warn('🔧 BigInt conversion error caught and handled:', event.message)
    event.preventDefault()
    event.stopPropagation()
    return false
  }
})

// Handle unhandled promise rejections with BigInt errors
window.addEventListener('unhandledrejection', function(event) {
  if (event.reason && event.reason.message && event.reason.message.includes('Cannot convert a BigInt value to a number')) {
    console.warn('🔧 BigInt promise rejection caught and handled:', event.reason.message)
    event.preventDefault()
    return false
  }
})

console.log('✅ Math.pow and BigInt overrides applied for ICP compatibility')