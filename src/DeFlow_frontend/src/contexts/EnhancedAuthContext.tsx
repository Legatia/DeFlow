// Import BigInt polyfill FIRST to prevent conversion errors
import '../utils/bigint-polyfill'

import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react'
import { Principal } from '@dfinity/principal'
import { Identity } from '@dfinity/agent'
import { useNFIDAuth } from '../hooks/useNFIDAuth'
import internetIdentityService from '../services/internetIdentityService'
import multiChainWalletService from '../services/multiChainWalletService'
import localCacheService from '../services/localCacheService'

export type UserMode = 'guest' | 'authenticated'

export type SubscriptionTier = 'standard' | 'premium' | 'pro'

export interface AuthContextValue {
  // User mode
  userMode: UserMode
  
  // Authentication state (unified for both NFID and II)
  isAuthenticated: boolean
  principal: Principal | null
  identity: Identity | null
  isLoading: boolean
  error: string | null
  authMethod: 'nfid' | 'internet-identity' | null

  // Subscription tier
  subscriptionTier: SubscriptionTier
  updateSubscriptionTier: (tier: SubscriptionTier) => void

  // Actions
  loginWithNFID: () => Promise<boolean>
  loginWithInternetIdentity: () => Promise<boolean>
  logout: () => Promise<void>
  
  // Guest mode functions
  switchToGuestMode: () => void
  
  // Subscription benefits
  subscriptionBenefits: {
    unlimitedWorkflows: boolean
    priorityExecution: boolean
    crossDeviceSync: boolean
    advancedAnalytics: boolean
    reducedFees: boolean
  }
  
  // Cache management
  exportLocalData: () => string
  clearLocalData: () => boolean
  getCacheSize: () => string
}

const EnhancedAuthContext = createContext<AuthContextValue | null>(null)

export const useEnhancedAuth = () => {
  const context = useContext(EnhancedAuthContext)
  if (!context) {
    throw new Error('useEnhancedAuth must be used within an EnhancedAuthProvider')
  }
  return context
}

interface EnhancedAuthProviderProps {
  children: ReactNode
}

