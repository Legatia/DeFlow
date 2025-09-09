/**
 * Wallet Service
 * Handles wallet operations, balance checking, and transaction signing
 * Provides secure seed phrase handling and multi-chain support
 */

export interface WalletBalance {
  token: string
  symbol: string
  balance: number
  balanceUSD: number
  decimals: number
  contractAddress?: string
}

export interface WalletInfo {
  address: string
  chainType: 'ethereum' | 'bitcoin' | 'solana' | 'icp'
  derivationPath: string
  balances: WalletBalance[]
  totalUSD: number
}

export interface ChainConfig {
  chainId: number
  name: string
  rpcUrl: string
  nativeCurrency: {
    name: string
    symbol: string
    decimals: number
  }
  blockExplorer: string
}

// Supported chain configurations
export const CHAIN_CONFIGS: Record<string, ChainConfig> = {
  ethereum: {
    chainId: 1,
    name: 'Ethereum',
    rpcUrl: 'https://eth.llamarpc.com',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
    blockExplorer: 'https://etherscan.io'
  },
  polygon: {
    chainId: 137,
    name: 'Polygon',
    rpcUrl: 'https://polygon.llamarpc.com',
    nativeCurrency: { name: 'MATIC', symbol: 'MATIC', decimals: 18 },
    blockExplorer: 'https://polygonscan.com'
  },
  arbitrum: {
    chainId: 42161,
    name: 'Arbitrum One',
    rpcUrl: 'https://arbitrum.llamarpc.com',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
    blockExplorer: 'https://arbiscan.io'
  },
  optimism: {
    chainId: 10,
    name: 'Optimism',
    rpcUrl: 'https://optimism.llamarpc.com',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
    blockExplorer: 'https://optimistic.etherscan.io'
  }
}

// Popular ERC-20 tokens
export const POPULAR_TOKENS: Record<string, Array<{address: string, symbol: string, decimals: number}>> = {
  ethereum: [
    { address: '0xA0b86a33E6417c70C8bd9Eff59bBf8B70dC7fF9D', symbol: 'USDC', decimals: 6 },
    { address: '0xdAC17F958D2ee523a2206206994597C13D831ec7', symbol: 'USDT', decimals: 6 },
    { address: '0x6B175474E89094C44Da98b954EedeAC495271d0F', symbol: 'DAI', decimals: 18 },
    { address: '0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599', symbol: 'WBTC', decimals: 8 }
  ],
  polygon: [
    { address: '0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174', symbol: 'USDC', decimals: 6 },
    { address: '0xc2132D05D31c914a87C6611C10748AEb04B58e8F', symbol: 'USDT', decimals: 6 },
    { address: '0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063', symbol: 'DAI', decimals: 18 }
  ]
}

class WalletService {
  private encryptionKey: CryptoKey | null = null

  constructor() {
    this.initializeEncryption()
  }

  /**
   * Initialize Web Crypto API for secure seed phrase encryption
   */
  private async initializeEncryption(): Promise<void> {
    try {
      // Generate or retrieve encryption key
      const keyData = localStorage.getItem('deflow_wallet_key')
      if (keyData) {
        // Import existing key
        const keyBuffer = Uint8Array.from(atob(keyData), c => c.charCodeAt(0))
        this.encryptionKey = await crypto.subtle.importKey(
          'raw',
          keyBuffer,
          { name: 'AES-GCM' },
          false,
          ['encrypt', 'decrypt']
        )
      } else {
        // Generate new key
        this.encryptionKey = await crypto.subtle.generateKey(
          { name: 'AES-GCM', length: 256 },
          true,
          ['encrypt', 'decrypt']
        )
        
        // Export and store key
        const keyBuffer = await crypto.subtle.exportKey('raw', this.encryptionKey)
        const keyString = btoa(String.fromCharCode(...new Uint8Array(keyBuffer)))
        localStorage.setItem('deflow_wallet_key', keyString)
      }
    } catch (error) {
      console.error('Failed to initialize encryption:', error)
      // Fallback to simplified encryption if Web Crypto API fails
    }
  }

