// Real-time workflow execution and monitoring service
import { WorkflowExecution, NodeExecution } from '../types'
import { ExecutionLog } from './executionEngine'
import { TimestampUtils } from '../utils/timestamp-utils'

export interface RealTimeUpdate {
  type: 'execution_start' | 'execution_complete' | 'execution_failed' | 'node_start' | 'node_complete' | 'node_failed' | 'log'
  executionId: string
  workflowId: string
  nodeId?: string
  timestamp: string
  data: any
}

export interface WebSocketConnection {
  id: string
  userId: string
  workflowIds: string[]
  isActive: boolean
  lastHeartbeat: string
}

export interface SubscriptionFilter {
  workflowIds?: string[]
  executionIds?: string[]
  nodeTypes?: string[]
  logLevels?: string[]
  userId?: string
}

class RealTimeService {
  private connections: Map<string, WebSocketConnection> = new Map()
  private subscribers: Map<string, SubscriptionFilter> = new Map()
  private messageQueue: Map<string, RealTimeUpdate[]> = new Map()
  private heartbeatInterval: NodeJS.Timeout | null = null
  private isEnabled: boolean = true

  constructor() {
    this.startHeartbeat()
  }

  // Connection management
  addConnection(connectionId: string, userId: string, workflowIds: string[] = []): void {
    const connection: WebSocketConnection = {
      id: connectionId,
      userId,
      workflowIds,
      isActive: true,
      lastHeartbeat: TimestampUtils.dateToICPTimestamp(new Date())
    }
    
    this.connections.set(connectionId, connection)
    this.messageQueue.set(connectionId, [])
    
    // Send connection confirmation
    this.sendToConnection(connectionId, {
      type: 'execution_start',
      executionId: 'system',
      workflowId: 'system',
      timestamp: TimestampUtils.dateToICPTimestamp(new Date()),
      data: { 
        message: 'Connected to DeFlow real-time service',
        connectionId,
        subscribedWorkflows: workflowIds
      }
    })
  }

  removeConnection(connectionId: string): void {
    this.connections.delete(connectionId)
    this.subscribers.delete(connectionId)
    this.messageQueue.delete(connectionId)
  }

  updateConnectionSubscription(connectionId: string, workflowIds: string[]): void {
    const connection = this.connections.get(connectionId)
    if (connection) {
      connection.workflowIds = workflowIds
      this.connections.set(connectionId, connection)
    }
  }

  subscribe(connectionId: string, filter: SubscriptionFilter): void {
    this.subscribers.set(connectionId, filter)
  }

  // Real-time updates
  broadcastExecutionStart(execution: WorkflowExecution): void {
    if (!this.isEnabled) return

    const update: RealTimeUpdate = {
      type: 'execution_start',
      executionId: execution.id,
      workflowId: execution.workflow_id,
      timestamp: execution.started_at,
      data: {
        status: execution.status,
        triggerData: execution.trigger_data
      }
    }

    this.broadcast(update)
  }

  broadcastExecutionComplete(execution: WorkflowExecution): void {
    if (!this.isEnabled) return

    const update: RealTimeUpdate = {
      type: execution.status === 'completed' ? 'execution_complete' : 'execution_failed',
      executionId: execution.id,
      workflowId: execution.workflow_id,
      timestamp: execution.completed_at || TimestampUtils.dateToICPTimestamp(new Date()),
      data: {
        status: execution.status,
        duration: execution.duration ? Number(execution.duration) / 1000000 : null, // Convert to ms
        error: execution.error_message,
        nodeExecutions: execution.node_executions.length
      }
    }

    this.broadcast(update)
  }

  broadcastNodeStart(nodeExecution: NodeExecution): void {
    if (!this.isEnabled) return

    const update: RealTimeUpdate = {
      type: 'node_start',
      executionId: nodeExecution.execution_id,
      workflowId: 'unknown', // Would need to be passed or looked up
      nodeId: nodeExecution.node_id,
      timestamp: nodeExecution.started_at,
      data: {
        nodeId: nodeExecution.node_id,
        status: nodeExecution.status,
        inputData: nodeExecution.input_data
      }
    }

    this.broadcast(update)
  }

  broadcastNodeComplete(nodeExecution: NodeExecution): void {
    if (!this.isEnabled) return

    const update: RealTimeUpdate = {
      type: nodeExecution.status === 'completed' ? 'node_complete' : 'node_failed',
      executionId: nodeExecution.execution_id,
      workflowId: 'unknown', // Would need to be passed or looked up
      nodeId: nodeExecution.node_id,
      timestamp: nodeExecution.completed_at || TimestampUtils.dateToICPTimestamp(new Date()),
      data: {
        nodeId: nodeExecution.node_id,
        status: nodeExecution.status,
        duration: nodeExecution.duration ? Number(nodeExecution.duration) / 1000000 : null,
        outputData: nodeExecution.output_data,
        error: nodeExecution.error_message
      }
    }

    this.broadcast(update)
  }

  broadcastLog(log: ExecutionLog, executionId: string, workflowId: string): void {
    if (!this.isEnabled) return

    const update: RealTimeUpdate = {
      type: 'log',
      executionId,
      workflowId,
      nodeId: log.nodeId,
      timestamp: log.timestamp,
      data: {
        level: log.level,
        message: log.message,
        data: log.data
      }
    }

    this.broadcast(update)
  }

  // Broadcasting logic
  private broadcast(update: RealTimeUpdate): void {
    this.connections.forEach((connection, connectionId) => {
      if (!connection.isActive) return

      // Check if connection should receive this update
      if (this.shouldReceiveUpdate(connectionId, update)) {
        this.sendToConnection(connectionId, update)
      }
    })
  }

