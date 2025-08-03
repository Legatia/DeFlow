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

export type NodeCategory = 'triggers' | 'actions' | 'conditions' | 'transformations' | 'integrations' | 'utilities'

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
  }
]