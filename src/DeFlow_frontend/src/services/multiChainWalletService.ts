// Multi-Chain Wallet Management Service
// Handles wallet connections and addresses for multiple blockchains

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
  | 'BSC'

export type WalletType = 
  | 'MetaMask' 
  | 'WalletConnect' 
  | 'Phantom' 
  | 'Coinbase' 
  | 'Trust' 
  | 'Manual' 
  | 'ICP'

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
    supportedWallets: ['Manual', 'ICP']
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
  BSC: {
    name: 'BNB Smart Chain',
    chainId: 56,
    symbol: 'BNB',
    rpcUrl: 'https://bsc-dataseed.binance.org',
    explorerUrl: 'https://bscscan.com',
    icon: '🟡',
    color: '#f3ba2f',
    supportedWallets: ['MetaMask', 'WalletConnect', 'Trust']
  }
}

class MultiChainWalletService {
  private wallet: MultiChainWallet = {
    addresses: [],
    lastSyncAt: 0
  }

  private listeners: Array<(wallet: MultiChainWallet) => void> = []

  constructor() {
    this.loadWalletFromStorage()
  }

  // Event listeners
  addListener(callback: (wallet: MultiChainWallet) => void) {
    this.listeners.push(callback)
  }

  removeListener(callback: (wallet: MultiChainWallet) => void) {
    this.listeners = this.listeners.filter(l => l !== callback)
  }

  private notifyListeners() {
    this.listeners.forEach(callback => callback(this.wallet))
  }

  // Get current wallet state
  getWallet(): MultiChainWallet {
    return { ...this.wallet }
  }

  getAddressForChain(chain: ChainType): WalletAddress | undefined {
    return this.wallet.addresses.find(addr => addr.chain === chain)
  }

  getConnectedChains(): ChainType[] {
    return this.wallet.addresses
      .filter(addr => addr.isConnected)
      .map(addr => addr.chain)
  }

  // Add or update wallet address
  async addWalletAddress(address: WalletAddress): Promise<void> {
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

  // Update balance for a specific chain
  async updateBalance(chain: ChainType): Promise<void> {
    const walletAddress = this.getAddressForChain(chain)
    if (!walletAddress) return

    try {
      const balance = await this.fetchBalance(chain, walletAddress.address)
      walletAddress.balance = balance
      walletAddress.lastUpdated = Date.now()

      await this.saveWalletToStorage()
      this.notifyListeners()
    } catch (error) {
      console.error(`Failed to update balance for ${chain}:`, error)
    }
  }

  // Update all balances
  async updateAllBalances(): Promise<void> {
    const promises = this.wallet.addresses.map(addr => 
      this.updateBalance(addr.chain)
    )
    await Promise.all(promises)
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
      case 'BSC':
        return /^0x[a-fA-F0-9]{40}$/.test(address)
      case 'Solana':
        return /^[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(address)
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
      BSC: '0.78'
    }
    
    return mockBalances[chain] || '0.00'
  }

  // Storage methods
  private loadWalletFromStorage(): void {
    try {
      const stored = localStorage.getItem('deflow_multichain_wallet')
      if (stored) {
        this.wallet = JSON.parse(stored)
      }
    } catch (error) {
      console.error('Failed to load wallet from storage:', error)
    }
  }

  private async saveWalletToStorage(): Promise<void> {
    try {
      localStorage.setItem('deflow_multichain_wallet', JSON.stringify(this.wallet))
    } catch (error) {
      console.error('Failed to save wallet to storage:', error)
    }
  }
}

// Extend window object for wallet types
declare global {
  interface Window {
    ethereum?: any
    solana?: any
  }
}

export default new MultiChainWalletService()