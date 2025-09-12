/**
 * Spending Approval Manager Component
 * Allows users to set specific token spending limits for DeFlow automation
 * Shows after successful seed phrase import
 */

import React, { useState, useEffect } from 'react'
import { useEnhancedAuth } from '../contexts/EnhancedAuthContext'

interface SpendingLimit {
  token: string
  symbol: string
  contractAddress?: string
  maxAmount: string
  dailyLimit: string
  operationsAllowed: AutomationOperation[]
  enabled: boolean
  currentApproval?: string // Current on-chain approval amount
}

interface AutomationOperation {
  type: 'swap' | 'stake' | 'lend' | 'provide_liquidity' | 'yield_farm' | 'rebalance'
  label: string
  description: string
  riskLevel: 'low' | 'medium' | 'high'
}

const AUTOMATION_OPERATIONS: AutomationOperation[] = [
  {
    type: 'swap',
    label: 'Token Swaps',
    description: 'Exchange tokens on DEXes (Uniswap, PancakeSwap, etc.)',
    riskLevel: 'low'
  },
  {
    type: 'stake',
    label: 'Staking',
    description: 'Stake tokens in protocols for rewards',
    riskLevel: 'low'
  },
  {
    type: 'lend',
    label: 'Lending',
    description: 'Lend tokens on Aave, Compound, etc.',
    riskLevel: 'medium'
  },
  {
    type: 'provide_liquidity',
    label: 'Liquidity Provision',
    description: 'Add liquidity to DEX pools',
    riskLevel: 'medium'
  },
  {
    type: 'yield_farm',
    label: 'Yield Farming',
    description: 'Complex yield farming strategies',
    riskLevel: 'high'
  },
  {
    type: 'rebalance',
    label: 'Portfolio Rebalancing',
    description: 'Automatic portfolio rebalancing',
    riskLevel: 'high'
  }
]

interface Props {
  walletAddress: string
  availableTokens: Array<{
    symbol: string
    balance: string
    balanceUSD: number
    contractAddress?: string
    decimals: number
  }>
  onApprovalComplete: (approvals: SpendingLimit[]) => void
  onSkip: () => void
}

