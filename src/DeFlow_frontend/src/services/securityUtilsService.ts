// SECURITY: Security utilities for HTTPS enforcement and error sanitization
import '../utils/bigint-polyfill'

class SecurityUtilsService {
  // SECURITY: Enforce HTTPS in all environments except localhost
  enforceHTTPS(): void {
    if (this.shouldEnforceHTTPS() && !this.isHTTPS()) {
      const httpsUrl = window.location.href.replace('http://', 'https://')
      console.warn('SECURITY: Redirecting to HTTPS for secure communication')
      window.location.replace(httpsUrl)
    }
  }

  // SECURITY: Check if HTTPS should be enforced
  private shouldEnforceHTTPS(): boolean {
    const hostname = window.location.hostname
    
    // Don't enforce HTTPS for localhost development (including canister subdomains)
    if (hostname === 'localhost' || hostname === '127.0.0.1' || hostname.endsWith('.localhost')) {
      return false
    }
    
    // Don't enforce for local development IPs
    if (hostname.startsWith('192.168.') || hostname.startsWith('10.') || hostname.startsWith('172.')) {
      return false
    }
    
    // Don't enforce for local IC development domains
    if (hostname.includes('localhost') || hostname.includes('127.0.0.1')) {
      return false
    }
    
    // Only enforce HTTPS for production domains
    return process.env.NODE_ENV === 'production' && !hostname.includes('localhost')
  }

  // SECURITY: Check if current connection is HTTPS
  private isHTTPS(): boolean {
    return window.location.protocol === 'https:'
  }

  // SECURITY: Sanitize error messages to prevent information disclosure
  sanitizeErrorMessage(error: Error | string | unknown, context?: string): string {
    let message: string

    if (error instanceof Error) {
      message = error.message
    } else if (typeof error === 'string') {
      message = error
    } else {
      message = 'An unexpected error occurred'
    }

    // Remove potentially sensitive information
    const sanitized = message
      .replace(/\b(?:\d{1,3}\.){3}\d{1,3}\b/g, '[IP_ADDRESS]') // IP addresses
      .replace(/\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b/g, '[EMAIL]') // Email addresses
      .replace(/\b(?:key|token|secret|password|hash)\s*[=:]\s*[^\s]+/gi, '[REDACTED]') // API keys/tokens
      .replace(/\b[A-Za-z0-9]{20,}\b/g, '[TOKEN]') // Long alphanumeric strings (likely tokens)
      .replace(/\bfile:\/\/[^\s]+/g, '[FILE_PATH]') // File paths
      .replace(/\bc:\\[^\s]+/gi, '[FILE_PATH]') // Windows paths
      .replace(/\/[a-zA-Z0-9_\-\/]+\.(js|ts|jsx|tsx|json)/g, '[SOURCE_FILE]') // Source file paths
      .replace(/at\s+[^\s]+\s+\([^)]+\)/g, '[STACK_TRACE]') // Stack trace locations
      .replace(/line\s+\d+/gi, '[LINE_NUMBER]') // Line numbers
      .replace(/column\s+\d+/gi, '[COLUMN_NUMBER]') // Column numbers

    // Limit message length to prevent verbose internal errors
    const maxLength = 200
    const truncated = sanitized.length > maxLength 
      ? sanitized.substring(0, maxLength) + '...' 
      : sanitized

    // Add context if provided
    const contextPrefix = context ? `${context}: ` : ''
    
