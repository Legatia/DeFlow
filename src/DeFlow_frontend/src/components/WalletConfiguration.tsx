/**
 * Wallet Configuration Component
 * Handles seed phrase entry, wallet creation, and balance management
 * Provides secure wallet configuration for DeFi automation
 */

import React, { useState, useEffect } from 'react'
import { useEnhancedAuth } from '../contexts/EnhancedAuthContext'
import walletService, { WalletBalance } from '../services/walletService'
import SpendingApprovalManager from './SpendingApprovalManager'
import spendingApprovalService from '../services/spendingApprovalService'

interface WalletConfig {
  id: string
  name: string
  address: string
  derivationPath: string
  chainType: 'ethereum' | 'bitcoin' | 'solana' | 'icp'
  isConnected: boolean
  balances: WalletBalance[]
  createdAt: string
  lastChecked?: string
  encryptedSeed?: string // Encrypted seed phrase
}

interface DerivationOption {
  path: string
  label: string
  chainType: 'ethereum' | 'bitcoin' | 'solana' | 'icp'
  description: string
}

const DERIVATION_PATHS: DerivationOption[] = [
  {
    path: "m/44'/60'/0'/0/0",
    label: "Ethereum Standard",
    chainType: "ethereum",
    description: "Ethereum, Polygon, Arbitrum, Optimism, BSC"
  },
  {
    path: "m/44'/0'/0'/0/0",
    label: "Bitcoin Standard",
    chainType: "bitcoin",
    description: "Bitcoin mainnet"
  },
  {
    path: "m/44'/501'/0'/0'",
    label: "Solana Standard",
    chainType: "solana",
    description: "Solana mainnet"
  },
  {
    path: "m/44'/223'/0'/0/0",
    label: "Internet Computer",
    chainType: "icp",
    description: "ICP ecosystem"
  }
]

