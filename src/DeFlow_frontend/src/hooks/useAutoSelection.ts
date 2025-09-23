// React hook for auto-selecting best protocol and chain
import { useState, useEffect, useCallback } from 'react';
import {
  backendService,
  convertTradingStyleFromString,
  AutoSelectionResult,
  YieldOpportunity,
  TradingStyleParams
} from '../services/backendService';

export interface UseAutoSelectionProps {
  tradingStyle: string;
  amount: number;
  token: string;
  enabled: boolean;
}

export interface UseAutoSelectionResult {
  // Auto-selection data
  autoSelection: AutoSelectionResult | null;
  yieldOpportunities: YieldOpportunity[];
  tradingStyleParams: TradingStyleParams | null;

  // Loading states
  loading: boolean;
  loadingSelection: boolean;
  loadingOpportunities: boolean;

  // Error states
  error: string | null;

  // Actions
  performAutoSelection: () => Promise<void>;
  refreshOpportunities: () => Promise<void>;

  // Utilities
  isAutoSelectEnabled: boolean;
  getDisplayName: (protocol: string, chain: string) => string;
}

export const useAutoSelection = ({
  tradingStyle,
  amount,
  token,
  enabled
}: UseAutoSelectionProps): UseAutoSelectionResult => {
  const [autoSelection, setAutoSelection] = useState<AutoSelectionResult | null>(null);
  const [yieldOpportunities, setYieldOpportunities] = useState<YieldOpportunity[]>([]);
  const [tradingStyleParams, setTradingStyleParams] = useState<TradingStyleParams | null>(null);

  const [loadingSelection, setLoadingSelection] = useState(false);
  const [loadingOpportunities, setLoadingOpportunities] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loading = loadingSelection || loadingOpportunities;

  // Convert string to backend trading style format
  const tradingStyleObj = convertTradingStyleFromString(tradingStyle);

  // Get trading style parameters
  const fetchTradingStyleParams = useCallback(async () => {
    try {
      const params = await backendService.getTradingStyleParams(tradingStyleObj);
      setTradingStyleParams(params);
    } catch (error) {
      console.error('Error fetching trading style params:', error);
    }
  }, [tradingStyleObj]);

  // Perform auto-selection
  const performAutoSelection = useCallback(async () => {
    if (!enabled || amount <= 0) return;

    setLoadingSelection(true);
    setError(null);

    try {
      const result = await backendService.autoSelectBestOption(
        tradingStyleObj,
        amount,
        token
      );

      setAutoSelection(result);
      console.log('🤖 Auto-selection result:', result);
    } catch (error) {
      const errorMessage = `Auto-selection failed: ${error}`;
      setError(errorMessage);
      console.error(errorMessage, error);
    } finally {
      setLoadingSelection(false);
    }
  }, [tradingStyleObj, amount, token, enabled]);

  // Refresh yield opportunities
  const refreshOpportunities = useCallback(async () => {
    if (!enabled || amount <= 0) return;

    setLoadingOpportunities(true);
    setError(null);

    try {
      const opportunities = await backendService.getYieldOpportunities(
        tradingStyleObj,
        amount
      );

      setYieldOpportunities(opportunities);
      console.log('📊 Yield opportunities:', opportunities);
    } catch (error) {
      const errorMessage = `Failed to fetch opportunities: ${error}`;
      setError(errorMessage);
      console.error(errorMessage, error);
    } finally {
      setLoadingOpportunities(false);
    }
  }, [tradingStyleObj, amount, enabled]);

  // Auto-refresh when parameters change
  useEffect(() => {
    if (enabled && amount > 0) {
      fetchTradingStyleParams();
      performAutoSelection();
      refreshOpportunities();
    }
  }, [enabled, amount, token, tradingStyle, fetchTradingStyleParams, performAutoSelection, refreshOpportunities]);

  // Utility functions
  const isAutoSelectEnabled = enabled && amount > 0;

  const getDisplayName = useCallback((protocol: string, chain: string): string => {
    if (protocol === 'AUTO_SELECT' && chain === 'AUTO_SELECT') {
      return '🤖 Auto-Selected';
    } else if (protocol === 'AUTO_SELECT') {
      return `🤖 Auto-Select Protocol on ${chain}`;
    } else if (chain === 'AUTO_SELECT') {
      return `🤖 Auto-Select Chain for ${protocol}`;
    }
    return `${protocol} on ${chain}`;
  }, []);

  return {
    // Data
    autoSelection,
    yieldOpportunities,
    tradingStyleParams,

    // Loading states
    loading,
    loadingSelection,
    loadingOpportunities,

    // Error state
    error,

    // Actions
    performAutoSelection,
    refreshOpportunities,

    // Utilities
    isAutoSelectEnabled,
    getDisplayName,
  };
};

// Hook for evaluating position moves
export const usePositionEvaluation = () => {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const evaluateMove = useCallback(async (
    tradingStyle: string,
    positionAmountUsd: number,
    currentApy: number,
    targetApy: number,
    estimatedGasCost: number,
    isEthL1: boolean
  ) => {
    setLoading(true);
    setError(null);

    try {
      const tradingStyleObj = convertTradingStyleFromString(tradingStyle);
      const decision = await backendService.evaluatePositionMove(
        tradingStyleObj,
        positionAmountUsd,
        currentApy,
        targetApy,
        estimatedGasCost,
        isEthL1
      );

      console.log('🎯 Position evaluation:', decision);
      return decision;
    } catch (error) {
      const errorMessage = `Position evaluation failed: ${error}`;
      setError(errorMessage);
      console.error(errorMessage, error);
      return null;
    } finally {
      setLoading(false);
    }
  }, []);

  return {
    evaluateMove,
    loading,
    error
  };
};