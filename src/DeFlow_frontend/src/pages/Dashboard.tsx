import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { useWorkflowStore } from '../stores/workflowStore'
import { TimestampUtils } from '../utils/timestamp-utils'


type ViewMode = 'overview'

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

  useEffect(() => {
    loadWorkflows()
    loadExecutions()
  }, [loadWorkflows, loadExecutions])

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

  
  return (
    <div className="space-y-6">
      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
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
        
      </div>

      {/* Yield Optimization Demo Highlight */}
      <div className="bg-gradient-to-r from-blue-50 to-purple-50 border border-blue-200 rounded-lg p-6">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-4">
            <div className="flex-shrink-0">
              <span className="text-3xl">🚀</span>
            </div>
            <div>
              <h3 className="text-lg font-semibold text-gray-900">
                Try Our Yield Optimization Demo
              </h3>
              <p className="text-sm text-gray-600 mt-1">
                Experience automated stable coin yield optimization with real protocol data. 
                See how DeFlow automatically selects the best yields while managing gas costs and bridge fees.
              </p>
              <div className="flex items-center space-x-4 mt-2">
                <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800">
                  📊 Real Data Simulation
                </span>
                <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                  💰 Cost Optimization
                </span>
                <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-purple-100 text-purple-800">
                  🎯 Risk Management
                </span>
              </div>
            </div>
          </div>
          <div className="flex-shrink-0">
            <Link 
              to="/demo/yield-optimization" 
              className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg font-medium transition-colors"
            >
              Launch Demo
            </Link>
          </div>
        </div>
      </div>
      
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
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          
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