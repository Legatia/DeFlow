// Custom Strategy Service - API calls for workflow-based strategy creation
import { StrategyConfig } from '../types/defi-strategy'
import { Node, Edge } from 'reactflow'
import strategyExecutionService from './strategyExecutionService'
import multiChainWalletService, { ChainType } from './multiChainWalletService'

export interface WorkflowDefinition {
  name: string
  description: string
  risk_level: number
  max_allocation_usd: number
  nodes: WorkflowNode[]
  edges: WorkflowEdge[]
}

export interface WorkflowNode {
  id: string
  node_type: string
  config: Record<string, string>
  position: { x: number; y: number }
}

export interface WorkflowEdge {
  id: string
  source: string
  target: string
  source_handle?: string
  target_handle?: string
}

export interface StrategyCreationResponse {
  strategy_id: string
  status: string
  message: string
  deployment_status?: string
}

class CustomStrategyService {
  private backendUrl = process.env.REACT_APP_BACKEND_URL || 'http://localhost:8000'

  /**
   * Create a custom strategy from workflow definition
   */
  async createStrategyFromWorkflow(
    nodes: Node[], 
    edges: Edge[], 
    metadata: {
      name: string
      description: string
      riskLevel: number
      maxAllocation: number
    }
  ): Promise<StrategyCreationResponse> {
    
    // Convert ReactFlow nodes/edges to backend format
    const workflowDefinition: WorkflowDefinition = {
      name: metadata.name,
      description: metadata.description,
      risk_level: metadata.riskLevel,
      max_allocation_usd: metadata.maxAllocation,
      nodes: nodes.map(node => ({
        id: node.id,
        node_type: node.data?.nodeType?.id || node.type || 'unknown',
        config: this.convertConfigToStrings(node.data?.config || {}),
        position: node.position
      })),
      edges: edges.map(edge => ({
        id: edge.id,
        source: edge.source,
        target: edge.target,
        source_handle: edge.sourceHandle || undefined,
        target_handle: edge.targetHandle || undefined
      }))
    }

    try {
      // For now, simulate the API call with local compilation
      // In production, this would call the backend canister
      const response = await this.simulateBackendCall(workflowDefinition)
      
      console.log('Strategy creation response:', response)
      return response
    } catch (error) {
      console.error('Failed to create strategy from workflow:', error)
      throw new Error(`Strategy creation failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
    }
  }

  /**
   * Validate workflow before creation
   */
  async validateWorkflow(nodes: Node[], edges: Edge[]): Promise<{ valid: boolean; errors: string[] }> {
    const errors: string[] = []

    // Check for required trigger node
    const triggerNodes = nodes.filter(node => 
      node.data?.nodeType?.category === 'triggers' || 
      node.data?.nodeType?.id?.includes('trigger')
    )
    
    if (triggerNodes.length === 0) {
      errors.push('Workflow must contain at least one trigger node (schedule, price trigger, etc.)')
    }

    // Check for DeFi action nodes
    const actionNodes = nodes.filter(node => 
      ['yield-farming', 'arbitrage', 'dca-strategy', 'rebalance'].includes(node.data?.nodeType?.id)
    )
    
    if (actionNodes.length === 0) {
      errors.push('Workflow must contain at least one DeFi action (yield farming, arbitrage, DCA, or rebalancing)')
    }

    // Validate node configurations
    for (const node of nodes) {
      const nodeErrors = this.validateNodeConfig(node)
      errors.push(...nodeErrors)
    }

    // Check workflow connectivity
    const connectivityErrors = this.validateWorkflowConnectivity(nodes, edges)
    errors.push(...connectivityErrors)

    return {
      valid: errors.length === 0,
      errors
    }
  }

  private validateNodeConfig(node: Node): string[] {
    const errors: string[] = []
    const nodeType = node.data?.nodeType?.id
    const config = node.data?.config || {}

    switch (nodeType) {
      case 'yield-farming':
        if (!config.protocol) {
          errors.push(`Yield farming node "${node.id}" missing protocol configuration`)
        }
        if (!config.token) {
          errors.push(`Yield farming node "${node.id}" missing token configuration`)
        }
        break
      
      case 'arbitrage':
        if (!config.asset) {
          errors.push(`Arbitrage node "${node.id}" missing asset configuration`)
        }
        if (!config.min_profit_percent) {
          errors.push(`Arbitrage node "${node.id}" missing minimum profit configuration`)
        }
        break
      
      case 'dca-strategy':
        if (!config.target_token) {
          errors.push(`DCA node "${node.id}" missing target token configuration`)
        }
        if (!config.amount_per_execution) {
          errors.push(`DCA node "${node.id}" missing amount per execution configuration`)
        }
        break
    }

    return errors
  }

  private validateWorkflowConnectivity(nodes: Node[], edges: Edge[]): string[] {
    const errors: string[] = []

    // Check if all nodes are connected
    const nodeIds = new Set(nodes.map(n => n.id))
    const connectedNodes = new Set<string>()
    
    edges.forEach(edge => {
      connectedNodes.add(edge.source)
      connectedNodes.add(edge.target)
    })

    const isolatedNodes = Array.from(nodeIds).filter(id => !connectedNodes.has(id))
    
    if (isolatedNodes.length > 0 && nodes.length > 1) {
      errors.push(`Isolated nodes found: ${isolatedNodes.join(', ')}. All nodes should be connected.`)
    }

    return errors
  }

  private convertConfigToStrings(config: Record<string, any>): Record<string, string> {
    const stringConfig: Record<string, string> = {}
    
    for (const [key, value] of Object.entries(config)) {
      if (value !== null && value !== undefined) {
        stringConfig[key] = String(value)
      }
    }
    
    return stringConfig
  }

  /**
   * Simulate backend API call for development
   * In production, this would be replaced with actual canister call
   */
  private async simulateBackendCall(workflowDefinition: WorkflowDefinition): Promise<StrategyCreationResponse> {
    // Simulate network delay
    await new Promise(resolve => setTimeout(resolve, 2000))
    
    // Simulate validation and compilation
    const validation = await this.validateWorkflow(
      workflowDefinition.nodes.map(n => ({
        id: n.id,
        type: 'workflowNode',
        position: n.position,
        data: {
          nodeType: { id: n.node_type },
          config: n.config
        }
      })) as Node[],
      workflowDefinition.edges.map(e => ({
        id: e.id,
        source: e.source,
        target: e.target,
        sourceHandle: e.source_handle,
        targetHandle: e.target_handle
      })) as Edge[]
    )

    if (!validation.valid) {
      throw new Error(`Workflow validation failed: ${validation.errors.join(', ')}`)
    }

    return {
      strategy_id: `custom-strategy-${Date.now()}`,
      status: 'created',
      message: 'Custom strategy created successfully from workflow',
      deployment_status: 'ready'
    }
  }

  /**
   * Activate strategy with wallet validation and authorization
   */
  async activateStrategyWithWallets(
    strategyId: string,
    capitalAmount: number,
    requiredChains: ChainType[]
  ): Promise<{ success: boolean; message: string }> {
    try {
      // Step 1: Validate wallet addresses
      console.log('Validating wallet addresses for chains:', requiredChains)
      const validationResult = await strategyExecutionService.validateWalletAddresses(requiredChains)
      
      if (!validationResult.all_valid) {
        const invalidChains = Object.entries(validationResult.validation_results)
          .filter(([_, result]) => !result.is_valid)
          .map(([chain]) => chain)
        
        return {
          success: false,
          message: `Wallet validation failed for chains: ${invalidChains.join(', ')}`
        }
      }

      // Step 2: Create execution authorization
      console.log('Creating execution authorization...')
      const authChallenge = await strategyExecutionService.createExecutionAuthorization(
        strategyId,
        capitalAmount
      )

      // Step 3: Request signature from user if required
      let authorizationId = authChallenge.authorization_id
      if (authChallenge.signature_required) {
        console.log('Requesting signature from user...')
        const primaryChain = requiredChains[0] // Use first chain as primary
        
        await strategyExecutionService.signAuthorizationChallenge(
          authChallenge.authorization_id,
          authChallenge.challenge_message,
          primaryChain
        )
      }

      // Step 4: Activate strategy with validated wallets
      console.log('Activating strategy...')
      const success = await strategyExecutionService.activateStrategyWithWallets(
        strategyId,
        capitalAmount,
        requiredChains,
        authorizationId
      )

      if (success) {
        return {
          success: true,
          message: 'Strategy activated successfully with multi-chain wallet integration'
        }
      } else {
        return {
          success: false,
          message: 'Strategy activation failed'
        }
      }
    } catch (error) {
      console.error('Strategy activation with wallets failed:', error)
      return {
        success: false,
        message: `Activation failed: ${error instanceof Error ? error.message : 'Unknown error'}`
      }
    }
  }

  /**
   * Execute strategy with wallet addresses
   */
  async executeStrategy(strategyId: string): Promise<{ success: boolean; result?: any; error?: string }> {
    try {
      console.log('Executing strategy:', strategyId)
      const result = await strategyExecutionService.executeStrategy(strategyId)
      
      return {
        success: result.success,
        result: result
      }
    } catch (error) {
      console.error('Strategy execution failed:', error)
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error'
      }
    }
  }

  /**
   * Check wallet compatibility with strategy requirements
   */
  async checkWalletCompatibility(requiredChains: ChainType[]): Promise<{
    compatible: boolean
    availableChains: ChainType[]
    missingChains: ChainType[]
    connectedChains: ChainType[]
  }> {
    const wallet = multiChainWalletService.getWallet()
    
    const availableChains = wallet.addresses.map(addr => addr.chain)
    const connectedChains = wallet.addresses
      .filter(addr => addr.isConnected)
      .map(addr => addr.chain)
    
    const missingChains = requiredChains.filter(chain => !availableChains.includes(chain))
    const compatible = missingChains.length === 0

    return {
      compatible,
      availableChains,
      missingChains,
      connectedChains
    }
  }

  /**
   * Extract required chains from workflow definition
   */
  extractRequiredChainsFromWorkflow(workflowDefinition: WorkflowDefinition): ChainType[] {
    const chains = new Set<ChainType>()

    // Analyze nodes for chain requirements
    workflowDefinition.nodes.forEach(node => {
      if (node.node_type.includes('yield-farming') || 
          node.node_type.includes('arbitrage') ||
          node.node_type.includes('dca-strategy')) {
        
        // Extract chain from config
        const chain = node.config.chain || node.config.target_chain
        if (chain && this.isValidChainType(chain)) {
          chains.add(chain as ChainType)
        }
      }
    })

    // If no specific chains found, default to Ethereum for DeFi strategies
    if (chains.size === 0) {
      const hasDeFiNodes = workflowDefinition.nodes.some(node =>
        ['yield-farming', 'arbitrage', 'dca-strategy', 'rebalance'].some(type =>
          node.node_type.includes(type)
        )
      )
      
      if (hasDeFiNodes) {
        chains.add('Ethereum')
      }
    }

    return Array.from(chains)
  }

  private isValidChainType(chain: string): boolean {
    const validChains: ChainType[] = [
      'Bitcoin', 'Ethereum', 'Arbitrum', 'Optimism', 
      'Polygon', 'Base', 'Avalanche', 'Solana', 'BSC'
    ]
    return validChains.includes(chain as ChainType)
  }

  /**
   * Get available node types for the workflow builder
   */
  getAvailableNodeTypes() {
    return [
      // Triggers
      'schedule-trigger',
      'price-trigger',
      'manual-trigger',
      
      // DeFi Actions  
      'yield-farming',
      'arbitrage',
      'dca-strategy',
      'rebalance',
      
      // Conditions
      'yield-condition',
      'price-condition',
      
      // Utilities
      'price-check',
      'gas-optimizer',
      'delay'
    ]
  }
}

export default new CustomStrategyService()