// Import BigInt polyfill FIRST to prevent conversion errors
import '../utils/bigint-polyfill'

import React, { useState } from 'react'
import { useEnhancedAuth } from '../contexts/EnhancedAuthContext'
import { useNavigate } from 'react-router-dom'
import localCacheService from '../services/localCacheService'

const PaymentPage = () => {
  const [selectedPlan, setSelectedPlan] = useState<'standard' | 'premium' | 'pro'>('premium')
  const [isProcessing, setIsProcessing] = useState(false)
  const auth = useEnhancedAuth()
  const navigate = useNavigate()

  const plans = {
    standard: {
      price: '0',
      period: 'forever',
      savings: null,
      title: 'Standard',
      subtitle: 'Get started with DeFlow',
      feeRate: '0.85%',
      breakEven: null,
      targetUsers: 'New users, light traders, trial usage',
      features: [
        'Basic workflow automation',
        'Community support',
        'Standard execution speed',
        '0.85% transaction fees'
      ]
    },
    premium: {
      price: '19',
      period: 'month',
      savings: '70%',
      title: 'Premium',
      subtitle: 'Most popular choice',
      feeRate: '0.25%',
      breakEven: '$3,167',
      targetUsers: 'Active DeFi users, moderate volume',
      features: [
        'All Standard features',
        'Priority execution queue',
        'Email support (24h response)',
        'Basic analytics dashboard',
        '0.25% transaction fees (70% savings!)',
        'Break-even at $3,167/month volume'
      ]
    },
    pro: {
      price: '149',
      period: 'month',
      savings: '88%',
      title: 'Pro',
      subtitle: 'For professional traders',
      feeRate: '0.1%',
      breakEven: '$19,867',
      targetUsers: 'Professional traders, funds, API users',
      features: [
        'All Premium features',
        'Full API access',
        'Custom strategy development',
        'Portfolio insurance options',
        'Priority phone support',
        'Advanced risk management tools',
        '0.1% transaction fees (88% savings!)',
        'Break-even at $19,867/month volume'
      ]
    }
  }

  const currentPlan = plans[selectedPlan]

  const handleSubscribe = async () => {
    // Handle Standard (free) plan
    if (selectedPlan === 'standard') {
      if (!auth.isAuthenticated) {
        localCacheService.addNotification({
          id: `standard_guest_${Date.now()}`,
          title: 'Standard Plan Active',
          message: 'You are using the Standard plan (0.85% fees). Login to enable cross-device sync.',
          type: 'info',
          createdAt: Date.now(),
          read: false
        })
      } else {
        localCacheService.addNotification({
          id: `standard_success_${Date.now()}`,
          title: 'Standard Plan Active',
          message: 'You are using the Standard plan (0.85% fees) with cross-device sync enabled.',
          type: 'success',
          createdAt: Date.now(),
          read: false
        })
      }
      navigate('/')
      return
    }

    // Handle Premium plans
    if (!auth.isAuthenticated) {
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
      // Simulate payment processing with realistic delay
      await new Promise(resolve => setTimeout(resolve, 3000))
      
      // Simulate payment success (90% success rate for demo)
      if (Math.random() > 0.1) {
        // Payment successful
        localCacheService.addNotification({
          id: `subscription_success_${Date.now()}`,
          title: 'Subscription Activated! 🎉',
          message: `Premium ${selectedPlan} plan is now active. Enjoy reduced fees and premium features!`,
          type: 'success',
          createdAt: Date.now(),
          read: false
        })
        
        // TODO: In production, implement:
        // 1. Process payment through Stripe/PayPal/ICP canister
        // 2. Store subscription in backend canister
        // 3. Update user's premium status in auth context
        // 4. Set up recurring billing
        
        console.log(`✅ Subscribed to ${selectedPlan} plan for $${currentPlan.price}/${currentPlan.period}`)
        
        // Redirect to dashboard after successful subscription
        setTimeout(() => {
          navigate('/')
        }, 2000)
        
      } else {
        // Simulate payment failure
        throw new Error('Payment processing failed')
      }
      
    } catch (error) {
      console.error('Payment failed:', error)
      
      localCacheService.addNotification({
        id: `subscription_error_${Date.now()}`,
        title: 'Payment Failed ❌',
        message: 'There was an issue processing your payment. Please check your payment method and try again.',
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
        <h1 className="text-3xl font-bold text-gray-900 mb-4">Choose Your DeFlow Plan</h1>
        <p className="text-lg text-gray-600 max-w-3xl mx-auto">
          Start free and upgrade when it makes financial sense. Our transparent break-even points ensure you always pay optimally.
        </p>
        <div className="mt-4 text-sm text-gray-500">
          💡 Strong subscription incentive: Free tier at 0.85% encourages upgrades while subscriptions offer massive 70-88% savings!
        </div>
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
      <div className="grid lg:grid-cols-3 md:grid-cols-1 gap-8 mb-8 max-w-5xl mx-auto">
        {/* Standard Plan */}
        <div 
          className={`border-2 rounded-xl p-6 cursor-pointer transition-all ${
            selectedPlan === 'standard' 
              ? 'border-gray-500 bg-gray-50 shadow-lg' 
              : 'border-gray-200 hover:border-gray-400'
          }`}
          onClick={() => setSelectedPlan('standard')}
        >
          <div className="flex items-center justify-between mb-4">
            <div>
              <h3 className="text-xl font-semibold text-gray-700">{plans.standard.title}</h3>
              <p className="text-sm text-gray-600">{plans.standard.subtitle}</p>
            </div>
            <div className={`w-6 h-6 rounded-full border-2 ${
              selectedPlan === 'standard' 
                ? 'border-gray-500 bg-gray-500' 
                : 'border-gray-300'
            }`}>
              {selectedPlan === 'standard' && (
                <div className="w-2 h-2 bg-white rounded-full mx-auto mt-1"></div>
              )}
            </div>
          </div>
          
          <div className="mb-4">
            <span className="text-3xl font-bold text-gray-800">Free</span>
            <div className="text-sm text-orange-600 font-medium mt-1">
              {plans.standard.feeRate} transaction fees
            </div>
          </div>
          
          <div className="text-sm text-gray-600 mb-4">
            {plans.standard.targetUsers}
          </div>
          
          <ul className="space-y-2 text-sm">
            {plans.standard.features.map((feature, index) => (
              <li key={index} className="flex items-start space-x-2">
                <span className="text-green-500 mt-0.5">✓</span>
                <span>{feature}</span>
              </li>
            ))}
          </ul>
        </div>

        {/* Premium Plan */}
        <div 
          className={`border-2 rounded-xl p-6 cursor-pointer transition-all relative ${
            selectedPlan === 'premium' 
              ? 'border-purple-500 bg-purple-50 shadow-lg' 
              : 'border-gray-200 hover:border-purple-300'
          }`}
          onClick={() => setSelectedPlan('premium')}
        >
          {/* Most Popular Badge */}
          <div className="absolute -top-3 left-1/2 transform -translate-x-1/2">
            <span className="bg-gradient-to-r from-purple-600 to-blue-600 text-white px-3 py-1 rounded-full text-xs font-medium">
              Most Popular
            </span>
          </div>
          
          <div className="flex items-center justify-between mb-4">
            <div>
              <h3 className="text-xl font-semibold text-purple-700">{plans.premium.title}</h3>
              <p className="text-sm text-gray-600">{plans.premium.subtitle}</p>
            </div>
            <div className={`w-6 h-6 rounded-full border-2 ${
              selectedPlan === 'premium' 
                ? 'border-purple-500 bg-purple-500' 
                : 'border-gray-300'
            }`}>
              {selectedPlan === 'premium' && (
                <div className="w-2 h-2 bg-white rounded-full mx-auto mt-1"></div>
              )}
            </div>
          </div>
          
          <div className="mb-4">
            <span className="text-3xl font-bold text-purple-700">${plans.premium.price}</span>
            <span className="text-gray-600 text-lg">/month</span>
            <div className="text-sm text-green-600 font-medium mt-1">
              {plans.premium.feeRate} fees ({plans.premium.savings} savings!)
            </div>
          </div>
          
          <div className="text-sm text-gray-600 mb-4">
            {plans.premium.targetUsers}
          </div>
          
          <ul className="space-y-2 text-sm">
            {plans.premium.features.map((feature, index) => (
              <li key={index} className="flex items-start space-x-2">
                <span className="text-green-500 mt-0.5">✓</span>
                <span>{feature}</span>
              </li>
            ))}
          </ul>
        </div>

        {/* Pro Plan */}
        <div 
          className={`border-2 rounded-xl p-6 cursor-pointer transition-all ${
            selectedPlan === 'pro' 
              ? 'border-yellow-500 bg-yellow-50 shadow-lg' 
              : 'border-gray-200 hover:border-yellow-400'
          }`}
          onClick={() => setSelectedPlan('pro')}
        >
          <div className="flex items-center justify-between mb-4">
            <div>
              <h3 className="text-xl font-semibold text-yellow-700">{plans.pro.title}</h3>
              <p className="text-sm text-gray-600">{plans.pro.subtitle}</p>
            </div>
            <div className={`w-6 h-6 rounded-full border-2 ${
              selectedPlan === 'pro' 
                ? 'border-yellow-500 bg-yellow-500' 
                : 'border-gray-300'
            }`}>
              {selectedPlan === 'pro' && (
                <div className="w-2 h-2 bg-white rounded-full mx-auto mt-1"></div>
              )}
            </div>
          </div>
          
          <div className="mb-4">
            <span className="text-3xl font-bold text-yellow-700">${plans.pro.price}</span>
            <span className="text-gray-600 text-lg">/month</span>
            <div className="text-sm text-green-600 font-medium mt-1">
              {plans.pro.feeRate} fees ({plans.pro.savings} savings!)
            </div>
          </div>
          
          <div className="text-sm text-gray-600 mb-4">
            {plans.pro.targetUsers}
          </div>
          
          <ul className="space-y-2 text-sm">
            {plans.pro.features.map((feature, index) => (
              <li key={index} className="flex items-start space-x-2">
                <span className="text-green-500 mt-0.5">✓</span>
                <span>{feature}</span>
              </li>
            ))}
          </ul>
        </div>
      </div>

      {/* Team and Enterprise Contact */}
      <div className="text-center mb-8">
        <div className="bg-gradient-to-r from-blue-50 to-purple-50 rounded-lg p-6 border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-800 mb-2">Need Team or Enterprise Solutions?</h3>
          <p className="text-gray-600 mb-4">
            Looking for team collaboration, white-label deployment, or custom enterprise features? 
            We offer specialized plans for teams (5+ members) and institutions with ultra-low 0.02-0.08% transaction fees.
          </p>
          <button className="bg-gradient-to-r from-blue-600 to-purple-600 text-white px-6 py-2 rounded-lg hover:from-blue-700 hover:to-purple-700 transition-colors">
            Contact Sales Team
          </button>
          <div className="text-sm text-gray-500 mt-2">
            Custom pricing available • Volume discounts • Dedicated support
          </div>
        </div>
      </div>

      {/* Payment Section - Only show for paid plans */}
      {selectedPlan !== 'standard' && (
        <div className="bg-white border border-gray-200 rounded-xl p-6 mb-8">
          <h3 className="text-lg font-semibold mb-4">Payment Summary</h3>
          
          <div className="flex justify-between items-center py-2">
            <span>DeFlow {currentPlan.title}</span>
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
      )}

      {/* Subscribe Button */}
      <div className="text-center">
        <button
          onClick={handleSubscribe}
          disabled={isProcessing}
          className={`w-full max-w-md py-4 px-8 rounded-lg font-semibold text-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed ${
            selectedPlan === 'standard' 
              ? 'bg-gradient-to-r from-gray-600 to-blue-600 hover:from-gray-700 hover:to-blue-700 text-white'
              : 'bg-gradient-to-r from-purple-600 to-blue-600 hover:from-purple-700 hover:to-blue-700 text-white'
          }`}
        >
          {isProcessing ? (
            <span className="flex items-center justify-center space-x-2">
              <span className="animate-spin">⏳</span>
              <span>Processing...</span>
            </span>
          ) : selectedPlan === 'standard' ? (
            'Continue with Standard Plan'
          ) : auth.isAuthenticated ? (
            `Subscribe for $${currentPlan.price}/${currentPlan.period}`
          ) : (
            'Login to Subscribe'
          )}
        </button>
        
        <p className="text-sm text-gray-500 mt-3">
          {selectedPlan === 'standard' 
            ? 'Free forever. Upgrade anytime to unlock massive fee savings (70-88% off!)'
            : auth.isAuthenticated 
              ? 'Cancel anytime. No long-term commitments. Immediate fee savings start now!' 
              : 'Login required for paid plans. Save up to 88% on transaction fees!'}
        </p>
      </div>

      {/* Fee Savings Comparison */}
      <div className="mt-12 bg-gradient-to-br from-purple-50 to-blue-50 rounded-xl p-6">
        <h3 className="text-lg font-semibold mb-6 text-center">Transaction Fee Comparison</h3>
        
        <div className="grid md:grid-cols-3 gap-6 mb-6 max-w-4xl mx-auto">
          <div className="text-center p-6 bg-white rounded-lg shadow-sm">
            <div className="text-xl font-bold text-gray-600">Standard</div>
            <div className="text-4xl font-bold text-orange-600 my-3">0.85%</div>
            <div className="text-sm text-gray-500 mb-2">$850 fees on $100K</div>
            <div className="text-xs text-orange-600 font-medium">Strong upgrade incentive!</div>
          </div>
          <div className="text-center p-6 bg-white rounded-lg border-2 border-purple-500 shadow-sm">
            <div className="text-xl font-bold text-purple-600">Premium</div>
            <div className="text-4xl font-bold text-purple-600 my-3">0.25%</div>
            <div className="text-sm text-green-600 mb-2">$250 fees on $100K</div>
            <div className="text-xs font-bold text-green-600">Save $600!</div>
          </div>
          <div className="text-center p-6 bg-white rounded-lg shadow-sm">
            <div className="text-xl font-bold text-yellow-600">Pro</div>
            <div className="text-4xl font-bold text-yellow-600 my-3">0.1%</div>
            <div className="text-sm text-green-600 mb-2">$100 fees on $100K</div>
            <div className="text-xs font-bold text-green-600">Save $750!</div>
          </div>
        </div>
        
        <div className="text-center">
          <div className="text-sm text-gray-700 mb-2 font-medium">
            💡 Clean 3-tier structure: Any subscription saves massive amounts vs 0.85% standard tier!
          </div>
          <div className="text-sm text-gray-600">
            Break-even points: Premium at $3,167/month • Pro at $19,867/month
          </div>
        </div>
      </div>

      {/* Key Benefits */}
      <div className="mt-8 bg-gradient-to-br from-gray-50 to-white rounded-xl p-6">
        <h3 className="text-lg font-semibold mb-4 text-center">Why Choose DeFlow Premium?</h3>
        
        <div className="grid md:grid-cols-4 gap-4">
          <div className="text-center">
            <div className="text-3xl mb-2">💰</div>
            <div className="font-medium mb-1">Massive Fee Savings</div>
            <div className="text-sm text-gray-600">Save up to 96% on transaction fees</div>
          </div>
          
          <div className="text-center">
            <div className="text-3xl mb-2">⚡</div>
            <div className="font-medium mb-1">Priority Execution</div>
            <div className="text-sm text-gray-600">Your workflows run first in queue</div>
          </div>
          
          <div className="text-center">
            <div className="text-3xl mb-2">🔄</div>
            <div className="font-medium mb-1">Cross-Device Sync</div>
            <div className="text-sm text-gray-600">Access from any device, anywhere</div>
          </div>
          
          <div className="text-center">
            <div className="text-3xl mb-2">🛡️</div>
            <div className="font-medium mb-1">Advanced Features</div>
            <div className="text-sm text-gray-600">API access, custom strategies, insurance</div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default PaymentPage