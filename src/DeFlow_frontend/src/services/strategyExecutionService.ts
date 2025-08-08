// Strategy Execution Service - Multi-chain DeFi strategy execution with wallet integration
import { Actor, HttpAgent } from '@dfinity/agent'
import { AuthClient } from '@dfinity/auth-client'
import { Principal } from '@dfinity/principal'
import multiChainWalletService, { MultiChainWallet, ChainType } from './multiChainWalletService'

// Strategy execution interfaces
export interface StrategyActivationRequest {
  strategy_id: string
  capital_amount: number
  wallet_addresses: Record<string, string>
  authorization_signature?: string
}

export interface StrategyExecutionResult {
  execution_id: string
  strategy_id: string
  success: boolean
  transaction_hashes: string[]
  gas_cost_usd: number
  actual_return: number
  error_message?: string
  executed_at: string
}

export interface WalletValidationRequest {
  wallet_addresses: Record<string, string>
}

export interface ValidationResult {
  is_valid: boolean
  validated_at: string
  validation_method: string
  error_message?: string
}

export interface WalletValidationResponse {
  validation_results: Record<string, ValidationResult>
  all_valid: boolean
}

export interface AuthorizationChallenge {
  authorization_id: string
  challenge_message: string
  expires_at: string
  signature_required: boolean
}

export interface ExecutionAuthorization {
  authorization_id: string
  strategy_id: string
  execution_amount: number
  signature?: string
  expires_at: string
}

// Backend canister interface for strategy execution
interface StrategyExecutionCanister {
  // Wallet validation
  validate_wallet_addresses: (req: WalletValidationRequest) => Promise<WalletValidationResponse>
  
  // Authorization management
  create_execution_authorization: (strategy_id: string, capital_amount: number) => Promise<AuthorizationChallenge>
  authorize_with_signature: (authorization_id: string, signature: string) => Promise<boolean>
  
  // Strategy activation and execution
  activate_strategy_with_wallets: (req: StrategyActivationRequest) => Promise<boolean>
  execute_strategy: (strategy_id: string, authorization_id?: string) => Promise<StrategyExecutionResult>
  get_strategy_executions: (strategy_id: string) => Promise<StrategyExecutionResult[]>
}

class StrategyExecutionService {
  private agent: HttpAgent | null = null
  private actor: Actor | null = null
  private canisterId = process.env.REACT_APP_STRATEGY_CANISTER_ID || 'rdmx6-jaaaa-aaaah-qdrqq-cai'

  constructor() {
    this.initializeActor()
  }

  private async initializeActor() {
    try {
      const authClient = await AuthClient.create()
      
      this.agent = new HttpAgent({
        host: process.env.REACT_APP_IC_HOST || 'http://localhost:4943'
      })

      if (process.env.NODE_ENV === 'development') {
        await this.agent.fetchRootKey()
      }

      // Create actor interface (simplified for demo)
      this.actor = Actor.createActor(
        // In production, import the actual canister interface
        { } as any, 
        {
          agent: this.agent,
          canisterId: this.canisterId
        }
      )
    } catch (error) {
      console.error('Failed to initialize strategy execution actor:', error)
    }
  }

