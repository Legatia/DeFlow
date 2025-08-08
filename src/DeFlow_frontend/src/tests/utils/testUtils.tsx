// Test utilities and helpers for DeFlow
import React, { ReactElement } from 'react'
import { render, RenderOptions } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { vi } from 'vitest'
import { Workflow, WorkflowExecution, User } from '../../types'
import { TimestampUtils } from '../../utils/timestamp-utils'
import { BigIntUtils } from '../../utils/bigint-utils'

// Bitcoin DeFi types for testing
export interface BitcoinPortfolio {
  addresses: BitcoinAddress[]
  total_btc: number
  total_satoshis: number
  total_value_usd: number
  utxos: BitcoinUTXO[]
  last_updated: bigint
}

export interface BitcoinAddress {
  address: string
  address_type: 'P2PKH' | 'P2WPKH' | 'P2TR'
  derivation_path: string
  balance_satoshis: number
  utxo_count: number
}

export interface BitcoinUTXO {
  txid: string
  vout: number
  value_satoshis: number
  script_pubkey: string
  confirmations: number
}

export interface BitcoinSendResult {
  success: boolean
  transaction_id?: string
  from_address: string
  to_address: string
  amount_satoshis: number
  fee_satoshis: number
  change_amount_satoshis: number
  confirmation_time_estimate_minutes: number
  error_message?: string
}

// Create a custom render function that includes providers
const AllTheProviders = ({ children }: { children: React.ReactNode }) => {
  return (
    <BrowserRouter>
      {children}
    </BrowserRouter>
  )
}

const customRender = (
  ui: ReactElement,
  options?: Omit<RenderOptions, 'wrapper'>
) => render(ui, { wrapper: AllTheProviders, ...options })

export * from '@testing-library/react'
export { customRender as render }

// Mock data factories
export const createMockWorkflow = (overrides: Partial<Workflow> = {}): Workflow => ({
  id: 'workflow_test_001',
  name: 'Test Workflow',
  description: 'A test workflow for unit testing',
  active: true,
  created_at: TimestampUtils.dateToICPTimestamp(new Date('2024-01-01')),
  updated_at: TimestampUtils.dateToICPTimestamp(new Date('2024-01-01')),
  owner: 'user_test_001',
  nodes: [
    {
      id: 'node_001',
      node_type: 'manual-trigger',
      position: { x: 100, y: 100 },
      configuration: {
        parameters: { name: 'Test Trigger' }
      },
      metadata: {
        label: 'Start',
        description: 'Manual trigger for testing',
        tags: ['trigger'],
        icon: '▶️',
        color: '#3b82f6'
      }
    },
    {
      id: 'node_002',
      node_type: 'send-email',
      position: { x: 400, y: 100 },
      configuration: {
        parameters: {
          to: 'test@example.com',
          subject: 'Test Email',
          body: 'This is a test email'
        }
      },
      metadata: {
        label: 'Send Email',
        description: 'Send test email',
        tags: ['email'],
        icon: '📧',
        color: '#ef4444'
      }
    }
  ],
  connections: [
    {
      id: 'conn_001',
      source_node_id: 'node_001',
      target_node_id: 'node_002',
      source_output: 'trigger',
      target_input: 'data'
    }
  ],
  triggers: [{ type: 'manual' }],
  tags: ['test'],
  version: '1.0.0',
  ...overrides
})

export const createMockWorkflowExecution = (overrides: Partial<WorkflowExecution> = {}): WorkflowExecution => ({
  id: 'exec_test_001',
  workflow_id: 'workflow_test_001',
  status: 'completed',
  started_at: TimestampUtils.dateToICPTimestamp(new Date('2024-01-01T10:00:00Z')),
  completed_at: TimestampUtils.dateToICPTimestamp(new Date('2024-01-01T10:01:00Z')),
  trigger_data: { manual: true },
  node_executions: [
    {
      id: 'node_exec_001',
      execution_id: 'exec_test_001',
      node_id: 'node_001',
      status: 'completed',
      started_at: TimestampUtils.dateToICPTimestamp(new Date('2024-01-01T10:00:00Z')),
      completed_at: TimestampUtils.dateToICPTimestamp(new Date('2024-01-01T10:00:30Z')),
      input_data: {},
      output_data: { trigger: 'manual' },
      error_message: null,
      duration: BigIntUtils.dateToTimestamp(new Date(Date.now() + 30000)) // 30 seconds from now
    }
  ],
  error_message: null,
  duration: BigIntUtils.dateToTimestamp(new Date(Date.now() + 60000)), // 60 seconds from now
  ...overrides
})

