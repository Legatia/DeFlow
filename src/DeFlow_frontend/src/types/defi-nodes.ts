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

  // DeFi Strategy Actions - Simplified Yield Farming
  {
    id: 'select-yield-protocol',
    name: 'Select Yield Protocol',
    description: 'Choose DeFi protocol for farming',
    category: 'actions',
    icon: '🏦',
    color: '#10b981',
    inputs: [
      { id: 'trigger', name: 'Execute', type: 'trigger', required: true }
    ],
    outputs: [
      { id: 'protocol_data', name: 'Protocol Data', type: 'data', required: true }
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
          { label: 'Curve', value: 'Curve' }
        ],
        defaultValue: 'Aave'
      }
    ],
    defaultConfig: { protocol: 'Aave' },
    tieredPricing: {
      standard: { executionFee: 0.05, description: 'Basic protocol selection' },
      premium: { executionFee: 0.03, description: 'Fast protocol selection' },
      pro: { executionFee: 0.01, description: 'Instant protocol selection' }
    }
  },

  {
    id: 'set-farm-amount',
    name: 'Set Farm Amount',
    description: 'Configure farming amount and token',
    category: 'actions',
    icon: '💰',
    color: '#10b981',
    inputs: [
      { id: 'protocol_data', name: 'Protocol Data', type: 'data', required: true }
    ],
    outputs: [
      { id: 'farm_config', name: 'Farm Config', type: 'data', required: true }
    ],
    configSchema: [
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
        validation: { min: 1 },
        defaultValue: 1000
      }
    ],
    defaultConfig: { token: 'USDC', amount: 1000 },
    tieredPricing: {
      standard: { executionFee: 0.02, description: 'Basic amount configuration' },
      premium: { executionFee: 0.01, description: 'Fast configuration' },
      pro: { executionFee: 0.005, description: 'Instant configuration' }
    }
  },

  {
    id: 'execute-yield-farm',
    name: 'Execute Yield Farm',
    description: 'Execute the yield farming operation',
    category: 'actions',
    icon: '🌾',
    color: '#10b981',
    inputs: [
      { id: 'farm_config', name: 'Farm Config', type: 'data', required: true }
    ],
    outputs: [
      { id: 'result', name: 'Farm Result', type: 'data', required: true }
    ],
    configSchema: [
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
      }
    ],
    defaultConfig: { min_apy: 5.0, auto_compound: true },
    tieredPricing: {
      standard: { executionFee: 0.1, description: 'Basic execution with standard speed' },
      premium: { executionFee: 0.05, description: 'Faster execution with priority processing' },
      pro: { executionFee: 0.02, description: 'Fastest execution with advanced features' }
    }
  },

  // Simplified Arbitrage Blocks
  {
    id: 'select-arbitrage-asset',
    name: 'Select Arbitrage Asset',
    description: 'Choose asset for arbitrage trading',
    category: 'actions',
    icon: '💎',
    color: '#10b981',
    inputs: [
      { id: 'trigger', name: 'Execute', type: 'trigger', required: true }
    ],
    outputs: [
      { id: 'asset_data', name: 'Asset Data', type: 'data', required: true }
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
          { label: 'USDC', value: 'USDC' }
        ],
        defaultValue: 'BTC'
      }
    ],
    defaultConfig: { asset: 'BTC' },
    tieredPricing: {
      standard: { executionFee: 0.02, description: 'Basic asset selection' },
      premium: { executionFee: 0.01, description: 'Fast selection' },
      pro: { executionFee: 0.005, description: 'Instant selection' }
    }
  },

  {
    id: 'set-arbitrage-chains',
    name: 'Set Arbitrage Chains',
    description: 'Configure buy and sell chains',
    category: 'actions',
    icon: '🔗',
    color: '#10b981',
    inputs: [
      { id: 'asset_data', name: 'Asset Data', type: 'data', required: true }
    ],
    outputs: [
      { id: 'chain_config', name: 'Chain Config', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'buy_chain',
        name: 'Buy Chain',
        type: 'select',
        required: true,
        options: [
          { label: 'Ethereum', value: 'Ethereum' },
          { label: 'Arbitrum', value: 'Arbitrum' },
          { label: 'Polygon', value: 'Polygon' },
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
          { label: 'Polygon', value: 'Polygon' },
          { label: 'Solana', value: 'Solana' },
          { label: 'ICP', value: 'ICP' }
        ],
        defaultValue: 'Arbitrum'
      }
    ],
    defaultConfig: { buy_chain: 'Ethereum', sell_chain: 'Arbitrum' },
    tieredPricing: {
      standard: { executionFee: 0.03, description: 'Basic chain configuration' },
      premium: { executionFee: 0.02, description: 'Fast configuration' },
      pro: { executionFee: 0.01, description: 'Instant configuration' }
    }
  },

  {
    id: 'execute-arbitrage',
    name: 'Execute Arbitrage',
    description: 'Execute arbitrage opportunity',
    category: 'actions',
    icon: '⚖️',
    color: '#10b981',
    inputs: [
      { id: 'chain_config', name: 'Chain Config', type: 'data', required: true }
    ],
    outputs: [
      { id: 'result', name: 'Arbitrage Result', type: 'data', required: true }
    ],
    configSchema: [
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
      }
    ],
    defaultConfig: { min_profit_percent: 1.0, max_amount: 5000 },
    tieredPricing: {
      standard: { executionFee: 0.1, description: 'Basic execution with standard speed' },
      premium: { executionFee: 0.05, description: 'Faster execution with priority processing' },
      pro: { executionFee: 0.02, description: 'Fastest execution with advanced features' }
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

  // Simplified Cycles Monitoring
  {
    id: 'check-cycles',
    name: 'Check Cycles',
    description: 'Check current canister cycle balance',
    category: 'utilities',
    icon: '🔋',
    color: '#8b5cf6',
    inputs: [
      { id: 'trigger', name: 'Check Cycles', type: 'trigger', required: true }
    ],
    outputs: [
      { id: 'cycles_data', name: 'Cycles Data', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'canister_id',
        name: 'Canister ID',
        type: 'text',
        required: false,
        placeholder: 'rdmx6-jaaaa-aaaah-qdrva-cai',
        description: 'Leave empty to monitor current canister'
      }
    ],
    defaultConfig: { canister_id: '' },
    tieredPricing: {
      standard: { executionFee: 0.02, description: 'Basic cycles check' },
      premium: { executionFee: 0.01, description: 'Fast cycles check' },
      pro: { executionFee: 0.005, description: 'Instant cycles check' }
    }
  },

  {
    id: 'cycles-alert',
    name: 'Cycles Alert',
    description: 'Alert when cycles are running low',
    category: 'conditions',
    icon: '⚠️',
    color: '#f59e0b',
    inputs: [
      { id: 'cycles_data', name: 'Cycles Data', type: 'data', required: true }
    ],
    outputs: [
      { id: 'low_cycles', name: 'Low Cycles Alert', type: 'condition', required: true },
      { id: 'cycles_ok', name: 'Cycles OK', type: 'condition', required: true }
    ],
    configSchema: [
      {
        key: 'warning_threshold',
        name: 'Warning Threshold (T Cycles)',
        type: 'number',
        required: true,
        validation: { min: 1 },
        defaultValue: 10,
        description: 'Alert when cycles drop below this level (in trillions)'
      }
    ],
    defaultConfig: { warning_threshold: 10 },
    tieredPricing: {
      standard: { executionFee: 0.01, description: 'Basic threshold check' },
      premium: { executionFee: 0.005, description: 'Fast threshold check' },
      pro: { executionFee: 0.002, description: 'Instant threshold check' }
    }
  },

  {
    id: 'auto-topup-cycles',
    name: 'Auto Top-up Cycles',
    description: 'Automatically request cycles top-up',
    category: 'actions',
    icon: '⛽',
    color: '#8b5cf6',
    inputs: [
      { id: 'low_cycles', name: 'Low Cycles Alert', type: 'condition', required: true }
    ],
    outputs: [
      { id: 'topup_result', name: 'Top-up Result', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'topup_amount',
        name: 'Top-up Amount (T Cycles)',
        type: 'number',
        required: true,
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
      }
    ],
    defaultConfig: { topup_amount: 20, notification_channel: 'email' },
    tieredPricing: {
      standard: { executionFee: 0.1, description: 'Basic auto top-up' },
      premium: { executionFee: 0.05, description: 'Priority top-up with notifications' },
      pro: { executionFee: 0.02, description: 'Enterprise top-up with custom alerts' }
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