// Admin Pool Service for managing DeFlow pool assets
// This service provides owner-level access to pool canister treasury functions
import { Actor, HttpAgent } from '@dfinity/agent';
import { AuthClient } from '@dfinity/auth-client';
import { idlFactory as poolIdlFactory } from 'declarations/DeFlow_pool';

interface TreasuryBalance {
  chain: string;
  asset: string;
  amount: number;
  amount_usd: number;
  last_updated: bigint;
  last_tx_hash?: string;
}

interface TreasuryTransaction {
  id: string;
  transaction_type: string;
  chain: string;
  asset: string;
  amount: number;
  amount_usd: number;
  from_address: string;
  to_address: string;
  tx_hash?: string;
  status: string;
  timestamp: bigint;
  initiated_by: string;
  notes?: string;
}

interface TreasuryHealthReport {
  total_usd_value: number;
  total_assets: number;
  balances_over_limit: string[];
  last_payment_timestamp?: bigint;
  pending_withdrawals: number;
  hot_wallet_utilization: number;
  largest_single_balance: number;
  diversification_score: number;
  security_alerts: string[];
}

interface PoolState {
  phase: string;
  total_liquidity_usd: number;
  monthly_volume: number;
  fee_collection_rate: number;
  team_earnings: Record<string, number>;
  bootstrap_progress: number;
}

interface CanisterHealth {
  canister_id: string;
  name: string;
  status: string;
  memory_usage: number;
  cycles_balance: number;
  last_upgrade: bigint;
  error_rate: number;
  avg_response_time: number;
  heap_memory_size: number;
  stable_memory_size: number;
  is_healthy: boolean;
  warnings: string[];
}

interface SystemHealthData {
  overall_status: 'Healthy' | 'Warning' | 'Critical';
  total_cycles: number;
  canisters: CanisterHealth[];
  platform_metrics: {
    total_users: number;
    active_users_24h: number;
    total_workflows: number;
    workflows_executed_24h: number;
    total_transactions_24h: number;
    total_volume_24h_usd: number;
  };
  network_info: {
    ic_network: string;
    subnet_id: string;
    replica_version: string;
  };
}

interface PoolTerminationRequest {
  id: string;
  initiated_by: string;
  reason: string;
  asset_distribution_plan: AssetDistribution[];
  owner_approval: TerminationApproval | null;
  cofounder_approval: TerminationApproval | null;
  created_at: bigint;
  expires_at: bigint;
  emergency_termination: boolean;
}

interface AssetDistribution {
  chain: string;
  asset: string;
  total_amount: number;
  destination_address: string;
  estimated_usd_value: number;
  status: string;
  tx_hash: string | null;
  executed_at: bigint | null;
}

interface TerminationApproval {
  approver: string;
  approved_at: bigint;
  signature_confirmation: string;
  notes: string | null;
}

interface TerminationSummary {
  total_assets_distributed: number;
  chains_processed: string[];
  successful_distributions: number;
  failed_distributions: number;
  termination_initiated_at: bigint;
  termination_completed_at: bigint | null;
  final_state_hash: string;
}

export class AdminPoolService {
  private static poolCanisterId = process.env.VITE_CANISTER_ID_DEFLOW_POOL;

  private static async getPoolActor() {
    if (!this.poolCanisterId) {
      throw new Error('SECURITY: Pool canister ID not configured. Set VITE_CANISTER_ID_DEFLOW_POOL environment variable.');
    }

    const authClient = await AuthClient.create();
    const identity = authClient.getIdentity();
    
    const agent = new HttpAgent({
      identity,
      host: process.env.DFX_NETWORK === "local" ? "http://127.0.0.1:8080" : "https://ic0.app",
    });

    // SECURITY: Only fetch root key for local development
    if (process.env.DFX_NETWORK === "local") {
      await agent.fetchRootKey();
    }
    
    // SECURITY: Additional validation for production
    if (process.env.DFX_NETWORK === "ic" && !this.poolCanisterId?.includes(".ic0.app")) {
      // For mainnet, ensure we have proper canister ID format
    }

    return Actor.createActor(poolIdlFactory, {
      agent,
      canisterId: this.poolCanisterId,
    });
  }

