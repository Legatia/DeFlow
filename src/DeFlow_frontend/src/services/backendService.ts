// Backend service for connecting to DeFlow backend canister
import { Actor, HttpAgent } from '@dfinity/agent';
import { Principal } from '@dfinity/principal';

// Backend canister interface based on our Rust implementation
export interface TradingStyle {
  WaveRider?: null;
  SteadyEarner?: null;
  GasHunter?: null;
  YieldChaser?: null;
  Balanced?: null;
}

export interface TradingStyleParams {
  name: string;
  description: string;
  min_apy: number;
  max_gas_cost_usd: number;
  position_stickiness_multiplier: number;
  payback_days: number;
  preferred_chains: string[];
  gas_priority: string;
  eth_l1_penalty_multiplier: number;
  apy_degradation_threshold: number;
}

export interface MoveDecision {
  should_move: boolean;
  reason: string;
  cost_benefit_ratio: number;
  estimated_gas_cost: number;
  apy_improvement: number;
}

export interface YieldOpportunity {
  protocol: string;
  chain: string;
  apy: number;
  risk_score: number;
  gas_cost_estimate: number;
  liquidity_usd: number;
  min_deposit_usd: number;
}

export interface AutoSelectionResult {
  recommended_protocol: string;
  recommended_chain: string;
  expected_apy: number;
  estimated_gas_cost: number;
  reasoning: string;
  alternatives: YieldOpportunity[];
}

// Backend canister IDL (Interface Definition Language)
const idlFactory = ({ IDL }: any) => {
  const TradingStyle = IDL.Variant({
    'WaveRider': IDL.Null,
    'SteadyEarner': IDL.Null,
    'GasHunter': IDL.Null,
    'YieldChaser': IDL.Null,
    'Balanced': IDL.Null,
  });

  const TradingStyleParams = IDL.Record({
    'name': IDL.Text,
    'description': IDL.Text,
    'min_apy': IDL.Float64,
    'max_gas_cost_usd': IDL.Float64,
    'position_stickiness_multiplier': IDL.Float64,
    'payback_days': IDL.Nat32,
    'preferred_chains': IDL.Vec(IDL.Text),
    'gas_priority': IDL.Text,
    'eth_l1_penalty_multiplier': IDL.Float64,
    'apy_degradation_threshold': IDL.Float64,
  });

  const MoveDecision = IDL.Record({
    'should_move': IDL.Bool,
    'reason': IDL.Text,
    'cost_benefit_ratio': IDL.Float64,
    'estimated_gas_cost': IDL.Float64,
    'apy_improvement': IDL.Float64,
  });

  const YieldOpportunity = IDL.Record({
    'protocol': IDL.Text,
    'chain': IDL.Text,
    'apy': IDL.Float64,
    'risk_score': IDL.Nat8,
    'gas_cost_estimate': IDL.Float64,
    'liquidity_usd': IDL.Nat64,
    'min_deposit_usd': IDL.Nat64,
  });

  const AutoSelectionResult = IDL.Record({
    'recommended_protocol': IDL.Text,
    'recommended_chain': IDL.Text,
    'expected_apy': IDL.Float64,
    'estimated_gas_cost': IDL.Float64,
    'reasoning': IDL.Text,
    'alternatives': IDL.Vec(YieldOpportunity),
  });

  return IDL.Service({
    // Trading Styles API
    'get_trading_styles': IDL.Query([], IDL.Vec(IDL.Tuple(TradingStyle, TradingStyleParams))),
    'get_trading_style_params': IDL.Query([TradingStyle], TradingStyleParams),
    'evaluate_position_move': IDL.Query([
      TradingStyle,
      IDL.Float64, // position_amount_usd
      IDL.Float64, // current_apy
      IDL.Float64, // target_apy
      IDL.Float64, // estimated_gas_cost
      IDL.Bool,    // is_eth_l1
    ], MoveDecision),

    // Auto-selection API (mock for now, will be implemented in backend)
    'auto_select_best_protocol_and_chain': IDL.Query([
      TradingStyle,
      IDL.Float64, // amount_usd
      IDL.Text,    // token
    ], AutoSelectionResult),

    // Get current yield opportunities
    'get_yield_opportunities': IDL.Query([
      TradingStyle,
      IDL.Float64, // amount_usd
    ], IDL.Vec(YieldOpportunity)),
  });
};

class BackendService {
  private actor: any = null;
  private agent: HttpAgent | null = null;

