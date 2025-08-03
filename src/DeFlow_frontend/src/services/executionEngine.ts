// Advanced workflow execution engine for DeFlow
import { Workflow, WorkflowExecution, NodeExecution, WorkflowNode, WorkflowConnection } from '../types'
import { TimestampUtils } from '../utils/timestamp-utils'
import { NodeType, NODE_TYPES } from '../types/nodes'

export interface ExecutionContext {
  workflowId: string
  executionId: string
  variables: Record<string, any>
  currentData: any
  userId: string
  metadata: Record<string, any>
}

export interface ExecutionResult {
  success: boolean
  data: any
  error?: string
  duration: number
  logs: ExecutionLog[]
}

export interface ExecutionLog {
  timestamp: string
  level: 'info' | 'warn' | 'error' | 'debug'
  message: string
  nodeId?: string
  data?: any
}

export interface NodeExecutor {
  nodeType: string
  execute(node: WorkflowNode, context: ExecutionContext): Promise<ExecutionResult>
}

class WorkflowExecutionEngine {
  private nodeExecutors: Map<string, NodeExecutor> = new Map()
  private executionQueue: Map<string, WorkflowExecution> = new Map()
  private logs: Map<string, ExecutionLog[]> = new Map()

  constructor() {
    this.registerNodeExecutors()
  }

  private registerNodeExecutors() {
    // Register all node type executors
    this.nodeExecutors.set('manual-trigger', new ManualTriggerExecutor())
    this.nodeExecutors.set('webhook-trigger', new WebhookTriggerExecutor())
    this.nodeExecutors.set('schedule-trigger', new ScheduleTriggerExecutor())
    this.nodeExecutors.set('send-email', new EmailExecutor())
    this.nodeExecutors.set('http-request', new HttpRequestExecutor())
    this.nodeExecutors.set('transform-data', new DataTransformExecutor())
    this.nodeExecutors.set('condition', new ConditionExecutor())
    this.nodeExecutors.set('delay', new DelayExecutor())
  }

  async executeWorkflow(
    workflow: Workflow, 
    trigger?: any, 
    userId: string = 'anonymous'
  ): Promise<WorkflowExecution> {
    const executionId = this.generateExecutionId()
    const execution: WorkflowExecution = {
      id: executionId,
      workflow_id: workflow.id,
      status: 'running',
      started_at: TimestampUtils.dateToICPTimestamp(new Date()),
      completed_at: null,
      trigger_data: trigger || {},
      node_executions: [],
      error_message: null,
      duration: null
    }

    this.executionQueue.set(executionId, execution)
    this.logs.set(executionId, [])

    try {
      this.addLog(executionId, 'info', `Starting workflow execution: ${workflow.name}`)
      
      const context: ExecutionContext = {
        workflowId: workflow.id,
        executionId,
        variables: {},
        currentData: trigger || {},
        userId,
        metadata: {}
      }

      // Find trigger nodes
      const triggerNodes = this.findTriggerNodes(workflow)
      if (triggerNodes.length === 0) {
        throw new Error('No trigger nodes found in workflow')
      }

      // Execute workflow starting from trigger nodes
      const results = await Promise.all(
        triggerNodes.map(node => this.executeNode(node, workflow, context))
      )

      // Check if all trigger executions were successful
      const allSuccessful = results.every(result => result.success)
      
      execution.status = allSuccessful ? 'completed' : 'failed'
      execution.completed_at = TimestampUtils.dateToICPTimestamp(new Date())
      execution.duration = this.calculateDuration(execution.started_at, execution.completed_at)

      if (!allSuccessful) {
        const errors = results.filter(r => !r.success).map(r => r.error).join('; ')
        execution.error_message = errors
        this.addLog(executionId, 'error', `Workflow execution failed: ${errors}`)
      } else {
        this.addLog(executionId, 'info', 'Workflow execution completed successfully')
      }

    } catch (error) {
      execution.status = 'failed'
      execution.completed_at = TimestampUtils.dateToICPTimestamp(new Date())
      execution.duration = this.calculateDuration(execution.started_at, execution.completed_at!)
      execution.error_message = error instanceof Error ? error.message : 'Unknown error'
      this.addLog(executionId, 'error', `Workflow execution error: ${execution.error_message}`)
    }

    this.executionQueue.set(executionId, execution)
    return execution
  }

