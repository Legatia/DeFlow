// Auto-selection panel for DeFi workflow nodes
import React, { useState, useEffect } from 'react';
import { useAutoSelection, usePositionEvaluation } from '../hooks/useAutoSelection';

interface AutoSelectionPanelProps {
  tradingStyle: string;
  protocol: string;
  chain: string;
  token: string;
  amount: number;
  onProtocolChange: (protocol: string) => void;
  onChainChange: (chain: string) => void;
  className?: string;
}

const AutoSelectionPanel: React.FC<AutoSelectionPanelProps> = ({
  tradingStyle,
  protocol,
  chain,
  token,
  amount,
  onProtocolChange,
  onChainChange,
  className = ''
}) => {
  const [showDetails, setShowDetails] = useState(false);

  // Check if auto-selection is enabled
  const isAutoSelectEnabled = protocol === 'AUTO_SELECT' || chain === 'AUTO_SELECT';

  // Use auto-selection hook
  const {
    autoSelection,
    yieldOpportunities,
    tradingStyleParams,
    loading,
    error,
    performAutoSelection,
    refreshOpportunities,
    getDisplayName
  } = useAutoSelection({
    tradingStyle,
    amount,
    token,
    enabled: isAutoSelectEnabled
  });

  const { evaluateMove } = usePositionEvaluation();

  // Auto-apply selection when backend returns result
  useEffect(() => {
    if (autoSelection && isAutoSelectEnabled) {
      if (protocol === 'AUTO_SELECT') {
        onProtocolChange(autoSelection.recommended_protocol);
      }
      if (chain === 'AUTO_SELECT') {
        onChainChange(autoSelection.recommended_chain);
      }
    }
  }, [autoSelection, protocol, chain, onProtocolChange, onChainChange, isAutoSelectEnabled]);

  if (!isAutoSelectEnabled) {
    return null; // Don't show anything if auto-selection is not enabled
  }

  return (
    <div className={`bg-slate-800/50 border border-slate-600/40 rounded-lg p-4 space-y-4 ${className}`}>
      {/* Auto-Selection Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-2">
          <div className="w-2 h-2 bg-cyan-400 rounded-full animate-pulse"></div>
          <h3 className="text-sm font-medium text-cyan-400">
            🤖 AI Auto-Selection Active
          </h3>
        </div>
        <button
          onClick={() => setShowDetails(!showDetails)}
          className="text-xs text-slate-400 hover:text-slate-200 transition-colors"
        >
          {showDetails ? 'Hide Details' : 'Show Details'}
        </button>
      </div>

      {/* Loading State */}
      {loading && (
        <div className="flex items-center space-x-2 text-sm text-slate-400">
          <div className="w-4 h-4 border-2 border-cyan-400/30 border-t-cyan-400 rounded-full animate-spin"></div>
          <span>Analyzing best options...</span>
        </div>
      )}

      {/* Error State */}
      {error && (
        <div className="text-sm text-red-400 bg-red-900/20 border border-red-500/30 rounded p-2">
          {error}
        </div>
      )}

      {/* Auto-Selection Result */}
      {autoSelection && !loading && (
        <div className="space-y-3">
          <div className="text-sm">
            <div className="flex items-center justify-between">
              <span className="text-slate-300">Selected:</span>
              <span className="text-cyan-400 font-medium">
                {getDisplayName(autoSelection.recommended_protocol, autoSelection.recommended_chain)}
              </span>
            </div>
            <div className="flex items-center justify-between mt-1">
              <span className="text-slate-400">Expected APY:</span>
              <span className="text-green-400 font-medium">
                {autoSelection.expected_apy.toFixed(2)}%
              </span>
            </div>
            <div className="flex items-center justify-between mt-1">
              <span className="text-slate-400">Est. Gas Cost:</span>
              <span className={`font-medium ${autoSelection.estimated_gas_cost > 20 ? 'text-red-400' : 'text-green-400'}`}>
                ${autoSelection.estimated_gas_cost.toFixed(2)}
              </span>
            </div>
          </div>

          {/* Reasoning */}
          <div className="text-xs text-slate-400 bg-slate-700/30 rounded p-2">
            <strong>Reasoning:</strong> {autoSelection.reasoning}
          </div>

          {/* Action Buttons */}
          <div className="flex space-x-2">
            <button
              onClick={performAutoSelection}
              disabled={loading}
              className="flex-1 px-3 py-1.5 bg-cyan-600/20 text-cyan-400 text-xs rounded hover:bg-cyan-600/30 transition-colors disabled:opacity-50"
            >
              🔄 Refresh Selection
            </button>
            <button
              onClick={refreshOpportunities}
              disabled={loading}
              className="flex-1 px-3 py-1.5 bg-emerald-600/20 text-emerald-400 text-xs rounded hover:bg-emerald-600/30 transition-colors disabled:opacity-50"
            >
              📊 View Options
            </button>
          </div>
        </div>
      )}

      {/* Trading Style Info */}
      {tradingStyleParams && showDetails && (
        <div className="border-t border-slate-600/40 pt-3 space-y-2">
          <h4 className="text-xs font-medium text-slate-300">
            {tradingStyleParams.name} Strategy
          </h4>
          <div className="text-xs text-slate-400 space-y-1">
            <div>{tradingStyleParams.description}</div>
            <div className="grid grid-cols-2 gap-2 mt-2">
              <div>
                <span className="text-slate-500">Min APY:</span>{' '}
                <span className="text-slate-300">{tradingStyleParams.min_apy}%</span>
              </div>
              <div>
                <span className="text-slate-500">Max Gas:</span>{' '}
                <span className="text-slate-300">${tradingStyleParams.max_gas_cost_usd}</span>
              </div>
              <div>
                <span className="text-slate-500">Payback:</span>{' '}
                <span className="text-slate-300">{tradingStyleParams.payback_days} days</span>
              </div>
              <div>
                <span className="text-slate-500">Chains:</span>{' '}
                <span className="text-slate-300">{tradingStyleParams.preferred_chains.length}</span>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Yield Opportunities */}
      {yieldOpportunities.length > 0 && showDetails && (
        <div className="border-t border-slate-600/40 pt-3 space-y-2">
          <h4 className="text-xs font-medium text-slate-300">Alternative Options</h4>
          <div className="space-y-2 max-h-32 overflow-y-auto">
            {yieldOpportunities.slice(0, 3).map((opportunity, index) => (
              <div
                key={index}
                className="text-xs bg-slate-700/30 rounded p-2 hover:bg-slate-700/50 transition-colors cursor-pointer"
                onClick={() => {
                  if (protocol === 'AUTO_SELECT') onProtocolChange(opportunity.protocol);
                  if (chain === 'AUTO_SELECT') onChainChange(opportunity.chain);
                }}
              >
                <div className="flex justify-between items-center">
                  <span className="text-slate-300">
                    {opportunity.protocol} on {opportunity.chain}
                  </span>
                  <span className="text-green-400">{opportunity.apy.toFixed(2)}%</span>
                </div>
                <div className="flex justify-between items-center mt-1 text-slate-400">
                  <span>Risk: {opportunity.risk_score}/10</span>
                  <span>Gas: ${opportunity.gas_cost_estimate.toFixed(2)}</span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export default AutoSelectionPanel;