  private shouldReceiveUpdate(connectionId: string, update: RealTimeUpdate): boolean {
    const connection = this.connections.get(connectionId)
    const filter = this.subscribers.get(connectionId)

    if (!connection) return false

    // Check workflow subscription
    if (connection.workflowIds.length > 0 && !connection.workflowIds.includes(update.workflowId)) {
      return false
    }

    // Apply additional filters if specified
    if (filter) {
      if (filter.workflowIds && !filter.workflowIds.includes(update.workflowId)) return false
      if (filter.executionIds && !filter.executionIds.includes(update.executionId)) return false
      if (filter.logLevels && update.type === 'log' && !filter.logLevels.includes(update.data.level)) return false
    }

    return true
  }

  private sendToConnection(connectionId: string, update: RealTimeUpdate): void {
    // In a real implementation, this would send via WebSocket
    // For now, we'll add to a message queue that can be polled
    const queue = this.messageQueue.get(connectionId) || []
    queue.push(update)
    
    // Keep only last 100 messages per connection
    if (queue.length > 100) {
      queue.splice(0, queue.length - 100)
    }
    
    this.messageQueue.set(connectionId, queue)

    // Simulate WebSocket send
    console.log(`[WebSocket] Sending to ${connectionId}:`, update)
  }

  // Message polling (for simulation)
  getMessages(connectionId: string, lastMessageId?: string): RealTimeUpdate[] {
    const queue = this.messageQueue.get(connectionId) || []
    
    if (!lastMessageId) {
      return queue.slice(-10) // Return last 10 messages
    }

    // Find index of last message and return newer ones
    const lastIndex = queue.findIndex(msg => 
      `${msg.executionId}-${msg.timestamp}` === lastMessageId
    )
    
    if (lastIndex === -1) {
      return queue.slice(-10) // Return last 10 if not found
    }

    return queue.slice(lastIndex + 1)
  }

  // Connection health
  updateHeartbeat(connectionId: string): void {
    const connection = this.connections.get(connectionId)
    if (connection) {
      connection.lastHeartbeat = TimestampUtils.dateToICPTimestamp(new Date())
      connection.isActive = true
      this.connections.set(connectionId, connection)
    }
  }

  private startHeartbeat(): void {
    this.heartbeatInterval = setInterval(() => {
      const now = new Date()
      const fiveMinutesAgo = new Date(now.getTime() - 5 * 60 * 1000)

      this.connections.forEach((connection, connectionId) => {
        const lastHeartbeat = TimestampUtils.icpTimestampToDate(connection.lastHeartbeat)
        
        if (lastHeartbeat < fiveMinutesAgo) {
          connection.isActive = false
          this.connections.set(connectionId, connection)
          
          // Clean up inactive connections after 30 minutes
          const thirtyMinutesAgo = new Date(now.getTime() - 30 * 60 * 1000)
          if (lastHeartbeat < thirtyMinutesAgo) {
            this.removeConnection(connectionId)
          }
        }
      })
    }, 60000) // Check every minute
  }

  // Statistics
  getConnectionStats(): {
    totalConnections: number
    activeConnections: number
    connectionsByWorkflow: Record<string, number>
    messageQueueSizes: Record<string, number>
  } {
    const totalConnections = this.connections.size
    const activeConnections = Array.from(this.connections.values()).filter(c => c.isActive).length
    
    const connectionsByWorkflow: Record<string, number> = {}
    const messageQueueSizes: Record<string, number> = {}

    this.connections.forEach((connection, connectionId) => {
      connection.workflowIds.forEach(workflowId => {
        connectionsByWorkflow[workflowId] = (connectionsByWorkflow[workflowId] || 0) + 1
      })

      const queueSize = this.messageQueue.get(connectionId)?.length || 0
      messageQueueSizes[connectionId] = queueSize
    })

    return {
      totalConnections,
      activeConnections,
      connectionsByWorkflow,
      messageQueueSizes
    }
  }

  // Control
  enable(): void {
    this.isEnabled = true
  }

  disable(): void {
    this.isEnabled = false
  }

  isServiceEnabled(): boolean {
    return this.isEnabled
  }

  // Cleanup
  destroy(): void {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval)
    }
    this.connections.clear()
    this.subscribers.clear()
    this.messageQueue.clear()
  }

  // Bulk operations
  broadcastToWorkflow(workflowId: string, message: any): void {
    const update: RealTimeUpdate = {
      type: 'log',
      executionId: 'broadcast',
      workflowId,
      timestamp: TimestampUtils.dateToICPTimestamp(new Date()),
      data: message
    }

    this.broadcast(update)
  }

  broadcastSystemMessage(message: string, level: 'info' | 'warn' | 'error' = 'info'): void {
    const update: RealTimeUpdate = {
      type: 'log',
      executionId: 'system',
      workflowId: 'system',
      timestamp: TimestampUtils.dateToICPTimestamp(new Date()),
      data: {
        level,
        message,
        isSystemMessage: true
      }
    }

    // Send to all active connections
    this.connections.forEach((connection, connectionId) => {
      if (connection.isActive) {
        this.sendToConnection(connectionId, update)
      }
    })
  }

  // Performance monitoring
  getPerformanceMetrics(): {
    averageMessageDelay: number
    messagesPerSecond: number
    connectionThroughput: Record<string, number>
    queueOverflows: number
  } {
    // In a real implementation, these would be actual performance metrics
    return {
      averageMessageDelay: 25, // milliseconds
      messagesPerSecond: 150,
      connectionThroughput: {},
      queueOverflows: 0
    }
  }
}

// Create and export singleton instance
export const realTimeService = new RealTimeService()
export default realTimeService