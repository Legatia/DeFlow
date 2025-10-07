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
  },

  {
    id: 'discord-portfolio-embed',
    name: 'Discord Portfolio Embed',
    description: 'Send beautiful rich embed portfolio updates to Discord channels',
    category: 'notification',
    difficulty: 'beginner',
    estimatedTime: '10 minutes',
    useCase: 'Professional Discord portfolio notifications with rich formatting and charts',
    tags: ['discord', 'portfolio', 'embed', 'defi', 'notification'],
    nodes: [
      {
        id: 'schedule-discord',
        type: 'workflowNode',
        position: { x: 100, y: 100 },
        data: {
          nodeType: getNodeType('schedule-trigger'),
          config: {
            cron: '0 8,20 * * *',
            timezone: 'UTC'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'embed-builder',
        type: 'workflowNode',
        position: { x: 400, y: 100 },
        data: {
          nodeType: getNodeType('discord-embed-builder'),
          config: {
            title: 'Portfolio Performance Update',
            description: 'Daily summary of your DeFi investments and strategies',
            color: 'green',
            fields_json: '[{"name": "💰 Total Value", "value": "${{portfolio_value}}", "inline": true}, {"name": "📈 24h Change", "value": "{{daily_change}}%", "inline": true}, {"name": "🏆 Best Strategy", "value": "{{top_strategy}}", "inline": true}]',
            thumbnail_url: 'https://charts.deflow.app/portfolio-thumb.png',
            footer_text: 'DeFlow • Automated DeFi Management'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'discord-webhook',
        type: 'workflowNode',
        position: { x: 700, y: 100 },
        data: {
          nodeType: getNodeType('discord-webhook'),
          config: {
            webhook_url: '',
            username: 'DeFlow Portfolio Bot',
            avatar_url: ''
          },
          isValid: false,
          errors: ['Webhook URL required']
        }
      }
    ],
    edges: [
      {
        id: 'edge-discord-1',
        source: 'schedule-discord',
        target: 'embed-builder',
        sourceHandle: 'time',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-discord-2',
        source: 'embed-builder',
        target: 'discord-webhook',
        sourceHandle: 'embed',
        targetHandle: 'message',
        type: 'smoothstep'
      }
    ]
  },

  {
    id: 'discord-trading-signals',
    name: 'Discord Trading Community Signals',
    description: 'Share trading signals and market analysis with Discord trading communities',
    category: 'integration',
    difficulty: 'intermediate',
    estimatedTime: '15 minutes',
    useCase: 'Broadcast DeFi trading opportunities and market insights to Discord servers',
    tags: ['discord', 'trading', 'signals', 'community', 'defi'],
    nodes: [
      {
        id: 'market-trigger',
        type: 'workflowNode',
        position: { x: 100, y: 100 },
        data: {
          nodeType: getNodeType('technical-indicators'),
          config: {
            asset_symbol: 'BTC',
            indicator_type: 'rsi',
            timeframe: '4h',
            trigger_condition: 'crosses_below',
            threshold_value: 30,
            period: 14,
            data_source: 'binance',
            check_interval: 30
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'text-message',
        type: 'workflowNode',
        position: { x: 300, y: 50 },
        data: {
          nodeType: getNodeType('discord-text-message'),
          config: {
            content: '@here 📡 **TRADING SIGNAL DETECTED**',
            mentions: 'here'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'signal-embed',
        type: 'workflowNode',
        position: { x: 300, y: 150 },
        data: {
          nodeType: getNodeType('discord-embed-builder'),
          config: {
            title: '⚡ BUY Signal: {{asset_symbol}}',
            description: 'Technical analysis indicates strong oversold bounce opportunity',
            color: 'blue',
            fields_json: '[{"name": "🎯 Asset", "value": "{{asset_symbol}}", "inline": true}, {"name": "📊 RSI", "value": "{{rsi_value}} (Oversold)", "inline": true}, {"name": "💡 Signal", "value": "Bullish Reversal", "inline": true}, {"name": "📈 Entry Zone", "value": "${{entry_price}}", "inline": true}, {"name": "🎯 Target", "value": "${{target_price}} (+{{target_percent}}%)", "inline": true}, {"name": "🛡️ Stop Loss", "value": "${{stop_price}} (-{{stop_percent}}%)", "inline": true}]',
            image_url: 'https://charts.tradingview.com/{{asset_symbol}}.png',
            footer_text: '⚠️ Not Financial Advice • Always DYOR'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'json-combiner',
        type: 'workflowNode',
        position: { x: 500, y: 100 },
        data: {
          nodeType: getNodeType('json-builder'),
          config: {
            template: '{\n  "content": "{{content}}",\n  "embeds": [{{embed}}]\n}'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'discord-webhook',
        type: 'workflowNode',
        position: { x: 700, y: 100 },
        data: {
          nodeType: getNodeType('discord-webhook'),
          config: {
            webhook_url: '',
            username: 'DeFlow Trading Bot',
            avatar_url: ''
          },
          isValid: false,
          errors: ['Webhook URL required']
        }
      }
    ],
    edges: [
      {
        id: 'edge-signal-1',
        source: 'market-trigger',
        target: 'text-message',
        sourceHandle: 'signal',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-signal-2',
        source: 'market-trigger',
        target: 'signal-embed',
        sourceHandle: 'signal',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-signal-3',
        source: 'text-message',
        target: 'json-combiner',
        sourceHandle: 'message',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-signal-4',
        source: 'signal-embed',
        target: 'json-combiner',
        sourceHandle: 'embed',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-signal-5',
        source: 'json-combiner',
        target: 'discord-webhook',
        sourceHandle: 'json',
        targetHandle: 'message',
        type: 'smoothstep'
      }
    ]
  },

  {
    id: 'discord-risk-alerts',
    name: 'Discord Risk Management Alerts',
    description: 'Send critical risk alerts and portfolio warnings to Discord with priority notifications',
    category: 'notification',
    difficulty: 'beginner',
    estimatedTime: '8 minutes',
    useCase: 'Critical risk management alerts for DeFi portfolios',
    tags: ['discord', 'risk', 'alerts', 'management', 'defi'],
    nodes: [
      {
        id: 'risk-monitor',
        type: 'workflowNode',
        position: { x: 100, y: 100 },
        data: {
          nodeType: getNodeType('webhook-trigger'),
          config: {
            path: '/webhook/risk-alert',
            method: 'POST'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'risk-text',
        type: 'workflowNode',
        position: { x: 300, y: 100 },
        data: {
          nodeType: getNodeType('discord-text-message'),
          config: {
            content: '@everyone 🚨 **CRITICAL PORTFOLIO RISK ALERT**',
            mentions: 'everyone'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'risk-embed',
        type: 'workflowNode',
        position: { x: 500, y: 100 },
        data: {
          nodeType: getNodeType('discord-embed-builder'),
          config: {
            title: '🚨 Critical Risk Event Detected',
            description: '{{risk_description}}',
            color: 'red',
            fields_json: '[{"name": "⚠️ Risk Type", "value": "{{risk_type}}", "inline": true}, {"name": "📊 Impact Level", "value": "{{impact_level}}", "inline": true}, {"name": "💰 Affected Value", "value": "${{affected_value}}", "inline": true}, {"name": "⏰ Time to Act", "value": "{{time_to_act}}", "inline": true}, {"name": "💡 Recommendation", "value": "{{recommendation}}", "inline": true}, {"name": "🎯 Action Required", "value": "{{required_action}}", "inline": true}]',
            footer_text: 'Immediate action required • DeFlow Risk Management'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'json-combiner',
        type: 'workflowNode',
        position: { x: 700, y: 100 },
        data: {
          nodeType: getNodeType('json-builder'),
          config: {
            template: '{\n  "content": "{{content}}",\n  "embeds": [{{embed}}]\n}'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'discord-webhook',
        type: 'workflowNode',
        position: { x: 900, y: 100 },
        data: {
          nodeType: getNodeType('discord-webhook'),
          config: {
            webhook_url: '',
            username: 'DeFlow Risk Manager',
            avatar_url: ''
          },
          isValid: false,
          errors: ['Webhook URL required']
        }
      }
    ],
    edges: [
      {
        id: 'edge-risk-1',
        source: 'risk-monitor',
        target: 'risk-text',
        sourceHandle: 'data',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-risk-2',
        source: 'risk-monitor',
        target: 'risk-embed',
        sourceHandle: 'data',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-risk-3',
        source: 'risk-text',
        target: 'json-combiner',
        sourceHandle: 'message',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-risk-4',
        source: 'risk-embed',
        target: 'json-combiner',
        sourceHandle: 'embed',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-risk-5',
        source: 'json-combiner',
        target: 'discord-webhook',
        sourceHandle: 'json',
        targetHandle: 'message',
        type: 'smoothstep'
      }
    ]
  },

  {
    id: 'twitter-portfolio-update',
    name: 'Twitter Portfolio Update',
    description: 'Share portfolio performance updates on Twitter with charts and hashtags',
    category: 'notification',
    difficulty: 'beginner',
    estimatedTime: '8 minutes',
    useCase: 'Automated Twitter posts about DeFi portfolio performance',
    tags: ['twitter', 'portfolio', 'social', 'defi', 'automation'],
    nodes: [
      {
        id: 'schedule-twitter',
        type: 'workflowNode',
        position: { x: 100, y: 100 },
        data: {
          nodeType: getNodeType('schedule-trigger'),
          config: {
            cron: '0 12 * * *',
            timezone: 'UTC'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'social-text',
        type: 'workflowNode',
        position: { x: 300, y: 100 },
        data: {
          nodeType: getNodeType('social-media-text'),
          config: {
            content: '🚀 Daily Portfolio Update!\n\n💰 Value: ${{portfolio_value}}\n📈 24h: {{daily_change}}%\n🏆 Top: {{best_strategy}}\n\nAutomated with DeFlow 🤖',
            hashtags: '#DeFi #crypto #portfolio #automation #DeFlow',
            mentions: '',
            platform: 'twitter'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'twitter-post',
        type: 'workflowNode',
        position: { x: 500, y: 100 },
        data: {
          nodeType: getNodeType('social-media-post'),
          config: {},
          isValid: true,
          errors: []
        }
      }
    ],
    edges: [
      {
        id: 'edge-twitter-1',
        source: 'schedule-twitter',
        target: 'social-text',
        sourceHandle: 'time',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-twitter-2',
        source: 'social-text',
        target: 'platform-select',
        sourceHandle: 'message',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-twitter-3',
        source: 'platform-select',
        target: 'social-post',
        sourceHandle: 'platform_config',
        targetHandle: 'platform_config',
        type: 'smoothstep'
      }
    ]
  },

  {
    id: 'twitter-trading-signals',
    name: 'Twitter Trading Signals',
    description: 'Share DeFi trading signals and market analysis on Twitter',
    category: 'integration',
    difficulty: 'intermediate',
    estimatedTime: '12 minutes',
    useCase: 'Broadcast trading opportunities and market insights to Twitter followers',
    tags: ['twitter', 'trading', 'signals', 'defi', 'community'],
    nodes: [
      {
        id: 'market-analysis',
        type: 'workflowNode',
        position: { x: 100, y: 100 },
        data: {
          nodeType: getNodeType('technical-indicators'),
          config: {
            asset_symbol: 'ETH',
            indicator_type: 'rsi',
            timeframe: '1h',
            trigger_condition: 'crosses_above',
            threshold_value: 70,
            period: 14,
            data_source: 'binance',
            check_interval: 30
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'signal-content',
        type: 'workflowNode',
        position: { x: 300, y: 100 },
        data: {
          nodeType: getNodeType('social-media-with-image'),
          config: {
            content: '📡 SIGNAL: {{asset_symbol}} RSI {{rsi_value}}\n\n⚠️ OVERBOUGHT - Consider taking profits\n\n💡 Entry: ${{entry_price}}\n🎯 Target: ${{target_price}}\n\n#NotFinancialAdvice',
            media_type: 'image',
            media_url: 'https://charts.tradingview.com/{{asset_symbol}}.png',
            alt_text: '{{asset_symbol}} trading chart with RSI indicator',
            hashtags: '#{{asset_symbol}} #trading #DeFi #signals'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'platform-select-2',
        type: 'workflowNode',
        position: { x: 500, y: 100 },
        data: {
          nodeType: getNodeType('select-platform'),
          config: {
            platform: 'twitter',
            target_id: ''
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'social-post-2',
        type: 'workflowNode',
        position: { x: 700, y: 100 },
        data: {
          nodeType: getNodeType('social-media-post'),
          config: {},
          isValid: true,
          errors: []
        }
      }
    ],
    edges: [
      {
        id: 'edge-signal-1',
        source: 'market-analysis',
        target: 'signal-content',
        sourceHandle: 'signal',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-signal-2',
        source: 'signal-content',
        target: 'platform-select-2',
        sourceHandle: 'message',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-signal-3',
        source: 'platform-select-2',
        target: 'social-post-2',
        sourceHandle: 'platform_config',
        targetHandle: 'platform_config',
        type: 'smoothstep'
      }
    ]
  },

  {
    id: 'cross-platform-announcement',
    name: 'Cross-Platform Social Announcement',
    description: 'Send the same announcement to Discord and Twitter simultaneously',
    category: 'integration',
    difficulty: 'intermediate',
    estimatedTime: '15 minutes',
    useCase: 'Broadcast important updates across multiple social platforms',
    tags: ['discord', 'twitter', 'cross-platform', 'announcement', 'social'],
    nodes: [
      {
        id: 'announcement-trigger',
        type: 'workflowNode',
        position: { x: 100, y: 150 },
        data: {
          nodeType: getNodeType('webhook-trigger'),
          config: {
            path: '/webhook/announcement',
            method: 'POST'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'social-content',
        type: 'workflowNode',
        position: { x: 300, y: 150 },
        data: {
          nodeType: getNodeType('social-media-text'),
          config: {
            content: '🎉 {{announcement_title}}\n\n{{announcement_body}}\n\nLearn more: {{learn_more_url}}',
            hashtags: '#DeFlow #DeFi #announcement',
            mentions: '',
            platform: 'general'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'discord-embed',
        type: 'workflowNode',
        position: { x: 500, y: 50 },
        data: {
          nodeType: getNodeType('discord-embed-builder'),
          config: {
            title: '{{announcement_title}}',
            description: '{{announcement_body}}',
            color: 'blue',
            fields_json: '[{"name": "📅 Date", "value": "{{date}}", "inline": true}, {"name": "🔗 Learn More", "value": "[Click here]({{learn_more_url}})", "inline": true}]',
            footer_text: 'DeFlow • Official Announcement'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'discord-json',
        type: 'workflowNode',
        position: { x: 700, y: 50 },
        data: {
          nodeType: getNodeType('json-builder'),
          config: {
            template: '{\n  "content": "{{content}}",\n  "embeds": [{{embed}}]\n}'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'discord-webhook',
        type: 'workflowNode',
        position: { x: 900, y: 50 },
        data: {
          nodeType: getNodeType('discord-webhook'),
          config: {
            webhook_url: '',
            username: 'DeFlow Announcements',
            avatar_url: ''
          },
          isValid: false,
          errors: ['Webhook URL required']
        }
      },
      {
        id: 'twitter-post',
        type: 'workflowNode',
        position: { x: 500, y: 250 },
        data: {
          nodeType: getNodeType('social-media-post'),
          config: {},
          isValid: true,
          errors: []
        }
      }
    ],
    edges: [
      {
        id: 'edge-cross-1',
        source: 'announcement-trigger',
        target: 'social-content',
        sourceHandle: 'data',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-cross-2',
        source: 'social-content',
        target: 'discord-embed',
        sourceHandle: 'message',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-cross-3',
        source: 'discord-embed',
        target: 'discord-json',
        sourceHandle: 'embed',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-cross-4',
        source: 'discord-json',
        target: 'discord-webhook',
        sourceHandle: 'json',
        targetHandle: 'message',
        type: 'smoothstep'
      },
      {
        id: 'edge-cross-5',
        source: 'social-content',
        target: 'twitter-post',
        sourceHandle: 'message',
        targetHandle: 'message',
        type: 'smoothstep'
      }
    ]
  },

  // LinkedIn Templates
  {
    id: 'linkedin-professional-update',
    name: 'LinkedIn Professional Update',
    description: 'Schedule professional updates to your LinkedIn profile with business metrics and insights',
    category: 'integration',
    difficulty: 'beginner',
    estimatedTime: '10 minutes',
    useCase: 'Perfect for sharing business achievements, DeFi insights, and professional updates',
    tags: ['linkedin', 'professional', 'business', 'defi', 'automation'],
    nodes: [
      {
        id: 'schedule-linkedin',
        type: 'workflowNode',
        position: { x: 100, y: 100 },
        data: {
          nodeType: getNodeType('schedule-trigger'),
          config: {
            schedule_type: 'interval',
            interval: '24h',
            name: 'Daily LinkedIn Update'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'social-content',
        type: 'workflowNode',
        position: { x: 400, y: 100 },
        data: {
          nodeType: getNodeType('social-media-text'),
          config: {
            content: '📊 Daily Business Update - {{date}}\n\n🎯 Key Insights:\n• Portfolio Growth: +{{growth_percentage}}%\n• DeFi Strategy: {{active_strategy}}\n• Market Analysis: {{market_sentiment}}\n\n💡 Professional Tip: {{business_tip}}\n\nWhat\'s your take on today\'s DeFi trends?\n\n#DeFi #Business #FinTech #Blockchain #ProfessionalGrowth',
            hashtags: '#DeFi #Business #FinTech #Blockchain #ProfessionalGrowth',
            platform: 'linkedin'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'linkedin-post',
        type: 'workflowNode',
        position: { x: 700, y: 100 },
        data: {
          nodeType: getNodeType('social-media-post'),
          config: {
            access_token: '',
            post_type: 'person',
            organization_id: ''
          },
          isValid: false,
          errors: ['Access token required']
        }
      }
    ],
    edges: [
      {
        id: 'edge-linkedin-1',
        source: 'schedule-linkedin',
        target: 'social-content',
        sourceHandle: 'trigger',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-linkedin-2',
        source: 'social-content',
        target: 'linkedin-post',
        sourceHandle: 'message',
        targetHandle: 'message',
        type: 'smoothstep'
      }
    ]
  },

  {
    id: 'linkedin-company-announcements',
    name: 'LinkedIn Company Announcements',
    description: 'Automatically post company updates and product announcements to LinkedIn business page',
    category: 'integration',
    difficulty: 'intermediate',
    estimatedTime: '15 minutes',
    useCase: 'Great for product launches, company milestones, and team achievements',
    tags: ['linkedin', 'company', 'announcements', 'business', 'marketing'],
    nodes: [
      {
        id: 'manual-trigger',
        type: 'workflowNode',
        position: { x: 100, y: 100 },
        data: {
          nodeType: getNodeType('manual-trigger'),
          config: { name: 'Announce Company Update' },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'company-content',
        type: 'workflowNode',
        position: { x: 400, y: 100 },
        data: {
          nodeType: getNodeType('social-media-with-image'),
          config: {
            content: '🚀 Exciting Company Update!\n\n{{announcement_title}}\n\n{{announcement_details}}\n\nThis milestone represents our commitment to {{company_value}} and pushing the boundaries of {{industry_focus}}.\n\n🎯 What this means:\n• {{benefit_1}}\n• {{benefit_2}}\n• {{benefit_3}}\n\nThank you to our amazing team and supporters!\n\n#Innovation #TeamWork #BusinessGrowth #CompanyUpdate',
            media_type: 'image',
            media_url: '{{announcement_image}}',
            platform: 'linkedin'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'linkedin-company-post',
        type: 'workflowNode',
        position: { x: 700, y: 100 },
        data: {
          nodeType: getNodeType('social-media-post'),
          config: {
            access_token: '',
            post_type: 'organization',
            organization_id: ''
          },
          isValid: false,
          errors: ['Access token and organization ID required']
        }
      }
    ],
    edges: [
      {
        id: 'edge-company-1',
        source: 'manual-trigger',
        target: 'company-content',
        sourceHandle: 'trigger',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-company-2',
        source: 'company-content',
        target: 'linkedin-company-post',
        sourceHandle: 'message',
        targetHandle: 'message',
        type: 'smoothstep'
      }
    ]
  },

  // Facebook Templates
  {
    id: 'facebook-community-engagement',
    name: 'Facebook Community Engagement',
    description: 'Schedule engaging posts to build community and encourage interaction on your Facebook page',
    category: 'integration',
    difficulty: 'beginner',
    estimatedTime: '12 minutes',
    useCase: 'Perfect for building community, sharing updates, and engaging with your audience',
    tags: ['facebook', 'community', 'engagement', 'social', 'marketing'],
    nodes: [
      {
        id: 'schedule-facebook',
        type: 'workflowNode',
        position: { x: 100, y: 100 },
        data: {
          nodeType: getNodeType('schedule-trigger'),
          config: {
            schedule_type: 'interval',
            interval: '12h',
            name: 'Bi-daily Facebook Post'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'community-content',
        type: 'workflowNode',
        position: { x: 400, y: 100 },
        data: {
          nodeType: getNodeType('social-media-text'),
          config: {
            content: '🤔 Community Question of the Day\n\n{{discussion_topic}}\n\nWe\'d love to hear your thoughts! 💭\n\n📊 Quick Poll in comments:\nA) {{option_a}}\nB) {{option_b}}\nC) {{option_c}}\n\nDrop your answer below and tell us why! 👇\n\nYour insights help shape our community discussions.\n\n#CommunityEngagement #Discussion #YourVoiceMatters #DeFi #Finance',
            hashtags: '#CommunityEngagement #Discussion #YourVoiceMatters #DeFi #Finance',
            platform: 'facebook'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'facebook-post',
        type: 'workflowNode',
        position: { x: 700, y: 100 },
        data: {
          nodeType: getNodeType('social-media-post'),
          config: {
            access_token: '',
            page_id: '',
            post_type: 'page'
          },
          isValid: false,
          errors: ['Access token and page ID required']
        }
      }
    ],
    edges: [
      {
        id: 'edge-facebook-1',
        source: 'schedule-facebook',
        target: 'community-content',
        sourceHandle: 'trigger',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-facebook-2',
        source: 'community-content',
        target: 'facebook-post',
        sourceHandle: 'message',
        targetHandle: 'message',
        type: 'smoothstep'
      }
    ]
  },

  {
    id: 'facebook-event-promotion',
    name: 'Facebook Event Promotion',
    description: 'Multi-stage event promotion campaign on Facebook with countdown and updates',
    category: 'integration',
    difficulty: 'advanced',
    estimatedTime: '25 minutes',
    useCase: 'Perfect for webinars, product launches, workshops, and community events',
    tags: ['facebook', 'events', 'promotion', 'marketing', 'campaign'],
    nodes: [
      {
        id: 'event-trigger',
        type: 'workflowNode',
        position: { x: 100, y: 100 },
        data: {
          nodeType: getNodeType('manual-trigger'),
          config: { name: 'Start Event Campaign' },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'announcement-content',
        type: 'workflowNode',
        position: { x: 400, y: 50 },
        data: {
          nodeType: getNodeType('social-media-with-image'),
          config: {
            content: '🎉 EVENT ANNOUNCEMENT 🎉\n\n{{event_title}}\n📅 {{event_date}}\n🕐 {{event_time}}\n📍 {{event_location}}\n\n{{event_description}}\n\n🎯 What you\'ll learn:\n• {{learning_1}}\n• {{learning_2}}\n• {{learning_3}}\n\n🎟️ Register now: {{registration_link}}\n\nDon\'t miss out on this amazing opportunity!\n\n#Event #Learning #Community #DeFi #Education',
            media_type: 'image',
            media_url: '{{event_banner}}',
            platform: 'facebook'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'reminder-content',
        type: 'workflowNode',
        position: { x: 400, y: 200 },
        data: {
          nodeType: getNodeType('social-media-text'),
          config: {
            content: '⏰ REMINDER: Only {{days_left}} days left!\n\n{{event_title}} is coming up!\n\n📅 {{event_date}} at {{event_time}}\n\nLast chance to register: {{registration_link}}\n\n✨ Special highlights:\n{{event_highlights}}\n\nSee you there! 🚀\n\n#EventReminder #LastChance #DontMissOut #{{event_hashtag}}',
            platform: 'facebook'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'facebook-announcement',
        type: 'workflowNode',
        position: { x: 700, y: 50 },
        data: {
          nodeType: getNodeType('social-media-post'),
          config: {
            access_token: '',
            page_id: '',
            post_type: 'page'
          },
          isValid: false,
          errors: ['Access token and page ID required']
        }
      },
      {
        id: 'delay-node',
        type: 'workflowNode',
        position: { x: 700, y: 150 },
        data: {
          nodeType: getNodeType('delay'),
          config: {
            delay_amount: 3,
            delay_unit: 'days'
          },
          isValid: true,
          errors: []
        }
      },
      {
        id: 'facebook-reminder',
        type: 'workflowNode',
        position: { x: 1000, y: 200 },
        data: {
          nodeType: getNodeType('social-media-post'),
          config: {
            access_token: '',
            page_id: '',
            post_type: 'page'
          },
          isValid: false,
          errors: ['Access token and page ID required']
        }
      }
    ],
    edges: [
      {
        id: 'edge-event-1',
        source: 'event-trigger',
        target: 'announcement-content',
        sourceHandle: 'trigger',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-event-2',
        source: 'announcement-content',
        target: 'facebook-announcement',
        sourceHandle: 'message',
        targetHandle: 'message',
        type: 'smoothstep'
      },
      {
        id: 'edge-event-3',
        source: 'facebook-announcement',
        target: 'delay-node',
        sourceHandle: 'result',
        targetHandle: 'trigger',
        type: 'smoothstep'
      },
      {
        id: 'edge-event-4',
        source: 'delay-node',
        target: 'reminder-content',
        sourceHandle: 'delayed',
        targetHandle: 'data',
        type: 'smoothstep'
      },
      {
        id: 'edge-event-5',
        source: 'reminder-content',
        target: 'facebook-reminder',
        sourceHandle: 'message',
        targetHandle: 'message',
        type: 'smoothstep'
      }
    ]
  }
]

// Group templates by category
export const TEMPLATE_CATEGORIES = [
  { id: 'defi', name: 'DeFi Strategies', icon: '💰', description: 'Automated DeFi trading and yield strategies' },
  { id: 'automation', name: 'Automation', icon: '🤖', description: 'Automated workflows and scheduled tasks' },
  { id: 'notification', name: 'Notifications', icon: '📢', description: 'Email alerts and messaging' },
  { id: 'data-processing', name: 'Data Processing', icon: '⚙️', description: 'Transform and process data' },
  { id: 'integration', name: 'Integrations', icon: '🔗', description: 'Connect different services' }
]

export const getTemplatesByCategory = (category: string) => {
  return WORKFLOW_TEMPLATES.filter(template => template && template.category === category)
}

export const getTemplatesByDifficulty = (difficulty: string) => {
  return WORKFLOW_TEMPLATES.filter(template => template && template.difficulty === difficulty)
}