export const createMockUser = (overrides: Partial<User> = {}): User => ({
  id: 'user_test_001',
  email: 'test@example.com',
  username: 'testuser',
  displayName: 'Test User',
  role: 'user',
  permissions: [
    { id: 'perm_001', name: 'Read Workflows', resource: 'workflow', action: 'read' },
    { id: 'perm_002', name: 'Create Workflows', resource: 'workflow', action: 'create' }
  ],
  isActive: true,
  createdAt: TimestampUtils.dateToICPTimestamp(new Date('2024-01-01')),
  lastLoginAt: TimestampUtils.dateToICPTimestamp(new Date()),
  profile: {
    timezone: 'UTC',
    notifications: {
      email: true,
      push: false,
      workflow_completion: true,
      workflow_failure: true,
      system_alerts: false
    },
    preferences: {
      theme: 'light',
      language: 'en',
      defaultWorkflowView: 'grid',
      autoSaveInterval: 30
    }
  },
  ...overrides
})

// Mock service functions
export const createMockWorkflowService = () => ({
  getWorkflows: vi.fn().mockResolvedValue([createMockWorkflow()]),
  getWorkflow: vi.fn().mockResolvedValue(createMockWorkflow()),
  createWorkflow: vi.fn().mockResolvedValue(createMockWorkflow()),
  updateWorkflow: vi.fn().mockResolvedValue(createMockWorkflow()),
  deleteWorkflow: vi.fn().mockResolvedValue(true),
  executeWorkflow: vi.fn().mockResolvedValue(createMockWorkflowExecution()),
  getExecutions: vi.fn().mockResolvedValue([createMockWorkflowExecution()]),
  getExecution: vi.fn().mockResolvedValue(createMockWorkflowExecution())
})

export const createMockAuthService = () => ({
  login: vi.fn().mockResolvedValue({ 
    user: createMockUser(), 
    session: { id: 'session_001', token: 'mock-token' } 
  }),
  register: vi.fn().mockResolvedValue({ 
    user: createMockUser(), 
    session: { id: 'session_001', token: 'mock-token' } 
  }),
  logout: vi.fn().mockResolvedValue(undefined),
  getCurrentUser: vi.fn().mockReturnValue(createMockUser()),
  isAuthenticated: vi.fn().mockReturnValue(true),
  hasPermission: vi.fn().mockReturnValue(true)
})

// Test helpers
export const waitForLoadingToFinish = () => 
  new Promise(resolve => setTimeout(resolve, 0))

export const mockApiResponse = <T,>(data: T, delay = 0) => 
  new Promise<T>(resolve => setTimeout(() => resolve(data), delay))

export const mockApiError = (message = 'API Error', delay = 0) =>
  new Promise((_, reject) => 
    setTimeout(() => reject(new Error(message)), delay)
  )

// Component testing helpers
export const getByTestId = (container: HTMLElement, testId: string) =>
  container.querySelector(`[data-testid="${testId}"]`)

export const getAllByTestId = (container: HTMLElement, testId: string) =>
  Array.from(container.querySelectorAll(`[data-testid="${testId}"]`))

// Form testing helpers
export const fillForm = (container: HTMLElement, formData: Record<string, string>) => {
  Object.entries(formData).forEach(([name, value]) => {
    const input = container.querySelector(`[name="${name}"]`) as HTMLInputElement
    if (input) {
      input.value = value
      input.dispatchEvent(new Event('input', { bubbles: true }))
      input.dispatchEvent(new Event('change', { bubbles: true }))
    }
  })
}

export const submitForm = (container: HTMLElement, formSelector = 'form') => {
  const form = container.querySelector(formSelector) as HTMLFormElement
  if (form) {
    form.dispatchEvent(new Event('submit', { bubbles: true }))
  }
}

// Workflow testing helpers
export const createMockReactFlowNode = (id: string, type = 'workflowNode', data = {}) => ({
  id,
  type,
  position: { x: 100, y: 100 },
  data: {
    nodeType: {
      id: 'manual-trigger',
      name: 'Manual Trigger',
      description: 'Trigger workflow manually',
      category: 'triggers',
      icon: '▶️',
      color: '#3b82f6',
      inputs: [],
      outputs: [{ id: 'trigger', name: 'Trigger', type: 'trigger' }],
      configSchema: [],
      defaultConfig: {}
    },
    config: {},
    isValid: true,
    errors: [],
    ...data
  }
})

export const createMockReactFlowEdge = (source: string, target: string, id?: string) => ({
  id: id || `${source}-${target}`,
  source,
  target,
  sourceHandle: 'output',
  targetHandle: 'input',
  type: 'smoothstep'
})

