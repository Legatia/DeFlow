# DeFlow Testnet Configuration Guide

This document provides comprehensive guidance for setting up and using testnet environments for DeFlow's multi-chain arbitrage testing capabilities.

## Overview

DeFlow supports testing across multiple blockchain testnets to validate arbitrage strategies before deploying capital on mainnet. This guide covers testnet token acquisition, configuration, and testing best practices.

## Supported Testnet Networks

### 1. Ethereum Testnets
- **Sepolia** (Recommended)
  - Chain ID: 11155111
  - RPC: `https://sepolia.infura.io/v3/YOUR_KEY`
  - Explorer: `https://sepolia.etherscan.io`
  - Faucet: `https://sepoliafaucet.com`
  - Token Limit: 0.5 ETH per day

- **Goerli** (Being deprecated)
  - Chain ID: 5
  - RPC: `https://goerli.infura.io/v3/YOUR_KEY`
  - Explorer: `https://goerli.etherscan.io`
  - Faucet: `https://goerlifaucet.com`

### 2. Layer 2 Testnets

#### Arbitrum Sepolia
- Chain ID: 421614
- RPC: `https://sepolia-rollup.arbitrum.io/rpc`
- Explorer: `https://sepolia.arbiscan.io`
- Faucet: Bridge from Ethereum Sepolia
- Dependencies: Requires Sepolia ETH first

#### Optimism Sepolia
- Chain ID: 11155420
- RPC: `https://sepolia.optimism.io`
- Explorer: `https://sepolia-optimistic.etherscan.io`
- Faucet: `https://app.optimism.io/faucet`
- Token Limit: 1 ETH per day

#### Polygon Mumbai (Being deprecated - use Amoy)
- Chain ID: 80001
- RPC: `https://rpc-mumbai.maticvigil.com`
- Explorer: `https://mumbai.polygonscan.com`
- Faucet: `https://faucet.polygon.technology`

#### Polygon Amoy (New testnet)
- Chain ID: 80002
- RPC: `https://rpc-amoy.polygon.technology`
- Explorer: `https://www.oklink.com/amoy`
- Faucet: `https://faucet.polygon.technology`
- Token Limit: 1 MATIC per day

#### Base Sepolia
- Chain ID: 84532
- RPC: `https://sepolia.base.org`
- Explorer: `https://sepolia-explorer.base.org`
- Faucet: Bridge from Ethereum Sepolia via Base Bridge

### 3. Other Chain Testnets

#### Avalanche Fuji
- Chain ID: 43113
- RPC: `https://api.avax-test.network/ext/bc/C/rpc`
- Explorer: `https://testnet.snowtrace.io`
- Faucet: `https://faucet.avax.network`
- Token Limit: 2 AVAX per day

#### BSC Testnet
- Chain ID: 97
- RPC: `https://data-seed-prebsc-1-s1.binance.org:8545`
- Explorer: `https://testnet.bscscan.com`
- Faucet: `https://testnet.binance.org/faucet-smart`
- Token Limit: 0.1 BNB per day

#### Solana Devnet
- Cluster: devnet
- RPC: `https://api.devnet.solana.com`
- Explorer: `https://explorer.solana.com/?cluster=devnet`
- Faucet: `solana airdrop 1` (CLI) or web faucets
- Token Limit: 1 SOL per request

## Testnet Token Acquisition Strategy

### Tier 1: Direct Faucets (Easy)
```bash
# Ethereum Sepolia
# Visit https://sepoliafaucet.com, connect wallet, claim 0.5 ETH

# Optimism Sepolia
# Visit https://app.optimism.io/faucet, connect wallet, claim 1 ETH

# Polygon Amoy
# Visit https://faucet.polygon.technology, select Amoy, claim 1 MATIC

# Avalanche Fuji
# Visit https://faucet.avax.network, enter address, claim 2 AVAX

# BSC Testnet
# Visit https://testnet.binance.org/faucet-smart, enter address, claim 0.1 BNB
```

