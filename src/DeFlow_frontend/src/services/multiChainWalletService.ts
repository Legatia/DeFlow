// Multi-Chain Wallet Management Service
// Handles wallet connections and addresses for multiple blockchains

// Import BigInt polyfill FIRST to prevent conversion errors
import '../utils/bigint-polyfill'
import secureStorageService from './secureStorageService'
import performanceService from './performanceOptimizationService'

export interface WalletAddress {
  chain: ChainType
  address: string
  balance?: string
  isConnected: boolean
  walletType: WalletType
  lastUpdated: number
}

export interface MultiChainWallet {
  addresses: WalletAddress[]
  primaryWallet?: WalletAddress
  totalValueUSD?: number
  lastSyncAt: number
}

export type ChainType = 
  | 'Bitcoin' 
  | 'Ethereum' 
  | 'Arbitrum' 
  | 'Optimism' 
  | 'Polygon' 
  | 'Base' 
  | 'Avalanche' 
  | 'Solana' 
  | 'ICP'

export type WalletType = 
  | 'MetaMask' 
  | 'WalletConnect' 
  | 'Phantom' 
  | 'Coinbase' 
  | 'Trust' 
  | 'Manual' 
  | 'InternetIdentity'
  | 'Plug'
  | 'Stoic'

export interface ChainConfig {
  name: string
  chainId: number | string
  symbol: string
  rpcUrl: string
  explorerUrl: string
  icon: string
  color: string
  supportedWallets: WalletType[]
}

export const SUPPORTED_CHAINS: Record<ChainType, ChainConfig> = {
  Bitcoin: {
    name: 'Bitcoin',
    chainId: 'bitcoin',
    symbol: 'BTC',
    rpcUrl: 'https://bitcoin.org',
    explorerUrl: 'https://blockstream.info',
    icon: '₿',
    color: '#f7931a',
    supportedWallets: ['Manual']
  },
  Ethereum: {
    name: 'Ethereum',
    chainId: 1,
    symbol: 'ETH',
    rpcUrl: 'https://eth.llamarpc.com',
    explorerUrl: 'https://etherscan.io',
    icon: 'Ξ',
    color: '#627eea',
    supportedWallets: ['MetaMask', 'WalletConnect', 'Coinbase', 'Trust']
  },
  Arbitrum: {
    name: 'Arbitrum One',
    chainId: 42161,
    symbol: 'ETH',
    rpcUrl: 'https://arb1.arbitrum.io/rpc',
    explorerUrl: 'https://arbiscan.io',
    icon: '🔵',
    color: '#28a0f0',
    supportedWallets: ['MetaMask', 'WalletConnect', 'Coinbase']
  },
  Optimism: {
    name: 'Optimism',
    chainId: 10,
    symbol: 'ETH',
    rpcUrl: 'https://mainnet.optimism.io',
    explorerUrl: 'https://optimistic.etherscan.io',
    icon: '🔴',
    color: '#ff0420',
    supportedWallets: ['MetaMask', 'WalletConnect', 'Coinbase']
  },
  Polygon: {
    name: 'Polygon',
    chainId: 137,
    symbol: 'MATIC',
    rpcUrl: 'https://polygon-rpc.com',
    explorerUrl: 'https://polygonscan.com',
    icon: '🟣',
    color: '#8247e5',
    supportedWallets: ['MetaMask', 'WalletConnect', 'Trust']
  },
  Base: {
    name: 'Base',
    chainId: 8453,
    symbol: 'ETH',
    rpcUrl: 'https://mainnet.base.org',
    explorerUrl: 'https://basescan.org',
    icon: '🔵',
    color: '#0052ff',
    supportedWallets: ['MetaMask', 'WalletConnect', 'Coinbase']
  },
  Avalanche: {
    name: 'Avalanche',
    chainId: 43114,
    symbol: 'AVAX',
    rpcUrl: 'https://api.avax.network/ext/bc/C/rpc',
    explorerUrl: 'https://snowtrace.io',
    icon: '🔺',
    color: '#e84142',
    supportedWallets: ['MetaMask', 'WalletConnect']
  },
  Solana: {
    name: 'Solana',
    chainId: 'solana-mainnet',
    symbol: 'SOL',
    rpcUrl: 'https://api.mainnet-beta.solana.com',
    explorerUrl: 'https://explorer.solana.com',
    icon: '◉',
    color: '#9945ff',
    supportedWallets: ['Phantom', 'Coinbase']
  },
  ICP: {
    name: 'Internet Computer',
    chainId: 'icp-mainnet',
    symbol: 'ICP',
    rpcUrl: 'https://ic0.app',
    explorerUrl: 'https://dashboard.internetcomputer.org',
    icon: '∞',
    color: '#3b00b9',
    supportedWallets: ['InternetIdentity', 'Plug', 'Stoic', 'Manual']
  }
}