  /**
   * Encrypt seed phrase using Web Crypto API
   */
  async encryptSeedPhrase(seedPhrase: string): Promise<string> {
    if (!this.encryptionKey) {
      // Fallback encryption
      return btoa(seedPhrase + '|' + Date.now())
    }

    try {
      const encoder = new TextEncoder()
      const data = encoder.encode(seedPhrase)
      const iv = crypto.getRandomValues(new Uint8Array(12))
      
      const encrypted = await crypto.subtle.encrypt(
        { name: 'AES-GCM', iv: iv },
        this.encryptionKey,
        data
      )
      
      // Combine IV and encrypted data
      const combined = new Uint8Array(iv.length + encrypted.byteLength)
      combined.set(iv)
      combined.set(new Uint8Array(encrypted), iv.length)
      
      return btoa(String.fromCharCode(...combined))
    } catch (error) {
      console.error('Encryption failed:', error)
      throw new Error('Failed to encrypt seed phrase')
    }
  }

  /**
   * Decrypt seed phrase using Web Crypto API
   */
  async decryptSeedPhrase(encryptedData: string): Promise<string> {
    if (!this.encryptionKey) {
      // Fallback decryption
      try {
        const decoded = atob(encryptedData)
        return decoded.split('|')[0]
      } catch {
        throw new Error('Failed to decrypt seed phrase')
      }
    }

    try {
      const combined = Uint8Array.from(atob(encryptedData), c => c.charCodeAt(0))
      const iv = combined.slice(0, 12)
      const encrypted = combined.slice(12)
      
      const decrypted = await crypto.subtle.decrypt(
        { name: 'AES-GCM', iv: iv },
        this.encryptionKey,
        encrypted
      )
      
      const decoder = new TextDecoder()
      return decoder.decode(decrypted)
    } catch (error) {
      console.error('Decryption failed:', error)
      throw new Error('Failed to decrypt seed phrase')
    }
  }

  /**
   * Validate seed phrase format
   */
  validateSeedPhrase(seedPhrase: string): { valid: boolean; error?: string } {
    const words = seedPhrase.trim().split(/\s+/)
    
    if (words.length < 12 || words.length > 24) {
      return { valid: false, error: 'Seed phrase must be 12-24 words' }
    }
    
    if (words.length % 3 !== 0) {
      return { valid: false, error: 'Seed phrase must be 12, 15, 18, 21, or 24 words' }
    }
    
    // Check for duplicate words
    const uniqueWords = new Set(words)
    if (uniqueWords.size !== words.length) {
      return { valid: false, error: 'Seed phrase cannot contain duplicate words' }
    }
    
    // Basic word validation (real implementation would check against BIP39 wordlist)
    if (words.some(word => word.length < 3 || !/^[a-z]+$/.test(word))) {
      return { valid: false, error: 'Invalid characters in seed phrase' }
    }
    
    return { valid: true }
  }

  /**
   * Generate wallet address from seed phrase (mock implementation)
   * In production, use proper crypto libraries like ethers.js, bitcoinjs-lib, etc.
   */
  async generateWalletFromSeed(
    seedPhrase: string, 
    derivationPath: string, 
    chainType: 'ethereum' | 'bitcoin' | 'solana' | 'icp'
  ): Promise<{ address: string; publicKey: string }> {
    // This is a MOCK implementation for demo purposes
    // In production, use proper cryptographic libraries:
    
    // For Ethereum: ethers.js
    // const wallet = ethers.Wallet.fromMnemonic(seedPhrase, derivationPath)
    // return { address: wallet.address, publicKey: wallet.publicKey }
    
    // For Bitcoin: bitcoinjs-lib
    // const seed = bip39.mnemonicToSeedSync(seedPhrase)
    // const root = bip32.fromSeed(seed)
    // const child = root.derivePath(derivationPath)
    // const { address } = bitcoin.payments.p2pkh({ pubkey: child.publicKey })
    
    // Mock generation for demo
    const mockHash = await crypto.subtle.digest(
      'SHA-256', 
      new TextEncoder().encode(seedPhrase + derivationPath + chainType)
    )
    const hashArray = Array.from(new Uint8Array(mockHash))
    
    let address: string
    switch (chainType) {
      case 'ethereum':
        address = '0x' + hashArray.slice(0, 20).map(b => b.toString(16).padStart(2, '0')).join('')
        break
      case 'bitcoin':
        address = '1' + hashArray.slice(0, 25).map(b => b.toString(36)).join('').slice(0, 33)
        break
      case 'solana':
        address = hashArray.map(b => b.toString(36)).join('').slice(0, 44)
        break
      case 'icp':
        address = hashArray.map(b => b.toString(16).padStart(2, '0')).join('').slice(0, 63)
        break
      default:
        throw new Error('Unsupported chain type')
    }
    
    const publicKey = hashArray.map(b => b.toString(16).padStart(2, '0')).join('')
    
    return { address, publicKey }
  }