### Tier 2: Bridge-Based Faucets (Medium)
```bash
# Arbitrum Sepolia - Requires Ethereum Sepolia first
1. Get Sepolia ETH from faucet
2. Bridge to Arbitrum Sepolia via official bridge
3. Wait 10-15 minutes for L2 confirmation

# Base Sepolia - Requires Ethereum Sepolia first
1. Get Sepolia ETH from faucet
2. Use Base Sepolia bridge
3. Bridge small amounts (0.01-0.1 ETH recommended)
```

### Tier 3: Community/Social Faucets (Hard)
```bash
# Alternative faucets requiring social verification:
# - QuickNode faucets (Twitter/Discord verification)
# - Alchemy faucets (account required)
# - Chainlink faucets (GitHub verification)
# - Community Discord servers
```

### Tier 4: Testnet Token Swaps (Advanced)
```bash
# Some testnets allow token swaps:
# - Use testnet DEXs (Uniswap on testnets)
# - Cross-chain testnet bridges
# - Testnet yield farming protocols
```

## DeFlow Testnet Configuration

### 1. Environment Configuration

Create testnet configuration in DeFlow:

```typescript
// src/config/testnet.ts
export const TESTNET_CHAINS = {
  'ethereum-sepolia': {
    name: 'Ethereum Sepolia',
    chainId: 11155111,
    symbol: 'ETH',
    rpcUrl: 'https://sepolia.infura.io/v3/YOUR_KEY',
    explorerUrl: 'https://sepolia.etherscan.io',
    isTestnet: true,
    faucetUrl: 'https://sepoliafaucet.com'
  },
  'arbitrum-sepolia': {
    name: 'Arbitrum Sepolia',
    chainId: 421614,
    symbol: 'ETH',
    rpcUrl: 'https://sepolia-rollup.arbitrum.io/rpc',
    explorerUrl: 'https://sepolia.arbiscan.io',
    isTestnet: true,
    faucetUrl: 'https://bridge.arbitrum.io'
  },
  'optimism-sepolia': {
    name: 'Optimism Sepolia',
    chainId: 11155420,
    symbol: 'ETH',
    rpcUrl: 'https://sepolia.optimism.io',
    explorerUrl: 'https://sepolia-optimistic.etherscan.io',
    isTestnet: true,
    faucetUrl: 'https://app.optimism.io/faucet'
  },
  'polygon-amoy': {
    name: 'Polygon Amoy',
    chainId: 80002,
    symbol: 'MATIC',
    rpcUrl: 'https://rpc-amoy.polygon.technology',
    explorerUrl: 'https://www.oklink.com/amoy',
    isTestnet: true,
    faucetUrl: 'https://faucet.polygon.technology'
  },
  'base-sepolia': {
    name: 'Base Sepolia',
    chainId: 84532,
    symbol: 'ETH',
    rpcUrl: 'https://sepolia.base.org',
    explorerUrl: 'https://sepolia-explorer.base.org',
    isTestnet: true,
    faucetUrl: 'https://bridge.base.org'
  },
  'avalanche-fuji': {
    name: 'Avalanche Fuji',
    chainId: 43113,
    symbol: 'AVAX',
    rpcUrl: 'https://api.avax-test.network/ext/bc/C/rpc',
    explorerUrl: 'https://testnet.snowtrace.io',
    isTestnet: true,
    faucetUrl: 'https://faucet.avax.network'
  },
  'bsc-testnet': {
    name: 'BSC Testnet',
    chainId: 97,
    symbol: 'BNB',
    rpcUrl: 'https://data-seed-prebsc-1-s1.binance.org:8545',
    explorerUrl: 'https://testnet.bscscan.com',
    isTestnet: true,
    faucetUrl: 'https://testnet.binance.org/faucet-smart'
  },
  'solana-devnet': {
    name: 'Solana Devnet',
    chainId: 'solana-devnet',
    symbol: 'SOL',
    rpcUrl: 'https://api.devnet.solana.com',
    explorerUrl: 'https://explorer.solana.com/?cluster=devnet',
    isTestnet: true,
    faucetUrl: 'https://solfaucet.com'
  }
}
```