  /**
   * Validate that user controls all wallet addresses required for strategy
   */
  async validateWalletAddresses(strategyChains: ChainType[]): Promise<WalletValidationResponse> {
    const wallet = multiChainWalletService.getWallet()
    
    // Get wallet addresses for required chains
    const walletAddresses: Record<string, string> = {}
    const missingChains: ChainType[] = []

    for (const chain of strategyChains) {
      const address = wallet.addresses.find(addr => addr.chain === chain)
      if (address) {
        walletAddresses[chain] = address.address
      } else {
        missingChains.push(chain)
      }
    }

    // Check for missing wallets
    if (missingChains.length > 0) {
      return {
        validation_results: {},
        all_valid: false
      }
    }

    try {
      if (this.actor) {
        // Call backend validation
        const response = await (this.actor as any).validate_wallet_addresses({
          wallet_addresses: walletAddresses
        })
        return response
      } else {
        // Fallback: simulate validation for development
        return this.simulateWalletValidation(walletAddresses)
      }
    } catch (error) {
      console.error('Wallet validation failed:', error)
      throw new Error(`Wallet validation failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
    }
  }

  /**
   * Create authorization challenge for strategy execution
   */
  async createExecutionAuthorization(
    strategyId: string, 
    capitalAmount: number
  ): Promise<AuthorizationChallenge> {
    try {
      if (this.actor) {
        const response = await (this.actor as any).create_execution_authorization(
          strategyId,
          capitalAmount
        )
        return response
      } else {
        // Simulate for development
        return {
          authorization_id: `auth_${Date.now()}`,
          challenge_message: this.generateChallengeMessage(strategyId, capitalAmount),
          expires_at: new Date(Date.now() + 5 * 60 * 1000).toISOString(),
          signature_required: true
        }
      }
    } catch (error) {
      console.error('Failed to create authorization:', error)
      throw new Error(`Authorization creation failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
    }
  }

  /**
   * Sign authorization challenge with user's wallet
   */
  async signAuthorizationChallenge(
    authorizationId: string,
    challengeMessage: string,
    primaryChain: ChainType
  ): Promise<string> {
    const wallet = multiChainWalletService.getWallet()
    const address = wallet.addresses.find(addr => addr.chain === primaryChain)

    if (!address || !address.isConnected) {
      throw new Error(`No connected wallet found for ${primaryChain}`)
    }

    try {
      // Request signature from connected wallet
      let signature: string

      if (primaryChain === 'Ethereum' || primaryChain === 'Arbitrum' || 
          primaryChain === 'Optimism' || primaryChain === 'Polygon' || 
          primaryChain === 'Base' || primaryChain === 'Avalanche') {
        // EVM signature
        signature = await this.signWithEVM(challengeMessage, address.address)
      } else if (primaryChain === 'Solana') {
        // Solana signature
        signature = await this.signWithSolana(challengeMessage, address.address)
      } else {
        throw new Error(`Signature not supported for chain: ${primaryChain}`)
      }

      // Submit signature to backend
      if (this.actor) {
        await (this.actor as any).authorize_with_signature(authorizationId, signature)
      }

      return signature
    } catch (error) {
      console.error('Failed to sign authorization:', error)
      throw new Error(`Signature failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
    }
  }

  /**
   * Activate strategy with validated wallets and authorization
   */
  async activateStrategyWithWallets(
    strategyId: string,
    capitalAmount: number,
    requiredChains: ChainType[],
    authorizationId?: string
  ): Promise<boolean> {
    // Get wallet addresses for required chains
    const wallet = multiChainWalletService.getWallet()
    const walletAddresses: Record<string, string> = {}

    for (const chain of requiredChains) {
      const address = wallet.addresses.find(addr => addr.chain === chain)
      if (!address) {
        throw new Error(`No wallet address found for required chain: ${chain}`)
      }
      walletAddresses[chain] = address.address
    }

    const request: StrategyActivationRequest = {
      strategy_id: strategyId,
      capital_amount: capitalAmount,
      wallet_addresses: walletAddresses,
      authorization_signature: authorizationId
    }

    try {
      if (this.actor) {
        const success = await (this.actor as any).activate_strategy_with_wallets(request)
        return success
      } else {
        // Simulate for development
        console.log('Simulating strategy activation:', request)
        return true
      }
    } catch (error) {
      console.error('Strategy activation failed:', error)
      throw new Error(`Strategy activation failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
    }
  }

  /**
   * Execute strategy with optional authorization
   */
  async executeStrategy(
    strategyId: string, 
    authorizationId?: string
  ): Promise<StrategyExecutionResult> {
    try {
      if (this.actor) {
        const result = await (this.actor as any).execute_strategy(strategyId, authorizationId)
        return result
      } else {
        // Simulate execution for development
        return {
          execution_id: `exec_${Date.now()}`,
          strategy_id: strategyId,
          success: true,
          transaction_hashes: [`0x${Math.random().toString(16).substr(2, 64)}`],
          gas_cost_usd: Math.random() * 50,
          actual_return: Math.random() * 100,
          executed_at: new Date().toISOString()
        }
      }
    } catch (error) {
      console.error('Strategy execution failed:', error)
      throw new Error(`Strategy execution failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
    }
  }

  /**
   * Get execution history for a strategy
   */
  async getStrategyExecutions(strategyId: string): Promise<StrategyExecutionResult[]> {
    try {
      if (this.actor) {
        return await (this.actor as any).get_strategy_executions(strategyId)
      } else {
        // Simulate for development
        return [
          {
            execution_id: `exec_${Date.now() - 86400000}`,
            strategy_id: strategyId,
            success: true,
            transaction_hashes: [`0x${Math.random().toString(16).substr(2, 64)}`],
            gas_cost_usd: 25.50,
            actual_return: 150.75,
            executed_at: new Date(Date.now() - 86400000).toISOString()
          }
        ]
      }
    } catch (error) {
      console.error('Failed to get strategy executions:', error)
      return []
    }
  }

  // Private helper methods

  private async signWithEVM(message: string, address: string): Promise<string> {
    if (typeof window !== 'undefined' && (window as any).ethereum) {
      try {
        const signature = await (window as any).ethereum.request({
          method: 'personal_sign',
          params: [message, address]
        })
        return signature
      } catch (error) {
        throw new Error(`EVM signature failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
      }
    } else {
      throw new Error('No Ethereum wallet detected')
    }
  }

  private async signWithSolana(message: string, address: string): Promise<string> {
    if (typeof window !== 'undefined' && (window as any).solana) {
      try {
        const encodedMessage = new TextEncoder().encode(message)
        const signature = await (window as any).solana.signMessage(encodedMessage, 'utf8')
        return signature.signature
      } catch (error) {
        throw new Error(`Solana signature failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
      }
    } else {
      throw new Error('No Solana wallet detected')
    }
  }

  private generateChallengeMessage(strategyId: string, capitalAmount: number): string {
    return `DeFlow Strategy Execution Authorization

Strategy ID: ${strategyId}
Execution Amount: $${capitalAmount.toLocaleString()}
Timestamp: ${Date.now()}

By signing this message, you authorize the execution of the above strategy with the specified capital amount.`
  }

  private simulateWalletValidation(walletAddresses: Record<string, string>): WalletValidationResponse {
    const validationResults: Record<string, ValidationResult> = {}
    let allValid = true

    for (const [chain, address] of Object.entries(walletAddresses)) {
      // Simulate validation based on address format
      const isValid = this.isValidAddressFormat(chain as ChainType, address)
      
      validationResults[chain] = {
        is_valid: isValid,
        validated_at: new Date().toISOString(),
        validation_method: 'format_check',
        error_message: isValid ? undefined : 'Invalid address format'
      }

      if (!isValid) allValid = false
    }

    return { validation_results: validationResults, all_valid: allValid }
  }

  private isValidAddressFormat(chain: ChainType, address: string): boolean {
    switch (chain) {
      case 'Bitcoin':
        return /^(1|3|bc1)[a-zA-HJ-NP-Z0-9]{25,62}$/.test(address)
      case 'Ethereum':
      case 'Arbitrum':
      case 'Optimism':
      case 'Polygon':
      case 'Base':
      case 'Avalanche':
        return /^0x[a-fA-F0-9]{40}$/.test(address)
      case 'Solana':
        return /^[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(address)
      default:
        return false
    }
  }
}

// Export singleton instance
export const strategyExecutionService = new StrategyExecutionService()
export default strategyExecutionService