  private async executeNode(
    node: WorkflowNode,
    workflow: Workflow,
    context: ExecutionContext
  ): Promise<ExecutionResult> {
    const startTime = Date.now()
    const nodeExecution: NodeExecution = {
      id: this.generateNodeExecutionId(),
      execution_id: context.executionId,
      node_id: node.id,
      status: 'running',
      started_at: TimestampUtils.dateToICPTimestamp(new Date()),
      completed_at: null,
      input_data: context.currentData,
      output_data: null,
      error_message: null,
      duration: null
    }

    const execution = this.executionQueue.get(context.executionId)!
    execution.node_executions.push(nodeExecution)

    try {
      this.addLog(context.executionId, 'info', `Executing node: ${node.id}`, node.id)

      const executor = this.nodeExecutors.get(node.node_type)
      if (!executor) {
        throw new Error(`No executor found for node type: ${node.node_type}`)
      }

      const result = await executor.execute(node, context)
      
      nodeExecution.status = result.success ? 'completed' : 'failed'
      nodeExecution.completed_at = TimestampUtils.dateToICPTimestamp(new Date())
      nodeExecution.output_data = result.data
      nodeExecution.duration = String(result.duration * 1000000) // Convert ms to nanoseconds as string
      nodeExecution.error_message = result.error || null

      if (result.success) {
        this.addLog(context.executionId, 'info', `Node completed successfully: ${node.id}`, node.id)
        
        // Execute connected nodes
        const connectedNodes = this.findConnectedNodes(node, workflow)
        if (connectedNodes.length > 0) {
          // Update context with output data
          const newContext = {
            ...context,
            currentData: result.data
          }

          // Execute connected nodes
          await Promise.all(
            connectedNodes.map(connectedNode => 
              this.executeNode(connectedNode, workflow, newContext)
            )
          )
        }
      } else {
        this.addLog(context.executionId, 'error', `Node failed: ${node.id} - ${result.error}`, node.id)
      }

      return result

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error'
      nodeExecution.status = 'failed'
      nodeExecution.completed_at = TimestampUtils.dateToICPTimestamp(new Date())
      nodeExecution.error_message = errorMessage
      nodeExecution.duration = String((Date.now() - startTime) * 1000000)

      this.addLog(context.executionId, 'error', `Node execution error: ${node.id} - ${errorMessage}`, node.id)

      return {
        success: false,
        data: null,
        error: errorMessage,
        duration: Date.now() - startTime,
        logs: []
      }
    }
  }

  private findTriggerNodes(workflow: Workflow): WorkflowNode[] {
    return workflow.nodes.filter(node => {
      const nodeType = NODE_TYPES.find(nt => nt.id === node.node_type)
      return nodeType?.category === 'triggers'
    })
  }

  private findConnectedNodes(sourceNode: WorkflowNode, workflow: Workflow): WorkflowNode[] {
    const connectedNodeIds = workflow.connections
      .filter(conn => conn.source_node_id === sourceNode.id)
      .map(conn => conn.target_node_id)

    return workflow.nodes.filter(node => connectedNodeIds.includes(node.id))
  }

  private addLog(executionId: string, level: ExecutionLog['level'], message: string, nodeId?: string, data?: any) {
    const log: ExecutionLog = {
      timestamp: TimestampUtils.dateToICPTimestamp(new Date()),
      level,
      message,
      nodeId,
      data
    }

    const logs = this.logs.get(executionId) || []
    logs.push(log)
    this.logs.set(executionId, logs)
  }

  private calculateDuration(startTime: string, endTime: string): string {
    const start = TimestampUtils.icpTimestampToDate(startTime)
    const end = TimestampUtils.icpTimestampToDate(endTime)
    return String((end.getTime() - start.getTime()) * 1000000) // Convert ms to nanoseconds as string
  }

  private generateExecutionId(): string {
    return `exec_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
  }

  private generateNodeExecutionId(): string {
    return `node_exec_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
  }

  // Public methods for monitoring
  getExecution(executionId: string): WorkflowExecution | null {
    return this.executionQueue.get(executionId) || null
  }

