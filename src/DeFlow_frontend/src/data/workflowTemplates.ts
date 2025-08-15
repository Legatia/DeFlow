// Pre-built workflow templates for users to get started quickly
import { Node, Edge } from 'reactflow'
import { NODE_TYPES } from '../types/nodes'

export interface WorkflowTemplate {
  id: string
  name: string
  description: string
  category: 'automation' | 'notification' | 'data-processing' | 'integration'
  difficulty: 'beginner' | 'intermediate' | 'advanced'
  nodes: Node[]
  edges: Edge[]
  tags: string[]
  estimatedTime: string
  useCase: string
}

// Helper function to find node type by ID
const getNodeType = (id: string) => NODE_TYPES.find(nt => nt.id === id)!

export const WORKFLOW_TEMPLATES: WorkflowTemplate[] = [
  {
    id: 'email-notification',
    name: 'Email Notification',
    description: 'Send email notifications when triggered manually or by webhook',
    category: 'notification',
    difficulty: 'beginner',
    estimatedTime: '5 minutes',
    useCase: 'Perfect for alerts, reminders, and simple notifications',
    tags: ['email', 'notification', 'simple'],
    nodes: [
      {
        id: 'trigger-1',
        type: 'workflowNode',
        position: { x: 100, y: 100 },
        data: {
          nodeType: getNodeType('manual-trigger'),
          config: { name: 'Start Email Notification' },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'email-1',
        type: 'workflowNode',
        position: { x: 400, y: 100 },
        data: {
          nodeType: getNodeType('send-email'),
          config: {
            to: 'admin@example.com',
            subject: 'Workflow Notification',
            body: 'This is an automated notification from your DeFlow workflow.',
            useTemplate: false
          },
          isValid: true,
          errors: []
        }
      }
    ],
    edges: [
      {
        id: 'edge-1',
        source: 'trigger-1',
        target: 'email-1',
        sourceHandle: 'trigger',
        targetHandle: 'data',
        type: 'smoothstep'
      }
    ]
  },

  {
    id: 'webhook-to-email',
    name: 'Webhook to Email',
    description: 'Receive webhook data and send formatted email notifications',
    category: 'integration',
    difficulty: 'beginner',
    estimatedTime: '10 minutes',
    useCase: 'Connect external services to email notifications',
    tags: ['webhook', 'email', 'integration'],
    nodes: [
      {
        id: 'webhook-1',
        type: 'workflowNode',
        position: { x: 100, y: 100 },
        data: {
          nodeType: getNodeType('webhook-trigger'),
          config: {
            path: '/webhook/notification',
            method: 'POST'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'email-2',
        type: 'workflowNode',
        position: { x: 400, y: 100 },
        data: {
          nodeType: getNodeType('send-email'),
          config: {
            to: 'notifications@example.com',
            subject: 'Webhook Alert: {{data.title}}',
            body: 'Received webhook data:\n\nMessage: {{data.message}}\nTime: {{data.timestamp}}',
            useTemplate: true
          },
          isValid: true,
          errors: []
        }
      }
    ],
    edges: [
      {
        id: 'edge-2',
        source: 'webhook-1',
        target: 'email-2',
        sourceHandle: 'data',
        targetHandle: 'data',
        type: 'smoothstep'
      }
    ]
  },

  {
    id: 'scheduled-report',
    name: 'Scheduled Report',
    description: 'Automatically fetch data and send reports on a schedule',
    category: 'automation',
    difficulty: 'intermediate',
    estimatedTime: '15 minutes',
    useCase: 'Daily/weekly reports, status updates, monitoring',
    tags: ['schedule', 'api', 'email', 'reporting'],
    nodes: [
      {
        id: 'schedule-1',
        type: 'workflowNode',
        position: { x: 100, y: 100 },
        data: {
          nodeType: getNodeType('schedule-trigger'),
          config: {
            cron: '0 9 * * 1-5',
            timezone: 'UTC'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'api-1',
        type: 'workflowNode',
        position: { x: 400, y: 100 },
        data: {
          nodeType: getNodeType('http-request'),
          config: {
            url: 'https://api.example.com/stats',
            method: 'GET',
            headers: '{"Authorization": "Bearer YOUR_TOKEN"}',
            body: ''
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'email-3',
        type: 'workflowNode',
        position: { x: 700, y: 100 },
        data: {
          nodeType: getNodeType('send-email'),
          config: {
            to: 'team@example.com',
            subject: 'Daily Report - {{date}}',
            body: 'Here\'s your daily report:\n\nStats: {{response.data}}\n\nGenerated automatically by DeFlow.',
            useTemplate: true
          },
          isValid: true,
          errors: []
        }
      }
    ],
    edges: [
      {
        id: 'edge-3',
        source: 'schedule-1',
        target: 'api-1',
        sourceHandle: 'time',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-4',
        source: 'api-1',
        target: 'email-3',
        sourceHandle: 'response',
        targetHandle: 'data',
        type: 'smoothstep'
      }
    ]
  },

  {
    id: 'conditional-workflow',
    name: 'Conditional Processing',
    description: 'Branch workflow based on conditions with different actions',
    category: 'automation',
    difficulty: 'intermediate',
    estimatedTime: '20 minutes',
    useCase: 'Smart workflows that react differently based on data',
    tags: ['condition', 'branching', 'logic'],
    nodes: [
      {
        id: 'webhook-2',
        type: 'workflowNode',
        position: { x: 100, y: 150 },
        data: {
          nodeType: getNodeType('webhook-trigger'),
          config: {
            path: '/webhook/status',
            method: 'POST'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'condition-1',
        type: 'workflowNode',
        position: { x: 400, y: 150 },
        data: {
          nodeType: getNodeType('condition'),
          config: {
            field: 'data.status',
            operator: 'equals',
            value: 'error'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'email-error',
        type: 'workflowNode',
        position: { x: 700, y: 50 },
        data: {
          nodeType: getNodeType('send-email'),
          config: {
            to: 'alerts@example.com',
            subject: 'URGENT: Error Detected',
            body: 'Error occurred: {{data.message}}\n\nPlease investigate immediately.',
            useTemplate: true
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'email-success',
        type: 'workflowNode',
        position: { x: 700, y: 250 },
        data: {
          nodeType: getNodeType('send-email'),
          config: {
            to: 'team@example.com',
            subject: 'Status Update: Success',
            body: 'Good news! Operation completed successfully.\n\nDetails: {{data.message}}',
            useTemplate: true
          },
          isValid: true,
          errors: []
        }
      }
    ],
    edges: [
      {
        id: 'edge-5',
        source: 'webhook-2',
        target: 'condition-1',
        sourceHandle: 'data',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-6',
        source: 'condition-1',
        target: 'email-error',
        sourceHandle: 'true',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-7',
        source: 'condition-1',
        target: 'email-success',
        sourceHandle: 'false',
        targetHandle: 'data',
        type: 'smoothstep'
      }
    ]
  },

  {
    id: 'data-processing-pipeline',
    name: 'Data Processing Pipeline',
    description: 'Transform and process data through multiple steps',
    category: 'data-processing',
    difficulty: 'advanced',
    estimatedTime: '30 minutes',
    useCase: 'ETL processes, data transformation, automated processing',
    tags: ['data', 'transform', 'pipeline', 'api'],
    nodes: [
      {
        id: 'schedule-2',
        type: 'workflowNode',
        position: { x: 100, y: 200 },
        data: {
          nodeType: getNodeType('schedule-trigger'),
          config: {
            cron: '0 */6 * * *',
            timezone: 'UTC'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'fetch-data',
        type: 'workflowNode',
        position: { x: 300, y: 200 },
        data: {
          nodeType: getNodeType('http-request'),
          config: {
            url: 'https://api.source.com/data',
            method: 'GET',
            headers: '{"API-Key": "your-key"}',
            body: ''
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'transform-1',
        type: 'workflowNode',
        position: { x: 500, y: 200 },
        data: {
          nodeType: getNodeType('transform-data'),
          config: {
            operation: 'map',
            config: '{"mapping": {"user_id": "id", "full_name": "name", "email_address": "email"}}'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'delay-1',
        type: 'workflowNode',
        position: { x: 700, y: 200 },
        data: {
          nodeType: getNodeType('delay'),
          config: {
            duration: 2,
            unit: 'seconds'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'send-data',
        type: 'workflowNode',
        position: { x: 900, y: 200 },
        data: {
          nodeType: getNodeType('http-request'),
          config: {
            url: 'https://api.destination.com/data',
            method: 'POST',
            headers: '{"Content-Type": "application/json"}',
            body: '{{transformedData}}'
          },
          isValid: true,
          errors: []
        }
      }
    ],
    edges: [
      {
        id: 'edge-8',
        source: 'schedule-2',
        target: 'fetch-data',
        sourceHandle: 'time',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-9',
        source: 'fetch-data',
        target: 'transform-1',
        sourceHandle: 'response',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-10',
        source: 'transform-1',
        target: 'delay-1',
        sourceHandle: 'result',
        targetHandle: 'trigger',
        type: 'smoothstep'
      },
      {
        id: 'edge-11',
        source: 'delay-1',
        target: 'send-data',
        sourceHandle: 'continue',
        targetHandle: 'data',
        type: 'smoothstep'
      }
    ]
  },

  {
    id: 'telegram-portfolio-alert',
    name: 'Telegram Portfolio Alert',
    description: 'Send portfolio performance alerts to Telegram with interactive buttons',
    category: 'notification',
    difficulty: 'beginner',
    estimatedTime: '8 minutes',
    useCase: 'Get instant Telegram notifications about DeFi portfolio changes',
    tags: ['telegram', 'portfolio', 'defi', 'notification'],
    nodes: [
      {
        id: 'schedule-portfolio',
        type: 'workflowNode',
        position: { x: 100, y: 100 },
        data: {
          nodeType: getNodeType('schedule-trigger'),
          config: {
            cron: '0 9,17 * * *',
            timezone: 'UTC'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'telegram-alert',
        type: 'workflowNode',
        position: { x: 400, y: 100 },
        data: {
          nodeType: getNodeType('telegram-bot'),
          config: {
            bot_token: '',
            chat_id: '',
            message_type: 'text',
            message: '📊 *Daily Portfolio Update*\n\n💰 Total Value: ${{portfolio_value}}\n📈 24h Change: {{daily_change}}%\n🏆 Best Performer: {{top_strategy}}\n\n⚡ Powered by DeFlow',
            parse_mode: 'Markdown',
            inline_keyboard: '[{"text": "📊 View Dashboard", "url": "https://deflow.app/dashboard"}, {"text": "⚙️ Manage", "url": "https://deflow.app/strategies"}]',
            disable_preview: false,
            silent: false
          },
          isValid: false,
          errors: ['Bot token and Chat ID required']
        }
      }
    ],
    edges: [
      {
        id: 'edge-telegram-1',
        source: 'schedule-portfolio',
        target: 'telegram-alert',
        sourceHandle: 'time',
        targetHandle: 'data',
        type: 'smoothstep'
      }
    ]
  },

  {
    id: 'telegram-defi-signals',
    name: 'DeFi Trading Signals via Telegram',
    description: 'Monitor market conditions and send trading signals to Telegram groups',
    category: 'integration',
    difficulty: 'intermediate',
    estimatedTime: '15 minutes',
    useCase: 'Share DeFi trading opportunities with your community via Telegram',
    tags: ['telegram', 'defi', 'trading', 'signals', 'community'],
    nodes: [
      {
        id: 'technical-trigger',
        type: 'workflowNode',
        position: { x: 100, y: 100 },
        data: {
          nodeType: getNodeType('technical-indicators'),
          config: {
            asset_symbol: 'ETH',
            indicator_type: 'rsi',
            timeframe: '1h',
            trigger_condition: 'below',
            threshold_value: 30,
            period: 14,
            data_source: 'binance',
            check_interval: 15
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'signal-message',
        type: 'workflowNode',
        position: { x: 400, y: 100 },
        data: {
          nodeType: getNodeType('telegram-bot'),
          config: {
            bot_token: '',
            chat_id: '',
            message_type: 'text',
            message: '⚡ *TRADING SIGNAL ALERT*\n\n🎯 Asset: {{asset_symbol}}\n📊 RSI: {{rsi_value}} (Oversold)\n💡 Signal: BUY OPPORTUNITY\n⏰ Time: {{signal_time}}\n\n📈 Strategy Recommendation:\n• Consider DCA entry\n• Set stop-loss at -5%\n• Target: +10-15%\n\n⚠️ *Always DYOR - Not Financial Advice*',
            parse_mode: 'Markdown',
            inline_keyboard: '[{"text": "📊 View Chart", "url": "https://tradingview.com"}, {"text": "💰 Execute Trade", "callback_data": "execute_trade"}]',
            disable_preview: false,
            silent: false
          },
          isValid: false,
          errors: ['Bot token and Chat ID required']
        }
      }
    ],
    edges: [
      {
        id: 'edge-signal-1',
        source: 'technical-trigger',
        target: 'signal-message',
        sourceHandle: 'signal',
        targetHandle: 'data',
        type: 'smoothstep'
      }
    ]
  },

  {
    id: 'telegram-multi-alert',
    name: 'Multi-Channel Telegram Alerts',
    description: 'Send different types of alerts to multiple Telegram channels/users',
    category: 'notification',
    difficulty: 'advanced',
    estimatedTime: '20 minutes',
    useCase: 'Manage multiple Telegram channels with different alert types',
    tags: ['telegram', 'multi-channel', 'alerts', 'automation'],
    nodes: [
      {
        id: 'webhook-multi',
        type: 'workflowNode',
        position: { x: 100, y: 200 },
        data: {
          nodeType: getNodeType('webhook-trigger'),
          config: {
            path: '/webhook/defi-event',
            method: 'POST'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'condition-alert-type',
        type: 'workflowNode',
        position: { x: 300, y: 200 },
        data: {
          nodeType: getNodeType('condition'),
          config: {
            field: 'alert_type',
            operator: 'equals',
            value: 'critical'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'telegram-critical',
        type: 'workflowNode',
        position: { x: 500, y: 100 },
        data: {
          nodeType: getNodeType('telegram-bot'),
          config: {
            bot_token: '',
            chat_id: '',
            message_type: 'text',
            message: '🚨 *CRITICAL ALERT*\n\n{{alert_title}}\n\n{{alert_description}}\n\n⚠️ Immediate action required!',
            parse_mode: 'Markdown',
            silent: false,
            protect_content: true
          },
          isValid: false,
          errors: ['Configuration required']
        }
      },
      {
        id: 'telegram-general',
        type: 'workflowNode',
        position: { x: 500, y: 300 },
        data: {
          nodeType: getNodeType('telegram-bot'),
          config: {
            bot_token: '',
            chat_id: '',
            message_type: 'text',
            message: '📢 *General Update*\n\n{{alert_title}}\n\n{{alert_description}}\n\nℹ️ For your information.',
            parse_mode: 'Markdown',
            silent: true
          },
          isValid: false,
          errors: ['Configuration required']
        }
      }
    ],
    edges: [
      {
        id: 'edge-multi-1',
        source: 'webhook-multi',
        target: 'condition-alert-type',
        sourceHandle: 'data',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-multi-2',
        source: 'condition-alert-type',
        target: 'telegram-critical',
        sourceHandle: 'true',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-multi-3',
        source: 'condition-alert-type',
        target: 'telegram-general',
        sourceHandle: 'false',
        targetHandle: 'data',
        type: 'smoothstep'
      }
    ]
  }
]

// Group templates by category
export const TEMPLATE_CATEGORIES = [
  { id: 'automation', name: 'Automation', icon: '🤖', description: 'Automated workflows and scheduled tasks' },
  { id: 'notification', name: 'Notifications', icon: '📢', description: 'Email alerts and messaging' },
  { id: 'data-processing', name: 'Data Processing', icon: '⚙️', description: 'Transform and process data' },
  { id: 'integration', name: 'Integrations', icon: '🔗', description: 'Connect different services' }
]

export const getTemplatesByCategory = (category: string) => {
  return WORKFLOW_TEMPLATES.filter(template => template.category === category)
}

export const getTemplatesByDifficulty = (difficulty: string) => {
  return WORKFLOW_TEMPLATES.filter(template => template.difficulty === difficulty)
}