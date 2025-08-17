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
      price: 'Free',
      monthlyPrice: '$0',
      color: '#6b7280',
      feeRate: '0.85%',
      features: [
        'Telegram & Discord nodes only',
        'Basic workflow automation', 
        'Community support',
        'Standard execution speed',
        '0.85% transaction fees'
      ],
      limitations: [
        'Limited to Telegram and Discord integrations',
        'No access to social media nodes (Twitter, Facebook, LinkedIn)',
        'No email/SMS capabilities',
        'No DeFi integrations'
      ]
    },
    'premium': {
      name: 'Premium', 
      price: '$19/month',
      monthlyPrice: '$19',
      color: '#3b82f6',
      feeRate: '0.25%',
      features: [
        'All Standard features',
        'Full social media integrations (Twitter, Facebook, LinkedIn)',
        'Email & SMS sending capabilities',
        'HTTP API calls & webhooks',
        'Advanced data processing tools',
        'Priority execution queue',
        'Email support (24h response)',
        '0.25% transaction fees (70% savings!)',
        'Break-even at $3,167/month volume'
      ]
    },
    'pro': {
      name: 'Pro',
      price: '$149/month',
      monthlyPrice: '$149', 
      color: '#7c3aed',
      feeRate: '0.1%',
      features: [
        'All Premium features',
        'Complete DeFi integration suite',
        'Full API access',
        'Custom strategy development',
        'Portfolio insurance options',
        'Priority phone support',
        'Advanced risk management tools',
        '0.1% transaction fees (88% savings!)',
        'Break-even at $19,867/month volume'
      ]
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