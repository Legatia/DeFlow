// Error handling utilities for DeFi protocol interactions
export interface ProtocolError {
  type: 'protocol' | 'network' | 'validation' | 'execution' | 'timeout';
  protocol?: string;
  message: string;
  code?: number;
  details?: any;
  retryable: boolean;
  suggestedAction?: string;
}

export class DeFiErrorHandler {
  private static readonly MAX_RETRIES = 3;
  private static readonly RETRY_DELAY = 1000; // 1 second

  /**
   * Handle protocol-specific errors and provide user-friendly messages
   */
  static handleProtocolError(error: any, protocol?: string): ProtocolError {
    // Network connectivity errors
    if (error.message?.includes('Failed to fetch') || error.message?.includes('Network Error')) {
      return {
        type: 'network',
        protocol,
        message: 'Network connectivity issue. Please check your connection and try again.',
        retryable: true,
        suggestedAction: 'Retry in a few moments'
      };
    }

    // Rate limiting errors
    if (error.message?.includes('Rate Limited') || error.message?.includes('Too Many Requests')) {
      return {
        type: 'network',
        protocol,
        message: 'Too many requests. Please wait before trying again.',
        retryable: true,
        suggestedAction: 'Wait 1-2 minutes before retrying'
      };
    }

    // Protocol-specific errors
    if (protocol) {
      switch (protocol.toLowerCase()) {
        case 'aave':
          return this.handleAaveError(error);
        case 'uniswap':
        case 'uniswap v3':
          return this.handleUniswapError(error);
        case 'compound':
          return this.handleCompoundError(error);
        case 'curve':
          return this.handleCurveError(error);
      }
    }

    // Execution errors
    if (error.message?.includes('execution failed') || error.message?.includes('transaction failed')) {
      return {
        type: 'execution',
        protocol,
        message: 'Transaction execution failed. This could be due to insufficient liquidity or gas issues.',
        retryable: true,
        suggestedAction: 'Check gas settings and liquidity availability'
      };
    }

    // Validation errors
    if (error.message?.includes('validation') || error.message?.includes('invalid')) {
      return {
        type: 'validation',
        protocol,
        message: 'Invalid configuration or insufficient funds.',
        retryable: false,
        suggestedAction: 'Check your strategy configuration and wallet balance'
      };
    }

    // Timeout errors
    if (error.message?.includes('timeout') || error.message?.includes('Request timeout')) {
      return {
        type: 'timeout',
        protocol,
        message: 'Request timed out. The network may be congested.',
        retryable: true,
        suggestedAction: 'Try again with higher gas price'
      };
    }

    // Generic error
    return {
      type: 'protocol',
      protocol,
      message: error.message || 'An unexpected error occurred with the DeFi protocol.',
      retryable: true,
      suggestedAction: 'Contact support if this persists'
    };
  }

  private static handleAaveError(error: any): ProtocolError {
    if (error.message?.includes('insufficient liquidity')) {
      return {
        type: 'execution',
        protocol: 'Aave',
        message: 'Insufficient liquidity available in Aave pool.',
        retryable: true,
        suggestedAction: 'Try a smaller amount or wait for liquidity to improve'
      };
    }

    if (error.message?.includes('health factor')) {
      return {
        type: 'validation',
        protocol: 'Aave',
        message: 'Health factor too low for this operation.',
        retryable: false,
        suggestedAction: 'Improve your health factor before proceeding'
      };
    }

    return {
      type: 'protocol',
      protocol: 'Aave',
      message: error.message || 'Aave protocol error occurred.',
      retryable: true,
      suggestedAction: 'Check Aave protocol status'
    };
  }

  private static handleUniswapError(error: any): ProtocolError {
    if (error.message?.includes('slippage')) {
      return {
        type: 'execution',
        protocol: 'Uniswap',
        message: 'Transaction failed due to price slippage.',
        retryable: true,
        suggestedAction: 'Increase slippage tolerance or try smaller amount'
      };
    }

    if (error.message?.includes('insufficient')) {
      return {
        type: 'validation',
        protocol: 'Uniswap',
        message: 'Insufficient token balance for this swap.',
        retryable: false,
        suggestedAction: 'Check your token balance'
      };
    }

    return {
      type: 'protocol',
      protocol: 'Uniswap',
      message: error.message || 'Uniswap protocol error occurred.',
      retryable: true,
      suggestedAction: 'Check Uniswap protocol status'
    };
  }

  private static handleCompoundError(error: any): ProtocolError {
    if (error.message?.includes('market not listed')) {
      return {
        type: 'validation',
        protocol: 'Compound',
        message: 'Token not supported on Compound.',
        retryable: false,
        suggestedAction: 'Choose a different token or protocol'
      };
    }

    return {
      type: 'protocol',
      protocol: 'Compound',
      message: error.message || 'Compound protocol error occurred.',
      retryable: true,
      suggestedAction: 'Check Compound protocol status'
    };
  }