    return `${contextPrefix}${truncated}`
  }

  // SECURITY: Create user-friendly error messages
  getUserFriendlyErrorMessage(error: Error | string | unknown, operation?: string): string {
    const sanitized = this.sanitizeErrorMessage(error)
    
    // Map common error patterns to user-friendly messages
    if (sanitized.toLowerCase().includes('network')) {
      return 'Network connection failed. Please check your internet connection and try again.'
    }
    
    if (sanitized.toLowerCase().includes('timeout')) {
      return 'The request timed out. Please try again.'
    }
    
    if (sanitized.toLowerCase().includes('unauthorized') || sanitized.toLowerCase().includes('authentication')) {
      return 'Authentication failed. Please log in again.'
    }
    
    if (sanitized.toLowerCase().includes('forbidden') || sanitized.toLowerCase().includes('permission')) {
      return 'You do not have permission to perform this action.'
    }
    
    if (sanitized.toLowerCase().includes('not found')) {
      return 'The requested resource was not found.'
    }
    
    if (sanitized.toLowerCase().includes('validation') || sanitized.toLowerCase().includes('invalid')) {
      return 'Please check your input and try again.'
    }
    
    if (sanitized.toLowerCase().includes('rate limit')) {
      return 'Too many requests. Please wait a moment and try again.'
    }
    
    if (sanitized.toLowerCase().includes('server error') || sanitized.toLowerCase().includes('internal error')) {
      return 'A server error occurred. Please try again later.'
    }

    // Default message based on operation
    const operationPrefix = operation ? `Failed to ${operation}` : 'An error occurred'
    return `${operationPrefix}. Please try again or contact support if the problem persists.`
  }

  // SECURITY: Validate and sanitize URLs
  sanitizeAndValidateURL(url: string, allowedProtocols: string[] = ['https:', 'http:']): string | null {
    try {
      // Remove any malicious characters
      const cleanUrl = url.trim().replace(/[<>\"']/g, '')
      
      const parsedUrl = new URL(cleanUrl)
      
      // Check protocol
      if (!allowedProtocols.includes(parsedUrl.protocol)) {
        console.warn(`SECURITY: Blocked unsafe protocol: ${parsedUrl.protocol}`)
        return null
      }
      
      // Block javascript: and data: protocols
      if (parsedUrl.protocol === 'javascript:' || parsedUrl.protocol === 'data:') {
        console.warn(`SECURITY: Blocked dangerous protocol: ${parsedUrl.protocol}`)
        return null
      }
      
      // In production, enforce HTTPS for external requests
      if (this.shouldEnforceHTTPS() && parsedUrl.protocol === 'http:' && !this.isLocalhost(parsedUrl.hostname)) {
        console.warn('SECURITY: Converting HTTP URL to HTTPS')
        parsedUrl.protocol = 'https:'
      }
      
      return parsedUrl.toString()
    } catch (error) {
      console.warn('SECURITY: Invalid URL provided:', this.sanitizeErrorMessage(error))
      return null
    }
  }

  // SECURITY: Check if hostname is localhost
  private isLocalhost(hostname: string): boolean {
    return hostname === 'localhost' || 
           hostname === '127.0.0.1' || 
           hostname.startsWith('192.168.') || 
           hostname.startsWith('10.') || 
           hostname.startsWith('172.')
  }

  // SECURITY: Secure fetch wrapper with automatic HTTPS upgrade
  async secureFetch(url: string, options: RequestInit = {}): Promise<Response> {
    // Validate and sanitize URL
    const sanitizedUrl = this.sanitizeAndValidateURL(url)
    if (!sanitizedUrl) {
      throw new Error('Invalid URL provided')
    }

    // Add security headers
    const secureHeaders = {
      ...options.headers,
      'X-Requested-With': 'XMLHttpRequest',
      'Cache-Control': 'no-cache',
      'Pragma': 'no-cache'
    }

    const secureOptions: RequestInit = {
      ...options,
      headers: secureHeaders,
      mode: 'cors',
      credentials: 'same-origin'
    }

    try {
      const response = await fetch(sanitizedUrl, secureOptions)
      
      // Log failed requests for monitoring
      if (!response.ok) {
        console.warn(`SECURITY: Request failed - ${response.status} ${response.statusText}`)
      }
      
      return response
    } catch (error) {
      // Sanitize and re-throw error
      const sanitizedError = this.sanitizeErrorMessage(error, 'Network request failed')
      throw new Error(sanitizedError)
    }
  }

  // SECURITY: Content Security Policy violation handler
  handleCSPViolation(violationEvent: SecurityPolicyViolationEvent): void {
    console.warn('SECURITY: CSP Violation detected:', {
      directive: violationEvent.violatedDirective,
      blockedURI: this.sanitizeErrorMessage(violationEvent.blockedURI),
      documentURI: this.sanitizeErrorMessage(violationEvent.documentURI),
      lineNumber: '[REDACTED]', // Don't log line numbers for security
      sourceFile: '[REDACTED]' // Don't log source files for security
    })

    // In production, you might want to report CSP violations to a monitoring service
    // this.reportSecurityViolation('csp', violationEvent)
  }

  // SECURITY: Initialize security event listeners
  initializeSecurityListeners(): void {
    // CSP violation monitoring
    document.addEventListener('securitypolicyviolation', this.handleCSPViolation.bind(this))

    // Enforce HTTPS on page load
    this.enforceHTTPS()

    // Monitor for mixed content warnings
    if (this.isHTTPS()) {
      console.info('SECURITY: Secure HTTPS connection established')
    }
  }

  // SECURITY: Remove security listeners
  removeSecurityListeners(): void {
    document.removeEventListener('securitypolicyviolation', this.handleCSPViolation.bind(this))
  }

  // SECURITY: Get security status
  getSecurityStatus(): {
    isHTTPS: boolean
    shouldEnforceHTTPS: boolean
    notificationPermission: string
    cspSupported: boolean
  } {
    return {
      isHTTPS: this.isHTTPS(),
      shouldEnforceHTTPS: this.shouldEnforceHTTPS(),
      notificationPermission: 'Notification' in window ? Notification.permission : 'unsupported',
      cspSupported: 'SecurityPolicyViolationEvent' in window
    }
  }

  // SECURITY: Check if current environment is secure
  isSecureEnvironment(): boolean {
    if (!this.shouldEnforceHTTPS()) {
      return true // Localhost development is considered secure
    }
    
    return this.isHTTPS()
  }

  // SECURITY: Generate Content Security Policy directive
  generateCSPDirective(): string {
    const isProduction = process.env.NODE_ENV === 'production'
    
    const directives = [
      "default-src 'self'",
      "script-src 'self' 'unsafe-inline' 'unsafe-eval'", // Note: In production, remove unsafe-*
      "style-src 'self' 'unsafe-inline' https://fonts.googleapis.com",
      "font-src 'self' https://fonts.gstatic.com",
      "img-src 'self' data: https:",
      "connect-src 'self' https: wss:",
      "frame-src 'none'",
      "object-src 'none'",
      "base-uri 'self'",
      "form-action 'self'"
    ]

    if (isProduction) {
      // Stricter policy for production
      directives[1] = "script-src 'self'" // Remove unsafe-* in production
      directives.push("upgrade-insecure-requests")
      directives.push("block-all-mixed-content")
    }

    return directives.join('; ')
  }
}

// Export singleton instance
export const securityUtilsService = new SecurityUtilsService()
export default securityUtilsService