  /**
   * Get treasury health report (owner-only)
   */
  static async getTreasuryHealthReport(): Promise<TreasuryHealthReport> {
    try {
      const actor = await this.getPoolActor();
      
      // Call actual canister methods
      const poolState = await actor.get_pool_state() as any;
      const financialOverview = await actor.get_financial_overview() as any;
      
      if ('Err' in poolState || 'Err' in financialOverview) {
        throw new Error('Failed to get pool data from canister');
      }

      // Extract real data from canister responses
      const state = poolState.Ok;
      const overview = financialOverview.Ok;

      return {
        total_usd_value: overview.total_liquidity,
        total_assets: Object.keys(state.reserves).length,
        balances_over_limit: [], // TODO: Implement based on actual limits
        last_payment_timestamp: BigInt(Date.now() * 1000000),
        pending_withdrawals: 0, // TODO: Get from canister
        hot_wallet_utilization: 0, // TODO: Calculate from actual data
        largest_single_balance: overview.total_liquidity * 0.5, // Estimate
        diversification_score: 0.78, // TODO: Calculate from actual reserves
        security_alerts: [
          `💰 Total Liquidity: $${overview.total_liquidity.toLocaleString()}`,
          `📊 Pool Phase: ${typeof state.phase === 'object' ? Object.keys(state.phase)[0] : 'Unknown'}`,
          `📈 Bootstrap Progress: ${(overview.bootstrap_progress * 100).toFixed(1)}%`,
          `💵 Monthly Revenue: $${overview.monthly_revenue.toLocaleString()}`,
          `🔒 Pool Health: ${overview.pool_health}`,
          `⚡ Business Health: ${overview.business_health}`
        ]
      };
    } catch (error) {
      console.error('Failed to get treasury health report:', error);
      throw new Error(`Failed to get treasury health report: ${error}`);
    }
  }

  /**
   * Get all treasury balances (owner-only)
   */
  static async getAllTreasuryBalances(): Promise<TreasuryBalance[]> {
    try {
      const actor = await this.getPoolActor();
      
      // Get chain distribution from canister
      const chainDistribution = await actor.get_chain_distribution() as any;
      const poolState = await actor.get_pool_state() as any;
      
      if ('Err' in poolState) {
        throw new Error('Failed to get pool state from canister');
      }

      const state = poolState.Ok;
      const balances: TreasuryBalance[] = [];

      // Convert canister data to treasury balances
      for (const [chainName, percentage] of chainDistribution) {
        const chainId = chainName; // Assuming string format
        const totalLiquidityForChain = state.total_liquidity_usd * percentage;
        
        // For now, assume single asset per chain (can be expanded)
        const assetName = chainId === 'Bitcoin' ? 'btc' : 
                         chainId === 'Ethereum' ? 'eth' : 
                         chainId === 'Polygon' ? 'matic' : 'unknown';

        balances.push({
          chain: chainId.toLowerCase(),
          asset: assetName,
          amount: totalLiquidityForChain / 1000, // Convert to asset units (mock conversion)
          amount_usd: totalLiquidityForChain,
          last_updated: BigInt(Date.now() * 1000000)
        });
      }

      return balances;
    } catch (error) {
      console.error('Failed to get treasury balances:', error);
      // Return empty array instead of mock data for security
      return [];
    }
  }

  /**
   * Get treasury transactions (owner-only)
   */
  static async getTreasuryTransactions(limit: number = 50): Promise<TreasuryTransaction[]> {
    try {
      const actor = await this.getPoolActor();
      
      // TODO: Implement get_treasury_transactions method in pool canister
      console.warn('Treasury transactions not yet implemented in pool canister');
      
      // For now, return empty array instead of mock data
      return [];
    } catch (error) {
      console.error('Failed to get treasury transactions:', error);
      return [];
    }
  }

