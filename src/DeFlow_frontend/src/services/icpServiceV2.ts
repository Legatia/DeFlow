// Proper ICP Service with bignumber.js BigInt handling
import { Actor, HttpAgent } from '@dfinity/agent'
import { AuthClient } from '@dfinity/auth-client'
import { Principal } from '@dfinity/principal'
import { Workflow, WorkflowExecution, NodeDefinition } from '../types'
import { BigIntUtils } from '../utils/bigint-utils'

// Import the generated declarations
let canisterModule: any = null

// Lazy load the canister declarations to avoid BigInt issues during initial load
async function getCanisterModule() {
  if (!canisterModule) {
    try {
      canisterModule = await import('../../../declarations/DeFlow_backend')
    } catch (error) {
      console.warn('Failed to load canister declarations, using fallback:', error)
      // Return a mock module for development
      canisterModule = {
        canisterId: 'rdmx6-jaaaa-aaaaa-aaadq-cai',
        createActor: () => null
      }
    }
  }
  return canisterModule
}

// Backend canister interface
interface BackendCanister {
  greet: (name: string) => Promise<string>
  create_workflow: (workflow: any) => Promise<string>
  update_workflow: (workflow: any) => Promise<void>
  get_workflow: (id: string) => Promise<any>
  list_workflows: () => Promise<any[]>
  delete_workflow: (id: string) => Promise<void>
  start_execution: (workflowId: string, triggerData?: any) => Promise<string>
  get_execution: (id: string) => Promise<any>
  list_executions: (workflowId?: string) => Promise<any[]>
  list_node_types: () => Promise<string[]>
  get_node_definition: (nodeType: string) => Promise<any>
}

class ICPServiceV2 {
  private actor: BackendCanister | null = null
  private authClient: AuthClient | null = null
  private isInitialized = false
  private isLocal = true

  async initialize(): Promise<void> {
    if (this.isInitialized) return

    try {

      // Initialize auth client
      this.authClient = await AuthClient.create()

      // Create agent
      const agent = new HttpAgent({
        host: this.isLocal ? 'http://localhost:4943' : 'https://ic0.app',
      })

      // Fetch root key for local development
      if (this.isLocal) {
        try {
          await agent.fetchRootKey()
        } catch (error) {
          console.warn('Could not fetch root key:', error)
        }
      }

      // Get canister module with lazy loading
      const { canisterId, createActor } = await getCanisterModule()

      if (createActor && canisterId) {
        try {
          this.actor = createActor(canisterId, { agent })
        } catch (error) {
          console.warn('Failed to create real actor, using mock:', error)
          this.actor = this.createMockActor()
        }
      } else {
        this.actor = this.createMockActor()
      }

      this.isInitialized = true

    } catch (error) {
      console.error('Failed to initialize ICP service:', error)
      this.actor = this.createMockActor()
      this.isInitialized = true
    }
  }

  private createMockActor(): BackendCanister {
    return {
      greet: async (name: string) => `Hello, ${name}! (ICP Mock Mode with BigInt support)`,
      
      create_workflow: async (workflow: any) => {
        return `workflow_${Date.now()}`
      },
      
      update_workflow: async (workflow: any) => {
      },
      
      get_workflow: async (id: string) => {
        return {
          id,
          name: 'Mock Workflow with BigInt',
          description: 'This workflow demonstrates proper BigInt handling',
          nodes: [],
          connections: [],
          triggers: [],
          created_at: BigIntUtils.dateToTimestamp(new Date()),
          updated_at: BigIntUtils.dateToTimestamp(new Date()),
          active: true
        }
      },
      
      list_workflows: async () => {
        return [
          {
            id: 'mock_workflow_1',
            name: 'Sample BigInt Workflow',
            description: 'Demonstrates proper BigInt timestamp handling',
            nodes: [],
            connections: [],
            triggers: [{ type: 'manual' }],
            created_at: BigIntUtils.dateToTimestamp(new Date(Date.now() - 86400000)), // 1 day ago
            updated_at: BigIntUtils.dateToTimestamp(new Date()),
            active: true
          }
        ]
      },
      
      delete_workflow: async (id: string) => {
      },
      
      start_execution: async (workflowId: string, triggerData?: any) => {
        return `execution_${Date.now()}`
      },
      
      get_execution: async (id: string) => {
        return {
          id,
          workflow_id: 'mock_workflow_1',
          status: 'completed',
          started_at: BigIntUtils.dateToTimestamp(new Date(Date.now() - 60000)), // 1 minute ago
          completed_at: BigIntUtils.dateToTimestamp(new Date()),
          node_executions: []
        }
      },
      
      list_executions: async (workflowId?: string) => {
        return [
          {
            id: 'mock_execution_1',
            workflow_id: workflowId || 'mock_workflow_1',
            status: 'completed',
            started_at: BigIntUtils.dateToTimestamp(new Date(Date.now() - 120000)), // 2 minutes ago
            completed_at: BigIntUtils.dateToTimestamp(new Date(Date.now() - 60000)), // 1 minute ago
            node_executions: []
          }
        ]
      },
      
      list_node_types: async () => {
        return ['trigger', 'action', 'condition', 'transform']
      },
      
      get_node_definition: async (nodeType: string) => {
        return {
          node_type: nodeType,
          name: `${nodeType.charAt(0).toUpperCase() + nodeType.slice(1)} Node`,
          description: `Mock ${nodeType} node with BigInt support`,
          category: 'mock',
          version: '1.0.0',
          input_schema: [],
          output_schema: [],
          configuration_schema: []
        }
      }
    }
  }

