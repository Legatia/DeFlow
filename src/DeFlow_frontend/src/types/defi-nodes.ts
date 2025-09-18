// DeFi-specific node types for custom strategy building
import { NodeType } from './nodes'

export const DEFI_NODE_TYPES: NodeType[] = [
  // DeFi Triggers
  {
    id: 'price-trigger',
    name: 'Price Trigger',
    description: 'Trigger when asset price meets condition',
    category: 'triggers',
    icon: '📈',
    color: '#3b82f6',
    inputs: [],
    outputs: [
      { id: 'trigger', name: 'Price Met', type: 'trigger', required: true },
      { id: 'data', name: 'Price Data', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'asset',
        name: 'Asset',
        type: 'select',
        required: true,
        options: [
          { label: 'Bitcoin (BTC)', value: 'BTC' },
          { label: 'Ethereum (ETH)', value: 'ETH' },
          { label: 'USDC', value: 'USDC' },
          { label: 'USDT', value: 'USDT' }
        ],
        defaultValue: 'BTC'
      },
      {
        key: 'condition',
        name: 'Price Condition',
        type: 'select',
        required: true,
        options: [
          { label: 'Greater Than', value: 'greater_than' },
          { label: 'Less Than', value: 'less_than' },
          { label: 'Drops by %', value: 'drops_percent' },
          { label: 'Rises by %', value: 'rises_percent' }
        ],
        defaultValue: 'greater_than'
      },
      {
        key: 'value',
        name: 'Target Value',
        type: 'number',
        required: true,
        placeholder: '50000',
        validation: { min: 0 }
      }
    ],
    defaultConfig: { asset: 'BTC', condition: 'greater_than', value: 50000 },
    tieredPricing: {
      standard: { executionFee: 0.1, description: 'Basic execution with standard speed' },
      premium: { executionFee: 0.05, description: 'Faster execution with priority processing' },
      pro: { executionFee: 0.02, description: 'Fastest execution with advanced features' }
    }
  },

  // Consolidated Yield Farming Strategy
  {
    id: 'yield-farming-strategy',
    name: 'Yield Farming Strategy',
    description: 'Complete yield farming workflow in one node',
    category: 'actions',
    icon: '🌾',
    color: '#10b981',
    inputs: [
      { id: 'trigger', name: 'Execute', type: 'trigger', required: true }
    ],
    outputs: [
      { id: 'farm_result', name: 'Farm Result', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'protocol',
        name: 'DeFi Protocol',
        type: 'select',
        required: true,
        options: [
          { label: 'Aave', value: 'Aave' },
          { label: 'Compound', value: 'Compound' },
          { label: 'Uniswap V3', value: 'UniswapV3' },
          { label: 'Curve', value: 'Curve' },
          { label: 'Yearn Finance', value: 'Yearn' }
        ],
        defaultValue: 'Aave'
      },
      {
        key: 'chain',
        name: 'Chain',
        type: 'select',
        required: true,
        options: [
          { label: 'Ethereum', value: 'Ethereum' },
          { label: 'Arbitrum', value: 'Arbitrum' },
          { label: 'Optimism', value: 'Optimism' },
          { label: 'Polygon', value: 'Polygon' }
        ],
        defaultValue: 'Ethereum'
      },
      {
        key: 'token',
        name: 'Token to Farm',
        type: 'select',
        required: true,
        options: [
          { label: 'USDC', value: 'USDC' },
          { label: 'ETH', value: 'ETH' },
          { label: 'USDT', value: 'USDT' },
          { label: 'DAI', value: 'DAI' }
        ],
        defaultValue: 'USDC'
      },
      {
        key: 'amount',
        name: 'Amount (USD)',
        type: 'number',
        required: true,
        validation: { min: 10 },
        defaultValue: 1000
      },
      {
        key: 'amount_type',
        name: 'Amount Type',
        type: 'select',
        required: true,
        options: [
          { label: 'Fixed USD Amount', value: 'fixed' },
          { label: 'Percentage of Balance', value: 'percentage' }
        ],
        defaultValue: 'fixed'
      },
      {
        key: 'min_apy',
        name: 'Minimum APY (%)',
        type: 'number',
        required: true,
        validation: { min: 0, max: 100 },
        defaultValue: 5.0
      },
      {
        key: 'auto_compound',
        name: 'Auto Compound',
        type: 'boolean',
        required: false,
        defaultValue: true
      },
      {
        key: 'gas_priority',
        name: 'Gas Priority',
        type: 'select',
        required: true,
        options: [
          { label: 'Low (Cheapest)', value: 'low' },
          { label: 'Medium (Standard)', value: 'medium' },
          { label: 'High (Fast)', value: 'high' }
        ],
        defaultValue: 'medium'
      }
    ],
    defaultConfig: { 
      protocol: 'Aave', 
      chain: 'Ethereum', 
      token: 'USDC', 
      amount: 1000, 
      amount_type: 'fixed',
      min_apy: 5.0, 
      auto_compound: true,
      gas_priority: 'medium' 
    },
    tieredPricing: {
      standard: { executionFee: 0.15, description: 'Complete yield farming execution' },
      premium: { executionFee: 0.08, description: 'Priority yield farming with optimization' },
      pro: { executionFee: 0.04, description: 'Advanced yield farming with MEV protection' }
    }
  },

  // Consolidated Arbitrage Strategy
  {
    id: 'arbitrage-strategy',
    name: 'Arbitrage Strategy',
    description: 'Complete cross-chain arbitrage in one node',
    category: 'actions',
    icon: '⚖️',
    color: '#10b981',
    inputs: [
      { id: 'trigger', name: 'Execute', type: 'trigger', required: true }
    ],
    outputs: [
      { id: 'arbitrage_result', name: 'Arbitrage Result', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'asset',
        name: 'Asset to Arbitrage',
        type: 'select',
        required: true,
        options: [
          { label: 'Bitcoin (BTC)', value: 'BTC' },
          { label: 'Ethereum (ETH)', value: 'ETH' },
          { label: 'USDC', value: 'USDC' },
          { label: 'USDT', value: 'USDT' },
          { label: 'DAI', value: 'DAI' }
        ],
        defaultValue: 'BTC'
      },
      {
        key: 'buy_chain',
        name: 'Buy Chain',
        type: 'select',
        required: true,
        options: [
          { label: 'Ethereum', value: 'Ethereum' },
          { label: 'Arbitrum', value: 'Arbitrum' },
          { label: 'Optimism', value: 'Optimism' },
          { label: 'Polygon', value: 'Polygon' },
          { label: 'Base', value: 'Base' },
          { label: 'Solana', value: 'Solana' },
          { label: 'ICP', value: 'ICP' }
        ],
        defaultValue: 'Ethereum'
      },
      {
        key: 'sell_chain',
        name: 'Sell Chain',
        type: 'select',
        required: true,
        options: [
          { label: 'Ethereum', value: 'Ethereum' },
          { label: 'Arbitrum', value: 'Arbitrum' },
          { label: 'Optimism', value: 'Optimism' },
          { label: 'Polygon', value: 'Polygon' },
          { label: 'Base', value: 'Base' },
          { label: 'Solana', value: 'Solana' },
          { label: 'ICP', value: 'ICP' }
        ],
        defaultValue: 'Arbitrum'
      },
      {
        key: 'min_profit_percent',
        name: 'Min Profit (%)',
        type: 'number',
        required: true,
        validation: { min: 0.1, max: 50 },
        defaultValue: 1.0
      },
      {
        key: 'max_amount',
        name: 'Max Amount (USD)',
        type: 'number',
        required: true,
        validation: { min: 100 },
        defaultValue: 5000
      },
      {
        key: 'gas_optimization',
        name: 'Gas Optimization',
        type: 'select',
        required: true,
        options: [
          { label: 'Standard', value: 'standard' },
          { label: 'Aggressive', value: 'aggressive' },
          { label: 'Conservative', value: 'conservative' }
        ],
        defaultValue: 'standard'
      }
    ],
    defaultConfig: { 
      asset: 'BTC', 
      buy_chain: 'Ethereum', 
      sell_chain: 'Arbitrum', 
      min_profit_percent: 1.0, 
      max_amount: 5000,
      gas_optimization: 'standard'
    },
    tieredPricing: {
      standard: { executionFee: 0.12, description: 'Complete arbitrage execution' },
      premium: { executionFee: 0.07, description: 'Priority arbitrage with MEV protection' },
      pro: { executionFee: 0.04, description: 'Advanced arbitrage with flash loans' }
    }
  },

  {
    id: 'dca-strategy',
    name: 'Dollar Cost Average',
    description: 'Execute DCA investment strategy',
    category: 'actions',
    icon: '💰',
    color: '#10b981',
    inputs: [
      { id: 'trigger', name: 'Execute', type: 'trigger', required: true }
    ],
    outputs: [
      { id: 'result', name: 'DCA Result', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'target_token',
        name: 'Target Token',
        type: 'select',
        required: true,
        options: [
          { label: 'Bitcoin (BTC)', value: 'BTC' },
          { label: 'Ethereum (ETH)', value: 'ETH' },
          { label: 'Solana (SOL)', value: 'SOL' }
        ],
        defaultValue: 'ETH'
      },
      {
        key: 'amount_per_execution',
        name: 'Amount per Buy (USD)',
        type: 'number',
        required: true,
        validation: { min: 10 },
        defaultValue: 100
      },
      {
        key: 'price_threshold_percentage',
        name: 'Price Drop Threshold (%)',
        type: 'number',
        required: false,
        validation: { min: 0, max: 50 },
        placeholder: 'Optional - only buy on dips',
        description: 'Only execute if price dropped by this percentage'
      }
    ],
    defaultConfig: { 
      target_token: 'ETH', 
      amount_per_execution: 100, 
      price_threshold_percentage: null 
    },
    tieredPricing: {
      standard: { executionFee: 0.1, description: 'Basic execution with standard speed' },
      premium: { executionFee: 0.05, description: 'Faster execution with priority processing' },
      pro: { executionFee: 0.02, description: 'Fastest execution with advanced features' }
    }
  },

  // Simplified Portfolio Rebalancing
  {
    id: 'set-portfolio-allocation',
    name: 'Set Portfolio Allocation',
    description: 'Define target portfolio percentages',
    category: 'actions',
    icon: '📊',
    color: '#10b981',
    inputs: [
      { id: 'trigger', name: 'Execute', type: 'trigger', required: true }
    ],
    outputs: [
      { id: 'allocation_data', name: 'Allocation Data', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'btc_percent',
        name: 'Bitcoin (%)',
        type: 'number',
        required: true,
        validation: { min: 0, max: 100 },
        defaultValue: 60
      },
      {
        key: 'eth_percent',
        name: 'Ethereum (%)',
        type: 'number',
        required: true,
        validation: { min: 0, max: 100 },
        defaultValue: 30
      },
      {
        key: 'stable_percent',
        name: 'Stablecoin (%)',
        type: 'number',
        required: true,
        validation: { min: 0, max: 100 },
        defaultValue: 10
      }
    ],
    defaultConfig: { btc_percent: 60, eth_percent: 30, stable_percent: 10 },
    tieredPricing: {
      standard: { executionFee: 0.02, description: 'Basic allocation setup' },
      premium: { executionFee: 0.01, description: 'Fast allocation setup' },
      pro: { executionFee: 0.005, description: 'Instant allocation setup' }
    }
  },

  {
    id: 'execute-rebalance',
    name: 'Execute Rebalance',
    description: 'Rebalance portfolio to target allocation',
    category: 'actions',
    icon: '⚖️',
    color: '#10b981',
    inputs: [
      { id: 'allocation_data', name: 'Allocation Data', type: 'data', required: true }
    ],
    outputs: [
      { id: 'result', name: 'Rebalance Result', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'rebalance_threshold',
        name: 'Rebalance Threshold (%)',
        type: 'number',
        required: true,
        validation: { min: 1, max: 50 },
        defaultValue: 5,
        description: 'Trigger rebalance when drift exceeds this percentage'
      },
      {
        key: 'min_trade_amount',
        name: 'Min Trade Amount (USD)',
        type: 'number',
        required: true,
        validation: { min: 10 },
        defaultValue: 50,
        description: 'Minimum trade size to avoid dust trades'
      }
    ],
    defaultConfig: { rebalance_threshold: 5, min_trade_amount: 50 },
    tieredPricing: {
      standard: { executionFee: 0.1, description: 'Basic execution with standard speed' },
      premium: { executionFee: 0.05, description: 'Faster execution with priority processing' },
      pro: { executionFee: 0.02, description: 'Fastest execution with advanced features' }
    }
  },

  // DeFi Conditions
  {
    id: 'yield-condition',
    name: 'Yield Condition',
    description: 'Check if yield meets criteria',
    category: 'conditions',
    icon: '📊',
    color: '#f59e0b',
    inputs: [
      { id: 'data', name: 'Input', type: 'data', required: true }
    ],
    outputs: [
      { id: 'true', name: 'Yield Good', type: 'condition', required: true },
      { id: 'false', name: 'Yield Poor', type: 'condition', required: true }
    ],
    configSchema: [
      {
        key: 'protocol',
        name: 'Protocol to Check',
        type: 'select',
        required: true,
        options: [
          { label: 'Aave', value: 'Aave' },
          { label: 'Compound', value: 'Compound' },
          { label: 'Uniswap V3', value: 'UniswapV3' }
        ],
        defaultValue: 'Aave'
      },
      {
        key: 'asset',
        name: 'Asset',
        type: 'select',
        required: true,
        options: [
          { label: 'USDC', value: 'USDC' },
          { label: 'ETH', value: 'ETH' },
          { label: 'USDT', value: 'USDT' }
        ],
        defaultValue: 'USDC'
      },
      {
        key: 'min_apy',
        name: 'Minimum APY (%)',
        type: 'number',
        required: true,
        validation: { min: 0 },
        defaultValue: 5.0
      }
    ],
    defaultConfig: { protocol: 'Aave', asset: 'USDC', min_apy: 5.0 },
    tieredPricing: {
      standard: { executionFee: 0.1, description: 'Basic execution with standard speed' },
      premium: { executionFee: 0.05, description: 'Faster execution with priority processing' },
      pro: { executionFee: 0.02, description: 'Fastest execution with advanced features' }
    }
  },

  // DeFi Utilities
  {
    id: 'price-check',
    name: 'Price Check',
    description: 'Get current asset price',
    category: 'utilities',
    icon: '💲',
    color: '#8b5cf6',
    inputs: [
      { id: 'trigger', name: 'Check Price', type: 'trigger', required: true }
    ],
    outputs: [
      { id: 'price_data', name: 'Price Data', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'asset',
        name: 'Asset',
        type: 'select',
        required: true,
        options: [
          { label: 'Bitcoin (BTC)', value: 'BTC' },
          { label: 'Ethereum (ETH)', value: 'ETH' },
          { label: 'USDC', value: 'USDC' },
          { label: 'Solana (SOL)', value: 'SOL' }
        ],
        defaultValue: 'BTC'
      },
      {
        key: 'chain',
        name: 'Chain',
        type: 'select',
        required: true,
        options: [
          { label: 'Ethereum', value: 'Ethereum' },
          { label: 'Bitcoin', value: 'Bitcoin' },
          { label: 'Solana', value: 'Solana' },
          { label: 'Arbitrum', value: 'Arbitrum' },
          { label: 'ICP', value: 'ICP' }
        ],
        defaultValue: 'Ethereum'
      }
    ],
    defaultConfig: { asset: 'ICP', chain: 'ICP' },
    tieredPricing: {
      standard: { executionFee: 0.1, description: 'Basic execution with standard speed' },
      premium: { executionFee: 0.05, description: 'Faster execution with priority processing' },
      pro: { executionFee: 0.02, description: 'Fastest execution with advanced features' }
    }
  },

  // Consolidated Cycles Monitor
  {
    id: 'cycles-monitor',
    name: 'Cycles Monitor',
    description: 'Monitor cycles and auto top-up when low',
    category: 'utilities',
    icon: '🔋',
    color: '#8b5cf6',
    inputs: [
      { id: 'trigger', name: 'Monitor Cycles', type: 'trigger', required: true }
    ],
    outputs: [
      { id: 'cycles_status', name: 'Cycles Status', type: 'data', required: true },
      { id: 'low_cycles', name: 'Low Cycles Alert', type: 'condition', required: false },
      { id: 'topup_result', name: 'Top-up Result', type: 'data', required: false }
    ],
    configSchema: [
      {
        key: 'canister_id',
        name: 'Canister ID',
        type: 'text',
        required: false,
        placeholder: 'rdmx6-jaaaa-aaaah-qdrva-cai',
        description: 'Leave empty to monitor current canister'
      },
      {
        key: 'warning_threshold',
        name: 'Warning Threshold (T Cycles)',
        type: 'number',
        required: true,
        validation: { min: 1 },
        defaultValue: 10,
        description: 'Alert when cycles drop below this level (in trillions)'
      },
      {
        key: 'auto_topup',
        name: 'Auto Top-up',
        type: 'boolean',
        required: false,
        defaultValue: false,
        description: 'Automatically top-up when threshold is reached'
      },
      {
        key: 'topup_amount',
        name: 'Top-up Amount (T Cycles)',
        type: 'number',
        required: false,
        validation: { min: 1 },
        defaultValue: 20,
        description: 'Amount to request for top-up (in trillions)'
      },
      {
        key: 'notification_channel',
        name: 'Notification Channel',
        type: 'select',
        required: true,
        options: [
          { label: 'Email', value: 'email' },
          { label: 'Discord', value: 'discord' },
          { label: 'Telegram', value: 'telegram' },
          { label: 'Slack', value: 'slack' }
        ],
        defaultValue: 'email'
      },
      {
        key: 'check_interval',
        name: 'Check Interval',
        type: 'select',
        required: true,
        options: [
          { label: 'Every Hour', value: '1h' },
          { label: 'Every 6 Hours', value: '6h' },
          { label: 'Daily', value: '24h' },
          { label: 'Weekly', value: '168h' }
        ],
        defaultValue: '6h'
      }
    ],
    defaultConfig: { 
      canister_id: '', 
      warning_threshold: 10, 
      auto_topup: false, 
      topup_amount: 20, 
      notification_channel: 'email',
      check_interval: '6h'
    },
    tieredPricing: {
      standard: { executionFee: 0.08, description: 'Complete cycles monitoring with alerts' },
      premium: { executionFee: 0.04, description: 'Advanced monitoring with auto top-up' },
      pro: { executionFee: 0.02, description: 'Enterprise monitoring with custom thresholds' }
    }
  },

  // ================================
  // Common Reusable DeFi Utility Nodes
  // ================================

  // Asset Selector - Reusable across yield, arbitrage, portfolio
  {
    id: 'select-asset',
    name: 'Select Asset',
    description: 'Choose any cryptocurrency asset',
    category: 'utilities',
    icon: '💎',
    color: '#8b5cf6',
    inputs: [
      { id: 'trigger', name: 'Execute', type: 'trigger', required: true }
    ],
    outputs: [
      { id: 'asset_data', name: 'Asset Data', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'asset_symbol',
        name: 'Asset Symbol',
        type: 'select',
        required: true,
        options: [
          { label: 'Bitcoin (BTC)', value: 'BTC' },
          { label: 'Ethereum (ETH)', value: 'ETH' },
          { label: 'USDC', value: 'USDC' },
          { label: 'USDT', value: 'USDT' },
          { label: 'DAI', value: 'DAI' },
          { label: 'SOL', value: 'SOL' },
          { label: 'MATIC', value: 'MATIC' },
          { label: 'AVAX', value: 'AVAX' }
        ],
        defaultValue: 'BTC'
      }
    ],
    defaultConfig: { asset_symbol: 'BTC' },
    tieredPricing: {
      standard: { executionFee: 0.01, description: 'Basic asset selection' },
      premium: { executionFee: 0.005, description: 'Fast asset selection' },
      pro: { executionFee: 0.002, description: 'Instant asset selection' }
    }
  },

  // Chain Selector - Reusable across all multi-chain operations
  {
    id: 'select-chain',
    name: 'Select Chain',
    description: 'Choose blockchain network',
    category: 'utilities',
    icon: '🔗',
    color: '#8b5cf6',
    inputs: [
      { id: 'trigger', name: 'Execute', type: 'trigger', required: true }
    ],
    outputs: [
      { id: 'chain_data', name: 'Chain Data', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'chain_name',
        name: 'Blockchain',
        type: 'select',
        required: true,
        options: [
          { label: 'Ethereum', value: 'Ethereum' },
          { label: 'Bitcoin', value: 'Bitcoin' },
          { label: 'Arbitrum', value: 'Arbitrum' },
          { label: 'Optimism', value: 'Optimism' },
          { label: 'Polygon', value: 'Polygon' },
          { label: 'Base', value: 'Base' },
          { label: 'Avalanche', value: 'Avalanche' },
          { label: 'Solana', value: 'Solana' },
          { label: 'ICP', value: 'ICP' }
        ],
        defaultValue: 'Ethereum'
      }
    ],
    defaultConfig: { chain_name: 'Ethereum' },
    tieredPricing: {
      standard: { executionFee: 0.01, description: 'Basic chain selection' },
      premium: { executionFee: 0.005, description: 'Fast chain selection' },
      pro: { executionFee: 0.002, description: 'Instant chain selection' }
    }
  },

  // Amount Setter - Reusable for all amount-based operations
  {
    id: 'set-amount',
    name: 'Set Amount',
    description: 'Configure transaction amount',
    category: 'utilities',
    icon: '💰',
    color: '#8b5cf6',
    inputs: [
      { id: 'asset_data', name: 'Asset Data', type: 'data', required: false }
    ],
    outputs: [
      { id: 'amount_data', name: 'Amount Data', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'amount',
        name: 'Amount',
        type: 'number',
        required: true,
        validation: { min: 0.01 },
        defaultValue: 100
      },
      {
        key: 'currency',
        name: 'Currency Unit',
        type: 'select',
        required: true,
        options: [
          { label: 'USD', value: 'USD' },
          { label: 'Native Token', value: 'NATIVE' },
          { label: 'Percentage (%)', value: 'PERCENT' }
        ],
        defaultValue: 'USD'
      }
    ],
    defaultConfig: { amount: 100, currency: 'USD' },
    tieredPricing: {
      standard: { executionFee: 0.01, description: 'Basic amount configuration' },
      premium: { executionFee: 0.005, description: 'Fast configuration' },
      pro: { executionFee: 0.002, description: 'Instant configuration' }
    }
  },

  // Price Checker - Reusable for price queries
  {
    id: 'check-price',
    name: 'Check Price',
    description: 'Get current asset price',
    category: 'utilities',
    icon: '📈',
    color: '#8b5cf6',
    inputs: [
      { id: 'asset_data', name: 'Asset Data', type: 'data', required: true }
    ],
    outputs: [
      { id: 'price_data', name: 'Price Data', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'vs_currency',
        name: 'Price In',
        type: 'select',
        required: true,
        options: [
          { label: 'USD', value: 'USD' },
          { label: 'ETH', value: 'ETH' },
          { label: 'BTC', value: 'BTC' }
        ],
        defaultValue: 'USD'
      }
    ],
    defaultConfig: { vs_currency: 'USD' },
    tieredPricing: {
      standard: { executionFee: 0.02, description: 'Basic price check' },
      premium: { executionFee: 0.01, description: 'Real-time price check' },
      pro: { executionFee: 0.005, description: 'Priority price feed' }
    }
  },

  // Balance Checker - Reusable for balance queries
  {
    id: 'check-balance',
    name: 'Check Balance',
    description: 'Get wallet balance for asset',
    category: 'utilities',
    icon: '💳',
    color: '#8b5cf6',
    inputs: [
      { id: 'asset_data', name: 'Asset Data', type: 'data', required: true },
      { id: 'chain_data', name: 'Chain Data', type: 'data', required: false }
    ],
    outputs: [
      { id: 'balance_data', name: 'Balance Data', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'wallet_address',
        name: 'Wallet Address (Optional)',
        type: 'text',
        required: false,
        placeholder: '0x... or use connected wallet',
        description: 'Leave empty to use connected wallet'
      }
    ],
    defaultConfig: { wallet_address: '' },
    tieredPricing: {
      standard: { executionFee: 0.02, description: 'Basic balance check' },
      premium: { executionFee: 0.01, description: 'Fast balance check' },
      pro: { executionFee: 0.005, description: 'Instant balance check' }
    }
  },

  // Gas Estimator - Reusable for transaction cost estimation
  {
    id: 'estimate-gas',
    name: 'Estimate Gas',
    description: 'Estimate transaction gas costs',
    category: 'utilities',
    icon: '⛽',
    color: '#8b5cf6',
    inputs: [
      { id: 'chain_data', name: 'Chain Data', type: 'data', required: true },
      { id: 'amount_data', name: 'Amount Data', type: 'data', required: false }
    ],
    outputs: [
      { id: 'gas_estimate', name: 'Gas Estimate', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'transaction_type',
        name: 'Transaction Type',
        type: 'select',
        required: true,
        options: [
          { label: 'Simple Transfer', value: 'transfer' },
          { label: 'Token Swap', value: 'swap' },
          { label: 'DeFi Interaction', value: 'defi' },
          { label: 'NFT Transaction', value: 'nft' }
        ],
        defaultValue: 'transfer'
      },
      {
        key: 'gas_priority',
        name: 'Gas Priority',
        type: 'select',
        required: true,
        options: [
          { label: 'Low (Slow)', value: 'low' },
          { label: 'Medium (Standard)', value: 'medium' },
          { label: 'High (Fast)', value: 'high' },
          { label: 'Urgent (Fastest)', value: 'urgent' }
        ],
        defaultValue: 'medium'
      }
    ],
    defaultConfig: { transaction_type: 'transfer', gas_priority: 'medium' },
    tieredPricing: {
      standard: { executionFee: 0.03, description: 'Basic gas estimation' },
      premium: { executionFee: 0.02, description: 'Advanced gas estimation' },
      pro: { executionFee: 0.01, description: 'Real-time gas optimization' }
    }
  },

]