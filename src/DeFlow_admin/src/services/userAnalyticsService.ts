// User Analytics Service - API calls to DeFlow_backend for user analytics

import { Actor, HttpAgent } from '@dfinity/agent';

export interface UserAnalytics {
  total_users: bigint;
  active_users_30d: bigint;
  new_users_7d: bigint;
  users_by_tier: [string, bigint][];
  total_workflows_created: bigint;
  total_executions: bigint;
  executions_last_24h: bigint;
  average_workflows_per_user: number;
  top_node_types: [string, bigint][];
  total_revenue_usd: number;
  monthly_recurring_revenue: number;
  user_retention_rate: number;
  last_updated: bigint;
}

export interface UserDetail {
  principal_id: string;
  subscription_tier: { Standard: null } | { Premium: null } | { Pro: null };
  created_at: bigint;
  last_activity: bigint;
  days_since_registration: bigint;
  total_workflows: bigint;
  total_executions: bigint;
  monthly_executions: bigint;
  monthly_volume_usd: number;
  total_volume_usd: number;
  is_active: boolean;
  payment_history_count: bigint;
  total_paid_usd: number;
  preferred_nodes: string[];
}

export interface SubscriptionTier {
  Standard: null;
  Premium: null;
  Pro: null;
}

// IDL definition for user analytics endpoints
const idl = ({ IDL }: any) => {
  const SubscriptionTier = IDL.Variant({
    Standard: IDL.Null,
    Premium: IDL.Null,
    Pro: IDL.Null,
  });

  const UserAnalytics = IDL.Record({
    total_users: IDL.Nat64,
    active_users_30d: IDL.Nat64,
    new_users_7d: IDL.Nat64,
    users_by_tier: IDL.Vec(IDL.Tuple(IDL.Text, IDL.Nat64)),
    total_workflows_created: IDL.Nat64,
    total_executions: IDL.Nat64,
    executions_last_24h: IDL.Nat64,
    average_workflows_per_user: IDL.Float64,
    top_node_types: IDL.Vec(IDL.Tuple(IDL.Text, IDL.Nat64)),
    total_revenue_usd: IDL.Float64,
    monthly_recurring_revenue: IDL.Float64,
    user_retention_rate: IDL.Float64,
    last_updated: IDL.Nat64,
  });

  const UserDetail = IDL.Record({
    principal_id: IDL.Text,
    subscription_tier: SubscriptionTier,
    created_at: IDL.Nat64,
    last_activity: IDL.Nat64,
    days_since_registration: IDL.Nat64,
    total_workflows: IDL.Nat64,
    total_executions: IDL.Nat64,
    monthly_executions: IDL.Nat64,
    monthly_volume_usd: IDL.Float64,
    total_volume_usd: IDL.Float64,
    is_active: IDL.Bool,
    payment_history_count: IDL.Nat64,
    total_paid_usd: IDL.Float64,
    preferred_nodes: IDL.Vec(IDL.Text),
  });

  return IDL.Service({
    get_user_analytics: IDL.Func([], [UserAnalytics], ['query']),
    get_all_users_details: IDL.Func([], [IDL.Vec(UserDetail)], ['query']),
    get_users_filtered: IDL.Func(
      [IDL.Opt(SubscriptionTier), IDL.Bool, IDL.Opt(IDL.Nat64)],
      [IDL.Vec(UserDetail)],
      ['query']
    ),
    get_user_growth_trend: IDL.Func(
      [],
      [IDL.Vec(IDL.Tuple(IDL.Text, IDL.Nat64))],
      ['query']
    ),
    get_revenue_breakdown: IDL.Func(
      [],
      [IDL.Vec(IDL.Tuple(IDL.Text, IDL.Float64))],
      ['query']
    ),
    get_top_users: IDL.Func(
      [IDL.Nat32, IDL.Text],
      [IDL.Vec(UserDetail)],
      ['query']
    ),
    search_users: IDL.Func([IDL.Text], [IDL.Vec(UserDetail)], ['query']),
    get_total_user_count: IDL.Func([], [IDL.Nat64], ['query']),
  });
};

class UserAnalyticsService {
  private actor: any = null;
  private canisterId: string = '';

