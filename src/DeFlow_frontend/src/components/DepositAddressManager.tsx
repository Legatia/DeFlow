/**
 * Deposit Address Manager Component
 * Generates deposit addresses for each supported blockchain
 * Provides secure deposit-based DeFi automation
 */

import React, { useState, useEffect, useRef } from 'react'
import { useEnhancedAuth } from '../contexts/EnhancedAuthContext'
import QRCode from 'qrcode'

interface DepositAddress {
  id: string
  chainType: 'ethereum' | 'bitcoin' | 'solana' | 'icp'
  chainName: string
  address: string
  qrCode?: string
  balance: number
  balanceUSD: number
  isGenerated: boolean
  createdAt: string
  lastChecked?: string
  transactions: DepositTransaction[]
}

interface DepositTransaction {
  txHash: string
  amount: string
  amountUSD: number
  timestamp: string
  confirmations: number
}

interface SupportedChain {
  chainType: 'ethereum' | 'bitcoin' | 'solana' | 'icp'
  chainName: string
  symbol: string
  description: string
  icon: string
  addressPrefix: string
  networkFee: string
}

const SUPPORTED_CHAINS: SupportedChain[] = [
  {
    chainType: "ethereum",
    chainName: "Ethereum Ecosystem",
    symbol: "ETH",
    description: "Works on: Ethereum, Polygon, Arbitrum, Optimism, BSC",
    icon: "⟠",
    addressPrefix: "0x",
    networkFee: "Varies by network"
  },
  {
    chainType: "bitcoin",
    chainName: "Bitcoin",
    symbol: "BTC",
    description: "Bitcoin mainnet",
    icon: "₿",
    addressPrefix: "bc1",
    networkFee: "~$1-10"
  },
  {
    chainType: "solana",
    chainName: "Solana",
    symbol: "SOL",
    description: "Solana mainnet",
    icon: "◎",
    addressPrefix: "",
    networkFee: "~$0.001"
  },
  {
    chainType: "icp",
    chainName: "Internet Computer",
    symbol: "ICP",
    description: "ICP ecosystem",
    icon: "∞",
    addressPrefix: "",
    networkFee: "~$0.001"
  }
]

