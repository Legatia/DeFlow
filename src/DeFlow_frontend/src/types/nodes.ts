// Subscription tiers
export type SubscriptionTier = 'standard' | 'premium' | 'pro'

// Tiered pricing structure for nodes
export interface TieredPricing {
  standard: { executionFee: number; description: string }
  premium: { executionFee: number; description: string }
  pro: { executionFee: number; description: string }
}

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
  defaultConfig: Record<string, unknown>
  requiredTier?: SubscriptionTier  // Minimum tier required to use this node (defaults to 'standard')
  tieredPricing?: TieredPricing    // Optional tiered pricing (mainly for DeFi nodes)
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
    defaultConfig: { name: 'Start Workflow' },
    requiredTier: 'standard'
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
    description: 'Schedule workflows with universal date format (dd/mm/yy hh:mm:ss) or cron expressions',
    category: 'triggers',
    icon: '⏰',
    color: '#3b82f6',
    inputs: [],
    outputs: [
      { id: 'time', name: 'Trigger Time', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'schedule_mode',
        name: 'Schedule Mode',
        type: 'select',
        required: true,
        options: [
          { label: 'One-time Execution', value: 'one_time' },
          { label: 'Recurring Schedule', value: 'recurring' },
          { label: 'Cron Expression', value: 'cron' }
        ],
        defaultValue: 'one_time',
        description: 'Type of schedule to create'
      },
      {
        key: 'datetime_format',
        name: 'Date/Time Format',
        type: 'select',
        required: true,
        options: [
          { label: 'Universal: dd/mm/yy hh:mm:ss', value: 'universal' },
          { label: 'ISO: yyyy-mm-dd hh:mm:ss', value: 'iso' },
          { label: 'Cron Expression', value: 'cron' }
        ],
        defaultValue: 'universal',
        description: 'Choose your preferred date/time format'
      },
      {
        key: 'datetime_string',
        name: 'Date and Time',
        type: 'text',
        required: false,
        placeholder: '25/12/24 09:30:00',
        description: 'Enter date and time in selected format. Examples: 25/12/24 09:30:00, 2024-12-25 09:30:00'
      },
      {
        key: 'cron',
        name: 'Cron Expression',
        type: 'text',
        required: false,
        placeholder: '0 9 * * 1-5',
        description: 'Advanced cron expression (only for Cron mode)'
      },
      {
        key: 'recurring_interval',
        name: 'Recurring Interval',
        type: 'select',
        required: false,
        options: [
          { label: 'Every 5 minutes', value: '300' },
          { label: 'Every 15 minutes', value: '900' },
          { label: 'Every 30 minutes', value: '1800' },
          { label: 'Every hour', value: '3600' },
          { label: 'Every 4 hours', value: '14400' },
          { label: 'Every 12 hours', value: '43200' },
          { label: 'Daily', value: '86400' },
          { label: 'Weekly', value: '604800' },
          { label: 'Custom (seconds)', value: 'custom' }
        ],
        defaultValue: '3600',
        description: 'How often to repeat (for recurring schedules)'
      },
      {
        key: 'custom_interval_seconds',
        name: 'Custom Interval (seconds)',
        type: 'number',
        required: false,
        placeholder: '7200',
        validation: { min: 60, max: 2592000 },
        description: 'Custom interval in seconds (60 sec to 30 days)'
      },
      {
        key: 'timezone',
        name: 'Timezone',
        type: 'select',
        required: true,
        options: [
          { label: 'UTC (Coordinated Universal Time)', value: 'UTC' },
          { label: 'EST - Eastern Time (US)', value: 'America/New_York' },
          { label: 'PST - Pacific Time (US)', value: 'America/Los_Angeles' },
          { label: 'GMT - Greenwich Mean Time', value: 'Europe/London' },
          { label: 'CET - Central European Time', value: 'Europe/Paris' },
          { label: 'JST - Japan Standard Time', value: 'Asia/Tokyo' },
          { label: 'CST - China Standard Time', value: 'Asia/Shanghai' },
          { label: 'IST - India Standard Time', value: 'Asia/Kolkata' },
          { label: 'AEST - Australian Eastern Time', value: 'Australia/Sydney' }
        ],
        defaultValue: 'UTC',
        description: 'Timezone for schedule execution'
      },
      {
        key: 'max_executions',
        name: 'Max Executions',
        type: 'number',
        required: false,
        placeholder: '10',
        validation: { min: 1, max: 10000 },
        description: 'Maximum number of executions (leave empty for unlimited)'
      },
      {
        key: 'end_date',
        name: 'End Date',
        type: 'text',
        required: false,
        placeholder: '31/12/24 23:59:59',
        description: 'Stop executing after this date (same format as start date)'
      },
      {
        key: 'skip_weekends',
        name: 'Skip Weekends',
        type: 'boolean',
        required: false,
        defaultValue: false,
        description: 'Skip execution on Saturday and Sunday'
      },
      {
        key: 'retry_on_failure',
        name: 'Retry on Failure',
        type: 'boolean',
        required: false,
        defaultValue: true,
        description: 'Automatically retry if execution fails'
      }
    ],
    defaultConfig: { 
      schedule_mode: 'one_time',
      datetime_format: 'universal',
      datetime_string: '',
      cron: '0 9 * * 1-5',
      recurring_interval: '3600',
      custom_interval_seconds: null,
      timezone: 'UTC',
      max_executions: null,
      end_date: '',
      skip_weekends: false,
      retry_on_failure: true
    }
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
    },
    requiredTier: 'premium'  // Email sending requires Premium tier
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
    },
    requiredTier: 'premium'  // HTTP requests require Premium tier
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
    id: 'discord-webhook',
    name: 'Discord Webhook',
    description: 'Send messages to Discord via webhook - accepts text or JSON message data',
    category: 'integrations',
    icon: '💬',
    color: '#5865f2',
    inputs: [
      { id: 'message', name: 'Message Data', type: 'data', required: true }
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
        placeholder: 'https://discord.com/api/webhooks/123456789/abcdefghijk',
        description: 'Discord webhook URL from channel settings → Integrations'
      },
      {
        key: 'username',
        name: 'Bot Username',
        type: 'text',
        required: false,
        placeholder: 'DeFlow Bot',
        defaultValue: 'DeFlow Bot',
        description: 'Custom username for the bot'
      },
      {
        key: 'avatar_url',
        name: 'Bot Avatar URL',
        type: 'url',
        required: false,
        placeholder: 'https://cdn.deflow.app/bot-avatar.png',
        description: 'Custom avatar image URL'
      }
    ],
    defaultConfig: {
      webhook_url: '',
      username: 'DeFlow Bot',
      avatar_url: ''
    },
    requiredTier: 'standard'  // Discord and Telegram are allowed for Standard tier
  },

  {
    id: 'discord-text-message',
    name: 'Discord Text Message',
    description: 'Create a simple Discord text message with mentions and formatting',
    category: 'utilities',
    icon: '📝',
    color: '#5865f2',
    inputs: [
      { id: 'data', name: 'Input Data', type: 'data', required: false }
    ],
    outputs: [
      { id: 'message', name: 'Message Data', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'content',
        name: 'Message Content',
        type: 'textarea',
        required: true,
        placeholder: '🚀 **Portfolio Alert!**\n\n💰 Total Value: ${{portfolio_value}}\n📈 24h Change: {{daily_change}}%\n\nSupports Discord markdown and {{variables}}!',
        description: 'Message content with template variables and Discord markdown'
      },
      {
        key: 'mentions',
        name: 'Mentions',
        type: 'select',
        required: false,
        options: [
          { label: 'None', value: 'none' },
          { label: '@here (online users)', value: 'here' },
          { label: '@everyone (all users)', value: 'everyone' }
        ],
        defaultValue: 'none',
        description: 'Who to mention in the message'
      }
    ],
    defaultConfig: {
      content: '',
      mentions: 'none'
    }
  },

  {
    id: 'discord-embed-builder',
    name: 'Discord Embed Builder',
    description: 'Create rich Discord embeds with fields, images, and custom formatting',
    category: 'utilities',
    icon: '📊',
    color: '#5865f2',
    inputs: [
      { id: 'data', name: 'Input Data', type: 'data', required: false }
    ],
    outputs: [
      { id: 'embed', name: 'Embed Data', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'title',
        name: 'Embed Title',
        type: 'text',
        required: true,
        placeholder: 'Portfolio Performance Update',
        description: 'Main title of the embed'
      },
      {
        key: 'description',
        name: 'Description',
        type: 'textarea',
        required: false,
        placeholder: 'Your DeFi portfolio summary for {{date}}',
        description: 'Main description text'
      },
      {
        key: 'color',
        name: 'Color',
        type: 'select',
        required: false,
        options: [
          { label: '🟢 Green (Success)', value: 'green' },
          { label: '🔴 Red (Error/Loss)', value: 'red' },
          { label: '🔵 Blue (Info)', value: 'blue' },
          { label: '🟡 Yellow (Warning)', value: 'yellow' },
          { label: '🟣 Purple (Special)', value: 'purple' },
          { label: 'Discord Blurple', value: 'blurple' }
        ],
        defaultValue: 'blurple',
        description: 'Embed color theme'
      },
      {
        key: 'fields_json',
        name: 'Fields (JSON)',
        type: 'textarea',
        required: false,
        placeholder: '[\n  {"name": "💰 Value", "value": "${{portfolio_value}}", "inline": true},\n  {"name": "📈 Change", "value": "{{daily_change}}%", "inline": true}\n]',
        description: 'JSON array of embed fields'
      },
      {
        key: 'thumbnail_url',
        name: 'Thumbnail URL',
        type: 'url',
        required: false,
        placeholder: 'https://charts.deflow.app/thumbnail.png',
        description: 'Small image in top-right corner'
      },
      {
        key: 'image_url',
        name: 'Image URL',
        type: 'url',
        required: false,
        placeholder: 'https://charts.deflow.app/portfolio-chart.png',
        description: 'Large image in embed body'
      },
      {
        key: 'footer_text',
        name: 'Footer Text',
        type: 'text',
        required: false,
        placeholder: 'DeFlow • Automated DeFi Management',
        description: 'Footer text at bottom'
      }
    ],
    defaultConfig: {
      title: '',
      description: '',
      color: 'blurple',
      fields_json: '',
      thumbnail_url: '',
      image_url: '',
      footer_text: ''
    }
  },

  {
    id: 'json-builder',
    name: 'JSON Builder',
    description: 'Build custom JSON objects from input data and templates',
    category: 'utilities',
    icon: '🔧',
    color: '#6366f1',
    inputs: [
      { id: 'data', name: 'Input Data', type: 'data', required: false }
    ],
    outputs: [
      { id: 'json', name: 'JSON Object', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'template',
        name: 'JSON Template',
        type: 'textarea',
        required: true,
        placeholder: '{\n  "content": "Alert: {{title}}",\n  "embeds": [{\n    "title": "{{embed_title}}",\n    "description": "{{description}}",\n    "color": 65280\n  }]\n}',
        description: 'JSON template with {{variable}} placeholders'
      }
    ],
    defaultConfig: {
      template: '{\n  "message": "{{content}}"\n}'
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
    description: 'Send rich messages, photos, and interactive content via Telegram Bot',
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
        description: 'Bot token from @BotFather (see Telegram Bot API Guide for setup)'
      },
      {
        key: 'chat_id',
        name: 'Chat ID',
        type: 'text',
        required: true,
        placeholder: '123456789 (user) or -1001234567890 (group)',
        description: 'Chat ID: positive for users, negative starting with -100 for groups'
      },
      {
        key: 'message_type',
        name: 'Message Type',
        type: 'select',
        required: true,
        options: [
          { label: 'Text Message', value: 'text' },
          { label: 'Photo with Caption', value: 'photo' },
          { label: 'Document/File', value: 'document' },
          { label: 'Location', value: 'location' },
          { label: 'Poll', value: 'poll' }
        ],
        defaultValue: 'text',
        description: 'Type of message to send'
      },
      {
        key: 'message',
        name: 'Message Text/Caption',
        type: 'textarea',
        required: true,
        placeholder: '📊 *Portfolio Alert*\n\nGain: +{{change}}%\nValue: ${{value}}\n\nSupports {{variable}} templates and *Markdown* formatting!',
        description: 'Message content with template variables - supports Markdown/HTML formatting'
      },
      {
        key: 'parse_mode',
        name: 'Parse Mode',
        type: 'select',
        required: false,
        options: [
          { label: 'Plain Text', value: '' },
          { label: 'Markdown', value: 'Markdown' },
          { label: 'MarkdownV2 (Recommended)', value: 'MarkdownV2' },
          { label: 'HTML', value: 'HTML' }
        ],
        defaultValue: 'Markdown',
        description: 'Text formatting mode for rich text'
      },
      {
        key: 'photo_url',
        name: 'Photo URL',
        type: 'url',
        required: false,
        placeholder: 'https://example.com/chart.png or {{dynamic_chart_url}}',
        description: 'URL of photo to send (for photo message type)'
      },
      {
        key: 'document_url',
        name: 'Document URL',
        type: 'url',
        required: false,
        placeholder: 'https://example.com/report.pdf',
        description: 'URL of document/file to send (for document message type)'
      },
      {
        key: 'location_lat',
        name: 'Latitude',
        type: 'number',
        required: false,
        placeholder: '40.7128',
        description: 'Latitude for location messages'
      },
      {
        key: 'location_lng',
        name: 'Longitude',
        type: 'number',
        required: false,
        placeholder: '-74.0060',
        description: 'Longitude for location messages'
      },
      {
        key: 'poll_question',
        name: 'Poll Question',
        type: 'text',
        required: false,
        placeholder: 'Which DeFi strategy interests you most?',
        description: 'Question for poll messages'
      },
      {
        key: 'poll_options',
        name: 'Poll Options',
        type: 'textarea',
        required: false,
        placeholder: 'Yield Farming\nLiquidity Mining\nArbitrage\nLending',
        description: 'Poll options (one per line, max 10 options)'
      },
      {
        key: 'inline_keyboard',
        name: 'Inline Keyboard (JSON)',
        type: 'textarea',
        required: false,
        placeholder: '[{"text": "📊 View Portfolio", "url": "https://deflow.app"}, {"text": "⚙️ Settings", "callback_data": "settings"}]',
        description: 'JSON array of inline keyboard buttons for interactive messages'
      },
      {
        key: 'disable_preview',
        name: 'Disable Link Preview',
        type: 'boolean',
        required: false,
        defaultValue: false,
        description: 'Disable web page preview for links in messages'
      },
      {
        key: 'silent',
        name: 'Silent Message',
        type: 'boolean',
        required: false,
        defaultValue: false,
        description: 'Send message silently (users receive without notification sound)'
      },
      {
        key: 'protect_content',
        name: 'Protect Content',
        type: 'boolean',
        required: false,
        defaultValue: false,
        description: 'Protect message content from forwarding and saving'
      },
      {
        key: 'reply_to_message_id',
        name: 'Reply to Message ID',
        type: 'text',
        required: false,
        placeholder: '12345',
        description: 'Message ID to reply to (creates threaded conversation)'
      },
      {
        key: 'thread_id',
        name: 'Thread ID',
        type: 'text',
        required: false,
        placeholder: '67890',
        description: 'Thread ID for forum groups (Telegram Premium feature)'
      }
    ],
    defaultConfig: { 
      bot_token: '', 
      chat_id: '', 
      message_type: 'text',
      message: '',
      parse_mode: 'Markdown',
      photo_url: '',
      document_url: '',
      location_lat: null,
      location_lng: null,
      poll_question: '',
      poll_options: '',
      inline_keyboard: '',
      disable_preview: false,
      silent: false,
      protect_content: false,
      reply_to_message_id: '',
      thread_id: ''
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
  },

  // Social Media Authentication & Setup
  {
    id: 'social-auth-setup',
    name: 'Social Auth Setup',
    description: 'Configure social media platform credentials once for your workflow',
    category: 'utilities',
    icon: '🔐',
    color: '#6366f1',
    inputs: [],
    outputs: [
      { id: 'auth_data', name: 'Auth Data', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'platform',
        name: 'Platform',
        type: 'select',
        required: true,
        options: [
          { label: 'Twitter/X', value: 'twitter' },
          { label: 'LinkedIn', value: 'linkedin' },
          { label: 'Facebook', value: 'facebook' },
          { label: 'Instagram', value: 'instagram' },
          { label: 'YouTube', value: 'youtube' }
        ],
        defaultValue: 'twitter',
        description: 'Select social media platform'
      },
      {
        key: 'auth_token',
        name: 'Access Token',
        type: 'password',
        required: true,
        placeholder: 'Platform access token or API key',
        description: 'Main authentication token for the platform'
      }
    ],
    defaultConfig: {
      platform: 'twitter',
      auth_token: ''
    },
    requiredTier: 'premium'
  },

  {
    id: 'select-platform',
    name: 'Select Platform',
    description: 'Choose target social media platform for posting',
    category: 'utilities',
    icon: '📱',
    color: '#8b5cf6',
    inputs: [
      { id: 'auth_data', name: 'Auth Data', type: 'data', required: false }
    ],
    outputs: [
      { id: 'platform_config', name: 'Platform Config', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'platform',
        name: 'Platform',
        type: 'select',
        required: true,
        options: [
          { label: 'Twitter/X', value: 'twitter' },
          { label: 'LinkedIn Personal', value: 'linkedin_personal' },
          { label: 'LinkedIn Company', value: 'linkedin_company' },
          { label: 'Facebook Page', value: 'facebook_page' },
          { label: 'Facebook Group', value: 'facebook_group' },
          { label: 'Instagram', value: 'instagram' },
          { label: 'YouTube Community', value: 'youtube' }
        ],
        defaultValue: 'twitter',
        description: 'Target platform for posting'
      },
      {
        key: 'target_id',
        name: 'Target ID',
        type: 'text',
        required: false,
        placeholder: 'Page ID, Group ID, or Company ID (if applicable)',
        description: 'ID for specific page/group/company (required for some platforms)'
      }
    ],
    defaultConfig: {
      platform: 'twitter',
      target_id: ''
    },
    requiredTier: 'premium'
  },

  {
    id: 'social-media-post',
    name: 'Social Media Post',
    description: 'Execute the social media post to the selected platform',
    category: 'integrations',
    icon: '📝',
    color: '#10b981',
    inputs: [
      { id: 'platform_config', name: 'Platform Config', type: 'data', required: true },
      { id: 'content_data', name: 'Content Data', type: 'data', required: true }
    ],
    outputs: [
      { id: 'result', name: 'Post Result', type: 'data', required: true }
    ],
    configSchema: [],
    defaultConfig: {},
    requiredTier: 'premium'
  },

  {
    id: 'social-media-text',
    name: 'Social Media Text',
    description: 'Create social media posts with hashtags, mentions, and platform-specific formatting',
    category: 'utilities',
    icon: '📱',
    color: '#1DA1F2',
    inputs: [
      { id: 'data', name: 'Input Data', type: 'data', required: false }
    ],
    outputs: [
      { id: 'message', name: 'Message Data', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'content',
        name: 'Post Content',
        type: 'textarea',
        required: true,
        placeholder: '🚀 Just made +{{profit}}% with DeFlow!\n\n💰 Portfolio: ${{value}}\n📈 Strategy: {{strategy}}\n\nSupports {{variables}} and emojis!',
        description: 'Main post content with template variables'
      },
      {
        key: 'hashtags',
        name: 'Hashtags',
        type: 'text',
        required: false,
        placeholder: '#DeFi #crypto #automation #trading',
        description: 'Space-separated hashtags (# optional)'
      },
      {
        key: 'mentions',
        name: 'Mentions',
        type: 'text',
        required: false,
        placeholder: '@defi_protocol @trading_bot',
        description: 'Space-separated mentions (@ optional)'
      },
      {
        key: 'platform',
        name: 'Platform',
        type: 'select',
        required: false,
        options: [
          { label: 'Twitter/X (280 chars)', value: 'twitter' },
          { label: 'Discord (2000 chars)', value: 'discord' },
          { label: 'LinkedIn (3000 chars)', value: 'linkedin' },
          { label: 'Facebook (63,206 chars)', value: 'facebook' },
          { label: 'General (no limit)', value: 'general' }
        ],
        defaultValue: 'twitter',
        description: 'Target platform for character limits'
      }
    ],
    defaultConfig: {
      content: '',
      hashtags: '',
      mentions: '',
      platform: 'twitter'
    }
  },

  {
    id: 'social-media-with-image',
    name: 'Social Media with Image',
    description: 'Create social media posts with images, GIFs, or videos',
    category: 'utilities',
    icon: '🖼️',
    color: '#1DA1F2',
    inputs: [
      { id: 'data', name: 'Input Data', type: 'data', required: false }
    ],
    outputs: [
      { id: 'message', name: 'Message Data', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'content',
        name: 'Post Content',
        type: 'textarea',
        required: true,
        placeholder: '📊 Portfolio performance update!\n\nCheck out my latest DeFi gains 🚀\n\n{{portfolio_summary}}',
        description: 'Post text content'
      },
      {
        key: 'media_type',
        name: 'Media Type',
        type: 'select',
        required: true,
        options: [
          { label: 'Image/Photo', value: 'image' },
          { label: 'GIF', value: 'gif' },
          { label: 'Video', value: 'video' }
        ],
        defaultValue: 'image',
        description: 'Type of media to attach'
      },
      {
        key: 'media_url',
        name: 'Media URL',
        type: 'url',
        required: true,
        placeholder: 'https://charts.deflow.app/portfolio-chart.png',
        description: 'URL of image, GIF, or video to attach'
      },
      {
        key: 'alt_text',
        name: 'Alt Text',
        type: 'text',
        required: false,
        placeholder: 'Portfolio performance chart showing gains',
        description: 'Accessibility description for the media'
      },
      {
        key: 'hashtags',
        name: 'Hashtags',
        type: 'text',
        required: false,
        placeholder: '#DeFi #portfolio #gains',
        description: 'Space-separated hashtags'
      }
    ],
    defaultConfig: {
      content: '',
      media_type: 'image',
      media_url: '',
      alt_text: '',
      hashtags: ''
    }
  },

  {
    id: 'nft-operations',
    name: 'NFT Operations',
    description: 'Mint, transfer, list NFTs across chains',
    category: 'actions',
    icon: '🏷️',
    color: '#ff6b35',
    inputs: [
      { id: 'trigger', name: 'Execute', type: 'trigger', required: true }
    ],
    outputs: [
      { id: 'result', name: 'NFT Result', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'chain',
        name: 'Blockchain',
        type: 'select',
        required: true,
        options: [
          { label: 'Ethereum', value: 'ethereum' },
          { label: 'Arbitrum', value: 'arbitrum' },
          { label: 'Optimism', value: 'optimism' },
          { label: 'Polygon', value: 'polygon' },
          { label: 'Base', value: 'base' },
          { label: 'Solana', value: 'solana' }
        ],
        defaultValue: 'ethereum'
      },
      {
        key: 'operation',
        name: 'Operation Type',
        type: 'select',
        required: true,
        options: [
          { label: 'Mint NFT', value: 'mint' },
          { label: 'Transfer NFT', value: 'transfer' },
          { label: 'List on Marketplace', value: 'list' },
          { label: 'Buy from Marketplace', value: 'buy' },
          { label: 'Check Ownership', value: 'check_owner' }
        ],
        defaultValue: 'mint'
      },
      {
        key: 'contract_address',
        name: 'Contract Address',
        type: 'text',
        required: true,
        placeholder: '0x742d35Cc6636C0532925a3b8D0C9e3d4d7b7C94A',
        description: 'NFT contract address'
      },
      {
        key: 'token_id',
        name: 'Token ID',
        type: 'text',
        required: false,
        placeholder: '1234',
        description: 'Token ID (required for transfer/list/buy operations)'
      },
      {
        key: 'recipient_address',
        name: 'Recipient Address',
        type: 'text',
        required: false,
        placeholder: '0x123...',
        description: 'Address to send NFT to (for transfer/mint)'
      },
      {
        key: 'metadata_uri',
        name: 'Metadata URI',
        type: 'url',
        required: false,
        placeholder: 'https://api.example.com/nft/metadata/1',
        description: 'IPFS or HTTP URI for NFT metadata (for minting)'
      },
      {
        key: 'marketplace',
        name: 'Marketplace',
        type: 'select',
        required: false,
        options: [
          { label: 'OpenSea', value: 'opensea' },
          { label: 'LooksRare', value: 'looksrare' },
          { label: 'Magic Eden', value: 'magiceden' },
          { label: 'Blur', value: 'blur' }
        ],
        defaultValue: 'opensea',
        description: 'Marketplace for list/buy operations'
      },
      {
        key: 'price_eth',
        name: 'Price (ETH)',
        type: 'number',
        required: false,
        placeholder: '0.5',
        validation: { min: 0 },
        description: 'Price in ETH (for listing/buying)'
      },
      {
        key: 'auto_approve',
        name: 'Auto Approve',
        type: 'boolean',
        required: false,
        defaultValue: false,
        description: 'Automatically approve marketplace for transfers'
      }
    ],
    defaultConfig: { 
      chain: 'ethereum',
      operation: 'mint',
      contract_address: '',
      token_id: '',
      recipient_address: '',
      metadata_uri: '',
      marketplace: 'opensea',
      price_eth: null,
      auto_approve: false
    }
  },

  {
    id: 'advanced-scheduling',
    name: 'Advanced Scheduler',
    description: 'Complex scheduling (lunar calendar, market hours, events)',
    category: 'triggers',
    icon: '📅',
    color: '#3b82f6',
    inputs: [],
    outputs: [
      { id: 'time', name: 'Scheduled Trigger', type: 'trigger', required: true },
      { id: 'context', name: 'Schedule Context', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'schedule_type',
        name: 'Schedule Type',
        type: 'select',
        required: true,
        options: [
          { label: 'Market Hours Only', value: 'market_hours' },
          { label: 'Lunar Calendar', value: 'lunar_calendar' },
          { label: 'Economic Events', value: 'economic_events' },
          { label: 'Custom Business Hours', value: 'business_hours' },
          { label: 'Advanced Cron', value: 'advanced_cron' }
        ],
        defaultValue: 'market_hours'
      },
      {
        key: 'market_timezone',
        name: 'Market Timezone',
        type: 'select',
        required: false,
        options: [
          { label: 'NYSE (America/New_York)', value: 'America/New_York' },
          { label: 'LSE (Europe/London)', value: 'Europe/London' },
          { label: 'TSE (Asia/Tokyo)', value: 'Asia/Tokyo' },
          { label: 'SSE (Asia/Shanghai)', value: 'Asia/Shanghai' },
          { label: 'Crypto 24/7', value: 'crypto' }
        ],
        defaultValue: 'America/New_York',
        description: 'Market timezone for market hours scheduling'
      },
      {
        key: 'lunar_phase',
        name: 'Lunar Phase',
        type: 'select',
        required: false,
        options: [
          { label: 'New Moon', value: 'new_moon' },
          { label: 'First Quarter', value: 'first_quarter' },
          { label: 'Full Moon', value: 'full_moon' },
          { label: 'Last Quarter', value: 'last_quarter' }
        ],
        defaultValue: 'full_moon',
        description: 'Lunar phase for lunar calendar scheduling'
      },
      {
        key: 'economic_events',
        name: 'Economic Events',
        type: 'text',
        required: false,
        placeholder: 'FOMC,NFP,CPI,GDP',
        description: 'Comma-separated economic events to trigger on'
      },
      {
        key: 'business_start',
        name: 'Business Start Time',
        type: 'text',
        required: false,
        placeholder: '09:00',
        description: 'Business hours start time (HH:MM format)'
      },
      {
        key: 'business_end',
        name: 'Business End Time',
        type: 'text',
        required: false,
        placeholder: '17:00',
        description: 'Business hours end time (HH:MM format)'
      },
      {
        key: 'business_days',
        name: 'Business Days',
        type: 'text',
        required: false,
        placeholder: '1,2,3,4,5',
        defaultValue: '1,2,3,4,5',
        description: 'Days of week (1=Monday, 7=Sunday)'
      },
      {
        key: 'advanced_cron',
        name: 'Advanced Cron Expression',
        type: 'text',
        required: false,
        placeholder: '0 9-17 * * 1-5',
        description: 'Complex cron expression with conditions'
      },
      {
        key: 'blackout_dates',
        name: 'Blackout Dates',
        type: 'textarea',
        required: false,
        placeholder: '2024-12-25\n2024-01-01\n2024-07-04',
        description: 'Dates to skip (YYYY-MM-DD format, one per line)'
      }
    ],
    defaultConfig: { 
      schedule_type: 'market_hours',
      market_timezone: 'America/New_York',
      lunar_phase: 'full_moon',
      economic_events: '',
      business_start: '09:00',
      business_end: '17:00',
      business_days: '1,2,3,4,5',
      advanced_cron: '',
      blackout_dates: ''
    }
  },

  {
    id: 'loop-controller',
    name: 'Loop Controller',
    description: 'Repeat actions with conditions and limits',
    category: 'utilities',
    icon: '🔄',
    color: '#8b5cf6',
    inputs: [
      { id: 'trigger', name: 'Start Loop', type: 'trigger', required: true },
      { id: 'data', name: 'Loop Data', type: 'data', required: false }
    ],
    outputs: [
      { id: 'iteration', name: 'Each Iteration', type: 'data', required: true },
      { id: 'complete', name: 'Loop Complete', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'loop_type',
        name: 'Loop Type',
        type: 'select',
        required: true,
        options: [
          { label: 'Fixed Count', value: 'count' },
          { label: 'While Condition', value: 'while' },
          { label: 'For Each Item', value: 'foreach' },
          { label: 'Until Success', value: 'until_success' }
        ],
        defaultValue: 'count'
      },
      {
        key: 'max_iterations',
        name: 'Max Iterations',
        type: 'number',
        required: true,
        validation: { min: 1, max: 1000 },
        defaultValue: 10,
        description: 'Maximum number of iterations (safety limit)'
      },
      {
        key: 'break_condition',
        name: 'Break Condition',
        type: 'textarea',
        required: false,
        placeholder: 'data.success === true',
        description: 'JavaScript expression to break the loop'
      },
      {
        key: 'delay_between',
        name: 'Delay Between Iterations (seconds)',
        type: 'number',
        required: true,
        validation: { min: 0, max: 3600 },
        defaultValue: 1,
        description: 'Wait time between each iteration'
      },
      {
        key: 'data_source',
        name: 'Data Source Path',
        type: 'text',
        required: false,
        placeholder: 'data.items',
        description: 'Path to array for foreach loops'
      },
      {
        key: 'iteration_variable',
        name: 'Iteration Variable Name',
        type: 'text',
        required: false,
        placeholder: 'item',
        defaultValue: 'item',
        description: 'Variable name for current iteration data'
      },
      {
        key: 'continue_on_error',
        name: 'Continue on Error',
        type: 'boolean',
        required: false,
        defaultValue: true,
        description: 'Continue loop if an iteration fails'
      },
      {
        key: 'parallel_execution',
        name: 'Parallel Execution',
        type: 'boolean',
        required: false,
        defaultValue: false,
        description: 'Execute iterations in parallel (advanced)'
      }
    ],
    defaultConfig: { 
      loop_type: 'count',
      max_iterations: 10,
      break_condition: '',
      delay_between: 1,
      data_source: '',
      iteration_variable: 'item',
      continue_on_error: true,
      parallel_execution: false
    }
  },

  {
    id: 'technical-indicators',
    name: 'Technical Indicators',
    description: 'RSI, MACD, Bollinger Bands triggers',
    category: 'triggers',
    icon: '📊',
    color: '#3b82f6',
    inputs: [],
    outputs: [
      { id: 'signal', name: 'Indicator Signal', type: 'trigger', required: true },
      { id: 'data', name: 'Indicator Data', type: 'data', required: true }
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
          { label: 'Solana (SOL)', value: 'SOL' },
          { label: 'Cardano (ADA)', value: 'ADA' },
          { label: 'Chainlink (LINK)', value: 'LINK' }
        ],
        defaultValue: 'BTC'
      },
      {
        key: 'indicator_type',
        name: 'Technical Indicator',
        type: 'select',
        required: true,
        options: [
          { label: 'RSI (Relative Strength Index)', value: 'rsi' },
          { label: 'MACD (Moving Average Convergence Divergence)', value: 'macd' },
          { label: 'Bollinger Bands', value: 'bollinger' },
          { label: 'Moving Average', value: 'sma' },
          { label: 'EMA (Exponential Moving Average)', value: 'ema' },
          { label: 'Stochastic Oscillator', value: 'stochastic' }
        ],
        defaultValue: 'rsi'
      },
      {
        key: 'timeframe',
        name: 'Timeframe',
        type: 'select',
        required: true,
        options: [
          { label: '1 Minute', value: '1m' },
          { label: '5 Minutes', value: '5m' },
          { label: '15 Minutes', value: '15m' },
          { label: '1 Hour', value: '1h' },
          { label: '4 Hours', value: '4h' },
          { label: '1 Day', value: '1d' }
        ],
        defaultValue: '1h'
      },
      {
        key: 'trigger_condition',
        name: 'Trigger Condition',
        type: 'select',
        required: true,
        options: [
          { label: 'Above Threshold', value: 'above' },
          { label: 'Below Threshold', value: 'below' },
          { label: 'Crosses Above', value: 'crosses_above' },
          { label: 'Crosses Below', value: 'crosses_below' },
          { label: 'Bullish Signal', value: 'bullish' },
          { label: 'Bearish Signal', value: 'bearish' }
        ],
        defaultValue: 'below'
      },
      {
        key: 'threshold_value',
        name: 'Threshold Value',
        type: 'number',
        required: true,
        validation: { min: 0, max: 100 },
        defaultValue: 30,
        description: 'Threshold value for the indicator (0-100 for RSI, etc.)'
      },
      {
        key: 'period',
        name: 'Period',
        type: 'number',
        required: true,
        validation: { min: 2, max: 200 },
        defaultValue: 14,
        description: 'Period for indicator calculation (e.g., 14 for RSI)'
      },
      {
        key: 'data_source',
        name: 'Data Source',
        type: 'select',
        required: true,
        options: [
          { label: 'Binance', value: 'binance' },
          { label: 'Coinbase', value: 'coinbase' },
          { label: 'CoinGecko', value: 'coingecko' },
          { label: 'TradingView', value: 'tradingview' }
        ],
        defaultValue: 'binance'
      },
      {
        key: 'check_interval',
        name: 'Check Interval (minutes)',
        type: 'number',
        required: true,
        validation: { min: 1, max: 1440 },
        defaultValue: 5,
        description: 'How often to check the indicator'
      }
    ],
    defaultConfig: { 
      asset_symbol: 'BTC',
      indicator_type: 'rsi',
      timeframe: '1h',
      trigger_condition: 'below',
      threshold_value: 30,
      period: 14,
      data_source: 'binance',
      check_interval: 5
    }
  },

  // AI Content Generation - Simplified Approach
  {
    id: 'ai-content-setup',
    name: 'AI Content Setup',
    description: 'Configure AI provider and basic settings once',
    category: 'utilities',
    icon: '🤖',
    color: '#6366f1',
    inputs: [],
    outputs: [
      { id: 'ai_config', name: 'AI Config', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'provider',
        name: 'AI Provider',
        type: 'select',
        required: true,
        options: [
          { label: 'OpenAI (GPT-4)', value: 'openai' },
          { label: 'Anthropic (Claude)', value: 'anthropic' },
          { label: 'Google (Gemini)', value: 'google' }
        ],
        defaultValue: 'openai',
        description: 'Select AI provider'
      },
      {
        key: 'api_key',
        name: 'API Key',
        type: 'password',
        required: true,
        placeholder: 'Your AI API key',
        description: 'API key for the selected AI provider'
      }
    ],
    defaultConfig: {
      provider: 'openai',
      api_key: ''
    },
    requiredTier: 'premium'
  },

  {
    id: 'generate-content',
    name: 'Generate Content',
    description: 'Generate AI content with simple prompt and data input',
    category: 'utilities',
    icon: '✍️',
    color: '#8b5cf6',
    inputs: [
      { id: 'ai_config', name: 'AI Config', type: 'data', required: true },
      { id: 'data', name: 'Input Data', type: 'data', required: false }
    ],
    outputs: [
      { id: 'content', name: 'Generated Content', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'content_type',
        name: 'Content Type',
        type: 'select',
        required: true,
        options: [
          { label: 'Social Media Post', value: 'social_post' },
          { label: 'Twitter Thread', value: 'twitter_thread' },
          { label: 'Discord Message', value: 'discord_message' },
          { label: 'Market Analysis', value: 'market_analysis' },
          { label: 'Custom Prompt', value: 'custom' }
        ],
        defaultValue: 'social_post',
        description: 'Type of content to generate'
      },
      {
        key: 'prompt',
        name: 'Prompt',
        type: 'textarea',
        required: true,
        placeholder: 'Generate a tweet about {{topic}} with portfolio value {{value}}',
        description: 'Content prompt with {{variable}} placeholders'
      },
      {
        key: 'max_length',
        name: 'Max Length',
        type: 'number',
        required: true,
        validation: { min: 50, max: 2000 },
        defaultValue: 280,
        description: 'Maximum content length'
      }
    ],
    defaultConfig: {
      content_type: 'social_post',
      prompt: 'Generate a professional tweet about {{topic}}',
      max_length: 280
    },
    requiredTier: 'premium'
  },

  {
    id: 'content-optimizer',
    name: 'Content Optimizer',
    description: 'Optimize generated content for specific platforms',
    category: 'utilities',
    icon: '⚡',
    color: '#f59e0b',
    inputs: [
      { id: 'content', name: 'Raw Content', type: 'data', required: true }
    ],
    outputs: [
      { id: 'optimized_content', name: 'Optimized Content', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'platform',
        name: 'Target Platform',
        type: 'select',
        required: true,
        options: [
          { label: 'Twitter/X (280 chars)', value: 'twitter' },
          { label: 'LinkedIn (3000 chars)', value: 'linkedin' },
          { label: 'Discord (2000 chars)', value: 'discord' },
          { label: 'Facebook (no limit)', value: 'facebook' }
        ],
        defaultValue: 'twitter',
        description: 'Platform to optimize for'
      },
      {
        key: 'add_hashtags',
        name: 'Add Hashtags',
        type: 'boolean',
        required: false,
        defaultValue: true,
        description: 'Automatically add relevant hashtags'
      },
      {
        key: 'add_emojis',
        name: 'Add Emojis',
        type: 'boolean',
        required: false,
        defaultValue: true,
        description: 'Add relevant emojis to content'
      }
    ],
    defaultConfig: {
      platform: 'twitter',
      add_hashtags: true,
      add_emojis: true
    },
    requiredTier: 'premium'
  },

  {
    id: 'ai-responder',
    name: 'AI Responder',
    description: 'Generate AI responses to social media mentions or comments',
    category: 'integrations',
    icon: '🧠',
    color: '#10b981',
    inputs: [
      { id: 'ai_config', name: 'AI Config', type: 'data', required: true },
      { id: 'mention', name: 'Social Mention', type: 'data', required: true }
    ],
    outputs: [
      { id: 'response', name: 'AI Response', type: 'data', required: true }
    ],
    configSchema: [
      {
        key: 'personality',
        name: 'Response Style',
        type: 'select',
        required: true,
        options: [
          { label: 'Professional & Helpful', value: 'professional' },
          { label: 'Friendly & Casual', value: 'friendly' },
          { label: 'Expert & Technical', value: 'expert' }
        ],
        defaultValue: 'professional',
        description: 'AI response personality'
      },
      {
        key: 'guidelines',
        name: 'Response Guidelines',
        type: 'textarea',
        required: true,
        placeholder: 'Always be helpful and accurate. Keep responses under 280 characters.',
        description: 'Guidelines for AI responses'
      }
    ],
    defaultConfig: {
      personality: 'professional',
      guidelines: 'Always be helpful and accurate. Keep responses under 280 characters.'
    },
    requiredTier: 'premium'
  }
]

// Combined export of all node types will be done in a separate file