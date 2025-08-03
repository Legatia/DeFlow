// Tests for the workflow execution engine
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { executionEngine } from '../../services/executionEngine'
import { createMockWorkflow } from '../utils/testUtils'
import { TimestampUtils } from '../../utils/timestamp-utils'

describe('ExecutionEngine', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    executionEngine.clearExecutionHistory()
  })

  describe('Workflow Execution', () => {
    it('should execute a simple workflow successfully', async () => {
      const workflow = createMockWorkflow({
        nodes: [
          {
            id: 'trigger-1',
            node_type: 'manual-trigger',
            position: { x: 100, y: 100 },
            configuration: { parameters: { name: 'Test Trigger' } },
            metadata: {
              label: 'Start',
              description: 'Manual trigger',
              tags: ['trigger'],
              icon: '▶️',
              color: '#3b82f6'
            }
          },
          {
            id: 'email-1',
            node_type: 'send-email',
            position: { x: 400, y: 100 },
            configuration: {
              parameters: {
                to: 'test@example.com',
                subject: 'Test Email',
                body: 'Hello World'
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
            id: 'conn-1',
            source_node_id: 'trigger-1',
            target_node_id: 'email-1',
            source_output: 'trigger',
            target_input: 'data'
          }
        ]
      })

      const execution = await executionEngine.executeWorkflow(workflow, {}, 'test-user')

      expect(execution).toBeDefined()
      expect(execution.status).toBe('completed')
      expect(execution.workflow_id).toBe(workflow.id)
      expect(execution.node_executions).toHaveLength(2)
      expect(execution.error_message).toBeNull()
      expect(execution.duration).toBeDefined()
    })

    it('should handle workflow execution failure', async () => {
      const workflow = createMockWorkflow({
        nodes: [
          {
            id: 'trigger-1',
            node_type: 'manual-trigger',
            position: { x: 100, y: 100 },
            configuration: { parameters: { name: 'Test Trigger' } },
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
            node_type: 'invalid-node-type',
            position: { x: 400, y: 100 },
            configuration: { parameters: {} },
            metadata: {
              label: 'Invalid Node',
              description: 'This should fail',
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

      const execution = await executionEngine.executeWorkflow(workflow, {}, 'test-user')

      expect(execution.status).toBe('failed')
      expect(execution.error_message).toContain('No executor found for node type')
    })

    it('should handle workflows with no trigger nodes', async () => {
      const workflow = createMockWorkflow({
        nodes: [
          {
            id: 'email-1',
            node_type: 'send-email',
            position: { x: 400, y: 100 },
            configuration: {
              parameters: {
                to: 'test@example.com',
                subject: 'Test Email',
                body: 'Hello World'
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
        connections: []
      })

      const execution = await executionEngine.executeWorkflow(workflow, {}, 'test-user')

      expect(execution.status).toBe('failed')
      expect(execution.error_message).toBe('No trigger nodes found in workflow')
    })
  })

  describe('Node Execution', () => {
    it('should execute manual trigger node', async () => {
      const workflow = createMockWorkflow({
        nodes: [
          {
            id: 'trigger-1',
            node_type: 'manual-trigger',
            position: { x: 100, y: 100 },
            configuration: { parameters: { name: 'Test Trigger' } },
            metadata: {
              label: 'Start',
              description: 'Manual trigger',
              tags: ['trigger'],
              icon: '▶️',
              color: '#3b82f6'
            }
          }
        ],
        connections: []
      })

      const execution = await executionEngine.executeWorkflow(workflow, { test: 'data' }, 'test-user')

      expect(execution.status).toBe('completed')
      expect(execution.node_executions).toHaveLength(1)
      
      const nodeExecution = execution.node_executions[0]
      expect(nodeExecution.node_id).toBe('trigger-1')
      expect(nodeExecution.status).toBe('completed')
      expect(nodeExecution.output_data).toMatchObject({
        trigger: 'manual',
        test: 'data'
      })
    })

    it('should execute email node with template processing', async () => {
      const workflow = createMockWorkflow({
        nodes: [
          {
            id: 'trigger-1',
            node_type: 'manual-trigger',
            position: { x: 100, y: 100 },
            configuration: { parameters: { name: 'Test Trigger' } },
            metadata: {
              label: 'Start',
              description: 'Manual trigger',
              tags: ['trigger'],
              icon: '▶️',
              color: '#3b82f6'
            }
          },
          {
            id: 'email-1',
            node_type: 'send-email',
            position: { x: 400, y: 100 },
            configuration: {
              parameters: {
                to: 'test@example.com',
                subject: 'Hello {{data.name}}',
                body: 'Welcome {{data.name}}! Your ID is {{data.id}}.',
                useTemplate: true
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
            id: 'conn-1',
            source_node_id: 'trigger-1',
            target_node_id: 'email-1',
            source_output: 'trigger',
            target_input: 'data'
          }
        ]
      })

      const triggerData = { name: 'John Doe', id: '12345' }
      const execution = await executionEngine.executeWorkflow(workflow, triggerData, 'test-user')

      expect(execution.status).toBe('completed')
      
      const emailExecution = execution.node_executions.find(ne => ne.node_id === 'email-1')
      expect(emailExecution).toBeDefined()
      expect(emailExecution!.status).toBe('completed')
      expect(emailExecution!.output_data.emailSent).toBe(true)
    })

    it('should execute HTTP request node', async () => {
      const workflow = createMockWorkflow({
        nodes: [
          {
            id: 'trigger-1',
            node_type: 'manual-trigger',
            position: { x: 100, y: 100 },
            configuration: { parameters: { name: 'Test Trigger' } },
            metadata: {
              label: 'Start',
              description: 'Manual trigger',
              tags: ['trigger'],
              icon: '▶️',
              color: '#3b82f6'
            }
          },
          {
            id: 'http-1',
            node_type: 'http-request',
            position: { x: 400, y: 100 },
            configuration: {
              parameters: {
                url: 'https://api.example.com/stats',
                method: 'GET',
                headers: '{}',
                body: ''
              }
            },
            metadata: {
              label: 'HTTP Request',
              description: 'Make API call',
              tags: ['http'],
              icon: '🌐',
              color: '#8b5cf6'
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
          }
        ]
      })

      const execution = await executionEngine.executeWorkflow(workflow, {}, 'test-user')

      expect(execution.status).toBe('completed')
      
      const httpExecution = execution.node_executions.find(ne => ne.node_id === 'http-1')
      expect(httpExecution).toBeDefined()
      expect(httpExecution!.status).toBe('completed')
      expect(httpExecution!.output_data.response).toBeDefined()
      expect(httpExecution!.output_data.status).toBe(200)
    })

    it('should execute condition node with branching', async () => {
      const workflow = createMockWorkflow({
        nodes: [
          {
            id: 'trigger-1',
            node_type: 'manual-trigger',
            position: { x: 100, y: 100 },
            configuration: { parameters: { name: 'Test Trigger' } },
            metadata: {
              label: 'Start',
              description: 'Manual trigger',
              tags: ['trigger'],
              icon: '▶️',
              color: '#3b82f6'
            }
          },
          {
            id: 'condition-1',
            node_type: 'condition',
            position: { x: 400, y: 100 },
            configuration: {
              parameters: {
                field: 'status',
                operator: 'equals',
                value: 'success'
              }
            },
            metadata: {
              label: 'Check Status',
              description: 'Check if status is success',
              tags: ['condition'],
              icon: '❓',
              color: '#f59e0b'
            }
          }
        ],
        connections: [
          {
            id: 'conn-1',
            source_node_id: 'trigger-1',
            target_node_id: 'condition-1',
            source_output: 'trigger',
            target_input: 'data'
          }
        ]
      })

      const execution = await executionEngine.executeWorkflow(
        workflow, 
        { status: 'success' }, 
        'test-user'
      )

      expect(execution.status).toBe('completed')
      
      const conditionExecution = execution.node_executions.find(ne => ne.node_id === 'condition-1')
      expect(conditionExecution).toBeDefined()
      expect(conditionExecution!.status).toBe('completed')
      expect(conditionExecution!.output_data.conditionMet).toBe(true)
    })

    it('should execute delay node', async () => {
      const workflow = createMockWorkflow({
        nodes: [
          {
            id: 'trigger-1',
            node_type: 'manual-trigger',
            position: { x: 100, y: 100 },
            configuration: { parameters: { name: 'Test Trigger' } },
            metadata: {
              label: 'Start',
              description: 'Manual trigger',
              tags: ['trigger'],
              icon: '▶️',
              color: '#3b82f6'
            }
          },
          {
            id: 'delay-1',
            node_type: 'delay',
            position: { x: 400, y: 100 },
            configuration: {
              parameters: {
                duration: 100,
                unit: 'milliseconds'
              }
            },
            metadata: {
              label: 'Wait',
              description: 'Wait 100ms',
              tags: ['delay'],
              icon: '⏱️',
              color: '#6b7280'
            }
          }
        ],
        connections: [
          {
            id: 'conn-1',
            source_node_id: 'trigger-1',
            target_node_id: 'delay-1',
            source_output: 'trigger',
            target_input: 'trigger'
          }
        ]
      })

      const startTime = Date.now()
      const execution = await executionEngine.executeWorkflow(workflow, {}, 'test-user')
      const endTime = Date.now()

      expect(execution.status).toBe('completed')
      expect(endTime - startTime).toBeGreaterThanOrEqual(100)
      
      const delayExecution = execution.node_executions.find(ne => ne.node_id === 'delay-1')
      expect(delayExecution).toBeDefined()
      expect(delayExecution!.status).toBe('completed')
      expect(delayExecution!.output_data.delayed).toBe(true)
    })

    it('should execute data transformation node', async () => {
      const workflow = createMockWorkflow({
        nodes: [
          {
            id: 'trigger-1',
            node_type: 'manual-trigger',
            position: { x: 100, y: 100 },
            configuration: { parameters: { name: 'Test Trigger' } },
            metadata: {
              label: 'Start',
              description: 'Manual trigger',
              tags: ['trigger'],
              icon: '▶️',
              color: '#3b82f6'
            }
          },
          {
            id: 'transform-1',
            node_type: 'transform-data',
            position: { x: 400, y: 100 },
            configuration: {
              parameters: {
                operation: 'map',
                config: JSON.stringify({
                  mapping: {
                    'user_id': 'id',
                    'full_name': 'name',
                    'email_address': 'email'
                  }
                })
              }
            },
            metadata: {
              label: 'Transform Data',
              description: 'Map data fields',
              tags: ['transform'],
              icon: '🔄',
              color: '#10b981'
            }
          }
        ],
        connections: [
          {
            id: 'conn-1',
            source_node_id: 'trigger-1',
            target_node_id: 'transform-1',
            source_output: 'trigger',
            target_input: 'data'
          }
        ]
      })

      const triggerData = {
        id: '123',
        name: 'John Doe',
        email: 'john@example.com'
      }

      const execution = await executionEngine.executeWorkflow(workflow, triggerData, 'test-user')

      expect(execution.status).toBe('completed')
      
      const transformExecution = execution.node_executions.find(ne => ne.node_id === 'transform-1')
      expect(transformExecution).toBeDefined()
      expect(transformExecution!.status).toBe('completed')
      expect(transformExecution!.output_data.transformedData).toEqual({
        user_id: '123',
        full_name: 'John Doe',
        email_address: 'john@example.com'
      })
    })
  })

  describe('Execution Management', () => {
    it('should track execution history', async () => {
      const workflow = createMockWorkflow()
      
      await executionEngine.executeWorkflow(workflow, {}, 'test-user')
      await executionEngine.executeWorkflow(workflow, {}, 'test-user')

      const executions = executionEngine.getAllExecutions()
      expect(executions).toHaveLength(2)
      expect(executions[0].workflow_id).toBe(workflow.id)
      expect(executions[1].workflow_id).toBe(workflow.id)
    })

    it('should retrieve specific execution by ID', async () => {
      const workflow = createMockWorkflow()
      const execution = await executionEngine.executeWorkflow(workflow, {}, 'test-user')

      const retrieved = executionEngine.getExecution(execution.id)
      expect(retrieved).toEqual(execution)
    })

    it('should retrieve execution logs', async () => {
      const workflow = createMockWorkflow()
      const execution = await executionEngine.executeWorkflow(workflow, {}, 'test-user')

      const logs = executionEngine.getExecutionLogs(execution.id)
      expect(logs).toBeDefined()
      expect(logs.length).toBeGreaterThan(0)
      expect(logs[0]).toMatchObject({
        level: expect.any(String),
        message: expect.any(String),
        timestamp: expect.any(String)
      })
    })

    it('should clear execution history', async () => {
      const workflow = createMockWorkflow()
      await executionEngine.executeWorkflow(workflow, {}, 'test-user')

      executionEngine.clearExecutionHistory()
      
      const executions = executionEngine.getAllExecutions()
      expect(executions).toHaveLength(0)
    })
  })

  describe('Error Handling', () => {
    it('should handle node execution errors gracefully', async () => {
      // This test would need a way to force a node to fail
      // For now, we test with an invalid node type which should fail
      const workflow = createMockWorkflow({
        nodes: [
          {
            id: 'trigger-1',
            node_type: 'manual-trigger',
            position: { x: 100, y: 100 },
            configuration: { parameters: { name: 'Test Trigger' } },
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
            node_type: 'non-existent-type',
            position: { x: 400, y: 100 },
            configuration: { parameters: {} },
            metadata: {
              label: 'Invalid',
              description: 'Invalid node',
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

      const execution = await executionEngine.executeWorkflow(workflow, {}, 'test-user')

      expect(execution.status).toBe('failed')
      expect(execution.error_message).toContain('No executor found for node type')
      
      // Trigger should still execute successfully
      const triggerExecution = execution.node_executions.find(ne => ne.node_id === 'trigger-1')
      expect(triggerExecution?.status).toBe('completed')
      
      // Invalid node should fail
      const invalidExecution = execution.node_executions.find(ne => ne.node_id === 'invalid-1')
      expect(invalidExecution?.status).toBe('failed')
      expect(invalidExecution?.error_message).toContain('No executor found for node type')
    })

    it('should handle execution timeout gracefully', async () => {
      // This would test timeout functionality if implemented
      // For now, we just verify that executions complete within reasonable time
      const workflow = createMockWorkflow()
      
      const startTime = Date.now()
      const execution = await executionEngine.executeWorkflow(workflow, {}, 'test-user')
      const endTime = Date.now()
      
      expect(endTime - startTime).toBeLessThan(5000) // Should complete within 5 seconds
      expect(execution.status).toBe('completed')
    })
  })
})