class MultiChainWalletService {
  private wallet: MultiChainWallet = {
    addresses: [],
    lastSyncAt: 0
  }

  private listeners: Array<(wallet: MultiChainWallet) => void> = []
  private initialized = false
  private initializationPromise: Promise<void> | null = null

  constructor() {
    // Start initialization but don't block constructor
    this.initializationPromise = this.initialize()
    
    // PERFORMANCE: Setup cleanup on page unload
    if (typeof window !== 'undefined') {
      const cleanup = performanceService.addEventListenerSafely(
        window,
        'beforeunload',
        () => {
          this.cleanup()
        }
      )
      
      // Store cleanup function for manual cleanup
      ;(this as any)._cleanup = cleanup
    }
  }

  // SECURITY: Async initialization with secure storage
  private async initialize(): Promise<void> {
    if (this.initialized) return
    
    try {
      await this.loadWalletFromStorage()
      this.initialized = true
      console.log('🔒 MultiChainWalletService initialized securely')
    } catch (error) {
      console.error('❌ Failed to initialize MultiChainWalletService:', error)
      // Continue with empty wallet for security
      this.wallet = {
        addresses: [],
        lastSyncAt: Date.now()
      }
      this.initialized = true
    }
  }

  // SECURITY: Ensure initialization before any wallet operations
  async ensureInitialized(): Promise<void> {
    if (!this.initialized && this.initializationPromise) {
      await this.initializationPromise
    }
  }

  // Event listeners
  addListener(callback: (wallet: MultiChainWallet) => void) {
    this.listeners.push(callback)
  }

  removeListener(callback: (wallet: MultiChainWallet) => void) {
    this.listeners = this.listeners.filter(l => l !== callback)
  }

  // PERFORMANCE: Optimized listener notification with error handling
  private notifyListeners() {
    // PERFORMANCE: Use requestAnimationFrame for DOM updates
    performanceService.batchDomOperations([
      () => {
        this.listeners.forEach(callback => {
          try {
            callback(this.wallet)
          } catch (error) {
            console.error('Wallet listener callback failed:', error)
          }
        })
      }
    ])
  }

  // Get current wallet state
  async getWallet(): Promise<MultiChainWallet> {
    await this.ensureInitialized()
    return { ...this.wallet }
  }

  async getAddressForChain(chain: ChainType): Promise<WalletAddress | undefined> {
    await this.ensureInitialized()
    return this.wallet.addresses.find(addr => addr.chain === chain)
  }

  async getConnectedChains(): Promise<ChainType[]> {
    await this.ensureInitialized()
    return this.wallet.addresses
      .filter(addr => addr.isConnected)
      .map(addr => addr.chain)
  }

  // Add or update wallet address
  async addWalletAddress(address: WalletAddress): Promise<void> {
    await this.ensureInitialized()
    const existingIndex = this.wallet.addresses.findIndex(
      addr => addr.chain === address.chain
    )

    if (existingIndex >= 0) {
      this.wallet.addresses[existingIndex] = {
        ...this.wallet.addresses[existingIndex],
        ...address,
        lastUpdated: Date.now()
      }
    } else {
      this.wallet.addresses.push({
        ...address,
        lastUpdated: Date.now()
      })
    }

    // Set as primary if it's the first connected wallet
    if (!this.wallet.primaryWallet && address.isConnected) {
      this.wallet.primaryWallet = address
    }

    await this.saveWalletToStorage()
    this.notifyListeners()
  }

