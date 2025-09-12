/**
 * Spending Limits Summary Component
 * Shows current spending limits and approval status for a wallet
 */

import React, { useState, useEffect } from 'react'
import spendingApprovalService from '../services/spendingApprovalService'

interface Props {
  walletAddress: string
  onEditLimits: () => void
}

export default function SpendingLimitsSummary({ walletAddress, onEditLimits }: Props) {
  const [approvals, setApprovals] = useState<any[]>([])
  const [remainingLimits, setRemainingLimits] = useState<Record<string, {daily: number, total: number}>>({})
  const [analytics, setAnalytics] = useState<any>(null)

  useEffect(() => {
    loadApprovalData()
  }, [walletAddress])

  const loadApprovalData = async () => {
    try {
      await spendingApprovalService.initialize(walletAddress)
      
      const activeApprovals = spendingApprovalService.getActiveApprovals()
      const limits = spendingApprovalService.getRemainingLimits()
      const stats = spendingApprovalService.getSpendingAnalytics()
      
      setApprovals(activeApprovals)
      setRemainingLimits(limits)
      setAnalytics(stats)
    } catch (error) {
      console.error('Failed to load approval data:', error)
    }
  }

  const getTotalApprovedValue = () => {
    return approvals.reduce((total, approval) => {
      return total + parseFloat(approval.approvedAmount)
    }, 0)
  }

  const getStatusColor = (remaining: number, total: number) => {
    const percentage = total > 0 ? remaining / total : 0
    if (percentage > 0.7) return 'text-green-600'
    if (percentage > 0.3) return 'text-yellow-600'
    return 'text-red-600'
  }

  const getStatusIcon = (remaining: number, total: number) => {
    const percentage = total > 0 ? remaining / total : 0
    if (percentage > 0.7) return '✅'
    if (percentage > 0.3) return '⚠️'
    return '🔴'
  }

  if (approvals.length === 0) {
    return (
      <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center">
            <span className="text-yellow-600 mr-2">⚠️</span>
            <div>
              <p className="text-sm font-medium text-yellow-800">No spending limits set</p>
              <p className="text-xs text-yellow-600">Set spending limits to enable DeFi automation</p>
            </div>
          </div>
          <button
            onClick={onEditLimits}
            className="px-3 py-1 bg-yellow-600 text-white text-xs rounded hover:bg-yellow-700 transition-colors"
          >
            Set Limits
          </button>
        </div>
      </div>
    )
  }

  return (
    <div className="bg-gray-50 border border-gray-200 rounded-lg p-4 space-y-3">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center">
          <span className="text-blue-600 mr-2">🔒</span>
          <div>
            <p className="text-sm font-medium text-gray-900">Spending Limits Active</p>
            <p className="text-xs text-gray-500">
              {approvals.length} token{approvals.length !== 1 ? 's' : ''} • 
              ${getTotalApprovedValue().toLocaleString()} total approved
            </p>
          </div>
        </div>
        <button
          onClick={onEditLimits}
          className="px-3 py-1 bg-gray-600 text-white text-xs rounded hover:bg-gray-700 transition-colors"
        >
          Manage
        </button>
      </div>

      {/* Token Limits */}
      <div className="grid grid-cols-2 gap-3">
        {approvals.slice(0, 4).map(approval => {
          const limits = remainingLimits[approval.symbol]
          if (!limits) return null
          
          return (
            <div key={approval.symbol} className="bg-white rounded-lg p-3 border border-gray-100">
              <div className="flex items-center justify-between mb-2">
                <div className="flex items-center">
                  <div className="w-6 h-6 bg-gradient-to-br from-blue-400 to-purple-500 rounded-full flex items-center justify-center text-white text-xs font-bold mr-2">
                    {approval.symbol.slice(0, 2)}
                  </div>
                  <span className="text-sm font-medium">{approval.symbol}</span>
                </div>
                <span className="text-xs">
                  {getStatusIcon(limits.daily, parseFloat(approval.dailyLimit))}
                </span>
              </div>
              
              <div className="space-y-1">
                <div className="flex justify-between text-xs">
                  <span className="text-gray-500">Daily:</span>
                  <span className={getStatusColor(limits.daily, parseFloat(approval.dailyLimit))}>
                    ${limits.daily.toLocaleString()} / ${parseFloat(approval.dailyLimit).toLocaleString()}
                  </span>
                </div>
                <div className="flex justify-between text-xs">
                  <span className="text-gray-500">Total:</span>
                  <span className={getStatusColor(limits.total, parseFloat(approval.approvedAmount))}>
                    ${limits.total.toLocaleString()}
                  </span>
                </div>
              </div>
              
              {/* Progress bar */}
              <div className="mt-2">
                <div className="w-full bg-gray-200 rounded-full h-1">
                  <div 
                    className={`h-1 rounded-full ${
                      limits.daily / parseFloat(approval.dailyLimit) > 0.7 
                        ? 'bg-green-500' 
                        : limits.daily / parseFloat(approval.dailyLimit) > 0.3 
                        ? 'bg-yellow-500' 
                        : 'bg-red-500'
                    }`}
                    style={{ 
                      width: `${Math.max(5, (limits.daily / parseFloat(approval.dailyLimit)) * 100)}%` 
                    }}
                  ></div>
                </div>
              </div>
            </div>
          )
        })}
      </div>

      {/* Show more indicator */}
      {approvals.length > 4 && (
        <div className="text-center">
          <button
            onClick={onEditLimits}
            className="text-xs text-blue-600 hover:text-blue-800 font-medium"
          >
            +{approvals.length - 4} more token{approvals.length - 4 !== 1 ? 's' : ''}
          </button>
        </div>
      )}

      {/* Quick Stats */}
      {analytics && (
        <div className="flex justify-between text-xs text-gray-500 pt-2 border-t">
          <span>Operations: {analytics.operationsCount}</span>
          <span>Spent: ${analytics.totalSpent.toLocaleString()}</span>
        </div>
      )}
    </div>
  )
}