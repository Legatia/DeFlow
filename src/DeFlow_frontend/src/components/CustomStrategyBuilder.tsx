import React, { useState, useCallback } from 'react'
import { Node, Edge } from 'reactflow'
import WorkflowBuilder from './WorkflowBuilder'
import WalletIntegration from './WalletIntegration'
import { StrategyConfig } from '../types/defi-strategy'
import customStrategyService from '../services/customStrategyService'
import { ChainType } from '../services/multiChainWalletService'

interface CustomStrategyBuilderProps {
  onStrategyCreated: (strategy: StrategyConfig) => void
  onCancel: () => void
}

const CustomStrategyBuilder = ({ onStrategyCreated, onCancel }: CustomStrategyBuilderProps) => {
  const [strategyName, setStrategyName] = useState('')
  const [strategyDescription, setStrategyDescription] = useState('')
  const [riskLevel, setRiskLevel] = useState(5)
  const [maxAllocation, setMaxAllocation] = useState(1000)
  const [isCompiling, setIsCompiling] = useState(false)
  const [selectedChains, setSelectedChains] = useState<ChainType[]>([])
  const [showWalletPanel, setShowWalletPanel] = useState(false)

  const handleWorkflowSave = useCallback(async (nodes: Node[], edges: Edge[]) => {
    if (!strategyName.trim()) {
      alert('Please enter a strategy name')
      return
    }

    setIsCompiling(true)
    try {
      // Validate workflow first
      const validation = await customStrategyService.validateWorkflow(nodes, edges)
      if (!validation.valid) {
        throw new Error(`Workflow validation failed:\n${validation.errors.join('\n')}`)
      }

      // Create strategy from workflow
      const response = await customStrategyService.createStrategyFromWorkflow(
        nodes, 
        edges, 
        {
          name: strategyName,
          description: strategyDescription,
          riskLevel,
          maxAllocation
        }
      )

      // Convert response to StrategyConfig format for the parent component
      const strategyConfig: StrategyConfig = {
        name: strategyName,
        description: strategyDescription,
        strategy_type: {
          type: 'Composite', // Will be determined by backend
          config: {}
        },
        target_chains: ['Ethereum'], // Default, will be extracted from workflow
        target_protocols: ['Aave'], // Default, will be extracted from workflow  
        risk_level: riskLevel,
        max_allocation_usd: maxAllocation,
        min_return_threshold: 1.0,
        execution_interval_minutes: 1440,
        gas_limit_usd: 50.0,
        auto_compound: true,
        stop_loss_percentage: null,
        take_profit_percentage: null,
        workflow_definition: {
          nodes,
          edges,
          compiled_at: new Date().toISOString()
        }
      }

      onStrategyCreated(strategyConfig)
    } catch (error) {
      console.error('Failed to create strategy:', error)
      alert(`Failed to create strategy: ${error instanceof Error ? error.message : 'Unknown error'}`)
    } finally {
      setIsCompiling(false)
    }
  }, [strategyName, strategyDescription, riskLevel, maxAllocation, onStrategyCreated])

  return (
    <div className="h-screen flex flex-col bg-gray-50">
      {/* Header */}
      <div className="bg-white border-b border-gray-200 px-6 py-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-gray-900">Custom Strategy Builder</h1>
            <p className="text-gray-600 mt-1">Create your own DeFi strategy using visual workflow builder</p>
          </div>
          <div className="flex items-center space-x-3">
            <div className="flex items-center space-x-3">
              <button
                onClick={() => setShowWalletPanel(!showWalletPanel)}
                className={`px-4 py-2 border rounded-lg text-sm transition-colors ${
                  showWalletPanel 
                    ? 'border-blue-500 text-blue-700 bg-blue-50' 
                    : 'border-gray-300 text-gray-700 hover:bg-gray-50'
                }`}
                disabled={isCompiling}
              >
                🔗 {showWalletPanel ? 'Hide' : 'Show'} Wallets
              </button>
              <button
                onClick={onCancel}
                className="px-4 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-50 transition-colors"
                disabled={isCompiling}
              >
                Cancel
              </button>
            </div>
          </div>
        </div>

        {/* Strategy Configuration */}
        <div className="mt-4 grid grid-cols-1 md:grid-cols-4 gap-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Strategy Name *
            </label>
            <input
              type="text"
              value={strategyName}
              onChange={(e) => setStrategyName(e.target.value)}
              placeholder="My Custom Strategy"
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              disabled={isCompiling}
            />
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Description
            </label>
            <input
              type="text"
              value={strategyDescription}
              onChange={(e) => setStrategyDescription(e.target.value)}
              placeholder="Strategy description"
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              disabled={isCompiling}
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Risk Level (1-10)
            </label>
            <input
              type="range"
              min="1"
              max="10"
              value={riskLevel}
              onChange={(e) => setRiskLevel(parseInt(e.target.value))}
              className="w-full"
              disabled={isCompiling}
            />
            <div className="text-sm text-gray-600 text-center">{riskLevel}/10</div>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Max Allocation (USD)
            </label>
            <input
              type="number"
              value={maxAllocation}
              onChange={(e) => setMaxAllocation(parseInt(e.target.value) || 0)}
              min="100"
              step="100"
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              disabled={isCompiling}
            />
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 flex">
        {/* Wallet Panel */}
        {showWalletPanel && (
          <div className="w-80 bg-white border-r border-gray-200 p-4 overflow-y-auto">
            <WalletIntegration 
              selectedChains={selectedChains}
              onWalletChange={(wallet) => {
                // Extract chains from connected wallets
                const chains = wallet.addresses.map(addr => addr.chain as ChainType)
                setSelectedChains(chains)
              }}
            />
          </div>
        )}

        {/* Workflow Builder */}
        <div className="flex-1">
          <WorkflowBuilder
            initialNodes={[]}
            initialEdges={[]}
            onSave={handleWorkflowSave}
            readOnly={isCompiling}
          />
        </div>
      </div>

      {/* Compilation Status */}
      {isCompiling && (
        <div className="absolute inset-0 bg-black bg-opacity-50 flex items-center justify-center">
          <div className="bg-white rounded-lg p-6 max-w-md w-full mx-4">
            <div className="flex items-center space-x-3">
              <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-500"></div>
              <div>
                <h3 className="text-lg font-medium text-gray-900">Compiling Strategy</h3>
                <p className="text-gray-600">Converting your workflow to executable strategy...</p>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}


export default CustomStrategyBuilder