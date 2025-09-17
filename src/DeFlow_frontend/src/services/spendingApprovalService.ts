/**
 * Spending Approval Service
 * Handles smart contract approvals for token spending limits
 * Integrates with ERC-20 approve() functions and custom DeFlow contracts
 */

export interface TokenApproval {
  token: string
  symbol: string
  contractAddress: string
  spenderAddress: string // DeFlow automation contract
  approvedAmount: string
  dailyLimit: string
  operationsAllowed: string[]
  chainId: number
  transactionHash?: string
  blockNumber?: number
  createdAt: number
  expiresAt?: number
  isActive: boolean
}

export interface ApprovalTransaction {
  hash: string
  token: string
  amount: string
  operation: string
  timestamp: number
  gasUsed: number
  gasPrice: string
  status: 'pending' | 'confirmed' | 'failed'
}

export interface SpendingLimits {
  daily: Record<string, number> // token -> remaining limit
  monthly: Record<string, number>
  lastReset: Record<string, number> // token -> timestamp
}

class SpendingApprovalService {
  private approvals: Map<string, TokenApproval> = new Map()
  private transactions: ApprovalTransaction[] = []
  private spendingLimits: SpendingLimits = {
    daily: {},
    monthly: {},
    lastReset: {}
  }

  // DeFlow automation contract addresses per chain
  private readonly DEFLOW_CONTRACTS: Record<number, string> = {
    1: '0x1234567890123456789012345678901234567890', // Ethereum mainnet
    137: '0x2345678901234567890123456789012345678901', // Polygon
    42161: '0x3456789012345678901234567890123456789012', // Arbitrum
    10: '0x4567890123456789012345678901234567890123' // Optimism
  }

  /**
   * Initialize service and load existing approvals
   */
  async initialize(userAddress: string): Promise<void> {
    try {
      // Load approvals from local storage or backend
      const stored = localStorage.getItem(`deflow_approvals_${userAddress}`)
      if (stored) {
        const data = JSON.parse(stored)
        this.approvals = new Map(data.approvals || [])
        this.spendingLimits = data.spendingLimits || this.spendingLimits
      }

      // Sync with on-chain state
      await this.syncWithChain(userAddress)
    } catch (error) {
      console.error('Failed to initialize spending approval service:', error)
    }
  }

  /**
   * Create token spending approvals on-chain
   */
  async createApprovals(
    userAddress: string,
    approvals: Array<{
      token: string
      symbol: string
      contractAddress: string
      maxAmount: string
      dailyLimit: string
      operationsAllowed: string[]
      chainId: number
    }>
  ): Promise<{success: boolean, transactions: string[], errors: string[]}> {
    const results = {
      success: true,
      transactions: [] as string[],
      errors: [] as string[]
    }

    for (const approval of approvals) {
      try {
        const txHash = await this.approveToken(
          userAddress,
          approval.contractAddress,
          approval.maxAmount,
          approval.chainId
        )

        if (txHash) {
          // Store approval locally
          const tokenApproval: TokenApproval = {
            token: approval.token,
            symbol: approval.symbol,
            contractAddress: approval.contractAddress,
            spenderAddress: this.DEFLOW_CONTRACTS[approval.chainId],
            approvedAmount: approval.maxAmount,
            dailyLimit: approval.dailyLimit,
            operationsAllowed: approval.operationsAllowed,
            chainId: approval.chainId,
            transactionHash: txHash,
            createdAt: Date.now(),
            isActive: true
          }

          this.approvals.set(`${approval.chainId}-${approval.contractAddress}`, tokenApproval)
          
          // Initialize spending limits
          this.spendingLimits.daily[approval.token] = parseFloat(approval.dailyLimit)
          this.spendingLimits.monthly[approval.token] = parseFloat(approval.maxAmount)
          this.spendingLimits.lastReset[approval.token] = Date.now()

          results.transactions.push(txHash)
        }
      } catch (error) {
        console.error(`Failed to approve ${approval.symbol}:`, error)
        results.errors.push(`${approval.symbol}: ${error instanceof Error ? error.message : 'Unknown error'}`)
        results.success = false
      }
    }

    // Save to storage
    await this.saveApprovals(userAddress)
    
    return results
  }

  /**
   * Check if operation is allowed within spending limits
   */
  canExecuteOperation(
    token: string,
    amount: number,
    operation: string
  ): {allowed: boolean, reason?: string, remainingLimit?: number} {
    const approval = this.getTokenApproval(token)
    if (!approval || !approval.isActive) {
      return {allowed: false, reason: 'No active approval for this token'}
    }

    // Check if operation is allowed
    if (!approval.operationsAllowed.includes(operation)) {
      return {allowed: false, reason: `Operation '${operation}' not permitted`}
    }

    // Check daily limit
    const now = Date.now()
    const lastReset = this.spendingLimits.lastReset[token] || 0
    const daysSinceReset = (now - lastReset) / (24 * 60 * 60 * 1000)
    
    if (daysSinceReset >= 1) {
      // Reset daily limit
      this.spendingLimits.daily[token] = parseFloat(approval.dailyLimit)
      this.spendingLimits.lastReset[token] = now
    }

    const remainingDaily = this.spendingLimits.daily[token] || 0
    if (amount > remainingDaily) {
      return {
        allowed: false, 
        reason: 'Exceeds daily spending limit',
        remainingLimit: remainingDaily
      }
    }

    // Check total approval amount
    const remainingApproval = parseFloat(approval.approvedAmount)
    if (amount > remainingApproval) {
      return {
        allowed: false,
        reason: 'Exceeds total approved amount',
        remainingLimit: remainingApproval
      }
    }

    return {allowed: true, remainingLimit: Math.min(remainingDaily, remainingApproval)}
  }

