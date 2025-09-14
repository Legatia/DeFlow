import { Actor, HttpAgent } from '@dfinity/agent';
import { Principal } from '@dfinity/principal';

// Types for the FLOW token system
export interface Phase1Status {
  current_phase: { Phase1PreLaunch: null } | { Phase2AssetBacked: null };
  total_pre_launch_distributed: number;
  pool_asset_value_usd: number;
  launch_threshold_usd: number;
  progress_to_launch: number;
  btc_amount: number;
  ckbtc_staked: number;
  eligible_users: number;
}

export interface UserPhase1Balance {
  pre_launch_balance: number;
  tradeable_balance: number;
  total_balance: number;
  phase1_airdrop_received: number;
  phase1_activity_rewards: number;
  eligible_for_phase2_conversion: boolean;
  estimated_phase2_value: number;
}

export interface FlowTransaction {
  transaction_id: string;
  user: Principal;
  transaction_type: any;
  amount: number;
  timestamp: number;
  details: string;
}

// IDL for the pool canister FLOW token functions
const flowTokenIDL = ({ IDL }: any) => {
  const TokenLaunchPhase = IDL.Variant({
    'Phase1PreLaunch' : IDL.Null,
    'Phase2AssetBacked' : IDL.Null,
  });
  
  const Phase1Status = IDL.Record({
    'current_phase' : TokenLaunchPhase,
    'total_pre_launch_distributed' : IDL.Nat64,
    'pool_asset_value_usd' : IDL.Float64,
    'launch_threshold_usd' : IDL.Float64,
    'progress_to_launch' : IDL.Float64,
    'btc_amount' : IDL.Float64,
    'ckbtc_staked' : IDL.Float64,
    'eligible_users' : IDL.Nat64,
  });

  const UserPhase1Balance = IDL.Record({
    'pre_launch_balance' : IDL.Nat64,
    'tradeable_balance' : IDL.Nat64,
    'total_balance' : IDL.Nat64,
    'phase1_airdrop_received' : IDL.Nat64,
    'phase1_activity_rewards' : IDL.Nat64,
    'eligible_for_phase2_conversion' : IDL.Bool,
    'estimated_phase2_value' : IDL.Nat64,
  });

  return IDL.Service({
    'get_phase1_status' : IDL.Func([], [Phase1Status], ['query']),
    'get_user_phase1_balance' : IDL.Func([IDL.Principal], [UserPhase1Balance], ['query']),
    'phase1_early_adopter_airdrop' : IDL.Func(
      [IDL.Vec(IDL.Principal)], 
      [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })], 
      []
    ),
    'update_pool_assets' : IDL.Func(
      [IDL.Float64, IDL.Float64, IDL.Float64], 
      [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })], 
      []
    ),
    'phase1_first_defi_bonus' : IDL.Func(
      [IDL.Principal], 
      [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })], 
      []
    ),
    'phase1_referral_reward' : IDL.Func(
      [IDL.Principal, IDL.Principal], 
      [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })], 
      []
    ),
  });
};

class FlowTokenService {
  private actor: any = null;
  private agent: HttpAgent | null = null;

  private async getActor() {
    if (!this.actor) {
      // Initialize agent
      const host = process.env.DFX_NETWORK === 'ic' 
        ? 'https://ic0.app'
        : 'http://localhost:4943';
      
      this.agent = new HttpAgent({ host });
      
      // Fetch root key for local development
      if (process.env.DFX_NETWORK !== 'ic') {
        try {
          await this.agent.fetchRootKey();
        } catch (error) {
          console.warn('Unable to fetch root key. Check if local replica is running');
        }
      }

      // Get canister ID from environment
      const canisterId = process.env.CANISTER_ID_DEFLOW_POOL 
        || process.env.VITE_CANISTER_ID_DEFLOW_POOL
        || 'rrkah-fqaaa-aaaaa-aaaaq-cai'; // Default local canister ID

      this.actor = Actor.createActor(flowTokenIDL, {
        agent: this.agent,
        canisterId,
      });
    }

    return this.actor;
  }

