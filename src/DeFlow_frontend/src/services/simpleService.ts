// Simple service without any @dfinity dependencies
// This will be the main service, and we'll add ICP functionality later via lazy loading

import { Workflow, WorkflowExecution, NodeDefinition } from '../types'
import { TimestampUtils } from '../utils/timestamp-utils'

// Generate simple IDs
const generateId = () => `id_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`

// In-memory storage (will be replaced with ICP calls later)
let mockWorkflows: Workflow[] = []

let mockExecutions: WorkflowExecution[] = []

let mockNodeDefinitions: NodeDefinition[] = [
  {
    node_type: 'trigger',
    name: 'Manual Trigger',
    description: 'Manually start a workflow',
    category: 'triggers',
    version: '1.0.0',
    input_schema: [],
    output_schema: [
      {
        name: 'triggered',
        parameter_type: 'boolean',
        required: true,
        description: 'Indicates the workflow was triggered'
      }
    ],
    configuration_schema: []
  },
  {
    node_type: 'action',
    name: 'Action Node',
    description: 'Perform an action',
    category: 'actions',
    version: '1.0.0',
    input_schema: [
      {
        name: 'input',
        parameter_type: 'object',
        required: true,
        description: 'Input data'
      }
    ],
    output_schema: [
      {
        name: 'result',
        parameter_type: 'object',
        required: true,
        description: 'Action result'
      }
    ],
    configuration_schema: [
      {
        name: 'message',
        parameter_type: 'string',
        required: false,
        description: 'Message to display'
      }
    ]
  }
]

// Service implementation without any external dependencies
export class SimpleService {
  private delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms))

  async listWorkflows(): Promise<Workflow[]> {
    await this.delay(200) // Simulate network delay
    return [...mockWorkflows]
  }

  async getWorkflow(id: string): Promise<Workflow> {
    await this.delay(150)
    const workflow = mockWorkflows.find(w => w.id === id)
    if (!workflow) {
      throw new Error(`Workflow ${id} not found`)
    }
    return { ...workflow }
  }

  async createWorkflow(workflow: Omit<Workflow, 'id' | 'created_at' | 'updated_at'>): Promise<string> {
    await this.delay(300)
    const id = generateId()
    const now = TimestampUtils.dateToICPTimestamp()
    const newWorkflow: Workflow = {
      ...workflow,
      id,
      created_at: now,
      updated_at: now
    }
    mockWorkflows.push(newWorkflow)
    return id
  }

  async updateWorkflow(workflow: Workflow): Promise<void> {
    await this.delay(250)
    const index = mockWorkflows.findIndex(w => w.id === workflow.id)
    if (index === -1) {
      throw new Error(`Workflow ${workflow.id} not found`)
    }
    mockWorkflows[index] = {
      ...workflow,
      updated_at: TimestampUtils.dateToICPTimestamp()
    }
  }

  async deleteWorkflow(id: string): Promise<void> {
    await this.delay(200)
    const index = mockWorkflows.findIndex(w => w.id === id)
    if (index === -1) {
      throw new Error(`Workflow ${id} not found`)
    }
    mockWorkflows.splice(index, 1)
  }

  async startExecution(workflowId: string, triggerData?: Record<string, any>): Promise<string> {
    await this.delay(300)
    const executionId = generateId()
    const execution: WorkflowExecution = {
      id: executionId,
      workflow_id: workflowId,
      status: 'running',
      started_at: TimestampUtils.dateToICPTimestamp(),
      trigger_data: triggerData,
      node_executions: []
    }
    mockExecutions.push(execution)
    
    // Simulate completion
    setTimeout(() => {
      const exec = mockExecutions.find(e => e.id === executionId)
      if (exec) {
        exec.status = 'completed'
        exec.completed_at = TimestampUtils.dateToICPTimestamp()
      }
    }, 2000)
    
    return executionId
  }

  async getExecution(id: string): Promise<WorkflowExecution> {
    await this.delay(150)
    const execution = mockExecutions.find(e => e.id === id)
    if (!execution) {
      throw new Error(`Execution ${id} not found`)
    }
    return { ...execution }
  }

  async listExecutions(workflowId?: string): Promise<WorkflowExecution[]> {
    await this.delay(200)
    if (workflowId) {
      return mockExecutions.filter(e => e.workflow_id === workflowId)
    }
    return [...mockExecutions]
  }

  async listNodeDefinitions(): Promise<NodeDefinition[]> {
    await this.delay(150)
    return [...mockNodeDefinitions]
  }

  async getNodeDefinition(nodeType: string): Promise<NodeDefinition> {
    await this.delay(100)
    const definition = mockNodeDefinitions.find(d => d.node_type === nodeType)
    if (!definition) {
      throw new Error(`Node definition ${nodeType} not found`)
    }
    return { ...definition }
  }

  async greet(name: string): Promise<string> {
    await this.delay(100)
    return `Hello, ${name}! Welcome to DeFlow (Simple Mode).`
  }

  // Authentication placeholder methods
  async login(): Promise<boolean> {
    await this.delay(500)
    console.log('Simple mode: Authentication not required')
    return true
  }

  async logout(): Promise<void> {
    await this.delay(200)
    console.log('Simple mode: Logout complete')
  }

  async isAuthenticated(): Promise<boolean> {
    return true // Always authenticated in simple mode
  }

  async getIdentity(): Promise<any> {
    return {
      getPrincipal: () => ({
        toString: () => 'simple-mode-principal'
      })
    }
  }
}

export const simpleService = new SimpleService()