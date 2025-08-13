import React, { useState, useEffect } from 'react'
import multiChainWalletService, { 
  MultiChainWallet, 
  ChainType, 
  WalletType, 
  SUPPORTED_CHAINS,
  WalletAddress 
} from '../services/multiChainWalletService'

interface MultiChainWalletProps {
  isOpen: boolean
  onClose: () => void
}

const MultiChainWalletComponent = ({ isOpen, onClose }: MultiChainWalletProps) => {
  const [wallet, setWallet] = useState<MultiChainWallet>(multiChainWalletService.getWallet())
  const [selectedChain, setSelectedChain] = useState<ChainType | null>(null)
  const [manualAddress, setManualAddress] = useState('')
  const [isConnecting, setIsConnecting] = useState<{ chain: ChainType; walletType: WalletType } | null>(null)
  const [isAddingManual, setIsAddingManual] = useState<ChainType | null>(null)

  useEffect(() => {
    const handleWalletUpdate = (updatedWallet: MultiChainWallet) => {
      setWallet(updatedWallet)
    }

    multiChainWalletService.addListener(handleWalletUpdate)
    
    // Load initial balances
    multiChainWalletService.updateAllBalances()

    return () => {
      multiChainWalletService.removeListener(handleWalletUpdate)
    }
  }, [])

  const handleConnectWallet = async (chain: ChainType, walletType: WalletType) => {
    setIsConnecting({ chain, walletType })
    try {
      await multiChainWalletService.connectWallet(chain, walletType)
    } catch (error) {
      alert(`Failed to connect ${walletType}: ${error instanceof Error ? error.message : 'Unknown error'}`)
    } finally {
      setIsConnecting(null)
    }
  }

  const handleAddManualAddress = async (chain: ChainType) => {
    if (!manualAddress.trim()) {
      alert('Please enter an address')
      return
    }

    try {
      await multiChainWalletService.addManualAddress(chain, manualAddress.trim())
      setManualAddress('')
      setIsAddingManual(null)
    } catch (error) {
      alert(`Failed to add address: ${error instanceof Error ? error.message : 'Unknown error'}`)
    }
  }

  const handleDisconnectWallet = async (chain: ChainType) => {
    if (confirm(`Disconnect ${SUPPORTED_CHAINS[chain].name} wallet?`)) {
      await multiChainWalletService.disconnectWallet(chain)
    }
  }

  const handleRemoveAddress = async (chain: ChainType) => {
    if (confirm(`Remove ${SUPPORTED_CHAINS[chain].name} address?`)) {
      await multiChainWalletService.removeWalletAddress(chain)
    }
  }

  const handleRefreshBalance = async (chain: ChainType) => {
    await multiChainWalletService.updateBalance(chain)
  }

  const getWalletStatus = (address: WalletAddress): { status: string; color: string } => {
    if (address.isConnected) {
      return { status: 'Connected', color: 'text-green-600 bg-green-100' }
    }
    return { status: 'Manual', color: 'text-gray-600 bg-gray-100' }
  }

  const formatAddress = (address: string): string => {
    if (address.length <= 16) return address
    return `${address.slice(0, 8)}...${address.slice(-8)}`
  }

  if (!isOpen) return null

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
          {/* Connected Wallets Summary */}
          {wallet.addresses.length > 0 && (
            <div className="mb-6 p-4 bg-blue-50 rounded-lg">
              <h3 className="text-lg font-medium text-blue-900 mb-2">Connected Wallets</h3>
              <div className="flex flex-wrap gap-2">
                {wallet.addresses.map((addr) => (
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

          {/* Chain List */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {Object.entries(SUPPORTED_CHAINS).map(([chainKey, config]) => {
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
                            className="text-xs text-blue-600 hover:text-blue-800"
                            title="Refresh balance"
                          >
                            🔄
                          </button>
                          <button
                            onClick={() => navigator.clipboard.writeText(existingAddress.address)}
                            className="text-xs text-gray-600 hover:text-gray-800"
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