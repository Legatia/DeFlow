// DeFi Strategy Configuration Types
import { Node, Edge } from 'reactflow'

export interface StrategyConfig {
  name: string
  description: string
  strategy_type: StrategyTypeConfig
  target_chains: string[]
  target_protocols: string[]
  risk_level: number // 1-10
  max_allocation_usd: number
  min_return_threshold: number
  execution_interval_minutes: number
  gas_limit_usd: number
  auto_compound: boolean
  stop_loss_percentage?: number | null
  take_profit_percentage?: number | null
  workflow_definition?: WorkflowDefinition
}

export interface WorkflowDefinition {
  nodes: Node[]
  edges: Edge[]
  compiled_at: string
}

export interface StrategyTypeConfig {
  type: 'YieldFarming' | 'Arbitrage' | 'DCA' | 'Rebalancing' | 'LiquidityMining' | 'Composite'
  config: any // Strategy-specific configuration
}

// Strategy Type Specific Configs
export interface YieldFarmingConfig {
  min_apy_threshold: number
  preferred_tokens: string[]
  max_impermanent_loss_percentage: number
  auto_harvest_rewards: boolean
}

export interface ArbitrageConfig {
  min_profit_percentage: number
  max_execution_time_seconds: number
  max_slippage_percentage: number
  preferred_dex_pairs: Array<[string, string]>
}

export interface DCAConfig {
  target_token: string
  amount_per_execution: number
  price_threshold_percentage?: number | null
}

export interface RebalancingConfig {
  target_allocation: Record<string, number>
  rebalance_threshold: number
  min_trade_amount: number
}

export interface LiquidityMiningConfig {
  preferred_pairs: Array<[string, string]>
  max_pool_concentration_percentage: number
}

// Strategy Creation Response
export interface StrategyCreationResult {
  strategy_id: string
  status: 'created' | 'error'
  message: string
  deployment_status?: string
}