  // Initialize connection to backend canister
  async initialize(canisterId?: string) {
    try {
      // Create agent (use local development by default)
      this.agent = new HttpAgent({
        host: process.env.NODE_ENV === 'development'
          ? 'http://localhost:4943'
          : 'https://ic0.app'
      });

      // Fetch root key for local development
      if (process.env.NODE_ENV === 'development') {
        await this.agent?.fetchRootKey();
      }

      // Use environment variable for canister ID or fallback
      const backendCanisterId = canisterId ||
        process.env.REACT_APP_BACKEND_CANISTER_ID ||
        'rdmx6-jaaaa-aaaah-qdrya-cai'; // Default local canister ID

      // Create actor
      this.actor = Actor.createActor(idlFactory, {
        agent: this.agent,
        canisterId: backendCanisterId,
      });

      console.log('✅ Backend service initialized successfully');
      return true;
    } catch (error) {
      console.error('❌ Failed to initialize backend service:', error);
      return false;
    }
  }

  // Get all available trading styles
  async getTradingStyles(): Promise<[TradingStyle, TradingStyleParams][]> {
    try {
      if (!this.actor) await this.initialize();
      return await this.actor.get_trading_styles();
    } catch (error) {
      console.error('Error getting trading styles:', error);
      // Fallback to mock data
      return this.getMockTradingStyles();
    }
  }

  // Get parameters for a specific trading style
  async getTradingStyleParams(style: TradingStyle): Promise<TradingStyleParams> {
    try {
      if (!this.actor) await this.initialize();
      return await this.actor.get_trading_style_params(style);
    } catch (error) {
      console.error('Error getting trading style params:', error);
      return this.getMockStyleParams(style);
    }
  }

  // Auto-select best protocol and chain based on trading style
  async autoSelectBestOption(
    tradingStyle: TradingStyle,
    amountUsd: number,
    token: string
  ): Promise<AutoSelectionResult> {
    try {
      if (!this.actor) await this.initialize();
      // This would call the real backend auto-selection
      return await this.actor.auto_select_best_protocol_and_chain(tradingStyle, amountUsd, token);
    } catch (error) {
      console.error('Error auto-selecting best option:', error);
      // Fallback to mock selection
      return this.getMockAutoSelection(tradingStyle, amountUsd, token);
    }
  }

  // Evaluate whether to move a position
  async evaluatePositionMove(
    tradingStyle: TradingStyle,
    positionAmountUsd: number,
    currentApy: number,
    targetApy: number,
    estimatedGasCost: number,
    isEthL1: boolean
  ): Promise<MoveDecision> {
    try {
      if (!this.actor) await this.initialize();
      return await this.actor.evaluate_position_move(
        tradingStyle,
        positionAmountUsd,
        currentApy,
        targetApy,
        estimatedGasCost,
        isEthL1
      );
    } catch (error) {
      console.error('Error evaluating position move:', error);
      return this.getMockMoveDecision(tradingStyle, currentApy, targetApy, estimatedGasCost);
    }
  }

  // Get current yield opportunities
  async getYieldOpportunities(
    tradingStyle: TradingStyle,
    amountUsd: number
  ): Promise<YieldOpportunity[]> {
    try {
      if (!this.actor) await this.initialize();
      return await this.actor.get_yield_opportunities(tradingStyle, amountUsd);
    } catch (error) {
      console.error('Error getting yield opportunities:', error);
      return this.getMockYieldOpportunities(tradingStyle, amountUsd);
    }
  }

  // Mock data fallbacks for development
  private getMockTradingStyles(): [TradingStyle, TradingStyleParams][] {
    return [
      [
        { Balanced: null },
        {
          name: 'Balanced',
          description: 'Smart balance of yield and costs',
          min_apy: 2.0,
          max_gas_cost_usd: 20.0,
          position_stickiness_multiplier: 1.0,
          payback_days: 7,
          preferred_chains: ['Arbitrum', 'Polygon', 'Optimism', 'Base'],
          gas_priority: 'Medium',
          eth_l1_penalty_multiplier: 2.5,
          apy_degradation_threshold: 20.0,
        }
      ],
      [
        { WaveRider: null },
        {
          name: 'Wave Rider',
          description: 'Patient strategy that catches high APY waves',
          min_apy: 1.0,
          max_gas_cost_usd: 25.0,
          position_stickiness_multiplier: 1.5,
          payback_days: 10,
          preferred_chains: ['Arbitrum', 'Polygon', 'Optimism', 'Base'],
          gas_priority: 'Low',
          eth_l1_penalty_multiplier: 3.0,
          apy_degradation_threshold: 25.0,
        }
      ]
    ];
  }

