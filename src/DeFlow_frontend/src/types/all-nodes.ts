// Combined node types - includes both base nodes and DeFi nodes
import { NODE_TYPES, NodeType } from './nodes'
import { DEFI_NODE_TYPES } from './defi-nodes'

// Export all node types combined
export const ALL_NODE_TYPES: NodeType[] = [...NODE_TYPES, ...DEFI_NODE_TYPES]

// Export function for consistency with previous API
export function getAllNodeTypes(): NodeType[] {
  return ALL_NODE_TYPES
}

// Re-export other types for convenience
export type { NodeType, NodeCategory } from './nodes'