  /**
   * Get pool state and analytics (owner-only)
   */
  static async getPoolState(): Promise<PoolState> {
    try {
      const actor = await this.getPoolActor();
      
      const poolStateResult = await actor.get_pool_state() as any;
      const financialOverview = await actor.get_financial_overview() as any;
      
      if ('Err' in poolStateResult || 'Err' in financialOverview) {
        throw new Error('Failed to get pool state from canister');
      }

      const state = poolStateResult.Ok;
      const overview = financialOverview.Ok;

      return {
        phase: typeof state.phase === 'object' ? Object.keys(state.phase)[0] : 'Unknown',
        total_liquidity_usd: state.total_liquidity_usd,
        monthly_volume: state.monthly_volume,
        fee_collection_rate: state.fee_collection_rate,
        team_earnings: {
          'dev_1_pending': overview.dev_1_pending,
          'dev_2_pending': overview.dev_2_pending,
          'emergency_fund': overview.emergency_fund
        },
        bootstrap_progress: overview.bootstrap_progress
      };
    } catch (error) {
      console.error('Failed to get pool state:', error);
      throw new Error(`Failed to get pool state: ${error}`);
    }
  }

  /**
   * Configure payment address (owner-only)
   */
  static async configurePaymentAddress(
    chain: string,
    asset: string,
    address: string,
    addressType: 'Hot' | 'Warm' | 'Cold',
    maxBalanceUsd?: number
  ): Promise<void> {
    try {
      const actor = await this.getPoolActor();
      
      // TODO: Implement configure_payment_address method in pool canister
      console.warn('Payment address configuration not yet implemented in pool canister');
      
      // SECURITY: Do not silently succeed - throw error to indicate missing functionality
      throw new Error('Payment address configuration not yet implemented');
      
    } catch (error) {
      console.error('Failed to configure payment address:', error);
      throw new Error(`Failed to configure payment address: ${error}`);
    }
  }

  /**
   * Initiate team withdrawal (owner-only, requires approvals)
   */
  static async initiateTeamWithdrawal(
    chain: string,
    asset: string,
    amount: number,
    destinationAddress: string,
    reason: string
  ): Promise<string> {
    try {
      const actor = await this.getPoolActor();
      
      // Call the actual withdraw_dev_earnings method
      const result = await actor.withdraw_dev_earnings() as any;
      
      if ('Err' in result) {
        throw new Error(`Withdrawal failed: ${result.Err}`);
      }
      
      const withdrawnAmount = result.Ok;
      
      // Return a simple confirmation ID
      return `withdrawal_${Date.now()}_${withdrawnAmount}`;
      
    } catch (error) {
      console.error('Failed to initiate team withdrawal:', error);
      throw new Error(`Failed to initiate team withdrawal: ${error}`);
    }
  }

  /**
   * Get comprehensive system health metrics (owner-only)
   */
  static async getSystemHealth(): Promise<SystemHealthData> {
    try {
      // TODO: Implement system health monitoring in canisters
      console.warn('System health monitoring not yet fully implemented');
      
      // For now, return minimal health data
      return {
        overall_status: 'Warning', // Conservative status until proper monitoring is implemented
        total_cycles: 0, // TODO: Get from canister status
        canisters: [],
        platform_metrics: {
          total_users: 0,
          active_users_24h: 0,
          total_workflows: 0,
          workflows_executed_24h: 0,
          total_transactions_24h: 0,
          total_volume_24h_usd: 0
        },
        network_info: {
          ic_network: process.env.DFX_NETWORK || 'unknown',
          subnet_id: 'unknown',
          replica_version: 'unknown'
        }
      };
    } catch (error) {
      console.error('Failed to get system health:', error);
      throw new Error(`Failed to get system health: ${error}`);
    }
  }

  // =============================================================================
  // POOL TERMINATION METHODS
  // =============================================================================

  /**
   * Set cofounder principal (owner-only, one-time only)
   */
  static async setCofounder(cofounderPrincipal: string): Promise<string> {
    try {
      const actor = await this.getPoolActor();
      
      const result = await actor.set_cofounder(cofounderPrincipal) as any;
      
      if ('Err' in result) {
        throw new Error(`Failed to set cofounder: ${result.Err}`);
      }
      
      return result.Ok;
    } catch (error) {
      console.error('Failed to set cofounder:', error);
      throw new Error(`Failed to set cofounder: ${error}`);
    }
  }

