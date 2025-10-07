import { SubscriptionTier } from '../types/nodes'

/**
 * Comprehensive tier assignment policy for DeFlow
 * This centralizes all tier decisions and makes it easy to manage upgrades
 */

// ALL FEATURES UNLOCKED FOR ALL TIERS
// Focus on user traction through social media management
const ALL_NODES = [
  // Core workflow nodes
  'manual-trigger',
  'schedule-trigger',
  'delay',
  'condition',
  'transform-data',
  'data-filter',
  'data-validator',

  // Social Media Integrations (PRIMARY FOCUS)
  'discord-webhook',
  'discord-text-message',
  'discord-embed-builder',
  'telegram-bot',
  'social-auth-setup',
  'select-platform',
  'social-media-post',
  'social-media-text',
  'social-media-with-image',

  // Individual Platform Nodes (Better UX)
  'twitter-post',
  'facebook-post',
  'linkedin-post',
  'instagram-post',

  // Picture/Image Nodes
  'upload-image',
  'ai-generate-image',
  'image-from-url',
  'image-editor',

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

  // DeFi integrations (all unlocked)
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

// All tiers get all nodes now
export const STANDARD_TIER_NODES = ALL_NODES
export const PREMIUM_TIER_NODES = ALL_NODES
export const PRO_TIER_NODES = ALL_NODES

/**
 * Get all allowed node IDs for a subscription tier
 */
export function getAllowedNodeIds(tier: SubscriptionTier): string[] {
  // All tiers get all nodes
  return ALL_NODES
}

/**
 * Check if a node ID is allowed for a subscription tier
 */
export function isNodeAllowedForTier(nodeId: string, tier: SubscriptionTier): boolean {
  // All nodes allowed for all tiers
  return true
}

/**
 * Get the minimum tier required for a node ID
 */
export function getMinimumTierForNode(nodeId: string): SubscriptionTier {
  // All nodes available at standard tier
  return 'standard'
}

/**
 * Updated tier benefits - ALL FEATURES UNLOCKED
 * Focus on social media management for user traction
 */
export const TIER_UPGRADE_MESSAGES = {
  premium: {
    title: 'Upgrade to Premium',
    price: '$19/month',
    savings: '70% fee savings (0.25% vs 0.85%)',
    breakEven: 'Break-even at $3,167/month volume',
    benefits: [
      '🚀 Priority execution queue',
      '💬 Priority email support (24h response)',
      '💰 0.25% transaction fees (70% savings!)',
      '📊 Advanced analytics dashboard',
      '🔔 Priority notifications',
      '⚡ Faster workflow execution'
    ]
  },
  pro: {
    title: 'Upgrade to Pro',
    price: '$149/month',
    savings: '88% fee savings (0.1% vs 0.85%)',
    breakEven: 'Break-even at $19,867/month volume',
    benefits: [
      '✨ All Premium features',
      '📞 24/7 priority phone support',
      '💰 0.1% transaction fees (88% savings!)',
      '🎯 Dedicated account manager',
      '🔒 Enhanced security features',
      '📊 Custom reporting & analytics',
      '🚀 Maximum execution priority',
      '💼 White-label options'
    ]
  },
  standard: {
    title: 'Standard Plan (Free)',
    price: 'Free forever',
    limitations: 'All features unlocked!',
    benefits: [
      '🌐 FULL social media integrations (Twitter, Discord, Telegram, etc.)',
      '🏦 Complete DeFi integration suite',
      '⚙️ Advanced workflow automation',
      '🔗 HTTP API calls & webhooks',
      '📧 Email & SMS capabilities',
      '👥 Community support',
      '⚡ Standard execution speed',
      '💸 0.85% transaction fees'
    ]
  }
} as const