const WalletConfiguration: React.FC = () => {
  const { subscriptionTier } = useEnhancedAuth()
  const [wallets, setWallets] = useState<WalletConfig[]>([])
  const [isAddingWallet, setIsAddingWallet] = useState(false)
  const [testingWallet, setTestingWallet] = useState<string | null>(null)
  const [showSeedPhrase, setShowSeedPhrase] = useState(false)
  
  const [newWallet, setNewWallet] = useState<{
    name: string
    seedPhrase: string
    derivationPath: string
    chainType: 'ethereum' | 'bitcoin' | 'solana' | 'icp'
  }>({
    name: '',
    seedPhrase: '',
    derivationPath: "m/44'/60'/0'/0/0",
    chainType: 'ethereum'
  })
  
  const [validationErrors, setValidationErrors] = useState<Record<string, string>>({})
  const [testResults, setTestResults] = useState<Record<string, { success: boolean; message: string }>>({})
  
  // Spending approval flow states
  const [showSpendingApproval, setShowSpendingApproval] = useState(false)
  const [newlyCreatedWallet, setNewlyCreatedWallet] = useState<WalletConfig | null>(null)
  const [availableTokens, setAvailableTokens] = useState<Array<{
    symbol: string
    balance: string
    balanceUSD: number
    contractAddress?: string
    decimals: number
  }>>([])

  // Load saved wallets from localStorage
  useEffect(() => {
    loadWallets()
  }, [])

  const loadWallets = () => {
    try {
      const savedWallets = localStorage.getItem('deflow_wallets')
      if (savedWallets) {
        setWallets(JSON.parse(savedWallets))
      }
    } catch (error) {
      console.error('Error loading wallets:', error)
    }
  }

  const saveWallets = (updatedWallets: WalletConfig[]) => {
    try {
      localStorage.setItem('deflow_wallets', JSON.stringify(updatedWallets))
      setWallets(updatedWallets)
    } catch (error) {
      console.error('Error saving wallets:', error)
    }
  }

  const validateSeedPhrase = (seedPhrase: string): boolean => {
    const validation = walletService.validateSeedPhrase(seedPhrase)
    return validation.valid
  }

  const validateWalletConfig = (config: typeof newWallet): Record<string, string> => {
    const errors: Record<string, string> = {}

    if (!config.name.trim()) {
      errors.name = 'Wallet name is required'
    }

    if (!config.seedPhrase.trim()) {
      errors.seedPhrase = 'Seed phrase is required'
    } else if (!validateSeedPhrase(config.seedPhrase)) {
      errors.seedPhrase = 'Invalid seed phrase. Must be 12-24 words separated by spaces'
    }

    if (!config.derivationPath.trim()) {
      errors.derivationPath = 'Derivation path is required'
    }

    return errors
  }


  const handleDerivationPathChange = (path: string) => {
    const option = DERIVATION_PATHS.find(opt => opt.path === path)
    if (option) {
      setNewWallet({
        ...newWallet,
        derivationPath: path,
        chainType: option.chainType
      })
    }
  }

  const handleAddWallet = async () => {
    const errors = validateWalletConfig(newWallet)
    setValidationErrors(errors)

    if (Object.keys(errors).length > 0) {
      return
    }

    // Check for duplicate names
    if (wallets.some(wallet => wallet.name === newWallet.name)) {
      setValidationErrors({ name: 'A wallet with this name already exists' })
      return
    }

    try {
      // Generate wallet from seed phrase
      const walletData = await walletService.generateWalletFromSeed(
        newWallet.seedPhrase, 
        newWallet.derivationPath,
        newWallet.chainType
      )

      // Encrypt seed phrase
      const encryptedSeed = await walletService.encryptSeedPhrase(newWallet.seedPhrase)

      const walletConfig: WalletConfig = {
        id: Date.now().toString(),
        name: newWallet.name,
        address: walletData.address,
        derivationPath: newWallet.derivationPath,
        chainType: newWallet.chainType,
        isConnected: false,
        balances: [],
        createdAt: new Date().toISOString(),
        encryptedSeed: encryptedSeed
      }

      const updatedWallets = [...wallets, walletConfig]
      saveWallets(updatedWallets)

      // Load available tokens for spending approval
      const mockTokens = await generateMockTokenBalances(walletData.address, newWallet.chainType)
      setAvailableTokens(mockTokens)
      
      // Store newly created wallet and show spending approval flow
      setNewlyCreatedWallet(walletConfig)
      setShowSpendingApproval(true)

      // Reset form
      setNewWallet({
        name: '',
        seedPhrase: '',
        derivationPath: "m/44'/60'/0'/0/0",
        chainType: 'ethereum'
      })
      setValidationErrors({})
      setIsAddingWallet(false)
      setShowSeedPhrase(false)

    } catch (error) {
      setValidationErrors({ 
        seedPhrase: 'Failed to generate wallet. Please check your seed phrase.' 
      })
    }
  }

  const handleTestWallet = async (wallet: WalletConfig) => {
    setTestingWallet(wallet.id)
    setTestResults(prev => ({ ...prev, [wallet.id]: { success: false, message: 'Checking balances...' } }))

    try {
      // Test wallet connection and get balances
      const testResult = await walletService.testWalletConnection(wallet.address, wallet.chainType)
      
      if (!testResult.valid) {
        setTestResults(prev => ({
          ...prev,
          [wallet.id]: { success: false, message: `❌ ${testResult.error}` }
        }))
        return
      }

      // Get wallet balances
      const balances = await walletService.checkWalletBalances(wallet.address, wallet.chainType)

      setTestResults(prev => ({
        ...prev,
        [wallet.id]: { 
          success: true, 
          message: `✅ Connected successfully! Found ${balances.length} tokens with total value ${walletService.formatUSD(balances.reduce((sum, b) => sum + b.balanceUSD, 0))}.` 
        }
      }))

      // Update wallet with balances and connection status
      const updatedWallets = wallets.map(w => 
        w.id === wallet.id 
          ? { 
              ...w, 
              isConnected: true, 
              balances: balances,
              lastChecked: new Date().toISOString() 
            }
          : w
      )
      saveWallets(updatedWallets)

    } catch (error) {
      setTestResults(prev => ({
        ...prev,
        [wallet.id]: { 
          success: false, 
          message: `❌ Connection failed: ${error instanceof Error ? error.message : 'Unknown error'}` 
        }
      }))
    } finally {
      setTestingWallet(null)
    }
  }

  const handleDeleteWallet = (walletId: string) => {
    if (confirm('Delete this wallet configuration? This cannot be undone. Make sure you have your seed phrase backed up.')) {
      const updatedWallets = wallets.filter(wallet => wallet.id !== walletId)
      saveWallets(updatedWallets)
      // Clean up test results
      setTestResults(prev => {
        const newResults = { ...prev }
        delete newResults[walletId]
        return newResults
      })
    }
  }

  const maskSeedPhrase = (phrase: string): string => {
    const words = phrase.split(' ')
    if (words.length < 4) return '••• ••• ••• •••'
    return `${words[0]} ${words[1]} ••• ••• ••• ${words[words.length-2]} ${words[words.length-1]}`
  }

  // Helper function to generate mock token balances for demo
  const generateMockTokenBalances = async (address: string, chainType: string) => {
    // Mock token balances based on chain type
    const mockBalances = {
      ethereum: [
        { symbol: 'ETH', balance: '2.45', balanceUSD: 6125, decimals: 18 },
        { symbol: 'USDC', balance: '1500.00', balanceUSD: 1500, contractAddress: '0xA0b86a33E6417c70C8bd9Eff59bBf8B70dC7fF9D', decimals: 6 },
        { symbol: 'USDT', balance: '800.50', balanceUSD: 800.50, contractAddress: '0xdAC17F958D2ee523a2206206994597C13D831ec7', decimals: 6 },
        { symbol: 'DAI', balance: '1200.00', balanceUSD: 1200, contractAddress: '0x6B175474E89094C44Da98b954EedeAC495271d0F', decimals: 18 },
      ],
      bitcoin: [
        { symbol: 'BTC', balance: '0.05', balanceUSD: 2250, decimals: 8 }
      ],
      solana: [
        { symbol: 'SOL', balance: '25.0', balanceUSD: 2500, decimals: 9 },
        { symbol: 'USDC', balance: '1000.00', balanceUSD: 1000, contractAddress: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v', decimals: 6 }
      ]
    }
    
    return mockBalances[chainType as keyof typeof mockBalances] || []
  }

  // Handle spending approval completion
  const handleApprovalComplete = async (approvals: any[]) => {
    try {
      // Initialize spending approval service with new wallet
      if (newlyCreatedWallet) {
        await spendingApprovalService.initialize(newlyCreatedWallet.address)
        
        // Create approvals on-chain
        const result = await spendingApprovalService.createApprovals(
          newlyCreatedWallet.address,
          approvals.map(approval => ({
            token: approval.token,
            symbol: approval.symbol,
            contractAddress: approval.contractAddress || '',
            maxAmount: approval.maxAmount,
            dailyLimit: approval.dailyLimit,
            operationsAllowed: approval.operationsAllowed.map((op: any) => op.type),
            chainId: 1 // Default to Ethereum mainnet
          }))
        )
        
        console.log('Spending approvals created:', result)
      }
      
      // Close spending approval flow
      setShowSpendingApproval(false)
      setNewlyCreatedWallet(null)
      setAvailableTokens([])
      
    } catch (error) {
      console.error('Failed to create spending approvals:', error)
    }
  }

  // Handle skipping spending approval
  const handleSkipApproval = () => {
    setShowSpendingApproval(false)
    setNewlyCreatedWallet(null)
    setAvailableTokens([])
  }

  // Check if user has premium features
  const canAddWallet = subscriptionTier === 'premium' || wallets.length === 0

  // Show spending approval manager if active
  if (showSpendingApproval && newlyCreatedWallet) {
    return (
      <SpendingApprovalManager
        walletAddress={newlyCreatedWallet.address}
        availableTokens={availableTokens}
        onApprovalComplete={handleApprovalComplete}
        onSkip={handleSkipApproval}
      />
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium text-gray-900">Wallet Configuration</h3>
          <p className="text-sm text-gray-600">
            Connect your wallets using seed phrases for automated DeFi strategies.
            {!canAddWallet && (
              <span className="text-amber-600 font-medium ml-1">Premium required for multiple wallets</span>
            )}
          </p>
        </div>
        
        <button
          onClick={() => setIsAddingWallet(true)}
          disabled={!canAddWallet}
          className="px-4 py-2 bg-gradient-to-r from-purple-600 to-blue-600 text-white rounded-lg hover:from-purple-700 hover:to-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center space-x-2"
        >
          <span>+</span>
          <span>Add Wallet</span>
        </button>
      </div>

      {/* Security Warning */}
      {wallets.length === 0 && !isAddingWallet && (
        <div className="bg-gradient-to-r from-purple-50 to-blue-50 border border-purple-200 rounded-lg p-6">
          <h4 className="font-medium text-purple-900 mb-3">🚀 Get Started with Wallet Automation</h4>
          <div className="space-y-3 text-sm text-purple-800">
            <p>
              <strong>How it works:</strong> DeFlow uses your seed phrase to generate wallet addresses 
              and execute DeFi strategies automatically without requiring manual approvals.
            </p>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mt-4">
              <div className="bg-white/60 rounded-lg p-3">
                <h5 className="font-medium text-purple-900 mb-1">✅ What you get:</h5>
                <ul className="text-xs text-purple-700 space-y-1">
                  <li>• Fully automated trading strategies</li>
                  <li>• 24/7 portfolio rebalancing</li>
                  <li>• Cross-chain DeFi operations</li>
                  <li>• No manual transaction approvals</li>
                </ul>
              </div>
              <div className="bg-white/60 rounded-lg p-3">
                <h5 className="font-medium text-purple-900 mb-1">🔐 Security measures:</h5>
                <ul className="text-xs text-purple-700 space-y-1">
                  <li>• Seed phrases encrypted locally</li>
                  <li>• Never transmitted to servers</li>
                  <li>• Optional spending limits</li>
                  <li>• Full transaction logging</li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Add Wallet Form */}
      {isAddingWallet && (
        <div className="bg-gradient-to-br from-slate-50 to-gray-50 border border-gray-200 rounded-xl p-6">
          <h4 className="font-medium text-gray-900 mb-6">Add New Wallet</h4>
          
          <div className="space-y-6">
            {/* Wallet Name */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Wallet Name *
              </label>
              <input
                type="text"
                value={newWallet.name}
                onChange={(e) => setNewWallet({ ...newWallet, name: e.target.value })}
                placeholder="Main Portfolio"
                className={`w-full px-4 py-3 border rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500 transition-all ${
                  validationErrors.name ? 'border-red-500 bg-red-50' : 'border-gray-300 bg-white'
                }`}
              />
              {validationErrors.name && (
                <p className="text-red-600 text-sm mt-1">{validationErrors.name}</p>
              )}
            </div>

            {/* Derivation Path Selection */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Blockchain Network *
              </label>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                {DERIVATION_PATHS.map((option) => (
                  <label
                    key={option.path}
                    className={`cursor-pointer border rounded-lg p-4 transition-all ${
                      newWallet.derivationPath === option.path
                        ? 'border-purple-500 bg-purple-50 ring-2 ring-purple-200'
                        : 'border-gray-200 hover:border-gray-300'
                    }`}
                  >
                    <input
                      type="radio"
                      name="derivationPath"
                      value={option.path}
                      checked={newWallet.derivationPath === option.path}
                      onChange={(e) => handleDerivationPathChange(e.target.value)}
                      className="sr-only"
                    />
                    <div className="flex items-center space-x-3">
                      <div className={`w-4 h-4 rounded-full border-2 flex items-center justify-center ${
                        newWallet.derivationPath === option.path
                          ? 'border-purple-500'
                          : 'border-gray-300'
                      }`}>
                        {newWallet.derivationPath === option.path && (
                          <div className="w-2 h-2 bg-purple-500 rounded-full" />
                        )}
                      </div>
                      <div className="flex-1">
                        <h5 className="font-medium text-gray-900">{option.label}</h5>
                        <p className="text-xs text-gray-500 mt-1">{option.description}</p>
                        <p className="text-xs font-mono text-gray-400 mt-1">{option.path}</p>
                      </div>
                    </div>
                  </label>
                ))}
              </div>
            </div>

            {/* Seed Phrase Input */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Seed Phrase (12-24 words) *
              </label>
              <div className="relative">
                <textarea
                  rows={4}
                  value={newWallet.seedPhrase}
                  onChange={(e) => setNewWallet({ ...newWallet, seedPhrase: e.target.value })}
                  placeholder="abandon ability able about above absent absorb abstract absurd abuse access accident..."
                  className={`w-full px-4 py-3 border rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500 resize-none font-mono text-sm transition-all ${
                    validationErrors.seedPhrase ? 'border-red-500 bg-red-50' : 'border-gray-300 bg-white'
                  } ${showSeedPhrase ? '' : 'blur-sm'}`}
                />
                <button
                  type="button"
                  onClick={() => setShowSeedPhrase(!showSeedPhrase)}
                  className="absolute top-3 right-3 text-gray-400 hover:text-gray-600 transition-colors"
                >
                  {showSeedPhrase ? '👁️' : '👁️‍🗨️'}
                </button>
              </div>
              {validationErrors.seedPhrase && (
                <p className="text-red-600 text-sm mt-1">{validationErrors.seedPhrase}</p>
              )}
              <p className="text-xs text-gray-500 mt-2">
                <strong>Security:</strong> Your seed phrase is encrypted and stored locally. 
                Make sure you have it backed up securely.
              </p>
            </div>
          </div>

          <div className="mt-8 flex space-x-3">
            <button
              onClick={handleAddWallet}
              className="px-6 py-3 bg-gradient-to-r from-purple-600 to-blue-600 text-white rounded-lg hover:from-purple-700 hover:to-blue-700 transition-colors font-medium"
            >
              Add Wallet
            </button>
            <button
              onClick={() => {
                setIsAddingWallet(false)
                setValidationErrors({})
                setNewWallet({
                  name: '',
                  seedPhrase: '',
                  derivationPath: "m/44'/60'/0'/0/0",
                  chainType: 'ethereum'
                })
                setShowSeedPhrase(false)
              }}
              className="px-6 py-3 bg-gray-300 text-gray-700 rounded-lg hover:bg-gray-400 transition-colors"
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {/* Configured Wallets */}
      {wallets.length > 0 && (
        <div className="space-y-4">
          <h4 className="font-medium text-gray-900">Connected Wallets ({wallets.length})</h4>
          
          {wallets.map((wallet) => (
            <div key={wallet.id} className="bg-white border border-gray-200 rounded-xl p-6 shadow-sm">
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center space-x-3 mb-3">
                    <h4 className="font-medium text-gray-900">{wallet.name}</h4>
                    <span className={`px-3 py-1 rounded-full text-xs font-medium ${
                      wallet.isConnected 
                        ? 'bg-green-100 text-green-800'
                        : 'bg-yellow-100 text-yellow-800'
                    }`}>
                      {wallet.isConnected ? '✅ Connected' : '⚠️ Not Tested'}
                    </span>
                    <span className="px-2 py-1 bg-gray-100 text-gray-700 text-xs rounded-full font-mono">
                      {wallet.chainType.toUpperCase()}
                    </span>
                  </div>
                  
                  <div className="space-y-2 text-sm text-gray-600">
                    <p>📍 Address: <span className="font-mono">{wallet.address}</span></p>
                    <p>🛤️ Path: <span className="font-mono">{wallet.derivationPath}</span></p>
                    <p>📅 Added: {new Date(wallet.createdAt).toLocaleDateString()}</p>
                    {wallet.lastChecked && (
                      <p>🔍 Last checked: {new Date(wallet.lastChecked).toLocaleString()}</p>
                    )}
                  </div>

                  {/* Balances */}
                  {wallet.isConnected && wallet.balances.length > 0 && (
                    <div className="mt-4 p-3 bg-gray-50 rounded-lg">
                      <h5 className="font-medium text-gray-700 mb-2">Token Balances</h5>
                      <div className="space-y-2">
                        {wallet.balances.map((balance) => (
                          <div key={balance.symbol} className="flex justify-between items-center">
                            <span className="text-gray-600">{balance.symbol}:</span>
                            <div className="text-right">
                              <div className="font-mono text-gray-900">{walletService.formatBalance(balance.balance, balance.decimals)}</div>
                              <div className="text-xs text-gray-500">{walletService.formatUSD(balance.balanceUSD)}</div>
                            </div>
                          </div>
                        ))}
                        <div className="border-t pt-2 mt-2">
                          <div className="flex justify-between font-medium">
                            <span className="text-gray-700">Total:</span>
                            <span className="text-gray-900">{walletService.formatUSD(wallet.balances.reduce((sum, b) => sum + b.balanceUSD, 0))}</span>
                          </div>
                        </div>
                      </div>
                    </div>
                  )}

                  {/* Test Results */}
                  {testResults[wallet.id] && (
                    <div className={`mt-4 p-3 rounded-lg text-sm ${
                      testResults[wallet.id].success 
                        ? 'bg-green-50 text-green-800 border border-green-200'
                        : 'bg-red-50 text-red-800 border border-red-200'
                    }`}>
                      {testResults[wallet.id].message}
                    </div>
                  )}
                </div>

                <div className="flex space-x-2 ml-4">
                  <button
                    onClick={() => handleTestWallet(wallet)}
                    disabled={testingWallet === wallet.id}
                    className="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed text-sm"
                  >
                    {testingWallet === wallet.id ? '⏳ Testing...' : '🧪 Test'}
                  </button>
                  
                  <button
                    onClick={() => handleDeleteWallet(wallet.id)}
                    className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors text-sm"
                  >
                    🗑️ Delete
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Usage Instructions */}
      {wallets.length > 0 && (
        <div className="bg-gradient-to-br from-blue-50 to-indigo-50 border border-blue-200 rounded-xl p-6">
          <h4 className="font-medium text-blue-900 mb-3">💡 Using Wallets in Workflows</h4>
          <div className="text-sm text-blue-800 space-y-2">
            <p>Your configured wallets can now be used in automation workflows:</p>
            <ul className="space-y-1 list-disc list-inside ml-4">
              <li><strong>DeFi Strategies:</strong> Automated yield farming, liquidity provision, arbitrage</li>
              <li><strong>Portfolio Rebalancing:</strong> Maintain target asset allocations automatically</li>
              <li><strong>Stop-Loss/Take-Profit:</strong> Automated risk management based on price conditions</li>
              <li><strong>Cross-Chain Operations:</strong> Bridge assets and execute multi-chain strategies</li>
            </ul>
            <p className="mt-3 p-3 bg-blue-100/50 rounded-lg">
              <strong>⚡ Pro Tip:</strong> Use template variables like <code>{'{{wallet.eth_balance}}'}</code> in your workflows 
              to make decisions based on current balances.
            </p>
          </div>
        </div>
      )}
    </div>
  )
}

export default WalletConfiguration