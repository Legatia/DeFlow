/**
 * Workflow Converter - Handles conversion between ReactFlow and Backend formats
 *
 * Critical for proper data synchronization between frontend canvas and backend execution
 */

import { Node, Edge } from 'reactflow'
import { WorkflowNode, NodeConnection } from '../types'

/**
 * ConfigValue type matching backend Rust enum
 */
export type ConfigValue =
  | { String: string }
  | { Number: number }
  | { Boolean: boolean }
  | { Array: ConfigValue[] }
  | { Object: Record<string, ConfigValue> }

/**
 * Convert ReactFlow nodes and edges to backend Workflow format
 */
export function reactFlowToBackend(nodes: Node[], edges: Edge[]) {
  const workflowNodes: WorkflowNode[] = nodes.map(node => ({
    id: node.id,
    node_type: node.data.nodeType?.id || node.data.nodeType || 'unknown',
    position: node.position,
    configuration: {
      parameters: toConfigValues(node.data.config || {})
    },
    metadata: {
      label: node.data.nodeType?.name || node.data.label || 'Unnamed Node',
      description: node.data.nodeType?.description || node.data.description,
      version: '1.0.0',
      tags: [],
      icon: node.data.nodeType?.icon,
      color: node.data.nodeType?.color
    }
  }))

  const connections: NodeConnection[] = edges.map(edge => ({
    id: edge.id,
    source_node_id: edge.source,
    target_node_id: edge.target,
    source_output: edge.sourceHandle || 'output',
    target_input: edge.targetHandle || 'input'
  }))

  return { nodes: workflowNodes, connections }
}

/**
 * Convert backend Workflow format to ReactFlow nodes and edges
 */
export function backendToReactFlow(
  workflowNodes: WorkflowNode[],
  connections: NodeConnection[],
  nodeTypeRegistry?: Map<string, any>  // Optional registry to restore full node type info
) {
  const nodes: Node[] = workflowNodes.map(node => ({
    id: node.id,
    type: 'workflowNode',
    position: node.position,
    data: {
      nodeType: nodeTypeRegistry?.get(node.node_type) || {
        id: node.node_type,
        name: node.metadata.label,
        description: node.metadata.description || '',
        category: 'actions' as any,
        icon: '⚡',
        color: '#10b981',
        inputs: [],
        outputs: [],
        configSchema: [],
        defaultConfig: {}
      },
      config: fromConfigValues((node.configuration as any).parameters || {}),
      isValid: true,
      errors: []
    }
  }))

  const edges: Edge[] = connections.map(conn => ({
    id: conn.id,
    source: conn.source_node_id,
    target: conn.target_node_id,
    sourceHandle: conn.source_output,
    targetHandle: conn.target_input,
    type: 'smoothstep'
  }))

  return { nodes, edges }
}

/**
 * Convert plain JavaScript values to backend ConfigValue enum format
 */
export function toConfigValues(config: Record<string, any>): Record<string, ConfigValue> {
  const result: Record<string, ConfigValue> = {}
  for (const [key, value] of Object.entries(config)) {
    result[key] = wrapConfigValue(value)
  }
  return result
}

/**
 * Wrap a single value in ConfigValue enum
 */
function wrapConfigValue(value: any): ConfigValue {
  // Handle null/undefined
  if (value === null || value === undefined) {
    return { String: '' }
  }

  // Handle primitives
  if (typeof value === 'string') {
    return { String: value }
  }
  if (typeof value === 'number') {
    return { Number: value }
  }
  if (typeof value === 'boolean') {
    return { Boolean: value }
  }

  // Handle arrays
  if (Array.isArray(value)) {
    return { Array: value.map(wrapConfigValue) }
  }

  // Handle objects
  if (typeof value === 'object') {
    const wrapped: Record<string, ConfigValue> = {}
    for (const [k, v] of Object.entries(value)) {
      wrapped[k] = wrapConfigValue(v)
    }
    return { Object: wrapped }
  }

  // Fallback: convert to string
  return { String: String(value) }
}

/**
 * Convert backend ConfigValue enum format to plain JavaScript values
 */
export function fromConfigValues(params: Record<string, any>): Record<string, any> {
  const result: Record<string, any> = {}
  for (const [key, value] of Object.entries(params)) {
    result[key] = unwrapConfigValue(value)
  }
  return result
}

/**
 * Unwrap a ConfigValue enum to plain JavaScript value
 */
function unwrapConfigValue(value: any): any {
  // Already unwrapped or invalid
  if (!value || typeof value !== 'object') {
    return value
  }

  // Check for ConfigValue enum variants
  if ('String' in value) return value.String
  if ('Number' in value) return value.Number
  if ('Boolean' in value) return value.Boolean
  if ('Array' in value) {
    return Array.isArray(value.Array)
      ? value.Array.map(unwrapConfigValue)
      : value.Array
  }
  if ('Object' in value) {
    return fromConfigValues(value.Object)
  }

  // Fallback: return as-is
  return value
}

/**
 * Validate that a workflow has all required fields for backend
 */
export function validateWorkflowForBackend(nodes: Node[], edges: Edge[]): {
  valid: boolean
  errors: string[]
} {
  const errors: string[] = []

  // Check for nodes
  if (nodes.length === 0) {
    errors.push('Workflow must have at least one node')
  }

  // Validate each node
  nodes.forEach((node, index) => {
    if (!node.id) {
      errors.push(`Node ${index} missing id`)
    }
    if (!node.data?.nodeType?.id) {
      errors.push(`Node ${node.id || index} missing node_type`)
    }
    if (!node.position) {
      errors.push(`Node ${node.id || index} missing position`)
    }
  })

  // Validate connections
  edges.forEach((edge, index) => {
    if (!edge.source || !edge.target) {
      errors.push(`Connection ${index} missing source or target`)
    }
    // Check that source and target nodes exist
    if (!nodes.find(n => n.id === edge.source)) {
      errors.push(`Connection ${edge.id} references non-existent source node ${edge.source}`)
    }
    if (!nodes.find(n => n.id === edge.target)) {
      errors.push(`Connection ${edge.id} references non-existent target node ${edge.target}`)
    }
  })

  return {
    valid: errors.length === 0,
    errors
  }
}

/**
 * Debug helper: print conversion comparison
 */
export function debugConversion(nodes: Node[], edges: Edge[]) {
  console.group('🔍 Workflow Conversion Debug')

  console.log('📥 Input (ReactFlow format):')
  console.log('Nodes:', nodes.length)
  console.log('Edges:', edges.length)

  const { nodes: backendNodes, connections } = reactFlowToBackend(nodes, edges)

  console.log('📤 Output (Backend format):')
  console.log('Backend Nodes:', backendNodes)
  console.log('Connections:', connections)

  const validation = validateWorkflowForBackend(nodes, edges)
  console.log('✅ Validation:', validation)

  console.groupEnd()

  return validation
}
