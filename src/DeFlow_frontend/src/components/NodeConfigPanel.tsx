import { useState, useEffect } from 'react'
import { Node } from 'reactflow'
import { NodeType, ConfigField } from '../types/nodes'

interface NodeConfigPanelProps {
  node: Node
  onConfigChange: (nodeId: string, config: Record<string, any>) => void
  onDelete: (nodeId: string) => void
}

const NodeConfigPanel = ({ node, onConfigChange, onDelete }: NodeConfigPanelProps) => {
  const nodeType: NodeType = node.data.nodeType
  const [config, setConfig] = useState(node.data.config || {})
  const [errors, setErrors] = useState<Record<string, string>>({})

  // Update local config when node changes
  useEffect(() => {
    setConfig(node.data.config || {})
  }, [node.data.config])

  // Validate a single field
  const validateField = (field: ConfigField, value: any): string | null => {
    if (field.required && (!value || value === '')) {
      return `${field.name} is required`
    }

    if (field.validation) {
      const { pattern, min, max, minLength, maxLength } = field.validation

      if (pattern && typeof value === 'string') {
        const regex = new RegExp(pattern)
        if (!regex.test(value)) {
          return `${field.name} format is invalid`
        }
      }

      if (field.type === 'number' && typeof value === 'number') {
        if (min !== undefined && value < min) {
          return `${field.name} must be at least ${min}`
        }
        if (max !== undefined && value > max) {
          return `${field.name} must be at most ${max}`
        }
      }

      if (typeof value === 'string') {
        if (minLength !== undefined && value.length < minLength) {
          return `${field.name} must be at least ${minLength} characters`
        }
        if (maxLength !== undefined && value.length > maxLength) {
          return `${field.name} must be at most ${maxLength} characters`
        }
      }
    }

    // Type-specific validation
    if (field.type === 'email' && value) {
      const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
      if (!emailRegex.test(value)) {
        return `${field.name} must be a valid email address`
      }
    }

    if (field.type === 'url' && value) {
      try {
        new URL(value)
      } catch {
        return `${field.name} must be a valid URL`
      }
    }

    return null
  }

  // Handle field value changes
  const handleFieldChange = (field: ConfigField, value: any) => {
    const newConfig = { ...config, [field.key]: value }
    setConfig(newConfig)

    // Validate the field
    const error = validateField(field, value)
    const newErrors = { ...errors }
    if (error) {
      newErrors[field.key] = error
    } else {
      delete newErrors[field.key]
    }
    setErrors(newErrors)

    // Update the node immediately
    onConfigChange(node.id, newConfig)
  }

  // Render form field based on type
  const renderField = (field: ConfigField) => {
    const value = config[field.key] ?? field.defaultValue ?? ''
    const error = errors[field.key]
    const fieldId = `${node.id}-${field.key}`

    const baseInputClass = `
      w-full px-3 py-2 border rounded-lg text-sm bg-slate-700/60 text-slate-100 placeholder-slate-300
      focus:ring-2 focus:ring-cyan-400/50 focus:border-cyan-400/50 backdrop-blur-sm transition-all duration-200
      ${error ? 'border-red-400/60' : 'border-slate-500/40'}
    `

    switch (field.type) {
      case 'text':
      case 'email':
      case 'url':
      case 'password':
        return (
          <input
            id={fieldId}
            type={field.type}
            value={value}
            placeholder={field.placeholder}
            onChange={(e) => handleFieldChange(field, e.target.value)}
            className={baseInputClass}
          />
        )

      case 'number':
        return (
          <input
            id={fieldId}
            type="number"
            value={value}
            placeholder={field.placeholder}
            min={field.validation?.min}
            max={field.validation?.max}
            onChange={(e) => handleFieldChange(field, parseFloat(e.target.value) || 0)}
            className={baseInputClass}
          />
        )

      case 'boolean':
        return (
          <label className="flex items-center space-x-2 cursor-pointer">
            <input
              id={fieldId}
              type="checkbox"
              checked={value}
              onChange={(e) => handleFieldChange(field, e.target.checked)}
              className="w-4 h-4 text-cyan-400 bg-slate-700 border-slate-500 rounded focus:ring-cyan-400"
            />
            <span className="text-sm text-slate-200">
              {field.description || `Enable ${field.name}`}
            </span>
          </label>
        )

      case 'select':
        return (
          <select
            id={fieldId}
            value={value}
            onChange={(e) => handleFieldChange(field, e.target.value)}
            className={baseInputClass}
          >
            <option value="">Select {field.name}</option>
            {field.options?.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        )

      case 'textarea':
        return (
          <textarea
            id={fieldId}
            value={value}
            placeholder={field.placeholder}
            rows={4}
            onChange={(e) => handleFieldChange(field, e.target.value)}
            className={baseInputClass}
          />
        )

      default:
        return (
          <input
            id={fieldId}
            type="text"
            value={value}
            placeholder={field.placeholder}
            onChange={(e) => handleFieldChange(field, e.target.value)}
            className={baseInputClass}
          />
        )
    }
  }

  return (
    <div className="p-4 space-y-6">
      {/* Node Info */}
      <div className="flex items-center space-x-3">
        <div 
          className="w-12 h-12 rounded-lg flex items-center justify-center text-white text-xl"
          style={{ backgroundColor: nodeType.color }}
        >
          {nodeType.icon}
        </div>
        <div>
          <h4 className="font-medium text-slate-100">{nodeType.name}</h4>
          <p className="text-sm text-slate-300">{nodeType.description}</p>
        </div>
      </div>

      {/* Configuration Form */}
      <div className="space-y-4">
        <h5 className="font-medium text-slate-100">Configuration</h5>
        
        {nodeType.configSchema.length === 0 ? (
          <div className="text-sm text-slate-400 py-4 text-center">
            This node doesn't require any configuration.
          </div>
        ) : (
          <div className="space-y-4">
            {nodeType.configSchema.map((field) => (
              <div key={field.key} className="space-y-2">
                <label 
                  htmlFor={`${node.id}-${field.key}`}
                  className="block text-sm font-medium text-slate-200"
                >
                  {field.name}
                  {field.required && <span className="text-red-400 ml-1">*</span>}
                </label>
                
                {renderField(field)}
                
                {field.description && field.type !== 'boolean' && (
                  <p className="text-xs text-slate-400">{field.description}</p>
                )}
                
                {errors[field.key] && (
                  <p className="text-xs text-red-400">{errors[field.key]}</p>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Node Details */}
      <div className="space-y-4 pt-4 border-t border-slate-600/50">
        <h5 className="font-medium text-slate-100">Node Details</h5>
        
        <div className="space-y-2 text-sm">
          <div className="flex justify-between">
            <span className="text-slate-300">Category:</span>
            <span className="font-medium capitalize text-slate-100">{nodeType.category}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-slate-300">Inputs:</span>
            <span className="font-medium text-slate-100">{nodeType.inputs.length}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-slate-300">Outputs:</span>
            <span className="font-medium text-slate-100">{nodeType.outputs.length}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-slate-300">Node ID:</span>
            <span className="font-mono text-xs text-slate-200">{node.id}</span>
          </div>
        </div>

        {/* Input/Output Details */}
        {nodeType.inputs.length > 0 && (
          <div className="space-y-2">
            <h6 className="text-sm font-medium text-slate-200">Inputs</h6>
            <div className="space-y-1">
              {nodeType.inputs.map((input) => (
                <div key={input.id} className="flex items-center space-x-2 text-xs">
                  <div 
                    className="w-2 h-2 rounded-full"
                    style={{ backgroundColor: getPortColor(input.type) }}
                  />
                  <span className="font-medium text-slate-200">{input.name}</span>
                  <span className="text-slate-400">({input.type})</span>
                  {input.required && <span className="text-red-400">*</span>}
                </div>
              ))}
            </div>
          </div>
        )}

        {nodeType.outputs.length > 0 && (
          <div className="space-y-2">
            <h6 className="text-sm font-medium text-slate-200">Outputs</h6>
            <div className="space-y-1">
              {nodeType.outputs.map((output) => (
                <div key={output.id} className="flex items-center space-x-2 text-xs">
                  <div 
                    className="w-2 h-2 rounded-full"
                    style={{ backgroundColor: getPortColor(output.type) }}
                  />
                  <span className="font-medium text-slate-200">{output.name}</span>
                  <span className="text-slate-400">({output.type})</span>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Actions */}
      <div className="pt-4 border-t border-slate-600/50">
        <button
          onClick={() => onDelete(node.id)}
          className="w-full px-4 py-2 bg-red-500/80 backdrop-blur-sm text-white text-sm rounded-lg hover:bg-red-500 transition-all duration-200 border border-red-400/30"
        >
          Delete Node
        </button>
      </div>
    </div>
  )
}

// Helper function to get port colors
function getPortColor(type: string): string {
  const colors = {
    data: '#6b7280',
    trigger: '#3b82f6',
    condition: '#f59e0b',
    webhook: '#10b981',
    email: '#ef4444',
    api: '#8b5cf6',
    file: '#f97316',
  }
  return colors[type as keyof typeof colors] || '#6b7280'
}

export default NodeConfigPanel