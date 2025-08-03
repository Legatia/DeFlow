// Test utilities and helpers for DeFlow
import React, { ReactElement } from 'react'
import { render, RenderOptions } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { vi } from 'vitest'
import { Workflow, WorkflowExecution, User } from '../../types'
import { TimestampUtils } from '../../utils/timestamp-utils'

// Create a custom render function that includes providers
const AllTheProviders = ({ children }: { children: React.ReactNode }) => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  })

  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        {children}
      </BrowserRouter>
    </QueryClientProvider>
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
      duration: BigInt(30000000000) // 30 seconds in nanoseconds
    }
  ],
  error_message: null,
  duration: BigInt(60000000000), // 60 seconds in nanoseconds
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