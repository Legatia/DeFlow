// Import BigInt polyfill FIRST to prevent conversion errors
import '../utils/bigint-polyfill'

import { AuthClient } from '@dfinity/auth-client'
import { Identity } from '@dfinity/agent'
import { Principal } from '@dfinity/principal'

export interface InternetIdentityAuthState {
  isAuthenticated: boolean
  principal: Principal | null
  identity: Identity | null
  isLoading: boolean
  error: string | null
}

// Internet Identity Configuration
const II_CONFIG = {
  // Internet Identity canister ID
  identityProvider: process.env.NODE_ENV === 'development' 
    ? 'http://localhost:4943/?canisterId=be2us-64aaa-aaaaa-qaabq-cai' // Local II
    : 'https://identity.ic0.app', // Mainnet II
  
  // Maximum delegation expiry (8 hours in nanoseconds)
  maxTimeToLive: BigInt(8 * 60 * 60 * 1000 * 1000 * 1000),
  
  // Window features for II popup
  windowOpenerFeatures: 'toolbar=0,location=0,menubar=0,width=500,height=500,left=100,top=100',
  
  // Create identity options
  createOptions: {
    idleOptions: {
      // Idle timeout (30 minutes in milliseconds)
      idleTimeout: 30 * 60 * 1000,
      disableDefaultIdleCallback: true,
    },
  },
}

class InternetIdentityService {
  private authClient: AuthClient | null = null
  private listeners: Array<(state: InternetIdentityAuthState) => void> = []
  private currentState: InternetIdentityAuthState = {
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
        this.authClient = await AuthClient.create(II_CONFIG.createOptions)
        
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
        console.error('Failed to initialize II auth client:', error)
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

  // Authenticate with Internet Identity
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

        await new Promise<void>((resolve, reject) => {
          authClient.login({
            identityProvider: II_CONFIG.identityProvider,
            maxTimeToLive: II_CONFIG.maxTimeToLive,
            windowOpenerFeatures: II_CONFIG.windowOpenerFeatures,
            onSuccess: () => resolve(),
            onError: (error) => reject(new Error(error || 'Authentication failed')),
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
      localStorage.setItem('deflow_ii_auth_state', 'authenticated')

      return true
    } catch (error) {
      console.error('Internet Identity authentication failed:', error)
      
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
      localStorage.removeItem('deflow_ii_auth_state')
      
    } catch (error) {
      console.error('Internet Identity logout failed:', error)
      this.updateState({
        ...this.currentState,
        error: error instanceof Error ? error.message : 'Logout failed'
      })
    }
  }

  // Get current state
  getState(): InternetIdentityAuthState {
    return { ...this.currentState }
  }

  // Subscribe to auth state changes
  subscribe(callback: (state: InternetIdentityAuthState) => void): () => void {
    this.listeners.push(callback)
    
    // Immediately call with current state
    callback(this.currentState)
    
    // Return unsubscribe function
    return () => {
      this.listeners = this.listeners.filter(listener => listener !== callback)
    }
  }

  // Update state and notify listeners
  private updateState(newState: InternetIdentityAuthState): void {
    this.currentState = newState
    this.listeners.forEach(listener => listener(newState))
  }

  // Initialize on first import
  async initialize(): Promise<void> {
    await this.initializeAuthClient()
  }
}

// Create singleton instance
export const internetIdentityService = new InternetIdentityService()

// Initialize immediately
internetIdentityService.initialize()

export default internetIdentityService