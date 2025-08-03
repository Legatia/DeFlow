import { useCallback, useState, useMemo } from 'react'
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
} from 'reactflow'
import 'reactflow/dist/style.css'

import { NODE_TYPES, NodeType } from '../types/nodes'
import WorkflowNode from './WorkflowNode'
import NodePalette from './NodePalette'
import NodeConfigPanel from './NodeConfigPanel'

interface WorkflowBuilderProps {
  initialNodes?: Node[]
  initialEdges?: Edge[]
  onSave?: (nodes: Node[], edges: Edge[]) => void
  readOnly?: boolean
}

const WorkflowBuilder = ({ 
  initialNodes = [], 
  initialEdges = [], 
  onSave,
  readOnly = false 
}: WorkflowBuilderProps) => {
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes)
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges)
  const [selectedNode, setSelectedNode] = useState<Node | null>(null)
  const [isConfigPanelOpen, setIsConfigPanelOpen] = useState(false)

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

  // Handle dropping new nodes from palette
  const onDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault()

      const nodeTypeId = event.dataTransfer.getData('application/reactflow')
      const nodeType = NODE_TYPES.find(nt => nt.id === nodeTypeId)
      
      if (!nodeType) return

      // Get the position where the node was dropped
      const reactFlowBounds = event.currentTarget.getBoundingClientRect()
      const position = {
        x: event.clientX - reactFlowBounds.left - 100, // Offset for node width
        y: event.clientY - reactFlowBounds.top - 25,   // Offset for node height
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

      setNodes((nds) => nds.concat(newNode))
    },
    [setNodes]
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

  // Save workflow
  const handleSave = useCallback(() => {
    if (onSave) {
      onSave(nodes, edges)
    }
  }, [nodes, edges, onSave])

  // Clear workflow
  const handleClear = useCallback(() => {
    setNodes([])
    setEdges([])
    setSelectedNode(null)
    setIsConfigPanelOpen(false)
  }, [setNodes, setEdges])

  return (
    <div className="h-full w-full flex">
      {/* Node Palette - Left Sidebar */}
      {!readOnly && (
        <div className="w-64 bg-gray-50 border-r border-gray-200 flex flex-col">
          <div className="p-4 border-b border-gray-200">
            <h3 className="text-lg font-medium text-gray-900">Node Palette</h3>
            <p className="text-sm text-gray-600 mt-1">Drag nodes to canvas</p>
          </div>
          <div className="flex-1 overflow-y-auto">
            <NodePalette />
          </div>
        </div>
      )}

      {/* Main Canvas Area */}
      <div className="flex-1 relative">
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          onNodeClick={onNodeClick}
          onDrop={onDrop}
          onDragOver={onDragOver}
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
          <MiniMap 
            nodeStrokeColor="#374151"
            nodeColor="#f3f4f6"
            nodeBorderRadius={8}
            maskColor="rgba(0, 0, 0, 0.1)"
          />
          <Background 
            variant={BackgroundVariant.Dots} 
            gap={20} 
            size={1} 
            color="#e5e7eb"
          />
          
          {/* Top Panel with Actions */}
          {!readOnly && (
            <Panel position="top-right" className="space-x-2">
              <button
                onClick={handleClear}
                className="px-3 py-1 bg-red-600 text-white text-sm rounded hover:bg-red-700 transition-colors"
              >
                Clear
              </button>
              <button
                onClick={handleSave}
                className="px-3 py-1 bg-blue-600 text-white text-sm rounded hover:bg-blue-700 transition-colors"
              >
                Save
              </button>
            </Panel>
          )}

          {/* Info Panel */}
          <Panel position="top-left">
            <div className="bg-white bg-opacity-90 backdrop-blur-sm rounded-lg p-3 shadow-sm">
              <div className="text-sm text-gray-600">
                <div>Nodes: {nodes.length}</div>
                <div>Connections: {edges.length}</div>
                {readOnly && <div className="text-blue-600 font-medium">Read Only</div>}
              </div>
            </div>
          </Panel>
        </ReactFlow>
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
    </div>
  )
}

export default WorkflowBuilder