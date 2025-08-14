// Import BigInt polyfill FIRST to prevent conversion errors
import '../utils/bigint-polyfill'

import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react'
import { Principal } from '@dfinity/principal'
import { Identity } from '@dfinity/agent'
import { useNFIDAuth } from '../hooks/useNFIDAuth'
import multiChainWalletService from '../services/multiChainWalletService'
import localCacheService from '../services/localCacheService'

export type UserMode = 'guest' | 'authenticated'

export interface AuthContextValue {
  // User mode
  userMode: UserMode
  
  // NFID Authentication state
  isAuthenticated: boolean
  principal: Principal | null
  identity: Identity | null
  isLoading: boolean
  error: string | null

  // Actions
  login: () => Promise<boolean>
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
  const nfidAuth = useNFIDAuth()

  // Initialize user mode based on authentication state
  useEffect(() => {
    if (nfidAuth.isAuthenticated) {
      setUserMode('authenticated')
    } else {
      // Check if user previously chose guest mode
      const guestModePreference = localStorage.getItem('deflow_guest_mode')
      setUserMode(guestModePreference === 'true' ? 'guest' : 'guest')
    }
  }, [nfidAuth.isAuthenticated])

  // Enhanced login with data migration
  const login = async (): Promise<boolean> => {
    const success = await nfidAuth.login()
    
    if (success) {
      setUserMode('authenticated')
      
      // Welcome notification
      localCacheService.addNotification({
        id: `login_welcome_${Date.now()}`,
        title: 'Welcome to DeFlow!',
        message: 'You now have access to premium features including cross-device sync and reduced fees.',
        type: 'success',
        createdAt: Date.now(),
        read: false
      })
      
      // Clear guest mode preference
      localStorage.removeItem('deflow_guest_mode')
      
      // Here you could sync local data to cloud storage
      // syncLocalDataToCloud()
    }
    
    return success
  }

  // Enhanced logout with data preservation
  const logout = async (): Promise<void> => {
    await nfidAuth.logout()
    setUserMode('guest')
    
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

  const contextValue: AuthContextValue = {
    // User mode
    userMode,
    
    // NFID Authentication state
    isAuthenticated: nfidAuth.isAuthenticated,
    principal: nfidAuth.principal,
    identity: nfidAuth.identity,
    isLoading: nfidAuth.isLoading,
    error: nfidAuth.error,
    
    // Actions
    login,
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