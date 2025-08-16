import { useState, useEffect, useCallback } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { Node, Edge } from 'reactflow'
import { useWorkflowStore } from '../stores/workflowStore'
import { Workflow } from '../types'
import WorkflowBuilder from '../components/WorkflowBuilder'
import WorkflowTemplates from '../components/WorkflowTemplates'
import { WorkflowTemplate } from '../data/workflowTemplates'

const WorkflowEditor = () => {
  const { id } = useParams()
  const navigate = useNavigate()
  const { 
    currentWorkflow, 
    isLoading, 
    error, 
    loadWorkflow, 
    createWorkflow, 
    updateWorkflow 
  } = useWorkflowStore()

  const [formData, setFormData] = useState({
    name: '',
    description: '',
    active: true
  })
  const [showBuilder, setShowBuilder] = useState(false)
  const [showTemplates, setShowTemplates] = useState(!Boolean(id)) // Show templates for new workflows
  const [workflowNodes, setWorkflowNodes] = useState<Node[]>([])
  const [workflowEdges, setWorkflowEdges] = useState<Edge[]>([])

  const isEditing = Boolean(id)

  useEffect(() => {
    if (isEditing && id) {
      loadWorkflow(id)
    }
  }, [id, isEditing, loadWorkflow])

  useEffect(() => {
    if (currentWorkflow && isEditing) {
      setFormData({
        name: currentWorkflow.name,
        description: currentWorkflow.description || '',
        active: currentWorkflow.active
      })
      
      // Convert workflow nodes to React Flow format
      const nodes: Node[] = currentWorkflow.nodes.map(node => ({
        id: node.id,
        type: 'workflowNode',
        position: node.position,
        data: {
          nodeType: {
            id: node.node_type,
            name: node.metadata.label || node.node_type,
            description: node.metadata.description || '',
            category: 'actions' as any,
            icon: node.metadata.icon || '⚡',
            color: node.metadata.color || '#10b981',
            inputs: [],
            outputs: [],
            configSchema: [],
            defaultConfig: {}
          },
          config: node.configuration.parameters || {},
          isValid: true,
          errors: []
        }
      }))

      const edges: Edge[] = currentWorkflow.connections.map(conn => ({
        id: conn.id,
        source: conn.source_node_id,
        target: conn.target_node_id,
        sourceHandle: conn.source_output,
        targetHandle: conn.target_input,
        type: 'smoothstep'
      }))

      setWorkflowNodes(nodes)
      setWorkflowEdges(edges)
    }
  }, [currentWorkflow, isEditing])

  const handleSaveWorkflow = useCallback(async (nodes: Node[], edges: Edge[]) => {
    if (!formData.name) {
      alert('Please enter a workflow name first')
      return
    }

    try {
      // Convert React Flow format back to workflow format
      const workflowNodes = nodes.map(node => ({
        id: node.id,
        node_type: node.data.nodeType.id,
        position: node.position,
        configuration: {
          parameters: node.data.config || {}
        },
        metadata: {
          label: node.data.nodeType.name,
          description: node.data.nodeType.description,
          tags: [node.data.nodeType.category],
          icon: node.data.nodeType.icon,
          color: node.data.nodeType.color
        }
      }))

      const workflowConnections = edges.map(edge => ({
        id: edge.id,
        source_node_id: edge.source,
        target_node_id: edge.target,
        source_output: edge.sourceHandle || 'output',
        target_input: edge.targetHandle || 'input'
      }))

      const workflowData = {
        ...formData,
        nodes: workflowNodes,
        connections: workflowConnections,
        triggers: [{ type: 'manual' as const }],
        state: 'published' as const
      }

      if (isEditing && currentWorkflow) {
        const updatedWorkflow: Workflow = {
          ...currentWorkflow,
          ...workflowData
        }
        await updateWorkflow(updatedWorkflow)
      } else {
        await createWorkflow(workflowData)
      }

      alert('Workflow saved successfully!')
    } catch (error) {
      console.error('Failed to save workflow:', error)
      alert(`Failed to ${isEditing ? 'update' : 'create'} workflow`)
    }
  }, [formData, isEditing, currentWorkflow, createWorkflow, updateWorkflow])

  const handleBasicInfoSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    setShowBuilder(true)
  }

  const handleSelectTemplate = useCallback((template: WorkflowTemplate) => {
    setFormData({
      name: template.name,
      description: template.description,
      active: true
    })
    setWorkflowNodes(template.nodes)
    setWorkflowEdges(template.edges)
    setShowTemplates(false)
    setShowBuilder(true)
  }, [])

  const handleCreateBlank = useCallback(() => {
    setShowTemplates(false)
  }, [])

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

  if (showBuilder || (isEditing && formData.name)) {
    return (
      <div className="h-full flex flex-col">
        {/* Header */}
        <div className="bg-white border-b border-gray-200 px-6 py-4 flex items-center justify-between">
          <div>
            <h1 className="text-xl font-bold text-gray-900">
              {isEditing ? 'Edit' : 'Create'}: {formData.name}
            </h1>
            <p className="text-sm text-gray-600">{formData.description || 'No description'}</p>
          </div>
          <div className="flex items-center space-x-3">
            <button
              onClick={() => setShowBuilder(false)}
              className="px-4 py-2 text-gray-600 border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
            >
              Back to Info
            </button>
            <button
              onClick={() => navigate('/workflows')}
              className="px-4 py-2 text-gray-600 border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
            >
              Cancel
            </button>
          </div>
        </div>

        {/* Workflow Builder */}
        <div className="flex-1">
          <WorkflowBuilder 
            initialNodes={workflowNodes}
            initialEdges={workflowEdges}
            onSave={handleSaveWorkflow}
          />
        </div>
      </div>
    )
  }

  // Show template selection for new workflows
  if (showTemplates && !isEditing) {
    return (
      <WorkflowTemplates 
        onSelectTemplate={handleSelectTemplate}
        onCreateBlank={handleCreateBlank}
      />
    )
  }

  return (
    <div className="max-w-2xl mx-auto">
      <div className="bg-white rounded-lg shadow p-6">
        <h1 className="text-2xl font-bold text-gray-900 mb-6">
          {isEditing ? 'Edit Workflow Info' : 'Create New Workflow'}
        </h1>

        <form onSubmit={handleBasicInfoSubmit} className="space-y-6">
          <div>
            <label htmlFor="name" className="block text-sm font-medium text-gray-700 mb-1">
              Workflow Name
            </label>
            <input
              type="text"
              id="name"
              required
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              placeholder="Enter workflow name"
            />
          </div>

          <div>
            <label htmlFor="description" className="block text-sm font-medium text-gray-700 mb-1">
              Description (Optional)
            </label>
            <textarea
              id="description"
              rows={3}
              value={formData.description}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              placeholder="Describe what this workflow does"
            />
          </div>

          <div className="flex items-center">
            <input
              type="checkbox"
              id="active"
              checked={formData.active}
              onChange={(e) => setFormData({ ...formData, active: e.target.checked })}
              className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
            />
            <label htmlFor="active" className="ml-2 block text-sm text-gray-700">
              Active (workflow can be triggered)
            </label>
          </div>

          {/* Next Steps Preview */}
          <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
            <h3 className="text-lg font-medium text-blue-900 mb-2">Next Steps</h3>
            <p className="text-blue-700 text-sm mb-3">
              After saving the basic info, you'll be able to:
            </p>
            <ul className="text-blue-700 text-sm space-y-1">
              <li>• 🎨 Design your workflow visually with drag & drop</li>
              <li>• 🔗 Connect nodes to create automation flows</li>
              <li>• ⚙️ Configure each node with specific settings</li>
              <li>• 🧪 Test your workflow before deployment</li>
            </ul>
          </div>

          <div className="flex items-center justify-between pt-6 border-t">
            <button
              type="button"
              onClick={() => navigate('/workflows')}
              className="px-4 py-2 text-gray-600 border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
            >
              Continue to Builder →
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

export default WorkflowEditor