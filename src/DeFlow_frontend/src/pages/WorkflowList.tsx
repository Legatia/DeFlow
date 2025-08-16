import { useEffect, useState, useMemo } from 'react'
import { Link } from 'react-router-dom'
import { useWorkflowStore } from '../stores/workflowStore'
import { TimestampUtils } from '../utils/timestamp-utils'
import { Workflow, WorkflowState } from '../types/index'

type TabType = 'published' | 'draft' | 'template'

const WorkflowList = () => {
  const { 
    workflows, 
    isLoading, 
    error, 
    loadWorkflows, 
    deleteWorkflow,
    executeWorkflow 
  } = useWorkflowStore()
  
  const [activeTab, setActiveTab] = useState<TabType>('published')

  useEffect(() => {
    loadWorkflows()
  }, [loadWorkflows])

  // Filter workflows by state
  const filteredWorkflows = useMemo(() => {
    return workflows.filter(workflow => {
      const state = workflow.state || 'published' // Default to published for existing workflows
      return state === activeTab
    })
  }, [workflows, activeTab])

  // Count workflows by state
  const workflowCounts = useMemo(() => {
    const counts = { published: 0, draft: 0, template: 0 }
    workflows.forEach(workflow => {
      const state = workflow.state || 'published'
      counts[state as WorkflowState]++
    })
    return counts
  }, [workflows])

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

  // Get workflow state display info
  const getWorkflowStateInfo = (workflow: Workflow) => {
    const state = workflow.state || 'published'
    
    switch (state) {
      case 'draft':
        return {
          badge: 'bg-yellow-100 text-yellow-800',
          label: 'Draft',
          icon: '📝'
        }
      case 'template':
        return {
          badge: 'bg-purple-100 text-purple-800',
          label: 'Template',
          icon: '📋'
        }
      case 'published':
      default:
        return {
          badge: workflow.active ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800',
          label: workflow.active ? 'Active' : 'Inactive',
          icon: workflow.active ? '🚀' : '⏸️'
        }
    }
  }

  // Get appropriate actions for workflow type
  const getWorkflowActions = (workflow: Workflow) => {
    const state = workflow.state || 'published'
    
    const baseActions = [
      <Link 
        key="edit"
        to={`/workflows/${workflow.id}`}
        className="flex-1 px-3 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors text-center"
      >
        {state === 'draft' ? 'Continue' : 'Edit'}
      </Link>,
      <button 
        key="delete"
        onClick={() => handleDelete(workflow.id, workflow.name)}
        className="px-3 py-2 text-sm bg-red-600 text-white rounded hover:bg-red-700 transition-colors"
      >
        Delete
      </button>
    ]

    switch (state) {
      case 'draft':
        return baseActions
      case 'template':
        return [
          <Link 
            key="use"
            to={`/workflows/new?template=${workflow.id}`}
            className="flex-1 px-3 py-2 text-sm bg-green-600 text-white rounded hover:bg-green-700 transition-colors text-center"
          >
            Use Template
          </Link>,
          ...baseActions
        ]
      case 'published':
      default:
        return [
          baseActions[0], // Edit button
          <button 
            key="run"
            onClick={() => handleExecute(workflow.id, workflow.name)}
            disabled={!workflow.active}
            className="px-3 py-2 text-sm bg-green-600 text-white rounded hover:bg-green-700 transition-colors disabled:bg-gray-400"
          >
            Run
          </button>,
          baseActions[1] // Delete button
        ]
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

      {/* Tabs */}
      <div className="border-b border-gray-200">
        <nav className="-mb-px flex space-x-8">
          {[
            { key: 'published', label: 'Published', icon: '🚀' },
            { key: 'draft', label: 'Drafts', icon: '📝' },
            { key: 'template', label: 'Templates', icon: '📋' }
          ].map((tab) => (
            <button
              key={tab.key}
              onClick={() => setActiveTab(tab.key as TabType)}
              className={`py-2 px-1 border-b-2 font-medium text-sm whitespace-nowrap flex items-center space-x-2 ${
                activeTab === tab.key
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              <span>{tab.icon}</span>
              <span>{tab.label}</span>
              <span className="bg-gray-100 text-gray-600 py-0.5 px-2 rounded-full text-xs">
                {workflowCounts[tab.key as WorkflowState]}
              </span>
            </button>
          ))}
        </nav>
      </div>

      {/* Workflows Grid */}
      {filteredWorkflows.length === 0 ? (
        <div className="bg-white rounded-lg shadow p-8 text-center">
          <span className="text-6xl mb-4 block">
            {activeTab === 'draft' ? '📝' : activeTab === 'template' ? '📋' : '🚀'}
          </span>
          <h3 className="text-lg font-medium text-gray-900 mb-2">
            {activeTab === 'draft' && 'No drafts yet'}
            {activeTab === 'template' && 'No templates yet'}
            {activeTab === 'published' && 'No published workflows yet'}
          </h3>
          <p className="text-gray-600 mb-4">
            {activeTab === 'draft' && 'Save work-in-progress workflows as drafts to continue later'}
            {activeTab === 'template' && 'Create reusable templates for common automation patterns'}
            {activeTab === 'published' && 'Get started by creating your first workflow'}
          </p>
          <Link 
            to="/workflows/new"
            className="inline-flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            Create Your First Workflow
          </Link>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {filteredWorkflows.map((workflow) => {
            const stateInfo = getWorkflowStateInfo(workflow)
            const actions = getWorkflowActions(workflow)
            
            return (
              <div key={workflow.id} className="bg-white rounded-lg shadow hover:shadow-md transition-shadow">
                <div className="p-6">
                  <div className="flex items-start justify-between mb-4">
                    <h3 className="text-lg font-medium text-gray-900 truncate">{workflow.name}</h3>
                    <span className={`px-2 py-1 text-xs rounded-full ${stateInfo.badge}`}>
                      {stateInfo.label}
                    </span>
                  </div>
                  
                  <p className="text-sm text-gray-600 mb-4 line-clamp-2">
                    {workflow.description || workflow.metadata?.templateDescription || 'No description provided'}
                  </p>

                  {/* Template-specific info */}
                  {activeTab === 'template' && workflow.metadata?.templateCategory && (
                    <div className="mb-4">
                      <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                        {workflow.metadata.templateCategory}
                      </span>
                      {workflow.metadata.usageCount !== undefined && (
                        <span className="ml-2 text-xs text-gray-500">
                          Used {workflow.metadata.usageCount} times
                        </span>
                      )}
                    </div>
                  )}
                  
                  <div className="flex items-center justify-between text-xs text-gray-500 mb-4">
                    <span>{workflow.nodes.length} nodes</span>
                    <span>{workflow.connections.length} connections</span>
                    <span>{workflow.triggers.length} triggers</span>
                  </div>
                  
                  <div className="text-xs text-gray-500 mb-4">
                    {activeTab === 'template' ? 'Created' : 'Updated'} {TimestampUtils.icpTimestampToDate(
                      activeTab === 'template' ? workflow.created_at : workflow.updated_at
                    ).toLocaleDateString()}
                  </div>
                  
                  <div className="flex items-center space-x-2">
                    {actions}
                  </div>
                </div>
              </div>
            )
          })}
        </div>
      )}
    </div>
  )
}

export default WorkflowList