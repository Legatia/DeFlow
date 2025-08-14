// Import BigInt polyfill FIRST to prevent conversion errors
import '../utils/bigint-polyfill'

import React, { useState } from 'react'
import { useEnhancedAuth } from '../contexts/EnhancedAuthContext'
import localCacheService from '../services/localCacheService'

const PaymentPage = () => {
  const [selectedPlan, setSelectedPlan] = useState<'monthly' | 'yearly'>('monthly')
  const [isProcessing, setIsProcessing] = useState(false)
  const auth = useEnhancedAuth()

  const plans = {
    monthly: {
      price: '9.99',
      period: 'month',
      savings: null,
      features: [
        'Unlimited Workflows',
        'Priority Execution',
        'Cross-Device Sync',
        'Advanced Analytics', 
        '50% Reduced Transaction Fees',
        'Email Support',
        'Export/Import Data'
      ]
    },
    yearly: {
      price: '99.99',
      period: 'year',
      savings: '16%',
      features: [
        'Unlimited Workflows',
        'Priority Execution',
        'Cross-Device Sync',
        'Advanced Analytics',
        '50% Reduced Transaction Fees',
        'Premium Support',
        'Export/Import Data',
        'Beta Features Access',
        'API Access'
      ]
    }
  }

  const currentPlan = plans[selectedPlan]

  const handleSubscribe = async () => {
    if (!auth.isAuthenticated) {
      // User needs to login first
      localCacheService.addNotification({
        id: `login_required_${Date.now()}`,
        title: 'Login Required',
        message: 'Please login with NFID or Internet Identity to subscribe to premium features.',
        type: 'info',
        createdAt: Date.now(),
        read: false
      })
      return
    }

    setIsProcessing(true)
    
    try {
      // Mock payment processing
      await new Promise(resolve => setTimeout(resolve, 2000))
      
      // Add success notification
      localCacheService.addNotification({
        id: `subscription_success_${Date.now()}`,
        title: 'Subscription Activated!',
        message: `Premium ${selectedPlan} plan is now active. Enjoy reduced fees and premium features!`,
        type: 'success',
        createdAt: Date.now(),
        read: false
      })
      
      // In a real app, you would:
      // 1. Process payment through Stripe/PayPal/ICP
      // 2. Update user subscription status in backend
      // 3. Refresh auth context with premium status
      
      console.log(`Subscribed to ${selectedPlan} plan for $${currentPlan.price}`)
      
    } catch (error) {
      console.error('Payment failed:', error)
      
      localCacheService.addNotification({
        id: `subscription_error_${Date.now()}`,
        title: 'Payment Failed',
        message: 'There was an issue processing your payment. Please try again.',
        type: 'error',
        createdAt: Date.now(),
        read: false
      })
    } finally {
      setIsProcessing(false)
    }
  }

  return (
    <div className="max-w-4xl mx-auto py-8">
      {/* Header */}
      <div className="text-center mb-8">
        <h1 className="text-3xl font-bold text-gray-900 mb-4">Upgrade to DeFlow Premium</h1>
        <p className="text-lg text-gray-600 max-w-2xl mx-auto">
          Unlock the full potential of DeFlow with premium features, reduced fees, and priority support.
        </p>
      </div>

      {/* Current Status */}
      {auth.isAuthenticated && (
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-8">
          <div className="flex items-center space-x-3">
            <div className="w-8 h-8 bg-purple-600 rounded-full flex items-center justify-center">
              <span className="text-white font-bold text-sm">G</span>
            </div>
            <div>
              <div className="font-medium text-blue-900">
                Logged in as: {auth.principal?.toString().slice(0, 16)}...
              </div>
              <div className="text-sm text-blue-700">
                Current Status: {auth.userMode === 'authenticated' ? 'Premium User' : 'Guest User'}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Plan Selection */}
      <div className="grid md:grid-cols-2 gap-6 mb-8">
        {/* Monthly Plan */}
        <div 
          className={`border-2 rounded-xl p-6 cursor-pointer transition-all ${
            selectedPlan === 'monthly' 
              ? 'border-purple-500 bg-purple-50 shadow-lg' 
              : 'border-gray-200 hover:border-purple-300'
          }`}
          onClick={() => setSelectedPlan('monthly')}
        >
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-xl font-semibold">Monthly Plan</h3>
            <div className={`w-6 h-6 rounded-full border-2 ${
              selectedPlan === 'monthly' 
                ? 'border-purple-500 bg-purple-500' 
                : 'border-gray-300'
            }`}>
              {selectedPlan === 'monthly' && (
                <div className="w-2 h-2 bg-white rounded-full mx-auto mt-1"></div>
              )}
            </div>
          </div>
          
          <div className="mb-4">
            <span className="text-3xl font-bold">${plans.monthly.price}</span>
            <span className="text-gray-600 ml-1">/month</span>
          </div>
          
          <ul className="space-y-2 text-sm">
            {plans.monthly.features.map((feature, index) => (
              <li key={index} className="flex items-center space-x-2">
                <span className="text-green-500">✓</span>
                <span>{feature}</span>
              </li>
            ))}
          </ul>
        </div>

        {/* Yearly Plan */}
        <div 
          className={`border-2 rounded-xl p-6 cursor-pointer transition-all relative ${
            selectedPlan === 'yearly' 
              ? 'border-purple-500 bg-purple-50 shadow-lg' 
              : 'border-gray-200 hover:border-purple-300'
          }`}
          onClick={() => setSelectedPlan('yearly')}
        >
          {/* Most Popular Badge */}
          <div className="absolute -top-3 left-1/2 transform -translate-x-1/2">
            <span className="bg-gradient-to-r from-purple-600 to-blue-600 text-white px-4 py-1 rounded-full text-xs font-medium">
              Most Popular
            </span>
          </div>
          
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-xl font-semibold">Yearly Plan</h3>
            <div className={`w-6 h-6 rounded-full border-2 ${
              selectedPlan === 'yearly' 
                ? 'border-purple-500 bg-purple-500' 
                : 'border-gray-300'
            }`}>
              {selectedPlan === 'yearly' && (
                <div className="w-2 h-2 bg-white rounded-full mx-auto mt-1"></div>
              )}
            </div>
          </div>
          
          <div className="mb-4">
            <span className="text-3xl font-bold">${plans.yearly.price}</span>
            <span className="text-gray-600 ml-1">/year</span>
            <div className="text-green-600 text-sm font-medium">
              Save {plans.yearly.savings} vs Monthly
            </div>
          </div>
          
          <ul className="space-y-2 text-sm">
            {plans.yearly.features.map((feature, index) => (
              <li key={index} className="flex items-center space-x-2">
                <span className="text-green-500">✓</span>
                <span>{feature}</span>
              </li>
            ))}
          </ul>
        </div>
      </div>

      {/* Payment Section */}
      <div className="bg-white border border-gray-200 rounded-xl p-6 mb-8">
        <h3 className="text-lg font-semibold mb-4">Payment Summary</h3>
        
        <div className="flex justify-between items-center py-2">
          <span>DeFlow Premium ({selectedPlan})</span>
          <span className="font-semibold">${currentPlan.price}</span>
        </div>
        
        <div className="border-t border-gray-200 pt-2 mt-2">
          <div className="flex justify-between items-center font-semibold text-lg">
            <span>Total</span>
            <span>${currentPlan.price}</span>
          </div>
        </div>

        {/* Payment Methods */}
        <div className="mt-6">
          <h4 className="font-medium mb-3">Payment Method</h4>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
            <div className="border border-gray-300 rounded-lg p-3 text-center opacity-50">
              <div className="text-2xl mb-1">💳</div>
              <div className="text-xs">Credit Card</div>
              <div className="text-xs text-gray-500">Coming Soon</div>
            </div>
            <div className="border border-gray-300 rounded-lg p-3 text-center opacity-50">
              <div className="text-2xl mb-1">🪙</div>
              <div className="text-xs">ICP Tokens</div>
              <div className="text-xs text-gray-500">Coming Soon</div>
            </div>
            <div className="border border-gray-300 rounded-lg p-3 text-center opacity-50">
              <div className="text-2xl mb-1">₿</div>
              <div className="text-xs">Bitcoin</div>
              <div className="text-xs text-gray-500">Coming Soon</div>
            </div>
            <div className="border border-gray-300 rounded-lg p-3 text-center opacity-50">
              <div className="text-2xl mb-1">Ξ</div>
              <div className="text-xs">Ethereum</div>
              <div className="text-xs text-gray-500">Coming Soon</div>
            </div>
          </div>
        </div>
      </div>

      {/* Subscribe Button */}
      <div className="text-center">
        <button
          onClick={handleSubscribe}
          disabled={isProcessing}
          className="w-full max-w-md bg-gradient-to-r from-purple-600 to-blue-600 text-white py-4 px-8 rounded-lg font-semibold text-lg hover:from-purple-700 hover:to-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {isProcessing ? (
            <span className="flex items-center justify-center space-x-2">
              <span className="animate-spin">⏳</span>
              <span>Processing...</span>
            </span>
          ) : auth.isAuthenticated ? (
            `Subscribe for $${currentPlan.price}/${currentPlan.period}`
          ) : (
            'Login to Subscribe'
          )}
        </button>
        
        <p className="text-sm text-gray-500 mt-3">
          {auth.isAuthenticated ? 'Cancel anytime. No long-term commitments.' : 'You need to be logged in to subscribe.'}
        </p>
      </div>

      {/* Benefits Summary */}
      <div className="mt-12 bg-gradient-to-br from-purple-50 to-blue-50 rounded-xl p-6">
        <h3 className="text-lg font-semibold mb-4 text-center">Why Choose DeFlow Premium?</h3>
        
        <div className="grid md:grid-cols-3 gap-6">
          <div className="text-center">
            <div className="text-3xl mb-2">⚡</div>
            <div className="font-medium mb-1">Priority Execution</div>
            <div className="text-sm text-gray-600">Your workflows run first in the queue</div>
          </div>
          
          <div className="text-center">
            <div className="text-3xl mb-2">💰</div>
            <div className="font-medium mb-1">Reduced Fees</div>
            <div className="text-sm text-gray-600">Save 50% on all transaction fees</div>
          </div>
          
          <div className="text-center">
            <div className="text-3xl mb-2">🔄</div>
            <div className="font-medium mb-1">Cross-Device Sync</div>
            <div className="text-sm text-gray-600">Access your workflows from any device</div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default PaymentPage