  // Connect to wallet for specific chain
  async connectWallet(chain: ChainType, walletType: WalletType): Promise<string> {
    try {
      let address: string

      switch (walletType) {
        case 'MetaMask':
          address = await this.connectMetaMask(chain)
          break
        case 'Phantom':
          address = await this.connectPhantom()
          break
        case 'WalletConnect':
          address = await this.connectWalletConnect(chain)
          break
        case 'Coinbase':
          address = await this.connectCoinbase(chain)
          break
        case 'InternetIdentity':
          address = await this.connectInternetIdentity()
          break
        case 'Plug':
          address = await this.connectPlug()
          break
        case 'Stoic':
          address = await this.connectStoic()
          break
        default:
          throw new Error(`${walletType} connection not implemented yet`)
      }

      await this.addWalletAddress({
        chain,
        address,
        isConnected: true,
        walletType,
        lastUpdated: Date.now()
      })

      // Fetch balance after connection
      this.updateBalance(chain)

      return address
    } catch (error) {
      console.error(`Failed to connect ${walletType} for ${chain}:`, error)
      throw error
    }
  }

  // Manual address input
  async addManualAddress(chain: ChainType, address: string): Promise<void> {
    // Validate address format
    if (!this.validateAddress(chain, address)) {
      throw new Error(`Invalid ${chain} address format`)
    }

    await this.addWalletAddress({
      chain,
      address,
      isConnected: false,
      walletType: 'Manual',
      lastUpdated: Date.now()
    })

    // Fetch balance for manual address
    this.updateBalance(chain)
  }

  // Disconnect wallet for specific chain
  async disconnectWallet(chain: ChainType): Promise<void> {
    const addressIndex = this.wallet.addresses.findIndex(addr => addr.chain === chain)
    
    if (addressIndex >= 0) {
      this.wallet.addresses[addressIndex].isConnected = false
      
      // Update primary wallet if needed
      if (this.wallet.primaryWallet?.chain === chain) {
        this.wallet.primaryWallet = this.wallet.addresses.find(addr => 
          addr.isConnected && addr.chain !== chain
        )
      }

      await this.saveWalletToStorage()
      this.notifyListeners()
    }
  }

  // Remove wallet address
  async removeWalletAddress(chain: ChainType): Promise<void> {
    this.wallet.addresses = this.wallet.addresses.filter(addr => addr.chain !== chain)
    
    // Update primary wallet if needed
    if (this.wallet.primaryWallet?.chain === chain) {
      this.wallet.primaryWallet = this.wallet.addresses.find(addr => addr.isConnected)
    }

    await this.saveWalletToStorage()
    this.notifyListeners()
  }

  // PERFORMANCE: Optimized balance updates with caching
  async updateBalance(chain: ChainType): Promise<void> {
    const walletAddress = await this.getAddressForChain(chain)
    if (!walletAddress) return

    // PERFORMANCE: Check cache first to avoid unnecessary API calls
    const cacheKey = `balance_${chain}_${walletAddress.address}`
    const cachedBalance = performanceService.getCache<string>(cacheKey)
    
    if (cachedBalance && Date.now() - walletAddress.lastUpdated < 30000) {
      // Use cached balance if less than 30 seconds old
      walletAddress.balance = cachedBalance
      this.notifyListeners()
      return
    }

    try {
      performanceService.trackApiCall()
      const balance = await this.fetchBalance(chain, walletAddress.address)
      
      // Update balance and cache
      walletAddress.balance = balance
      walletAddress.lastUpdated = Date.now()
      performanceService.setCache(cacheKey, balance, 60000) // Cache for 1 minute

      await this.saveWalletToStorage()
      this.notifyListeners()
    } catch (error) {
      console.error(`Failed to update balance for ${chain}:`, error)
    }
  }

  // PERFORMANCE: Batch balance updates with rate limiting
  async updateAllBalances(): Promise<void> {
    if (this.wallet.addresses.length === 0) return
    
    // PERFORMANCE: Limit concurrent API calls to prevent rate limiting
    const batchSize = 3
    const batches: ChainType[][] = []
    
    for (let i = 0; i < this.wallet.addresses.length; i += batchSize) {
      const batch = this.wallet.addresses
        .slice(i, i + batchSize)
        .map(addr => addr.chain)
      batches.push(batch)
    }
    
    // Process batches sequentially with delay
    for (const batch of batches) {
      const promises = batch.map(chain => this.updateBalance(chain))
      await Promise.all(promises)
      
      // Small delay between batches to prevent overwhelming APIs
      if (batch !== batches[batches.length - 1]) {
        await new Promise(resolve => setTimeout(resolve, 1000))
      }
    }
    
    this.wallet.lastSyncAt = Date.now()
  }


