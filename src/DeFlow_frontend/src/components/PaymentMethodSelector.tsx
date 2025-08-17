import React, { useState, useEffect } from 'react';
import { PaymentService, PaymentMethod, PaymentPurpose } from '../services/paymentService';

interface PaymentMethodSelectorProps {
  amountUsd: number;
  purpose: PaymentPurpose;
  onPaymentMethodSelected: (method: PaymentMethod) => void;
  onPaymentCreated: (paymentRequest: any) => void;
}

const PaymentMethodSelector: React.FC<PaymentMethodSelectorProps> = ({
  amountUsd,
  purpose,
  onPaymentMethodSelected,
  onPaymentCreated
}) => {
  const [paymentMethods, setPaymentMethods] = useState<PaymentMethod[]>([]);
  const [selectedMethod, setSelectedMethod] = useState<PaymentMethod | null>(null);
  const [senderAddress, setSenderAddress] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);

  useEffect(() => {
    loadPaymentMethods();
  }, []);

  const loadPaymentMethods = async () => {
    try {
      setLoading(true);
      setError(null);
      const methods = await PaymentService.getSupportedPaymentMethods();
      
      // Filter methods that support the requested amount
      const eligibleMethods = methods.filter(
        method => method.enabled && 
                 amountUsd >= method.min_amount_usd && 
                 amountUsd <= method.max_amount_usd
      );
      
      setPaymentMethods(eligibleMethods);

      // Auto-select recommended method
      const recommended = await PaymentService.getRecommendedPaymentMethod(amountUsd);
      if (recommended && eligibleMethods.find(m => m.id === recommended.id)) {
        setSelectedMethod(recommended);
        onPaymentMethodSelected(recommended);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load payment methods');
    } finally {
      setLoading(false);
    }
  };

  const handleMethodSelection = (method: PaymentMethod) => {
    setSelectedMethod(method);
    onPaymentMethodSelected(method);
  };

  const createPaymentRequest = async () => {
    if (!selectedMethod || !senderAddress.trim()) {
      setError('Please select a payment method and enter your wallet address');
      return;
    }

    try {
      setCreating(true);
      setError(null);

      const paymentRequest = await PaymentService.createPaymentRequest(
        selectedMethod.id,
        amountUsd,
        purpose,
        senderAddress.trim()
      );

      onPaymentCreated(paymentRequest);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create payment request');
    } finally {
      setCreating(false);
    }
  };

  const getChainIcon = (chain: string) => {
    switch (chain) {
      case 'Polygon': return '🟣';
      case 'Arbitrum': return '🔵';
      case 'Base': return '🔷';
      case 'Ethereum': return '💎';
      default: return '⚪';
    }
  };

  const getAssetIcon = (asset: string) => {
    switch (asset) {
      case 'USDC': return '💵';
      case 'USDT': return '💰';
      default: return '💳';
    }
  };

  if (loading) {
    return (
      <div className="bg-white rounded-lg p-6 border border-gray-200">
        <div className="animate-pulse">
          <div className="h-6 bg-gray-300 rounded mb-4"></div>
          <div className="space-y-3">
            <div className="h-16 bg-gray-300 rounded"></div>
            <div className="h-16 bg-gray-300 rounded"></div>
            <div className="h-16 bg-gray-300 rounded"></div>
          </div>
        </div>
      </div>
    );
  }

  if (error && paymentMethods.length === 0) {
    return (
      <div className="bg-red-50 border border-red-300 rounded-lg p-6">
        <h3 className="text-red-800 font-medium">Error Loading Payment Methods</h3>
        <p className="text-red-600 mt-2">{error}</p>
        <button 
          onClick={loadPaymentMethods}
          className="mt-4 bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700"
        >
          Retry
        </button>
      </div>
    );
  }

  if (paymentMethods.length === 0) {
    return (
      <div className="bg-yellow-50 border border-yellow-300 rounded-lg p-6">
        <h3 className="text-yellow-800 font-medium">No Payment Methods Available</h3>
        <p className="text-yellow-600 mt-2">
          No payment methods support the amount ${amountUsd.toFixed(2)}. 
          Please try a different amount.
        </p>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg p-6 border border-gray-200">
      <h3 className="text-gray-900 text-lg font-medium mb-4">
        Select Payment Method
      </h3>
      <p className="text-gray-600 mb-6">
        Choose how you'd like to pay ${amountUsd.toFixed(2)} USD
      </p>

      {/* Payment Method Options */}
      <div className="space-y-3 mb-6">
        {paymentMethods.map((method) => {
          const details = PaymentService.calculatePaymentDetails(method, amountUsd);
          const isSelected = selectedMethod?.id === method.id;
          
          return (
            <div
              key={method.id}
              onClick={() => handleMethodSelection(method)}
              className={`border rounded-lg p-4 cursor-pointer transition-all ${
                isSelected 
                  ? 'border-blue-500 bg-blue-50' 
                  : 'border-gray-300 bg-white hover:border-gray-400'
              }`}
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-3">
                  <span className="text-2xl">
                    {getChainIcon(method.chain)} {getAssetIcon(method.asset)}
                  </span>
                  <div>
                    <div className="text-gray-900 font-medium">
                      {method.asset} on {method.chain}
                    </div>
                    <div className="text-gray-600 text-sm">
                      Fee: {details.feePercentage}% • ~{Math.round(details.estimatedTime / 60)} min
                    </div>
                  </div>
                </div>
                <div className="text-right">
                  <div className="text-gray-900 font-medium">
                    ${details.totalAmount.toFixed(2)}
                  </div>
                  <div className="text-gray-600 text-sm">
                    +${details.feeAmount.toFixed(2)} fee
                  </div>
                </div>
              </div>
              
              {isSelected && (
                <div className="mt-3 pt-3 border-t border-gray-300">
                  <div className="text-sm text-gray-700">
                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <span className="text-gray-600">Confirmations:</span>
                        <span className="ml-2 text-gray-900">{method.confirmation_blocks}</span>
                      </div>
                      <div>
                        <span className="text-gray-600">Settlement:</span>
                        <span className="ml-2 text-gray-900">{Math.round(method.estimated_settlement_time / 60)} min</span>
                      </div>
                    </div>
                  </div>
                </div>
              )}
            </div>
          );
        })}
      </div>

      {/* Wallet Address Input */}
      {selectedMethod && (
        <div className="mb-6">
          <label className="block text-gray-700 text-sm font-medium mb-2">
            Your {selectedMethod.chain} Wallet Address
          </label>
          <input
            type="text"
            value={senderAddress}
            onChange={(e) => setSenderAddress(e.target.value)}
            placeholder={`Enter your ${selectedMethod.chain} wallet address`}
            className="w-full bg-white border border-gray-300 rounded-lg px-3 py-2 text-gray-900 placeholder-gray-500 focus:outline-none focus:border-blue-500"
          />
          <p className="text-gray-600 text-xs mt-1">
            This helps us track your payment and provide support if needed
          </p>
        </div>
      )}

      {/* Payment Instructions Preview */}
      {selectedMethod && (
        <div className="mb-6 bg-blue-50 rounded-lg p-4 border border-blue-200">
          <h4 className="text-gray-900 font-medium mb-2">Payment Instructions</h4>
          <ul className="text-gray-700 text-sm space-y-1">
            {PaymentService.getPaymentInstructions(selectedMethod).map((instruction, index) => (
              <li key={index} className="flex items-start">
                <span className="text-blue-600 mr-2">•</span>
                {instruction}
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Error Display */}
      {error && (
        <div className="mb-4 bg-red-50 border border-red-300 rounded-lg p-3">
          <p className="text-red-700 text-sm">{error}</p>
        </div>
      )}

      {/* Create Payment Button */}
      <button
        onClick={createPaymentRequest}
        disabled={!selectedMethod || !senderAddress.trim() || creating}
        className="w-full bg-blue-600 text-white py-3 px-4 rounded-lg font-medium hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
      >
        {creating ? (
          <div className="flex items-center justify-center">
            <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
            Creating Payment Request...
          </div>
        ) : (
          `Create Payment Request - $${selectedMethod ? PaymentService.calculatePaymentDetails(selectedMethod, amountUsd).totalAmount.toFixed(2) : amountUsd.toFixed(2)}`
        )}
      </button>
    </div>
  );
};

export default PaymentMethodSelector;