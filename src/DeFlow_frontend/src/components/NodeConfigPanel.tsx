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
      w-full px-3 py-2 border rounded-md text-sm
      focus:ring-2 focus:ring-blue-500 focus:border-transparent
      ${error ? 'border-red-300' : 'border-gray-300'}
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
              className="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
            />
            <span className="text-sm text-gray-700">
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
          <h4 className="font-medium text-gray-900">{nodeType.name}</h4>
          <p className="text-sm text-gray-600">{nodeType.description}</p>
        </div>
      </div>

      {/* Configuration Form */}
      <div className="space-y-4">
        <h5 className="font-medium text-gray-900">Configuration</h5>
        
        {nodeType.configSchema.length === 0 ? (
          <div className="text-sm text-gray-500 py-4 text-center">
            This node doesn't require any configuration.
          </div>
        ) : (
          <div className="space-y-4">
            {nodeType.configSchema.map((field) => (
              <div key={field.key} className="space-y-2">
                <label 
                  htmlFor={`${node.id}-${field.key}`}
                  className="block text-sm font-medium text-gray-700"
                >
                  {field.name}
                  {field.required && <span className="text-red-500 ml-1">*</span>}
                </label>
                
                {renderField(field)}
                
                {field.description && field.type !== 'boolean' && (
                  <p className="text-xs text-gray-500">{field.description}</p>
                )}
                
                {errors[field.key] && (
                  <p className="text-xs text-red-600">{errors[field.key]}</p>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Node Details */}
      <div className="space-y-4 pt-4 border-t border-gray-200">
        <h5 className="font-medium text-gray-900">Node Details</h5>
        
        <div className="space-y-2 text-sm">
          <div className="flex justify-between">
            <span className="text-gray-600">Category:</span>
            <span className="font-medium capitalize">{nodeType.category}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-600">Inputs:</span>
            <span className="font-medium">{nodeType.inputs.length}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-600">Outputs:</span>
            <span className="font-medium">{nodeType.outputs.length}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-600">Node ID:</span>
            <span className="font-mono text-xs">{node.id}</span>
          </div>
        </div>

        {/* Input/Output Details */}
        {nodeType.inputs.length > 0 && (
          <div className="space-y-2">
            <h6 className="text-sm font-medium text-gray-700">Inputs</h6>
            <div className="space-y-1">
              {nodeType.inputs.map((input) => (
                <div key={input.id} className="flex items-center space-x-2 text-xs">
                  <div 
                    className="w-2 h-2 rounded-full"
                    style={{ backgroundColor: getPortColor(input.type) }}
                  />
                  <span className="font-medium">{input.name}</span>
                  <span className="text-gray-500">({input.type})</span>
                  {input.required && <span className="text-red-500">*</span>}
                </div>
              ))}
            </div>
          </div>
        )}

        {nodeType.outputs.length > 0 && (
          <div className="space-y-2">
            <h6 className="text-sm font-medium text-gray-700">Outputs</h6>
            <div className="space-y-1">
              {nodeType.outputs.map((output) => (
                <div key={output.id} className="flex items-center space-x-2 text-xs">
                  <div 
                    className="w-2 h-2 rounded-full"
                    style={{ backgroundColor: getPortColor(output.type) }}
                  />
                  <span className="font-medium">{output.name}</span>
                  <span className="text-gray-500">({output.type})</span>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Actions */}
      <div className="pt-4 border-t border-gray-200">
        <button
          onClick={() => onDelete(node.id)}
          className="w-full px-4 py-2 bg-red-600 text-white text-sm rounded-md hover:bg-red-700 transition-colors"
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