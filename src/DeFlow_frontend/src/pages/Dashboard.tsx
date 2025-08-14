import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { useWorkflowStore } from '../stores/workflowStore'
import { TimestampUtils } from '../utils/timestamp-utils'
import DeFiTemplates from '../components/DeFiTemplates'
import StrategyCreationFlow from '../components/StrategyCreationFlow'
import CustomStrategyBuilder from '../components/CustomStrategyBuilder'
import simpleDefiTemplateService, { 
  DeFiWorkflowTemplate, 
  StrategyFromTemplateResponse 
} from '../services/defiTemplateServiceSimple'
import { StrategyConfig } from '../types/defi-strategy'

interface ActiveStrategy extends StrategyFromTemplateResponse {
  template: DeFiWorkflowTemplate;
  created_at: string;
  status: 'active' | 'paused' | 'completed';
  current_value: number;
  total_return: number;
  roi_percentage: number;
}

type ViewMode = 'overview' | 'defi-templates' | 'create-strategy' | 'custom-strategy'

const Dashboard = () => {
  const { 
    workflows, 
    executions, 
    isLoading, 
    error, 
    loadWorkflows, 
    loadExecutions 
  } = useWorkflowStore()
  
  const [viewMode, setViewMode] = useState<ViewMode>('overview')
  const [selectedTemplate, setSelectedTemplate] = useState<DeFiWorkflowTemplate | null>(null)
  const [activeStrategies, setActiveStrategies] = useState<ActiveStrategy[]>([])
  const [portfolioValue, setPortfolioValue] = useState(0)
  const [totalReturn, setTotalReturn] = useState(0)

  useEffect(() => {
    loadWorkflows()
    loadExecutions()
    loadPortfolioData()
  }, [loadWorkflows, loadExecutions])
  
  const loadPortfolioData = async () => {
    try {
      // Load DeFi strategies from localStorage
      const mockStrategies: ActiveStrategy[] = JSON.parse(
        localStorage.getItem('defi_strategies') || '[]'
      )
      
      setActiveStrategies(mockStrategies)
      
      // Calculate portfolio metrics
      const totalValue = mockStrategies.reduce((sum, strategy) => sum + strategy.current_value, 0)
      const totalRet = mockStrategies.reduce((sum, strategy) => sum + strategy.total_return, 0)
      
      setPortfolioValue(totalValue)
      setTotalReturn(totalRet)
    } catch (error) {
      console.error('Error loading portfolio data:', error)
    }
  }
  
  const handleTemplateSelect = (template: DeFiWorkflowTemplate) => {
    setSelectedTemplate(template)
    setViewMode('create-strategy')
  }
  
  const handleStrategyCreated = (response: StrategyFromTemplateResponse) => {
    const newStrategy: ActiveStrategy = {
      ...response,
      template: selectedTemplate!,
      created_at: new Date().toISOString(),
      status: 'active',
      current_value: Math.random() * 10000 + 5000,
      total_return: Math.random() * 2000 - 1000,
      roi_percentage: (Math.random() * 20) - 10
    }
    
    const updatedStrategies = [...activeStrategies, newStrategy]
    setActiveStrategies(updatedStrategies)
    
    // Save to localStorage
    localStorage.setItem('defi_strategies', JSON.stringify(updatedStrategies))
    
    // Update portfolio metrics
    loadPortfolioData()
    setViewMode('overview')
  }
  
  const handleCustomStrategyCreated = (config: StrategyConfig) => {
    console.log('Custom strategy created:', config)
    // Handle custom strategy creation
    setViewMode('overview')
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-4">
        <h3 className="text-red-800 font-medium">Error</h3>
        <p className="text-red-600 text-sm mt-1">{error}</p>
      </div>
    )
  }

  // Different view modes
  if (viewMode === 'defi-templates') {
    return (
      <div>
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-2xl font-bold text-gray-900">DeFi Strategy Templates</h2>
          <button 
            onClick={() => setViewMode('overview')}
            className="px-4 py-2 text-sm bg-gray-600 text-white rounded-lg hover:bg-gray-700"
          >
            Back to Dashboard
          </button>
        </div>
        <DeFiTemplates 
          onSelectTemplate={handleTemplateSelect} 
          onCreateCustom={() => setViewMode('custom-strategy')} 
        />
      </div>
    )
  }
  
  if (viewMode === 'create-strategy' && selectedTemplate) {
    return (
      <div>
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-2xl font-bold text-gray-900">Create Strategy: {selectedTemplate.name}</h2>
          <button 
            onClick={() => setViewMode('defi-templates')}
            className="px-4 py-2 text-sm bg-gray-600 text-white rounded-lg hover:bg-gray-700"
          >
            Back to Templates
          </button>
        </div>
        <StrategyCreationFlow 
          template={selectedTemplate} 
          onStrategyCreated={handleStrategyCreated}
          onCancel={() => setViewMode('defi-templates')}
        />
      </div>
    )
  }
  
  if (viewMode === 'custom-strategy') {
    return (
      <div>
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-2xl font-bold text-gray-900">Custom Strategy Builder</h2>
          <button 
            onClick={() => setViewMode('overview')}
            className="px-4 py-2 text-sm bg-gray-600 text-white rounded-lg hover:bg-gray-700"
          >
            Back to Dashboard
          </button>
        </div>
        <CustomStrategyBuilder 
          onStrategyCreated={handleCustomStrategyCreated}
          onCancel={() => setViewMode('overview')}
        />
      </div>
    )
  }
  
  return (
    <div className="space-y-6">
      {/* Portfolio Overview (DeFi Strategies) */}
      {activeStrategies.length > 0 && (
        <div className="bg-white rounded-lg shadow p-6 mb-6">
          <h3 className="text-lg font-medium text-gray-900 mb-4">DeFi Portfolio Overview</h3>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="text-center">
              <div className="text-2xl font-bold text-blue-600">${portfolioValue.toFixed(2)}</div>
              <div className="text-sm text-gray-600">Total Value</div>
            </div>
            <div className="text-center">
              <div className={`text-2xl font-bold ${
                totalReturn >= 0 ? 'text-green-600' : 'text-red-600'
              }`}>
                ${totalReturn >= 0 ? '+' : ''}${totalReturn.toFixed(2)}
              </div>
              <div className="text-sm text-gray-600">Total Return</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-purple-600">{activeStrategies.length}</div>
              <div className="text-sm text-gray-600">Active Strategies</div>
            </div>
          </div>
        </div>
      )}
      
      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <div className="bg-white rounded-lg shadow p-6">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <span className="text-2xl">⚡</span>
            </div>
            <div className="ml-4">
              <h3 className="text-lg font-medium text-gray-900">Total Workflows</h3>
              <p className="text-3xl font-bold text-blue-600">{workflows.length}</p>
            </div>
          </div>
        </div>

        <div className="bg-white rounded-lg shadow p-6">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <span className="text-2xl">▶️</span>
            </div>
            <div className="ml-4">
              <h3 className="text-lg font-medium text-gray-900">Active Workflows</h3>
              <p className="text-3xl font-bold text-green-600">
                {workflows.filter(w => w.active).length}
              </p>
            </div>
          </div>
        </div>

        <div className="bg-white rounded-lg shadow p-6">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <span className="text-2xl">📊</span>
            </div>
            <div className="ml-4">
              <h3 className="text-lg font-medium text-gray-900">Total Executions</h3>
              <p className="text-3xl font-bold text-purple-600">{executions.length}</p>
            </div>
          </div>
        </div>
        
        <div className="bg-white rounded-lg shadow p-6">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <span className="text-2xl">💰</span>
            </div>
            <div className="ml-4">
              <h3 className="text-lg font-medium text-gray-900">DeFi Strategies</h3>
              <p className="text-3xl font-bold text-yellow-600">{activeStrategies.length}</p>
            </div>
          </div>
        </div>
      </div>

      {/* Active DeFi Strategies */}
      {activeStrategies.length > 0 && (
        <div className="bg-white rounded-lg shadow">
          <div className="px-6 py-4 border-b">
            <div className="flex items-center justify-between">
              <h3 className="text-lg font-medium text-gray-900">Active DeFi Strategies</h3>
              <button 
                onClick={() => setViewMode('defi-templates')}
                className="text-blue-600 hover:text-blue-700 text-sm font-medium"
              >
                Browse templates
              </button>
            </div>
          </div>
          
          <div className="p-6">
            <div className="space-y-4">
              {activeStrategies.slice(0, 3).map((strategy, index) => (
                <div key={index} className="flex items-center justify-between p-4 border rounded-lg">
                  <div>
                    <h4 className="font-medium text-gray-900">{strategy.template.name}</h4>
                    <p className="text-sm text-gray-600">{strategy.template.description}</p>
                    <p className="text-xs text-gray-500 mt-1">
                      Created {new Date(strategy.created_at).toLocaleDateString()}
                    </p>
                  </div>
                  <div className="text-right">
                    <div className={`text-sm font-medium ${
                      strategy.total_return >= 0 ? 'text-green-600' : 'text-red-600'
                    }`}>
                      ${strategy.total_return >= 0 ? '+' : ''}${strategy.total_return.toFixed(2)}
                    </div>
                    <div className="text-xs text-gray-500">
                      ${strategy.current_value.toFixed(2)}
                    </div>
                    <span className={`px-2 py-1 text-xs rounded-full ${
                      strategy.status === 'active' 
                        ? 'bg-green-100 text-green-800' 
                        : 'bg-gray-100 text-gray-800'
                    }`}>
                      {strategy.status}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
      
      {/* Recent Workflows */}
      <div className="bg-white rounded-lg shadow">
        <div className="px-6 py-4 border-b">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-medium text-gray-900">Recent Workflows</h3>
            <Link 
              to="/workflows" 
              className="text-blue-600 hover:text-blue-700 text-sm font-medium"
            >
              View all
            </Link>
          </div>
        </div>
        
        <div className="p-6">
          {workflows.length === 0 ? (
            <div className="text-center py-8">
              <span className="text-4xl mb-4 block">📝</span>
              <h4 className="text-lg font-medium text-gray-900 mb-2">No workflows yet</h4>
              <p className="text-gray-600 mb-4">Get started by creating your first workflow</p>
              <Link 
                to="/workflows/new"
                className="inline-flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
              >
                Create Workflow
              </Link>
            </div>
          ) : (
            <div className="space-y-4">
              {workflows.slice(0, 3).map((workflow) => (
                <div key={workflow.id} className="flex items-center justify-between p-4 border rounded-lg">
                  <div>
                    <h4 className="font-medium text-gray-900">{workflow.name}</h4>
                    <p className="text-sm text-gray-600">{workflow.description || 'No description'}</p>
                    <p className="text-xs text-gray-500 mt-1">
                      Created {TimestampUtils.icpTimestampToDate(workflow.created_at).toLocaleDateString()}
                    </p>
                  </div>
                  <div className="flex items-center space-x-2">
                    <span className={`px-2 py-1 text-xs rounded-full ${
                      workflow.active 
                        ? 'bg-green-100 text-green-800' 
                        : 'bg-gray-100 text-gray-800'
                    }`}>
                      {workflow.active ? 'Active' : 'Inactive'}
                    </span>
                    <Link 
                      to={`/workflows/${workflow.id}`}
                      className="text-blue-600 hover:text-blue-700 text-sm"
                    >
                      Edit
                    </Link>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Quick Actions */}
      <div className="bg-white rounded-lg shadow p-6">
        <h3 className="text-lg font-medium text-gray-900 mb-4">Quick Actions</h3>
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          <button
            onClick={() => setViewMode('defi-templates')}
            className="flex items-center p-4 border-2 border-dashed border-gray-300 rounded-lg hover:border-blue-400 hover:bg-blue-50 transition-colors"
          >
            <span className="text-2xl mr-3">💰</span>
            <div>
              <h4 className="font-medium text-gray-900">DeFi Templates</h4>
              <p className="text-sm text-gray-600">Browse strategy templates</p>
            </div>
          </button>
          
          <Link 
            to="/workflows/new"
            className="flex items-center p-4 border-2 border-dashed border-gray-300 rounded-lg hover:border-blue-400 hover:bg-blue-50 transition-colors"
          >
            <span className="text-2xl mr-3">➕</span>
            <div>
              <h4 className="font-medium text-gray-900">Custom Workflow</h4>
              <p className="text-sm text-gray-600">Build from scratch</p>
            </div>
          </Link>
          
          <Link 
            to="/executions"
            className="flex items-center p-4 border-2 border-dashed border-gray-300 rounded-lg hover:border-blue-400 hover:bg-blue-50 transition-colors"
          >
            <span className="text-2xl mr-3">📋</span>
            <div>
              <h4 className="font-medium text-gray-900">View Executions</h4>
              <p className="text-sm text-gray-600">Monitor workflow runs</p>
            </div>
          </Link>
          
          <Link 
            to="/settings"
            className="flex items-center p-4 border-2 border-dashed border-gray-300 rounded-lg hover:border-blue-400 hover:bg-blue-50 transition-colors"
          >
            <span className="text-2xl mr-3">⚙️</span>
            <div>
              <h4 className="font-medium text-gray-900">Settings</h4>
              <p className="text-sm text-gray-600">Configure your setup</p>
            </div>
          </Link>
        </div>
      </div>
    </div>
  )
}

export default Dashboard