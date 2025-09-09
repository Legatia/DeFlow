import React, { useCallback, useState, useMemo, useEffect, memo, useRef } from 'react'
import ReactFlow, {
  MiniMap,
  Controls,
  Background,
  useNodesState,
  useEdgesState,
  addEdge,
  Node,
  Edge,
  Connection,
  NodeTypes,
  BackgroundVariant,
  Panel,
  useReactFlow,
  ReactFlowInstance,
} from 'reactflow'
import 'reactflow/dist/style.css'

import { getAllNodeTypes, NodeType } from '../types/all-nodes'
import { useEnhancedAuth } from '../contexts/EnhancedAuthContext'
import { canAddNodeToWorkflow, getUpgradePath } from '../utils/subscriptionUtils'
import { Workflow, WorkflowState } from '../types/index'
import WorkflowNode from './WorkflowNode'
import { EnhancedNodePalette } from './EnhancedNodePalette'
import NodeConfigPanel from './NodeConfigPanel'
import SaveWorkflowModal from './SaveWorkflowModal'
import { Waves } from './ui/wave-background'

interface WorkflowBuilderProps {
  initialNodes?: Node[]
  initialEdges?: Edge[]
  onSave?: (nodes: Node[], edges: Edge[]) => void
  onSaveAsDraft?: (nodes: Node[], edges: Edge[], name: string) => void
  onPublish?: (nodes: Node[], edges: Edge[], name: string) => void
  onSaveAsTemplate?: (nodes: Node[], edges: Edge[], name: string, category: string, description: string) => void
  readOnly?: boolean
  currentWorkflow?: Partial<Workflow>
}

