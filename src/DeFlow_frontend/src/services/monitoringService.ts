// Comprehensive monitoring and logging service for DeFlow
import { WorkflowExecution, NodeExecution } from '../types'
import { ExecutionLog } from './executionEngine'
import { TimestampUtils } from '../utils/timestamp-utils'

export interface MonitoringMetrics {
  totalExecutions: number
  successfulExecutions: number
  failedExecutions: number
  averageExecutionTime: number
  activeExecutions: number
  executionsToday: number
  executionsThisWeek: number
  successRate: number
  mostActiveWorkflows: Array<{
    workflowId: string
    executions: number
    successRate: number
  }>
  recentErrors: Array<{
    executionId: string
    workflowId: string
    error: string
    timestamp: string
  }>
}

export interface SystemHealth {
  status: 'healthy' | 'warning' | 'critical'
  uptime: number
  memoryUsage: number
  activeConnections: number
  queueSize: number
  errorRate: number
  responseTime: number
  lastHealthCheck: string
}

export interface AlertRule {
  id: string
  name: string
  type: 'execution_failure' | 'high_error_rate' | 'slow_execution' | 'system_health'
  threshold: number
  enabled: boolean
  webhookUrl?: string
  emailNotification?: boolean
  conditions: Record<string, any>
}

export interface Alert {
  id: string
  ruleId: string
  type: string
  severity: 'low' | 'medium' | 'high' | 'critical'
  message: string
  timestamp: string
  acknowledged: boolean
  metadata: Record<string, any>
}

class MonitoringService {
  private executions: Map<string, WorkflowExecution> = new Map()
  private logs: Map<string, ExecutionLog[]> = new Map()
  private alerts: Alert[] = []
  private alertRules: AlertRule[] = []
  private systemStartTime: Date = new Date()
  private healthCheckInterval: NodeJS.Timeout | null = null

  constructor() {
    this.initializeDefaultAlertRules()
    this.startHealthChecks()
  }

  // Execution tracking
  trackExecution(execution: WorkflowExecution): void {
    this.executions.set(execution.id, execution)
    this.checkAlertRules(execution)
  }

  trackExecutionLogs(executionId: string, logs: ExecutionLog[]): void {
    this.logs.set(executionId, logs)
  }

  updateExecution(execution: WorkflowExecution): void {
    this.executions.set(execution.id, execution)
    this.checkAlertRules(execution)
  }

  // Metrics calculation
  getMetrics(): MonitoringMetrics {
    const executions = Array.from(this.executions.values())
    const now = new Date()
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
    const weekAgo = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000)

    const totalExecutions = executions.length
    const successfulExecutions = executions.filter(e => e.status === 'completed').length
    const failedExecutions = executions.filter(e => e.status === 'failed').length
    const activeExecutions = executions.filter(e => e.status === 'running').length

    const executionsToday = executions.filter(e => {
      const executionDate = TimestampUtils.icpTimestampToDate(e.started_at)
      return executionDate >= today
    }).length

    const executionsThisWeek = executions.filter(e => {
      const executionDate = TimestampUtils.icpTimestampToDate(e.started_at)
      return executionDate >= weekAgo
    }).length

    const completedExecutions = executions.filter(e => e.status !== 'running')
    const averageExecutionTime = completedExecutions.length > 0
      ? completedExecutions.reduce((sum, e) => {
          if (e.duration) {
            return sum + Number(e.duration) / 1000000 // Convert nanoseconds to milliseconds
          }
          return sum
        }, 0) / completedExecutions.length
      : 0

    const successRate = totalExecutions > 0 ? (successfulExecutions / totalExecutions) * 100 : 0

    // Calculate most active workflows
    const workflowStats = new Map<string, { executions: number; successful: number }>()
    executions.forEach(e => {
      const stats = workflowStats.get(e.workflow_id) || { executions: 0, successful: 0 }
      stats.executions++
      if (e.status === 'completed') stats.successful++
      workflowStats.set(e.workflow_id, stats)
    })

    const mostActiveWorkflows = Array.from(workflowStats.entries())
      .map(([workflowId, stats]) => ({
        workflowId,
        executions: stats.executions,
        successRate: (stats.successful / stats.executions) * 100
      }))
      .sort((a, b) => b.executions - a.executions)
      .slice(0, 10)