  private static handleCurveError(error: any): ProtocolError {
    if (error.message?.includes('pool')) {
      return {
        type: 'execution',
        protocol: 'Curve',
        message: 'Curve pool error. Pool may be imbalanced.',
        retryable: true,
        suggestedAction: 'Try a different pool or smaller amount'
      };
    }

    return {
      type: 'protocol',
      protocol: 'Curve',
      message: error.message || 'Curve protocol error occurred.',
      retryable: true,
      suggestedAction: 'Check Curve protocol status'
    };
  }

  /**
   * Retry a failed operation with exponential backoff
   */
  static async retryOperation<T>(
    operation: () => Promise<T>,
    maxRetries: number = this.MAX_RETRIES,
    baseDelay: number = this.RETRY_DELAY
  ): Promise<T> {
    let lastError: any;

    for (let attempt = 0; attempt <= maxRetries; attempt++) {
      try {
        return await operation();
      } catch (error) {
        lastError = error;
        
        // Don't retry if it's not retryable
        const protocolError = this.handleProtocolError(error);
        if (!protocolError.retryable || attempt === maxRetries) {
          throw error;
        }

        // Exponential backoff
        const delay = baseDelay * Math.pow(2, attempt);
        await new Promise(resolve => setTimeout(resolve, delay));
        
        console.warn(`Retry attempt ${attempt + 1}/${maxRetries} after ${delay}ms delay`);
      }
    }

    throw lastError;
  }

  /**
   * Create a user-friendly error message for display
   */
  static formatErrorForUser(error: ProtocolError): { title: string; message: string; action?: string } {
    const protocolName = error.protocol ? ` (${error.protocol})` : '';
    
    switch (error.type) {
      case 'network':
        return {
          title: `Network Issue${protocolName}`,
          message: error.message,
          action: error.suggestedAction
        };
      
      case 'validation':
        return {
          title: `Configuration Error${protocolName}`,
          message: error.message,
          action: error.suggestedAction
        };
      
      case 'execution':
        return {
          title: `Execution Failed${protocolName}`,
          message: error.message,
          action: error.suggestedAction
        };
      
      case 'timeout':
        return {
          title: `Request Timeout${protocolName}`,
          message: error.message,
          action: error.suggestedAction
        };
      
      default:
        return {
          title: `Protocol Error${protocolName}`,
          message: error.message,
          action: error.suggestedAction
        };
    }
  }

  /**
   * Log errors for debugging and monitoring
   */
  static logError(error: ProtocolError, context?: string): void {
    const logData = {
      timestamp: new Date().toISOString(),
      context: context || 'unknown',
      type: error.type,
      protocol: error.protocol,
      message: error.message,
      code: error.code,
      details: error.details,
      retryable: error.retryable
    };

    console.error('DeFi Protocol Error:', logData);
    
    // In production, you might want to send this to a monitoring service
    // Example: sendToMonitoring(logData);
  }
}

/**
 * Circuit breaker for protocol operations
 */
export class ProtocolCircuitBreaker {
  private static readonly FAILURE_THRESHOLD = 5;
  private static readonly RECOVERY_TIMEOUT = 60000; // 1 minute
  private static protocolStates: Map<string, {
    failures: number;
    lastFailure: number;
    isOpen: boolean;
  }> = new Map();

  static async executeWithCircuitBreaker<T>(
    protocol: string,
    operation: () => Promise<T>
  ): Promise<T> {
    const state = this.getProtocolState(protocol);

    // Check if circuit is open
    if (state.isOpen) {
      const timeSinceLastFailure = Date.now() - state.lastFailure;
      if (timeSinceLastFailure < this.RECOVERY_TIMEOUT) {
        throw new Error(`Circuit breaker is open for ${protocol}. Try again later.`);
      } else {
        // Try to close the circuit
        state.isOpen = false;
        state.failures = 0;
      }
    }

    try {
      const result = await operation();
      // Success - reset failure count
      state.failures = 0;
      return result;
    } catch (error) {
      // Failure - increment count
      state.failures++;
      state.lastFailure = Date.now();

      // Open circuit if threshold reached
      if (state.failures >= this.FAILURE_THRESHOLD) {
        state.isOpen = true;
        console.warn(`Circuit breaker opened for ${protocol} after ${state.failures} failures`);
      }

      throw error;
    }
  }

  private static getProtocolState(protocol: string) {
    if (!this.protocolStates.has(protocol)) {
      this.protocolStates.set(protocol, {
        failures: 0,
        lastFailure: 0,
        isOpen: false
      });
    }
    return this.protocolStates.get(protocol)!;
  }

  static getProtocolStatus(protocol: string): { failures: number; isOpen: boolean; canRetry: boolean } {
    const state = this.getProtocolState(protocol);
    const canRetry = !state.isOpen || (Date.now() - state.lastFailure) >= this.RECOVERY_TIMEOUT;
    
    return {
      failures: state.failures,
      isOpen: state.isOpen,
      canRetry
    };
  }
}

export default DeFiErrorHandler;