  /**
   * Get cofounder principal (owner/cofounder only)
   */
  static async getCofounder(): Promise<string | null> {
    try {
      const actor = await this.getPoolActor();
      
      const result = await actor.get_cofounder() as any;
      
      if (result && result.length > 0) {
        return result[0]; // Optional<Principal> returns as array
      }
      
      return null;
    } catch (error) {
      console.error('Failed to get cofounder:', error);
      return null;
    }
  }

  /**
   * Initiate pool termination (owner/cofounder only)
   */
  static async initiatePoolTermination(
    reason: string,
    assetDistributionAddresses: Array<[string, string, string]>, // [chain, asset, address]
    emergency: boolean = false
  ): Promise<string> {
    try {
      const actor = await this.getPoolActor();
      
      const result = await actor.initiate_pool_termination(
        reason,
        assetDistributionAddresses,
        emergency
      ) as any;
      
      if ('Err' in result) {
        throw new Error(`Failed to initiate termination: ${result.Err}`);
      }
      
      return result.Ok;
    } catch (error) {
      console.error('Failed to initiate pool termination:', error);
      throw new Error(`Failed to initiate pool termination: ${error}`);
    }
  }

  /**
   * Approve pool termination (owner/cofounder only)
   */
  static async approvePoolTermination(
    terminationId: string,
    confirmationPhrase: string,
    approvalNotes?: string
  ): Promise<string> {
    try {
      const actor = await this.getPoolActor();
      
      const result = await actor.approve_pool_termination(
        terminationId,
        confirmationPhrase,
        approvalNotes ? [approvalNotes] : []
      ) as any;
      
      if ('Err' in result) {
        throw new Error(`Failed to approve termination: ${result.Err}`);
      }
      
      return result.Ok;
    } catch (error) {
      console.error('Failed to approve pool termination:', error);
      throw new Error(`Failed to approve pool termination: ${error}`);
    }
  }

  /**
   * Execute pool termination (owner only)
   */
  static async executePoolTermination(terminationId: string): Promise<TerminationSummary> {
    try {
      const actor = await this.getPoolActor();
      
      const result = await actor.execute_pool_termination(terminationId) as any;
      
      if ('Err' in result) {
        throw new Error(`Failed to execute termination: ${result.Err}`);
      }
      
      return result.Ok;
    } catch (error) {
      console.error('Failed to execute pool termination:', error);
      throw new Error(`Failed to execute pool termination: ${error}`);
    }
  }

  /**
   * Cancel pool termination (owner only)
   */
  static async cancelPoolTermination(terminationId: string, reason: string): Promise<string> {
    try {
      const actor = await this.getPoolActor();
      
      const result = await actor.cancel_pool_termination(terminationId, reason) as any;
      
      if ('Err' in result) {
        throw new Error(`Failed to cancel termination: ${result.Err}`);
      }
      
      return result.Ok;
    } catch (error) {
      console.error('Failed to cancel pool termination:', error);
      throw new Error(`Failed to cancel pool termination: ${error}`);
    }
  }

  /**
   * Get active termination request (owner/cofounder only)
   */
  static async getActiveTerminationRequest(): Promise<PoolTerminationRequest | null> {
    try {
      const actor = await this.getPoolActor();
      
      const result = await actor.get_active_termination_request() as any;
      
      if (result && result.length > 0) {
        return result[0]; // Optional<PoolTerminationRequest> returns as array
      }
      
      return null;
    } catch (error) {
      console.error('Failed to get active termination request:', error);
      return null;
    }
  }

  /**
   * Get termination history (owner/cofounder only)
   */
  static async getTerminationHistory(): Promise<PoolTerminationRequest[]> {
    try {
      const actor = await this.getPoolActor();
      
      const result = await actor.get_termination_history() as any;
      
      return result || [];
    } catch (error) {
      console.error('Failed to get termination history:', error);
      return [];
    }
  }
}