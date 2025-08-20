import React, { useState } from 'react';
import PaymentFlow from '../components/PaymentFlow';
import { PaymentPurpose } from '../services/paymentService';

interface SubscriptionPlan {
  id: string;
  name: string;
  description: string;
  features: string[];
  monthlyPrice: number;
  yearlyPrice: number;
  popular?: boolean;
}

const SubscriptionPayment: React.FC = () => {
  const [selectedPlan, setSelectedPlan] = useState<SubscriptionPlan | null>(null);
  const [billingCycle, setBillingCycle] = useState<'monthly' | 'yearly'>('monthly');
  const [showPayment, setShowPayment] = useState(false);

  const subscriptionPlans: SubscriptionPlan[] = [
    {
      id: 'starter',
      name: 'Starter',
      description: 'Perfect for individuals getting started with DeFi workflows',
      features: [
        'Up to 10 workflows per month',
        'Basic DeFi integrations',
        'Email support',
        'Standard execution speed',
        '1 GB workflow storage'
      ],
      monthlyPrice: 19,
      yearlyPrice: 190, // ~17/month
    },
    {
      id: 'professional',
      name: 'Professional',
      description: 'For serious traders and DeFi professionals',
      features: [
        'Up to 100 workflows per month',
        'Advanced DeFi integrations',
        'Priority support',
        'Fast execution speed',
        '10 GB workflow storage',
        'Custom strategies',
        'Portfolio analytics'
      ],
      monthlyPrice: 49,
      yearlyPrice: 490, // ~41/month
      popular: true,
    },
    {
      id: 'enterprise',
      name: 'Enterprise',
      description: 'For teams and high-volume trading operations',
      features: [
        'Unlimited workflows',
        'All DeFi integrations',
        '24/7 dedicated support',
        'Ultra-fast execution',
        'Unlimited storage',
        'Custom integrations',
        'Team collaboration',
        'Advanced analytics',
        'API access'
      ],
      monthlyPrice: 199,
      yearlyPrice: 1990, // ~166/month
    },
  ];

  const handlePlanSelection = (plan: SubscriptionPlan) => {
    setSelectedPlan(plan);
  };

  const handleProceedToPayment = () => {
    if (!selectedPlan) return;
    setShowPayment(true);
  };

  const handlePaymentComplete = (paymentId: string) => {
    console.log('Payment completed:', paymentId);
    // Redirect to dashboard or show success message
    alert('Subscription activated successfully!');
    setShowPayment(false);
    setSelectedPlan(null);
  };

  const handleCancelPayment = () => {
    setShowPayment(false);
  };

  const getPaymentPurpose = (): PaymentPurpose => {
    if (!selectedPlan) throw new Error('No plan selected');

    return {
      Subscription: {
        plan: selectedPlan.name,
        duration_months: billingCycle === 'yearly' ? 12 : 1,
      }
    };
  };

  const getPaymentAmount = (): number => {
    if (!selectedPlan) return 0;
    return billingCycle === 'yearly' ? selectedPlan.yearlyPrice : selectedPlan.monthlyPrice;
  };

  const getSavings = (plan: SubscriptionPlan): number => {
    const yearlyMonthly = plan.yearlyPrice / 12;
    return Math.round(((plan.monthlyPrice - yearlyMonthly) / plan.monthlyPrice) * 100);
  };

  if (showPayment && selectedPlan) {
    return (
      <div className="min-h-screen bg-gray-900 py-8 px-4">
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
    <div className="min-h-screen bg-gray-900 py-12 px-4">
      <div className="max-w-6xl mx-auto">
        {/* Header */}
        <div className="text-center mb-12">
          <h1 className="text-4xl font-bold text-white mb-4">
            Choose Your DeFlow Plan
          </h1>
          <p className="text-xl text-gray-400 mb-8">
            Unlock the full potential of automated DeFi workflows
          </p>

          {/* Billing Toggle */}
          <div className="inline-flex bg-gray-800 rounded-lg p-1">
            <button
              onClick={() => setBillingCycle('monthly')}
              className={`px-6 py-2 rounded-md text-sm font-medium transition-all ${
                billingCycle === 'monthly'
                  ? 'bg-blue-600 text-white'
                  : 'text-gray-400 hover:text-white'
              }`}
            >
              Monthly
            </button>
            <button
              onClick={() => setBillingCycle('yearly')}
              className={`px-6 py-2 rounded-md text-sm font-medium transition-all ${
                billingCycle === 'yearly'
                  ? 'bg-blue-600 text-white'
                  : 'text-gray-400 hover:text-white'
              }`}
            >
              Yearly
              <span className="ml-2 bg-green-600 text-white text-xs px-2 py-1 rounded">
                Save up to 20%
              </span>
            </button>
          </div>
        </div>

        {/* Pricing Cards */}
        <div className="grid md:grid-cols-3 gap-8 mb-12">
          {subscriptionPlans.map((plan) => {
            const isSelected = selectedPlan?.id === plan.id;
            const price = billingCycle === 'yearly' ? plan.yearlyPrice : plan.monthlyPrice;
            const monthlyEquivalent = billingCycle === 'yearly' ? plan.yearlyPrice / 12 : plan.monthlyPrice;

            return (
              <div
                key={plan.id}
                className={`relative bg-gray-800 rounded-lg p-6 border-2 transition-all cursor-pointer ${
                  isSelected
                    ? 'border-blue-500 bg-blue-900/20'
                    : 'border-gray-700 hover:border-gray-600'
                } ${plan.popular ? 'ring-2 ring-blue-500' : ''}`}
                onClick={() => handlePlanSelection(plan)}
              >
                {plan.popular && (
                  <div className="absolute -top-3 left-1/2 transform -translate-x-1/2">
                    <span className="bg-blue-600 text-white text-xs font-bold px-3 py-1 rounded-full">
                      MOST POPULAR
                    </span>
                  </div>
                )}

                <div className="text-center mb-6">
                  <h3 className="text-xl font-bold text-white mb-2">{plan.name}</h3>
                  <p className="text-gray-400 text-sm mb-4">{plan.description}</p>
                  
                  <div className="mb-2">
                    <span className="text-4xl font-bold text-white">${price}</span>
                    <span className="text-gray-400">/{billingCycle === 'yearly' ? 'year' : 'month'}</span>
                  </div>
                  
                  {billingCycle === 'yearly' && (
                    <div className="text-sm">
                      <span className="text-gray-400">~${monthlyEquivalent.toFixed(0)}/month</span>
                      <span className="ml-2 text-green-400 font-medium">
                        Save {getSavings(plan)}%
                      </span>
                    </div>
                  )}
                </div>

                {/* Features */}
                <ul className="space-y-3 mb-6">
                  {plan.features.map((feature, index) => (
                    <li key={index} className="flex items-start">
                      <span className="text-blue-400 mr-3 mt-0.5">✓</span>
                      <span className="text-gray-300 text-sm">{feature}</span>
                    </li>
                  ))}
                </ul>

                {/* Selection Indicator */}
                {isSelected && (
                  <div className="absolute top-4 right-4">
                    <div className="w-6 h-6 bg-blue-600 rounded-full flex items-center justify-center">
                      <span className="text-white text-xs">✓</span>
                    </div>
                  </div>
                )}
              </div>
            );
          })}
        </div>

        {/* Payment Methods Preview */}
        <div className="bg-gray-800 rounded-lg p-6 mb-8">
          <h3 className="text-white text-lg font-medium mb-4">💳 Accepted Payment Methods</h3>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="bg-green-900/30 border border-green-500 rounded-lg p-4 text-center">
              <div className="flex items-center justify-center mb-2">
                <span className="text-3xl mr-2">🟣</span>
                <div>
                  <div className="text-white font-medium">Polygon</div>
                  <div className="text-green-400 text-sm font-medium">0.75% Fee</div>
                </div>
              </div>
              <div className="space-y-1">
                <div className="bg-blue-600 text-white text-xs px-2 py-1 rounded">
                  💵 USDC
                </div>
                <div className="bg-yellow-600 text-white text-xs px-2 py-1 rounded">
                  💰 USDT
                </div>
              </div>
              <div className="text-gray-300 text-xs mt-2">~5 min settlement</div>
            </div>
            
            <div className="bg-green-900/30 border border-green-500 rounded-lg p-4 text-center">
              <div className="flex items-center justify-center mb-2">
                <span className="text-3xl mr-2">🔵</span>
                <div>
                  <div className="text-white font-medium">Arbitrum</div>
                  <div className="text-green-400 text-sm font-medium">0.5% Fee</div>
                </div>
              </div>
              <div>
                <div className="bg-blue-600 text-white text-xs px-2 py-1 rounded">
                  💵 USDC
                </div>
              </div>
              <div className="text-gray-300 text-xs mt-2">~1 min settlement</div>
            </div>
            
            <div className="bg-green-900/30 border border-green-500 rounded-lg p-4 text-center">
              <div className="flex items-center justify-center mb-2">
                <span className="text-3xl mr-2">🔷</span>
                <div>
                  <div className="text-white font-medium">Base</div>
                  <div className="text-green-400 text-sm font-medium">0.5% Fee</div>
                </div>
              </div>
              <div>
                <div className="bg-blue-600 text-white text-xs px-2 py-1 rounded">
                  💵 USDC
                </div>
              </div>
              <div className="text-gray-300 text-xs mt-2">~1 min settlement</div>
            </div>
            
            <div className="bg-green-900/30 border border-green-500 rounded-lg p-4 text-center">
              <div className="flex items-center justify-center mb-2">
                <span className="text-3xl mr-2">💎</span>
                <div>
                  <div className="text-white font-medium">Ethereum</div>
                  <div className="text-green-400 text-sm font-medium">1% Fee</div>
                </div>
              </div>
              <div className="space-y-1">
                <div className="bg-blue-600 text-white text-xs px-2 py-1 rounded">
                  💵 USDC
                </div>
                <div className="bg-yellow-600 text-white text-xs px-2 py-1 rounded">
                  💰 USDT
                </div>
              </div>
              <div className="text-gray-300 text-xs mt-2">~15 min settlement</div>
            </div>
          </div>
          <div className="mt-6 grid md:grid-cols-3 gap-4 text-sm">
            <div className="bg-blue-900/30 border border-blue-500 rounded-lg p-3 text-center">
              <div className="text-blue-400 font-medium mb-1">🔒 Secure</div>
              <div className="text-gray-300">Blockchain-based payments</div>
            </div>
            <div className="bg-green-900/30 border border-green-500 rounded-lg p-3 text-center">
              <div className="text-green-400 font-medium mb-1">⚡ Fast</div>
              <div className="text-gray-300">1-15 minute settlements</div>
            </div>
            <div className="bg-purple-900/30 border border-purple-500 rounded-lg p-3 text-center">
              <div className="text-purple-400 font-medium mb-1">💰 Low Fees</div>
              <div className="text-gray-300">0.5-1% processing fee</div>
            </div>
          </div>
        </div>

        {/* Proceed Button */}
        <div className="text-center">
          <button
            onClick={handleProceedToPayment}
            disabled={!selectedPlan}
            className="bg-blue-600 text-white px-8 py-4 rounded-lg font-medium text-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {selectedPlan 
              ? `Proceed with ${selectedPlan.name} Plan - $${getPaymentAmount()}`
              : 'Select a Plan'
            }
          </button>
          
          {selectedPlan && (
            <p className="text-gray-400 text-sm mt-3">
              You'll be guided through a secure stablecoin payment process
            </p>
          )}
        </div>

        {/* Additional Info */}
        <div className="mt-16 text-center">
          <div className="bg-blue-900/20 border border-blue-500 rounded-lg p-6">
            <div className="text-3xl mb-3">🚀</div>
            <h3 className="text-white text-xl font-medium mb-3">Ready to get started?</h3>
            <p className="text-gray-300 mb-4">
              Choose your plan above and pay with USDC or USDT on your preferred network. 
              Fast, secure, and no credit card required.
            </p>
            <div className="text-sm text-gray-400">
              Questions? Contact us at support@deflow.io
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default SubscriptionPayment;