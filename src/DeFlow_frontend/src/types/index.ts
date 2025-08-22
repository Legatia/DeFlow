// Workflow states for better organization
export type WorkflowState = 'draft' | 'published' | 'template'

export interface WorkflowMetadata {
  templateCategory?: string
  templateDescription?: string
  usageCount?: number
  isPublic?: boolean
  originalWorkflowId?: string // For templates created from existing workflows
}

// Simple types without BigInt dependencies
export interface Workflow {
  id: string
  name: string
  description?: string
  nodes: WorkflowNode[]
  connections: NodeConnection[]
  triggers: WorkflowTrigger[]
  created_at: string // ICP timestamp as string (nanoseconds)
  updated_at: string
  active: boolean
  state: WorkflowState
  owner?: string
  tags?: string[]
  version?: string
  metadata?: WorkflowMetadata
}

export interface WorkflowNode {
  id: string
  node_type: string
  position: { x: number; y: number }
  configuration: Record<string, unknown>
  metadata: {
    label: string
    description?: string
    tags: string[]
    icon?: string
    color?: string
  }
}

export interface NodeConnection {
  id: string
  source_node_id: string
  target_node_id: string
  source_output: string
  target_input: string
}

// Alias for consistency with service layer
export type WorkflowConnection = NodeConnection

export type WorkflowTrigger = 
  | { type: 'manual' }
  | { type: 'schedule'; cron: string }
  | { type: 'webhook'; path: string }
  | { type: 'event'; event_type: string; conditions: Record<string, any> }

export interface WorkflowExecution {
  id: string
  workflow_id: string
  status: ExecutionStatus
  started_at: string
  completed_at?: string | null
  trigger_data?: Record<string, any>
  node_executions: NodeExecution[]
  error_message?: string | null
  duration?: string | null
}

export interface NodeExecution {
  id: string
  execution_id: string
  node_id: string
  status: ExecutionStatus
  started_at: string
  completed_at?: string | null
  input_data?: Record<string, any> | null
  output_data?: Record<string, any> | null
  error_message?: string | null
  duration?: string | null
}

export type ExecutionStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'

export interface NodeDefinition {
  node_type: string
  name: string
  description: string
  category: string
  version: string
  input_schema: ParameterSchema[]
  output_schema: ParameterSchema[]
  configuration_schema: ParameterSchema[]
}

export interface ParameterSchema {
  name: string
  parameter_type: string
  required: boolean
  description?: string
  default_value?: any
}

// Store types
export interface WorkflowStore {
  workflows: Workflow[]
  currentWorkflow?: Workflow
  executions: WorkflowExecution[]
  nodeDefinitions: NodeDefinition[]
  isLoading: boolean
  error?: string
  
  // Actions
  loadWorkflows: () => Promise<void>
  loadWorkflow: (id: string) => Promise<void>
  createWorkflow: (workflow: Omit<Workflow, 'id' | 'created_at' | 'updated_at'>) => Promise<string>
  updateWorkflow: (workflow: Workflow) => Promise<void>
  deleteWorkflow: (id: string) => Promise<void>
  executeWorkflow: (id: string, triggerData?: Record<string, any>) => Promise<string>
  loadExecutions: (workflowId?: string) => Promise<void>
  loadNodeDefinitions: () => Promise<void>
}