// Date and time helpers
export const createMockTimestamp = (date = new Date()) =>
  TimestampUtils.dateToICPTimestamp(date)

export const advanceTime = (ms: number) => {
  vi.advanceTimersByTime(ms)
}

// Error boundary testing
export const ThrowError = ({ shouldThrow = false }: { shouldThrow?: boolean }) => {
  if (shouldThrow) {
    throw new Error('Test error')
  }
  return <div>No error</div>
}

// Custom hooks testing helpers
export const createMockZustandStore = <T extends Record<string, any>,>(initialState: T) => {
  let state = { ...initialState }
  
  const store = {
    getState: () => state,
    setState: (newState: Partial<T>) => {
      state = { ...state, ...newState }
    },
    subscribe: vi.fn(),
    destroy: vi.fn()
  }
  
  return store
}

// Performance testing helpers
export const measurePerformance = async (fn: () => Promise<void> | void) => {
  const start = performance.now()
  await fn()
  const end = performance.now()
  return end - start
}

// Accessibility testing helpers
export const checkA11y = async (container: HTMLElement) => {
  // Mock axe-core for accessibility testing
  // In a real implementation, you would use @axe-core/react
  const violations = []
  
  // Check for missing alt text
  const images = container.querySelectorAll('img:not([alt])')
  if (images.length > 0) {
    violations.push('Images without alt text found')
  }
  
  // Check for missing labels
  const inputs = container.querySelectorAll('input:not([aria-label]):not([aria-labelledby])')
  inputs.forEach(input => {
    const id = input.getAttribute('id')
    if (!id || !container.querySelector(`label[for="${id}"]`)) {
      violations.push('Input without proper label found')
    }
  })
  
  return violations
}

// Network request mocking
export const mockFetch = (response: any, options: { status?: number; delay?: number } = {}) => {
  const { status = 200, delay = 0 } = options
  
  global.fetch = vi.fn().mockImplementation(() =>
    new Promise(resolve => 
      setTimeout(() => resolve({
        ok: status < 400,
        status,
        json: () => Promise.resolve(response),
        text: () => Promise.resolve(JSON.stringify(response))
      } as Response), delay)
    )
  )
}

export const mockFetchError = (error: string, delay = 0) => {
  global.fetch = vi.fn().mockImplementation(() =>
    new Promise((_, reject) => 
      setTimeout(() => reject(new Error(error)), delay)
    )
  )
}

// Local storage helpers
export const mockLocalStorage = (initialData: Record<string, string> = {}) => {
  const storage = { ...initialData }
  
  Storage.prototype.getItem = vi.fn((key: string) => storage[key] || null)
  Storage.prototype.setItem = vi.fn((key: string, value: string) => {
    storage[key] = value
  })
  Storage.prototype.removeItem = vi.fn((key: string) => {
    delete storage[key]
  })
  Storage.prototype.clear = vi.fn(() => {
    Object.keys(storage).forEach(key => delete storage[key])
  })
  
  return storage
}

// Debug helpers
export const debugComponent = (component: ReactElement) => {
  const { container, debug } = render(component)
  debug()
  return container
}

export const logRenderTime = (name: string, fn: () => void) => {
  console.time(`Render: ${name}`)
  fn()
  console.timeEnd(`Render: ${name}`)
}

// ================================
// Bitcoin DeFi Mock Factories
// ================================

// Generate realistic Bitcoin addresses based on type
const generateBitcoinAddress = (type: 'P2PKH' | 'P2WPKH' | 'P2TR'): string => {
  const randomSuffix = Math.random().toString(36).substring(2, 15)
  
  switch (type) {
    case 'P2PKH':
      return `1A1zP1eP5QGefi2DMPTfTL5SLmv7${randomSuffix.substring(0, 8)}`
    case 'P2WPKH':
      return `bcrt1q${randomSuffix}xy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh`
    case 'P2TR':
      return `bcrt1p${randomSuffix}5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr`
  }
}

export const createMockBitcoinAddress = (overrides: Partial<BitcoinAddress> = {}): BitcoinAddress => {
  const addressType = overrides.address_type || 'P2WPKH'
  const balance = overrides.balance_satoshis || Math.floor(Math.random() * 100000000)
  
  return {
    address: overrides.address || generateBitcoinAddress(addressType),
    address_type: addressType,
    derivation_path: overrides.derivation_path || `m/deflow/bitcoin/user_test_001`,
    balance_satoshis: balance,
    utxo_count: overrides.utxo_count || Math.floor(balance / 1000000), // Rough estimate
    ...overrides
  }
}