  private async ensureInitialized(): Promise<void> {
    if (!this.isInitialized) {
      await this.initialize()
    }
    if (!this.actor) {
      throw new Error('ICP service not properly initialized')
    }
  }

  // Convert backend data to frontend types with proper BigInt handling
  private convertWorkflow(backendWorkflow: any): Workflow {
    return {
      ...backendWorkflow,
      created_at: typeof backendWorkflow.created_at === 'bigint' 
        ? backendWorkflow.created_at 
        : BigIntUtils.toBigInt(backendWorkflow.created_at || Date.now() * 1_000_000),
      updated_at: typeof backendWorkflow.updated_at === 'bigint'
        ? backendWorkflow.updated_at
        : BigIntUtils.toBigInt(backendWorkflow.updated_at || Date.now() * 1_000_000)
    }
  }

  private convertExecution(backendExecution: any): WorkflowExecution {
    return {
      ...backendExecution,
      started_at: typeof backendExecution.started_at === 'bigint'
        ? backendExecution.started_at
        : BigIntUtils.toBigInt(backendExecution.started_at || Date.now() * 1_000_000),
      completed_at: backendExecution.completed_at 
        ? (typeof backendExecution.completed_at === 'bigint'
            ? backendExecution.completed_at
            : BigIntUtils.toBigInt(backendExecution.completed_at))
        : undefined,
      node_executions: backendExecution.node_executions?.map((ne: any) => ({
        ...ne,
        started_at: ne.started_at 
          ? (typeof ne.started_at === 'bigint' ? ne.started_at : BigIntUtils.toBigInt(ne.started_at))
          : undefined,
        completed_at: ne.completed_at 
          ? (typeof ne.completed_at === 'bigint' ? ne.completed_at : BigIntUtils.toBigInt(ne.completed_at))
          : undefined
      })) || []
    }
  }

  // Public API methods
  async greet(name: string): Promise<string> {
    await this.ensureInitialized()
    return this.actor!.greet(name)
  }

  async createWorkflow(workflow: Omit<Workflow, 'id' | 'created_at' | 'updated_at'>): Promise<string> {
    await this.ensureInitialized()
    
    const workflowData = {
      ...workflow,
      created_at: BigIntUtils.dateToTimestamp(),
      updated_at: BigIntUtils.dateToTimestamp()
    }
    
    return this.actor!.create_workflow(workflowData)
  }

  async updateWorkflow(workflow: Workflow): Promise<void> {
    await this.ensureInitialized()
    
    const workflowData = {
      ...workflow,
      updated_at: BigIntUtils.dateToTimestamp()
    }
    
    return this.actor!.update_workflow(workflowData)
  }

  async getWorkflow(id: string): Promise<Workflow> {
    await this.ensureInitialized()
    const backendWorkflow = await this.actor!.get_workflow(id)
    return this.convertWorkflow(backendWorkflow)
  }

  async listWorkflows(): Promise<Workflow[]> {
    await this.ensureInitialized()
    const backendWorkflows = await this.actor!.list_workflows()
    return backendWorkflows.map(w => this.convertWorkflow(w))
  }

  async deleteWorkflow(id: string): Promise<void> {
    await this.ensureInitialized()
    return this.actor!.delete_workflow(id)
  }

  async startExecution(workflowId: string, triggerData?: Record<string, any>): Promise<string> {
    await this.ensureInitialized()
    return this.actor!.start_execution(workflowId, triggerData)
  }

  async getExecution(id: string): Promise<WorkflowExecution> {
    await this.ensureInitialized()
    const backendExecution = await this.actor!.get_execution(id)
    return this.convertExecution(backendExecution)
  }

  async listExecutions(workflowId?: string): Promise<WorkflowExecution[]> {
    await this.ensureInitialized()
    const backendExecutions = await this.actor!.list_executions(workflowId)
    return backendExecutions.map(e => this.convertExecution(e))
  }

  async listNodeDefinitions(): Promise<NodeDefinition[]> {
    await this.ensureInitialized()
    const nodeTypes = await this.actor!.list_node_types()
    const definitions: NodeDefinition[] = []
    
    for (const nodeType of nodeTypes) {
      try {
        const definition = await this.actor!.get_node_definition(nodeType)
        definitions.push(definition)
      } catch (error) {
        console.warn(`Failed to load definition for ${nodeType}:`, error)
      }
    }
    
    return definitions
  }

  async getNodeDefinition(nodeType: string): Promise<NodeDefinition> {
    await this.ensureInitialized()
    return this.actor!.get_node_definition(nodeType)
  }

  // Authentication methods
  async login(): Promise<boolean> {
    if (!this.authClient) {
      await this.initialize()
    }

    return new Promise((resolve) => {
      this.authClient!.login({
        identityProvider: this.isLocal 
          ? 'http://localhost:4943/?canisterId=rdmx6-jaaaa-aaaaa-aaadq-cai'
          : 'https://identity.ic0.app',
        onSuccess: () => {
          resolve(true)
        },
        onError: (error) => {
          console.error('ICP login failed:', error)
          resolve(false)
        }
      })
    })
  }

  async logout(): Promise<void> {
    if (this.authClient) {
      await this.authClient.logout()
    }
  }

  async isAuthenticated(): Promise<boolean> {
    if (!this.authClient) {
      await this.initialize()
    }
    return this.authClient!.isAuthenticated()
  }

  async getIdentity(): Promise<any> {
    if (!this.authClient) {
      await this.initialize()
    }
    return this.authClient!.getIdentity()
  }
}

export const icpService = new ICPServiceV2()