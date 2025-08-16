import { SubscriptionTier } from '../types/nodes'

/**
 * Comprehensive tier assignment policy for DeFlow
 * This centralizes all tier decisions and makes it easy to manage upgrades
 */

// Nodes available for Standard tier (free)
export const STANDARD_TIER_NODES = [
  // Core workflow nodes
  'manual-trigger',
  'schedule-trigger',
  'delay',
  'condition',
  'transform-data',
  
  // Basic utilities
  'data-filter',
  'data-validator',
  
  // Allowed social integrations for Standard tier
  'discord-webhook',
  'discord-text-message', 
  'discord-embed-builder',
  'telegram-bot',
]

// Additional nodes for Premium tier ($19/month)
export const PREMIUM_TIER_NODES = [
  ...STANDARD_TIER_NODES,
  
  // Social Media Integrations
  'twitter-post',
  'facebook-post',
  'linkedin-post',
  'instagram-post',
  
  // Communication
  'send-email',
  'sms-send',
  
  // API Integrations
  'http-request',
  'webhook-trigger',
  'api-call',
  
  // Advanced utilities
  'data-transform',
  'json-processor',
  'text-processor',
  'file-processor',
  
  // Basic Analytics
  'analytics-track',
  'event-tracker',
]

// Additional nodes for Pro tier ($149/month)
export const PRO_TIER_NODES = [
  ...PREMIUM_TIER_NODES,
  
  // DeFi integrations (all DeFi nodes require Pro)
  'bitcoin-portfolio',
  'bitcoin-send',
  'bitcoin-address',
  'bitcoin-balance',
  'ethereum-portfolio',
  'ethereum-send',
  'ethereum-address',
  'ethereum-gas-estimate',
  'l2-optimization',
  'bridge-analysis',
  'defi-yield-farming',
  'defi-arbitrage',
  'portfolio-manager',
  
  // Advanced features
  'ai-analysis',
  'ml-prediction',
  'advanced-scheduler',
  'batch-processor',
  'multi-chain-bridge',
  
  // Enterprise features
  'audit-logger',
  'compliance-checker',
  'risk-assessment',
  'automated-reporting',
]

/**
 * Get all allowed node IDs for a subscription tier
 */
export function getAllowedNodeIds(tier: SubscriptionTier): string[] {
  switch (tier) {
    case 'standard':
      return STANDARD_TIER_NODES
    case 'premium':
      return PREMIUM_TIER_NODES
    case 'pro':
      return PRO_TIER_NODES
    default:
      return STANDARD_TIER_NODES
  }
}

/**
 * Check if a node ID is allowed for a subscription tier
 */
export function isNodeAllowedForTier(nodeId: string, tier: SubscriptionTier): boolean {
  return getAllowedNodeIds(tier).includes(nodeId)
}

/**
 * Get the minimum tier required for a node ID
 */
export function getMinimumTierForNode(nodeId: string): SubscriptionTier {
  if (STANDARD_TIER_NODES.includes(nodeId)) return 'standard'
  if (PREMIUM_TIER_NODES.includes(nodeId)) return 'premium'
  if (PRO_TIER_NODES.includes(nodeId)) return 'pro'
  
  // Default to premium for unknown nodes (safer)
  return 'premium'
}

/**
 * Tier upgrade suggestions based on restricted nodes
 */
export const TIER_UPGRADE_MESSAGES = {
  premium: {
    title: 'Upgrade to Premium',
    price: '$19/month',
    benefits: [
      'Full social media integrations (Twitter, Facebook, LinkedIn)',
      'Email & SMS sending capabilities', 
      'HTTP API calls & webhooks',
      'Advanced data processing tools',
      'Priority execution queue',
      'Email support (24h response)'
    ]
  },
  pro: {
    title: 'Upgrade to Pro', 
    price: '$149/month',
    benefits: [
      'All Premium features',
      'Complete DeFi integration suite',
      'AI/ML analysis capabilities',
      'Multi-chain bridge operations',
      'Enterprise compliance tools',
      'Advanced analytics & reporting',
      '24/7 priority support'
    ]
  }
} as const