export const createMockBitcoinUTXO = (overrides: Partial<BitcoinUTXO> = {}): BitcoinUTXO => ({
  txid: overrides.txid || `${Math.random().toString(16).substring(2)}${Math.random().toString(16).substring(2)}`,
  vout: overrides.vout || Math.floor(Math.random() * 10),
  value_satoshis: overrides.value_satoshis || Math.floor(Math.random() * 10000000),
  script_pubkey: overrides.script_pubkey || `76a914${Math.random().toString(16).substring(2, 42)}88ac`,
  confirmations: overrides.confirmations || Math.floor(Math.random() * 100) + 1,
  ...overrides
})

export const createMockBitcoinPortfolio = (overrides: Partial<BitcoinPortfolio> = {}): BitcoinPortfolio => {
  const addresses = overrides.addresses || [
    createMockBitcoinAddress({ address_type: 'P2PKH', balance_satoshis: 25000000 }),
    createMockBitcoinAddress({ address_type: 'P2WPKH', balance_satoshis: 50000000 }),
    createMockBitcoinAddress({ address_type: 'P2TR', balance_satoshis: 25000000 })
  ]
  
  const totalSatoshis = addresses.reduce((sum, addr) => sum + addr.balance_satoshis, 0)
  const totalBtc = totalSatoshis / 100000000
  const btcPrice = 45000 // Mock BTC price in USD
  
  const utxos = overrides.utxos || addresses.flatMap(addr => 
    Array.from({ length: addr.utxo_count }, () => 
      createMockBitcoinUTXO({ 
        value_satoshis: Math.floor(addr.balance_satoshis / addr.utxo_count) 
      })
    )
  )
  
  return {
    addresses,
    total_btc: overrides.total_btc || totalBtc,
    total_satoshis: overrides.total_satoshis || totalSatoshis,
    total_value_usd: overrides.total_value_usd || (totalBtc * btcPrice),
    utxos,
    last_updated: overrides.last_updated || BigIntUtils.dateToTimestamp(),
    ...overrides
  }
}

export const createMockBitcoinSendResult = (overrides: Partial<BitcoinSendResult> = {}): BitcoinSendResult => ({
  success: overrides.success !== undefined ? overrides.success : true,
  transaction_id: overrides.transaction_id || (overrides.success !== false ? 'abc123def456789' : undefined),
  from_address: overrides.from_address || generateBitcoinAddress('P2WPKH'),
  to_address: overrides.to_address || generateBitcoinAddress('P2PKH'),
  amount_satoshis: overrides.amount_satoshis || 100000,
  fee_satoshis: overrides.fee_satoshis || 1000,
  change_amount_satoshis: overrides.change_amount_satoshis || 49899000,
  confirmation_time_estimate_minutes: overrides.confirmation_time_estimate_minutes || 30,
  error_message: overrides.error_message,
  ...overrides
})

// Bitcoin workflow node mock factories
export const createMockBitcoinPortfolioNode = (overrides: any = {}) => ({
  id: 'bitcoin-portfolio-1',
  node_type: 'bitcoin_portfolio',
  position: { x: 100, y: 100 },
  configuration: { 
    parameters: {
      refresh_interval: { type: 'number', value: 60 },
      ...overrides.configuration?.parameters
    }
  },
  metadata: { 
    label: 'Bitcoin Portfolio', 
    description: 'Get user Bitcoin portfolio',
    version: '1.0.0'
  },
  ...overrides
})

export const createMockBitcoinSendNode = (overrides: any = {}) => ({
  id: 'bitcoin-send-1',
  node_type: 'bitcoin_send',
  position: { x: 300, y: 100 },
  configuration: { 
    parameters: {
      fee_satoshis: { type: 'number', value: 1000 },
      ...overrides.configuration?.parameters
    }
  },
  metadata: { 
    label: 'Send Bitcoin', 
    description: 'Send Bitcoin transaction',
    version: '1.0.0'
  },
  ...overrides
})

export const createMockBitcoinAddressNode = (overrides: any = {}) => ({
  id: 'bitcoin-address-1',
  node_type: 'bitcoin_address',
  position: { x: 100, y: 200 },
  configuration: { 
    parameters: {
      address_type: { type: 'string', value: 'P2WPKH' },
      ...overrides.configuration?.parameters
    }
  },
  metadata: { 
    label: 'Bitcoin Address', 
    description: 'Generate Bitcoin address',
    version: '1.0.0'
  },
  ...overrides
})

