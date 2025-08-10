// Comprehensive node type system for DeFlow
export interface NodeType {
  id: string
  name: string
  description: string
  category: NodeCategory
  icon: string
  color: string
  inputs: NodePort[]
  outputs: NodePort[]
  configSchema: ConfigField[]
  defaultConfig: Record<string, any>
}

export interface NodePort {
  id: string
  name: string
  type: PortType
  required: boolean
  description?: string
}

export type PortType = 'data' | 'trigger' | 'condition' | 'webhook' | 'email' | 'api' | 'file'

export type NodeCategory = 'triggers' | 'actions' | 'conditions' | 'transformations' | 'integrations' | 'utilities' | 'defi'

export interface ConfigField {
  key: string
  name: string
  type: 'text' | 'number' | 'boolean' | 'select' | 'textarea' | 'url' | 'email' | 'password'
  required: boolean
  description?: string
  placeholder?: string
  options?: { label: string; value: string }[]
  defaultValue?: any
  validation?: {
    pattern?: string
    min?: number
    max?: number
    minLength?: number
    maxLength?: number
  }
}

// Pre-defined node types for DeFlow
export const NODE_TYPES: NodeType[] = [
  // Triggers
  {
    id: 'manual-trigger',
    name: 'Manual Trigger',
    description: 'Manually start a workflow',
    category: 'triggers',
    icon: '▶️',
    color: '#3b82f6',
    inputs: [],
    outputs: [
      { id: 'trigger', name: 'Triggered', type: 'trigger', required: true }
    ],
    configSchema: [
      {
        key: 'name',
        name: 'Trigger Name',
        type: 'text',
        required: true,
        placeholder: 'Enter trigger name',
        defaultValue: 'Start Workflow'
      }
    ],
    defaultConfig: { name: 'Start Workflow' }
  },
  {
    id: 'webhook-trigger',
    name: 'Webhook Trigger',
    description: 'Triggered by HTTP webhook',
    category: 'triggers',
    icon: '🔗',
    color: '#3b82f6',
    inputs: [],
    outputs: [
      { id: 'data', name: 'Webhook Data', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'path',
        name: 'Webhook Path',
        type: 'text',
        required: true,
        placeholder: '/webhook/my-endpoint',
        defaultValue: '/webhook/new'
      },
      {
        key: 'method',
        name: 'HTTP Method',
        type: 'select',
        required: true,
        options: [
          { label: 'POST', value: 'POST' },
          { label: 'GET', value: 'GET' },
          { label: 'PUT', value: 'PUT' }
        ],
        defaultValue: 'POST'
      }
    ],
    defaultConfig: { path: '/webhook/new', method: 'POST' }
  },
  {
    id: 'schedule-trigger',
    name: 'Schedule Trigger',
    description: 'Triggered on a schedule (cron)',
    category: 'triggers',
    icon: '⏰',
    color: '#3b82f6',
    inputs: [],
    outputs: [
      { id: 'time', name: 'Trigger Time', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'cron',
        name: 'Cron Expression',
        type: 'text',
        required: true,
        placeholder: '0 9 * * 1-5',
        description: 'Cron expression for scheduling'
      },
      {
        key: 'timezone',
        name: 'Timezone',
        type: 'select',
        required: true,
        options: [
          { label: 'UTC', value: 'UTC' },
          { label: 'America/New_York', value: 'America/New_York' },
          { label: 'Europe/London', value: 'Europe/London' },
          { label: 'Asia/Tokyo', value: 'Asia/Tokyo' }
        ],
        defaultValue: 'UTC'
      }
    ],
    defaultConfig: { cron: '0 9 * * 1-5', timezone: 'UTC' }
  },

  // Actions
  {
    id: 'send-email',
    name: 'Send Email',
    description: 'Send an email notification',
    category: 'actions',
    icon: '📧',
    color: '#10b981',
    inputs: [
      { id: 'data', name: 'Input Data', type: 'data', required: false }
    ],
    outputs: [
      { id: 'result', name: 'Email Result', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'to',
        name: 'To Email',
        type: 'email',
        required: true,
        placeholder: 'recipient@example.com'
      },
      {
        key: 'subject',
        name: 'Subject',
        type: 'text',
        required: true,
        placeholder: 'Email subject'
      },
      {
        key: 'body',
        name: 'Email Body',
        type: 'textarea',
        required: true,
        placeholder: 'Email content...'
      },
      {
        key: 'useTemplate',
        name: 'Use Template Variables',
        type: 'boolean',
        required: false,
        description: 'Use {{variable}} syntax for dynamic content',
        defaultValue: true
      }
    ],
    defaultConfig: { 
      to: '', 
      subject: '', 
      body: '', 
      useTemplate: true 
    }
  },
  {
    id: 'http-request',
    name: 'HTTP Request',
    description: 'Make an HTTP API call',
    category: 'actions',
    icon: '🌐',
    color: '#10b981',
    inputs: [
      { id: 'data', name: 'Request Data', type: 'data', required: false }
    ],
    outputs: [
      { id: 'response', name: 'HTTP Response', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'url',
        name: 'URL',
        type: 'url',
        required: true,
        placeholder: 'https://api.example.com/endpoint'
      },
      {
        key: 'method',
        name: 'Method',
        type: 'select',
        required: true,
        options: [
          { label: 'GET', value: 'GET' },
          { label: 'POST', value: 'POST' },
          { label: 'PUT', value: 'PUT' },
          { label: 'DELETE', value: 'DELETE' }
        ],
        defaultValue: 'GET'
      },
      {
        key: 'headers',
        name: 'Headers (JSON)',
        type: 'textarea',
        required: false,
        placeholder: '{"Authorization": "Bearer token"}',
        description: 'HTTP headers as JSON object'
      },
      {
        key: 'body',
        name: 'Request Body',
        type: 'textarea',
        required: false,
        placeholder: 'Request body for POST/PUT'
      }
    ],
    defaultConfig: { 
      url: '', 
      method: 'GET', 
      headers: '{}', 
      body: '' 
    }
  },
  {
    id: 'delay',
    name: 'Delay',
    description: 'Wait for a specified amount of time',
    category: 'utilities',
    icon: '⏳',
    color: '#8b5cf6',
    inputs: [
      { id: 'trigger', name: 'Input', type: 'data', required: true }
    ],
    outputs: [
      { id: 'continue', name: 'Continue', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'duration',
        name: 'Duration (seconds)',
        type: 'number',
        required: true,
        validation: { min: 1, max: 3600 },
        defaultValue: 5
      },
      {
        key: 'unit',
        name: 'Time Unit',
        type: 'select',
        required: true,
        options: [
          { label: 'Seconds', value: 'seconds' },
          { label: 'Minutes', value: 'minutes' },
          { label: 'Hours', value: 'hours' }
        ],
        defaultValue: 'seconds'
      }
    ],
    defaultConfig: { duration: 5, unit: 'seconds' }
  },

  // Conditions
  {
    id: 'condition',
    name: 'Condition',
    description: 'Branch workflow based on conditions',
    category: 'conditions',
    icon: '🔀',
    color: '#f59e0b',
    inputs: [
      { id: 'data', name: 'Input Data', type: 'data', required: true }
    ],
    outputs: [
      { id: 'true', name: 'True', type: 'condition', required: true },
      { id: 'false', name: 'False', type: 'condition', required: true }
    ],
    configSchema: [
      {
        key: 'field',
        name: 'Field to Check',
        type: 'text',
        required: true,
        placeholder: 'data.status',
        description: 'Path to the field to check'
      },
      {
        key: 'operator',
        name: 'Operator',
        type: 'select',
        required: true,
        options: [
          { label: 'Equals', value: 'equals' },
          { label: 'Not Equals', value: 'not_equals' },
          { label: 'Greater Than', value: 'greater_than' },
          { label: 'Less Than', value: 'less_than' },
          { label: 'Contains', value: 'contains' },
          { label: 'Starts With', value: 'starts_with' }
        ],
        defaultValue: 'equals'
      },
      {
        key: 'value',
        name: 'Value',
        type: 'text',
        required: true,
        placeholder: 'Value to compare against'
      }
    ],
    defaultConfig: { field: '', operator: 'equals', value: '' }
  },

  // Transformations
  {
    id: 'transform-data',
    name: 'Transform Data',
    description: 'Transform and manipulate data',
    category: 'transformations',
    icon: '🔄',
    color: '#6366f1',
    inputs: [
      { id: 'data', name: 'Input Data', type: 'data', required: true }
    ],
    outputs: [
      { id: 'result', name: 'Transformed Data', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'operation',
        name: 'Operation',
        type: 'select',
        required: true,
        options: [
          { label: 'Extract Field', value: 'extract' },
          { label: 'Map Fields', value: 'map' },
          { label: 'Filter Array', value: 'filter' },
          { label: 'Format Date', value: 'format_date' },
          { label: 'Convert Type', value: 'convert' }
        ],
        defaultValue: 'extract'
      },
      {
        key: 'config',
        name: 'Operation Config (JSON)',
        type: 'textarea',
        required: true,
        placeholder: '{"field": "data.email", "newName": "userEmail"}',
        description: 'Configuration for the transformation'
      }
    ],
    defaultConfig: { operation: 'extract', config: '{}' }
  },

  // Tier 1 Components - Communication & Web3
  {
    id: 'discord-integration',
    name: 'Discord Message',
    description: 'Send messages to Discord channels or users',
    category: 'integrations',
    icon: '💬',
    color: '#5865f2',
    inputs: [
      { id: 'data', name: 'Message Data', type: 'data', required: false }
    ],
    outputs: [
      { id: 'result', name: 'Discord Result', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'webhook_url',
        name: 'Discord Webhook URL',
        type: 'url',
        required: true,
        placeholder: 'https://discord.com/api/webhooks/...',
        description: 'Discord webhook URL for the target channel'
      },
      {
        key: 'message',
        name: 'Message Content',
        type: 'textarea',
        required: true,
        placeholder: 'Your message here...\nSupports {{variable}} templates',
        description: 'Message content with template variable support'
      },
      {
        key: 'username',
        name: 'Bot Username',
        type: 'text',
        required: false,
        placeholder: 'DeFlow Bot',
        defaultValue: 'DeFlow Bot',
        description: 'Custom username for the bot message'
      },
      {
        key: 'avatar_url',
        name: 'Bot Avatar URL',
        type: 'url',
        required: false,
        placeholder: 'https://example.com/avatar.png',
        description: 'Custom avatar image for the bot'
      },
      {
        key: 'use_embed',
        name: 'Use Rich Embed',
        type: 'boolean',
        required: false,
        defaultValue: false,
        description: 'Send as rich embed with better formatting'
      },
      {
        key: 'embed_color',
        name: 'Embed Color',
        type: 'text',
        required: false,
        placeholder: '#00ff00',
        defaultValue: '#5865f2',
        description: 'Hex color for embed (only if Use Rich Embed is enabled)'
      },
      {
        key: 'embed_title',
        name: 'Embed Title',
        type: 'text',
        required: false,
        placeholder: 'Strategy Alert',
        description: 'Title for rich embed (only if Use Rich Embed is enabled)'
      }
    ],
    defaultConfig: { 
      webhook_url: '', 
      message: '', 
      username: 'DeFlow Bot',
      avatar_url: '',
      use_embed: false,
      embed_color: '#5865f2',
      embed_title: ''
    }
  },

  {
    id: 'push-notification',
    name: 'Push Notification',
    description: 'Send browser/mobile push notifications',
    category: 'integrations',
    icon: '📱',
    color: '#ff6b35',
    inputs: [
      { id: 'data', name: 'Notification Data', type: 'data', required: false }
    ],
    outputs: [
      { id: 'result', name: 'Push Result', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'title',
        name: 'Notification Title',
        type: 'text',
        required: true,
        placeholder: 'DeFlow Alert',
        validation: { maxLength: 50 }
      },
      {
        key: 'message',
        name: 'Notification Message',
        type: 'textarea',
        required: true,
        placeholder: 'Your strategy has been executed successfully!',
        validation: { maxLength: 200 },
        description: 'Supports {{variable}} templates'
      },
      {
        key: 'urgency',
        name: 'Urgency Level',
        type: 'select',
        required: true,
        options: [
          { label: 'Low', value: 'low' },
          { label: 'Normal', value: 'normal' },
          { label: 'High', value: 'high' },
          { label: 'Critical', value: 'critical' }
        ],
        defaultValue: 'normal'
      },
      {
        key: 'icon_url',
        name: 'Notification Icon',
        type: 'url',
        required: false,
        placeholder: 'https://example.com/icon.png',
        description: 'Custom icon for notification'
      },
      {
        key: 'action_url',
        name: 'Action URL',
        type: 'url',
        required: false,
        placeholder: 'https://app.deflow.com/strategies',
        description: 'URL to open when notification is clicked'
      },
      {
        key: 'tags',
        name: 'User Tags',
        type: 'text',
        required: false,
        placeholder: 'defi,alerts,trading',
        description: 'Comma-separated tags to target specific users'
      }
    ],
    defaultConfig: { 
      title: 'DeFlow Alert', 
      message: '', 
      urgency: 'normal',
      icon_url: '',
      action_url: '',
      tags: ''
    }
  },

  {
    id: 'telegram-bot',
    name: 'Telegram Message',
    description: 'Send messages via Telegram Bot',
    category: 'integrations',
    icon: '📬',
    color: '#0088cc',
    inputs: [
      { id: 'data', name: 'Message Data', type: 'data', required: false }
    ],
    outputs: [
      { id: 'result', name: 'Telegram Result', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'bot_token',
        name: 'Bot Token',
        type: 'password',
        required: true,
        placeholder: '1234567890:ABCdefGHIjklMNOpqrsTUVwxyz',
        description: 'Telegram bot token from @BotFather'
      },
      {
        key: 'chat_id',
        name: 'Chat ID',
        type: 'text',
        required: true,
        placeholder: '-1001234567890',
        description: 'Chat ID (user ID, group ID, or channel username)'
      },
      {
        key: 'message',
        name: 'Message Text',
        type: 'textarea',
        required: true,
        placeholder: 'Your message here...\nSupports {{variable}} templates',
        description: 'Message text with template variable support'
      },
      {
        key: 'parse_mode',
        name: 'Parse Mode',
        type: 'select',
        required: false,
        options: [
          { label: 'Plain Text', value: '' },
          { label: 'Markdown', value: 'Markdown' },
          { label: 'HTML', value: 'HTML' }
        ],
        defaultValue: 'Markdown',
        description: 'Text formatting mode'
      },
      {
        key: 'disable_preview',
        name: 'Disable Link Preview',
        type: 'boolean',
        required: false,
        defaultValue: false,
        description: 'Disable web page preview for links'
      },
      {
        key: 'silent',
        name: 'Silent Message',
        type: 'boolean',
        required: false,
        defaultValue: false,
        description: 'Send message silently (no notification sound)'
      }
    ],
    defaultConfig: { 
      bot_token: '', 
      chat_id: '', 
      message: '',
      parse_mode: 'Markdown',
      disable_preview: false,
      silent: false
    }
  },

  {
    id: 'on-chain-analytics',
    name: 'On-Chain Analytics',
    description: 'Analyze wallet activity and transaction history',
    category: 'utilities',
    icon: '📊',
    color: '#8b5cf6',
    inputs: [
      { id: 'trigger', name: 'Trigger', type: 'trigger', required: true }
    ],
    outputs: [
      { id: 'analytics', name: 'Analytics Data', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'address',
        name: 'Wallet Address',
        type: 'text',
        required: true,
        placeholder: '0x742d35Cc6636C0532925a3b8D0C9e3d4d7b7C94A',
        description: 'Wallet address to analyze'
      },
      {
        key: 'chain',
        name: 'Blockchain',
        type: 'select',
        required: true,
        options: [
          { label: 'Ethereum', value: 'ethereum' },
          { label: 'Bitcoin', value: 'bitcoin' },
          { label: 'Arbitrum', value: 'arbitrum' },
          { label: 'Optimism', value: 'optimism' },
          { label: 'Polygon', value: 'polygon' },
          { label: 'Solana', value: 'solana' }
        ],
        defaultValue: 'ethereum'
      },
      {
        key: 'analysis_type',
        name: 'Analysis Type',
        type: 'select',
        required: true,
        options: [
          { label: 'Transaction History', value: 'transactions' },
          { label: 'Token Holdings', value: 'holdings' },
          { label: 'DeFi Positions', value: 'defi_positions' },
          { label: 'NFT Collection', value: 'nfts' },
          { label: 'Trading Activity', value: 'trading' },
          { label: 'Whale Behavior', value: 'whale_tracking' }
        ],
        defaultValue: 'transactions'
      },
      {
        key: 'time_range',
        name: 'Time Range',
        type: 'select',
        required: true,
        options: [
          { label: 'Last 24 Hours', value: '24h' },
          { label: 'Last 7 Days', value: '7d' },
          { label: 'Last 30 Days', value: '30d' },
          { label: 'Last 90 Days', value: '90d' },
          { label: 'All Time', value: 'all' }
        ],
        defaultValue: '7d'
      },
      {
        key: 'min_value_usd',
        name: 'Minimum Value (USD)',
        type: 'number',
        required: false,
        placeholder: '100',
        validation: { min: 0 },
        description: 'Filter transactions below this USD value'
      }
    ],
    defaultConfig: { 
      address: '', 
      chain: 'ethereum',
      analysis_type: 'transactions',
      time_range: '7d',
      min_value_usd: null
    }
  },

  {
    id: 'cross-chain-event-listener',
    name: 'Cross-Chain Events',
    description: 'Monitor events across multiple blockchains',
    category: 'triggers',
    icon: '🌍',
    color: '#3b82f6',
    inputs: [],
    outputs: [
      { id: 'event', name: 'Event Data', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'chains',
        name: 'Target Chains',
        type: 'text',
        required: true,
        placeholder: 'ethereum,arbitrum,polygon',
        description: 'Comma-separated list of chains to monitor'
      },
      {
        key: 'contract_addresses',
        name: 'Contract Addresses',
        type: 'textarea',
        required: true,
        placeholder: '0x742d35Cc...,0x123abc...',
        description: 'Contract addresses to monitor (one per line or comma-separated)'
      },
      {
        key: 'event_signatures',
        name: 'Event Signatures',
        type: 'textarea',
        required: true,
        placeholder: 'Transfer(address,address,uint256)\nSwap(uint256,uint256)',
        description: 'Event signatures to listen for (one per line)'
      },
      {
        key: 'filter_conditions',
        name: 'Filter Conditions (JSON)',
        type: 'textarea',
        required: false,
        placeholder: '{"from": "0x123...", "value": {"$gt": 1000}}',
        description: 'JSON conditions to filter events'
      },
      {
        key: 'poll_interval',
        name: 'Polling Interval (seconds)',
        type: 'number',
        required: true,
        validation: { min: 10, max: 3600 },
        defaultValue: 60,
        description: 'How often to check for new events'
      }
    ],
    defaultConfig: { 
      chains: 'ethereum',
      contract_addresses: '',
      event_signatures: '',
      filter_conditions: '{}',
      poll_interval: 60
    }
  }
]

// Combined export of all node types will be done in a separate file