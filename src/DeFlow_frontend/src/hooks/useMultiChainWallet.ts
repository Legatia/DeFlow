// React hook for multi-chain wallet management
import { useState, useEffect } from 'react'
import multiChainWalletService, { MultiChainWallet, ChainType, WalletType } from '../services/multiChainWalletService'

export interface UseMultiChainWalletReturn {
  wallet: MultiChainWallet
  isConnected: boolean
  connectedChains: ChainType[]
  connectWallet: (chain: ChainType, walletType: WalletType) => Promise<string>
  disconnectWallet: (chain: ChainType) => Promise<void>
  addManualAddress: (chain: ChainType, address: string) => Promise<void>
  removeWalletAddress: (chain: ChainType) => Promise<void>
  updateBalance: (chain: ChainType) => Promise<void>
  updateAllBalances: () => Promise<void>
  getAddressForChain: (chain: ChainType) => string | undefined
  hasChain: (chain: ChainType) => boolean
  totalConnectedChains: number
  isLoading: boolean
  error: string | null
}

export const useMultiChainWallet = (): UseMultiChainWalletReturn => {
  const [wallet, setWallet] = useState<MultiChainWallet>(multiChainWalletService.getWallet())
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const handleWalletUpdate = (updatedWallet: MultiChainWallet) => {
      setWallet(updatedWallet)
    }

    multiChainWalletService.addListener(handleWalletUpdate)

    // Initial load
    multiChainWalletService.updateAllBalances().catch(err => {
      console.error('Failed to load initial balances:', err)
    })

    return () => {
      multiChainWalletService.removeListener(handleWalletUpdate)
    }
  }, [])

  const connectWallet = async (chain: ChainType, walletType: WalletType): Promise<string> => {
    setIsLoading(true)
    setError(null)
    
    try {
      const address = await multiChainWalletService.connectWallet(chain, walletType)
      return address
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to connect wallet'
      setError(errorMessage)
      throw err
    } finally {
      setIsLoading(false)
    }
  }

  const disconnectWallet = async (chain: ChainType): Promise<void> => {
    setIsLoading(true)
    setError(null)
    
    try {
      await multiChainWalletService.disconnectWallet(chain)
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to disconnect wallet'
      setError(errorMessage)
      throw err
    } finally {
      setIsLoading(false)
    }
  }

  const addManualAddress = async (chain: ChainType, address: string): Promise<void> => {
    setIsLoading(true)
    setError(null)
    
    try {
      await multiChainWalletService.addManualAddress(chain, address)
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to add address'
      setError(errorMessage)
      throw err
    } finally {
      setIsLoading(false)
    }
  }

  const removeWalletAddress = async (chain: ChainType): Promise<void> => {
    setIsLoading(true)
    setError(null)
    
    try {
      await multiChainWalletService.removeWalletAddress(chain)
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to remove address'
      setError(errorMessage)
      throw err
    } finally {
      setIsLoading(false)
    }
  }

  const updateBalance = async (chain: ChainType): Promise<void> => {
    try {
      await multiChainWalletService.updateBalance(chain)
    } catch (err) {
      console.error(`Failed to update balance for ${chain}:`, err)
    }
  }

  const updateAllBalances = async (): Promise<void> => {
    try {
      await multiChainWalletService.updateAllBalances()
    } catch (err) {
      console.error('Failed to update all balances:', err)
    }
  }

  const getAddressForChain = (chain: ChainType): string | undefined => {
    const address = multiChainWalletService.getAddressForChain(chain)
    return address?.address
  }

  const hasChain = (chain: ChainType): boolean => {
    return wallet.addresses.some(addr => addr.chain === chain)
  }

  const connectedChains = multiChainWalletService.getConnectedChains()
  const isConnected = wallet.addresses.length > 0
  const totalConnectedChains = wallet.addresses.length

  return {
    wallet,
    isConnected,
    connectedChains,
    connectWallet,
    disconnectWallet,
    addManualAddress,
    removeWalletAddress,
    updateBalance,
    updateAllBalances,
    getAddressForChain,
    hasChain,
    totalConnectedChains,
    isLoading,
    error
  }
}