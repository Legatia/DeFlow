// DEX Priority Configuration for Multi-Chain Trading
// Defines preferred DEXs for each blockchain network

// Import BigInt polyfill FIRST to prevent conversion errors
import '../utils/bigint-polyfill'

export interface DexInfo {
  name: string
  priority: number // 1 = highest priority
  website: string
  apiEndpoint?: string
  fee: string
  features: string[]
}

export const DEX_PRIORITIES: Record<string, DexInfo[]> = {
  'Ethereum': [
    {
      name: 'Uniswap V3',
      priority: 1,
      website: 'https://app.uniswap.org',
      apiEndpoint: 'https://api.uniswap.org',
      fee: '0.05-1%',
      features: ['Concentrated Liquidity', 'Multiple Fee Tiers', 'High Volume']
    },
    {
      name: 'SushiSwap',
      priority: 2,
      website: 'https://www.sushi.com',
      fee: '0.25-0.3%',
      features: ['Cross-chain', 'Yield Farming', 'Good Liquidity']
    },
    {
      name: '1inch',
      priority: 3,
      website: 'https://1inch.io',
      fee: 'Variable',
      features: ['DEX Aggregator', 'Best Prices', 'Gas Optimization']
    }
  ],
  
  'Arbitrum': [
    {
      name: 'Uniswap V3',
      priority: 1,
      website: 'https://app.uniswap.org',
      fee: '0.05-1%',
      features: ['Low Gas', 'High Liquidity', 'Fast Execution']
    },
    {
      name: 'Camelot',
      priority: 2,
      website: 'https://app.camelot.exchange',
      fee: '0.3%',
      features: ['Native to Arbitrum', 'Concentrated Liquidity', 'Good APY']
    },
    {
      name: 'Balancer',
      priority: 3,
      website: 'https://app.balancer.fi',
      fee: '0.1-1%',
      features: ['Multi-token Pools', 'Weighted Pools', 'Liquidity Mining']
    }
  ],
  
  'Optimism': [
    {
      name: 'Uniswap V3',
      priority: 1,
      website: 'https://app.uniswap.org',
      fee: '0.05-1%',
      features: ['Optimistic Rollup', 'Low Fees', 'Fast Finality']
    },
    {
      name: 'Velodrome',
      priority: 2,
      website: 'https://app.velodrome.finance',
      fee: '0.01-1%',
      features: ['ve(3,3) Model', 'Vote-Escrow', 'Native to OP']
    }
  ],
  
  'Polygon': [
    {
      name: 'QuickSwap',
      priority: 1,
      website: 'https://quickswap.exchange',
      fee: '0.3%',
      features: ['Native to Polygon', 'Dragon Syrup', 'Low Fees']
    },
    {
      name: 'SushiSwap',
      priority: 2,
      website: 'https://www.sushi.com',
      fee: '0.25-0.3%',
      features: ['Cross-chain', 'Established', 'Good Volume']
    },
    {
      name: 'Curve',
      priority: 3,
      website: 'https://curve.fi',
      fee: '0.04%',
      features: ['Stablecoin Focus', 'Low Slippage', 'High TVL']
    }
  ],
  
  'Base': [
    {
      name: 'BaseSwap',
      priority: 1,
      website: 'https://baseswap.fi',
      fee: '0.25%',
      features: ['Native to Base', 'Early Adopter', 'Growing Liquidity']
    },
    {
      name: 'Uniswap V3',
      priority: 2,
      website: 'https://app.uniswap.org',
      fee: '0.05-1%',
      features: ['Trusted Brand', 'High Volume', 'Multiple Tiers']
    }
  ],
  
  'Avalanche': [
    {
      name: 'Trader Joe',
      priority: 1,
      website: 'https://traderjoexyz.com',
      fee: '0.3%',
      features: ['Native to Avalanche', 'Liquidity Book', 'High Volume']
    },
    {
      name: 'Pangolin',
      priority: 2,
      website: 'https://app.pangolin.exchange',
      fee: '0.3%',
      features: ['Community-driven', 'PNG Rewards', 'Established']
    }
  ],
  
  'Solana': [
    {
      name: 'Raydium',
      priority: 1,
      website: 'https://raydium.io',
      fee: '0.25%',
      features: ['AMM + Order Book', 'High Volume', 'Serum Integration']
    },
    {
      name: 'Orca',
      priority: 2,
      website: 'https://www.orca.so',
      fee: '0.3%',
      features: ['User-friendly', 'Concentrated Liquidity', 'Fair Launch']
    },
    {
      name: 'Jupiter',
      priority: 3,
      website: 'https://jup.ag',
      fee: 'Variable',
      features: ['DEX Aggregator', 'Best Routes', 'Multiple DEXs']
    }
  ],
  
  
  'ICP': [
    {
      name: 'ICPSwap',
      priority: 1,
      website: 'https://app.icpswap.com',
      fee: '0.3%',
      features: ['Largest ICP DEX', 'High Liquidity', 'ICS Token']
    },
    {
      name: 'KongSwap',
      priority: 2,
      website: 'https://kongswap.io',
      fee: '0.3%',
      features: ['Chain Fusion', 'Multi-chain', 'Zero Gas', 'Bridgeless']
    },
  ]
}

// Helper functions for DEX management
export class DexPriorityService {
  static getPreferredDex(chain: string): DexInfo | null {
    const chainDexs = DEX_PRIORITIES[chain]
    if (!chainDexs || chainDexs.length === 0) return null
    
    return chainDexs.sort((a, b) => a.priority - b.priority)[0]
  }
  
  static getAllDexsForChain(chain: string): DexInfo[] {
    return DEX_PRIORITIES[chain] || []
  }
  
  static getSupportedChains(): string[] {
    return Object.keys(DEX_PRIORITIES)
  }
  
  static getDexByName(chain: string, dexName: string): DexInfo | null {
    const chainDexs = DEX_PRIORITIES[chain]
    if (!chainDexs) return null
    
    return chainDexs.find(dex => 
      dex.name.toLowerCase() === dexName.toLowerCase()
    ) || null
  }
  
  static getKongSwapInfo(): DexInfo | null {
    return this.getDexByName('ICP', 'KongSwap')
  }
  
  static isKongSwapSupported(): boolean {
    return this.getKongSwapInfo() !== null
  }
  
  // Check if a chain supports multiple DEXs for arbitrage
  static hasMultipleDexs(chain: string): boolean {
    const chainDexs = DEX_PRIORITIES[chain]
    return chainDexs ? chainDexs.length > 1 : false
  }
  
  // Get DEX recommendation based on trade size
  static getRecommendedDex(
    chain: string, 
    tradeSize: number, 
    tradePriority: 'speed' | 'cost' | 'liquidity' = 'liquidity'
  ): DexInfo | null {
    const chainDexs = DEX_PRIORITIES[chain]
    if (!chainDexs || chainDexs.length === 0) return null
    
    // For large trades, prioritize liquidity
    if (tradeSize > 10000) {
      return chainDexs.find(dex => 
        dex.features.includes('High Volume') || 
        dex.features.includes('High Liquidity')
      ) || chainDexs[0]
    }
    
    // For small trades, prioritize low fees
    if (tradeSize < 1000) {
      return chainDexs.find(dex => 
        dex.features.includes('Low Fees') ||
        dex.fee.includes('0.05') ||
        dex.fee.includes('0.1')
      ) || chainDexs[0]
    }
    
    // Default to highest priority DEX
    return chainDexs.sort((a, b) => a.priority - b.priority)[0]
  }
}

export default DexPriorityService