  // Wallet connection implementations
  private async connectMetaMask(chain: ChainType): Promise<string> {
    if (typeof window === 'undefined' || !window.ethereum) {
      throw new Error('MetaMask not installed')
    }

    try {
      // Request account access
      const accounts = await window.ethereum.request({
        method: 'eth_requestAccounts'
      })

      if (!accounts || accounts.length === 0) {
        throw new Error('No accounts found')
      }

      // Switch to correct network if needed
      const chainConfig = SUPPORTED_CHAINS[chain]
      if (typeof chainConfig.chainId === 'number') {
        await this.switchEthereumChain(chainConfig.chainId)
      }

      return accounts[0]
    } catch (error: any) {
      throw new Error(`MetaMask connection failed: ${error.message}`)
    }
  }

  private async connectPhantom(): Promise<string> {
    if (typeof window === 'undefined' || !window.solana?.isPhantom) {
      throw new Error('Phantom wallet not installed')
    }

    try {
      const response = await window.solana.connect()
      return response.publicKey.toString()
    } catch (error: any) {
      throw new Error(`Phantom connection failed: ${error.message}`)
    }
  }

  private async connectWalletConnect(chain: ChainType): Promise<string> {
    // WalletConnect implementation would go here
    // For now, throw an error indicating it needs implementation
    throw new Error('WalletConnect integration coming soon')
  }

  private async connectCoinbase(chain: ChainType): Promise<string> {
    // Coinbase Wallet integration would go here
    throw new Error('Coinbase Wallet integration coming soon')
  }

  private async connectInternetIdentity(): Promise<string> {
    // Internet Identity integration would go here
    // For now, return a placeholder principal ID
    throw new Error('Internet Identity integration coming soon')
  }

  private async connectPlug(): Promise<string> {
    if (typeof window === 'undefined' || !window.ic?.plug) {
      throw new Error('Plug wallet not installed')
    }

    try {
      const result = await window.ic.plug.requestConnect()
      if (result) {
        const principal = await window.ic.plug.agent.getPrincipal()
        return principal.toString()
      } else {
        throw new Error('Plug connection rejected')
      }
    } catch (error: any) {
      throw new Error(`Plug connection failed: ${error.message}`)
    }
  }

  private async connectStoic(): Promise<string> {
    // Stoic wallet integration would go here
    throw new Error('Stoic wallet integration coming soon')
  }


  // Helper methods
  private async switchEthereumChain(chainId: number): Promise<void> {
    const hexChainId = `0x${chainId.toString(16)}`
    
    try {
      await window.ethereum.request({
        method: 'wallet_switchEthereumChain',
        params: [{ chainId: hexChainId }]
      })
    } catch (error: any) {
      if (error.code === 4902) {
        // Chain not added to wallet, add it
        // Implementation depends on the specific chain
      }
    }
  }

