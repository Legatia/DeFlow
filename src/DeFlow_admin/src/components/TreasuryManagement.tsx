import React, { useState, useEffect } from 'react';
import { AdminPoolService } from '../services/adminPoolService';

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

const TreasuryManagement: React.FC = () => {
  const [isConnected, setIsConnected] = useState(false);
  const [loading, setLoading] = useState(true);
  const [activeSection, setActiveSection] = useState<'overview' | 'balances' | 'transactions' | 'earnings' | 'configure'>('overview');

  const [healthData, setHealthData] = useState<any>(null);
  const [balances, setBalances] = useState<TreasuryBalance[]>([]);
  const [transactions, setTransactions] = useState<TreasuryTransaction[]>([]);
  const [teamEarnings, setTeamEarnings] = useState<Record<string, any>>({});
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadTreasuryData();
  }, []);

  const loadTreasuryData = async () => {
    try {
      setLoading(true);
      setError(null);
      
      // Load real data from the pool canister
      const [healthReport, treasuryBalances, treasuryTransactions, allTeamEarnings] = await Promise.all([
        AdminPoolService.getTreasuryHealthReport(),
        AdminPoolService.getAllTreasuryBalances(), 
        AdminPoolService.getTreasuryTransactions(50),
        AdminPoolService.getAllTeamEarnings()
      ]);
      
      setHealthData(healthReport);
      setBalances(treasuryBalances);
      setTransactions(treasuryTransactions);
      setTeamEarnings(allTeamEarnings);
      setIsConnected(true);
    } catch (err) {
      console.error('Failed to load treasury data:', err);
      setError(err instanceof Error ? err.message : 'Failed to connect to treasury');
      setIsConnected(false);
    } finally {
      setLoading(false);
    }
  };

  const refreshData = async () => {
    await loadTreasuryData();
  };

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 2,
      maximumFractionDigits: 2
    }).format(amount);
  };

  const formatTimestamp = (timestamp: number) => {
    const date = new Date(timestamp);
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
  };

  const getTransactionTypeColor = (type: string) => {
    switch (type) {
      case 'TransactionFeeRevenue': return 'bg-green-100 text-green-800';
      case 'SubscriptionPayment': return 'bg-blue-100 text-blue-800';
      case 'WithdrawalToTeam': return 'bg-purple-100 text-purple-800';
      case 'TransferToCold': return 'bg-gray-100 text-gray-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Confirmed': return 'text-green-600';
      case 'Pending': return 'text-yellow-600';
      case 'Failed': return 'text-red-600';
      default: return 'text-gray-600';
    }
  };

  if (loading) {
    return (
      <div className="text-center py-12">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto"></div>
        <p className="text-gray-400 mt-4">Connecting to treasury services...</p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-white rounded-lg shadow-lg p-6">
        <div className="flex justify-between items-center">
          <div>
            <h2 className="text-2xl font-bold text-gray-900">Treasury Management</h2>
            <p className="text-gray-600 mt-1">Monitor and manage DeFlow treasury assets</p>
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
              { id: 'overview', label: 'Overview', icon: '📊' },
              { id: 'balances', label: 'Balances', icon: '💰' },
              { id: 'transactions', label: 'Transactions', icon: '📋' },
              { id: 'earnings', label: 'Team Earnings', icon: '💎' },
              { id: 'configure', label: 'Configure', icon: '⚙️' }
            ].map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveSection(tab.id as any)}
                className={`py-3 px-6 text-sm font-medium border-b-2 ${
                  activeSection === tab.id
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
          {/* Overview Section */}
          {activeSection === 'overview' && (
            <div className="space-y-6">
              {/* Key Metrics */}
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <div className="bg-blue-50 border border-blue-200 p-4 rounded-lg">
                  <h3 className="text-sm font-medium text-blue-700">Total Treasury Value</h3>
                  <p className="text-2xl font-bold text-blue-900 mt-1">
                    {formatCurrency(healthData?.total_usd_value || 0)}
                  </p>
                </div>
                <div className="bg-green-50 border border-green-200 p-4 rounded-lg">
                  <h3 className="text-sm font-medium text-green-700">Total Assets</h3>
                  <p className="text-2xl font-bold text-green-900 mt-1">
                    {healthData?.total_assets || 0}
                  </p>
                </div>
                <div className="bg-yellow-50 border border-yellow-200 p-4 rounded-lg">
                  <h3 className="text-sm font-medium text-yellow-700">Hot Wallet Usage</h3>
                  <p className="text-2xl font-bold text-yellow-900 mt-1">
                    {(healthData?.hot_wallet_utilization || 0).toFixed(1)}%
                  </p>
                </div>
                <div className="bg-purple-50 border border-purple-200 p-4 rounded-lg">
                  <h3 className="text-sm font-medium text-purple-700">Diversification</h3>
                  <p className="text-2xl font-bold text-purple-900 mt-1">
                    {((healthData?.diversification_score || 0) * 100).toFixed(1)}%
                  </p>
                </div>
              </div>

              {/* Security Status */}
              <div className="bg-gray-50 border border-gray-200 rounded-lg p-6">
                <h3 className="text-lg font-medium text-gray-900 mb-4">Security Status</h3>
                <div className="space-y-2">
                  {(healthData?.security_alerts || []).map((alert: string, index: number) => (
                    <div key={index} className={`p-3 rounded-lg text-sm ${
                      alert.includes('✅') ? 'bg-green-100 text-green-800' :
                      alert.includes('⚠️') ? 'bg-yellow-100 text-yellow-800' :
                      alert.includes('🕐') ? 'bg-blue-100 text-blue-800' :
                      'bg-red-100 text-red-800'
                    }`}>
                      {alert}
                    </div>
                  ))}
                </div>
              </div>

              {/* Quick Actions */}
              <div className="bg-gray-50 border border-gray-200 rounded-lg p-6">
                <h3 className="text-lg font-medium text-gray-900 mb-4">Quick Actions</h3>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                  <button className="bg-white border border-gray-300 rounded-lg p-4 text-left hover:bg-gray-50 transition-colors">
                    <div className="flex items-center">
                      <div className="bg-blue-100 rounded-lg p-2 mr-3">
                        <svg className="h-5 w-5 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
                        </svg>
                      </div>
                      <div>
                        <p className="font-medium text-gray-900">Add Treasury Address</p>
                        <p className="text-sm text-gray-500">Configure new wallet address</p>
                      </div>
                    </div>
                  </button>
                  
                  <button className="bg-white border border-gray-300 rounded-lg p-4 text-left hover:bg-gray-50 transition-colors">
                    <div className="flex items-center">
                      <div className="bg-green-100 rounded-lg p-2 mr-3">
                        <svg className="h-5 w-5 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 7h6m0 10v-3m-3 3h.01M9 17h.01M9 14h.01M12 14h.01M15 11h.01M12 11h.01M9 11h.01M7 21h10a2 2 0 002-2V5a2 2 0 00-2-2H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
                        </svg>
                      </div>
                      <div>
                        <p className="font-medium text-gray-900">Generate Report</p>
                        <p className="text-sm text-gray-500">Export treasury report</p>
                      </div>
                    </div>
                  </button>
                  
                  <button className="bg-white border border-gray-300 rounded-lg p-4 text-left hover:bg-gray-50 transition-colors">
                    <div className="flex items-center">
                      <div className="bg-purple-100 rounded-lg p-2 mr-3">
                        <svg className="h-5 w-5 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                        </svg>
                      </div>
                      <div>
                        <p className="font-medium text-gray-900">Security Audit</p>
                        <p className="text-sm text-gray-500">Run security checks</p>
                      </div>
                    </div>
                  </button>
                </div>
              </div>
            </div>
          )}

          {/* Balances Section */}
          {activeSection === 'balances' && (
            <div className="space-y-4">
              <h3 className="text-lg font-medium text-gray-900">Treasury Balances</h3>
              <div className="bg-white border border-gray-200 rounded-lg overflow-hidden">
                <table className="min-w-full divide-y divide-gray-200">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Chain</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Asset</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Amount</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">USD Value</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Last Updated</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Actions</th>
                    </tr>
                  </thead>
                  <tbody className="bg-white divide-y divide-gray-200">
                    {balances.map((balance, index) => (
                      <tr key={index} className="hover:bg-gray-50">
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          <span className="capitalize font-medium">{balance.chain}</span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          <span className="uppercase font-bold">{balance.asset}</span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {balance.amount.toLocaleString()}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                          {formatCurrency(balance.amount_usd)}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                          {formatTimestamp(Number(balance.last_updated) / 1000000)}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                          <button className="text-blue-600 hover:text-blue-800">View Details</button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {/* Transactions Section */}
          {activeSection === 'transactions' && (
            <div className="space-y-4">
              <h3 className="text-lg font-medium text-gray-900">Recent Transactions</h3>
              <div className="bg-white border border-gray-200 rounded-lg overflow-hidden">
                <table className="min-w-full divide-y divide-gray-200">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Type</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Asset</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Amount</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Status</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Date</th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Notes</th>
                    </tr>
                  </thead>
                  <tbody className="bg-white divide-y divide-gray-200">
                    {transactions.map((transaction, index) => (
                      <tr key={index} className="hover:bg-gray-50">
                        <td className="px-6 py-4 whitespace-nowrap text-sm">
                          <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getTransactionTypeColor(transaction.transaction_type)}`}>
                            {transaction.transaction_type === 'TransactionFeeRevenue' ? 'Fee Revenue' :
                             transaction.transaction_type.replace(/([A-Z])/g, ' $1').trim()}
                          </span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                          <span className="uppercase">{transaction.asset}</span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {formatCurrency(transaction.amount_usd)}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm">
                          <span className={`font-medium ${getStatusColor(transaction.status)}`}>
                            {transaction.status}
                          </span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                          {formatTimestamp(Number(transaction.timestamp) / 1000000)}
                        </td>
                        <td className="px-6 py-4 text-sm text-gray-500 max-w-xs truncate">
                          {transaction.notes || '-'}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {/* Team Earnings Section */}
          {activeSection === 'earnings' && (
            <div className="space-y-6">
              <h3 className="text-lg font-medium text-gray-900">Team Earnings & Distribution</h3>
              
              {/* Earnings Distribution Overview */}
              <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div className="bg-green-50 border border-green-200 p-4 rounded-lg">
                  <h4 className="text-sm font-medium text-green-700">Total Team Earnings</h4>
                  <p className="text-2xl font-bold text-green-900 mt-1">
                    {formatCurrency(
                      Object.values(teamEarnings).reduce((total: number, member: any) => 
                        total + (member.total_usd_value || 0), 0
                      )
                    )}
                  </p>
                  <p className="text-xs text-green-600 mt-1">Available for withdrawal</p>
                </div>
                <div className="bg-purple-50 border border-purple-200 p-4 rounded-lg">
                  <h4 className="text-sm font-medium text-purple-700">Team Members</h4>
                  <p className="text-2xl font-bold text-purple-900 mt-1">{Object.keys(teamEarnings).length}</p>
                  <p className="text-xs text-purple-600 mt-1">Active earning members</p>
                </div>
                <div className="bg-blue-50 border border-blue-200 p-4 rounded-lg">
                  <h4 className="text-sm font-medium text-blue-700">Treasury Health</h4>
                  <p className="text-2xl font-bold text-blue-900 mt-1">
                    {formatCurrency(healthData?.total_usd_value || 0)}
                  </p>
                  <p className="text-xs text-blue-600 mt-1">Total treasury value</p>
                </div>
              </div>

              {/* Individual Team Earnings */}
              <div className="bg-white border border-gray-200 rounded-lg">
                <div className="px-6 py-4 border-b border-gray-200">
                  <h4 className="text-lg font-medium text-gray-900">Team Member Earnings</h4>
                  <p className="text-sm text-gray-600">Real-time earnings from pool canister</p>
                </div>
                <div className="p-6">
                  {isConnected && Object.keys(teamEarnings).length > 0 ? (
                    <div className="space-y-4">
                      {Object.entries(teamEarnings).map(([principal, earnings]: [string, any]) => (
                        <div key={principal} className="flex items-center justify-between p-4 bg-gray-50 rounded-lg">
                          <div className="flex-1">
                            <div className="flex items-center">
                              <p className="font-mono text-sm text-gray-800">
                                {principal.slice(0, 10)}...{principal.slice(-10)}
                              </p>
                              <span className="ml-2 px-2 py-1 bg-blue-100 text-blue-800 text-xs rounded-full">
                                Team Member
                              </span>
                            </div>
                            <p className="text-xs text-gray-600 mt-1">
                              Last distribution: {earnings.last_distribution_time ? 
                                new Date(Number(earnings.last_distribution_time) / 1000000).toLocaleDateString() : 
                                'Never'
                              }
                            </p>
                          </div>
                          <div className="text-right">
                            <p className="text-lg font-semibold text-gray-900">
                              {formatCurrency(earnings.total_usd_value || 0)}
                            </p>
                            <p className="text-xs text-gray-600">Available to withdraw</p>
                          </div>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <div className="text-center py-8">
                      <div className="text-gray-400 mb-2">
                        <svg className="h-12 w-12 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1} d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                        </svg>
                      </div>
                      <p className="text-gray-500 text-sm">
                        {error ? error : isConnected ? 'No team members found' : 'Connecting to treasury...'}
                      </p>
                    </div>
                  )}
                </div>
              </div>

              {/* Withdrawal Status */}
              <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-6">
                <div className="flex items-center mb-4">
                  <svg className="h-5 w-5 text-yellow-600 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.732-.833-2.5 0L4.314 16.5c-.77.833.192 2.5 1.732 2.5z" />
                  </svg>
                  <h4 className="text-lg font-medium text-yellow-900">Withdrawal System</h4>
                </div>
                <p className="text-yellow-800 mb-4">
                  Automated withdrawal system will be available when connected to the treasury backend.
                  Team members will be able to withdraw their earnings based on their custom percentage allocation.
                </p>
                <div className="bg-white border border-yellow-200 rounded-lg p-4">
                  <h5 className="font-medium text-yellow-900 mb-2">Planned Features:</h5>
                  <ul className="text-yellow-800 space-y-1">
                    <li>• Automatic earnings calculation based on owner-set percentages</li>
                    <li>• Multi-token withdrawal options (BTC, ETH, USDC, SOL, etc.)</li>
                    <li>• Withdrawal address management and verification</li>
                    <li>• Monthly/weekly withdrawal scheduling</li>
                    <li>• Real-time earnings tracking dashboard</li>
                    <li>• Conversion to ICP for team members who prefer</li>
                  </ul>
                </div>
              </div>
            </div>
          )}

          {/* Configure Section */}
          {activeSection === 'configure' && (
            <div className="space-y-6">
              <h3 className="text-lg font-medium text-gray-900">Treasury Configuration</h3>
              
              {/* Connected vs Demo Mode */}
              {!isConnected ? (
                <div className="bg-blue-50 border border-blue-200 rounded-lg p-6">
                  <div className="flex items-center mb-4">
                    <svg className="h-5 w-5 text-blue-600 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                    <h4 className="text-lg font-medium text-blue-900">Demo Configuration Preview</h4>
                  </div>
                  <p className="text-blue-800 mb-4">
                    The following configuration options will be available when connected to the treasury backend:
                  </p>
                  
                  {/* Configuration Sections Grid */}
                  <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                    
                    {/* Payment Addresses */}
                    <div className="bg-white border border-blue-200 rounded-lg p-4">
                      <h5 className="font-semibold text-blue-900 mb-3 flex items-center">
                        <span className="mr-2">🏦</span>
                        Payment Address Management
                      </h5>
                      <div className="space-y-3">
                        <div className="text-sm">
                          <p className="font-medium text-blue-800">Supported Chains:</p>
                          <div className="mt-2 space-y-2">
                            {['Bitcoin', 'Ethereum', 'Solana', 'Polygon', 'Arbitrum'].map((chain) => (
                              <div key={chain} className="flex items-center justify-between bg-blue-50 p-2 rounded">
                                <span className="text-blue-700">{chain}</span>
                                <span className="text-xs text-blue-600">✓ Configured</span>
                              </div>
                            ))}
                          </div>
                        </div>
                      </div>
                      <p className="text-xs text-blue-600 mt-3">
                        <strong>API:</strong> configure_payment_address(), get_payment_address()
                      </p>
                    </div>

                    {/* Hot Wallet Limits */}
                    <div className="bg-white border border-blue-200 rounded-lg p-4">
                      <h5 className="font-semibold text-blue-900 mb-3 flex items-center">
                        <span className="mr-2">🔥</span>
                        Hot Wallet Limits
                      </h5>
                      <div className="space-y-2">
                        <div className="flex justify-between items-center bg-blue-50 p-2 rounded text-sm">
                          <span className="text-blue-700">BTC Limit</span>
                          <span className="font-medium text-blue-800">$50,000</span>
                        </div>
                        <div className="flex justify-between items-center bg-blue-50 p-2 rounded text-sm">
                          <span className="text-blue-700">ETH Limit</span>
                          <span className="font-medium text-blue-800">$100,000</span>
                        </div>
                        <div className="flex justify-between items-center bg-blue-50 p-2 rounded text-sm">
                          <span className="text-blue-700">USDC Limit</span>
                          <span className="font-medium text-blue-800">$200,000</span>
                        </div>
                      </div>
                      <p className="text-xs text-blue-600 mt-3">
                        <strong>API:</strong> set_hot_wallet_limit()
                      </p>
                    </div>

                    {/* Withdrawal Management */}
                    <div className="bg-white border border-blue-200 rounded-lg p-4">
                      <h5 className="font-semibold text-blue-900 mb-3 flex items-center">
                        <span className="mr-2">💸</span>
                        Withdrawal Configuration
                      </h5>
                      <div className="space-y-2 text-sm">
                        <div className="flex justify-between items-center">
                          <span className="text-blue-700">Auto-approval threshold:</span>
                          <span className="font-medium text-blue-800">$1,000</span>
                        </div>
                        <div className="flex justify-between items-center">
                          <span className="text-blue-700">Manual approval required:</span>
                          <span className="font-medium text-blue-800">&gt;$1,000</span>
                        </div>
                        <div className="flex justify-between items-center">
                          <span className="text-blue-700">Daily withdrawal limit:</span>
                          <span className="font-medium text-blue-800">$25,000</span>
                        </div>
                      </div>
                      <p className="text-xs text-blue-600 mt-3">
                        <strong>API:</strong> request_treasury_withdrawal(), set_withdrawal_address()
                      </p>
                    </div>

                    {/* Security Settings */}
                    <div className="bg-white border border-blue-200 rounded-lg p-4">
                      <h5 className="font-semibold text-blue-900 mb-3 flex items-center">
                        <span className="mr-2">🔐</span>
                        Security & Monitoring
                      </h5>
                      <div className="space-y-2 text-sm">
                        <div className="flex items-center justify-between">
                          <span className="text-blue-700">Multi-sig threshold</span>
                          <span className="font-medium text-green-600">✓ Active</span>
                        </div>
                        <div className="flex items-center justify-between">
                          <span className="text-blue-700">Rate limiting</span>
                          <span className="font-medium text-green-600">✓ Enabled</span>
                        </div>
                        <div className="flex items-center justify-between">
                          <span className="text-blue-700">Address validation</span>
                          <span className="font-medium text-green-600">✓ Strict</span>
                        </div>
                        <div className="flex items-center justify-between">
                          <span className="text-blue-700">Storage limits</span>
                          <span className="font-medium text-yellow-600">⚠ Monitored</span>
                        </div>
                      </div>
                      <p className="text-xs text-blue-600 mt-3">
                        <strong>API:</strong> get_treasury_health_report(), validate_blockchain_address()
                      </p>
                    </div>

                    {/* Transaction Monitoring */}
                    <div className="bg-white border border-blue-200 rounded-lg p-4">
                      <h5 className="font-semibold text-blue-900 mb-3 flex items-center">
                        <span className="mr-2">📊</span>
                        Transaction Monitoring
                      </h5>
                      <div className="space-y-2 text-sm">
                        <div className="flex justify-between items-center">
                          <span className="text-blue-700">Fee collection rate:</span>
                          <span className="font-medium text-blue-800">30% to treasury</span>
                        </div>
                        <div className="flex justify-between items-center">
                          <span className="text-blue-700">Auto-distribution:</span>
                          <span className="font-medium text-green-600">✓ Monthly</span>
                        </div>
                        <div className="flex justify-between items-center">
                          <span className="text-blue-700">Transaction limit:</span>
                          <span className="font-medium text-blue-800">1,000 records</span>
                        </div>
                      </div>
                      <p className="text-xs text-blue-600 mt-3">
                        <strong>API:</strong> get_treasury_transactions(), deposit_transaction_fee()
                      </p>
                    </div>

                    {/* Chain Fusion */}
                    <div className="bg-white border border-blue-200 rounded-lg p-4">
                      <h5 className="font-semibold text-blue-900 mb-3 flex items-center">
                        <span className="mr-2">🌐</span>
                        Chain Fusion Integration
                      </h5>
                      <div className="space-y-2 text-sm">
                        <div className="flex items-center justify-between">
                          <span className="text-blue-700">Native addresses</span>
                          <span className="font-medium text-green-600">✓ Generated</span>
                        </div>
                        <div className="flex items-center justify-between">
                          <span className="text-blue-700">Cross-chain validation</span>
                          <span className="font-medium text-green-600">✓ Active</span>
                        </div>
                        <div className="flex items-center justify-between">
                          <span className="text-blue-700">Supported chains</span>
                          <span className="font-medium text-blue-800">5 networks</span>
                        </div>
                      </div>
                      <p className="text-xs text-blue-600 mt-3">
                        <strong>API:</strong> get_native_address(), validate_canister_address()
                      </p>
                    </div>
                  </div>

                  {/* Implementation Status */}
                  <div className="mt-6 p-4 bg-green-50 border border-green-200 rounded-lg">
                    <h5 className="font-semibold text-green-800 mb-2">✅ Implementation Status</h5>
                    <p className="text-green-700 text-sm mb-3">
                      All treasury configuration features are <strong>fully implemented</strong> in the backend canister with comprehensive security measures:
                    </p>
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                      <div>
                        <p className="font-medium text-green-800">Security Features:</p>
                        <ul className="text-green-700 mt-1 space-y-1">
                          <li>• Blockchain address validation for all chains</li>
                          <li>• Rate limiting and DoS protection</li>
                          <li>• Storage limits with automatic cleanup</li>
                          <li>• Manager-only access controls</li>
                        </ul>
                      </div>
                      <div>
                        <p className="font-medium text-green-800">Management Features:</p>
                        <ul className="text-green-700 mt-1 space-y-1">
                          <li>• Multi-chain payment address configuration</li>
                          <li>• Hot wallet limit management</li>
                          <li>• Automated treasury transaction recording</li>
                          <li>• Withdrawal request processing</li>
                        </ul>
                      </div>
                    </div>
                  </div>
                </div>
              ) : (
                <div className="bg-green-50 border border-green-200 rounded-lg p-6">
                  <h4 className="text-lg font-medium text-green-900 mb-4">Connected to Treasury Backend</h4>
                  <p className="text-green-800">
                    Treasury configuration interface will appear here when connected to the backend canister.
                  </p>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default TreasuryManagement;