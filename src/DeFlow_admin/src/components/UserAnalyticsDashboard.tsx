import React, { useState, useEffect } from 'react';
import {
  userAnalyticsService,
  UserAnalytics,
  UserDetail,
  formatBigInt,
  formatTier,
  formatTimestamp,
  formatRelativeTime,
} from '../services/userAnalyticsService';

const UserAnalyticsDashboard: React.FC = () => {
  const [analytics, setAnalytics] = useState<UserAnalytics | null>(null);
  const [users, setUsers] = useState<UserDetail[]>([]);
  const [filteredUsers, setFilteredUsers] = useState<UserDetail[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Filters
  const [selectedTier, setSelectedTier] = useState<'all' | 'Standard' | 'Premium' | 'Pro'>('all');
  const [activeOnly, setActiveOnly] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [sortBy, setSortBy] = useState<'executions' | 'workflows' | 'revenue' | 'activity'>('executions');

  useEffect(() => {
    loadAnalytics();
    // Auto-refresh every 5 minutes
    const interval = setInterval(loadAnalytics, 5 * 60 * 1000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    applyFilters();
  }, [users, selectedTier, activeOnly, searchQuery, sortBy]);

  const loadAnalytics = async () => {
    try {
      setLoading(true);
      setError(null);

      const [analyticsData, usersData] = await Promise.all([
        userAnalyticsService.getAnalytics(),
        userAnalyticsService.getAllUsers(),
      ]);

      setAnalytics(analyticsData);
      setUsers(usersData);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load analytics');
      console.error('Analytics error:', err);
    } finally {
      setLoading(false);
    }
  };

  const applyFilters = () => {
    let filtered = [...users];

    // Filter by tier
    if (selectedTier !== 'all') {
      filtered = filtered.filter((user) => formatTier(user.subscription_tier) === selectedTier);
    }

    // Filter by active status
    if (activeOnly) {
      filtered = filtered.filter((user) => user.is_active);
    }

    // Filter by search query
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter((user) =>
        user.principal_id.toLowerCase().includes(query)
      );
    }

    // Sort
    filtered.sort((a, b) => {
      switch (sortBy) {
        case 'executions':
          return Number(b.total_executions - a.total_executions);
        case 'workflows':
          return Number(b.total_workflows - a.total_workflows);
        case 'revenue':
          return b.total_paid_usd - a.total_paid_usd;
        case 'activity':
          return Number(b.last_activity - a.last_activity);
        default:
          return 0;
      }
    });

    setFilteredUsers(filtered);
  };

  if (loading && !analytics) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
          <p className="text-gray-400">Loading analytics...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-900/20 border border-red-500 rounded-lg p-4">
        <div className="flex items-center">
          <svg className="h-5 w-5 text-red-500 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <p className="text-red-400">{error}</p>
        </div>
        <button
          onClick={loadAnalytics}
          className="mt-3 text-sm text-blue-400 hover:text-blue-300"
        >
          Try Again
        </button>
      </div>
    );
  }

  if (!analytics) return null;

  const retentionColor = analytics.user_retention_rate > 70 ? 'text-green-400' :
                         analytics.user_retention_rate > 50 ? 'text-yellow-400' : 'text-red-400';

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-bold text-white flex items-center">
            <span className="mr-2">👥</span>
            User Analytics Dashboard
          </h2>
          <p className="text-sm text-gray-400 mt-1">
            Last updated: {formatRelativeTime(analytics.last_updated)}
          </p>
        </div>
        <button
          onClick={loadAnalytics}
          disabled={loading}
          className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 disabled:opacity-50"
        >
          <svg className={`h-4 w-4 mr-2 ${loading ? 'animate-spin' : ''}`} fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          Refresh
        </button>
      </div>

      {/* Overview Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <MetricCard
          icon="👥"
          title="Total Users"
          value={formatBigInt(analytics.total_users)}
          subtitle={`${formatBigInt(analytics.new_users_7d)} new this week`}
          trend="up"
        />
        <MetricCard
          icon="✅"
          title="Active Users (30d)"
          value={formatBigInt(analytics.active_users_30d)}
          subtitle={`${analytics.user_retention_rate.toFixed(1)}% retention`}
          subtitleColor={retentionColor}
        />
        <MetricCard
          icon="🔄"
          title="Total Executions"
          value={formatBigInt(analytics.total_executions)}
          subtitle={`${analytics.average_workflows_per_user.toFixed(1)} workflows/user`}
        />
        <MetricCard
          icon="💰"
          title="Monthly Recurring Revenue"
          value={`$${analytics.monthly_recurring_revenue.toLocaleString()}`}
          subtitle={`$${analytics.total_revenue_usd.toLocaleString()} total`}
          trend="up"
        />
      </div>

      {/* Tier Distribution & Top Nodes */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Tier Distribution */}
        <div className="bg-gray-800 rounded-lg p-6">
          <h3 className="text-lg font-semibold text-white mb-4 flex items-center">
            <span className="mr-2">📊</span>
            User Tier Distribution
          </h3>
          <div className="space-y-3">
            {analytics.users_by_tier.map(([tier, count]) => {
              const percentage = (Number(count) / Number(analytics.total_users)) * 100;
              const tierColor = tier === 'Pro' ? 'bg-purple-500' :
                              tier === 'Premium' ? 'bg-blue-500' : 'bg-gray-500';

              return (
                <div key={tier}>
                  <div className="flex justify-between text-sm mb-1">
                    <span className="text-gray-300">{tier}</span>
                    <span className="text-gray-400">
                      {formatBigInt(count)} ({percentage.toFixed(1)}%)
                    </span>
                  </div>
                  <div className="w-full bg-gray-700 rounded-full h-2">
                    <div
                      className={`${tierColor} h-2 rounded-full transition-all duration-500`}
                      style={{ width: `${percentage}%` }}
                    ></div>
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {/* Top Node Types */}
        <div className="bg-gray-800 rounded-lg p-6">
          <h3 className="text-lg font-semibold text-white mb-4 flex items-center">
            <span className="mr-2">🔥</span>
            Most Used Nodes
          </h3>
          <div className="space-y-2">
            {analytics.top_node_types.slice(0, 5).map(([node, count], index) => (
              <div key={node} className="flex items-center justify-between py-2 px-3 bg-gray-700/50 rounded">
                <div className="flex items-center">
                  <span className="text-gray-400 mr-3 font-mono text-sm">#{index + 1}</span>
                  <span className="text-white">{node}</span>
                </div>
                <span className="text-blue-400 font-semibold">{formatBigInt(count)}</span>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* User List */}
      <div className="bg-gray-800 rounded-lg p-6">
        <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center mb-6 gap-4">
          <h3 className="text-lg font-semibold text-white flex items-center">
            <span className="mr-2">📋</span>
            User List ({filteredUsers.length})
          </h3>

          <div className="flex flex-wrap gap-2 w-full sm:w-auto">
            {/* Search */}
            <input
              type="text"
              placeholder="Search by principal..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 flex-1 sm:flex-initial sm:w-64"
            />

            {/* Tier Filter */}
            <select
              value={selectedTier}
              onChange={(e) => setSelectedTier(e.target.value as any)}
              className="px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="all">All Tiers</option>
              <option value="Standard">Standard</option>
              <option value="Premium">Premium</option>
              <option value="Pro">Pro</option>
            </select>

            {/* Sort By */}
            <select
              value={sortBy}
              onChange={(e) => setSortBy(e.target.value as any)}
              className="px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="executions">Sort by Executions</option>
              <option value="workflows">Sort by Workflows</option>
              <option value="revenue">Sort by Revenue</option>
              <option value="activity">Sort by Activity</option>
            </select>

            {/* Active Only Toggle */}
            <label className="inline-flex items-center px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white cursor-pointer hover:bg-gray-600">
              <input
                type="checkbox"
                checked={activeOnly}
                onChange={(e) => setActiveOnly(e.target.checked)}
                className="mr-2"
              />
              Active Only
            </label>
          </div>
        </div>

        {/* Table */}
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-700">
            <thead>
              <tr>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                  Principal
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                  Tier
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                  Workflows
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                  Executions
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                  Revenue
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                  Last Active
                </th>
                <th className="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">
                  Status
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-700">
              {filteredUsers.length === 0 ? (
                <tr>
                  <td colSpan={7} className="px-4 py-8 text-center text-gray-400">
                    No users found matching your filters
                  </td>
                </tr>
              ) : (
                filteredUsers.map((user) => (
                  <tr key={user.principal_id} className="hover:bg-gray-700/50">
                    <td className="px-4 py-3 text-sm text-white font-mono">
                      {user.principal_id.slice(0, 8)}...{user.principal_id.slice(-6)}
                    </td>
                    <td className="px-4 py-3 text-sm">
                      <TierBadge tier={formatTier(user.subscription_tier)} />
                    </td>
                    <td className="px-4 py-3 text-sm text-gray-300">
                      {formatBigInt(user.total_workflows)}
                    </td>
                    <td className="px-4 py-3 text-sm text-gray-300">
                      {formatBigInt(user.total_executions)}
                    </td>
                    <td className="px-4 py-3 text-sm text-gray-300">
                      ${user.total_paid_usd.toFixed(2)}
                    </td>
                    <td className="px-4 py-3 text-sm text-gray-400">
                      {formatRelativeTime(user.last_activity)}
                    </td>
                    <td className="px-4 py-3 text-sm">
                      {user.is_active ? (
                        <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-900/30 text-green-400">
                          Active
                        </span>
                      ) : (
                        <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-gray-700 text-gray-400">
                          Inactive
                        </span>
                      )}
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};

// Helper Components

interface MetricCardProps {
  icon: string;
  title: string;
  value: string;
  subtitle: string;
  subtitleColor?: string;
  trend?: 'up' | 'down';
}

const MetricCard: React.FC<MetricCardProps> = ({ icon, title, value, subtitle, subtitleColor = 'text-gray-400', trend }) => (
  <div className="bg-gray-800 rounded-lg p-6">
    <div className="flex items-center justify-between mb-2">
      <span className="text-2xl">{icon}</span>
      {trend && (
        <svg
          className={`h-5 w-5 ${trend === 'up' ? 'text-green-400' : 'text-red-400'}`}
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d={trend === 'up' ? 'M13 7h8m0 0v8m0-8l-8 8-4-4-6 6' : 'M13 17h8m0 0V9m0 8l-8-8-4 4-6-6'}
          />
        </svg>
      )}
    </div>
    <h3 className="text-sm font-medium text-gray-400 mb-1">{title}</h3>
    <p className="text-2xl font-bold text-white mb-1">{value}</p>
    <p className={`text-sm ${subtitleColor}`}>{subtitle}</p>
  </div>
);

interface TierBadgeProps {
  tier: string;
}

const TierBadge: React.FC<TierBadgeProps> = ({ tier }) => {
  const colors = {
    Standard: 'bg-gray-700 text-gray-300',
    Premium: 'bg-blue-900/30 text-blue-400',
    Pro: 'bg-purple-900/30 text-purple-400',
  };

  return (
    <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${colors[tier as keyof typeof colors] || colors.Standard}`}>
      {tier}
    </span>
  );
};

export default UserAnalyticsDashboard;
