// Import BigInt polyfill FIRST to prevent conversion errors
import '../utils/bigint-polyfill'

import { useState, useEffect } from 'react'
import nfidAuthService, { NFIDAuthState } from '../services/nfidAuthService'

export function useNFIDAuth() {
  const [authState, setAuthState] = useState<NFIDAuthState>(nfidAuthService.getState())

  useEffect(() => {
    // Subscribe to auth state changes
    const unsubscribe = nfidAuthService.subscribe((state) => {
      setAuthState(state)
    })

    return unsubscribe
  }, [])

  // Login function
  const login = async (): Promise<boolean> => {
    return await nfidAuthService.authenticate()
  }

  // Logout function
  const logout = async (): Promise<void> => {
    await nfidAuthService.logout()
  }

  return {
    ...authState,
    login,
    logout
  }
}