  getExecutionLogs(executionId: string): ExecutionLog[] {
    return this.logs.get(executionId) || []
  }

  getAllExecutions(): WorkflowExecution[] {
    return Array.from(this.executionQueue.values())
  }

  clearExecutionHistory(): void {
    this.executionQueue.clear()
    this.logs.clear()
  }
}

// Node Executors
class ManualTriggerExecutor implements NodeExecutor {
  nodeType = 'manual-trigger'
  
  async execute(node: WorkflowNode, context: ExecutionContext): Promise<ExecutionResult> {
    return {
      success: true,
      data: { trigger: 'manual', timestamp: new Date().toISOString(), ...context.currentData },
      duration: 10,
      logs: []
    }
  }
}

class WebhookTriggerExecutor implements NodeExecutor {
  nodeType = 'webhook-trigger'
  
  async execute(node: WorkflowNode, context: ExecutionContext): Promise<ExecutionResult> {
    const config = node.configuration.parameters
    return {
      success: true,
      data: { 
        trigger: 'webhook', 
        path: config.path,
        method: config.method,
        timestamp: new Date().toISOString(),
        ...context.currentData 
      },
      duration: 15,
      logs: []
    }
  }
}

class ScheduleTriggerExecutor implements NodeExecutor {
  nodeType = 'schedule-trigger'
  
  async execute(node: WorkflowNode, context: ExecutionContext): Promise<ExecutionResult> {
    const config = node.configuration.parameters
    return {
      success: true,
      data: { 
        trigger: 'schedule', 
        cron: config.cron,
        timezone: config.timezone,
        timestamp: new Date().toISOString(),
        ...context.currentData 
      },
      duration: 12,
      logs: []
    }
  }
}

class EmailExecutor implements NodeExecutor {
  nodeType = 'send-email'
  
  async execute(node: WorkflowNode, context: ExecutionContext): Promise<ExecutionResult> {
    const config = node.configuration.parameters
    
    try {
      // Simulate email sending with template processing
      const processedBody = this.processTemplate(config.body, context.currentData)
      const processedSubject = this.processTemplate(config.subject, context.currentData)
      
      // Simulate network delay
      await new Promise(resolve => setTimeout(resolve, 500 + Math.random() * 1000))
      
      return {
        success: true,
        data: {
          emailSent: true,
          to: config.to,
          subject: processedSubject,
          body: processedBody,
          sentAt: new Date().toISOString()
        },
        duration: 500 + Math.random() * 1000,
        logs: []
      }
    } catch (error) {
      return {
        success: false,
        data: null,
        error: `Failed to send email: ${error instanceof Error ? error.message : 'Unknown error'}`,
        duration: 100,
        logs: []
      }
    }
  }

  private processTemplate(template: string, data: any): string {
    if (!template) return ''
    
    return template.replace(/\{\{([^}]+)\}\}/g, (match, path) => {
      const value = this.getValueByPath(data, path.trim())
      return value !== undefined ? String(value) : match
    })
  }

  private getValueByPath(obj: any, path: string): any {
    return path.split('.').reduce((current, key) => current?.[key], obj)
  }
}

class HttpRequestExecutor implements NodeExecutor {
  nodeType = 'http-request'
  
  async execute(node: WorkflowNode, context: ExecutionContext): Promise<ExecutionResult> {
    const config = node.configuration.parameters
    
    try {
      // Simulate HTTP request
      await new Promise(resolve => setTimeout(resolve, 200 + Math.random() * 800))
      
      // Mock response based on URL
      const mockResponse = this.generateMockResponse(config.url, config.method)
      
      return {
        success: true,
        data: {
          response: mockResponse,
          status: 200,
          headers: { 'content-type': 'application/json' },
          requestedAt: new Date().toISOString()
        },
        duration: 200 + Math.random() * 800,
        logs: []
      }
    } catch (error) {
      return {
        success: false,
        data: null,
        error: `HTTP request failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        duration: 100,
        logs: []
      }
    }
  }

  private generateMockResponse(url: string, method: string): any {
    if (url.includes('api.example.com/stats')) {
      return {
        users: 1250,
        activeUsers: 847,
        revenue: 15420.50,
        growth: 12.5
      }
    }
    
    if (url.includes('api.source.com/data')) {
      return {
        data: [
          { id: 1, name: 'John Doe', email: 'john@example.com' },
          { id: 2, name: 'Jane Smith', email: 'jane@example.com' }
        ]
      }
    }
    
    return { success: true, timestamp: new Date().toISOString() }
  }
}

class DataTransformExecutor implements NodeExecutor {
  nodeType = 'transform-data'
  
  async execute(node: WorkflowNode, context: ExecutionContext): Promise<ExecutionResult> {
    const config = node.configuration.parameters
    
    try {
      let transformedData = context.currentData
      
      if (config.operation === 'map' && config.config) {
        const mapping = JSON.parse(config.config).mapping
        transformedData = this.mapData(context.currentData, mapping)
      }
      
      return {
        success: true,
        data: {
          originalData: context.currentData,
          transformedData,
          operation: config.operation
        },
        duration: 50 + Math.random() * 100,
        logs: []
      }
    } catch (error) {
      return {
        success: false,
        data: null,
        error: `Data transformation failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        duration: 25,
        logs: []
      }
    }
  }