  /**
   * Check wallet balances across multiple tokens
   */
  async checkWalletBalances(
    address: string, 
    chainType: 'ethereum' | 'bitcoin' | 'solana' | 'icp'
  ): Promise<WalletBalance[]> {
    // Mock implementation - replace with real API calls
    const mockBalances: WalletBalance[] = []
    
    // Add native currency balance
    const nativeBalance = Math.random() * 10
    const nativePrice = this.getMockPrice(chainType === 'ethereum' ? 'ETH' : 'BTC')
    
    mockBalances.push({
      token: chainType === 'ethereum' ? 'Ether' : 'Bitcoin',
      symbol: chainType === 'ethereum' ? 'ETH' : 'BTC',
      balance: nativeBalance,
      balanceUSD: nativeBalance * nativePrice,
      decimals: 18,
    })
    
    // Add popular token balances for Ethereum-based chains
    if (chainType === 'ethereum') {
      const tokens = POPULAR_TOKENS[chainType] || []
      for (const token of tokens) {
        const balance = Math.random() * 1000
        const price = this.getMockPrice(token.symbol)
        
        mockBalances.push({
          token: token.symbol,
          symbol: token.symbol,
          balance: balance,
          balanceUSD: balance * price,
          decimals: token.decimals,
          contractAddress: token.address
        })
      }
    }
    
    return mockBalances.filter(b => b.balance > 0)
  }

  /**
   * Get mock token prices (replace with real price API)
   */
  private getMockPrice(symbol: string): number {
    const mockPrices: Record<string, number> = {
      'ETH': 2500 + Math.random() * 500,
      'BTC': 45000 + Math.random() * 10000,
      'USDC': 1,
      'USDT': 1,
      'DAI': 1,
      'WBTC': 45000 + Math.random() * 10000,
      'MATIC': 0.8 + Math.random() * 0.4
    }
    
    return mockPrices[symbol] || 1
  }

  /**
   * Get wallet information including balances
   */
  async getWalletInfo(
    address: string, 
    chainType: 'ethereum' | 'bitcoin' | 'solana' | 'icp',
    derivationPath: string
  ): Promise<WalletInfo> {
    const balances = await this.checkWalletBalances(address, chainType)
    const totalUSD = balances.reduce((sum, balance) => sum + balance.balanceUSD, 0)
    
    return {
      address,
      chainType,
      derivationPath,
      balances,
      totalUSD
    }
  }

  /**
   * Test wallet connection and balances
   */
  async testWalletConnection(
    address: string, 
    chainType: 'ethereum' | 'bitcoin' | 'solana' | 'icp'
  ): Promise<{ valid: boolean; error?: string; balanceCount?: number }> {
    try {
      const balances = await this.checkWalletBalances(address, chainType)
      return { 
        valid: true, 
        balanceCount: balances.length 
      }
    } catch (error) {
      return { 
        valid: false, 
        error: error instanceof Error ? error.message : 'Connection test failed' 
      }
    }
  }

  /**
   * Format balance for display
   */
  formatBalance(amount: number, decimals: number = 18): string {
    const adjusted = amount / Math.pow(10, decimals)
    
    if (adjusted === 0) return '0'
    if (adjusted < 0.0001) return '<0.0001'
    if (adjusted < 1) return adjusted.toFixed(6)
    if (adjusted < 1000) return adjusted.toFixed(4)
    if (adjusted < 1000000) return (adjusted / 1000).toFixed(2) + 'K'
    return (adjusted / 1000000).toFixed(2) + 'M'
  }

  /**
   * Format USD value for display
   */
  formatUSD(amount: number): string {
    if (amount === 0) return '$0'
    if (amount < 0.01) return '<$0.01'
    if (amount < 1000) return '$' + amount.toFixed(2)
    if (amount < 1000000) return '$' + (amount / 1000).toFixed(2) + 'K'
    return '$' + (amount / 1000000).toFixed(2) + 'M'
  }
}

// Export singleton instance
export const walletService = new WalletService()
export default walletService