import React, { useState, useEffect } from 'react';
import { AdminPoolService } from '../services/adminPoolService';

interface CanisterHealth {
  canister_id: string;
  name: string;
  status: string;
  memory_usage: number;
  cycles_balance: number;
  last_upgrade?: bigint;
  error_rate?: number;
  avg_response_time?: number;
  heap_memory_size?: number;
  stable_memory_size?: number;
  health_score?: number;
  is_healthy?: boolean;
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

const SystemHealth: React.FC = () => {
  const [healthData, setHealthData] = useState<SystemHealthData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadSystemHealth();
    
    // Set up auto-refresh every 30 seconds
    const interval = setInterval(loadSystemHealth, 30000);
    return () => clearInterval(interval);
  }, []);

  const loadSystemHealth = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await AdminPoolService.getSystemHealth();
      setHealthData(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load system health');
    } finally {
      setLoading(false);
    }
  };

  const formatCycles = (cycles: number) => {
    if (cycles === 0) return 'N/A';
    if (cycles >= 1_000_000_000_000) {
      return `${(cycles / 1_000_000_000_000).toFixed(1)}T`;
    } else if (cycles >= 1_000_000_000) {
      return `${(cycles / 1_000_000_000).toFixed(1)}B`;
    } else if (cycles >= 1_000_000) {
      return `${(cycles / 1_000_000).toFixed(1)}M`;
    }
    return cycles.toString();
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return 'N/A';
    if (bytes >= 1_000_000) {
      return `${(bytes / 1_000_000).toFixed(1)}MB`;
    } else if (bytes >= 1_000) {
      return `${(bytes / 1_000).toFixed(1)}KB`;
    }
    return `${bytes}B`;
  };

  const formatTimestamp = (timestamp: bigint) => {
    const date = new Date(Number(timestamp) / 1000000);
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
  };

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 0,
      maximumFractionDigits: 0
    }).format(amount);
  };

  const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'healthy': return 'text-green-400';
      case 'running': return 'text-green-400';
      case 'warning': return 'text-yellow-400';
      case 'critical': return 'text-red-400';
      case 'stopped': return 'text-red-400';
      default: return 'text-gray-400';
    }
  };

  const getStatusBgColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'healthy': return 'bg-green-100 text-green-800';
      case 'running': return 'bg-green-100 text-green-800';
      case 'warning': return 'bg-yellow-100 text-yellow-800';
      case 'critical': return 'bg-red-100 text-red-800';
      case 'stopped': return 'bg-red-100 text-red-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  const getCanisterIcon = (name: string) => {
    switch (name) {
      case 'DeFlow Pool': return '🏊';
      case 'DeFlow Backend': return '⚙️';
      case 'DeFlow Frontend': return '🌐';
      case 'DeFlow Admin': return '👑';
      default: return '📦';
    }
  };

  if (loading && !healthData) {
    return (
      <div className="text-center py-12">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto"></div>
        <p className="text-gray-400 mt-4">Loading system health...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-900/20 border border-red-500 rounded-lg p-6">
        <h3 className="text-red-400 font-medium">Error Loading System Health</h3>
        <p className="text-red-300 mt-2">{error}</p>
        <button 
          onClick={loadSystemHealth}
          className="mt-4 bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700"
        >
          Retry
        </button>
      </div>
    );
  }

  if (!healthData) return null;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-gray-800 rounded-lg p-6">
        <div className="flex justify-between items-center">
          <div>
            <h2 className="text-2xl font-bold text-white">System Health</h2>
            <p className="text-gray-400 mt-1">Complete DeFlow platform monitoring</p>
          </div>
          <div className="flex items-center space-x-3">
            <span className={`inline-flex items-center px-3 py-1 rounded-full text-sm font-medium ${getStatusBgColor(healthData.overall_status)}`}>
              {healthData.overall_status}
            </span>
            <button 
              onClick={loadSystemHealth}
              disabled={loading}
              className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 disabled:opacity-50"
            >
              {loading ? 'Refreshing...' : 'Refresh'}
            </button>
          </div>
        </div>
      </div>

      {/* Platform Overview */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <div className="bg-blue-900/50 p-6 rounded-lg border border-blue-700">
          <h3 className="text-sm font-medium text-blue-300">Platform Status</h3>
          <p className={`text-2xl font-bold mt-2 ${getStatusColor(healthData.overall_status)}`}>
            {healthData.overall_status}
          </p>
          <p className="text-xs text-blue-200 mt-1">
            {healthData.canisters.length} canisters monitored
          </p>
        </div>

        <div className="bg-green-900/50 p-6 rounded-lg border border-green-700">
          <h3 className="text-sm font-medium text-green-300">Total Cycles</h3>
          <p className="text-2xl font-bold text-white mt-2">
            {formatCycles(healthData.total_cycles)}
          </p>
          <p className="text-xs text-green-200 mt-1">
            Across all canisters
          </p>
        </div>

        <div className="bg-purple-900/50 p-6 rounded-lg border border-purple-700">
          <h3 className="text-sm font-medium text-purple-300">Active Users</h3>
          <p className="text-2xl font-bold text-white mt-2">
            {healthData.platform_metrics.active_users_24h}
          </p>
          <p className="text-xs text-purple-200 mt-1">
            Last 24 hours
          </p>
        </div>

        <div className="bg-orange-900/50 p-6 rounded-lg border border-orange-700">
          <h3 className="text-sm font-medium text-orange-300">Daily Volume</h3>
          <p className="text-2xl font-bold text-white mt-2">
            {formatCurrency(healthData.platform_metrics.total_volume_24h_usd)}
          </p>
          <p className="text-xs text-orange-200 mt-1">
            Transaction volume
          </p>
        </div>
      </div>

      {/* Canister Health Grid */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h3 className="text-lg font-medium text-white mb-6">Canister Health Status</h3>
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {healthData.canisters.map((canister, index) => (
            <div key={index} className="bg-gray-700 rounded-lg p-4 border border-gray-600">
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center">
                  <span className="text-2xl mr-3">{getCanisterIcon(canister.name)}</span>
                  <div>
                    <h4 className="text-white font-medium">{canister.name}</h4>
                    <p className="text-xs text-gray-400 font-mono">{canister.canister_id}</p>
                  </div>
                </div>
                <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${getStatusBgColor(canister.status)}`}>
                  {canister.status}
                </span>
              </div>

              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <span className="text-gray-400">Memory Usage:</span>
                  <div className="flex items-center mt-1">
                    <span className="text-white font-medium">{canister.memory_usage === 0 ? 'N/A' : canister.memory_usage.toFixed(1) + '%'}</span>
                    {canister.memory_usage > 0 && (
                      <div className="ml-2 flex-1 bg-gray-600 rounded-full h-2">
                        <div 
                          className={`h-2 rounded-full ${
                            canister.memory_usage > 80 ? 'bg-red-500' :
                            canister.memory_usage > 60 ? 'bg-yellow-500' :
                            'bg-green-500'
                          }`}
                          style={{ width: `${canister.memory_usage}%` }}
                        ></div>
                      </div>
                    )}
                  </div>
                </div>

                <div>
                  <span className="text-gray-400">Cycles:</span>
                  <p className="text-white font-medium">{formatCycles(canister.cycles_balance)}</p>
                </div>

                <div>
                  <span className="text-gray-400">Response Time:</span>
                  <p className="text-white font-medium">{canister.avg_response_time ? canister.avg_response_time + 'ms' : 'N/A'}</p>
                </div>

                <div>
                  <span className="text-gray-400">Error Rate:</span>
                  <p className="text-white font-medium">{canister.error_rate ? (canister.error_rate * 100).toFixed(3) + '%' : 'N/A'}</p>
                </div>

                <div>
                  <span className="text-gray-400">Heap Memory:</span>
                  <p className="text-white font-medium">{canister.heap_memory_size ? formatBytes(canister.heap_memory_size) : 'N/A'}</p>
                </div>

                <div>
                  <span className="text-gray-400">Stable Memory:</span>
                  <p className="text-white font-medium">
                    {canister.stable_memory_size && canister.stable_memory_size > 0 ? formatBytes(canister.stable_memory_size) : 'N/A'}
                  </p>
                </div>
              </div>

              <div className="mt-3 pt-3 border-t border-gray-600">
                <span className="text-gray-400 text-xs">Last Upgrade:</span>
                <p className="text-white text-xs">{canister.last_upgrade ? formatTimestamp(canister.last_upgrade) : 'N/A'}</p>
              </div>

              {canister.warnings.length > 0 && (
                <div className="mt-3">
                  {canister.warnings.map((warning, idx) => (
                    <div key={idx} className="bg-yellow-900/30 border border-yellow-500 rounded p-2 text-yellow-300 text-xs">
                      ⚠️ {warning}
                    </div>
                  ))}
                </div>
              )}
            </div>
          ))}
        </div>
      </div>

      {/* Platform Metrics */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h3 className="text-lg font-medium text-white mb-4">Platform Metrics (24h)</h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="bg-gray-700 p-4 rounded-lg">
            <h4 className="text-sm font-medium text-gray-300">User Activity</h4>
            <div className="mt-2 space-y-2">
              <div className="flex justify-between">
                <span className="text-gray-400 text-sm">Total Users:</span>
                <span className="text-white">{healthData.platform_metrics.total_users.toLocaleString()}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-400 text-sm">Active (24h):</span>
                <span className="text-white">{healthData.platform_metrics.active_users_24h.toLocaleString()}</span>
              </div>
            </div>
          </div>

          <div className="bg-gray-700 p-4 rounded-lg">
            <h4 className="text-sm font-medium text-gray-300">Workflow Activity</h4>
            <div className="mt-2 space-y-2">
              <div className="flex justify-between">
                <span className="text-gray-400 text-sm">Total Workflows:</span>
                <span className="text-white">{healthData.platform_metrics.total_workflows.toLocaleString()}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-400 text-sm">Executed (24h):</span>
                <span className="text-white">{healthData.platform_metrics.workflows_executed_24h.toLocaleString()}</span>
              </div>
            </div>
          </div>

          <div className="bg-gray-700 p-4 rounded-lg">
            <h4 className="text-sm font-medium text-gray-300">Transaction Activity</h4>
            <div className="mt-2 space-y-2">
              <div className="flex justify-between">
                <span className="text-gray-400 text-sm">Transactions (24h):</span>
                <span className="text-white">{healthData.platform_metrics.total_transactions_24h.toLocaleString()}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-400 text-sm">Volume (24h):</span>
                <span className="text-white">{formatCurrency(healthData.platform_metrics.total_volume_24h_usd)}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Network Information */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h3 className="text-lg font-medium text-white mb-4">Network Information</h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="space-y-3">
            <div className="flex justify-between items-center">
              <span className="text-gray-400">IC Network:</span>
              <span className="text-white">{healthData.network_info.ic_network}</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-gray-400">Subnet ID:</span>
              <span className="text-white font-mono text-sm">{healthData.network_info.subnet_id}</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-gray-400">Replica Version:</span>
              <span className="text-white">{healthData.network_info.replica_version}</span>
            </div>
          </div>
          
          <div className="space-y-3">
            <div className="flex justify-between items-center">
              <span className="text-gray-400">Auto-Refresh:</span>
              <span className="text-green-400">30s intervals</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-gray-400">Monitoring Status:</span>
              <span className="text-green-400">Active</span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-gray-400">Last Update:</span>
              <span className="text-white">Just now</span>
            </div>
          </div>

          <div className="space-y-3">
            <div className="bg-blue-900/30 border border-blue-500 rounded-lg p-3">
              <span className="text-blue-400">ℹ️ All DeFlow canisters are monitored in real-time</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default SystemHealth;