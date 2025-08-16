import { SubscriptionTier, NodeType } from '../types/nodes'

// Define tier hierarchy (higher tiers include all lower tier access)
const TIER_HIERARCHY: Record<SubscriptionTier, number> = {
  'standard': 1,
  'premium': 2,
  'pro': 3
}

/**
 * Check if user's subscription tier allows access to a node that requires a specific tier
 */
export function canAccessNode(userTier: SubscriptionTier, requiredTier: SubscriptionTier): boolean {
  return TIER_HIERARCHY[userTier] >= TIER_HIERARCHY[requiredTier]
}

/**
 * Universal access control for any node type
 * This is the main function that should be used everywhere for feature gating
 */
export function canAccessNodeType(userTier: SubscriptionTier, nodeType: NodeType): boolean {
  const requiredTier = nodeType.requiredTier || 'standard'
  return canAccessNode(userTier, requiredTier)
}

/**
 * Check if a user can perform drag & drop operations on a node
 */
export function canDragNode(userTier: SubscriptionTier, nodeType: NodeType): boolean {
  return canAccessNodeType(userTier, nodeType)
}

/**
 * Check if a user can add a node to their workflow
 */
export function canAddNodeToWorkflow(userTier: SubscriptionTier, nodeType: NodeType): boolean {
  return canAccessNodeType(userTier, nodeType)
}

/**
 * Check if a user can execute/run workflows containing specific node types
 */
export function canExecuteWorkflowWithNodes(userTier: SubscriptionTier, nodeTypes: NodeType[]): {
  canExecute: boolean
  restrictedNodes: NodeType[]
  requiredUpgrade?: SubscriptionTier
} {
  const restrictedNodes = nodeTypes.filter(nodeType => !canAccessNodeType(userTier, nodeType))
  
  if (restrictedNodes.length === 0) {
    return { canExecute: true, restrictedNodes: [] }
  }

  // Find the minimum tier needed to access all restricted nodes
  const maxRequiredTier = restrictedNodes.reduce((maxTier, node) => {
    const nodeTier = node.requiredTier || 'standard'
    return TIER_HIERARCHY[nodeTier] > TIER_HIERARCHY[maxTier] ? nodeTier : maxTier
  }, 'standard' as SubscriptionTier)

  return {
    canExecute: false,
    restrictedNodes,
    requiredUpgrade: maxRequiredTier
  }
}

/**
 * Get subscription tier display information
 */
export function getSubscriptionTierInfo(tier: SubscriptionTier) {
  const tierInfo = {
    'standard': {
      name: 'Standard',
      price: '$0',
      color: '#6b7280',
      features: ['Telegram & Discord nodes', 'Basic workflow automation', 'Community support']
    },
    'premium': {
      name: 'Premium', 
      price: '$19',
      color: '#3b82f6',
      features: ['All Standard features', 'Full social media integrations', 'Priority execution', 'Email support']
    },
    'pro': {
      name: 'Pro',
      price: '$149', 
      color: '#7c3aed',
      features: ['All Premium features', 'Advanced analytics', 'API access', 'Priority support']
    }
  }
  
  return tierInfo[tier]
}

/**
 * Get upgrade path for accessing a restricted node
 */
export function getUpgradePath(userTier: SubscriptionTier, requiredTier: SubscriptionTier) {
  if (canAccessNode(userTier, requiredTier)) {
    return null // No upgrade needed
  }
  
  const upgradeTo = requiredTier === 'premium' ? 'premium' : 'pro'
  const tierInfo = getSubscriptionTierInfo(upgradeTo)
  
  return {
    tier: upgradeTo,
    name: tierInfo.name,
    price: tierInfo.price,
    features: tierInfo.features
  }
}