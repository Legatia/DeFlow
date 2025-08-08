// Bitcoin DeFi Workflow Integration Tests
// End-to-end testing of Bitcoin workflow execution

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { 
  createMockWorkflow, 
  createMockWorkflowExecution, 
  createMockBitcoinPortfolio,
  createMockBitcoinAddress,
  createMockUser 
} from '../utils/testUtils'
import { BigIntUtils } from '../../utils/bigint-utils'

// Mock services
const mockIcpService = {
  call: vi.fn(),
  query: vi.fn(),
}

const mockExecutionEngine = {
  startExecution: vi.fn(),
  getExecution: vi.fn(),
  stopExecution: vi.fn(),
}

vi.mock('../../services/icpService', () => ({
  default: mockIcpService,
}))

vi.mock('../../services/executionEngine', () => ({
  default: mockExecutionEngine,
}))

// Workflow execution types
interface WorkflowExecution {
  id: string
  workflow_id: string
  status: 'Pending' | 'Running' | 'Completed' | 'Failed' | 'Cancelled'
  started_at: bigint
  completed_at?: bigint
  node_executions: NodeExecution[]
  error_message?: string
}

interface NodeExecution {
  node_id: string
  status: 'Pending' | 'Running' | 'Completed' | 'Failed' | 'Cancelled'
  started_at?: bigint
  completed_at?: bigint
  input_data?: Record<string, any>
  output_data?: Record<string, any>
  error_message?: string
}

// Bitcoin Workflow Integration Service
class BitcoinWorkflowService {
  async executePortfolioWorkflow(workflowId: string): Promise<WorkflowExecution> {
    const execution = await mockExecutionEngine.startExecution(workflowId, {})
    return execution
  }

  async executeSendWorkflow(
    workflowId: string, 
    toAddress: string, 
    amountSatoshis: number
  ): Promise<WorkflowExecution> {
    const execution = await mockExecutionEngine.startExecution(workflowId, {
      to_address: toAddress,
      amount_satoshis: amountSatoshis
    })
    return execution
  }

  async executeAddressGenerationWorkflow(
    workflowId: string, 
    addressType: 'P2PKH' | 'P2WPKH' | 'P2TR'
  ): Promise<WorkflowExecution> {
    const execution = await mockExecutionEngine.startExecution(workflowId, {
      address_type: addressType
    })
    return execution
  }

  async monitorExecution(executionId: string): Promise<WorkflowExecution> {
    return await mockExecutionEngine.getExecution(executionId)
  }

  async waitForCompletion(executionId: string, timeoutMs: number = 30000): Promise<WorkflowExecution> {
    const startTime = Date.now()
    
    while (Date.now() - startTime < timeoutMs) {
      const execution = await this.monitorExecution(executionId)
      
      if (execution.status === 'Completed' || execution.status === 'Failed') {
        return execution
      }
      
      // Wait 100ms before checking again
      await new Promise(resolve => setTimeout(resolve, 100))
    }
    
    throw new Error('Workflow execution timed out')
  }
}

