import { SubscriptionTier, NodeType } from '../types/nodes'

// Define tier hierarchy (higher tiers include all lower tier access)
const TIER_HIERARCHY: Record<SubscriptionTier, number> = {
  'standard': 1,
  'premium': 2,
  'pro': 3
}

/**
 * Check if user's subscription tier allows access to a node that requires a specific tier
 * ALL FEATURES UNLOCKED - always returns true
 */
export function canAccessNode(userTier: SubscriptionTier, requiredTier: SubscriptionTier): boolean {
  return true // All features unlocked for all tiers
}

/**
 * Universal access control for any node type
 * This is the main function that should be used everywhere for feature gating
 * ALL FEATURES UNLOCKED - always returns true
 */
export function canAccessNodeType(userTier: SubscriptionTier, nodeType: NodeType): boolean {
  return true // All features unlocked for all tiers
}

/**
 * Check if a user can perform drag & drop operations on a node
 * ALL FEATURES UNLOCKED - always returns true
 */
export function canDragNode(userTier: SubscriptionTier, nodeType: NodeType): boolean {
  return true // All features unlocked for all tiers
}

/**
 * Check if a user can add a node to their workflow
 * ALL FEATURES UNLOCKED - always returns true
 */
export function canAddNodeToWorkflow(userTier: SubscriptionTier, nodeType: NodeType): boolean {
  return true // All features unlocked for all tiers
}

/**
 * Check if a user can execute/run workflows containing specific node types
 * ALL FEATURES UNLOCKED - always returns canExecute: true
 */
export function canExecuteWorkflowWithNodes(userTier: SubscriptionTier, nodeTypes: NodeType[]): {
  canExecute: boolean
  restrictedNodes: NodeType[]
  requiredUpgrade?: SubscriptionTier
} {
  // All nodes allowed for all tiers
  return { canExecute: true, restrictedNodes: [] }
}

/**
 * Get subscription tier display information
 * UPDATED: All features unlocked for all tiers
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
        '🌐 FULL social media integrations (Twitter, Discord, Telegram, etc.)',
        '🏦 Complete DeFi integration suite',
        '⚙️ Advanced workflow automation',
        '🔗 HTTP API calls & webhooks',
        '📧 Email & SMS capabilities',
        '👥 Community support',
        '⚡ Standard execution speed',
        '💸 0.85% transaction fees'
      ],
      limitations: [] // No limitations - all features unlocked!
    },
    'premium': {
      name: 'Premium',
      price: '$19/month',
      monthlyPrice: '$19',
      color: '#3b82f6',
      feeRate: '0.25%',
      features: [
        '✨ All Standard features',
        '🚀 Priority execution queue',
        '💬 Priority email support (24h response)',
        '💰 0.25% transaction fees (70% savings!)',
        '📊 Advanced analytics dashboard',
        '🔔 Priority notifications',
        '⚡ Faster workflow execution',
        '💎 Break-even at $3,167/month volume'
      ]
    },
    'pro': {
      name: 'Pro',
      price: '$149/month',
      monthlyPrice: '$149',
      color: '#7c3aed',
      feeRate: '0.1%',
      features: [
        '✨ All Premium features',
        '📞 24/7 priority phone support',
        '💰 0.1% transaction fees (88% savings!)',
        '🎯 Dedicated account manager',
        '🔒 Enhanced security features',
        '📊 Custom reporting & analytics',
        '🚀 Maximum execution priority',
        '💼 White-label options',
        '💎 Break-even at $19,867/month volume'
      ]
    }
  }

  return tierInfo[tier]
}

/**
 * Get upgrade path for accessing a restricted node
 * ALL FEATURES UNLOCKED - always returns null (no upgrade needed)
 */
export function getUpgradePath(userTier: SubscriptionTier, requiredTier: SubscriptionTier): { name: string; price: string } | null {
  return null // No upgrade needed - all features unlocked
}