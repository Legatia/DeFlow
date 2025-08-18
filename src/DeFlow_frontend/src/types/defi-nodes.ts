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

  // DeFi Strategy Actions
  {
    id: 'yield-farming',
    name: 'Yield Farming',
    description: 'Execute yield farming strategy',
    category: 'actions',
    icon: '🌾',
    color: '#10b981',
    inputs: [
      { id: 'trigger', name: 'Execute', type: 'trigger', required: true }
    ],
    outputs: [
      { id: 'result', name: 'Farm Result', type: 'data', required: true }
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
        validation: { min: 1 },
        defaultValue: 1000
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
      }
    ],
    defaultConfig: { 
      protocol: 'Aave', 
      token: 'USDC', 
      amount: 1000, 
      min_apy: 5.0, 
      auto_compound: true 
    },
    tieredPricing: {
      standard: { executionFee: 0.1, description: 'Basic execution with standard speed' },
      premium: { executionFee: 0.05, description: 'Faster execution with priority processing' },
      pro: { executionFee: 0.02, description: 'Fastest execution with advanced features' }
    }
  },

  {
    id: 'arbitrage',
    name: 'Arbitrage',
    description: 'Execute arbitrage opportunity',
    category: 'actions',
    icon: '⚖️',
    color: '#10b981',
    inputs: [
      { id: 'trigger', name: 'Execute', type: 'trigger', required: true }
    ],
    outputs: [
      { id: 'result', name: 'Arbitrage Result', type: 'data', required: true }
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
      },
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
      }
    ],
    defaultConfig: { 
      asset: 'BTC', 
      buy_chain: 'Ethereum', 
      sell_chain: 'ICP', 
      min_profit_percent: 1.0, 
      max_amount: 5000 
    },
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

  {
    id: 'rebalance',
    name: 'Portfolio Rebalance',
    description: 'Rebalance portfolio allocation',
    category: 'actions',
    icon: '⚖️',
    color: '#10b981',
    inputs: [
      { id: 'trigger', name: 'Execute', type: 'trigger', required: true }
    ],
    outputs: [
      { id: 'result', name: 'Rebalance Result', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'target_allocations',
        name: 'Target Allocations (JSON)',
        type: 'textarea',
        required: true,
        placeholder: '{"BTC": 60, "ETH": 30, "USDC": 10}',
        description: 'Target percentage allocations as JSON'
      },
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
    defaultConfig: { 
      target_allocations: '{"BTC": 60, "ETH": 30, "USDC": 10}', 
      rebalance_threshold: 5, 
      min_trade_amount: 50 
    },
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

  {
    id: 'gas-optimizer',
    name: 'Gas Optimizer',
    description: 'Optimize transaction gas costs',
    category: 'utilities',
    icon: '⛽',
    color: '#8b5cf6',
    inputs: [
      { id: 'transaction', name: 'Transaction', type: 'data', required: true }
    ],
    outputs: [
      { id: 'optimized', name: 'Optimized Tx', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'chain',
        name: 'Target Chain',
        type: 'select',
        required: true,
        options: [
          { label: 'Ethereum', value: 'Ethereum' },
          { label: 'Arbitrum', value: 'Arbitrum' },
          { label: 'Polygon', value: 'Polygon' },
          { label: 'Base', value: 'Base' },
          { label: 'ICP', value: 'ICP' }
        ],
        defaultValue: 'Ethereum'
      },
      {
        key: 'priority',
        name: 'Priority',
        type: 'select',
        required: true,
        options: [
          { label: 'Low (Cheapest)', value: 'low' },
          { label: 'Medium', value: 'medium' },
          { label: 'High (Fastest)', value: 'high' }
        ],
        defaultValue: 'medium'
      },
      {
        key: 'max_gas_price',
        name: 'Max Gas Price (Gwei)',
        type: 'number',
        required: false,
        validation: { min: 1 },
        placeholder: 'Optional gas price limit'
      }
    ],
    defaultConfig: { chain: 'Ethereum', priority: 'medium', max_gas_price: null },
    tieredPricing: {
      standard: { executionFee: 0.1, description: 'Basic execution with standard speed' },
      premium: { executionFee: 0.05, description: 'Faster execution with priority processing' },
      pro: { executionFee: 0.02, description: 'Fastest execution with advanced features' }
    }
  },

  // Tier 1 Web3 Components
  {
    id: 'dao-governance',
    name: 'DAO Governance',
    description: 'Participate in DAO governance (vote, propose, delegate)',
    category: 'actions',
    icon: '🗳️',
    color: '#10b981',
    inputs: [
      { id: 'trigger', name: 'Execute', type: 'trigger', required: true }
    ],
    outputs: [
      { id: 'result', name: 'Governance Result', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'dao_address',
        name: 'DAO Contract Address',
        type: 'text',
        required: true,
        placeholder: '0x742d35Cc6636C0532925a3b8D0C9e3d4d7b7C94A',
        description: 'Smart contract address of the DAO'
      },
      {
        key: 'chain',
        name: 'Blockchain',
        type: 'select',
        required: true,
        options: [
          { label: 'Ethereum', value: 'Ethereum' },
          { label: 'Arbitrum', value: 'Arbitrum' },
          { label: 'Optimism', value: 'Optimism' },
          { label: 'Polygon', value: 'Polygon' },
          { label: 'Base', value: 'Base' }
        ],
        defaultValue: 'Ethereum'
      },
      {
        key: 'action_type',
        name: 'Governance Action',
        type: 'select',
        required: true,
        options: [
          { label: 'Vote on Proposal', value: 'vote' },
          { label: 'Create Proposal', value: 'propose' },
          { label: 'Delegate Voting Power', value: 'delegate' },
          { label: 'Check Proposal Status', value: 'check_proposal' }
        ],
        defaultValue: 'vote'
      },
      {
        key: 'proposal_id',
        name: 'Proposal ID',
        type: 'text',
        required: false,
        placeholder: '123',
        description: 'ID of the proposal (required for voting and checking)'
      },
      {
        key: 'vote_choice',
        name: 'Vote Choice',
        type: 'select',
        required: false,
        options: [
          { label: 'For', value: 'for' },
          { label: 'Against', value: 'against' },
          { label: 'Abstain', value: 'abstain' }
        ],
        defaultValue: 'for',
        description: 'Vote choice (required for voting)'
      },
      {
        key: 'delegate_address',
        name: 'Delegate Address',
        type: 'text',
        required: false,
        placeholder: '0x123...',
        description: 'Address to delegate voting power to (required for delegation)'
      },
      {
        key: 'proposal_title',
        name: 'Proposal Title',
        type: 'text',
        required: false,
        placeholder: 'Increase governance rewards',
        description: 'Title for new proposal (required for creating proposals)'
      },
      {
        key: 'proposal_description',
        name: 'Proposal Description',
        type: 'textarea',
        required: false,
        placeholder: 'This proposal aims to...',
        description: 'Description for new proposal (required for creating proposals)'
      }
    ],
    defaultConfig: { 
      dao_address: '', 
      chain: 'Ethereum',
      action_type: 'vote',
      proposal_id: '',
      vote_choice: 'for',
      delegate_address: '',
      proposal_title: '',
      proposal_description: ''
    },
    tieredPricing: {
      standard: { executionFee: 0.1, description: 'Basic execution with standard speed' },
      premium: { executionFee: 0.05, description: 'Faster execution with priority processing' },
      pro: { executionFee: 0.02, description: 'Fastest execution with advanced features' }
    }
  }
]