describe('Bitcoin DeFi Workflow Integration', () => {
  let workflowService: BitcoinWorkflowService

  beforeEach(() => {
    workflowService = new BitcoinWorkflowService()
    vi.clearAllMocks()
    mockExecutionEngine.startExecution.mockClear()
    mockExecutionEngine.getExecution.mockClear()
    mockExecutionEngine.stopExecution.mockClear()
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  describe('Bitcoin Portfolio Workflow', () => {
    it('should execute portfolio fetch workflow successfully', async () => {
      // Arrange
      const mockWorkflow = createMockWorkflow({
        name: 'Bitcoin Portfolio Fetcher',
        nodes: [
          {
            id: 'portfolio-1',
            node_type: 'bitcoin_portfolio',
            position: { x: 100, y: 100 },
            configuration: { parameters: {} },
            metadata: { label: 'Get Portfolio', description: '', version: '1.0.0' }
          }
        ]
      })

      const mockPortfolio = createMockBitcoinPortfolio({
        total_btc: 1.5,
        total_value_usd: 67500,
        addresses: [
          createMockBitcoinAddress({ address_type: 'P2PKH', balance_satoshis: 50000000 }),
          createMockBitcoinAddress({ address_type: 'P2WPKH', balance_satoshis: 75000000 }),
          createMockBitcoinAddress({ address_type: 'P2TR', balance_satoshis: 25000000 })
        ]
      })

      const mockExecution = createMockWorkflowExecution({
        workflow_id: mockWorkflow.id,
        status: 'Completed',
        node_executions: [
          {
            node_id: 'portfolio-1',
            status: 'Completed',
            started_at: BigIntUtils.dateToTimestamp(new Date(Date.now() - 1000)),
            completed_at: BigIntUtils.dateToTimestamp(),
            output_data: {
              total_btc: mockPortfolio.total_btc,
              total_value_usd: mockPortfolio.total_value_usd,
              addresses: mockPortfolio.addresses
            }
          }
        ]
      })

      mockExecutionEngine.startExecution.mockResolvedValue({
        ...mockExecution,
        node_executions: [{
          node_id: 'portfolio-1',
          status: 'Completed',
          output_data: {
            total_btc: mockPortfolio.total_btc,
            total_value_usd: mockPortfolio.total_value_usd,
            addresses: mockPortfolio.addresses
          }
        }]
      })
      mockExecutionEngine.getExecution.mockResolvedValue({
        ...mockExecution,
        node_executions: [{
          node_id: 'portfolio-1',
          status: 'Completed',
          output_data: {
            total_btc: mockPortfolio.total_btc,
            total_value_usd: mockPortfolio.total_value_usd,
            addresses: mockPortfolio.addresses
          }
        }]
      })

      // Act
      const execution = await workflowService.executePortfolioWorkflow(mockWorkflow.id)
      const result = await workflowService.waitForCompletion(execution.id)

      // Assert
      expect(mockExecutionEngine.startExecution).toHaveBeenCalledWith(mockWorkflow.id, {})
      expect(result.status).toBe('Completed')
      expect(result.node_executions).toHaveLength(1)
      
      const portfolioNode = result.node_executions[0]
      expect(portfolioNode.node_id).toBe('portfolio-1')
      expect(portfolioNode.status).toBe('Completed')
      expect(portfolioNode.output_data?.total_btc).toBe(1.5)
      expect(portfolioNode.output_data?.addresses).toHaveLength(3)
    })

    it('should handle portfolio fetch errors gracefully', async () => {
      // Arrange
      const mockWorkflow = createMockWorkflow({
        name: 'Bitcoin Portfolio Fetcher',
        nodes: [
          {
            id: 'portfolio-1',
            node_type: 'bitcoin_portfolio',
            position: { x: 100, y: 100 },
            configuration: { parameters: {} },
            metadata: { label: 'Get Portfolio', description: '', version: '1.0.0' }
          }
        ]
      })

      const mockExecution = createMockWorkflowExecution({
        workflow_id: mockWorkflow.id,
        status: 'Failed',
        error_message: 'Bitcoin service not initialized',
        node_executions: [
          {
            node_id: 'portfolio-1',
            status: 'Failed',
            error_message: 'Bitcoin service not initialized'
          }
        ]
      })

      mockExecutionEngine.startExecution.mockResolvedValue({
        ...mockExecution,
        status: 'Failed',
        error_message: 'Bitcoin service not initialized',
        node_executions: [{
          node_id: 'portfolio-1',
          status: 'Failed',
          error_message: 'Bitcoin service not initialized'
        }]
      })
      mockExecutionEngine.getExecution.mockResolvedValue({
        ...mockExecution,
        status: 'Failed',
        error_message: 'Bitcoin service not initialized',
        node_executions: [{
          node_id: 'portfolio-1',
          status: 'Failed',
          error_message: 'Bitcoin service not initialized'
        }]
      })

      // Act
      const execution = await workflowService.executePortfolioWorkflow(mockWorkflow.id)
      const result = await workflowService.waitForCompletion(execution.id)

      // Assert
      expect(result.status).toBe('Failed')
      expect(result.error_message).toBe('Bitcoin service not initialized')
      expect(result.node_executions[0].status).toBe('Failed')
    })
  })

  describe('Bitcoin Send Workflow', () => {
    it('should execute send workflow successfully', async () => {
      // Arrange
      const mockWorkflow = createMockWorkflow({
        name: 'Bitcoin Send Transaction',
        nodes: [
          {
            id: 'send-1',
            node_type: 'bitcoin_send',
            position: { x: 100, y: 100 },
            configuration: { 
              parameters: { 
                fee_satoshis: { type: 'number', value: 1000 }
              } 
            },
            metadata: { label: 'Send Bitcoin', description: '', version: '1.0.0' }
          }
        ]
      })

      const mockSendResult = {
        success: true,
        transaction_id: 'abc123def456789',
        from_address: 'bcrt1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh',
        to_address: '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa',
        amount_satoshis: 100000,
        fee_satoshis: 1000,
        change_amount_satoshis: 49899000,
        confirmation_time_estimate_minutes: 30
      }

      const mockExecution = createMockWorkflowExecution({
        workflow_id: mockWorkflow.id,
        status: 'Completed',
        node_executions: [
          {
            node_id: 'send-1',
            status: 'Completed',
            input_data: {
              to_address: '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa',
              amount_satoshis: 100000
            },
            output_data: mockSendResult
          }
        ]
      })

      mockExecutionEngine.startExecution.mockResolvedValue({
        ...mockExecution,
        node_executions: [{
          node_id: 'send-1',
          status: 'Completed',
          input_data: {
            to_address: '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa',
            amount_satoshis: 100000
          },
          output_data: mockSendResult
        }]
      })
      mockExecutionEngine.getExecution.mockResolvedValue({
        ...mockExecution,
        node_executions: [{
          node_id: 'send-1',
          status: 'Completed',
          input_data: {
            to_address: '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa',
            amount_satoshis: 100000
          },
          output_data: mockSendResult
        }]
      })

      // Act
      const execution = await workflowService.executeSendWorkflow(
        mockWorkflow.id,
        '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa',
        100000
      )
      const result = await workflowService.waitForCompletion(execution.id)

      // Assert
      expect(mockExecutionEngine.startExecution).toHaveBeenCalledWith(mockWorkflow.id, {
        to_address: '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa',
        amount_satoshis: 100000
      })
      expect(result.status).toBe('Completed')
      
      const sendNode = result.node_executions[0]
      expect(sendNode.output_data?.success).toBe(true)
      expect(sendNode.output_data?.transaction_id).toBe('abc123def456789')
      expect(sendNode.output_data?.amount_satoshis).toBe(100000)
    })

    it('should handle insufficient funds error', async () => {
      // Arrange
      const mockWorkflow = createMockWorkflow({
        name: 'Bitcoin Send Transaction',
        nodes: [
          {
            id: 'send-1',
            node_type: 'bitcoin_send',
            position: { x: 100, y: 100 },
            configuration: { parameters: {} },
            metadata: { label: 'Send Bitcoin', description: '', version: '1.0.0' }
          }
        ]
      })

      const mockExecution = createMockWorkflowExecution({
        workflow_id: mockWorkflow.id,
        status: 'Failed',
        node_executions: [
          {
            node_id: 'send-1',
            status: 'Failed',
            error_message: 'Insufficient balance: need 101000 satoshis, have 50000 satoshis'
          }
        ]
      })

      mockExecutionEngine.startExecution.mockResolvedValue({
        ...mockExecution,
        status: 'Failed',
        node_executions: [{
          node_id: 'send-1',
          status: 'Failed',
          error_message: 'Insufficient balance: need 101000 satoshis, have 50000 satoshis'
        }]
      })
      mockExecutionEngine.getExecution.mockResolvedValue({
        ...mockExecution,
        status: 'Failed',
        node_executions: [{
          node_id: 'send-1',
          status: 'Failed',
          error_message: 'Insufficient balance: need 101000 satoshis, have 50000 satoshis'
        }]
      })

      // Act
      const execution = await workflowService.executeSendWorkflow(
        mockWorkflow.id,
        '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa',
        100000
      )
      const result = await workflowService.waitForCompletion(execution.id)

      // Assert
      expect(result.status).toBe('Failed')
      expect(result.node_executions[0].error_message).toContain('Insufficient balance')
    })

    it('should handle transaction broadcast failure', async () => {
      // Arrange
      const mockSendResult = {
        success: false,
        from_address: 'bcrt1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh',
        to_address: '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa',
        amount_satoshis: 100000,
        fee_satoshis: 1000,
        change_amount_satoshis: 0,
        confirmation_time_estimate_minutes: 0,
        error_message: 'Transaction rejected by network: invalid transaction'
      }

      const mockExecution = createMockWorkflowExecution({
        status: 'Completed', // Workflow completes even if transaction fails
        node_executions: [
          {
            node_id: 'send-1',
            status: 'Completed',
            output_data: mockSendResult
          }
        ]
      })

      mockExecutionEngine.startExecution.mockResolvedValue(mockExecution)
      mockExecutionEngine.getExecution.mockResolvedValue(mockExecution)

      // Act
      const execution = await workflowService.executeSendWorkflow(
        'workflow-1',
        '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa',
        100000
      )
      const result = await workflowService.waitForCompletion(execution.id)

      // Assert
      expect(result.status).toBe('Completed')
      expect(result.node_executions[0].output_data?.success).toBe(false)
      expect(result.node_executions[0].output_data?.error_message).toContain('Transaction rejected')
    })
  })

  describe('Bitcoin Address Generation Workflow', () => {
    it('should execute address generation workflow for all types', async () => {
      // Test all address types
      const addressTypes: Array<'P2PKH' | 'P2WPKH' | 'P2TR'> = ['P2PKH', 'P2WPKH', 'P2TR']
      
      for (const addressType of addressTypes) {
        // Arrange
        const mockWorkflow = createMockWorkflow({
          name: `Generate ${addressType} Address`,
          nodes: [
            {
              id: 'address-1',
              node_type: 'bitcoin_address',
              position: { x: 100, y: 100 },
              configuration: { 
                parameters: { 
                  address_type: { type: 'string', value: addressType }
                } 
              },
              metadata: { label: 'Generate Address', description: '', version: '1.0.0' }
            }
          ]
        })

        const mockAddress = createMockBitcoinAddress({ address_type: addressType })

        const mockExecution = createMockWorkflowExecution({
          workflow_id: mockWorkflow.id,
          status: 'Completed',
          node_executions: [
            {
              node_id: 'address-1',
              status: 'Completed',
              output_data: {
                address: mockAddress.address,
                address_type: mockAddress.address_type,
                balance_satoshis: mockAddress.balance_satoshis
              }
            }
          ]
        })

        mockExecutionEngine.startExecution.mockResolvedValue({
          ...mockExecution,
          node_executions: [{
            node_id: 'address-1',
            status: 'Completed',
            output_data: {
              address: mockAddress.address,
              address_type: mockAddress.address_type,
              balance_satoshis: mockAddress.balance_satoshis
            }
          }]
        })
        mockExecutionEngine.getExecution.mockResolvedValue({
          ...mockExecution,
          node_executions: [{
            node_id: 'address-1',
            status: 'Completed',
            output_data: {
              address: mockAddress.address,
              address_type: mockAddress.address_type,
              balance_satoshis: mockAddress.balance_satoshis
            }
          }]
        })

        // Act
        const execution = await workflowService.executeAddressGenerationWorkflow(
          mockWorkflow.id,
          addressType
        )
        const result = await workflowService.waitForCompletion(execution.id, 1000)

        // Assert
        expect(result.status).toBe('Completed')
        expect(result.node_executions[0].output_data?.address_type).toBe(addressType)
        expect(result.node_executions[0].output_data?.address).toMatch(
          addressType === 'P2PKH' ? /^1/ :
          addressType === 'P2WPKH' ? /^bcrt1q/ :
          /^bcrt1p/
        )

        vi.clearAllMocks()
      }
    })
  })

  describe('Complex Bitcoin Workflows', () => {
    it('should execute multi-node Bitcoin workflow', async () => {
      // Arrange - Portfolio check → Address generation → Send transaction
      const mockWorkflow = createMockWorkflow({
        name: 'Complete Bitcoin Workflow',
        nodes: [
          {
            id: 'portfolio-1',
            node_type: 'bitcoin_portfolio',
            position: { x: 100, y: 100 },
            configuration: { parameters: {} },
            metadata: { label: 'Check Portfolio', description: '', version: '1.0.0' }
          },
          {
            id: 'address-1',
            node_type: 'bitcoin_address',
            position: { x: 300, y: 100 },
            configuration: { 
              parameters: { 
                address_type: { type: 'string', value: 'P2WPKH' }
              } 
            },
            metadata: { label: 'Generate Address', description: '', version: '1.0.0' }
          },
          {
            id: 'send-1',
            node_type: 'bitcoin_send',
            position: { x: 500, y: 100 },
            configuration: { parameters: {} },
            metadata: { label: 'Send Transaction', description: '', version: '1.0.0' }
          }
        ],
        connections: [
          {
            id: 'conn-1',
            source_node_id: 'portfolio-1',
            source_output: 'output',
            target_node_id: 'address-1',
            target_input: 'input'
          },
          {
            id: 'conn-2',
            source_node_id: 'address-1',
            source_output: 'output',
            target_node_id: 'send-1',
            target_input: 'input'
          }
        ]
      })

      const mockExecution = createMockWorkflowExecution({
        workflow_id: mockWorkflow.id,
        status: 'Completed',
        node_executions: [
          {
            node_id: 'portfolio-1',
            status: 'Completed',
            output_data: { total_btc: 1.0, total_value_usd: 45000 }
          },
          {
            node_id: 'address-1',
            status: 'Completed',
            output_data: { 
              address: 'bcrt1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh',
              address_type: 'P2WPKH'
            }
          },
          {
            node_id: 'send-1',
            status: 'Completed',
            output_data: { 
              success: true,
              transaction_id: 'complex_workflow_tx',
              amount_satoshis: 50000
            }
          }
        ]
      })

      mockExecutionEngine.startExecution.mockResolvedValue({
        ...mockExecution,
        node_executions: [
          {
            node_id: 'portfolio-1',
            status: 'Completed',
            output_data: { total_btc: 1.0, total_value_usd: 45000 }
          },
          {
            node_id: 'address-1',
            status: 'Completed',
            output_data: { 
              address: 'bcrt1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh',
              address_type: 'P2WPKH'
            }
          },
          {
            node_id: 'send-1',
            status: 'Completed',
            output_data: { 
              success: true,
              transaction_id: 'complex_workflow_tx',
              amount_satoshis: 50000
            }
          }
        ]
      })
      mockExecutionEngine.getExecution.mockResolvedValue({
        ...mockExecution,
        node_executions: [
          {
            node_id: 'portfolio-1',
            status: 'Completed',
            output_data: { total_btc: 1.0, total_value_usd: 45000 }
          },
          {
            node_id: 'address-1',
            status: 'Completed',
            output_data: { 
              address: 'bcrt1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh',
              address_type: 'P2WPKH'
            }
          },
          {
            node_id: 'send-1',
            status: 'Completed',
            output_data: { 
              success: true,
              transaction_id: 'complex_workflow_tx',
              amount_satoshis: 50000
            }
          }
        ]
      })

      // Act
      const execution = await workflowService.executePortfolioWorkflow(mockWorkflow.id)
      const result = await workflowService.waitForCompletion(execution.id, 1000)

      // Assert
      expect(result.status).toBe('Completed')
      expect(result.node_executions).toHaveLength(3)
      
      // Verify each node executed successfully
      const nodeStatuses = result.node_executions.map(node => node.status)
      expect(nodeStatuses).toEqual(['Completed', 'Completed', 'Completed'])
      
      // Verify data flow between nodes
      expect(result.node_executions[0].output_data?.total_btc).toBe(1.0)
      expect(result.node_executions[1].output_data?.address_type).toBe('P2WPKH')
      expect(result.node_executions[2].output_data?.success).toBe(true)
    })

    it('should handle partial workflow failures', async () => {
      // Arrange - Workflow where second node fails
      const mockExecution = createMockWorkflowExecution({
        status: 'Failed',
        node_executions: [
          {
            node_id: 'portfolio-1',
            status: 'Completed',
            output_data: { total_btc: 0 } // Empty portfolio
          },
          {
            node_id: 'send-1',
            status: 'Failed',
            error_message: 'Cannot send from empty portfolio'
          }
        ]
      })

      mockExecutionEngine.startExecution.mockResolvedValue({
        ...mockExecution,
        status: 'Failed',
        node_executions: [
          {
            node_id: 'portfolio-1',
            status: 'Completed',
            output_data: { total_btc: 0 }
          },
          {
            node_id: 'send-1',
            status: 'Failed',
            error_message: 'Cannot send from empty portfolio'
          }
        ]
      })
      mockExecutionEngine.getExecution.mockResolvedValue({
        ...mockExecution,
        status: 'Failed',
        node_executions: [
          {
            node_id: 'portfolio-1',
            status: 'Completed',
            output_data: { total_btc: 0 }
          },
          {
            node_id: 'send-1',
            status: 'Failed',
            error_message: 'Cannot send from empty portfolio'
          }
        ]
      })

      // Act
      const execution = await workflowService.executePortfolioWorkflow('workflow-1')
      const result = await workflowService.waitForCompletion(execution.id, 1000)

      // Assert
      expect(result.status).toBe('Failed')
      expect(result.node_executions[0].status).toBe('Completed')
      expect(result.node_executions[1].status).toBe('Failed')
      expect(result.node_executions[1].error_message).toContain('Cannot send from empty portfolio')
    })
  })

  describe('Workflow Monitoring and Control', () => {
    it('should monitor workflow execution progress', async () => {
      // Arrange - Simulated long-running workflow
      const mockExecution = createMockWorkflowExecution({
        status: 'Running',
        node_executions: [
          {
            node_id: 'portfolio-1',
            status: 'Running'
          }
        ]
      })

      const completedExecution = createMockWorkflowExecution({
        status: 'Completed',
        node_executions: [
          {
            node_id: 'portfolio-1',
            status: 'Completed',
            output_data: { total_btc: 0.5 }
          }
        ]
      })

      mockExecutionEngine.startExecution.mockResolvedValue(mockExecution)
      mockExecutionEngine.getExecution
        .mockResolvedValueOnce(mockExecution)
        .mockResolvedValueOnce(mockExecution)
        .mockResolvedValue(completedExecution)

      // Act
      const execution = await workflowService.executePortfolioWorkflow('workflow-1')
      
      // Monitor progress
      let currentStatus = await workflowService.monitorExecution(execution.id)
      expect(currentStatus.status).toBe('Running')
      
      // Wait for completion
      const result = await workflowService.waitForCompletion(execution.id, 1000)

      // Assert
      expect(result.status).toBe('Completed')
      expect(mockExecutionEngine.getExecution).toHaveBeenCalledTimes(3)
    })

    it('should handle workflow execution timeout', async () => {
      // Arrange - Workflow that never completes
      const runningExecution = createMockWorkflowExecution({
        status: 'Running'
      })

      mockExecutionEngine.startExecution.mockResolvedValue(runningExecution)
      mockExecutionEngine.getExecution.mockResolvedValue(runningExecution)

      // Act & Assert
      const execution = await workflowService.executePortfolioWorkflow('workflow-1')
      
      await expect(
        workflowService.waitForCompletion(execution.id, 1000) // 1 second timeout
      ).rejects.toThrow('Workflow execution timed out')
    })
  })

  describe('Error Recovery and Resilience', () => {
    it('should retry failed Bitcoin operations', async () => {
      // Arrange - First attempt fails, second succeeds
      const failedExecution = createMockWorkflowExecution({
        status: 'Failed',
        error_message: 'Temporary network error'
      })

      const successExecution = createMockWorkflowExecution({
        status: 'Completed',
        node_executions: [
          {
            node_id: 'portfolio-1',
            status: 'Completed',
            output_data: { total_btc: 1.0 }
          }
        ]
      })

      mockExecutionEngine.startExecution
        .mockResolvedValueOnce(failedExecution)
        .mockResolvedValueOnce(successExecution)
      
      mockExecutionEngine.getExecution
        .mockResolvedValueOnce(failedExecution)
        .mockResolvedValueOnce(successExecution)

      // Act - Simulate retry logic
      let execution = await workflowService.executePortfolioWorkflow('workflow-1')
      let result = await workflowService.waitForCompletion(execution.id)
      
      if (result.status === 'Failed') {
        // Retry the workflow
        execution = await workflowService.executePortfolioWorkflow('workflow-1')
        result = await workflowService.waitForCompletion(execution.id)
      }

      // Assert
      expect(result.status).toBe('Completed')
      expect(mockExecutionEngine.startExecution).toHaveBeenCalledTimes(2)
    })

    it('should handle network disconnection gracefully', async () => {
      // Arrange - Network error during execution
      mockExecutionEngine.startExecution.mockRejectedValue(new Error('Network disconnected'))

      // Act & Assert
      await expect(
        workflowService.executePortfolioWorkflow('workflow-1')
      ).rejects.toThrow('Network disconnected')
    })
  })
})