  private getMockStyleParams(style: TradingStyle): TradingStyleParams {
    return {
      name: 'Balanced',
      description: 'Smart balance of yield and costs',
      min_apy: 2.0,
      max_gas_cost_usd: 20.0,
      position_stickiness_multiplier: 1.0,
      payback_days: 7,
      preferred_chains: ['Arbitrum', 'Polygon', 'Optimism', 'Base'],
      gas_priority: 'Medium',
      eth_l1_penalty_multiplier: 2.5,
      apy_degradation_threshold: 20.0,
    };
  }

  private getMockAutoSelection(tradingStyle: TradingStyle, amountUsd: number, token: string): AutoSelectionResult {
    // Simple mock logic based on trading style
    const styleKey = Object.keys(tradingStyle)[0];

    let protocol = 'Aave';
    let chain = 'Arbitrum';
    let expectedApy = 4.5;
    let estimatedGasCost = 5.0;

    switch (styleKey) {
      case 'GasHunter':
        protocol = 'Compound';
        chain = 'Polygon';
        expectedApy = 3.8;
        estimatedGasCost = 2.0;
        break;
      case 'YieldChaser':
        protocol = 'Pendle';
        chain = 'Ethereum';
        expectedApy = 8.2;
        estimatedGasCost = 25.0;
        break;
      case 'WaveRider':
        protocol = 'Uniswap V3';
        chain = 'Arbitrum';
        expectedApy = 6.1;
        estimatedGasCost = 8.0;
        break;
    }

    return {
      recommended_protocol: protocol,
      recommended_chain: chain,
      expected_apy: expectedApy,
      estimated_gas_cost: estimatedGasCost,
      reasoning: `Based on ${styleKey} trading style: optimized for ${styleKey === 'GasHunter' ? 'minimal costs' : styleKey === 'YieldChaser' ? 'maximum yield' : 'balance'}`,
      alternatives: [
        {
          protocol: 'Aave',
          chain: 'Arbitrum',
          apy: 4.2,
          risk_score: 3,
          gas_cost_estimate: 5.0,
          liquidity_usd: 1000000,
          min_deposit_usd: 100,
        }
      ]
    };
  }

  private getMockMoveDecision(
    tradingStyle: TradingStyle,
    currentApy: number,
    targetApy: number,
    estimatedGasCost: number
  ): MoveDecision {
    const apyDiff = targetApy - currentApy;
    const shouldMove = apyDiff > 1.0 && estimatedGasCost < 15.0;

    return {
      should_move: shouldMove,
      reason: shouldMove
        ? `APY improvement of ${apyDiff.toFixed(2)}% justifies $${estimatedGasCost.toFixed(2)} gas cost`
        : `APY improvement of ${apyDiff.toFixed(2)}% insufficient for $${estimatedGasCost.toFixed(2)} gas cost`,
      cost_benefit_ratio: apyDiff / (estimatedGasCost / 100),
      estimated_gas_cost: estimatedGasCost,
      apy_improvement: apyDiff,
    };
  }

  private getMockYieldOpportunities(tradingStyle: TradingStyle, amountUsd: number): YieldOpportunity[] {
    return [
      {
        protocol: 'Aave',
        chain: 'Arbitrum',
        apy: 4.2,
        risk_score: 3,
        gas_cost_estimate: 5.0,
        liquidity_usd: 1000000,
        min_deposit_usd: 100,
      },
      {
        protocol: 'Compound',
        chain: 'Polygon',
        apy: 3.8,
        risk_score: 2,
        gas_cost_estimate: 2.0,
        liquidity_usd: 500000,
        min_deposit_usd: 50,
      },
      {
        protocol: 'Uniswap V3',
        chain: 'Optimism',
        apy: 6.1,
        risk_score: 5,
        gas_cost_estimate: 8.0,
        liquidity_usd: 750000,
        min_deposit_usd: 200,
      }
    ];
  }
}

// Export singleton instance
export const backendService = new BackendService();

// Utility functions for frontend
export const convertTradingStyleFromString = (styleString: string): TradingStyle => {
  switch (styleString) {
    case 'WaveRider': return { WaveRider: null };
    case 'SteadyEarner': return { SteadyEarner: null };
    case 'GasHunter': return { GasHunter: null };
    case 'YieldChaser': return { YieldChaser: null };
    case 'Balanced':
    default: return { Balanced: null };
  }
};

export const convertTradingStyleToString = (style: TradingStyle): string => {
  return Object.keys(style)[0] as string;
};