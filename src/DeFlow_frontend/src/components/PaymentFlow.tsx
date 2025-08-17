import React, { useState } from 'react';
import PaymentMethodSelector from './PaymentMethodSelector';
import { PaymentService, PaymentMethod, PaymentPurpose } from '../services/paymentService';

interface PaymentFlowProps {
  amountUsd: number;
  purpose: PaymentPurpose;
  onPaymentComplete: (paymentId: string) => void;
  onCancel: () => void;
}

type PaymentStep = 'select' | 'pay' | 'confirm' | 'complete';

interface PaymentRequest {
  id: string;
  amount_usd: number;
  destination_address: string;
  payment_method: PaymentMethod;
  expires_at: bigint;
}

const PaymentFlow: React.FC<PaymentFlowProps> = ({
  amountUsd,
  purpose,
  onPaymentComplete,
  onCancel
}) => {
  const [currentStep, setCurrentStep] = useState<PaymentStep>('select');
  const [selectedMethod, setSelectedMethod] = useState<PaymentMethod | null>(null);
  const [paymentRequest, setPaymentRequest] = useState<PaymentRequest | null>(null);
  const [txHash, setTxHash] = useState('');
  const [confirming, setConfirming] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handlePaymentMethodSelected = (method: PaymentMethod) => {
    setSelectedMethod(method);
    setError(null);
  };

  const handlePaymentCreated = (request: PaymentRequest) => {
    setPaymentRequest(request);
    setCurrentStep('pay');
  };

  const handlePaymentSent = () => {
    setCurrentStep('confirm');
  };

  const handleConfirmPayment = async () => {
    if (!paymentRequest || !txHash.trim()) {
      setError('Please enter the transaction hash');
      return;
    }

    try {
      setConfirming(true);
      setError(null);

      await PaymentService.confirmPayment(paymentRequest.id, txHash.trim());
      
      setCurrentStep('complete');
      setTimeout(() => {
        onPaymentComplete(paymentRequest.id);
      }, 2000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to confirm payment');
    } finally {
      setConfirming(false);
    }
  };

  const formatTimeRemaining = (expiresAt: bigint) => {
    const now = BigInt(Date.now() * 1000000);
    const remaining = Number(expiresAt - now) / 1000000000; // Convert to seconds
    
    if (remaining <= 0) return 'Expired';
    
    const hours = Math.floor(remaining / 3600);
    const minutes = Math.floor((remaining % 3600) / 60);
    
    if (hours > 0) {
      return `${hours}h ${minutes}m remaining`;
    }
    return `${minutes}m remaining`;
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  const getPurposeDescription = (purpose: PaymentPurpose): string => {
    if ('Subscription' in purpose && purpose.Subscription) {
      return `${purpose.Subscription.plan} Subscription (${purpose.Subscription.duration_months} months)`;
    }
    if ('WorkflowExecution' in purpose && purpose.WorkflowExecution) {
      return `Workflow Execution (${purpose.WorkflowExecution.workflow_id})`;
    }
    if ('PremiumFeatures' in purpose && purpose.PremiumFeatures) {
      return `Premium Features: ${purpose.PremiumFeatures.features.join(', ')}`;
    }
    if ('TopUp' in purpose && purpose.TopUp) {
      return `Account Top-up (${purpose.TopUp.credits} credits)`;
    }
    return 'Payment';
  };

  const renderStepIndicator = () => {
    const steps = [
      { id: 'select', label: 'Select Method', icon: '💳' },
      { id: 'pay', label: 'Send Payment', icon: '💰' },
      { id: 'confirm', label: 'Confirm', icon: '✅' },
      { id: 'complete', label: 'Complete', icon: '🎉' }
    ];

    const currentIndex = steps.findIndex(step => step.id === currentStep);

    return (
      <div className="mb-8">
        <div className="flex items-center justify-between">
          {steps.map((step, index) => (
            <div key={step.id} className="flex items-center">
              <div className={`flex items-center justify-center w-10 h-10 rounded-full border-2 ${
                index <= currentIndex 
                  ? 'bg-blue-600 border-blue-600 text-white' 
                  : 'border-gray-600 text-gray-400'
              }`}>
                <span className="text-sm">{step.icon}</span>
              </div>
              <span className={`ml-2 text-sm ${
                index <= currentIndex ? 'text-white' : 'text-gray-400'
              }`}>
                {step.label}
              </span>
              {index < steps.length - 1 && (
                <div className={`mx-4 h-0.5 w-16 ${
                  index < currentIndex ? 'bg-blue-600' : 'bg-gray-600'
                }`} />
              )}
            </div>
          ))}
        </div>
      </div>
    );
  };

  const renderSelectStep = () => (
    <PaymentMethodSelector
      amountUsd={amountUsd}
      purpose={purpose}
      onPaymentMethodSelected={handlePaymentMethodSelected}
      onPaymentCreated={handlePaymentCreated}
    />
  );

  const renderPayStep = () => {
    if (!paymentRequest || !selectedMethod) return null;

    const details = PaymentService.calculatePaymentDetails(selectedMethod, amountUsd);

    return (
      <div className="bg-gray-800 rounded-lg p-6">
        <h3 className="text-white text-lg font-medium mb-4">
          Send Your Payment
        </h3>

        {/* Payment Summary */}
        <div className="bg-gray-700 rounded-lg p-4 mb-6">
          <h4 className="text-white font-medium mb-3">Payment Summary</h4>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-400">Purpose:</span>
              <span className="text-white">{getPurposeDescription(purpose)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Amount:</span>
              <span className="text-white">${amountUsd.toFixed(2)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Processing Fee:</span>
              <span className="text-white">${details.feeAmount.toFixed(2)}</span>
            </div>
            <div className="flex justify-between border-t border-gray-600 pt-2">
              <span className="text-white font-medium">Total:</span>
              <span className="text-white font-medium">${details.totalAmount.toFixed(2)}</span>
            </div>
          </div>
        </div>

        {/* Payment Instructions */}
        <div className="bg-blue-900/20 border border-blue-500 rounded-lg p-4 mb-6">
          <h4 className="text-blue-400 font-medium mb-3">
            Send {selectedMethod.asset} on {selectedMethod.chain}
          </h4>
          
          <div className="space-y-3">
            <div>
              <label className="block text-gray-400 text-xs mb-1">Destination Address:</label>
              <div className="flex items-center bg-gray-700 rounded px-3 py-2">
                <code className="text-white text-sm flex-1 font-mono">
                  {paymentRequest.destination_address}
                </code>
                <button
                  onClick={() => copyToClipboard(paymentRequest.destination_address)}
                  className="ml-2 text-blue-400 hover:text-blue-300 text-xs"
                >
                  Copy
                </button>
              </div>
            </div>

            <div>
              <label className="block text-gray-400 text-xs mb-1">Amount to Send:</label>
              <div className="flex items-center bg-gray-700 rounded px-3 py-2">
                <code className="text-white text-sm flex-1 font-mono">
                  {details.totalAmount.toFixed(6)} {selectedMethod.asset}
                </code>
                <button
                  onClick={() => copyToClipboard(details.totalAmount.toFixed(6))}
                  className="ml-2 text-blue-400 hover:text-blue-300 text-xs"
                >
                  Copy
                </button>
              </div>
            </div>

            {selectedMethod.token_address && (
              <div>
                <label className="block text-gray-400 text-xs mb-1">Token Contract:</label>
                <div className="flex items-center bg-gray-700 rounded px-3 py-2">
                  <code className="text-white text-sm flex-1 font-mono">
                    {selectedMethod.token_address}
                  </code>
                  <button
                    onClick={() => copyToClipboard(selectedMethod.token_address!)}
                    className="ml-2 text-blue-400 hover:text-blue-300 text-xs"
                  >
                    Copy
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Timer */}
        <div className="bg-yellow-900/20 border border-yellow-500 rounded-lg p-3 mb-6">
          <div className="flex items-center">
            <span className="text-yellow-400 mr-2">⏰</span>
            <span className="text-yellow-300 text-sm">
              {formatTimeRemaining(paymentRequest.expires_at)}
            </span>
          </div>
        </div>

        {/* Instructions */}
        <div className="mb-6">
          <ul className="text-gray-300 text-sm space-y-2">
            {PaymentService.getPaymentInstructions(selectedMethod).map((instruction, index) => (
              <li key={index} className="flex items-start">
                <span className="text-blue-400 mr-2">•</span>
                {instruction}
              </li>
            ))}
          </ul>
        </div>

        <div className="flex space-x-3">
          <button
            onClick={() => setCurrentStep('select')}
            className="flex-1 bg-gray-600 text-white py-3 px-4 rounded-lg font-medium hover:bg-gray-700"
          >
            Back
          </button>
          <button
            onClick={handlePaymentSent}
            className="flex-1 bg-blue-600 text-white py-3 px-4 rounded-lg font-medium hover:bg-blue-700"
          >
            I've Sent the Payment
          </button>
        </div>
      </div>
    );
  };

  const renderConfirmStep = () => (
    <div className="bg-gray-800 rounded-lg p-6">
      <h3 className="text-white text-lg font-medium mb-4">
        Confirm Your Payment
      </h3>

      <p className="text-gray-400 mb-6">
        Please enter the transaction hash from your wallet to confirm the payment.
      </p>

      <div className="mb-6">
        <label className="block text-gray-300 text-sm font-medium mb-2">
          Transaction Hash
        </label>
        <input
          type="text"
          value={txHash}
          onChange={(e) => setTxHash(e.target.value)}
          placeholder="0x..."
          className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
        />
        <p className="text-gray-400 text-xs mt-1">
          Find this in your wallet after sending the transaction
        </p>
      </div>

      {error && (
        <div className="mb-4 bg-red-900/20 border border-red-500 rounded-lg p-3">
          <p className="text-red-300 text-sm">{error}</p>
        </div>
      )}

      <div className="flex space-x-3">
        <button
          onClick={() => setCurrentStep('pay')}
          className="flex-1 bg-gray-600 text-white py-3 px-4 rounded-lg font-medium hover:bg-gray-700"
        >
          Back
        </button>
        <button
          onClick={handleConfirmPayment}
          disabled={!txHash.trim() || confirming}
          className="flex-1 bg-blue-600 text-white py-3 px-4 rounded-lg font-medium hover:bg-blue-700 disabled:opacity-50"
        >
          {confirming ? (
            <div className="flex items-center justify-center">
              <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
              Confirming...
            </div>
          ) : (
            'Confirm Payment'
          )}
        </button>
      </div>
    </div>
  );

  const renderCompleteStep = () => (
    <div className="bg-gray-800 rounded-lg p-6 text-center">
      <div className="text-6xl mb-4">🎉</div>
      <h3 className="text-white text-xl font-medium mb-2">
        Payment Successful!
      </h3>
      <p className="text-gray-400 mb-6">
        Your payment has been confirmed and processed. You'll receive a confirmation email shortly.
      </p>
      
      {paymentRequest && (
        <div className="bg-green-900/20 border border-green-500 rounded-lg p-4 mb-6">
          <p className="text-green-300 text-sm">
            Payment ID: {paymentRequest.id}
          </p>
        </div>
      )}

      <button
        onClick={() => onPaymentComplete(paymentRequest?.id || '')}
        className="bg-blue-600 text-white py-3 px-6 rounded-lg font-medium hover:bg-blue-700"
      >
        Continue
      </button>
    </div>
  );

  return (
    <div className="max-w-2xl mx-auto">
      <div className="mb-6">
        <button
          onClick={onCancel}
          className="text-gray-600 hover:text-gray-800 text-sm mb-4"
        >
          ← Cancel Payment
        </button>
        <h2 className="text-gray-900 text-2xl font-bold mb-2">
          Complete Your Payment
        </h2>
        <p className="text-gray-600">
          {getPurposeDescription(purpose)} - ${amountUsd.toFixed(2)}
        </p>
      </div>

      {renderStepIndicator()}

      {currentStep === 'select' && renderSelectStep()}
      {currentStep === 'pay' && renderPayStep()}
      {currentStep === 'confirm' && renderConfirmStep()}
      {currentStep === 'complete' && renderCompleteStep()}
    </div>
  );
};

export default PaymentFlow;