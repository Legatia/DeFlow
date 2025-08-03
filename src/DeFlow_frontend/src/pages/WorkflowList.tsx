import { useEffect } from 'react'
import { Link } from 'react-router-dom'
import { useWorkflowStore } from '../stores/workflowStore'
import { TimestampUtils } from '../utils/timestamp-utils'

const WorkflowList = () => {
  const { 
    workflows, 
    isLoading, 
    error, 
    loadWorkflows, 
    deleteWorkflow,
    executeWorkflow 
  } = useWorkflowStore()

  useEffect(() => {
    loadWorkflows()
  }, [loadWorkflows])

  const handleDelete = async (id: string, name: string) => {
    if (confirm(`Are you sure you want to delete "${name}"?`)) {
      try {
        await deleteWorkflow(id)
      } catch (error) {
        alert('Failed to delete workflow')
      }
    }
  }

  const handleExecute = async (id: string, name: string) => {
    try {
      const executionId = await executeWorkflow(id)
      alert(`Workflow "${name}" execution started. ID: ${executionId}`)
    } catch (error) {
      alert('Failed to start workflow execution')
    }
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

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-gray-900">Workflows</h1>
        <Link 
          to="/workflows/new"
          className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        >
          Create Workflow
        </Link>
      </div>

      {/* Workflows Grid */}
      {workflows.length === 0 ? (
        <div className="bg-white rounded-lg shadow p-8 text-center">
          <span className="text-6xl mb-4 block">📝</span>
          <h3 className="text-lg font-medium text-gray-900 mb-2">No workflows yet</h3>
          <p className="text-gray-600 mb-4">Get started by creating your first workflow</p>
          <Link 
            to="/workflows/new"
            className="inline-flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            Create Your First Workflow
          </Link>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {workflows.map((workflow) => (
            <div key={workflow.id} className="bg-white rounded-lg shadow hover:shadow-md transition-shadow">
              <div className="p-6">
                <div className="flex items-start justify-between mb-4">
                  <h3 className="text-lg font-medium text-gray-900 truncate">{workflow.name}</h3>
                  <span className={`px-2 py-1 text-xs rounded-full ${
                    workflow.active 
                      ? 'bg-green-100 text-green-800' 
                      : 'bg-gray-100 text-gray-800'
                  }`}>
                    {workflow.active ? 'Active' : 'Inactive'}
                  </span>
                </div>
                
                <p className="text-sm text-gray-600 mb-4 line-clamp-2">
                  {workflow.description || 'No description provided'}
                </p>
                
                <div className="flex items-center justify-between text-xs text-gray-500 mb-4">
                  <span>{workflow.nodes.length} nodes</span>
                  <span>{workflow.connections.length} connections</span>
                  <span>{workflow.triggers.length} triggers</span>
                </div>
                
                <div className="text-xs text-gray-500 mb-4">
                  Created {TimestampUtils.icpTimestampToDate(workflow.created_at).toLocaleDateString()}
                </div>
                
                <div className="flex items-center space-x-2">
                  <Link 
                    to={`/workflows/${workflow.id}`}
                    className="flex-1 px-3 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors text-center"
                  >
                    Edit
                  </Link>
                  <button 
                    onClick={() => handleExecute(workflow.id, workflow.name)}
                    disabled={!workflow.active}
                    className="px-3 py-2 text-sm bg-green-600 text-white rounded hover:bg-green-700 transition-colors disabled:bg-gray-400"
                  >
                    Run
                  </button>
                  <button 
                    onClick={() => handleDelete(workflow.id, workflow.name)}
                    className="px-3 py-2 text-sm bg-red-600 text-white rounded hover:bg-red-700 transition-colors"
                  >
                    Delete
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}

export default WorkflowList