  private validateAddress(chain: ChainType, address: string): boolean {
    switch (chain) {
      case 'Bitcoin':
        return /^[13][a-km-zA-HJ-NP-Z1-9]{25,34}$|^bc1[a-z0-9]{39,59}$/.test(address)
      case 'Ethereum':
      case 'Arbitrum':
      case 'Optimism':
      case 'Polygon':
      case 'Base':
      case 'Avalanche':
        return /^0x[a-fA-F0-9]{40}$/.test(address)
      case 'Solana':
        return /^[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(address)
      case 'ICP':
        return /^[a-z0-9]{5}-[a-z0-9]{5}-[a-z0-9]{5}-[a-z0-9]{5}-[a-z0-9]{3}$/.test(address)
      default:
        return false
    }
  }

  private async fetchBalance(chain: ChainType, address: string): Promise<string> {
    // This is a placeholder - in production, you'd integrate with actual blockchain APIs
    // For development, return mock data
    const mockBalances: Record<ChainType, string> = {
      Bitcoin: '0.05',
      Ethereum: '1.23',
      Arbitrum: '0.89',
      Optimism: '0.67',
      Polygon: '245.0',
      Base: '0.34',
      Avalanche: '12.5',
      Solana: '45.2',
      ICP: '12.5'
    }
    
    return mockBalances[chain] || '0.00'
  }

  // SECURITY: Secure storage methods using encryption
  private async loadWalletFromStorage(): Promise<void> {
    try {
      // Ensure secure storage is initialized
      if (!secureStorageService.isEncryptionReady()) {
        // Try to load existing key first
        const keyLoaded = await secureStorageService.loadExistingKey()
        if (!keyLoaded) {
          // Initialize with anonymous encryption
          await secureStorageService.initializeEncryption()
        }
      }

      // Load encrypted wallet data
      const stored = await secureStorageService.getSecureItem<MultiChainWallet>('multichain_wallet')
      if (stored) {
        this.wallet = stored
        console.log('🔒 Wallet loaded securely from encrypted storage')
      } else {
        // Check for old unencrypted data and migrate
        await this.migrateFromUnencryptedStorage()
      }
    } catch (error) {
      console.error('❌ Failed to load wallet from secure storage:', error)
      // Fallback to empty wallet for security
      this.wallet = {
        addresses: [],
        lastSyncAt: Date.now()
      }
    }
  }

  private async saveWalletToStorage(): Promise<void> {
    try {
      // Ensure secure storage is initialized
      if (!secureStorageService.isEncryptionReady()) {
        await secureStorageService.initializeEncryption()
      }

      // Save encrypted wallet data
      const success = await secureStorageService.setSecureItem('multichain_wallet', this.wallet)
      if (success) {
        console.log('🔒 Wallet saved securely to encrypted storage')
      } else {
        throw new Error('Failed to save wallet to secure storage')
      }
    } catch (error) {
      console.error('❌ Failed to save wallet to secure storage:', error)
      throw error // Re-throw to handle upstream
    }
  }

  // SECURITY: Migrate old unencrypted data to secure storage
  private async migrateFromUnencryptedStorage(): Promise<void> {
    try {
      const oldData = localStorage.getItem('deflow_multichain_wallet')
      if (oldData) {
        console.log('🔄 Migrating wallet data to secure storage...')
        
        // Parse old unencrypted data
        const oldWallet = JSON.parse(oldData) as MultiChainWallet
        
        // Save to secure storage
        const success = await secureStorageService.setSecureItem('multichain_wallet', oldWallet)
        
        if (success) {
          // Remove old unencrypted data
          localStorage.removeItem('deflow_multichain_wallet')
          this.wallet = oldWallet
          console.log('✅ Wallet migration to secure storage completed')
        } else {
          console.error('❌ Failed to migrate wallet data')
        }
      }
    } catch (error) {
      console.error('❌ Error during wallet migration:', error)
      // Remove potentially corrupted old data
      localStorage.removeItem('deflow_multichain_wallet')
    }
  }

  // SECURITY: Clear all wallet data securely with performance cleanup
  async clearWalletData(): Promise<void> {
    try {
      // Clear secure storage
      secureStorageService.removeSecureItem('multichain_wallet')
      
      // Clear any remaining unencrypted data
      localStorage.removeItem('deflow_multichain_wallet')
      
      // PERFORMANCE: Clear cached balance data
      this.wallet.addresses.forEach(addr => {
        const cacheKey = `balance_${addr.chain}_${addr.address}`
        performanceService.setCache(cacheKey, null, 0) // Expire cache immediately
      })
      
      // Reset wallet state
      this.wallet = {
        addresses: [],
        lastSyncAt: Date.now()
      }
      
      console.log('🧹 Wallet data cleared securely')
    } catch (error) {
      console.error('❌ Failed to clear wallet data:', error)
      throw error
    }
  }
  
  // PERFORMANCE: Manual cleanup method
  cleanup(): void {
    // Clear any cached data
    this.wallet.addresses.forEach(addr => {
      const cacheKey = `balance_${addr.chain}_${addr.address}`
      performanceService.setCache(cacheKey, null, 0)
    })
    
    // Clear listeners
    this.listeners.length = 0
    
    // Call window cleanup if available
    if ((this as any)._cleanup) {
      (this as any)._cleanup()
    }
    
    console.log('🧹 MultiChainWalletService cleanup completed')
  }
}

// Extend window object for wallet types
declare global {
  interface Window {
    ethereum?: any
    solana?: any
    ic?: {
      plug?: {
        requestConnect: () => Promise<boolean>
        agent: {
          getPrincipal: () => Promise<{ toString: () => string }>
        }
      }
    }
  }
}

export default new MultiChainWalletService()