  private mapData(data: any, mapping: Record<string, string>): any {
    if (Array.isArray(data)) {
      return data.map(item => this.mapObject(item, mapping))
    } else if (typeof data === 'object' && data !== null) {
      return this.mapObject(data, mapping)
    }
    return data
  }

  private mapObject(obj: any, mapping: Record<string, string>): any {
    const result: any = {}
    for (const [newKey, oldKey] of Object.entries(mapping)) {
      if (obj.hasOwnProperty(oldKey)) {
        result[newKey] = obj[oldKey]
      }
    }
    return result
  }
}

class ConditionExecutor implements NodeExecutor {
  nodeType = 'condition'
  
  async execute(node: WorkflowNode, context: ExecutionContext): Promise<ExecutionResult> {
    const config = node.configuration.parameters
    
    try {
      const fieldValue = this.getValueByPath(context.currentData, config.field)
      const conditionMet = this.evaluateCondition(fieldValue, config.operator, config.value)
      
      return {
        success: true,
        data: {
          conditionMet,
          field: config.field,
          fieldValue,
          operator: config.operator,
          expectedValue: config.value,
          originalData: context.currentData
        },
        duration: 25,
        logs: []
      }
    } catch (error) {
      return {
        success: false,
        data: null,
        error: `Condition evaluation failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        duration: 10,
        logs: []
      }
    }
  }

  private getValueByPath(obj: any, path: string): any {
    return path.split('.').reduce((current, key) => current?.[key], obj)
  }

  private evaluateCondition(value: any, operator: string, expected: any): boolean {
    switch (operator) {
      case 'equals':
        return value === expected
      case 'not_equals':
        return value !== expected
      case 'greater_than':
        return Number(value) > Number(expected)
      case 'less_than':
        return Number(value) < Number(expected)
      case 'contains':
        return String(value).includes(String(expected))
      case 'starts_with':
        return String(value).startsWith(String(expected))
      case 'ends_with':
        return String(value).endsWith(String(expected))
      default:
        return false
    }
  }
}

class DelayExecutor implements NodeExecutor {
  nodeType = 'delay'
  
  async execute(node: WorkflowNode, context: ExecutionContext): Promise<ExecutionResult> {
    const config = node.configuration.parameters
    const delayMs = this.calculateDelayMs(config.duration, config.unit)
    
    try {
      await new Promise(resolve => setTimeout(resolve, delayMs))
      
      return {
        success: true,
        data: {
          delayed: true,
          duration: config.duration,
          unit: config.unit,
          delayMs,
          continuedAt: new Date().toISOString(),
          originalData: context.currentData
        },
        duration: delayMs,
        logs: []
      }
    } catch (error) {
      return {
        success: false,
        data: null,
        error: `Delay failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        duration: 0,
        logs: []
      }
    }
  }

  private calculateDelayMs(duration: number, unit: string): number {
    const multipliers = {
      milliseconds: 1,
      seconds: 1000,
      minutes: 60000,
      hours: 3600000
    }
    return duration * (multipliers[unit as keyof typeof multipliers] || 1000)
  }
}

// Export singleton instance
export const executionEngine = new WorkflowExecutionEngine()
export default executionEngine