export const EnhancedAuthProvider = ({ children }: EnhancedAuthProviderProps) => {
  const [userMode, setUserMode] = useState<UserMode>('guest')
  const [authMethod, setAuthMethod] = useState<'nfid' | 'internet-identity' | null>(null)
  const [subscriptionTier, setSubscriptionTier] = useState<SubscriptionTier>('standard')
  const [iiAuth, setIIAuth] = useState({
    isAuthenticated: false,
    principal: null as Principal | null,
    identity: null as Identity | null,
    isLoading: false,
    error: null as string | null
  })
  const nfidAuth = useNFIDAuth()

  // Load subscription tier from localStorage
  useEffect(() => {
    const savedTier = localStorage.getItem('deflow_subscription_tier') as SubscriptionTier
    if (savedTier && ['standard', 'premium', 'pro'].includes(savedTier)) {
      setSubscriptionTier(savedTier)
    }
  }, [])

  const updateSubscriptionTier = (tier: SubscriptionTier) => {
    setSubscriptionTier(tier)
    localStorage.setItem('deflow_subscription_tier', tier)
  }

  // Initialize Internet Identity listener
  useEffect(() => {
    const unsubscribe = internetIdentityService.subscribe((state) => {
      setIIAuth({
        isAuthenticated: state.isAuthenticated,
        principal: state.principal,
        identity: state.identity,
        isLoading: state.isLoading,
        error: state.error
      })
    })
    return unsubscribe
  }, [])

  // Initialize user mode based on authentication state
  useEffect(() => {
    const isAuthenticated = nfidAuth.isAuthenticated || iiAuth.isAuthenticated
    
    if (isAuthenticated) {
      setUserMode('authenticated')
      // Set auth method based on which one is authenticated
      if (nfidAuth.isAuthenticated) {
        setAuthMethod('nfid')
      } else if (iiAuth.isAuthenticated) {
        setAuthMethod('internet-identity')
      }
    } else {
      // Check if user previously chose guest mode
      const guestModePreference = localStorage.getItem('deflow_guest_mode')
      setUserMode(guestModePreference === 'true' ? 'guest' : 'guest')
      setAuthMethod(null)
    }
  }, [nfidAuth.isAuthenticated, iiAuth.isAuthenticated])

  // NFID login
  const loginWithNFID = async (): Promise<boolean> => {
    const success = await nfidAuth.login()
    
    if (success) {
      setUserMode('authenticated')
      setAuthMethod('nfid')
      
      // Welcome notification
      localCacheService.addNotification({
        id: `nfid_login_welcome_${Date.now()}`,
        title: 'Welcome to DeFlow Premium!',
        message: 'Logged in with Google via NFID. You now have access to premium features.',
        type: 'success',
        createdAt: Date.now(),
        read: false
      })
      
      // Clear guest mode preference
      localStorage.removeItem('deflow_guest_mode')
    }
    
    return success
  }

  // Internet Identity login
  const loginWithInternetIdentity = async (): Promise<boolean> => {
    const success = await internetIdentityService.authenticate()
    
    if (success) {
      setUserMode('authenticated')
      setAuthMethod('internet-identity')
      
      // Welcome notification
      localCacheService.addNotification({
        id: `ii_login_welcome_${Date.now()}`,
        title: 'Welcome to DeFlow Premium!',
        message: 'Logged in with Internet Identity. You now have access to premium features.',
        type: 'success',
        createdAt: Date.now(),
        read: false
      })
      
      // Clear guest mode preference
      localStorage.removeItem('deflow_guest_mode')
    }
    
    return success
  }

  // Enhanced logout with data preservation
  const logout = async (): Promise<void> => {
    // Logout from both services
    await nfidAuth.logout()
    await internetIdentityService.logout()
    
    setUserMode('guest')
    setAuthMethod(null)
    
    // Set guest mode preference
    localStorage.setItem('deflow_guest_mode', 'true')
    
    // Notification about guest mode
    localCacheService.addNotification({
      id: `logout_guest_${Date.now()}`,
      title: 'Switched to Guest Mode',
      message: 'Your data is saved locally. Login anytime to access premium features.',
      type: 'info',
      createdAt: Date.now(),
      read: false
    })
  }

  // Switch to guest mode explicitly
  const switchToGuestMode = () => {
    setUserMode('guest')
    localStorage.setItem('deflow_guest_mode', 'true')
    
    localCacheService.addNotification({
      id: `guest_mode_${Date.now()}`,
      title: 'Guest Mode Active',
      message: 'Using DeFlow in guest mode. Your data is saved locally.',
      type: 'info',
      createdAt: Date.now(),
      read: false
    })
  }

  // Subscription benefits based on user mode
  const subscriptionBenefits = {
    unlimitedWorkflows: userMode === 'authenticated',
    priorityExecution: userMode === 'authenticated',
    crossDeviceSync: userMode === 'authenticated',
    advancedAnalytics: userMode === 'authenticated',
    reducedFees: userMode === 'authenticated'
  }

  // Cache management functions
  const exportLocalData = (): string => {
    return localCacheService.exportData()
  }

  const clearLocalData = (): boolean => {
    return localCacheService.clearCache()
  }

  const getCacheSize = (): string => {
    return localCacheService.getCacheSize()
  }

  // Show initial welcome message for new users
  useEffect(() => {
    const hasShownWelcome = localStorage.getItem('deflow_welcome_shown')
    
    if (!hasShownWelcome) {
      setTimeout(() => {
        localCacheService.addNotification({
          id: `welcome_${Date.now()}`,
          title: 'Welcome to DeFlow!',
          message: 'Create automated workflows for DeFi and multi-chain operations. Your data is saved locally.',
          type: 'info',
          createdAt: Date.now(),
          read: false
        })
        
        localStorage.setItem('deflow_welcome_shown', 'true')
      }, 2000)
    }
  }, [])

  // Get unified authentication state
  const isAuthenticated = nfidAuth.isAuthenticated || iiAuth.isAuthenticated
  const principal = nfidAuth.isAuthenticated ? nfidAuth.principal : iiAuth.principal
  const identity = nfidAuth.isAuthenticated ? nfidAuth.identity : iiAuth.identity
  const isLoading = nfidAuth.isLoading || iiAuth.isLoading
  const error = nfidAuth.error || iiAuth.error

  const contextValue: AuthContextValue = {
    // User mode
    userMode,
    
    // Unified Authentication state
    isAuthenticated,
    principal,
    identity,
    isLoading,
    error,
    authMethod,
    
    // Subscription tier
    subscriptionTier,
    updateSubscriptionTier,
    
    // Actions
    loginWithNFID,
    loginWithInternetIdentity,
    logout,
    switchToGuestMode,
    
    // Subscription benefits
    subscriptionBenefits,
    
    // Cache management
    exportLocalData,
    clearLocalData,
    getCacheSize
  }

  return (
    <EnhancedAuthContext.Provider value={contextValue}>
      {children}
    </EnhancedAuthContext.Provider>
  )
}