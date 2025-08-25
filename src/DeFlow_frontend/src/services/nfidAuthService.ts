// Import BigInt polyfill FIRST to prevent conversion errors
import '../utils/bigint-polyfill'

import { AuthClient } from '@dfinity/auth-client'
import { Identity } from '@dfinity/agent'
import { Principal } from '@dfinity/principal'

// NFID URL configuration
const NFID_URL = {
  local: "https://nfid.one/authenticate/?applicationName=DeFlow",
  ic: "https://nfid.one/authenticate/?applicationName=DeFlow"
}

export interface NFIDAuthState {
  isAuthenticated: boolean
  principal: Principal | null
  identity: Identity | null
  isLoading: boolean
  error: string | null
}

class NFIDAuthService {
  private authClient: AuthClient | null = null
  private listeners: Array<(state: NFIDAuthState) => void> = []
  private currentState: NFIDAuthState = {
    isAuthenticated: false,
    principal: null,
    identity: null,
    isLoading: true,
    error: null
  }

  // Initialize auth client
  async initializeAuthClient(): Promise<AuthClient> {
    if (!this.authClient) {
      try {
        this.authClient = await AuthClient.create()
        
        // Check if user is already authenticated
        const isAuthenticated = await this.authClient.isAuthenticated()
        
        if (isAuthenticated) {
          const identity = this.authClient.getIdentity()
          const principal = identity.getPrincipal()
          
          if (!principal.isAnonymous()) {
            this.updateState({
              isAuthenticated: true,
              principal,
              identity,
              isLoading: false,
              error: null
            })
          } else {
            this.updateState({
              isAuthenticated: false,
              principal: null,
              identity: null,
              isLoading: false,
              error: null
            })
          }
        } else {
          this.updateState({
            isAuthenticated: false,
            principal: null,
            identity: null,
            isLoading: false,
            error: null
          })
        }
      } catch (error) {
        console.error('Failed to initialize auth client:', error)
        this.updateState({
          isAuthenticated: false,
          principal: null,
          identity: null,
          isLoading: false,
          error: error instanceof Error ? error.message : 'Failed to initialize auth client'
        })
      }
    }
    return this.authClient!
  }

  // Authenticate with NFID Google login
  async authenticate(): Promise<boolean> {
    
    try {
      const authClient = await this.initializeAuthClient()
      const isAuthenticated = await authClient.isAuthenticated()
      

      if (!isAuthenticated) {
        
        this.updateState({
          ...this.currentState,
          isLoading: true,
          error: null
        })

        // Use the IC environment URL (both local dev and production use the same NFID service)
        const identityProviderUrl = NFID_URL.ic

        await new Promise<void>((resolve, reject) => {
          authClient.login({
            identityProvider: identityProviderUrl,
            onSuccess: () => resolve(),
            onError: (error) => reject(new Error(error || 'Authentication failed')),
            windowOpenerFeatures: `
              left=${window.screen.width / 2 - 525 / 2},
              top=${window.screen.height / 2 - 705 / 2},
              toolbar=0,location=0,menubar=0,width=525,height=705
            `,
          })
        })
      }


      const identity = authClient.getIdentity()
      const principal = identity.getPrincipal()

      if (principal.isAnonymous()) {
        throw new Error('Authentication failed - anonymous principal')
      }

      this.updateState({
        isAuthenticated: true,
        principal,
        identity,
        isLoading: false,
        error: null
      })

      // Store authentication state
      localStorage.setItem('deflow_auth_state', 'authenticated')

      return true
    } catch (error) {
      console.error('Authentication failed:', error)
      
      this.updateState({
        isAuthenticated: false,
        principal: null,
        identity: null,
        isLoading: false,
        error: error instanceof Error ? error.message : 'Authentication failed'
      })

      return false
    }
  }

  // Logout
  async logout(): Promise<void> {
    try {
      if (this.authClient) {
        await this.authClient.logout()
      }
      
      this.updateState({
        isAuthenticated: false,
        principal: null,
        identity: null,
        isLoading: false,
        error: null
      })

      // Clear stored auth state
      localStorage.removeItem('deflow_auth_state')
      
    } catch (error) {
      console.error('Logout failed:', error)
      this.updateState({
        ...this.currentState,
        error: error instanceof Error ? error.message : 'Logout failed'
      })
    }
  }

  // Get current state
  getState(): NFIDAuthState {
    return { ...this.currentState }
  }

  // Subscribe to auth state changes
  subscribe(callback: (state: NFIDAuthState) => void): () => void {
    this.listeners.push(callback)
    
    // Immediately call with current state
    callback(this.currentState)
    
    // Return unsubscribe function
    return () => {
      this.listeners = this.listeners.filter(listener => listener !== callback)
    }
  }

  // Update state and notify listeners
  private updateState(newState: NFIDAuthState): void {
    this.currentState = newState
    this.listeners.forEach(listener => listener(newState))
  }

  // Initialize on first import
  async initialize(): Promise<void> {
    await this.initializeAuthClient()
  }
}

// Create singleton instance
export const nfidAuthService = new NFIDAuthService()

// Initialize immediately
nfidAuthService.initialize()

export default nfidAuthService