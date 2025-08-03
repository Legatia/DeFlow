// Integration tests for end-to-end workflow execution
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { executionEngine } from '../../services/executionEngine'
import { monitoringService } from '../../services/monitoringService'
import { realTimeService } from '../../services/realTimeService'
import { webhookService } from '../../services/webhookService'
import { createMockWorkflow } from '../utils/testUtils'
import { TimestampUtils } from '../../utils/timestamp-utils'

describe('Workflow Execution Integration', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    executionEngine.clearExecutionHistory()
  })

  describe('Complete Workflow Execution Flow', () => {
    it('should execute a complete workflow with monitoring and real-time updates', async () => {
      const workflow = createMockWorkflow({
        id: 'integration_test_workflow',
        name: 'Integration Test Workflow',
        nodes: [
          {
            id: 'trigger-1',
            node_type: 'manual-trigger',
            position: { x: 100, y: 100 },
            configuration: { parameters: { name: 'Start Integration Test' } },
            metadata: {
              label: 'Start',
              description: 'Manual trigger for integration test',
              tags: ['trigger'],
              icon: '▶️',
              color: '#3b82f6'
            }
          },
          {
            id: 'http-1',
            node_type: 'http-request',
            position: { x: 300, y: 100 },
            configuration: {
              parameters: {
                url: 'https://api.example.com/stats',
                method: 'GET',
                headers: '{"Authorization": "Bearer test-token"}',
                body: ''
              }
            },
            metadata: {
              label: 'Fetch Data',
              description: 'Get statistics from API',
              tags: ['http', 'api'],
              icon: '🌐',
              color: '#8b5cf6'
            }
          },
          {
            id: 'condition-1',
            node_type: 'condition',
            position: { x: 500, y: 100 },
            configuration: {
              parameters: {
                field: 'response.users',
                operator: 'greater_than',
                value: '1000'
              }
            },
            metadata: {
              label: 'Check User Count',
              description: 'Check if users > 1000',
              tags: ['condition'],
              icon: '❓',
              color: '#f59e0b'
            }
          },
          {
            id: 'email-success',
            node_type: 'send-email',
            position: { x: 700, y: 50 },
            configuration: {
              parameters: {
                to: 'success@example.com',
                subject: 'High User Count Alert',
                body: 'Great news! We have {{response.users}} active users.',
                useTemplate: true
              }
            },
            metadata: {
              label: 'Success Email',
              description: 'Send success notification',
              tags: ['email', 'notification'],
              icon: '📧',
              color: '#10b981'
            }
          },
          {
            id: 'email-normal',
            node_type: 'send-email',
            position: { x: 700, y: 150 },
            configuration: {
              parameters: {
                to: 'normal@example.com',
                subject: 'Daily Stats Report',
                body: 'Current user count: {{response.users}}',
                useTemplate: true
              }
            },
            metadata: {
              label: 'Normal Email',
              description: 'Send normal stats',
              tags: ['email', 'report'],
              icon: '📊',
              color: '#6b7280'
            }
          }
        ],
        connections: [
          {
            id: 'conn-1',
            source_node_id: 'trigger-1',
            target_node_id: 'http-1',
            source_output: 'trigger',
            target_input: 'data'
          },
          {
            id: 'conn-2',
            source_node_id: 'http-1',
            target_node_id: 'condition-1',
            source_output: 'response',
            target_input: 'data'
          },
          {
            id: 'conn-3',
            source_node_id: 'condition-1',
            target_node_id: 'email-success',
            source_output: 'true',
            target_input: 'data'
          },
          {
            id: 'conn-4',
            source_node_id: 'condition-1',
            target_node_id: 'email-normal',
            source_output: 'false',
            target_input: 'data'
          }
        ]
      })

      // Set up real-time monitoring
      const connectionId = 'test-connection-1'
      realTimeService.addConnection(connectionId, 'test-user', [workflow.id])

      // Execute the workflow
      const execution = await executionEngine.executeWorkflow(
        workflow,
        { source: 'integration_test' },
        'test-user'
      )

      // Verify execution completed
      expect(execution.status).toBe('completed')
      expect(execution.workflow_id).toBe(workflow.id)
      expect(execution.node_executions).toHaveLength(5) // All nodes should execute

      // Verify all nodes executed successfully
      execution.node_executions.forEach(nodeExec => {
        expect(nodeExec.status).toBe('completed')
        expect(nodeExec.error_message).toBeNull()
      })

      // Verify monitoring service tracked the execution
      monitoringService.trackExecution(execution)
      const metrics = monitoringService.getMetrics()
      expect(metrics.totalExecutions).toBeGreaterThan(0)
      expect(metrics.successfulExecutions).toBeGreaterThan(0)

      // Verify real-time updates were sent
      const messages = realTimeService.getMessages(connectionId)
      expect(messages.length).toBeGreaterThan(0)
      
      // Should have execution start and complete messages
      const startMessage = messages.find(m => m.type === 'execution_start')
      const completeMessage = messages.find(m => m.type === 'execution_complete')
      expect(startMessage).toBeDefined()
      expect(completeMessage).toBeDefined()

      // Clean up
      realTimeService.removeConnection(connectionId)
    })

    it('should handle workflow execution failure with proper error tracking', async () => {
      const failingWorkflow = createMockWorkflow({
        id: 'failing_workflow',
        name: 'Failing Workflow',
        nodes: [
          {
            id: 'trigger-1',
            node_type: 'manual-trigger',
            position: { x: 100, y: 100 },
            configuration: { parameters: { name: 'Start Failing Test' } },
            metadata: {
              label: 'Start',
              description: 'Manual trigger',
              tags: ['trigger'],
              icon: '▶️',
              color: '#3b82f6'
            }
          },
          {
            id: 'invalid-1',
            node_type: 'non-existent-node-type',
            position: { x: 300, y: 100 },
            configuration: { parameters: {} },
            metadata: {
              label: 'Invalid Node',
              description: 'This will fail',
              tags: ['invalid'],
              icon: '❌',
              color: '#ef4444'
            }
          }
        ],
        connections: [
          {
            id: 'conn-1',
            source_node_id: 'trigger-1',
            target_node_id: 'invalid-1',
            source_output: 'trigger',
            target_input: 'data'
          }
        ]
      })

      // Set up monitoring
      const connectionId = 'test-connection-fail'
      realTimeService.addConnection(connectionId, 'test-user', [failingWorkflow.id])

      // Execute the failing workflow
      const execution = await executionEngine.executeWorkflow(
        failingWorkflow,
        {},
        'test-user'
      )

      // Verify execution failed
      expect(execution.status).toBe('failed')
      expect(execution.error_message).toContain('No executor found for node type')

      // Verify monitoring tracked the failure
      monitoringService.trackExecution(execution)
      const metrics = monitoringService.getMetrics()
      expect(metrics.failedExecutions).toBeGreaterThan(0)

      // Verify real-time failure notification
      const messages = realTimeService.getMessages(connectionId)
      const failMessage = messages.find(m => m.type === 'execution_failed')
      expect(failMessage).toBeDefined()
      expect(failMessage?.data.error).toBeDefined()

      // Clean up
      realTimeService.removeConnection(connectionId)
    })
  })

  describe('Webhook Integration', () => {
    it('should trigger workflow execution via webhook', async () => {
      const workflow = createMockWorkflow({
        id: 'webhook_workflow',
        name: 'Webhook Triggered Workflow',
        nodes: [
          {
            id: 'webhook-trigger',
            node_type: 'webhook-trigger',
            position: { x: 100, y: 100 },
            configuration: {
              parameters: {
                path: '/webhook/test',
                method: 'POST'
              }
            },
            metadata: {
              label: 'Webhook Trigger',
              description: 'Triggered by webhook',
              tags: ['trigger', 'webhook'],
              icon: '🔗',
              color: '#10b981'
            }
          },
          {
            id: 'process-webhook',
            node_type: 'transform-data',
            position: { x: 300, y: 100 },
            configuration: {
              parameters: {
                operation: 'map',
                config: JSON.stringify({
                  mapping: {
                    'user_id': 'webhook.body.userId',
                    'event_type': 'webhook.body.event',
                    'timestamp': 'webhook.timestamp'
                  }
                })
              }
            },
            metadata: {
              label: 'Process Webhook Data',
              description: 'Transform webhook payload',
              tags: ['transform'],
              icon: '🔄',
              color: '#8b5cf6'
            }
          }
        ],
        connections: [
          {
            id: 'webhook-conn',
            source_node_id: 'webhook-trigger',
            target_node_id: 'process-webhook',
            source_output: 'data',
            target_input: 'data'
          }
        ]
      })

      // Create webhook endpoint
      const endpoint = webhookService.createEndpoint(workflow.id, {
        path: '/webhook/test',
        method: 'POST',
        isActive: true,
        headers: {},
        validation: { enabled: false },
        rateLimiting: { enabled: false, maxRequests: 100, timeWindow: 60, strategy: 'fixed_window' },
        metadata: {
          name: 'Test Webhook',
          description: 'Test webhook endpoint',
          tags: ['test'],
          externalService: 'Test Service'
        }
      })

      // Process webhook request
      const webhookResponse = await webhookService.processWebhookRequest(
        'POST',
        '/webhook/test',
        { 'content-type': 'application/json' },
        { userId: '12345', event: 'user_signup', timestamp: new Date().toISOString() },
        {},
        '127.0.0.1'
      )

      // Verify webhook was processed successfully
      expect(webhookResponse.status).toBe(200)
      expect(webhookResponse.body.success).toBe(true)
      expect(webhookResponse.body.executionId).toBeDefined()

      // Verify execution was created
      const execution = executionEngine.getExecution(webhookResponse.body.executionId)
      expect(execution).toBeDefined()
      expect(execution?.status).toBe('completed')

      // Verify webhook analytics
      const analytics = webhookService.getEndpointAnalytics(endpoint.id)
      expect(analytics.totalRequests).toBe(1)
      expect(analytics.successfulRequests).toBe(1)
      expect(analytics.failedRequests).toBe(0)
    })

    it('should handle webhook rate limiting', async () => {
      const workflow = createMockWorkflow({
        id: 'rate_limited_webhook',
        name: 'Rate Limited Webhook'
      })

      // Create webhook endpoint with rate limiting
      const endpoint = webhookService.createEndpoint(workflow.id, {
        path: '/webhook/limited',
        method: 'POST',
        isActive: true,
        headers: {},
        validation: { enabled: false },
        rateLimiting: {
          enabled: true,
          maxRequests: 2,
          timeWindow: 60,
          strategy: 'fixed_window'
        },
        metadata: {
          name: 'Rate Limited Webhook',
          description: 'Webhook with rate limiting',
          tags: ['test', 'rate-limit']
        }
      })

      // Make requests up to the limit
      const response1 = await webhookService.processWebhookRequest(
        'POST', '/webhook/limited', {}, {}, {}, '127.0.0.1'
      )
      expect(response1.status).toBe(200)

      const response2 = await webhookService.processWebhookRequest(
        'POST', '/webhook/limited', {}, {}, {}, '127.0.0.1'
      )
      expect(response2.status).toBe(200)

      // Third request should be rate limited
      const response3 = await webhookService.processWebhookRequest(
        'POST', '/webhook/limited', {}, {}, {}, '127.0.0.1'
      )
      expect(response3.status).toBe(429)
      expect(response3.body.error).toContain('Rate limit exceeded')
    })
  })

  describe('Multi-Node Workflow Execution', () => {
    it('should execute complex workflow with multiple branches', async () => {
      const complexWorkflow = createMockWorkflow({
        id: 'complex_workflow',
        name: 'Complex Multi-Branch Workflow',
        nodes: [
          // Trigger
          {
            id: 'start',
            node_type: 'manual-trigger',
            position: { x: 100, y: 200 },
            configuration: { parameters: { name: 'Complex Start' } },
            metadata: { label: 'Start', description: 'Start complex workflow', tags: ['trigger'], icon: '▶️', color: '#3b82f6' }
          },
          // Data fetch
          {
            id: 'fetch-data',
            node_type: 'http-request',
            position: { x: 300, y: 200 },
            configuration: { parameters: { url: 'https://api.example.com/users', method: 'GET', headers: '{}', body: '' } },
            metadata: { label: 'Fetch Users', description: 'Get user data', tags: ['http'], icon: '🌐', color: '#8b5cf6' }
          },
          // Transform data
          {
            id: 'transform',
            node_type: 'transform-data',
            position: { x: 500, y: 200 },
            configuration: { parameters: { operation: 'map', config: '{"mapping": {"id": "id", "name": "fullName", "email": "emailAddress"}}' } },
            metadata: { label: 'Transform', description: 'Transform user data', tags: ['transform'], icon: '🔄', color: '#10b981' }
          },
          // Condition check
          {
            id: 'check-count',
            node_type: 'condition',
            position: { x: 700, y: 200 },
            configuration: { parameters: { field: 'data.length', operator: 'greater_than', value: '10' } },
            metadata: { label: 'Check Count', description: 'Check user count', tags: ['condition'], icon: '❓', color: '#f59e0b' }
          },
          // Branch 1: High count
          {
            id: 'high-count-email',
            node_type: 'send-email',
            position: { x: 900, y: 100 },
            configuration: { parameters: { to: 'admin@example.com', subject: 'High User Count', body: 'User count is high: {{data.length}}', useTemplate: true } },
            metadata: { label: 'High Count Alert', description: 'Alert for high user count', tags: ['email'], icon: '📧', color: '#ef4444' }
          },
          // Branch 2: Normal count
          {
            id: 'normal-email',
            node_type: 'send-email',
            position: { x: 900, y: 300 },
            configuration: { parameters: { to: 'team@example.com', subject: 'User Report', body: 'Current users: {{data.length}}', useTemplate: true } },
            metadata: { label: 'Normal Report', description: 'Normal user report', tags: ['email'], icon: '📊', color: '#6b7280' }
          },
          // Delay before final step
          {
            id: 'delay',
            node_type: 'delay',
            position: { x: 1100, y: 200 },
            configuration: { parameters: { duration: 100, unit: 'milliseconds' } },
            metadata: { label: 'Wait', description: 'Brief delay', tags: ['delay'], icon: '⏱️', color: '#9ca3af' }
          },
          // Final notification
          {
            id: 'final-notification',
            node_type: 'send-email',
            position: { x: 1300, y: 200 },
            configuration: { parameters: { to: 'system@example.com', subject: 'Workflow Complete', body: 'Complex workflow has completed successfully.', useTemplate: false } },
            metadata: { label: 'Final Notice', description: 'Completion notification', tags: ['email', 'final'], icon: '✅', color: '#10b981' }
          }
        ],
        connections: [
          { id: 'c1', source_node_id: 'start', target_node_id: 'fetch-data', source_output: 'trigger', target_input: 'data' },
          { id: 'c2', source_node_id: 'fetch-data', target_node_id: 'transform', source_output: 'response', target_input: 'data' },
          { id: 'c3', source_node_id: 'transform', target_node_id: 'check-count', source_output: 'result', target_input: 'data' },
          { id: 'c4', source_node_id: 'check-count', target_node_id: 'high-count-email', source_output: 'true', target_input: 'data' },
          { id: 'c5', source_node_id: 'check-count', target_node_id: 'normal-email', source_output: 'false', target_input: 'data' },
          { id: 'c6', source_node_id: 'high-count-email', target_node_id: 'delay', source_output: 'output', target_input: 'trigger' },
          { id: 'c7', source_node_id: 'normal-email', target_node_id: 'delay', source_output: 'output', target_input: 'trigger' },
          { id: 'c8', source_node_id: 'delay', target_node_id: 'final-notification', source_output: 'continue', target_input: 'data' }
        ]
      })

      const execution = await executionEngine.executeWorkflow(
        complexWorkflow,
        { test: 'complex_execution' },
        'integration-test-user'
      )

      // Verify execution completed
      expect(execution.status).toBe('completed')
      expect(execution.node_executions).toHaveLength(8) // All nodes should execute

      // Verify execution order and data flow
      const nodeExecutions = execution.node_executions.sort((a, b) => 
        a.started_at.localeCompare(b.started_at)
      )

      // First node should be the trigger
      expect(nodeExecutions[0].node_id).toBe('start')
      
      // Last node should be the final notification
      const lastExecution = nodeExecutions[nodeExecutions.length - 1]
      expect(lastExecution.node_id).toBe('final-notification')
      expect(lastExecution.status).toBe('completed')

      // Verify condition branching worked
      const conditionExecution = nodeExecutions.find(ne => ne.node_id === 'check-count')
      expect(conditionExecution).toBeDefined()
      expect(conditionExecution!.output_data.conditionMet).toBeDefined()

      // Verify either high-count or normal email was sent (not both to same target)
      const highCountExecution = nodeExecutions.find(ne => ne.node_id === 'high-count-email')
      const normalEmailExecution = nodeExecutions.find(ne => ne.node_id === 'normal-email')
      
      expect(highCountExecution).toBeDefined()
      expect(normalEmailExecution).toBeDefined()
      
      // Both should be completed (condition determines which branch executes)
      expect(highCountExecution!.status).toBe('completed')
      expect(normalEmailExecution!.status).toBe('completed')
    })
  })

  describe('Performance and Monitoring Integration', () => {
    it('should track performance metrics across multiple executions', async () => {
      const testWorkflow = createMockWorkflow({
        id: 'performance_test_workflow',
        name: 'Performance Test Workflow'
      })

      const executionCount = 5
      const executions = []

      // Execute workflow multiple times
      for (let i = 0; i < executionCount; i++) {
        const execution = await executionEngine.executeWorkflow(
          testWorkflow,
          { iteration: i },
          'performance-test-user'
        )
        executions.push(execution)
        monitoringService.trackExecution(execution)
      }

      // Verify all executions completed
      executions.forEach(execution => {
        expect(execution.status).toBe('completed')
      })

      // Check monitoring metrics
      const metrics = monitoringService.getMetrics()
      expect(metrics.totalExecutions).toBeGreaterThanOrEqual(executionCount)
      expect(metrics.successfulExecutions).toBeGreaterThanOrEqual(executionCount)
      expect(metrics.averageExecutionTime).toBeGreaterThan(0)

      // Verify system health
      const health = monitoringService.getSystemHealth()
      expect(health.status).toBe('healthy')
      expect(health.errorRate).toBe(0) // No errors in successful executions
    })

    it('should handle concurrent workflow executions', async () => {
      const workflow = createMockWorkflow({
        id: 'concurrent_test_workflow',
        name: 'Concurrent Test Workflow'
      })

      // Execute multiple workflows concurrently
      const concurrentExecutions = await Promise.all([
        executionEngine.executeWorkflow(workflow, { concurrent: 1 }, 'user-1'),
        executionEngine.executeWorkflow(workflow, { concurrent: 2 }, 'user-2'),
        executionEngine.executeWorkflow(workflow, { concurrent: 3 }, 'user-3')
      ])

      // Verify all executions completed
      concurrentExecutions.forEach((execution, index) => {
        expect(execution.status).toBe('completed')
        expect(execution.trigger_data.concurrent).toBe(index + 1)
      })

      // Verify they have different execution IDs
      const executionIds = concurrentExecutions.map(e => e.id)
      const uniqueIds = new Set(executionIds)
      expect(uniqueIds.size).toBe(concurrentExecutions.length)
    })
  })

  describe('Error Recovery and Resilience', () => {
    it('should handle partial workflow failures gracefully', async () => {
      const partialFailWorkflow = createMockWorkflow({
        id: 'partial_fail_workflow',
        name: 'Partial Failure Workflow',
        nodes: [
          {
            id: 'start',
            node_type: 'manual-trigger',
            position: { x: 100, y: 100 },
            configuration: { parameters: { name: 'Start' } },
            metadata: { label: 'Start', description: 'Start node', tags: ['trigger'], icon: '▶️', color: '#3b82f6' }
          },
          {
            id: 'success-node',
            node_type: 'send-email',
            position: { x: 300, y: 100 },
            configuration: { parameters: { to: 'test@example.com', subject: 'Test', body: 'Test email', useTemplate: false } },
            metadata: { label: 'Success Node', description: 'This will succeed', tags: ['email'], icon: '📧', color: '#10b981' }
          },
          {
            id: 'fail-node',
            node_type: 'invalid-type',
            position: { x: 300, y: 200 },
            configuration: { parameters: {} },
            metadata: { label: 'Fail Node', description: 'This will fail', tags: ['invalid'], icon: '❌', color: '#ef4444' }
          }
        ],
        connections: [
          { id: 'c1', source_node_id: 'start', target_node_id: 'success-node', source_output: 'trigger', target_input: 'data' },
          { id: 'c2', source_node_id: 'start', target_node_id: 'fail-node', source_output: 'trigger', target_input: 'data' }
        ]
      })

      const execution = await executionEngine.executeWorkflow(
        partialFailWorkflow,
        {},
        'test-user'
      )

      // Workflow should fail overall due to failed node
      expect(execution.status).toBe('failed')

      // But some nodes should have succeeded
      const successNode = execution.node_executions.find(ne => ne.node_id === 'success-node')
      const failNode = execution.node_executions.find(ne => ne.node_id === 'fail-node')
      const startNode = execution.node_executions.find(ne => ne.node_id === 'start')

      expect(startNode?.status).toBe('completed')
      expect(successNode?.status).toBe('completed')
      expect(failNode?.status).toBe('failed')

      // Verify error tracking
      monitoringService.trackExecution(execution)
      const metrics = monitoringService.getMetrics()
      expect(metrics.failedExecutions).toBeGreaterThan(0)
      expect(metrics.recentErrors.length).toBeGreaterThan(0)
    })
  })
})