export default function SpendingApprovalManager({ 
  walletAddress, 
  availableTokens, 
  onApprovalComplete, 
  onSkip 
}: Props) {
  const { } = useEnhancedAuth()
  const [spendingLimits, setSpendingLimits] = useState<SpendingLimit[]>([])
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [activeTab, setActiveTab] = useState<'setup' | 'advanced' | 'review'>('setup')

  useEffect(() => {
    // Initialize spending limits with user's token balances
    const initialLimits = availableTokens.map(token => ({
      token: token.symbol,
      symbol: token.symbol,
      contractAddress: token.contractAddress,
      maxAmount: '',
      dailyLimit: '',
      operationsAllowed: [] as AutomationOperation[],
      enabled: false,
      currentApproval: '0'
    }))
    setSpendingLimits(initialLimits)
  }, [availableTokens])

  const updateSpendingLimit = (tokenSymbol: string, updates: Partial<SpendingLimit>) => {
    setSpendingLimits(prev => 
      prev.map(limit => 
        limit.symbol === tokenSymbol 
          ? { ...limit, ...updates }
          : limit
      )
    )
  }

  const toggleOperation = (tokenSymbol: string, operation: AutomationOperation) => {
    setSpendingLimits(prev => 
      prev.map(limit => {
        if (limit.symbol !== tokenSymbol) return limit
        
        const hasOperation = limit.operationsAllowed.some(op => op.type === operation.type)
        const newOperations = hasOperation
          ? limit.operationsAllowed.filter(op => op.type !== operation.type)
          : [...limit.operationsAllowed, operation]
        
        return { ...limit, operationsAllowed: newOperations }
      })
    )
  }

  const setQuickPreset = (tokenSymbol: string, preset: 'conservative' | 'moderate' | 'aggressive') => {
    const token = availableTokens.find(t => t.symbol === tokenSymbol)
    if (!token) return

    const presetConfigs = {
      conservative: {
        maxAmount: (token.balanceUSD * 0.1).toString(), // 10% of balance
        dailyLimit: (token.balanceUSD * 0.02).toString(), // 2% daily
        operationsAllowed: AUTOMATION_OPERATIONS.filter(op => op.riskLevel === 'low')
      },
      moderate: {
        maxAmount: (token.balanceUSD * 0.3).toString(), // 30% of balance
        dailyLimit: (token.balanceUSD * 0.05).toString(), // 5% daily
        operationsAllowed: AUTOMATION_OPERATIONS.filter(op => op.riskLevel !== 'high')
      },
      aggressive: {
        maxAmount: (token.balanceUSD * 0.7).toString(), // 70% of balance
        dailyLimit: (token.balanceUSD * 0.1).toString(), // 10% daily
        operationsAllowed: AUTOMATION_OPERATIONS
      }
    }

    updateSpendingLimit(tokenSymbol, {
      ...presetConfigs[preset],
      enabled: true
    })
  }

  const handleSubmit = async () => {
    setIsSubmitting(true)
    try {
      // Filter only enabled approvals
      const enabledApprovals = spendingLimits.filter(limit => limit.enabled)
      
      // Here you would integrate with smart contract approval system
      // For now, we'll simulate the approval process
      console.log('Submitting spending approvals:', enabledApprovals)
      
      // Simulate API call to create on-chain approvals
      await new Promise(resolve => setTimeout(resolve, 2000))
      
      onApprovalComplete(enabledApprovals)
    } catch (error) {
      console.error('Failed to set spending approvals:', error)
    } finally {
      setIsSubmitting(false)
    }
  }

  const getRiskColor = (level: 'low' | 'medium' | 'high') => {
    switch (level) {
      case 'low': return 'text-green-600'
      case 'medium': return 'text-yellow-600'
      case 'high': return 'text-red-600'
    }
  }

  const getTotalUSDApproval = () => {
    return spendingLimits
      .filter(limit => limit.enabled)
      .reduce((total, limit) => {
        const token = availableTokens.find(t => t.symbol === limit.symbol)
        const amount = parseFloat(limit.maxAmount) || 0
        return total + amount
      }, 0)
  }

  return (
    <div className="max-w-4xl mx-auto p-6 bg-white rounded-lg shadow-lg">
      <div className="mb-8">
        <h2 className="text-3xl font-bold text-gray-900 mb-2">
          Set Spending Approvals
        </h2>
        <p className="text-gray-600">
          Choose how much of each token DeFlow can manage for automation. 
          You maintain full control and can revoke permissions anytime.
        </p>
      </div>

      {/* Tab Navigation */}
      <div className="flex space-x-1 mb-6 bg-gray-100 p-1 rounded-lg">
        {[
          { key: 'setup', label: 'Quick Setup', icon: '⚡' },
          { key: 'advanced', label: 'Advanced', icon: '⚙️' },
          { key: 'review', label: 'Review', icon: '👁️' }
        ].map(tab => (
          <button
            key={tab.key}
            onClick={() => setActiveTab(tab.key as any)}
            className={`flex-1 py-2 px-4 rounded-md text-sm font-medium transition-colors ${
              activeTab === tab.key
                ? 'bg-white text-blue-600 shadow-sm'
                : 'text-gray-500 hover:text-gray-700'
            }`}
          >
            <span className="mr-2">{tab.icon}</span>
            {tab.label}
          </button>
        ))}
      </div>

      {/* Quick Setup Tab */}
      {activeTab === 'setup' && (
        <div className="space-y-6">
          <div className="grid gap-6">
            {availableTokens.map(token => (
              <div key={token.symbol} className="border rounded-lg p-4 hover:shadow-md transition-shadow">
                <div className="flex items-center justify-between mb-4">
                  <div className="flex items-center space-x-3">
                    <div className="w-10 h-10 bg-gradient-to-br from-blue-400 to-purple-500 rounded-full flex items-center justify-center text-white font-bold">
                      {token.symbol.slice(0, 2)}
                    </div>
                    <div>
                      <h3 className="font-semibold text-lg">{token.symbol}</h3>
                      <p className="text-gray-500">
                        Balance: {parseFloat(token.balance).toLocaleString()} 
                        (${token.balanceUSD.toLocaleString()})
                      </p>
                    </div>
                  </div>
                  <label className="flex items-center">
                    <input
                      type="checkbox"
                      checked={spendingLimits.find(l => l.symbol === token.symbol)?.enabled || false}
                      onChange={(e) => updateSpendingLimit(token.symbol, { enabled: e.target.checked })}
                      className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                    />
                    <span className="ml-2 text-sm">Enable</span>
                  </label>
                </div>

                {spendingLimits.find(l => l.symbol === token.symbol)?.enabled && (
                  <div className="space-y-4 border-t pt-4">
                    {/* Quick Presets */}
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-2">
                        Quick Presets
                      </label>
                      <div className="flex space-x-2">
                        {['conservative', 'moderate', 'aggressive'].map(preset => (
                          <button
                            key={preset}
                            onClick={() => setQuickPreset(token.symbol, preset as any)}
                            className={`px-3 py-1 rounded text-xs font-medium border transition-colors ${
                              preset === 'conservative' 
                                ? 'border-green-300 text-green-700 hover:bg-green-50'
                                : preset === 'moderate'
                                ? 'border-yellow-300 text-yellow-700 hover:bg-yellow-50'
                                : 'border-red-300 text-red-700 hover:bg-red-50'
                            }`}
                          >
                            {preset.charAt(0).toUpperCase() + preset.slice(1)}
                          </button>
                        ))}
                      </div>
                    </div>

                    {/* Manual Amounts */}
                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-1">
                          Max Total Amount ($)
                        </label>
                        <input
                          type="number"
                          value={spendingLimits.find(l => l.symbol === token.symbol)?.maxAmount || ''}
                          onChange={(e) => updateSpendingLimit(token.symbol, { maxAmount: e.target.value })}
                          className="w-full border border-gray-300 rounded-md px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                          placeholder="0.00"
                        />
                      </div>
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-1">
                          Daily Limit ($)
                        </label>
                        <input
                          type="number"
                          value={spendingLimits.find(l => l.symbol === token.symbol)?.dailyLimit || ''}
                          onChange={(e) => updateSpendingLimit(token.symbol, { dailyLimit: e.target.value })}
                          className="w-full border border-gray-300 rounded-md px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                          placeholder="0.00"
                        />
                      </div>
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Advanced Tab */}
      {activeTab === 'advanced' && (
        <div className="space-y-6">
          {spendingLimits.filter(limit => limit.enabled).map(limit => (
            <div key={limit.symbol} className="border rounded-lg p-4">
              <h3 className="font-semibold text-lg mb-4 flex items-center">
                <div className="w-8 h-8 bg-gradient-to-br from-blue-400 to-purple-500 rounded-full flex items-center justify-center text-white text-xs font-bold mr-3">
                  {limit.symbol.slice(0, 2)}
                </div>
                {limit.symbol} Operations
              </h3>
              
              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                {AUTOMATION_OPERATIONS.map(operation => (
                  <label key={operation.type} className="flex items-start space-x-3 p-3 border rounded-lg hover:bg-gray-50 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={limit.operationsAllowed.some(op => op.type === operation.type)}
                      onChange={() => toggleOperation(limit.symbol, operation)}
                      className="mt-1 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                    />
                    <div className="flex-1">
                      <div className="flex items-center justify-between">
                        <span className="font-medium text-sm">{operation.label}</span>
                        <span className={`text-xs font-medium ${getRiskColor(operation.riskLevel)}`}>
                          {operation.riskLevel.toUpperCase()}
                        </span>
                      </div>
                      <p className="text-xs text-gray-500 mt-1">{operation.description}</p>
                    </div>
                  </label>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Review Tab */}
      {activeTab === 'review' && (
        <div className="space-y-6">
          <div className="bg-gradient-to-r from-blue-50 to-purple-50 border border-blue-200 rounded-lg p-6">
            <h3 className="font-semibold text-lg mb-2">Approval Summary</h3>
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <span className="text-gray-600">Total USD Approved:</span>
                <span className="ml-2 font-semibold text-blue-600">
                  ${getTotalUSDApproval().toLocaleString()}
                </span>
              </div>
              <div>
                <span className="text-gray-600">Active Tokens:</span>
                <span className="ml-2 font-semibold">
                  {spendingLimits.filter(l => l.enabled).length} of {spendingLimits.length}
                </span>
              </div>
            </div>
          </div>

          {spendingLimits.filter(l => l.enabled).map(limit => (
            <div key={limit.symbol} className="border rounded-lg p-4">
              <div className="flex items-center justify-between mb-3">
                <h4 className="font-semibold flex items-center">
                  <div className="w-6 h-6 bg-gradient-to-br from-blue-400 to-purple-500 rounded-full flex items-center justify-center text-white text-xs font-bold mr-2">
                    {limit.symbol.slice(0, 2)}
                  </div>
                  {limit.symbol}
                </h4>
                <div className="text-sm text-gray-500">
                  Max: ${parseFloat(limit.maxAmount || '0').toLocaleString()}
                </div>
              </div>
              
              <div className="grid grid-cols-2 gap-4 text-sm mb-3">
                <div>
                  <span className="text-gray-600">Daily Limit:</span>
                  <span className="ml-2">${parseFloat(limit.dailyLimit || '0').toLocaleString()}</span>
                </div>
                <div>
                  <span className="text-gray-600">Operations:</span>
                  <span className="ml-2">{limit.operationsAllowed.length}</span>
                </div>
              </div>
              
              <div className="flex flex-wrap gap-2">
                {limit.operationsAllowed.map(op => (
                  <span 
                    key={op.type}
                    className={`px-2 py-1 text-xs rounded-full border ${
                      op.riskLevel === 'low' 
                        ? 'bg-green-50 text-green-700 border-green-200'
                        : op.riskLevel === 'medium'
                        ? 'bg-yellow-50 text-yellow-700 border-yellow-200'
                        : 'bg-red-50 text-red-700 border-red-200'
                    }`}
                  >
                    {op.label}
                  </span>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Action Buttons */}
      <div className="flex justify-between items-center pt-8 border-t">
        <button
          onClick={onSkip}
          className="px-6 py-2 text-gray-600 hover:text-gray-800 font-medium transition-colors"
        >
          Skip for now
        </button>
        
        <div className="flex space-x-4">
          {activeTab !== 'setup' && (
            <button
              onClick={() => setActiveTab(activeTab === 'review' ? 'advanced' : 'setup')}
              className="px-6 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors"
            >
              Back
            </button>
          )}
          
          {activeTab !== 'review' ? (
            <button
              onClick={() => setActiveTab(activeTab === 'setup' ? 'advanced' : 'review')}
              className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
              disabled={spendingLimits.filter(l => l.enabled).length === 0}
            >
              Next
            </button>
          ) : (
            <button
              onClick={handleSubmit}
              disabled={isSubmitting || spendingLimits.filter(l => l.enabled).length === 0}
              className="px-6 py-2 bg-gradient-to-r from-blue-600 to-purple-600 text-white rounded-lg hover:from-blue-700 hover:to-purple-700 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isSubmitting ? (
                <span className="flex items-center">
                  <svg className="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                  Setting Approvals...
                </span>
              ) : (
                `Approve ${spendingLimits.filter(l => l.enabled).length} Tokens`
              )}
            </button>
          )}
        </div>
      </div>
    </div>
  )
}