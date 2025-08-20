// Import BigInt polyfill FIRST to prevent conversion errors
import '../utils/bigint-polyfill'

import React, { useState } from 'react'
import { useEnhancedAuth } from '../contexts/EnhancedAuthContext'
import { useNavigate } from 'react-router-dom'
import localCacheService from '../services/localCacheService'
import PaymentFlow from '../components/PaymentFlow'
import { PaymentPurpose } from '../services/paymentService'

const PaymentPage = () => {
  const [selectedPlan, setSelectedPlan] = useState<'standard' | 'premium' | 'pro'>('premium')
  const [isProcessing, setIsProcessing] = useState(false)
  const [showPayment, setShowPayment] = useState(false)
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

  const getPaymentPurpose = (): PaymentPurpose => {
    return {
      Subscription: {
        plan: currentPlan.title,
        duration_months: 1,
      }
    };
  };

  const getPaymentAmount = (): number => {
    return parseFloat(currentPlan.price);
  };

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

    // Handle Premium and Pro plans with payment flow
    setShowPayment(true)
  }

  const handlePaymentComplete = (paymentId: string) => {
    console.log('Payment completed:', paymentId);
    localCacheService.addNotification({
      id: `payment_success_${Date.now()}`,
      title: `${currentPlan.title} Plan Activated!`,
      message: `Your ${currentPlan.title} subscription is now active. Payment ID: ${paymentId}`,
      type: 'success',
      createdAt: Date.now(),
      read: false
    })
    setShowPayment(false)
    navigate('/')
  }

  const handleCancelPayment = () => {
    setShowPayment(false)
  }

  if (showPayment) {
    return (
      <div className="min-h-screen bg-white py-8 px-4">
        <PaymentFlow
          amountUsd={getPaymentAmount()}
          purpose={getPaymentPurpose()}
          onPaymentComplete={handlePaymentComplete}
          onCancel={handleCancelPayment}
        />
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-4xl mx-auto">
        <div className="text-center mb-12">
          <h1 className="text-4xl font-bold text-gray-900 mb-4">
            Choose Your Plan
          </h1>
          <p className="text-xl text-gray-600 max-w-2xl mx-auto">
            Unlock powerful DeFi automation with reduced fees and premium features
          </p>
        </div>

        <div className="bg-white rounded-2xl shadow-xl overflow-hidden">
          <div className="md:flex">
            {/* Plan Selection */}
            <div className="md:w-1/3 bg-gray-50 p-8">
              <h2 className="text-2xl font-bold text-gray-900 mb-6">Select Plan</h2>
              <div className="space-y-4">
                {Object.entries(plans).map(([key, plan]) => (
                  <button
                    key={key}
                    onClick={() => setSelectedPlan(key as 'standard' | 'premium' | 'pro')}
                    className={`w-full text-left p-4 rounded-lg border-2 transition-all ${
                      selectedPlan === key
                        ? 'border-blue-500 bg-blue-50'
                        : 'border-gray-200 hover:border-gray-300'
                    }`}
                  >
                    <div className="flex justify-between items-start mb-2">
                      <div>
                        <h3 className="font-bold text-gray-900">{plan.title}</h3>
                        <p className="text-sm text-gray-600">{plan.subtitle}</p>
                      </div>
                      <div className="text-right">
                        <span className="text-2xl font-bold text-gray-900">
                          {plan.price === '0' ? 'Free' : `$${plan.price}`}
                        </span>
                        {plan.period !== 'forever' && (
                          <span className="text-sm text-gray-600">/{plan.period}</span>
                        )}
                      </div>
                    </div>
                    {plan.savings && (
                      <span className="inline-block bg-green-100 text-green-800 text-xs px-2 py-1 rounded-full">
                        Save {plan.savings}
                      </span>
                    )}
                  </button>
                ))}
              </div>
            </div>

            {/* Plan Details */}
            <div className="md:w-2/3 p-8">
              <div className="mb-8">
                <h2 className="text-3xl font-bold text-gray-900 mb-2">
                  {currentPlan.title}
                </h2>
                <p className="text-gray-600 mb-4">{currentPlan.subtitle}</p>
                
                <div className="flex items-baseline mb-6">
                  <span className="text-5xl font-bold text-gray-900">
                    {currentPlan.price === '0' ? 'Free' : `$${currentPlan.price}`}
                  </span>
                  {currentPlan.period !== 'forever' && (
                    <span className="text-xl text-gray-600 ml-2">/{currentPlan.period}</span>
                  )}
                  {currentPlan.savings && (
                    <span className="ml-4 bg-green-100 text-green-800 text-sm px-3 py-1 rounded-full">
                      Save {currentPlan.savings}
                    </span>
                  )}
                </div>

                <div className="grid md:grid-cols-2 gap-6 mb-8">
                  <div>
                    <h4 className="font-semibold text-gray-900 mb-2">Transaction Fee</h4>
                    <p className="text-3xl font-bold text-blue-600">{currentPlan.feeRate}</p>
                    {currentPlan.breakEven && (
                      <p className="text-sm text-gray-600">Break-even: {currentPlan.breakEven}/month</p>
                    )}
                  </div>
                  <div>
                    <h4 className="font-semibold text-gray-900 mb-2">Best For</h4>
                    <p className="text-gray-600">{currentPlan.targetUsers}</p>
                  </div>
                </div>

                <div className="mb-8">
                  <h4 className="font-semibold text-gray-900 mb-4">Features</h4>
                  <ul className="space-y-2">
                    {currentPlan.features.map((feature, index) => (
                      <li key={index} className="flex items-center">
                        <svg className="h-5 w-5 text-green-500 mr-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                        </svg>
                        <span className="text-gray-700">{feature}</span>
                      </li>
                    ))}
                  </ul>
                </div>

                {/* Payment Methods Preview for Paid Plans */}
                {selectedPlan !== 'standard' && (
                  <div className="mb-8 p-4 bg-blue-50 rounded-lg border border-blue-200">
                    <h4 className="font-semibold text-gray-900 mb-3">💳 Accepted Payment Methods</h4>
                    <div className="grid grid-cols-2 gap-3">
                      <div className="flex items-center p-2 bg-white rounded border">
                        <span className="text-lg mr-2">🟣</span>
                        <div>
                          <div className="text-sm font-medium">Polygon</div>
                          <div className="text-xs text-gray-600">USDC/USDT • 0.75% fee</div>
                        </div>
                      </div>
                      <div className="flex items-center p-2 bg-white rounded border">
                        <span className="text-lg mr-2">🔵</span>
                        <div>
                          <div className="text-sm font-medium">Arbitrum</div>
                          <div className="text-xs text-gray-600">USDC • 0.5% fee</div>
                        </div>
                      </div>
                      <div className="flex items-center p-2 bg-white rounded border">
                        <span className="text-lg mr-2">🔷</span>
                        <div>
                          <div className="text-sm font-medium">Base</div>
                          <div className="text-xs text-gray-600">USDC • 0.5% fee</div>
                        </div>
                      </div>
                      <div className="flex items-center p-2 bg-white rounded border">
                        <span className="text-lg mr-2">💎</span>
                        <div>
                          <div className="text-sm font-medium">Ethereum</div>
                          <div className="text-xs text-gray-600">USDC/USDT • 1% fee</div>
                        </div>
                      </div>
                    </div>
                    <p className="text-xs text-gray-600 mt-2">
                      Fast, secure stablecoin payments. No credit card required.
                    </p>
                  </div>
                )}

                <button
                  onClick={handleSubscribe}
                  disabled={isProcessing}
                  className={`w-full py-4 px-6 rounded-lg font-semibold text-lg transition-colors ${
                    selectedPlan === 'standard'
                      ? 'bg-gray-600 hover:bg-gray-700 text-white'
                      : 'bg-blue-600 hover:bg-blue-700 text-white'
                  } disabled:opacity-50 disabled:cursor-not-allowed`}
                >
                  {isProcessing ? (
                    <div className="flex items-center justify-center">
                      <svg className="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                        <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                        <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                      </svg>
                      Processing...
                    </div>
                  ) : selectedPlan === 'standard' ? (
                    'Activate Standard Plan'
                  ) : (
                    `Subscribe to ${currentPlan.title} - $${currentPlan.price}/month`
                  )}
                </button>
              </div>
            </div>
          </div>
        </div>

        <div className="mt-8 text-center text-gray-600">
          <p className="text-sm">
            Questions about pricing? <a href="mailto:support@deflow.ai" className="text-blue-600 hover:underline">Contact our team</a>
          </p>
        </div>
      </div>
    </div>
  )
}

export default PaymentPage