  async getPhase1Status(): Promise<Phase1Status> {
    try {
      const actor = await this.getActor();
      const result = await actor.get_phase1_status();
      
      return {
        current_phase: result.current_phase,
        total_pre_launch_distributed: Number(result.total_pre_launch_distributed),
        pool_asset_value_usd: result.pool_asset_value_usd,
        launch_threshold_usd: result.launch_threshold_usd,
        progress_to_launch: result.progress_to_launch,
        btc_amount: result.btc_amount,
        ckbtc_staked: result.ckbtc_staked,
        eligible_users: Number(result.eligible_users)
      };
    } catch (error) {
      console.error('Error fetching Phase 1 status:', error);
      throw new Error(`Failed to fetch Phase 1 status: ${error}`);
    }
  }

  async getUserPhase1Balance(userPrincipal: Principal): Promise<UserPhase1Balance> {
    try {
      const actor = await this.getActor();
      const result = await actor.get_user_phase1_balance(userPrincipal);
      
      return {
        pre_launch_balance: Number(result.pre_launch_balance),
        tradeable_balance: Number(result.tradeable_balance),
        total_balance: Number(result.total_balance),
        phase1_airdrop_received: Number(result.phase1_airdrop_received),
        phase1_activity_rewards: Number(result.phase1_activity_rewards),
        eligible_for_phase2_conversion: result.eligible_for_phase2_conversion,
        estimated_phase2_value: Number(result.estimated_phase2_value)
      };
    } catch (error) {
      console.error('Error fetching user Phase 1 balance:', error);
      throw new Error(`Failed to fetch user balance: ${error}`);
    }
  }

  async triggerEarlyAdopterAirdrop(recipients: Principal[]): Promise<string> {
    try {
      const actor = await this.getActor();
      const result = await actor.phase1_early_adopter_airdrop(recipients);
      
      if ('Ok' in result) {
        return result.Ok;
      } else {
        throw new Error(result.Err);
      }
    } catch (error) {
      console.error('Error triggering early adopter airdrop:', error);
      throw new Error(`Failed to trigger airdrop: ${error}`);
    }
  }

  async updatePoolAssets(
    btcEquivalentUsd: number, 
    actualBtcAmount: number, 
    otherAssetsUsd: number
  ): Promise<string> {
    try {
      const actor = await this.getActor();
      const result = await actor.update_pool_assets(
        btcEquivalentUsd,
        actualBtcAmount,
        otherAssetsUsd
      );
      
      if ('Ok' in result) {
        return result.Ok;
      } else {
        throw new Error(result.Err);
      }
    } catch (error) {
      console.error('Error updating pool assets:', error);
      throw new Error(`Failed to update pool assets: ${error}`);
    }
  }

  async triggerFirstDeFiBonus(userPrincipal: Principal): Promise<string> {
    try {
      const actor = await this.getActor();
      const result = await actor.phase1_first_defi_bonus(userPrincipal);
      
      if ('Ok' in result) {
        return result.Ok;
      } else {
        throw new Error(result.Err);
      }
    } catch (error) {
      console.error('Error triggering first DeFi bonus:', error);
      throw new Error(`Failed to trigger first DeFi bonus: ${error}`);
    }
  }

  async triggerReferralReward(referrer: Principal, referred: Principal): Promise<string> {
    try {
      const actor = await this.getActor();
      const result = await actor.phase1_referral_reward(referrer, referred);
      
      if ('Ok' in result) {
        return result.Ok;
      } else {
        throw new Error(result.Err);
      }
    } catch (error) {
      console.error('Error triggering referral reward:', error);
      throw new Error(`Failed to trigger referral reward: ${error}`);
    }
  }

  // Utility functions
  formatFlowAmount(amount: number): string {
    return (amount / 100_000_000).toFixed(2); // Convert from 8 decimal places
  }

  formatCurrency(amount: number): string {
    return new Intl.NumberFormat('en-US', { 
      style: 'currency', 
      currency: 'USD',
      minimumFractionDigits: 2,
      maximumFractionDigits: 2
    }).format(amount);
  }

  getCurrentPhaseText(phase: Phase1Status['current_phase']): string {
    if ('Phase1PreLaunch' in phase) {
      return 'Phase 1: Pre-Launch';
    } else if ('Phase2AssetBacked' in phase) {
      return 'Phase 2: Asset-Backed';
    }
    return 'Unknown Phase';
  }
}

// Export singleton instance
export const flowTokenService = new FlowTokenService();
export default flowTokenService;