const WorkflowBuilder = memo(({ 
  initialNodes = [], 
  initialEdges = [], 
  onSave,
  onSaveAsDraft,
  onPublish,
  onSaveAsTemplate,
  readOnly = false,
  currentWorkflow 
}: WorkflowBuilderProps) => {
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes)
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges)
  const [selectedNode, setSelectedNode] = useState<Node | null>(null)
  const [isConfigPanelOpen, setIsConfigPanelOpen] = useState(false)
  const [reactFlowInstance, setReactFlowInstance] = useState<ReactFlowInstance | null>(null)
  const [showSaveModal, setShowSaveModal] = useState(false)
  const [saveModalType, setSaveModalType] = useState<'draft' | 'publish' | 'template'>('draft')
  const { subscriptionTier } = useEnhancedAuth()
  
  // PERFORMANCE: Refs for cleanup and performance tracking
  const autoSaveTimeoutRef = useRef<NodeJS.Timeout>()
  const isUnmountingRef = useRef(false)

  // Define custom node types
  const nodeTypes: NodeTypes = useMemo(
    () => ({
      workflowNode: WorkflowNode,
    }),
    []
  )

  // Handle new connections between nodes
  const onConnect = useCallback(
    (params: Connection) => {
      // Add validation logic here if needed
      setEdges((eds) => addEdge({ ...params, type: 'smoothstep' }, eds))
    },
    [setEdges]
  )

  // Handle node selection
  const onNodeClick = useCallback((event: React.MouseEvent, node: Node) => {
    if (readOnly) return
    setSelectedNode(node)
    setIsConfigPanelOpen(true)
  }, [readOnly])

  // PERFORMANCE: Memoized node types to prevent repeated calls
  const allNodeTypes = useMemo(() => getAllNodeTypes(), [])
  
  // Handle dropping new nodes from palette
  const onDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault()
      event.stopPropagation()

      const nodeTypeId = event.dataTransfer.getData('application/reactflow')
      console.log('Dropped node type ID:', nodeTypeId) // Debug log
      
      const nodeType = allNodeTypes.find(nt => nt.id === nodeTypeId)
      
      if (!nodeType) {
        console.warn('Node type not found:', nodeTypeId)
        return
      }

      // CHECK SUBSCRIPTION TIER ACCESS - Universal access control
      if (!canAddNodeToWorkflow(subscriptionTier, nodeType)) {
        const requiredTier = nodeType.requiredTier || 'standard'
        const upgradePath = getUpgradePath(subscriptionTier, requiredTier)
        
        if (upgradePath) {
          alert(`⚠️ Cannot add "${nodeType.name}" node\n\nThis node requires ${upgradePath.name} subscription (${upgradePath.price}/month).\n\nPlease upgrade to access this feature.`)
        }
        return // Prevent node creation
      }

      // Get accurate position using React Flow instance
      let position = { x: 100, y: 100 } // Default position
      
      if (reactFlowInstance) {
        try {
          const reactFlowBounds = (event.target as HTMLElement).closest('.react-flow')?.getBoundingClientRect()
          if (reactFlowBounds) {
            const x = event.clientX - reactFlowBounds.left
            const y = event.clientY - reactFlowBounds.top
            position = reactFlowInstance.screenToFlowPosition({ x, y })
          }
        } catch (error) {
          console.warn('Failed to get drop position:', error)
        }
      }

      const newNode: Node = {
        id: `${nodeType.id}-${Date.now()}`,
        type: 'workflowNode',
        position,
        data: {
          nodeType: nodeType,
          config: { ...nodeType.defaultConfig },
          isValid: true,
          errors: []
        },
      }

      console.log('Creating new node:', newNode) // Debug log
      setNodes((nds) => nds.concat(newNode))
    },
    [setNodes, subscriptionTier, reactFlowInstance]
  )

  const onDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault()
    event.dataTransfer.dropEffect = 'move'
  }, [])

  // Handle node configuration updates
  const onNodeConfigChange = useCallback((nodeId: string, config: Record<string, any>) => {
    setNodes((nds) =>
      nds.map((node) => {
        if (node.id === nodeId) {
          return {
            ...node,
            data: {
              ...node.data,
              config,
            },
          }
        }
        return node
      })
    )
  }, [setNodes])

  // Handle node deletion
  const onDeleteNode = useCallback((nodeId: string) => {
    setNodes((nds) => nds.filter((node) => node.id !== nodeId))
    setEdges((eds) => eds.filter((edge) => edge.source !== nodeId && edge.target !== nodeId))
    setIsConfigPanelOpen(false)
    setSelectedNode(null)
  }, [setNodes, setEdges])

  // Save workflow (original function)
  const handleSave = useCallback(() => {
    if (onSave) {
      onSave(nodes, edges)
    }
  }, [nodes, edges, onSave])

  // Open save modal
  const handleSaveAs = useCallback((type: 'draft' | 'publish' | 'template') => {
    setSaveModalType(type)
    setShowSaveModal(true)
  }, [])

  // PERFORMANCE: Memoized auto-save with error handling
  const handleAutoSaveDraft = useCallback(() => {
    if (onSaveAsDraft && nodes.length > 0 && !isUnmountingRef.current) {
      try {
        const draftName = `Auto-saved ${new Date().toLocaleTimeString()}`
        onSaveAsDraft(nodes, edges, draftName)
      } catch (error) {
        console.error('Auto-save failed:', error)
      }
    }
  }, [nodes, edges, onSaveAsDraft])

  // PERFORMANCE: Optimized auto-save with debouncing and cleanup
  useEffect(() => {
    if (!readOnly && nodes.length > 0) {
      // Clear any existing timeout
      if (autoSaveTimeoutRef.current) {
        clearTimeout(autoSaveTimeoutRef.current)
      }
      
      // Debounce auto-save to prevent excessive saves during rapid changes
      autoSaveTimeoutRef.current = setTimeout(() => {
        if (!isUnmountingRef.current) {
          handleAutoSaveDraft()
        }
      }, 30000) // Auto-save every 30 seconds
      
      return () => {
        if (autoSaveTimeoutRef.current) {
          clearTimeout(autoSaveTimeoutRef.current)
        }
      }
    }
  }, [handleAutoSaveDraft, nodes.length, readOnly])
  
  // PERFORMANCE: Cleanup effect to prevent memory leaks
  useEffect(() => {
    return () => {
      isUnmountingRef.current = true
      if (autoSaveTimeoutRef.current) {
        clearTimeout(autoSaveTimeoutRef.current)
      }
    }
  }, [])

  // Clear workflow
  const handleClear = useCallback(() => {
    setNodes([])
    setEdges([])
    setSelectedNode(null)
    setIsConfigPanelOpen(false)
  }, [setNodes, setEdges])

  return (
    <div className="h-full w-full flex">
      {/* Enhanced Node Palette - Left Sidebar with Liquid Glass */}
      {!readOnly && (
        <EnhancedNodePalette />
      )}

      {/* Main Canvas Area */}
      <div className="flex-1 relative overflow-hidden bg-gradient-to-br from-gray-900 via-slate-900 to-gray-800">
        {/* Animated Wave Background */}
        <div className="absolute inset-0 z-0 pointer-events-none">
          <Waves 
            className="w-full h-full" 
            strokeColor="rgba(255, 255, 255, 0.12)"
            backgroundColor="transparent"
            pointerSize={0.2}
          />
        </div>
        
        {/* ReactFlow Canvas */}
        <div className="relative z-10 w-full h-full">
          <ReactFlow
          className="w-full h-full"
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          onNodeClick={onNodeClick}
          onDrop={onDrop}
          onDragOver={onDragOver}
          onInit={setReactFlowInstance}
          nodeTypes={nodeTypes}
          deleteKeyCode={readOnly ? null : 'Delete'}
          multiSelectionKeyCode={readOnly ? null : 'Shift'}
          panOnDrag={true}
          panOnScroll={true}
          zoomOnScroll={true}
          zoomOnPinch={true}
          zoomOnDoubleClick={false}
          fitView
          attributionPosition="bottom-left"
        >
          <Controls showInteractive={!readOnly} />
          {/* PERFORMANCE: Conditionally render MiniMap for performance */}
          {nodes.length > 0 && (
            <MiniMap 
              nodeStrokeColor="#374151"
              nodeColor="#f3f4f6"
              nodeBorderRadius={8}
              maskColor="rgba(0, 0, 0, 0.1)"
              pannable
              zoomable
            />
          )}
          {/* Background removed - using animated waves instead */}
          
          {/* Top Panel with Actions */}
          {!readOnly && (
            <Panel position="top-right" className="flex space-x-2">
              <button
                onClick={handleClear}
                className="px-3 py-1 bg-red-600 text-white text-sm rounded hover:bg-red-700 transition-colors"
              >
                Clear
              </button>
              
              {/* Save Dropdown */}
              <div className="relative group">
                <button
                  onClick={handleSave}
                  className="px-3 py-1 bg-blue-600 text-white text-sm rounded hover:bg-blue-700 transition-colors"
                >
                  Save
                </button>
                <div className="absolute right-0 top-full mt-1 bg-white border border-gray-200 rounded-lg shadow-lg opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-200 z-50">
                  <div className="py-1 min-w-[160px]">
                    <button
                      onClick={() => handleSaveAs('draft')}
                      className="w-full text-left px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 flex items-center"
                    >
                      <span className="mr-2">📝</span>
                      Save as Draft
                    </button>
                    <button
                      onClick={() => handleSaveAs('publish')}
                      className="w-full text-left px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 flex items-center"
                    >
                      <span className="mr-2">🚀</span>
                      Publish
                    </button>
                    <button
                      onClick={() => handleSaveAs('template')}
                      className="w-full text-left px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 flex items-center"
                    >
                      <span className="mr-2">📋</span>
                      Save as Template
                    </button>
                  </div>
                </div>
              </div>
            </Panel>
          )}

          {/* Info Panel - PERFORMANCE: Memoized stats */}
          <Panel position="top-left">
            <div className="bg-white bg-opacity-90 backdrop-blur-sm rounded-lg p-3 shadow-sm">
              <div className="text-sm text-gray-600">
                <div>Nodes: {nodes.length}</div>
                <div>Connections: {edges.length}</div>
                <div>Memory: {((nodes.length + edges.length) * 0.1).toFixed(1)} KB</div>
                {readOnly && <div className="text-blue-600 font-medium">Read Only</div>}
              </div>
            </div>
          </Panel>
        </ReactFlow>
        </div>
      </div>

      {/* Node Configuration Panel - Right Sidebar */}
      {!readOnly && isConfigPanelOpen && selectedNode && (
        <div className="w-80 bg-white border-l border-gray-200 flex flex-col">
          <div className="p-4 border-b border-gray-200 flex items-center justify-between">
            <h3 className="text-lg font-medium text-gray-900">Configure Node</h3>
            <button
              onClick={() => setIsConfigPanelOpen(false)}
              className="text-gray-400 hover:text-gray-600"
            >
              ✕
            </button>
          </div>
          <div className="flex-1 overflow-y-auto">
            <NodeConfigPanel
              node={selectedNode}
              onConfigChange={onNodeConfigChange}
              onDelete={onDeleteNode}
            />
          </div>
        </div>
      )}

      {/* Save Modal */}
      {showSaveModal && (
        <SaveWorkflowModal
          type={saveModalType}
          onSave={(name, category?, description?) => {
            if (saveModalType === 'draft' && onSaveAsDraft) {
              onSaveAsDraft(nodes, edges, name)
            } else if (saveModalType === 'publish' && onPublish) {
              onPublish(nodes, edges, name)
            } else if (saveModalType === 'template' && onSaveAsTemplate && category && description) {
              onSaveAsTemplate(nodes, edges, name, category, description)
            }
            setShowSaveModal(false)
          }}
          onCancel={() => setShowSaveModal(false)}
          currentWorkflow={currentWorkflow}
        />
      )}
    </div>
  )
}) // End of memo

export default WorkflowBuilder