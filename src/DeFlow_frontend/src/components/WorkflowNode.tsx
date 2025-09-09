import { memo } from 'react'
import { Handle, Position, NodeProps } from 'reactflow'
import { NodeType } from '../types/nodes'
import { cn } from '../lib/utils'

interface WorkflowNodeData {
  nodeType: NodeType
  config: Record<string, any>
  isValid?: boolean
  errors?: string[]
}

const WorkflowNode = memo(({ data, selected }: NodeProps<WorkflowNodeData>) => {
  const { nodeType, config, isValid = true, errors = [] } = data

  // Get category-based styling
  const getCategoryStyle = (category: string) => {
    switch (category) {
      case 'triggers':
        return {
          gradient: 'from-blue-500/20 to-cyan-500/20',
          borderColor: 'border-blue-400/40',
          glowColor: '#3b82f6'
        }
      case 'actions':
        return {
          gradient: 'from-emerald-500/20 to-green-500/20',
          borderColor: 'border-emerald-400/40',
          glowColor: '#10b981'
        }
      case 'conditions':
        return {
          gradient: 'from-amber-500/20 to-orange-500/20',
          borderColor: 'border-amber-400/40',
          glowColor: '#f59e0b'
        }
      case 'integrations':
        return {
          gradient: 'from-purple-500/20 to-pink-500/20',
          borderColor: 'border-purple-400/40',
          glowColor: '#8b5cf6'
        }
      case 'transformations':
        return {
          gradient: 'from-teal-500/20 to-cyan-500/20',
          borderColor: 'border-teal-400/40',
          glowColor: '#14b8a6'
        }
      default:
        return {
          gradient: 'from-gray-500/20 to-slate-500/20',
          borderColor: 'border-gray-400/40',
          glowColor: '#6b7280'
        }
    }
  }

  const categoryStyle = getCategoryStyle(nodeType.category)
  const glowStyle = selected 
    ? { boxShadow: `0 0 20px ${categoryStyle.glowColor}33, 0 0 40px ${categoryStyle.glowColor}22, 0 0 60px ${categoryStyle.glowColor}11` }
    : {}

  return (
    <div
      className={cn(
        // Base liquid glass styles
        'relative rounded-xl backdrop-blur-md min-w-[180px] max-w-[220px]',
        'border transition-all duration-300 ease-out',
        'bg-gradient-to-br',
        categoryStyle.gradient,
        
        // Border and glow effects
        selected ? 'border-white/30 shadow-2xl' : cn(categoryStyle.borderColor, 'hover:border-white/20'),
        !isValid && 'border-red-400/60 shadow-red-500/25',
        
        // Liquid animation effects
        'before:absolute before:inset-0 before:rounded-xl before:bg-gradient-to-r',
        'before:from-transparent before:via-white/10 before:to-transparent',
        'before:translate-x-[-100%] before:transition-transform before:duration-700',
        'hover:before:translate-x-[100%]',
        
        // Glass reflection effect
        'after:absolute after:inset-[1px] after:rounded-[11px]',
        'after:bg-gradient-to-b after:from-white/20 after:to-transparent after:opacity-60',
        
        // Transform effects
        'hover:scale-105 hover:rotate-1 hover:shadow-xl',
        selected && 'scale-105 rotate-1'
      )}
      style={glowStyle}
    >
      {/* Input Handles - Enhanced with liquid glass */}
      {nodeType.inputs.map((input, index) => (
        <Handle
          key={input.id}
          type="target"
          position={Position.Left}
          id={input.id}
          style={{
            top: nodeType.inputs.length === 1 ? '50%' : `${((index + 1) / (nodeType.inputs.length + 1)) * 100}%`,
            background: getPortColor(input.type),
            border: '2px solid rgba(255, 255, 255, 0.3)',
            width: '14px',
            height: '14px',
            borderRadius: '50%',
            backdropFilter: 'blur(4px)',
            boxShadow: `0 0 10px ${getPortColor(input.type)}33`,
          }}
          className="transition-all duration-300 hover:scale-125 hover:shadow-lg hover:border-white/60"
        />
      ))}

      {/* Output Handles - Enhanced with liquid glass */}
      {nodeType.outputs.map((output, index) => (
        <Handle
          key={output.id}
          type="source"
          position={Position.Right}
          id={output.id}
          style={{
            top: nodeType.outputs.length === 1 ? '50%' : `${((index + 1) / (nodeType.outputs.length + 1)) * 100}%`,
            background: getPortColor(output.type),
            border: '2px solid rgba(255, 255, 255, 0.3)',
            width: '14px',
            height: '14px',
            borderRadius: '50%',
            backdropFilter: 'blur(4px)',
            boxShadow: `0 0 10px ${getPortColor(output.type)}33`,
          }}
          className="transition-all duration-300 hover:scale-125 hover:shadow-lg hover:border-white/60"
        />
      ))}

      {/* Node Header - Enhanced with liquid glass */}
      <div className="relative z-10 px-4 py-3 rounded-t-xl">
        <div 
          className="absolute inset-0 rounded-t-xl opacity-80 backdrop-blur-sm"
          style={{ 
            background: `linear-gradient(135deg, ${nodeType.color}80, ${nodeType.color}60)`,
          }}
        />
        <div className="relative flex items-center space-x-2">
          <div className="flex-shrink-0 w-8 h-8 rounded-lg bg-white/20 backdrop-blur-sm flex items-center justify-center">
            <span className="text-lg filter drop-shadow-sm">{nodeType.icon}</span>
          </div>
          <span className="truncate text-white text-sm font-semibold filter drop-shadow-sm">
            {nodeType.name}
          </span>
        </div>
      </div>

      {/* Node Body - Enhanced with liquid glass */}
      <div className="relative z-10 px-4 py-3 rounded-b-xl">
        <div className="text-xs text-gray-700 mb-2 truncate font-medium">
          {nodeType.description}
        </div>
        
        {/* Show key configuration */}
        {config && Object.keys(config).length > 0 && (
          <div className="text-xs text-gray-600 space-y-1 bg-white/10 rounded-lg p-2 backdrop-blur-sm">
            {Object.entries(config).slice(0, 2).map(([key, value]) => (
              <div key={key} className="truncate">
                <span className="font-semibold text-gray-700">{key}:</span>{' '}
                <span className="text-gray-600">{formatConfigValue(value)}</span>
              </div>
            ))}
            {Object.keys(config).length > 2 && (
              <div className="text-gray-500 font-medium">
                +{Object.keys(config).length - 2} more...
              </div>
            )}
          </div>
        )}

        {/* Error indicators */}
        {!isValid && errors.length > 0 && (
          <div className="mt-2 text-xs bg-red-500/20 border border-red-400/40 rounded-lg p-2 backdrop-blur-sm">
            <div className="flex items-center space-x-1 text-red-700">
              <span>⚠️</span>
              <span className="font-semibold">{errors.length} error{errors.length > 1 ? 's' : ''}</span>
            </div>
          </div>
        )}
      </div>

      {/* Category badge - Enhanced with liquid glass */}
      <div className="absolute -top-2 -right-2 z-20">
        <div className="bg-white/20 backdrop-blur-md text-gray-700 text-xs px-2 py-1 rounded-full border border-white/30 shadow-lg">
          {nodeType.category}
        </div>
      </div>

      {/* Enhanced connection indicators */}
      <div className="absolute -bottom-1 left-1/2 transform -translate-x-1/2 flex items-center space-x-2 text-xs z-20">
        {nodeType.inputs.length > 0 && (
          <div className="flex items-center space-x-1 bg-blue-500/20 backdrop-blur-sm rounded-full px-2 py-0.5 border border-blue-400/30">
            <div className="w-1.5 h-1.5 rounded-full bg-blue-400"></div>
            <span className="text-blue-700 font-medium">{nodeType.inputs.length}</span>
          </div>
        )}
        {nodeType.outputs.length > 0 && (
          <div className="flex items-center space-x-1 bg-green-500/20 backdrop-blur-sm rounded-full px-2 py-0.5 border border-green-400/30">
            <div className="w-1.5 h-1.5 rounded-full bg-green-400"></div>
            <span className="text-green-700 font-medium">{nodeType.outputs.length}</span>
          </div>
        )}
      </div>

      {/* Pulse effect for selected state */}
      {selected && (
        <div className="absolute inset-0 rounded-xl pointer-events-none animate-pulse">
          <div className="absolute inset-0 rounded-xl border-2 border-white/50 shadow-lg" />
        </div>
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