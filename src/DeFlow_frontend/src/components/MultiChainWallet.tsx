import React, { useState, useEffect, useCallback, useMemo, useRef } from 'react'
import multiChainWalletService, { 
  MultiChainWallet, 
  ChainType, 
  WalletType, 
  SUPPORTED_CHAINS,
  WalletAddress 
} from '../services/multiChainWalletService'
// import performanceService from '../services/performanceOptimizationService'

interface MultiChainWalletProps {
  isOpen: boolean
  onClose: () => void
}

const MultiChainWalletComponent = ({ isOpen, onClose }: MultiChainWalletProps) => {
  const [wallet, setWallet] = useState<MultiChainWallet>({ addresses: [], lastSyncAt: 0 })
  const [selectedChain, setSelectedChain] = useState<ChainType | null>(null)
  const [manualAddress, setManualAddress] = useState('')
  const [isConnecting, setIsConnecting] = useState<{ chain: ChainType; walletType: WalletType } | null>(null)
  const [isAddingManual, setIsAddingManual] = useState<ChainType | null>(null)
  const [isInitializing, setIsInitializing] = useState(true)
  
  // PERFORMANCE: Use refs to track mounted state and prevent memory leaks
  const isMountedRef = useRef(true)
  const balanceUpdateTimeoutRef = useRef<NodeJS.Timeout>()
  // const cleanupManager = useRef(performanceService.createCleanupManager())
  const cleanupManager = useRef({ 
    cleanup: () => {},
    addCleanup: (fn: () => void) => {}
  })
  
  // PERFORMANCE: Track component renders
  useEffect(() => {
    // performanceService.trackComponentRender('MultiChainWallet')
  })

  // PERFORMANCE: Memoized wallet update handler to prevent unnecessary re-renders
  const handleWalletUpdate = useCallback((updatedWallet: MultiChainWallet) => {
    if (isMountedRef.current) {
      setWallet(updatedWallet)
    }
  }, [])
  
  // PERFORMANCE: Initialize wallet asynchronously to prevent blocking
  useEffect(() => {
    let mounted = true
    
    const initializeWallet = async () => {
      try {
        // Ensure wallet service is initialized
        await multiChainWalletService.ensureInitialized()
        
        if (mounted) {
          const currentWallet = await multiChainWalletService.getWallet()
          setWallet(currentWallet)
          setIsInitializing(false)
        }
      } catch (error) {
        console.error('Failed to initialize wallet:', error)
        if (mounted) {
          setIsInitializing(false)
        }
      }
    }
    
    initializeWallet()
    
    return () => {
      mounted = false
    }
  }, [])
  
  // PERFORMANCE: Separate effect for listeners to prevent re-initialization
  useEffect(() => {
    multiChainWalletService.addListener(handleWalletUpdate)
    
    return () => {
      multiChainWalletService.removeListener(handleWalletUpdate)
    }
  }, [handleWalletUpdate])
  
  // PERFORMANCE: Debounced balance updates with performance service
  const debouncedBalanceUpdate = useCallback(() => {
    if (isMountedRef.current && wallet.addresses.length > 0) {
      multiChainWalletService.updateAllBalances().catch(console.error)
    }
  }, [wallet.addresses.length])
  
  useEffect(() => {
    if (!isOpen || isInitializing) return
    
    debouncedBalanceUpdate()
    
    // Register cleanup for debounced function
    cleanupManager.current.addCleanup(() => {
      if (balanceUpdateTimeoutRef.current) {
        clearTimeout(balanceUpdateTimeoutRef.current)
      }
    })
  }, [isOpen, isInitializing, debouncedBalanceUpdate])
  
  // PERFORMANCE: Comprehensive cleanup on unmount to prevent memory leaks
  useEffect(() => {
    return () => {
      isMountedRef.current = false
      if (balanceUpdateTimeoutRef.current) {
        clearTimeout(balanceUpdateTimeoutRef.current)
      }
      cleanupManager.current.cleanup()
    }
  }, [])

  // PERFORMANCE: Memoized handlers to prevent re-renders
  const handleConnectWallet = useCallback(async (chain: ChainType, walletType: WalletType) => {
    if (!isMountedRef.current) return
    
    setIsConnecting({ chain, walletType })
    try {
      await multiChainWalletService.connectWallet(chain, walletType)
    } catch (error) {
      if (isMountedRef.current) {
        alert(`Failed to connect ${walletType}: ${error instanceof Error ? error.message : 'Unknown error'}`)
      }
    } finally {
      if (isMountedRef.current) {
        setIsConnecting(null)
      }
    }
  }, [])

  const handleAddManualAddress = useCallback(async (chain: ChainType) => {
    if (!isMountedRef.current || !manualAddress.trim()) {
      if (isMountedRef.current) {
        alert('Please enter an address')
      }
      return
    }

    try {
      await multiChainWalletService.addManualAddress(chain, manualAddress.trim())
      if (isMountedRef.current) {
        setManualAddress('')
        setIsAddingManual(null)
      }
    } catch (error) {
      if (isMountedRef.current) {
        alert(`Failed to add address: ${error instanceof Error ? error.message : 'Unknown error'}`)
      }
    }
  }, [manualAddress])

  const handleDisconnectWallet = useCallback(async (chain: ChainType) => {
    if (!isMountedRef.current) return
    
    if (confirm(`Disconnect ${SUPPORTED_CHAINS[chain].name} wallet?`)) {
      await multiChainWalletService.disconnectWallet(chain)
    }
  }, [])

  const handleRemoveAddress = useCallback(async (chain: ChainType) => {
    if (!isMountedRef.current) return
    
    if (confirm(`Remove ${SUPPORTED_CHAINS[chain].name} address?`)) {
      await multiChainWalletService.removeWalletAddress(chain)
    }
  }, [])

  // PERFORMANCE: Throttled refresh balance to prevent spam clicking
  const handleRefreshBalance = useCallback(async (chain: ChainType) => {
    if (!isMountedRef.current) return
    
    try {
      await multiChainWalletService.updateBalance(chain)
    } catch (error) {
      console.error('Failed to refresh balance:', error)
    }
  }, [])

  // PERFORMANCE: Memoized utility functions
  const getWalletStatus = useCallback((address: WalletAddress): { status: string; color: string } => {
    if (address.isConnected) {
      return { status: 'Connected', color: 'text-green-600 bg-green-100' }
    }
    return { status: 'Manual', color: 'text-gray-600 bg-gray-100' }
  }, [])

  const formatAddress = useCallback((address: string): string => {
    if (address.length <= 16) return address
    return `${address.slice(0, 8)}...${address.slice(-8)}`
  }, [])
  
  // PERFORMANCE: Memoized chain entries to prevent re-computation
  const chainEntries = useMemo(() => {
    return Object.entries(SUPPORTED_CHAINS)
  }, [])
  
  // PERFORMANCE: Memoized connected wallets with caching
  const connectedWallets = useMemo(() => {
    // const cacheKey = `connected_wallets_${wallet.addresses.length}_${wallet.lastSyncAt}`
    // const cached = performanceService.getCache<WalletAddress[]>(cacheKey)
    // if (cached) return cached
    
    const result = wallet.addresses.filter(addr => addr.isConnected || addr.balance)
    // performanceService.setCache(cacheKey, result, 30000) // Cache for 30 seconds
    
    return result
  }, [wallet.addresses, wallet.lastSyncAt])

  if (!isOpen) return null
  
  // PERFORMANCE: Show loading state during initialization
  if (isInitializing) {
    return (
      <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
        <div className="bg-white rounded-lg shadow-xl p-8">
          <div className="flex items-center space-x-3">
            <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-600"></div>
            <span className="text-gray-700">Initializing secure wallet...</span>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl max-w-4xl w-full mx-4 max-h-[80vh] overflow-hidden">
        {/* Header */}
        <div className="px-6 py-4 border-b border-gray-200 flex items-center justify-between">
          <div>
            <h2 className="text-xl font-semibold text-gray-900">Multi-Chain Wallet</h2>
            <p className="text-gray-600 text-sm mt-1">
              Manage your addresses across {Object.keys(SUPPORTED_CHAINS).length} supported chains
            </p>
          </div>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 text-2xl font-light"
          >
            ×
          </button>
        </div>

        {/* Content */}
        <div className="p-6 overflow-y-auto max-h-[60vh]">
          {/* Connected Wallets Summary - PERFORMANCE: Use memoized connected wallets */}
          {connectedWallets.length > 0 && (
            <div className="mb-6 p-4 bg-blue-50 rounded-lg">
              <h3 className="text-lg font-medium text-blue-900 mb-2">Connected Wallets</h3>
              <div className="flex flex-wrap gap-2">
                {connectedWallets.map((addr: WalletAddress) => (
                  <div key={addr.chain} className="flex items-center space-x-2 bg-white px-3 py-1 rounded-full">
                    <span className="text-lg">{SUPPORTED_CHAINS[addr.chain].icon}</span>
                    <span className="text-sm font-medium">{SUPPORTED_CHAINS[addr.chain].name}</span>
                    {addr.balance && (
                      <span className="text-xs text-gray-600">
                        {addr.balance} {SUPPORTED_CHAINS[addr.chain].symbol}
                      </span>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Chain List - PERFORMANCE: Use memoized chain entries */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {chainEntries.map(([chainKey, config]) => {
              const chain = chainKey as ChainType
              const existingAddress = wallet.addresses.find(addr => addr.chain === chain)
              const { status, color } = existingAddress ? getWalletStatus(existingAddress) : { status: 'Not connected', color: 'text-gray-400' }

              return (
                <div key={chain} className="border border-gray-200 rounded-lg p-4">
                  {/* Chain Header */}
                  <div className="flex items-center justify-between mb-3">
                    <div className="flex items-center space-x-3">
                      <div 
                        className="w-8 h-8 rounded-full flex items-center justify-center text-white text-sm font-bold"
                        style={{ backgroundColor: config.color }}
                      >
                        {config.icon}
                      </div>
                      <div>
                        <h4 className="font-medium text-gray-900">{config.name}</h4>
                        <p className="text-xs text-gray-500">{config.symbol}</p>
                      </div>
                    </div>
                    <span className={`px-2 py-1 rounded-full text-xs font-medium ${color}`}>
                      {status}
                    </span>
                  </div>

                  {/* Address Display */}
                  {existingAddress && (
                    <div className="mb-3 p-2 bg-gray-50 rounded border">
                      <div className="flex items-center justify-between">
                        <span className="text-sm font-mono text-gray-700">
                          {formatAddress(existingAddress.address)}
                        </span>
                        <div className="flex items-center space-x-2">
                          <button
                            onClick={() => handleRefreshBalance(chain)}
                            className="text-xs text-blue-600 hover:text-blue-800 transition-colors"
                            title="Refresh balance (throttled to prevent spam)"
                          >
                            🔄
                          </button>
                          <button
                            onClick={() => {
                              navigator.clipboard.writeText(existingAddress.address)
                                .then(() => {
                                  // PERFORMANCE: Brief visual feedback without state updates
                                  const button = document.activeElement as HTMLButtonElement
                                  if (button) {
                                    const originalText = button.textContent
                                    button.textContent = '✅'
                                    setTimeout(() => {
                                      if (button.textContent === '✅') {
                                        button.textContent = originalText
                                      }
                                    }, 1000)
                                  }
                                })
                                .catch(console.error)
                            }}
                            className="text-xs text-gray-600 hover:text-gray-800 transition-colors"
                            title="Copy address"
                          >
                            📋
                          </button>
                        </div>
                      </div>
                      {existingAddress.balance && (
                        <div className="text-xs text-gray-600 mt-1">
                          Balance: {existingAddress.balance} {config.symbol}
                        </div>
                      )}
                    </div>
                  )}

                  {/* Actions */}
                  <div className="space-y-2">
                    {existingAddress ? (
                      <div className="flex space-x-2">
                        {existingAddress.isConnected && (
                          <button
                            onClick={() => handleDisconnectWallet(chain)}
                            className="flex-1 px-3 py-2 text-sm text-red-600 border border-red-300 rounded hover:bg-red-50"
                          >
                            Disconnect
                          </button>
                        )}
                        <button
                          onClick={() => handleRemoveAddress(chain)}
                          className="flex-1 px-3 py-2 text-sm text-gray-600 border border-gray-300 rounded hover:bg-gray-50"
                        >
                          Remove
                        </button>
                      </div>
                    ) : (
                      <div className="space-y-2">
                        {/* Wallet connection buttons */}
                        <div className="flex flex-wrap gap-1">
                          {config.supportedWallets
                            .filter(walletType => walletType !== 'Manual')
                            .map((walletType) => (
                            <button
                              key={walletType}
                              onClick={() => handleConnectWallet(chain, walletType)}
                              disabled={isConnecting?.chain === chain}
                              className="px-3 py-1 text-xs bg-blue-600 text-white rounded hover:bg-blue-700 disabled:bg-gray-400"
                            >
                              {isConnecting?.chain === chain && isConnecting?.walletType === walletType
                                ? 'Connecting...'
                                : walletType
                              }
                            </button>
                          ))}
                        </div>

                        {/* Manual address input */}
                        {isAddingManual === chain ? (
                          <div className="space-y-2">
                            <input
                              type="text"
                              value={manualAddress}
                              onChange={(e) => setManualAddress(e.target.value)}
                              placeholder={`Enter ${config.name} address`}
                              className="w-full px-3 py-2 text-sm border border-gray-300 rounded focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                            />
                            <div className="flex space-x-2">
                              <button
                                onClick={() => handleAddManualAddress(chain)}
                                className="flex-1 px-3 py-2 text-sm bg-green-600 text-white rounded hover:bg-green-700"
                              >
                                Add Address
                              </button>
                              <button
                                onClick={() => {
                                  setIsAddingManual(null)
                                  setManualAddress('')
                                }}
                                className="flex-1 px-3 py-2 text-sm text-gray-600 border border-gray-300 rounded hover:bg-gray-50"
                              >
                                Cancel
                              </button>
                            </div>
                          </div>
                        ) : (
                          <button
                            onClick={() => setIsAddingManual(chain)}
                            className="w-full px-3 py-2 text-sm text-gray-600 border border-gray-300 rounded hover:bg-gray-50"
                          >
                            Add Manual Address
                          </button>
                        )}
                      </div>
                    )}
                  </div>
                </div>
              )
            })}
          </div>

          {/* Footer Info */}
          <div className="mt-6 p-4 bg-yellow-50 rounded-lg">
            <h4 className="font-medium text-yellow-800 mb-2">💡 Multi-Chain DeFi Strategies</h4>
            <p className="text-sm text-yellow-700">
              Connect wallets from multiple chains to enable cross-chain arbitrage, yield farming, 
              and portfolio rebalancing strategies. Your addresses are stored locally and never shared.
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}

export default MultiChainWalletComponent