  /**
   * Record a spending transaction
   */
  async recordSpending(
    token: string,
    amount: number,
    operation: string,
    transactionHash: string
  ): Promise<void> {
    // Update remaining limits
    if (this.spendingLimits.daily[token]) {
      this.spendingLimits.daily[token] -= amount
    }

    // Record transaction
    this.transactions.push({
      hash: transactionHash,
      token,
      amount: amount.toString(),
      operation,
      timestamp: Date.now(),
      gasUsed: 0, // Will be updated when confirmed
      gasPrice: '0',
      status: 'pending'
    })

    // Update approval amount
    const approval = this.getTokenApproval(token)
    if (approval) {
      const newAmount = Math.max(0, parseFloat(approval.approvedAmount) - amount)
      approval.approvedAmount = newAmount.toString()
      
      if (newAmount === 0) {
        approval.isActive = false
      }
    }

    // Save changes
    const userAddress = 'current_user' // Get from context
    await this.saveApprovals(userAddress)
  }

  /**
   * Get spending history and analytics
   */
  getSpendingAnalytics(token?: string): {
    totalSpent: number
    operationsCount: number
    avgTransactionSize: number
    topOperations: Array<{operation: string, count: number, totalAmount: number}>
    dailySpending: Record<string, number>
  } {
    const relevantTxs = token 
      ? this.transactions.filter(tx => tx.token === token)
      : this.transactions

    const totalSpent = relevantTxs.reduce((sum, tx) => sum + parseFloat(tx.amount), 0)
    const operationsCount = relevantTxs.length
    const avgTransactionSize = operationsCount > 0 ? totalSpent / operationsCount : 0

    // Group by operation type
    const operationStats = new Map<string, {count: number, total: number}>()
    relevantTxs.forEach(tx => {
      const current = operationStats.get(tx.operation) || {count: 0, total: 0}
      current.count += 1
      current.total += parseFloat(tx.amount)
      operationStats.set(tx.operation, current)
    })

    const topOperations = Array.from(operationStats.entries())
      .map(([operation, stats]) => ({
        operation,
        count: stats.count,
        totalAmount: stats.total
      }))
      .sort((a, b) => b.totalAmount - a.totalAmount)

    // Daily spending over last 30 days
    const dailySpending: Record<string, number> = {}
    const thirtyDaysAgo = Date.now() - (30 * 24 * 60 * 60 * 1000)
    
    relevantTxs
      .filter(tx => tx.timestamp > thirtyDaysAgo)
      .forEach(tx => {
        const date = new Date(tx.timestamp).toISOString().split('T')[0]
        dailySpending[date] = (dailySpending[date] || 0) + parseFloat(tx.amount)
      })

    return {
      totalSpent,
      operationsCount,
      avgTransactionSize,
      topOperations,
      dailySpending
    }
  }

  /**
   * Revoke token approval
   */
  async revokeApproval(token: string, chainId: number): Promise<string> {
    const approvalKey = `${chainId}-${token}`
    const approval = this.approvals.get(approvalKey)
    
    if (!approval) {
      throw new Error('Approval not found')
    }

    try {
      // Set approval to 0 on-chain
      const txHash = await this.approveToken(
        'current_user_address', // Get from context
        approval.contractAddress,
        '0',
        chainId
      )

      // Update local state
      approval.isActive = false
      approval.approvedAmount = '0'

      // Save changes
      await this.saveApprovals('current_user_address')

      return txHash
    } catch (error) {
      console.error('Failed to revoke approval:', error)
      throw error
    }
  }

  /**
   * Get all active approvals for user
   */
  getActiveApprovals(): TokenApproval[] {
    return Array.from(this.approvals.values()).filter(approval => approval.isActive)
  }

  /**
   * Get specific token approval
   */
  getTokenApproval(token: string): TokenApproval | undefined {
    // Find by token symbol (could be improved with chain ID)
    return Array.from(this.approvals.values()).find(
      approval => approval.symbol === token && approval.isActive
    )
  }

  /**
   * Get remaining spending limits
   */
  getRemainingLimits(): Record<string, {daily: number, total: number}> {
    const limits: Record<string, {daily: number, total: number}> = {}
    
    for (const [token, dailyLimit] of Object.entries(this.spendingLimits.daily)) {
      const approval = this.getTokenApproval(token)
      limits[token] = {
        daily: dailyLimit,
        total: approval ? parseFloat(approval.approvedAmount) : 0
      }
    }
    
    return limits
  }

  // Private methods

  private async approveToken(
    userAddress: string,
    tokenContract: string,
    amount: string,
    chainId: number
  ): Promise<string> {
    // This would integrate with actual blockchain transaction
    // For now, return a mock transaction hash
    console.log(`Approving ${amount} tokens for contract ${tokenContract} on chain ${chainId}`)
    
    // Simulate transaction delay
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    // Return mock transaction hash
    return '0x' + Math.random().toString(16).slice(2, 66)
  }

  private async syncWithChain(userAddress: string): Promise<void> {
    // Sync local approvals with on-chain state
    // This would query the blockchain for actual approval amounts
    console.log(`Syncing approvals for ${userAddress}`)
  }

  private async saveApprovals(userAddress: string): Promise<void> {
    const data = {
      approvals: Array.from(this.approvals.entries()),
      spendingLimits: this.spendingLimits,
      timestamp: Date.now()
    }
    
    localStorage.setItem(`deflow_approvals_${userAddress}`, JSON.stringify(data))
  }
}

export default new SpendingApprovalService()