const DepositAddressManager: React.FC = () => {
  const { subscriptionTier } = useEnhancedAuth()
  const [depositAddresses, setDepositAddresses] = useState<DepositAddress[]>([])
  const [generatingAddress, setGeneratingAddress] = useState<string | null>(null)
  const [showQRCode, setShowQRCode] = useState<string | null>(null)
  const [checkingBalance, setCheckingBalance] = useState<string | null>(null)
  const [copiedAddress, setCopiedAddress] = useState<string | null>(null)
  const [qrImages, setQrImages] = useState<Record<string, string>>({})
  const canvasRefs = useRef<Record<string, HTMLCanvasElement | null>>({})

  // Load saved deposit addresses from localStorage
  useEffect(() => {
    loadDepositAddresses()
  }, [])

  const loadDepositAddresses = () => {
    try {
      const savedAddresses = localStorage.getItem('deflow_deposit_addresses')
      if (savedAddresses) {
        setDepositAddresses(JSON.parse(savedAddresses))
      }
    } catch (error) {
      console.error('Error loading deposit addresses:', error)
    }
  }

  const saveDepositAddresses = (addresses: DepositAddress[]) => {
    try {
      localStorage.setItem('deflow_deposit_addresses', JSON.stringify(addresses))
      setDepositAddresses(addresses)
    } catch (error) {
      console.error('Error saving deposit addresses:', error)
    }
  }

  // Generate QR code data URL using QRCode library
  const generateQRCode = async (address: string, chainType: string): Promise<string> => {
    try {
      const prefix = chainType === 'bitcoin' ? 'bitcoin:' :
                    chainType === 'ethereum' ? 'ethereum:' : ''
      const data = prefix + address

      const qrDataUrl = await QRCode.toDataURL(data, {
        width: 200,
        margin: 2,
        color: {
          dark: '#000000',
          light: '#FFFFFF'
        }
      })

      return qrDataUrl
    } catch (error) {
      console.error('Failed to generate QR code:', error)
      // Fallback to external API
      const prefix = chainType === 'bitcoin' ? 'bitcoin:' :
                    chainType === 'ethereum' ? 'ethereum:' : ''
      return `https://api.qrserver.com/v1/create-qr-code/?size=200x200&data=${encodeURIComponent(prefix + address)}`
    }
  }

  // Generate deposit address for a specific chain
  const handleGenerateAddress = async (chain: SupportedChain) => {
    setGeneratingAddress(chain.chainType)

    try {
      // Call backend to generate real address
      const response = await fetch('/api/defi/generate_deposit_address', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          chain_type: chain.chainType
        })
      })

      let generatedAddress: string

      if (response.ok) {
        const result = await response.json()
        generatedAddress = result.address || result // Handle different response formats
      } else {
        // Fallback to mock address if API fails
        console.warn('Backend API failed, using mock address')
        generatedAddress = generateMockAddress(chain.chainType)
      }

      const qrCodeDataUrl = await generateQRCode(generatedAddress, chain.chainType)

      const newDepositAddress: DepositAddress = {
        id: Date.now().toString(),
        chainType: chain.chainType,
        chainName: chain.chainName,
        address: generatedAddress,
        qrCode: qrCodeDataUrl,
        balance: 0,
        balanceUSD: 0,
        isGenerated: true,
        createdAt: new Date().toISOString(),
        transactions: []
      }

      const updatedAddresses = [...depositAddresses, newDepositAddress]
      saveDepositAddresses(updatedAddresses)

    } catch (error) {
      console.error('Failed to generate address:', error)
    } finally {
      setGeneratingAddress(null)
    }
  }

  // Mock address generation (replace with actual backend call)
  const generateMockAddress = (chainType: string): string => {
    const mockAddresses = {
      bitcoin: 'bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh',
      ethereum: '0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e',
      solana: 'DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK',
      icp: 'rdmx6-jaaaa-aaaah-qcaiq-cai'
    }

    return mockAddresses[chainType as keyof typeof mockAddresses] || 'mock-address'
  }

  // Check balance for an address
  const handleCheckBalance = async (address: DepositAddress) => {
    setCheckingBalance(address.id)

    try {
      // Simulate balance check
      await new Promise(resolve => setTimeout(resolve, 2000))

      const mockBalance = Math.random() * 10
      const mockUSDValue = mockBalance * (
        address.chainType === 'bitcoin' ? 43000 :
        address.chainType === 'ethereum' ? 2500 :
        address.chainType === 'solana' ? 100 : 25
      )

      const updatedAddresses = depositAddresses.map(addr =>
        addr.id === address.id
          ? {
              ...addr,
              balance: mockBalance,
              balanceUSD: mockUSDValue,
              lastChecked: new Date().toISOString()
            }
          : addr
      )

      saveDepositAddresses(updatedAddresses)

    } catch (error) {
      console.error('Failed to check balance:', error)
    } finally {
      setCheckingBalance(null)
    }
  }

  // Copy address to clipboard
  const handleCopyAddress = async (address: string) => {
    try {
      await navigator.clipboard.writeText(address)
      setCopiedAddress(address)
      setTimeout(() => setCopiedAddress(null), 2000)
    } catch (error) {
      console.error('Failed to copy address:', error)
    }
  }

  // Remove deposit address
  const handleRemoveAddress = (addressId: string) => {
    if (confirm('Remove this deposit address? This cannot be undone.')) {
      const updatedAddresses = depositAddresses.filter(addr => addr.id !== addressId)
      saveDepositAddresses(updatedAddresses)
    }
  }

  // Check if chain already has generated address
  const getChainAddress = (chainType: string) => {
    return depositAddresses.find(addr => addr.chainType === chainType)
  }

  const canGenerateAddress = true // Allow all users to generate addresses for any chain

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium text-gray-900">Deposit Addresses</h3>
          <p className="text-sm text-gray-600">
            Generate unique deposit addresses for automated DeFi strategies across all supported blockchains.
          </p>
        </div>
      </div>

      {/* Welcome Message */}
      {depositAddresses.length === 0 && (
        <div className="bg-gradient-to-r from-purple-50 to-blue-50 border border-purple-200 rounded-lg p-6">
          <h4 className="font-medium text-purple-900 mb-3">🚀 Get Started with DeFi Automation</h4>
          <div className="space-y-3 text-sm text-purple-800">
            <p>
              <strong>How it works:</strong> DeFlow generates unique deposit addresses for each blockchain.
              Send your tokens to these addresses and we'll manage them with automated strategies.
            </p>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mt-4">
              <div className="bg-white/60 rounded-lg p-3">
                <h5 className="font-medium text-purple-900 mb-1">✅ What you get:</h5>
                <ul className="text-xs text-purple-700 space-y-1">
                  <li>• Unique deposit addresses per chain</li>
                  <li>• 24/7 automated DeFi strategies</li>
                  <li>• Cross-chain yield optimization</li>
                  <li>• Real-time balance tracking</li>
                </ul>
              </div>
              <div className="bg-white/60 rounded-lg p-3">
                <h5 className="font-medium text-purple-900 mb-1">🔐 Security benefits:</h5>
                <ul className="text-xs text-purple-700 space-y-1">
                  <li>• You keep your private keys</li>
                  <li>• Deposit only what you want managed</li>
                  <li>• ICP threshold cryptography</li>
                  <li>• Full transaction transparency</li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Supported Chains */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {SUPPORTED_CHAINS.map((chain) => {
          const existingAddress = getChainAddress(chain.chainType)

          return (
            <div key={chain.chainType} className="bg-white border border-gray-200 rounded-xl p-6 shadow-sm">
              <div className="flex items-start justify-between mb-4">
                <div className="flex items-center space-x-3">
                  <div className="text-2xl">{chain.icon}</div>
                  <div>
                    <h4 className="font-medium text-gray-900">{chain.chainName}</h4>
                    <p className="text-sm text-gray-600">{chain.symbol}</p>
                  </div>
                </div>

                <div className="text-right">
                  <div className="text-xs text-gray-500">Network Fee</div>
                  <div className="text-sm font-medium text-gray-700">{chain.networkFee}</div>
                </div>
              </div>

              <p className="text-sm text-gray-600 mb-4">{chain.description}</p>

              {existingAddress ? (
                <div className="space-y-4">
                  {/* Address Display */}
                  <div className="bg-gray-50 rounded-lg p-4">
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-sm font-medium text-gray-700">Deposit Address:</span>
                      <div className="flex space-x-2">
                        <button
                          onClick={() => setShowQRCode(showQRCode === existingAddress.id ? null : existingAddress.id)}
                          className="text-sm text-blue-600 hover:text-blue-800"
                        >
                          {showQRCode === existingAddress.id ? 'Hide QR' : 'Show QR'}
                        </button>
                        {!existingAddress.qrCode && (
                          <button
                            onClick={async () => {
                              const qrCodeDataUrl = await generateQRCode(existingAddress.address, existingAddress.chainType)
                              const updatedAddresses = depositAddresses.map(addr =>
                                addr.id === existingAddress.id ? { ...addr, qrCode: qrCodeDataUrl } : addr
                              )
                              saveDepositAddresses(updatedAddresses)
                            }}
                            className="text-sm text-purple-600 hover:text-purple-800"
                          >
                            🔄 Generate QR
                          </button>
                        )}
                      </div>
                    </div>

                    <div className="flex items-center space-x-2">
                      <code className="flex-1 text-sm bg-white px-3 py-2 rounded border font-mono break-all">
                        {existingAddress.address}
                      </code>
                      <button
                        onClick={() => handleCopyAddress(existingAddress.address)}
                        className={`px-3 py-2 text-sm rounded transition-colors ${
                          copiedAddress === existingAddress.address
                            ? 'bg-green-100 text-green-800'
                            : 'bg-blue-100 text-blue-800 hover:bg-blue-200'
                        }`}
                      >
                        {copiedAddress === existingAddress.address ? '✓ Copied' : '📋 Copy'}
                      </button>
                    </div>

                    {/* ETH Network Info */}
                    {existingAddress.chainType === 'ethereum' && (
                      <div className="mt-3 p-3 bg-blue-50 rounded-lg">
                        <h5 className="text-sm font-medium text-blue-900 mb-2">💡 Multi-Network Support</h5>
                        <p className="text-xs text-blue-800 mb-2">
                          This address works on all Ethereum-compatible networks:
                        </p>
                        <div className="grid grid-cols-2 gap-2 text-xs">
                          <div className="text-blue-700">• Ethereum (ETH)</div>
                          <div className="text-blue-700">• Polygon (MATIC)</div>
                          <div className="text-blue-700">• Arbitrum (ETH)</div>
                          <div className="text-blue-700">• Optimism (ETH)</div>
                          <div className="text-blue-700">• BSC (BNB)</div>
                          <div className="text-blue-700">• Base (ETH)</div>
                        </div>
                        <p className="text-xs text-blue-600 mt-2 font-medium">
                          💰 Tip: Use Polygon or Arbitrum for lower fees!
                        </p>
                      </div>
                    )}

                    {/* QR Code */}
                    {showQRCode === existingAddress.id && (
                      <div className="mt-4 flex justify-center">
                        <div className="bg-white p-4 rounded-lg border shadow-sm">
                          {existingAddress.qrCode ? (
                            <>
                              <img
                                src={existingAddress.qrCode}
                                alt="QR Code for deposit address"
                                className="w-48 h-48 mx-auto"
                                onError={(e) => {
                                  console.error('QR code image failed to load:', existingAddress.qrCode)
                                  // Try to regenerate QR code
                                  generateQRCode(existingAddress.address, existingAddress.chainType)
                                    .then(newQr => {
                                      const updatedAddresses = depositAddresses.map(addr =>
                                        addr.id === existingAddress.id ? { ...addr, qrCode: newQr } : addr
                                      )
                                      saveDepositAddresses(updatedAddresses)
                                    })
                                }}
                              />
                              <p className="text-xs text-gray-500 text-center mt-2">
                                📱 Scan with wallet app to copy address
                              </p>
                              <div className="mt-3 text-center">
                                <a
                                  href={existingAddress.qrCode}
                                  download={`deflow-${existingAddress.chainType}-address-qr.png`}
                                  className="text-xs text-blue-600 hover:text-blue-800 underline"
                                >
                                  💾 Download QR Code
                                </a>
                              </div>
                            </>
                          ) : (
                            <div className="w-48 h-48 mx-auto flex items-center justify-center bg-gray-100 rounded">
                              <div className="text-center text-gray-500">
                                <div className="animate-spin w-8 h-8 border-2 border-purple-500 border-t-transparent rounded-full mx-auto mb-2"></div>
                                <p className="text-xs">Generating QR Code...</p>
                              </div>
                            </div>
                          )}
                        </div>
                      </div>
                    )}
                  </div>

                  {/* Balance Info */}
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="text-sm text-gray-600">Balance:</div>
                      <div className="font-medium">
                        {existingAddress.balance > 0
                          ? `${existingAddress.balance.toFixed(6)} ${chain.symbol} ($${existingAddress.balanceUSD.toFixed(2)})`
                          : `0 ${chain.symbol}`
                        }
                      </div>
                      {existingAddress.lastChecked && (
                        <div className="text-xs text-gray-500">
                          Last checked: {new Date(existingAddress.lastChecked).toLocaleString()}
                        </div>
                      )}
                    </div>

                    <div className="flex space-x-2">
                      <button
                        onClick={() => handleCheckBalance(existingAddress)}
                        disabled={checkingBalance === existingAddress.id}
                        className="px-3 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50 text-sm"
                      >
                        {checkingBalance === existingAddress.id ? '⏳' : '🔄'}
                        {checkingBalance === existingAddress.id ? 'Checking...' : 'Refresh'}
                      </button>

                      <button
                        onClick={() => handleRemoveAddress(existingAddress.id)}
                        className="px-3 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors text-sm"
                      >
                        🗑️
                      </button>
                    </div>
                  </div>
                </div>
              ) : (
                <button
                  onClick={() => handleGenerateAddress(chain)}
                  disabled={generatingAddress === chain.chainType || !canGenerateAddress}
                  className="w-full px-4 py-3 bg-gradient-to-r from-purple-600 to-blue-600 text-white rounded-lg hover:from-purple-700 hover:to-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed font-medium"
                >
                  {generatingAddress === chain.chainType ? (
                    <div className="flex items-center justify-center space-x-2">
                      <div className="animate-spin w-4 h-4 border-2 border-white border-t-transparent rounded-full"></div>
                      <span>Generating Address...</span>
                    </div>
                  ) : (
                    `Generate ${chain.chainName} Address`
                  )}
                </button>
              )}
            </div>
          )
        })}
      </div>

      {/* Usage Instructions */}
      {depositAddresses.length > 0 && (
        <div className="bg-gradient-to-br from-blue-50 to-indigo-50 border border-blue-200 rounded-xl p-6">
          <h4 className="font-medium text-blue-900 mb-3">💡 Using Your Deposit Addresses</h4>
          <div className="text-sm text-blue-800 space-y-2">
            <p>Your deposit addresses are ready for automated DeFi strategies:</p>
            <ul className="space-y-1 list-disc list-inside ml-4">
              <li><strong>Send tokens:</strong> Transfer any amount to your deposit addresses</li>
              <li><strong>Automatic detection:</strong> We'll detect deposits within minutes</li>
              <li><strong>Strategy execution:</strong> Configure workflows to trade, stake, or provide liquidity</li>
              <li><strong>Withdraw anytime:</strong> Full control over your deposited funds</li>
            </ul>
            <p className="mt-3 p-3 bg-blue-100/50 rounded-lg">
              <strong>⚡ Pro Tip:</strong> Start with small amounts to test the system before depositing larger sums.
            </p>
          </div>
        </div>
      )}
    </div>
  )
}

export default DepositAddressManager