    // Recent errors (last 24 hours)
    const yesterday = new Date(now.getTime() - 24 * 60 * 60 * 1000)
    const recentErrors = executions
      .filter(e => {
        const executionDate = TimestampUtils.icpTimestampToDate(e.started_at)
        return e.status === 'failed' && executionDate >= yesterday
      })
      .map(e => ({
        executionId: e.id,
        workflowId: e.workflow_id,
        error: e.error_message || 'Unknown error',
        timestamp: e.started_at
      }))
      .sort((a, b) => b.timestamp.localeCompare(a.timestamp))
      .slice(0, 20)

    return {
      totalExecutions,
      successfulExecutions,
      failedExecutions,
      averageExecutionTime,
      activeExecutions,
      executionsToday,
      executionsThisWeek,
      successRate,
      mostActiveWorkflows,
      recentErrors
    }
  }

  // System health monitoring
  getSystemHealth(): SystemHealth {
    const now = new Date()
    const uptime = now.getTime() - this.systemStartTime.getTime()
    const executions = Array.from(this.executions.values())
    
    // Calculate error rate (last hour)
    const lastHour = new Date(now.getTime() - 60 * 60 * 1000)
    const recentExecutions = executions.filter(e => {
      const executionDate = TimestampUtils.icpTimestampToDate(e.started_at)
      return executionDate >= lastHour
    })
    
    const errorRate = recentExecutions.length > 0
      ? (recentExecutions.filter(e => e.status === 'failed').length / recentExecutions.length) * 100
      : 0

    // Calculate average response time (last hour)
    const completedRecent = recentExecutions.filter(e => e.status !== 'running' && e.duration)
    const responseTime = completedRecent.length > 0
      ? completedRecent.reduce((sum, e) => sum + Number(e.duration!) / 1000000, 0) / completedRecent.length
      : 0

    const activeConnections = executions.filter(e => e.status === 'running').length
    const queueSize = 0 // Would be actual queue size in real implementation

    // Determine overall health status
    let status: SystemHealth['status'] = 'healthy'
    if (errorRate > 50 || responseTime > 10000) {
      status = 'critical'
    } else if (errorRate > 20 || responseTime > 5000) {
      status = 'warning'
    }

    return {
      status,
      uptime,
      memoryUsage: 0, // Would be actual memory usage
      activeConnections,
      queueSize,
      errorRate,
      responseTime,
      lastHealthCheck: TimestampUtils.dateToICPTimestamp(now)
    }
  }

  // Alert management
  addAlertRule(rule: Omit<AlertRule, 'id'>): AlertRule {
    const alertRule: AlertRule = {
      ...rule,
      id: this.generateId('rule')
    }
    this.alertRules.push(alertRule)
    return alertRule
  }

  updateAlertRule(ruleId: string, updates: Partial<AlertRule>): AlertRule | null {
    const index = this.alertRules.findIndex(r => r.id === ruleId)
    if (index === -1) return null

    this.alertRules[index] = { ...this.alertRules[index], ...updates }
    return this.alertRules[index]
  }

  deleteAlertRule(ruleId: string): boolean {
    const index = this.alertRules.findIndex(r => r.id === ruleId)
    if (index === -1) return false

    this.alertRules.splice(index, 1)
    return true
  }

  getAlertRules(): AlertRule[] {
    return [...this.alertRules]
  }

  getAlerts(limit: number = 50): Alert[] {
    return this.alerts
      .sort((a, b) => b.timestamp.localeCompare(a.timestamp))
      .slice(0, limit)
  }

  acknowledgeAlert(alertId: string): boolean {
    const alert = this.alerts.find(a => a.id === alertId)
    if (!alert) return false

    alert.acknowledged = true
    return true
  }

  clearAcknowledgedAlerts(): void {
    this.alerts = this.alerts.filter(a => !a.acknowledged)
  }

  // Private methods
  private initializeDefaultAlertRules(): void {
    this.alertRules = [
      {
        id: 'rule_001',
        name: 'High Error Rate',
        type: 'high_error_rate',
        threshold: 25, // 25% error rate
        enabled: true,
        emailNotification: true,
        conditions: { timeWindow: 60 } // minutes
      },
      {
        id: 'rule_002',
        name: 'Slow Execution',
        type: 'slow_execution',
        threshold: 10000, // 10 seconds
        enabled: true,
        emailNotification: false,
        conditions: { consecutive: 3 }
      },
      {
        id: 'rule_003',
        name: 'Execution Failure',
        type: 'execution_failure',
        threshold: 1,
        enabled: true,
        emailNotification: true,
        conditions: { severity: 'high' }
      }
    ]
  }

  private checkAlertRules(execution: WorkflowExecution): void {
    this.alertRules.forEach(rule => {
      if (!rule.enabled) return

      switch (rule.type) {
        case 'execution_failure':
          if (execution.status === 'failed') {
            this.createAlert(rule, 'high', `Workflow execution failed: ${execution.error_message}`, {
              executionId: execution.id,
              workflowId: execution.workflow_id
            })
          }
          break

        case 'slow_execution':
          if (execution.status !== 'running' && execution.duration) {
            const durationMs = Number(execution.duration) / 1000000
            if (durationMs > rule.threshold) {
              this.createAlert(rule, 'medium', `Slow execution detected: ${durationMs.toFixed(2)}ms`, {
                executionId: execution.id,
                workflowId: execution.workflow_id,
                duration: durationMs
              })
            }
          }
          break

        case 'high_error_rate':
          this.checkErrorRate(rule)
          break
      }
    })
  }

  private checkErrorRate(rule: AlertRule): void {
    const now = new Date()
    const timeWindow = rule.conditions.timeWindow || 60 // minutes
    const windowStart = new Date(now.getTime() - timeWindow * 60 * 1000)

    const recentExecutions = Array.from(this.executions.values()).filter(e => {
      const executionDate = TimestampUtils.icpTimestampToDate(e.started_at)
      return executionDate >= windowStart
    })

    if (recentExecutions.length === 0) return

    const errorRate = (recentExecutions.filter(e => e.status === 'failed').length / recentExecutions.length) * 100

    if (errorRate > rule.threshold) {
      this.createAlert(rule, 'high', `High error rate detected: ${errorRate.toFixed(1)}%`, {
        errorRate,
        timeWindow,
        totalExecutions: recentExecutions.length,
        failedExecutions: recentExecutions.filter(e => e.status === 'failed').length
      })
    }
  }

  private createAlert(rule: AlertRule, severity: Alert['severity'], message: string, metadata: Record<string, any>): void {
    // Check if similar alert already exists (debouncing)
    const existingAlert = this.alerts.find(a => 
      a.ruleId === rule.id && 
      !a.acknowledged && 
      new Date().getTime() - new Date(TimestampUtils.icpTimestampToDate(a.timestamp)).getTime() < 5 * 60 * 1000 // 5 minutes
    )

    if (existingAlert) return

    const alert: Alert = {
      id: this.generateId('alert'),
      ruleId: rule.id,
      type: rule.type,
      severity,
      message,
      timestamp: TimestampUtils.dateToICPTimestamp(new Date()),
      acknowledged: false,
      metadata
    }

    this.alerts.push(alert)

    // Trigger notifications if configured
    if (rule.emailNotification) {
      this.sendEmailNotification(alert)
    }
    if (rule.webhookUrl) {
      this.sendWebhookNotification(alert, rule.webhookUrl)
    }
  }

  private async sendEmailNotification(alert: Alert): Promise<void> {
    // Simulate email notification
    console.log(`[EMAIL ALERT] ${alert.severity.toUpperCase()}: ${alert.message}`)
  }

  private async sendWebhookNotification(alert: Alert, webhookUrl: string): Promise<void> {
    // Simulate webhook notification
    console.log(`[WEBHOOK ALERT] ${webhookUrl}: ${alert.message}`)
  }

  private startHealthChecks(): void {
    this.healthCheckInterval = setInterval(() => {
      const health = this.getSystemHealth()
      if (health.status !== 'healthy') {
        const rule = this.alertRules.find(r => r.type === 'system_health')
        if (rule?.enabled) {
          this.createAlert(rule, health.status === 'critical' ? 'critical' : 'medium', 
            `System health status: ${health.status}`, health)
        }
      }
    }, 60000) // Check every minute
  }

  private generateId(prefix: string): string {
    return `${prefix}_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
  }

  // Cleanup
  destroy(): void {
    if (this.healthCheckInterval) {
      clearInterval(this.healthCheckInterval)
    }
  }

  // Data export for analytics
  exportExecutionData(startDate?: Date, endDate?: Date): any[] {
    const executions = Array.from(this.executions.values())
    
    return executions
      .filter(e => {
        const executionDate = TimestampUtils.icpTimestampToDate(e.started_at)
        if (startDate && executionDate < startDate) return false
        if (endDate && executionDate > endDate) return false
        return true
      })
      .map(e => ({
        id: e.id,
        workflowId: e.workflow_id,
        status: e.status,
        startedAt: e.started_at,
        completedAt: e.completed_at,
        duration: e.duration ? Number(e.duration) / 1000000 : null, // Convert to milliseconds
        error: e.error_message,
        nodeCount: e.node_executions.length
      }))
  }
}

// Export singleton instance
export const monitoringService = new MonitoringService()
export default monitoringService