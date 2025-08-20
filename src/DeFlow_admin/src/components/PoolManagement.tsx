import React, { useState, useEffect } from 'react';
import { AdminPoolService } from '../services/adminPoolService';
import PoolTermination from './PoolTermination';

interface PoolState {
  phase: string;
  total_liquidity_usd: number;
  monthly_volume: number;
  fee_collection_rate: number;
  team_earnings: Record<string, number>;
  bootstrap_progress: number;
}

const PoolManagement: React.FC = () => {
  const [poolState, setPoolState] = useState<PoolState | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'overview' | 'termination'>('overview');

  useEffect(() => {
    loadPoolData();
  }, []);

  const loadPoolData = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await AdminPoolService.getPoolState();
      setPoolState(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load pool data');
    } finally {
      setLoading(false);
    }
  };

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 0,
      maximumFractionDigits: 0
    }).format(amount);
  };

  const showPoolOverview = !loading && !error && poolState;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-gray-800 rounded-lg p-6">
        <div className="flex justify-between items-center">
          <div>
            <h2 className="text-2xl font-bold text-white">Pool Management</h2>
            <p className="text-gray-400 mt-1">Monitor and control DeFlow liquidity pool</p>
          </div>
          <div className="flex items-center space-x-3">
            {loading && (
              <span className="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-blue-100 text-blue-800">
                Loading...
              </span>
            )}
            {error && (
              <span className="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-red-100 text-red-800">
                Error
              </span>
            )}
            {showPoolOverview && (
              <span className={`inline-flex items-center px-3 py-1 rounded-full text-sm font-medium ${
                poolState!.phase === 'Active' ? 'bg-green-100 text-green-800' :
                poolState!.phase === 'Bootstrapping' ? 'bg-yellow-100 text-yellow-800' :
                poolState!.phase === 'Terminating' ? 'bg-red-100 text-red-800' :
                poolState!.phase === 'Terminated' ? 'bg-gray-100 text-gray-800' :
                'bg-red-100 text-red-800'
              }`}>
                {poolState!.phase}
              </span>
            )}
            <button 
              onClick={loadPoolData}
              className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700"
              disabled={loading}
            >
              {loading ? 'Loading...' : 'Refresh'}
            </button>
          </div>
        </div>

        {/* Tab Navigation */}
        <div className="mt-6 border-b border-gray-700">
          <nav className="-mb-px flex space-x-8">
            {[
              { id: 'overview', label: 'Pool Overview', icon: '📊' },
              { id: 'termination', label: 'Termination Control', icon: '🔥' }
            ].map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id as any)}
                className={`py-2 px-1 border-b-2 font-medium text-sm ${
                  activeTab === tab.id
                    ? 'border-blue-500 text-blue-400'
                    : 'border-transparent text-gray-400 hover:text-gray-300 hover:border-gray-300'
                }`}
              >
                <span className="mr-2">{tab.icon}</span>
                {tab.label}
              </button>
            ))}
          </nav>
        </div>
      </div>

      {/* Tab Content */}
      {activeTab === 'overview' && (
        <>
          {/* Loading State */}
          {loading && (
            <div className="text-center py-12">
              <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto"></div>
              <p className="text-gray-400 mt-4">Loading pool data...</p>
            </div>
          )}

          {/* Error State */}
          {error && (
            <div className="bg-red-900/20 border border-red-500 rounded-lg p-6">
              <h3 className="text-red-400 font-medium">Error Loading Pool Data</h3>
              <p className="text-red-300 mt-2">{error}</p>
              <p className="text-red-200 text-sm mt-3">
                Pool overview temporarily unavailable. You can still access termination controls in the "Termination Control" tab.
              </p>
              <button 
                onClick={loadPoolData}
                className="mt-4 bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700"
              >
                Retry Loading Pool Data
              </button>
            </div>
          )}

          {/* Pool Overview - Only show when data is available */}
          {showPoolOverview && (
            <>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <div className="bg-blue-900/50 p-6 rounded-lg border border-blue-700">
          <h3 className="text-sm font-medium text-blue-300">Total Liquidity</h3>
          <p className="text-2xl font-bold text-white mt-2">
            {formatCurrency(poolState!.total_liquidity_usd)}
          </p>
          <p className="text-xs text-blue-200 mt-1">
            Across all chains
          </p>
        </div>

        <div className="bg-green-900/50 p-6 rounded-lg border border-green-700">
          <h3 className="text-sm font-medium text-green-300">Monthly Volume</h3>
          <p className="text-2xl font-bold text-white mt-2">
            {formatCurrency(poolState!.monthly_volume)}
          </p>
          <p className="text-xs text-green-200 mt-1">
            Transaction volume
          </p>
        </div>

        <div className="bg-purple-900/50 p-6 rounded-lg border border-purple-700">
          <h3 className="text-sm font-medium text-purple-300">Fee Collection</h3>
          <p className="text-2xl font-bold text-white mt-2">
            {(poolState!.fee_collection_rate * 100).toFixed(1)}%
          </p>
          <p className="text-xs text-purple-200 mt-1">
            Pool accumulation rate
          </p>
        </div>

        <div className="bg-orange-900/50 p-6 rounded-lg border border-orange-700">
          <h3 className="text-sm font-medium text-orange-300">Bootstrap Progress</h3>
          <p className="text-2xl font-bold text-white mt-2">
            {(poolState!.bootstrap_progress * 100).toFixed(0)}%
          </p>
          <p className="text-xs text-orange-200 mt-1">
            Target liquidity reached
          </p>
        </div>
      </div>

      {/* Team Earnings */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h3 className="text-lg font-medium text-white mb-4">Team Earnings (30% Split)</h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="bg-gray-700 p-4 rounded-lg">
            <h4 className="text-sm font-medium text-gray-300">Total Distributed</h4>
            <p className="text-xl font-bold text-white mt-1">
              {formatCurrency(poolState!.team_earnings.total_distributed)}
            </p>
            <p className="text-xs text-gray-400 mt-1">All-time payouts</p>
          </div>

          <div className="bg-gray-700 p-4 rounded-lg">
            <h4 className="text-sm font-medium text-gray-300">Pending Distribution</h4>
            <p className="text-xl font-bold text-white mt-1">
              {formatCurrency(poolState!.team_earnings.pending_distribution)}
            </p>
            <p className="text-xs text-gray-400 mt-1">Ready for withdrawal</p>
          </div>

          <div className="bg-gray-700 p-4 rounded-lg">
            <h4 className="text-sm font-medium text-gray-300">Monthly Average</h4>
            <p className="text-xl font-bold text-white mt-1">
              {formatCurrency(poolState!.team_earnings.monthly_average)}
            </p>
            <p className="text-xs text-gray-400 mt-1">Rolling 3-month avg</p>
          </div>
        </div>
      </div>

      {/* 7:3 Split Visualization */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h3 className="text-lg font-medium text-white mb-4">Fee Distribution Model</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div className="bg-blue-900/30 p-4 rounded-lg border border-blue-700">
            <div className="flex items-center justify-between mb-2">
              <h4 className="text-sm font-medium text-blue-300">Pool Reserves (70%)</h4>
              <span className="text-blue-400 font-bold">70%</span>
            </div>
            <div className="bg-blue-500 h-3 rounded-full mb-2"></div>
            <p className="text-xs text-blue-200">
              Grows liquidity for better arbitrage opportunities and lower slippage
            </p>
          </div>

          <div className="bg-green-900/30 p-4 rounded-lg border border-green-700">
            <div className="flex items-center justify-between mb-2">
              <h4 className="text-sm font-medium text-green-300">Treasury (30%)</h4>
              <span className="text-green-400 font-bold">30%</span>
            </div>
            <div className="bg-green-500 h-3 rounded-full mb-2"></div>
            <p className="text-xs text-green-200">
              Team earnings, development costs, and platform sustainability
            </p>
          </div>
        </div>
      </div>

      {/* Pool Controls */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h3 className="text-lg font-medium text-white mb-4">Pool Controls</h3>
        <div className="bg-yellow-900/20 border border-yellow-500 rounded-lg p-4">
          <p className="text-yellow-300">
            🚧 Pool management controls coming soon. Will include:
          </p>
          <ul className="text-yellow-200 mt-2 ml-4 space-y-1">
            <li>• Emergency pool pause/resume</li>
            <li>• Fee rate adjustments</li>
            <li>• Bootstrap target modifications</li>
            <li>• Cross-chain balance rebalancing</li>
            <li>• Liquidity provider management</li>
          </ul>
        </div>
      </div>
            </>
          )}
        </>
      )}

      {activeTab === 'termination' && <PoolTermination />}
    </div>
  );
};

export default PoolManagement;