import React, { useState } from 'react';
import simpleDefiTemplateService, { DeFiWorkflowTemplate, StrategyFromTemplateResponse } from '../services/defiTemplateServiceSimple';
import { AuthClient } from '@dfinity/auth-client';

interface StrategyCreationFlowProps {
  template: DeFiWorkflowTemplate;
  onStrategyCreated: (strategy: StrategyFromTemplateResponse) => void;
  onCancel: () => void;
}

const StrategyCreationFlow = ({ template, onStrategyCreated, onCancel }: StrategyCreationFlowProps) => {
  const [step, setStep] = useState(1);
  const [capitalAmount, setCapitalAmount] = useState(template.min_capital_usd);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  // User profile information
  const [userProfile, setUserProfile] = useState({
    riskTolerance: 5,
    investmentGoal: 'growth',
    timeHorizon: 'medium-term',
    experienceLevel: 'beginner'
  });

  const handleCreateStrategy = async () => {
    try {
      setLoading(true);
      setError(null);

      // Get authenticated user ID from Internet Identity
      const authClient = await AuthClient.create();
      const identity = authClient.getIdentity();
      const userId = identity.getPrincipal().toString();
      
      if (identity.getPrincipal().isAnonymous()) {
        throw new Error('Please authenticate with Internet Identity first');
      }
      
      const strategy = await simpleDefiTemplateService.createStrategyFromTemplate(
        template.id,
        userId,
        capitalAmount
      );
      
      onStrategyCreated(strategy);
    } catch (err) {
      console.error('Error creating strategy:', err);
      setError('Failed to create strategy. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const getStepIcon = (stepNumber: number) => {
    if (step > stepNumber) return '✅';
    if (step === stepNumber) return '🎯';
    return '⭕';
  };

  const renderStep = () => {
    switch (step) {
      case 1:
        return (
          <div className="space-y-6">
            <div className="text-center">
              <div className="text-6xl mb-4">
                {simpleDefiTemplateService.getCategoryIcon(template.category || 'unknown')}
              </div>
              <h2 className="text-2xl font-bold text-gray-900 mb-2">
                {template.name}
              </h2>
              <p className="text-gray-600 text-lg mb-4">
                {template.description}
              </p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
              <div className="text-center p-4 bg-green-50 rounded-lg">
                <div className="text-2xl font-bold text-green-600 mb-1">
                  {template.estimated_apy.toFixed(1)}%
                </div>
                <div className="text-sm text-gray-600">Estimated APY</div>
              </div>
              <div className="text-center p-4 bg-blue-50 rounded-lg">
                <div className={`text-2xl font-bold mb-1 ${
                  template.risk_score <= 3 ? 'text-green-600' :
                  template.risk_score <= 6 ? 'text-yellow-600' : 'text-red-600'
                }`}>
                  {template.risk_score}/10
                </div>
                <div className="text-sm text-gray-600">Risk Score</div>
              </div>
              <div className="text-center p-4 bg-purple-50 rounded-lg">
                <div className="text-2xl font-bold text-purple-600 mb-1">
                  {template.difficulty || 'beginner'}
                </div>
                <div className="text-sm text-gray-600">Difficulty</div>
              </div>
            </div>

            <div className="bg-gray-50 rounded-lg p-4">
              <h3 className="font-semibold text-gray-900 mb-2">Strategy Details:</h3>
              <ul className="space-y-1 text-sm text-gray-600">
                <li>• Category: {template.category || 'unknown'}</li>
                <li>• Minimum Capital: ${template.min_capital_usd.toLocaleString()}</li>
                <li>• Automated execution and management</li>
                <li>• Real-time monitoring and alerts</li>
                <li>• Risk management built-in</li>
              </ul>
            </div>

            <div className="flex justify-between">
              <button
                onClick={onCancel}
                className="px-6 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={() => setStep(2)}
                className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
              >
                Continue
              </button>
            </div>
          </div>
        );

      case 2:
        return (
          <div className="space-y-6">
            <div className="text-center">
              <h2 className="text-2xl font-bold text-gray-900 mb-2">
                Set Your Investment Amount
              </h2>
              <p className="text-gray-600">
                Choose how much you want to invest in this strategy
              </p>
            </div>

            <div className="max-w-md mx-auto">
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Investment Amount (USD)
              </label>
              <div className="relative">
                <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                  <span className="text-gray-500 sm:text-sm">$</span>
                </div>
                <input
                  type="number"
                  value={capitalAmount}
                  onChange={(e) => setCapitalAmount(parseFloat(e.target.value) || 0)}
                  className="block w-full pl-7 pr-12 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent text-lg"
                  min={template.min_capital_usd}
                  step="50"
                />
              </div>
              
              {capitalAmount < template.min_capital_usd && (
                <p className="text-red-600 text-sm mt-1">
                  Minimum amount is ${template.min_capital_usd.toLocaleString()}
                </p>
              )}

              <div className="mt-4 space-y-2">
                <div className="flex justify-between text-sm">
                  <span className="text-gray-600">Estimated monthly return:</span>
                  <span className="font-medium text-green-600">
                    ${((capitalAmount * template.estimated_apy / 100) / 12).toFixed(2)}
                  </span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-gray-600">Estimated annual return:</span>
                  <span className="font-medium text-green-600">
                    ${(capitalAmount * template.estimated_apy / 100).toFixed(2)}
                  </span>
                </div>
              </div>
            </div>

            <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
              <h3 className="font-semibold text-yellow-800 mb-2">⚠️ Important Disclaimer</h3>
              <ul className="text-sm text-yellow-700 space-y-1">
                <li>• DeFi investments carry significant risk</li>
                <li>• Returns are estimates and not guaranteed</li>
                <li>• You may lose part or all of your investment</li>
                <li>• Only invest what you can afford to lose</li>
              </ul>
            </div>

            <div className="flex justify-between">
              <button
                onClick={() => setStep(1)}
                className="px-6 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors"
              >
                Back
              </button>
              <button
                onClick={() => setStep(3)}
                disabled={capitalAmount < template.min_capital_usd}
                className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Review & Create
              </button>
            </div>
          </div>
        );

      case 3:
        return (
          <div className="space-y-6">
            <div className="text-center">
              <h2 className="text-2xl font-bold text-gray-900 mb-2">
                Review Your Strategy
              </h2>
              <p className="text-gray-600">
                Please review your strategy details before creating
              </p>
            </div>

            <div className="bg-white border border-gray-200 rounded-lg p-6">
              <div className="flex items-center justify-between mb-4">
                <h3 className="text-lg font-semibold text-gray-900">
                  {template.name}
                </h3>
                <span className={`px-3 py-1 text-sm rounded-full ${simpleDefiTemplateService.getRiskColor(template.risk_score)}`}>
                  Risk: {template.risk_score}/10
                </span>
              </div>

              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <span className="text-gray-600">Category:</span>
                  <span className="ml-2 font-medium">{template.category || 'unknown'}</span>
                </div>
                <div>
                  <span className="text-gray-600">Difficulty:</span>
                  <span className="ml-2 font-medium">{template.difficulty || 'beginner'}</span>
                </div>
                <div>
                  <span className="text-gray-600">Investment Amount:</span>
                  <span className="ml-2 font-medium text-blue-600">
                    ${capitalAmount.toLocaleString()}
                  </span>
                </div>
                <div>
                  <span className="text-gray-600">Est. Annual Return:</span>
                  <span className="ml-2 font-medium text-green-600">
                    ${(capitalAmount * template.estimated_apy / 100).toFixed(2)}
                  </span>
                </div>
              </div>
            </div>

            <div className="bg-blue-50 rounded-lg p-4">
              <h3 className="font-semibold text-blue-800 mb-2">🚀 What happens next?</h3>
              <ul className="text-sm text-blue-700 space-y-1">
                <li>• Strategy will be deployed automatically</li>
                <li>• You'll receive setup confirmation</li>
                <li>• Monitor progress in your dashboard</li>
                <li>• Receive real-time notifications</li>
              </ul>
            </div>

            {error && (
              <div className="bg-red-50 border border-red-200 rounded-lg p-4">
                <p className="text-red-800">{error}</p>
              </div>
            )}

            <div className="flex justify-between">
              <button
                onClick={() => setStep(2)}
                disabled={loading}
                className="px-6 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors disabled:opacity-50"
              >
                Back
              </button>
              <button
                onClick={handleCreateStrategy}
                disabled={loading}
                className="px-8 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50 flex items-center space-x-2"
              >
                {loading ? (
                  <>
                    <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                    <span>Creating...</span>
                  </>
                ) : (
                  <span>Create Strategy</span>
                )}
              </button>
            </div>
          </div>
        );

      default:
        return null;
    }
  };

  return (
    <div className="max-w-2xl mx-auto p-6">
      {/* Progress Steps */}
      <div className="mb-8">
        <div className="flex items-center justify-center space-x-8">
          <div className="flex items-center">
            <span className="text-2xl mr-2">{getStepIcon(1)}</span>
            <span className={`text-sm ${step >= 1 ? 'text-blue-600 font-medium' : 'text-gray-400'}`}>
              Template
            </span>
          </div>
          <div className="flex-1 h-0.5 bg-gray-200">
            <div className={`h-full bg-blue-600 transition-all duration-300 ${step > 1 ? 'w-full' : 'w-0'}`}></div>
          </div>
          <div className="flex items-center">
            <span className="text-2xl mr-2">{getStepIcon(2)}</span>
            <span className={`text-sm ${step >= 2 ? 'text-blue-600 font-medium' : 'text-gray-400'}`}>
              Amount
            </span>
          </div>
          <div className="flex-1 h-0.5 bg-gray-200">
            <div className={`h-full bg-blue-600 transition-all duration-300 ${step > 2 ? 'w-full' : 'w-0'}`}></div>
          </div>
          <div className="flex items-center">
            <span className="text-2xl mr-2">{getStepIcon(3)}</span>
            <span className={`text-sm ${step >= 3 ? 'text-blue-600 font-medium' : 'text-gray-400'}`}>
              Review
            </span>
          </div>
        </div>
      </div>

      {/* Step Content */}
      <div className="bg-white rounded-xl shadow-lg p-8">
        {renderStep()}
      </div>
    </div>
  );
};

export default StrategyCreationFlow;