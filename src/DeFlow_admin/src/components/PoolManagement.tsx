import React, { useState, useEffect } from 'react';
import { AdminPoolService } from '../services/adminPoolService';

interface PoolState {
  phase: string;
  total_liquidity_usd: number;
  monthly_volume: number;
  fee_collection_rate: number;
  team_earnings: Record<string, number>;
  bootstrap_progress: number;
}


const PoolManagement: React.FC = () => {
  const [isConnected, setIsConnected] = useState(false);
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<'overview' | 'liquidity' | 'arbitrage' | 'configure'>('overview');

  const [poolState, setPoolState] = useState<PoolState | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadPoolData();
  }, []);

  const loadPoolData = async () => {
    try {
      setLoading(true);
      setError(null);
      
      const state = await AdminPoolService.getPoolState();
      setPoolState(state);
      setIsConnected(true);
    } catch (err) {
      console.error('Failed to load pool data:', err);
      setError(err instanceof Error ? err.message : 'Failed to connect to pool');
      setIsConnected(false);
    } finally {
      setLoading(false);
    }
  };

  const refreshData = async () => {
    await loadPoolData();
  };

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 0,
      maximumFractionDigits: 0
    }).format(amount);
  };

  const formatPercentage = (value: number) => {
    return `${(value * 100).toFixed(2)}%`;
  };

  const getPhaseColor = (phase: string) => {
    switch (phase) {
      case 'Active': return 'bg-green-100 text-green-800 border-green-200';
      case 'Bootstrapping': return 'bg-blue-100 text-blue-800 border-blue-200';
      case 'Emergency': return 'bg-red-100 text-red-800 border-red-200';
      default: return 'bg-gray-100 text-gray-800 border-gray-200';
    }
  };

  const getConfidenceColor = (score: number) => {
    if (score >= 0.8) return 'text-green-600';
    if (score >= 0.6) return 'text-yellow-600';
    return 'text-red-600';
  };

  if (loading) {
    return (
      <div className="text-center py-12">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto"></div>
        <p className="text-gray-400 mt-4">Connecting to pool services...</p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-white rounded-lg shadow-lg p-6">
        <div className="flex justify-between items-center">
          <div>
            <h2 className="text-2xl font-bold text-gray-900">Pool Management</h2>
            <p className="text-gray-600 mt-1">Monitor and manage DeFlow liquidity pools and arbitrage</p>
          </div>
          <div className="flex items-center space-x-3">
            <div className={`flex items-center space-x-2 px-3 py-1 rounded-full text-sm ${
              isConnected ? 'bg-green-100 text-green-800' : 'bg-orange-100 text-orange-800'
            }`}>
              <div className={`w-2 h-2 rounded-full ${
                isConnected ? 'bg-green-500' : 'bg-orange-500'
              }`}></div>
              <span>{isConnected ? 'Connected' : error ? 'Error' : 'Connecting'}</span>
            </div>
            <button 
              onClick={refreshData}
              className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 disabled:opacity-50"
              disabled={loading}
            >
              <span className="flex items-center">
                <svg className={`h-4 w-4 mr-2 ${loading ? 'animate-spin' : ''}`} fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                </svg>
                Refresh
              </span>
            </button>
          </div>
        </div>

        {/* Connection Status Banner */}
        {error && (
          <div className="mt-4 bg-red-50 border border-red-200 rounded-lg p-4">
            <div className="flex">
              <svg className="h-5 w-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <div className="ml-3">
                <p className="text-sm text-red-800">
                  <strong>Connection Error:</strong> {error}
                </p>
                <button 
                  onClick={refreshData}
                  className="mt-2 text-sm text-red-600 hover:text-red-800 underline"
                >
                  Retry Connection
                </button>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Navigation */}
      <div className="bg-white rounded-lg shadow-lg">
        <div className="border-b border-gray-200">
          <nav className="-mb-px flex">
            {[
              { id: 'overview', label: 'Pool Overview', icon: '🏊‍♂️' },
              { id: 'liquidity', label: 'Liquidity Pools', icon: '💧' },
              { id: 'arbitrage', label: 'Arbitrage', icon: '⚖️' },
              { id: 'configure', label: 'Configure', icon: '⚙️' }
            ].map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id as any)}
                className={`py-3 px-6 text-sm font-medium border-b-2 ${
                  activeTab === tab.id
                    ? 'border-blue-500 text-blue-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                }`}
              >
                <span className="mr-2">{tab.icon}</span>
                {tab.label}
              </button>
            ))}
          </nav>
        </div>

        <div className="p-6">
          {/* Pool Overview Section */}
          {activeTab === 'overview' && (
            <div className="space-y-6">
              {/* Key Metrics */}
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <div className="bg-blue-50 border border-blue-200 p-4 rounded-lg">
                  <h3 className="text-sm font-medium text-blue-700">Total Liquidity</h3>
                  <p className="text-2xl font-bold text-blue-900 mt-1">
                    {formatCurrency(poolState?.total_liquidity_usd || 0)}
                  </p>
                </div>
                <div className="bg-green-50 border border-green-200 p-4 rounded-lg">
                  <h3 className="text-sm font-medium text-green-700">Monthly Volume</h3>
                  <p className="text-2xl font-bold text-green-900 mt-1">
                    {formatCurrency(poolState?.monthly_volume || 0)}
                  </p>
                </div>
                <div className="bg-purple-50 border border-purple-200 p-4 rounded-lg">
                  <h3 className="text-sm font-medium text-purple-700">Fee Collection</h3>
                  <p className="text-2xl font-bold text-purple-900 mt-1">
                    {formatPercentage(poolState?.fee_collection_rate || 0)}
                  </p>
                </div>
                <div className="bg-yellow-50 border border-yellow-200 p-4 rounded-lg">
                  <h3 className="text-sm font-medium text-yellow-700">Dev Earnings</h3>
                  <p className="text-2xl font-bold text-yellow-900 mt-1">
                    {formatCurrency(Object.values(poolState?.team_earnings || {}).reduce((a, b) => a + b, 0))}
                  </p>
                </div>
              </div>

              {/* Pool Status */}
              <div className="bg-gray-50 border border-gray-200 rounded-lg p-6">
                <div className="flex items-center justify-between mb-4">
                  <h3 className="text-lg font-medium text-gray-900">Pool Status</h3>
                  <span className={`px-3 py-1 rounded-full text-sm font-medium border ${getPhaseColor(poolState?.phase || 'Unknown')}`}>
                    {poolState?.phase || 'Unknown'}
                  </span>
                </div>
                
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                  <div className="bg-white border border-gray-200 rounded-lg p-4">
                    <h4 className="font-medium text-gray-900 mb-2">Bootstrap Progress</h4>
                    <div className="w-full bg-gray-200 rounded-full h-2">
                      <div 
                        className="bg-blue-600 h-2 rounded-full" 
                        style={{ width: `${(poolState?.bootstrap_progress || 0) * 100}%` }}
                      ></div>
                    </div>
                    <p className="text-sm text-gray-600 mt-1">{((poolState?.bootstrap_progress || 0) * 100).toFixed(1)}% Complete</p>
                  </div>
                  
                  <div className="bg-white border border-gray-200 rounded-lg p-4">
                    <h4 className="font-medium text-gray-900 mb-2">Emergency Fund</h4>
                    <p className="text-xl font-bold text-gray-900">
                      {formatCurrency(poolState?.team_earnings?.emergency_fund || 0)}
                    </p>
                    <p className="text-sm text-green-600 mt-1">Fully Funded</p>
                  </div>
                  
                  <div className="bg-white border border-gray-200 rounded-lg p-4">
                    <h4 className="font-medium text-gray-900 mb-2">Health Score</h4>
                    <p className="text-xl font-bold text-green-600">98.5%</p>
                    <p className="text-sm text-gray-600 mt-1">Excellent</p>
                  </div>
                </div>
              </div>

              {/* Recent Activity */}
              <div className="bg-gray-50 border border-gray-200 rounded-lg p-6">
                <h3 className="text-lg font-medium text-gray-900 mb-4">Recent Pool Activity</h3>
                <div className="space-y-3">
                  <div className="flex items-center justify-between bg-white border border-gray-200 rounded-lg p-3">
                    <div className="flex items-center">
                      <div className="bg-green-100 rounded-full p-2 mr-3">
                        <svg className="h-4 w-4 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
                        </svg>
                      </div>
                      <div>
                        <p className="font-medium text-gray-900">Liquidity Added</p>
                        <p className="text-sm text-gray-500">Ethereum Uniswap V3 • 2 hours ago</p>
                      </div>
                    </div>
                    <span className="text-green-600 font-medium">+$25,400</span>
                  </div>
                  
                  <div className="flex items-center justify-between bg-white border border-gray-200 rounded-lg p-3">
                    <div className="flex items-center">
                      <div className="bg-blue-100 rounded-full p-2 mr-3">
                        <svg className="h-4 w-4 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
                        </svg>
                      </div>
                      <div>
                        <p className="font-medium text-gray-900">Arbitrage Executed</p>
                        <p className="text-sm text-gray-500">BTC/ETH Arbitrum → Ethereum • 4 hours ago</p>
                      </div>
                    </div>
                    <span className="text-blue-600 font-medium">+$8,750</span>
                  </div>
                  
                  <div className="flex items-center justify-between bg-white border border-gray-200 rounded-lg p-3">
                    <div className="flex items-center">
                      <div className="bg-purple-100 rounded-full p-2 mr-3">
                        <svg className="h-4 w-4 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 9V7a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2m2 4h10a2 2 0 002-2v-6a2 2 0 00-2-2H9a2 2 0 00-2 2v2a2 2 0 002 2z" />
                        </svg>
                      </div>
                      <div>
                        <p className="font-medium text-gray-900">Fees Collected</p>
                        <p className="text-sm text-gray-500">Daily fee collection • 8 hours ago</p>
                      </div>
                    </div>
                    <span className="text-purple-600 font-medium">+$3,240</span>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Liquidity Pools Section */}
          {activeTab === 'liquidity' && (
            <div className="space-y-4">
              <h3 className="text-lg font-medium text-gray-900">Active Liquidity Pools</h3>
              <div className="bg-white border border-gray-200 rounded-lg overflow-hidden">
                <table className="min-w-full divide-y divide-gray-200">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Chain</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Protocol</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">TVL</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">APY</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">24h Fees</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Status</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Actions</th>
                    </tr>
                  </thead>
                  <tbody className="bg-white divide-y divide-gray-200">
                    {isConnected ? (
                      <tr>
                        <td colSpan={7} className="px-6 py-4 text-center text-gray-500">
                          Liquidity pools data will be loaded from the pool canister
                        </td>
                      </tr>
                    ) : (
                      <tr>
                        <td colSpan={7} className="px-6 py-4 text-center text-gray-500">
                          {error || 'Connecting to pool...'}
                        </td>
                      </tr>
                    )}
                    {/* Placeholder for when we implement pool management
                    {liquidityPools.map((pool, index) => (
                      <tr key={index} className="hover:bg-gray-50">
                        <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                          {pool.chain}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {pool.protocol}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {formatCurrency(pool.tvl_usd)}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-green-600">
                          {pool.apy.toFixed(1)}%
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {formatCurrency(pool.fees_24h)}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap">
                          <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
                            Active
                          </span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-blue-600">
                          <button className="hover:text-blue-800">Manage</button>
                        </td>
                      </tr>
                    ))
                    */}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {/* Arbitrage Section */}
          {activeTab === 'arbitrage' && (
            <div className="space-y-6">
              <h3 className="text-lg font-medium text-gray-900">Arbitrage Opportunities</h3>
              
              {isConnected ? (
                <div className="bg-blue-50 border border-blue-200 rounded-lg p-6">
                  <div className="flex items-center mb-4">
                    <svg className="h-5 w-5 text-blue-600 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                    <h4 className="text-lg font-medium text-blue-800">Arbitrage Opportunities</h4>
                  </div>
                  <p className="text-blue-700 mb-4">
                    Arbitrage opportunities will be displayed here when connected to the pool backend.
                    The system will automatically scan for profitable cross-chain arbitrage opportunities.
                  </p>
                  <div className="bg-white border border-blue-200 rounded-lg p-4">
                    <h5 className="font-medium text-blue-800 mb-2">Features:</h5>
                    <ul className="text-blue-700 text-sm space-y-1">
                      <li>• Real-time cross-chain price monitoring</li>
                      <li>• Automated arbitrage execution</li>
                      <li>• Risk assessment and confidence scoring</li>
                      <li>• Gas fee optimization</li>
                    </ul>
                  </div>
                </div>
              ) : (
                <div className="bg-red-50 border border-red-200 rounded-lg p-6">
                  <p className="text-red-700">
                    {error || 'Unable to connect to arbitrage services'}
                  </p>
                </div>
              )}

            </div>
          )}

          {/* Configure Section */}
          {activeTab === 'configure' && (
            <div className="space-y-6">
              <h3 className="text-lg font-medium text-gray-900">Pool Configuration</h3>
              <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-6">
                <div className="flex items-center mb-4">
                  <svg className="h-5 w-5 text-yellow-600 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                  </svg>
                  <h4 className="text-lg font-medium text-yellow-800">Pool Settings</h4>
                </div>
                <p className="text-yellow-700 mb-4">
                  Pool configuration features are coming soon and will include:
                </p>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div className="bg-white border border-yellow-200 rounded-lg p-4">
                    <h5 className="font-medium text-yellow-800 mb-2">Liquidity Management</h5>
                    <ul className="text-yellow-700 text-sm space-y-1">
                      <li>• Pool creation and termination</li>
                      <li>• Liquidity provision limits</li>
                      <li>• Fee structure configuration</li>
                      <li>• Emergency pause controls</li>
                    </ul>
                  </div>
                  <div className="bg-white border border-yellow-200 rounded-lg p-4">
                    <h5 className="font-medium text-yellow-800 mb-2">Arbitrage Settings</h5>
                    <ul className="text-yellow-700 text-sm space-y-1">
                      <li>• Minimum profit thresholds</li>
                      <li>• Risk management parameters</li>
                      <li>• Chain-specific configurations</li>
                      <li>• Automated execution settings</li>
                    </ul>
                  </div>
                </div>
                
                <div className="mt-4 bg-red-50 border border-red-200 rounded-lg p-4">
                  <div className="flex items-center mb-2">
                    <svg className="h-4 w-4 text-red-600 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
                    </svg>
                    <h5 className="font-medium text-red-800">Emergency Controls</h5>
                  </div>
                  <p className="text-red-700 text-sm mb-3">
                    Emergency pool termination and fund recovery tools (owner-only access)
                  </p>
                  <button className="bg-red-600 text-white px-4 py-2 rounded-lg hover:bg-red-700 transition-colors text-sm">
                    Emergency Pool Termination
                  </button>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default PoolManagement;