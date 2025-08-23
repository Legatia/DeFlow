import React, { useState, useEffect } from 'react';

interface PoolState {
  phase: string;
  total_liquidity_usd: number;
  monthly_volume: number;
  fee_collection_rate: number;
  bootstrap_progress: number;
  dev_earnings_pending: number;
  emergency_fund: number;
}

interface ArbitrageOpportunity {
  asset_pair: [string, string];
  buy_chain: string;
  sell_chain: string;
  price_difference: number;
  expected_profit: number;
  required_capital: number;
  confidence_score: number;
}

const PoolManagement: React.FC = () => {
  const [isConnected, setIsConnected] = useState(false);
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<'overview' | 'liquidity' | 'arbitrage' | 'configure'>('overview');

  // Simulate connection attempt
  useEffect(() => {
    const simulateConnection = async () => {
      setLoading(true);
      // Simulate network delay
      await new Promise(resolve => setTimeout(resolve, 1500));
      // For now, always show as disconnected (placeholder mode)
      setIsConnected(false);
      setLoading(false);
    };
    
    simulateConnection();
  }, []);

  // Sample data for demonstration
  const samplePoolState: PoolState = {
    phase: "Active",
    total_liquidity_usd: 2547830.50,
    monthly_volume: 15842650.00,
    fee_collection_rate: 0.003, // 0.3%
    bootstrap_progress: 100,
    dev_earnings_pending: 45230.80,
    emergency_fund: 125000.00
  };

  const sampleArbitrageOpportunities: ArbitrageOpportunity[] = [
    {
      asset_pair: ["BTC", "ETH"],
      buy_chain: "Bitcoin",
      sell_chain: "Ethereum",
      price_difference: 2.3,
      expected_profit: 8750.50,
      required_capital: 250000.00,
      confidence_score: 0.89
    },
    {
      asset_pair: ["ETH", "USDC"],
      buy_chain: "Arbitrum",
      sell_chain: "Ethereum",
      price_difference: 1.8,
      expected_profit: 3245.20,
      required_capital: 150000.00,
      confidence_score: 0.76
    },
    {
      asset_pair: ["SOL", "USDC"],
      buy_chain: "Solana",
      sell_chain: "Polygon", 
      price_difference: 0.9,
      expected_profit: 1850.30,
      required_capital: 85000.00,
      confidence_score: 0.92
    }
  ];

  const liquidityPools = [
    { chain: "Ethereum", protocol: "Uniswap V3", tvl_usd: 847250.00, apy: 12.5, fees_24h: 2840.50 },
    { chain: "Arbitrum", protocol: "SushiSwap", tvl_usd: 425800.00, apy: 18.3, fees_24h: 1950.20 },
    { chain: "Polygon", protocol: "QuickSwap", tvl_usd: 324650.00, apy: 22.1, fees_24h: 1850.80 },
    { chain: "Solana", protocol: "Raydium", tvl_usd: 298430.00, apy: 15.7, fees_24h: 1425.30 },
    { chain: "Base", protocol: "Aerodrome", tvl_usd: 185720.00, apy: 28.4, fees_24h: 980.60 },
    { chain: "Optimism", protocol: "Velodrome", tvl_usd: 165680.00, apy: 19.8, fees_24h: 850.40 }
  ];

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
              <span>{isConnected ? 'Connected' : 'Demo Mode'}</span>
            </div>
            <button className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700">
              <span className="flex items-center">
                <svg className="h-4 w-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                </svg>
                Refresh
              </span>
            </button>
          </div>
        </div>

        {/* Connection Status Banner */}
        {!isConnected && (
          <div className="mt-4 bg-orange-50 border border-orange-200 rounded-lg p-4">
            <div className="flex">
              <svg className="h-5 w-5 text-orange-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <div className="ml-3">
                <p className="text-sm text-orange-800">
                  <strong>Demo Mode:</strong> Unable to connect to pool backend services. 
                  Showing sample data for demonstration purposes.
                </p>
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
                    {formatCurrency(samplePoolState.total_liquidity_usd)}
                  </p>
                </div>
                <div className="bg-green-50 border border-green-200 p-4 rounded-lg">
                  <h3 className="text-sm font-medium text-green-700">Monthly Volume</h3>
                  <p className="text-2xl font-bold text-green-900 mt-1">
                    {formatCurrency(samplePoolState.monthly_volume)}
                  </p>
                </div>
                <div className="bg-purple-50 border border-purple-200 p-4 rounded-lg">
                  <h3 className="text-sm font-medium text-purple-700">Fee Collection</h3>
                  <p className="text-2xl font-bold text-purple-900 mt-1">
                    {formatPercentage(samplePoolState.fee_collection_rate)}
                  </p>
                </div>
                <div className="bg-yellow-50 border border-yellow-200 p-4 rounded-lg">
                  <h3 className="text-sm font-medium text-yellow-700">Dev Earnings</h3>
                  <p className="text-2xl font-bold text-yellow-900 mt-1">
                    {formatCurrency(samplePoolState.dev_earnings_pending)}
                  </p>
                </div>
              </div>

              {/* Pool Status */}
              <div className="bg-gray-50 border border-gray-200 rounded-lg p-6">
                <div className="flex items-center justify-between mb-4">
                  <h3 className="text-lg font-medium text-gray-900">Pool Status</h3>
                  <span className={`px-3 py-1 rounded-full text-sm font-medium border ${getPhaseColor(samplePoolState.phase)}`}>
                    {samplePoolState.phase}
                  </span>
                </div>
                
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                  <div className="bg-white border border-gray-200 rounded-lg p-4">
                    <h4 className="font-medium text-gray-900 mb-2">Bootstrap Progress</h4>
                    <div className="w-full bg-gray-200 rounded-full h-2">
                      <div 
                        className="bg-blue-600 h-2 rounded-full" 
                        style={{ width: `${samplePoolState.bootstrap_progress}%` }}
                      ></div>
                    </div>
                    <p className="text-sm text-gray-600 mt-1">{samplePoolState.bootstrap_progress}% Complete</p>
                  </div>
                  
                  <div className="bg-white border border-gray-200 rounded-lg p-4">
                    <h4 className="font-medium text-gray-900 mb-2">Emergency Fund</h4>
                    <p className="text-xl font-bold text-gray-900">
                      {formatCurrency(samplePoolState.emergency_fund)}
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
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {/* Arbitrage Section */}
          {activeTab === 'arbitrage' && (
            <div className="space-y-6">
              <h3 className="text-lg font-medium text-gray-900">Arbitrage Opportunities</h3>
              
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {sampleArbitrageOpportunities.map((opportunity, index) => (
                  <div key={index} className="bg-white border border-gray-200 rounded-lg p-6 hover:shadow-lg transition-shadow">
                    <div className="flex items-center justify-between mb-4">
                      <h4 className="font-medium text-gray-900">
                        {opportunity.asset_pair[0]} / {opportunity.asset_pair[1]}
                      </h4>
                      <span className={`font-medium ${getConfidenceColor(opportunity.confidence_score)}`}>
                        {(opportunity.confidence_score * 100).toFixed(0)}%
                      </span>
                    </div>
                    
                    <div className="space-y-3">
                      <div className="flex justify-between">
                        <span className="text-sm text-gray-500">Route:</span>
                        <span className="text-sm font-medium text-gray-900">
                          {opportunity.buy_chain} → {opportunity.sell_chain}
                        </span>
                      </div>
                      
                      <div className="flex justify-between">
                        <span className="text-sm text-gray-500">Price Difference:</span>
                        <span className="text-sm font-medium text-green-600">
                          +{opportunity.price_difference.toFixed(1)}%
                        </span>
                      </div>
                      
                      <div className="flex justify-between">
                        <span className="text-sm text-gray-500">Expected Profit:</span>
                        <span className="text-sm font-medium text-green-600">
                          {formatCurrency(opportunity.expected_profit)}
                        </span>
                      </div>
                      
                      <div className="flex justify-between">
                        <span className="text-sm text-gray-500">Required Capital:</span>
                        <span className="text-sm font-medium text-gray-900">
                          {formatCurrency(opportunity.required_capital)}
                        </span>
                      </div>
                    </div>
                    
                    <button className="mt-4 w-full bg-blue-600 text-white py-2 px-4 rounded-lg hover:bg-blue-700 transition-colors">
                      Execute Arbitrage
                    </button>
                  </div>
                ))}
              </div>

              <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
                <div className="flex items-center">
                  <svg className="h-5 w-5 text-blue-600 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  <p className="text-blue-800 text-sm">
                    <strong>Note:</strong> Arbitrage execution requires connection to the pool backend services.
                    These are demo opportunities for interface demonstration.
                  </p>
                </div>
              </div>
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