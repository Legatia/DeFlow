import { memo } from 'react'
import { Handle, Position, NodeProps } from 'reactflow'
import { NodeType } from '../types/nodes'

interface WorkflowNodeData {
  nodeType: NodeType
  config: Record<string, any>
  isValid?: boolean
  errors?: string[]
}

const WorkflowNode = memo(({ data, selected }: NodeProps<WorkflowNodeData>) => {
  const { nodeType, config, isValid = true, errors = [] } = data

  return (
    <div
      className={`
        relative bg-white rounded-lg shadow-md border-2 min-w-[150px] max-w-[200px]
        ${selected ? 'border-blue-500 shadow-lg' : 'border-gray-200 hover:border-gray-300'}
        ${!isValid ? 'border-red-500' : ''}
        transition-all duration-200
      `}
    >
      {/* Input Handles */}
      {nodeType.inputs.map((input, index) => (
        <Handle
          key={input.id}
          type="target"
          position={Position.Left}
          id={input.id}
          style={{
            top: nodeType.inputs.length === 1 ? '50%' : `${((index + 1) / (nodeType.inputs.length + 1)) * 100}%`,
            background: getPortColor(input.type),
            border: '2px solid white',
            width: '12px',
            height: '12px',
          }}
          className="transition-colors hover:bg-blue-500"
        />
      ))}

      {/* Output Handles */}
      {nodeType.outputs.map((output, index) => (
        <Handle
          key={output.id}
          type="source"
          position={Position.Right}
          id={output.id}
          style={{
            top: nodeType.outputs.length === 1 ? '50%' : `${((index + 1) / (nodeType.outputs.length + 1)) * 100}%`,
            background: getPortColor(output.type),
            border: '2px solid white',
            width: '12px',
            height: '12px',
          }}
          className="transition-colors hover:bg-blue-500"
        />
      ))}

      {/* Node Header */}
      <div 
        className="px-3 py-2 rounded-t-lg text-white text-sm font-medium flex items-center space-x-2"
        style={{ backgroundColor: nodeType.color }}
      >
        <span className="text-lg">{nodeType.icon}</span>
        <span className="truncate">{nodeType.name}</span>
      </div>

      {/* Node Body */}
      <div className="px-3 py-2">
        <div className="text-xs text-gray-600 mb-1 truncate">
          {nodeType.description}
        </div>
        
        {/* Show key configuration */}
        {config && Object.keys(config).length > 0 && (
          <div className="text-xs text-gray-500 space-y-1">
            {Object.entries(config).slice(0, 2).map(([key, value]) => (
              <div key={key} className="truncate">
                <span className="font-medium">{key}:</span>{' '}
                <span>{formatConfigValue(value)}</span>
              </div>
            ))}
            {Object.keys(config).length > 2 && (
              <div className="text-gray-400">
                +{Object.keys(config).length - 2} more...
              </div>
            )}
          </div>
        )}

        {/* Error indicators */}
        {!isValid && errors.length > 0 && (
          <div className="mt-2 text-xs text-red-600">
            <div className="flex items-center space-x-1">
              <span>⚠️</span>
              <span>{errors.length} error{errors.length > 1 ? 's' : ''}</span>
            </div>
          </div>
        )}
      </div>

      {/* Category badge */}
      <div className="absolute -top-2 -right-2">
        <div className="bg-gray-100 text-gray-600 text-xs px-2 py-1 rounded-full border border-gray-200">
          {nodeType.category}
        </div>
      </div>

      {/* Selection indicator */}
      {selected && (
        <div className="absolute inset-0 border-2 border-blue-500 rounded-lg pointer-events-none animate-pulse" />
      )}
    </div>
  )
})

// Helper function to get port colors based on type
function getPortColor(type: string): string {
  const colors = {
    data: '#6b7280',       // gray
    trigger: '#3b82f6',    // blue
    condition: '#f59e0b',  // amber
    webhook: '#10b981',    // green
    email: '#ef4444',      // red
    api: '#8b5cf6',        // violet
    file: '#f97316',       // orange
  }
  return colors[type as keyof typeof colors] || '#6b7280'
}

// Helper function to format config values for display
function formatConfigValue(value: any): string {
  if (typeof value === 'string') {
    return value.length > 20 ? value.substring(0, 20) + '...' : value
  }
  if (typeof value === 'boolean') {
    return value ? 'Yes' : 'No'
  }
  if (typeof value === 'number') {
    return value.toString()
  }
  if (typeof value === 'object') {
    return '[Object]'
  }
  return String(value)
}

WorkflowNode.displayName = 'WorkflowNode'

export default WorkflowNode