### 2. Testnet Mode Toggle

Add testnet mode to DeFlow configuration:

```typescript
// src/stores/configStore.ts
interface ConfigState {
  isTestnetMode: boolean
  activeNetwork: 'mainnet' | 'testnet'
  testnetChains: string[]
  toggleTestnetMode: () => void
  setActiveTestnetChains: (chains: string[]) => void
}

const useConfigStore = create<ConfigState>((set, get) => ({
  isTestnetMode: false,
  activeNetwork: 'mainnet',
  testnetChains: [],
  
  toggleTestnetMode: () => set((state) => ({
    isTestnetMode: !state.isTestnetMode,
    activeNetwork: state.isTestnetMode ? 'mainnet' : 'testnet'
  })),
  
  setActiveTestnetChains: (chains) => set({ testnetChains: chains })
}))
```

### 3. Testnet Arbitrage Configuration

Configure arbitrage parameters for testnet:

```typescript
// src/config/arbitrage-testnet.ts
export const TESTNET_ARBITRAGE_CONFIG = {
  // Reduced amounts for testnet
  maxTradeAmount: 0.1, // vs 5.0 on mainnet
  minProfitThreshold: 0.5, // Lower threshold for testing
  
  // More frequent scanning
  scanInterval: 30000, // 30 seconds vs 5 minutes on mainnet
  
  // Testnet-specific tokens
  supportedTokens: {
    'ethereum-sepolia': ['ETH', 'USDC', 'DAI'],
    'arbitrum-sepolia': ['ETH', 'USDC'],
    'optimism-sepolia': ['ETH', 'USDC'],
    'polygon-amoy': ['MATIC', 'USDC'],
    'base-sepolia': ['ETH', 'USDC'],
    'avalanche-fuji': ['AVAX', 'USDC'],
    'bsc-testnet': ['BNB', 'USDT'],
    'solana-devnet': ['SOL', 'USDC']
  },
  
  // Testnet DEX contracts (example addresses)
  dexContracts: {
    'ethereum-sepolia': {
      uniswapV2: '0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f',
      uniswapV3: '0xE592427A0AEce92De3Edee1F18E0157C05861564'
    },
    'arbitrum-sepolia': {
      uniswapV3: '0xE592427A0AEce92De3Edee1F18E0157C05861564'
    }
  }
}
```

## Arbitrage Testing Strategies

### 1. Cross-Chain Price Discovery
```bash
# Test Case: ETH price differences across L2s
1. Deploy small amounts (0.01-0.1 ETH) across:
   - Ethereum Sepolia
   - Arbitrum Sepolia  
   - Optimism Sepolia
   - Base Sepolia

2. Monitor price feeds from testnet DEXs
3. Execute mock arbitrage trades
4. Measure slippage and gas costs
```

### 2. Multi-Chain Yield Comparison
```bash
# Test Case: Yield farming opportunities
1. Deploy testnet tokens to various protocols:
   - Aave on different testnets
   - Compound on Ethereum Sepolia
   - Native protocols on each chain

2. Compare APY rates
3. Test automated rebalancing
4. Measure gas costs vs yield
```

### 3. Cross-Chain Bridge Arbitrage
```bash
# Test Case: Bridge fee arbitrage
1. Identify bridge fee discrepancies
2. Test bridging costs vs potential profits
3. Account for bridge delays (10min - 1hr)
4. Test emergency exit strategies
```

## Implementation Steps

### Phase 1: Basic Testnet Integration

1. **Update Chain Configuration**
   - Extend `SUPPORTED_CHAINS` in `multiChainWalletService.ts`
   - Add testnet chain configurations
   - Update chain type definitions

2. **Add Testnet Toggle**
   - Create testnet mode toggle in UI
   - Store preference in localStorage
   - Update all chain-dependent services