export const createMockBitcoinBalanceNode = (overrides: any = {}) => ({
  id: 'bitcoin-balance-1',
  node_type: 'bitcoin_balance',
  position: { x: 500, y: 100 },
  configuration: { 
    parameters: {
      ...overrides.configuration?.parameters
    }
  },
  metadata: { 
    label: 'Bitcoin Balance', 
    description: 'Check Bitcoin address balance',
    version: '1.0.0'
  },
  ...overrides
})

// Bitcoin workflow factory
export const createMockBitcoinWorkflow = (overrides: any = {}) => {
  const defaultNodes = [
    createMockBitcoinPortfolioNode(),
    createMockBitcoinAddressNode(),
    createMockBitcoinSendNode()
  ]
  
  return createMockWorkflow({
    name: 'Bitcoin DeFi Workflow',
    description: 'Complete Bitcoin DeFi operations',
    nodes: overrides.nodes || defaultNodes,
    connections: overrides.connections || [
      {
        id: 'conn-1',
        source_node_id: 'bitcoin-portfolio-1',
        source_output: 'output',
        target_node_id: 'bitcoin-send-1',
        target_input: 'input'
      }
    ],
    ...overrides
  })
}

// Bitcoin execution result factories
export const createMockBitcoinNodeExecution = (
  nodeId: string, 
  status: 'Pending' | 'Running' | 'Completed' | 'Failed' = 'Completed',
  overrides: any = {}
) => ({
  node_id: nodeId,
  status,
  started_at: overrides.started_at || BigIntUtils.dateToTimestamp(new Date(Date.now() - 1000)),
  completed_at: status === 'Completed' || status === 'Failed' ? 
    (overrides.completed_at || BigIntUtils.dateToTimestamp()) : undefined,
  input_data: overrides.input_data,
  output_data: overrides.output_data,
  error_message: status === 'Failed' ? 
    (overrides.error_message || 'Mock execution error') : undefined,
  ...overrides
})

// Helper to create complete Bitcoin workflow execution
export const createMockBitcoinWorkflowExecution = (overrides: any = {}) => {
  const portfolioResult = createMockBitcoinPortfolio()
  const sendResult = createMockBitcoinSendResult()
  
  return createMockWorkflowExecution({
    workflow_id: 'bitcoin-workflow-1',
    status: 'Completed',
    node_executions: [
      createMockBitcoinNodeExecution('bitcoin-portfolio-1', 'Completed', {
        output_data: {
          total_btc: portfolioResult.total_btc,
          total_value_usd: portfolioResult.total_value_usd,
          addresses: portfolioResult.addresses
        }
      }),
      createMockBitcoinNodeExecution('bitcoin-send-1', 'Completed', {
        input_data: {
          to_address: '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa',
          amount_satoshis: 100000
        },
        output_data: sendResult
      })
    ],
    ...overrides
  })
}

// Bitcoin test scenarios
export const createBitcoinTestScenarios = () => ({
  // Successful scenarios
  successful_portfolio_fetch: () => createMockBitcoinPortfolio({ 
    total_btc: 1.5, 
    total_value_usd: 67500 
  }),
  
  successful_send_transaction: () => createMockBitcoinSendResult({ 
    success: true,
    transaction_id: 'test_tx_success_123'
  }),
  
  successful_address_generation: () => ({
    P2PKH: createMockBitcoinAddress({ address_type: 'P2PKH' }),
    P2WPKH: createMockBitcoinAddress({ address_type: 'P2WPKH' }),
    P2TR: createMockBitcoinAddress({ address_type: 'P2TR' })
  }),
  
  // Error scenarios
  empty_portfolio: () => createMockBitcoinPortfolio({ 
    addresses: [],
    total_btc: 0,
    total_satoshis: 0,
    total_value_usd: 0,
    utxos: []
  }),
  
  insufficient_funds_error: () => createMockBitcoinSendResult({ 
    success: false,
    error_message: 'Insufficient balance: need 101000 satoshis, have 50000 satoshis'
  }),
  
  invalid_address_error: () => ({
    error: 'Invalid Bitcoin address format: invalid_address_123'
  }),
  
  network_error: () => ({
    error: 'Network timeout: Unable to connect to Bitcoin network'
  }),
  
  // Large data scenarios for performance testing
  large_portfolio: () => createMockBitcoinPortfolio({
    addresses: Array.from({ length: 100 }, (_, i) => 
      createMockBitcoinAddress({ 
        address_type: ['P2PKH', 'P2WPKH', 'P2TR'][i % 3] as any,
        balance_satoshis: Math.floor(Math.random() * 10000000)
      })
    )
  }),
  
  many_utxos: () => Array.from({ length: 1000 }, () => createMockBitcoinUTXO())
})