  async initialize() {
    // Get canister ID from environment or hardcoded
    this.canisterId = import.meta.env.VITE_CANISTER_ID_DEFLOW_BACKEND ||
                      import.meta.env.VITE_BACKEND_CANISTER_ID ||
                      '7pcz4-fiaaa-aaaad-abtvq-cai'; // Mainnet ID

    const isLocal = window.location.hostname === 'localhost';
    const host = isLocal
      ? 'http://localhost:4943'
      : 'https://ic0.app';

    const agent = new HttpAgent({ host });

    // Fetch root key for local development
    if (isLocal) {
      await agent.fetchRootKey();
    }

    this.actor = Actor.createActor(idl, {
      agent,
      canisterId: this.canisterId,
    });
  }

  async getAnalytics(): Promise<UserAnalytics> {
    if (!this.actor) await this.initialize();
    try {
      const result = await this.actor.get_user_analytics();
      return result;
    } catch (error) {
      console.error('Failed to get analytics:', error);
      throw error;
    }
  }

  async getAllUsers(): Promise<UserDetail[]> {
    if (!this.actor) await this.initialize();
    try {
      const result = await this.actor.get_all_users_details();
      return result;
    } catch (error) {
      console.error('Failed to get all users:', error);
      throw error;
    }
  }

  async getFilteredUsers(
    tier?: 'Standard' | 'Premium' | 'Pro',
    activeOnly: boolean = false,
    minExecutions?: number
  ): Promise<UserDetail[]> {
    if (!this.actor) await this.initialize();
    try {
      const tierVariant = tier ? [{ [tier]: null }] : [];
      const minExec = minExecutions !== undefined ? [BigInt(minExecutions)] : [];

      const result = await this.actor.get_users_filtered(
        tierVariant,
        activeOnly,
        minExec
      );
      return result;
    } catch (error) {
      console.error('Failed to get filtered users:', error);
      throw error;
    }
  }

  async getUserGrowthTrend(): Promise<[string, bigint][]> {
    if (!this.actor) await this.initialize();
    try {
      const result = await this.actor.get_user_growth_trend();
      return result;
    } catch (error) {
      console.error('Failed to get growth trend:', error);
      throw error;
    }
  }

  async getRevenueBreakdown(): Promise<Map<string, number>> {
    if (!this.actor) await this.initialize();
    try {
      const result = await this.actor.get_revenue_breakdown();
      return new Map(result);
    } catch (error) {
      console.error('Failed to get revenue breakdown:', error);
      throw error;
    }
  }

  async getTopUsers(limit: number = 10, sortBy: string = 'executions'): Promise<UserDetail[]> {
    if (!this.actor) await this.initialize();
    try {
      const result = await this.actor.get_top_users(limit, sortBy);
      return result;
    } catch (error) {
      console.error('Failed to get top users:', error);
      throw error;
    }
  }

  async searchUsers(query: string): Promise<UserDetail[]> {
    if (!this.actor) await this.initialize();
    try {
      const result = await this.actor.search_users(query);
      return result;
    } catch (error) {
      console.error('Failed to search users:', error);
      throw error;
    }
  }

  async getTotalUserCount(): Promise<bigint> {
    if (!this.actor) await this.initialize();
    try {
      const result = await this.actor.get_total_user_count();
      return result;
    } catch (error) {
      console.error('Failed to get total user count:', error);
      throw error;
    }
  }
}

// Export singleton instance
export const userAnalyticsService = new UserAnalyticsService();

// Helper function to format tier
export function formatTier(tier: { Standard: null } | { Premium: null } | { Pro: null }): string {
  if ('Standard' in tier) return 'Standard';
  if ('Premium' in tier) return 'Premium';
  if ('Pro' in tier) return 'Pro';
  return 'Unknown';
}

// Helper function to format bigint
export function formatBigInt(value: bigint): string {
  return value.toLocaleString();
}

// Helper function to format date
export function formatTimestamp(timestamp: bigint): string {
  const ms = Number(timestamp / BigInt(1_000_000));
  return new Date(ms).toLocaleDateString();
}

// Helper function to format relative time
export function formatRelativeTime(timestamp: bigint): string {
  const ms = Number(timestamp / BigInt(1_000_000));
  const now = Date.now();
  const diff = now - ms;

  const days = Math.floor(diff / (1000 * 60 * 60 * 24));
  if (days > 0) return `${days}d ago`;

  const hours = Math.floor(diff / (1000 * 60 * 60));
  if (hours > 0) return `${hours}h ago`;

  const minutes = Math.floor(diff / (1000 * 60));
  if (minutes > 0) return `${minutes}m ago`;

  return 'Just now';
}