3. **Testnet Wallet Support**
   - Ensure MetaMask testnet switching
   - Test wallet connections
   - Validate address formats

### Phase 2: Testnet-Specific Features

1. **Faucet Integration**
   - Add faucet links in UI
   - Create faucet request helpers
   - Monitor faucet rate limits

2. **Testnet Data Sources**
   - Configure testnet price oracles
   - Set up testnet DEX integrations
   - Test data reliability

3. **Reduced Risk Parameters**
   - Lower trade amounts
   - Shorter timeouts
   - More conservative slippage

### Phase 3: Advanced Testing Features

1. **Simulation Mode**
   - Paper trading functionality
   - Risk-free strategy testing
   - Performance analytics

2. **Testnet Analytics**
   - Track testnet vs mainnet performance
   - Gas cost comparisons
   - Success rate metrics

3. **Automated Testing**
   - Continuous testnet monitoring
   - Automated faucet requests
   - Strategy backtesting

## Best Practices

### 1. Testnet Limitations
- **Limited Liquidity**: Testnet DEXs have minimal liquidity
- **Artificial Pricing**: Prices don't reflect real market conditions
- **Network Stability**: Testnets can be unstable or reset
- **Token Availability**: Faucets have daily/hourly limits

### 2. Testing Approach
- **Start Small**: Use minimum viable amounts
- **Test Incrementally**: Build complexity gradually
- **Monitor Closely**: Watch for testnet-specific issues
- **Document Results**: Track what works and what doesn't

### 3. Risk Management
- **Never Use Real Funds**: Only use testnet tokens
- **Separate Configurations**: Keep testnet/mainnet configs separate
- **Validate Thoroughly**: Test all functionality before mainnet
- **Have Exit Strategies**: Plan for testnet failures

### 4. Maintenance
- **Update Regularly**: Testnets change frequently
- **Monitor Deprecations**: Watch for testnet sunsets
- **Keep Tokens Fresh**: Refresh testnet tokens regularly
- **Stay Informed**: Follow network announcements

## Common Issues and Solutions

### Issue 1: Insufficient Testnet Tokens
**Solution**:
- Use multiple faucets across different networks
- Join community Discord servers for additional faucets
- Use bridge-based faucets for L2 tokens
- Implement faucet rotation strategy

### Issue 2: Testnet Network Instability
**Solution**:
- Use multiple RPC endpoints
- Implement retry mechanisms
- Set longer timeouts for testnet operations
- Have fallback testnets configured

### Issue 3: Limited DEX Liquidity
**Solution**:
- Use smaller trade amounts
- Test with stablecoins when available
- Focus on gas optimization over profit
- Simulate larger trades without execution

### Issue 4: Testnet Reset/Wipe
**Solution**:
- Don't rely on long-term testnet state
- Regularly backup important configurations
- Use disposable testnet strategies
- Keep comprehensive documentation

## Conclusion

Testnet arbitrage testing is crucial for validating DeFlow strategies before mainnet deployment. While testnets have limitations, they provide a safe environment for:

1. **Strategy Development**: Test algorithms without financial risk
2. **Gas Optimization**: Optimize transaction costs
3. **Integration Testing**: Validate cross-chain functionality  
4. **Performance Monitoring**: Measure strategy effectiveness

By following this guide, you can set up comprehensive testnet environments for thorough DeFlow arbitrage testing across multiple blockchain networks.

## Next Steps

Would you like me to implement any specific part of this testnet configuration? I can help with:

1. **Creating the testnet chain configurations** in the existing service files
2. **Adding a testnet mode toggle** to the DeFlow UI
3. **Implementing faucet integration helpers** for automated token acquisition
4. **Setting up testnet-specific arbitrage parameters** for safe testing

The testnet infrastructure will allow you to validate arbitrage strategies across 8 different blockchain networks using free testnet tokens, providing confidence before deploying real capital on mainnet.