// SECURITY: Security initialization module
import securityUtilsService from '../services/securityUtilsService'
import secureStorageService from '../services/secureStorageService'

export class SecurityInitializer {
  private static initialized = false

  // SECURITY: Initialize all security features
  static initialize(): void {
    if (this.initialized) {
      console.warn('SECURITY: Security already initialized')
      return
    }

    try {
      console.info('SECURITY: Initializing security features...')

      // 1. Initialize security event listeners
      securityUtilsService.initializeSecurityListeners()

      // 2. Check environment security
      const securityStatus = securityUtilsService.getSecurityStatus()
      console.info('SECURITY: Environment status:', securityStatus)

      // 3. Warn about insecure environment
      if (!securityUtilsService.isSecureEnvironment()) {
        console.warn('SECURITY: Running in insecure environment. HTTPS recommended for production.')
      }

      // 4. Initialize crypto support check
      if (!this.isCryptoSupported()) {
        console.error('SECURITY: Browser does not support required crypto features')
        this.showCryptoUnsupportedWarning()
      }

      // 5. Set security headers for fetch requests
      this.setupGlobalFetchInterceptor()

      // 6. Set up error handling
      this.setupGlobalErrorHandler()

      this.initialized = true
      console.info('SECURITY: Security initialization completed')
      
    } catch (error) {
      console.error('SECURITY: Failed to initialize security features:', error)
      // Don't block app initialization, but warn user
      this.showSecurityInitializationError()
    }
  }

  // SECURITY: Cleanup security features
  static cleanup(): void {
    if (!this.initialized) {
      return
    }

    try {
      securityUtilsService.removeSecurityListeners()
      this.initialized = false
      console.info('SECURITY: Security cleanup completed')
    } catch (error) {
      console.error('SECURITY: Error during security cleanup:', error)
    }
  }

  // SECURITY: Global fetch interceptor for security headers
  private static setupGlobalFetchInterceptor(): void {
    const originalFetch = window.fetch
    
    window.fetch = async (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
      // Use security utils for external requests
      if (typeof input === 'string' && (input.startsWith('http://') || input.startsWith('https://'))) {
        return securityUtilsService.secureFetch(input, init)
      }
      
      // For relative URLs, add security headers
      const secureInit: RequestInit = {
        ...init,
        headers: {
          ...init?.headers,
          'X-Requested-With': 'XMLHttpRequest'
        }
      }
      
      return originalFetch(input, secureInit)
    }
  }

  // SECURITY: Global error handler for security events
  private static setupGlobalErrorHandler(): void {
    window.addEventListener('error', (event) => {
      // Sanitize error messages in logs
      const sanitizedMessage = securityUtilsService.sanitizeErrorMessage(event.error)
      console.error('SECURITY: Global error caught:', sanitizedMessage)
    })

    window.addEventListener('unhandledrejection', (event) => {
      // Sanitize promise rejection messages
      const sanitizedMessage = securityUtilsService.sanitizeErrorMessage(event.reason)
      console.error('SECURITY: Unhandled promise rejection:', sanitizedMessage)
    })
  }

  // SECURITY: Show crypto unsupported warning
  private static showCryptoUnsupportedWarning(): void {
    const message = 'Your browser does not support required security features. Please use a modern browser for full functionality.'
    
    // Show non-blocking notification
    if ('Notification' in window && Notification.permission === 'granted') {
      new Notification('Security Warning', {
        body: message,
        icon: '/favicon.ico'
      })
    } else {
      // Fallback to console warning
      console.warn('SECURITY:', message)
    }
  }

  // SECURITY: Show security initialization error
  private static showSecurityInitializationError(): void {
    const message = 'Some security features failed to initialize. The application may be less secure.'
    console.warn('SECURITY:', message)
  }

  // SECURITY: Check if crypto is supported
  static isCryptoSupported(): boolean {
    return !!(crypto && crypto.subtle && crypto.getRandomValues)
  }

  // SECURITY: Get security status
  static getSecurityStatus(): {
    initialized: boolean
    isSecure: boolean
    features: {
      https: boolean
      csp: boolean
      crypto: boolean
      notifications: string
    }
  } {
    const securityStatus = securityUtilsService.getSecurityStatus()
    
    return {
      initialized: this.initialized,
      isSecure: securityUtilsService.isSecureEnvironment(),
      features: {
        https: securityStatus.isHTTPS,
        csp: securityStatus.cspSupported,
        crypto: SecurityInitializer.isCryptoSupported(),
        notifications: securityStatus.notificationPermission
      }
    }
  }

  // SECURITY: Validate security configuration
  static validateSecurityConfiguration(): {
    isValid: boolean
    warnings: string[]
    errors: string[]
  } {
    const warnings: string[] = []
    const errors: string[] = []
    const status = this.getSecurityStatus()

    if (!status.isSecure) {
      warnings.push('Application is not running over HTTPS in production environment')
    }

    if (!status.features.crypto) {
      errors.push('Browser does not support required cryptographic features')
    }

    if (!status.features.csp) {
      warnings.push('Browser does not support Content Security Policy')
    }

    if (status.features.notifications === 'denied') {
      warnings.push('Browser notifications are disabled')
    }

    return {
      isValid: errors.length === 0,
      warnings,
      errors
    }
  }
}

// Auto-initialize security when module is imported
SecurityInitializer.initialize()

export default SecurityInitializer