import React, { useState, useEffect } from 'react'
import multiChainWalletService, { MultiChainWallet, ChainType, SUPPORTED_CHAINS } from '../services/multiChainWalletService'

interface WalletIntegrationProps {
  selectedChains?: ChainType[]
  onWalletChange?: (wallet: MultiChainWallet) => void
}

const WalletIntegration = ({ selectedChains, onWalletChange }: WalletIntegrationProps) => {
  const [wallet, setWallet] = useState<MultiChainWallet>({ addresses: [], lastSyncAt: 0 })
  const [expandedChain, setExpandedChain] = useState<ChainType | null>(null)

  // Initialize wallet asynchronously
  useEffect(() => {
    const initializeWallet = async () => {
      try {
        const currentWallet = await multiChainWalletService.getWallet()
        setWallet(currentWallet)
        onWalletChange?.(currentWallet)
      } catch (error) {
        console.error('Failed to initialize wallet in WalletIntegration:', error)
      }
    }
    
    initializeWallet()
  }, [onWalletChange])

  useEffect(() => {
    const handleWalletUpdate = (updatedWallet: MultiChainWallet) => {
      setWallet(updatedWallet)
      onWalletChange?.(updatedWallet)
    }

    multiChainWalletService.addListener(handleWalletUpdate)

    return () => {
      multiChainWalletService.removeListener(handleWalletUpdate)
    }
  }, [onWalletChange])

  const getRelevantAddresses = () => {
    if (!selectedChains || selectedChains.length === 0) {
      return wallet.addresses
    }
    return wallet.addresses.filter(addr => selectedChains.includes(addr.chain))
  }

  const getMissingChains = () => {
    if (!selectedChains) return []
    const connectedChains = wallet.addresses.map(addr => addr.chain)
    return selectedChains.filter(chain => !connectedChains.includes(chain))
  }

  const formatAddress = (address: string): string => {
    if (address.length <= 16) return address
    return `${address.slice(0, 8)}...${address.slice(-8)}`
  }

  const relevantAddresses = getRelevantAddresses()
  const missingChains = getMissingChains()

  if (wallet.addresses.length === 0) {
    return (
      <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
        <div className="flex items-start space-x-3">
          <span className="text-yellow-500 text-xl">⚠️</span>
          <div>
            <h4 className="text-yellow-800 font-medium">No Wallets Connected</h4>
            <p className="text-yellow-700 text-sm mt-1">
              Connect wallets to enable DeFi strategy execution. Click "Connect Wallet" in the header to get started.
            </p>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-4">
      {/* Connected Wallets for Selected Chains */}
      {relevantAddresses.length > 0 && (
        <div className="bg-green-50 border border-green-200 rounded-lg p-4">
          <h4 className="text-green-800 font-medium mb-3 flex items-center">
            <span className="mr-2">✅</span>
            Available Wallets for Strategy
          </h4>
          <div className="space-y-2">
            {relevantAddresses.map((addr) => {
              const chainConfig = SUPPORTED_CHAINS[addr.chain]
              const isExpanded = expandedChain === addr.chain
              
              return (
                <div key={addr.chain} className="bg-white rounded border border-green-200 p-3">
                  <div 
                    className="flex items-center justify-between cursor-pointer"
                    onClick={() => setExpandedChain(isExpanded ? null : addr.chain)}
                  >
                    <div className="flex items-center space-x-3">
                      <div 
                        className="w-6 h-6 rounded-full flex items-center justify-center text-white text-sm"
                        style={{ backgroundColor: chainConfig.color }}
                      >
                        {chainConfig.icon}
                      </div>
                      <div>
                        <span className="font-medium text-gray-900">{chainConfig.name}</span>
                        <span className="ml-2 text-sm text-gray-600">
                          ({addr.isConnected ? 'Connected' : 'Manual'})
                        </span>
                      </div>
                    </div>
                    <div className="flex items-center space-x-2">
                      {addr.balance && (
                        <span className="text-sm text-gray-600">
                          {addr.balance} {chainConfig.symbol}
                        </span>
                      )}
                      <span className="text-gray-400">
                        {isExpanded ? '▼' : '▶'}
                      </span>
                    </div>
                  </div>
                  
                  {isExpanded && (
                    <div className="mt-3 pt-3 border-t border-gray-100 space-y-2">
                      <div>
                        <label className="block text-xs font-medium text-gray-700 mb-1">
                          Address
                        </label>
                        <div className="flex items-center space-x-2">
                          <code className="flex-1 px-2 py-1 bg-gray-100 rounded text-sm font-mono">
                            {addr.address}
                          </code>
                          <button
                            onClick={() => navigator.clipboard.writeText(addr.address)}
                            className="text-blue-600 hover:text-blue-800 text-sm"
                            title="Copy address"
                          >
                            📋
                          </button>
                        </div>
                      </div>
                      
                      {addr.balance && (
                        <div>
                          <label className="block text-xs font-medium text-gray-700 mb-1">
                            Balance
                          </label>
                          <div className="text-sm text-gray-900">
                            {addr.balance} {chainConfig.symbol}
                          </div>
                        </div>
                      )}
                      
                      <div>
                        <label className="block text-xs font-medium text-gray-700 mb-1">
                          Status
                        </label>
                        <span className={`inline-flex px-2 py-1 rounded-full text-xs font-medium ${
                          addr.isConnected 
                            ? 'bg-green-100 text-green-800' 
                            : 'bg-gray-100 text-gray-800'
                        }`}>
                          {addr.isConnected ? `Connected via ${addr.walletType}` : 'Manual Address'}
                        </span>
                      </div>
                    </div>
                  )}
                </div>
              )
            })}
          </div>
        </div>
      )}

      {/* Missing Chains Warning */}
      {missingChains.length > 0 && (
        <div className="bg-orange-50 border border-orange-200 rounded-lg p-4">
          <div className="flex items-start space-x-3">
            <span className="text-orange-500 text-xl">⚠️</span>
            <div className="flex-1">
              <h4 className="text-orange-800 font-medium">Missing Wallet Connections</h4>
              <p className="text-orange-700 text-sm mt-1">
                Your strategy requires wallets on the following chains:
              </p>
              <div className="flex flex-wrap gap-2 mt-2">
                {missingChains.map((chain) => {
                  const chainConfig = SUPPORTED_CHAINS[chain]
                  return (
                    <div 
                      key={chain}
                      className="flex items-center space-x-2 bg-white px-3 py-1 rounded border border-orange-300"
                    >
                      <div 
                        className="w-4 h-4 rounded-full flex items-center justify-center text-white text-xs"
                        style={{ backgroundColor: chainConfig.color }}
                      >
                        {chainConfig.icon}
                      </div>
                      <span className="text-sm font-medium text-gray-900">
                        {chainConfig.name}
                      </span>
                    </div>
                  )
                })}
              </div>
              <p className="text-orange-700 text-sm mt-2">
                Connect these wallets to enable full strategy execution.
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Portfolio Summary */}
      {wallet.addresses.length > 0 && (
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <h4 className="text-blue-800 font-medium mb-3">💼 Portfolio Overview</h4>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
            <div className="bg-white rounded p-3 text-center">
              <div className="text-2xl font-bold text-gray-900">{wallet.addresses.length}</div>
              <div className="text-sm text-gray-600">Connected Chains</div>
            </div>
            <div className="bg-white rounded p-3 text-center">
              <div className="text-2xl font-bold text-gray-900">
                {wallet.addresses.filter(addr => addr.isConnected).length}
              </div>
              <div className="text-sm text-gray-600">Active Connections</div>
            </div>
            <div className="bg-white rounded p-3 text-center">
              <div className="text-2xl font-bold text-gray-900">
                {wallet.addresses.filter(addr => addr.balance && parseFloat(addr.balance) > 0).length}
              </div>
              <div className="text-sm text-gray-600">Funded Wallets</div>
            </div>
            <div className="bg-white rounded p-3 text-center">
              <div className="text-2xl font-bold text-green-600">Ready</div>
              <div className="text-sm text-gray-600">Strategy Status</div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default WalletIntegration