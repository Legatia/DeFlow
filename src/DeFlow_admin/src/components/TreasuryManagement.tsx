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

const TreasuryManagement: React.FC = () => {
  const [balances, setBalances] = useState<TreasuryBalance[]>([]);
  const [transactions, setTransactions] = useState<TreasuryTransaction[]>([]);
  const [healthReport, setHealthReport] = useState<TreasuryHealthReport | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeSection, setActiveSection] = useState<'overview' | 'balances' | 'transactions' | 'configure'>('overview');

  useEffect(() => {
    loadTreasuryData();
  }, []);

  const loadTreasuryData = async () => {
    try {
      setLoading(true);
      setError(null);

      const [healthData, balancesData, transactionsData] = await Promise.allSettled([
        AdminPoolService.getTreasuryHealthReport(),
        AdminPoolService.getAllTreasuryBalances(),
        AdminPoolService.getTreasuryTransactions(50)
      ]);

      if (healthData.status === 'fulfilled') {
        setHealthReport(healthData.value);
      }
      if (balancesData.status === 'fulfilled') {
        setBalances(balancesData.value);
      }
      if (transactionsData.status === 'fulfilled') {
        setTransactions(transactionsData.value);
      }

    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load treasury data');
    } finally {
      setLoading(false);
    }
  };

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 2,
      maximumFractionDigits: 2
    }).format(amount);
  };

  const formatTimestamp = (timestamp: bigint) => {
    const date = new Date(Number(timestamp) / 1000000);
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
        <p className="text-gray-400 mt-4">Loading treasury data...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-900/20 border border-red-500 rounded-lg p-6">
        <h3 className="text-red-400 font-medium">Error Loading Treasury Data</h3>
        <p className="text-red-300 mt-2">{error}</p>
        <button 
          onClick={loadTreasuryData}
          className="mt-4 bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700"
        >
          Retry
        </button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-gray-800 rounded-lg p-6">
        <div className="flex justify-between items-center">
          <div>
            <h2 className="text-2xl font-bold text-white">Treasury Management</h2>
            <p className="text-gray-400 mt-1">Monitor and manage DeFlow treasury assets</p>
          </div>
          <button 
            onClick={loadTreasuryData}
            className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700"
          >
            <span className="flex items-center">
              <svg className="h-4 w-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
              Refresh
            </span>
          </button>
        </div>
      </div>

      {/* Navigation */}
      <div className="bg-gray-800 rounded-lg">
        <div className="border-b border-gray-700">
          <nav className="-mb-px flex">
            {[
              { id: 'overview', label: 'Overview', icon: '📊' },
              { id: 'balances', label: 'Balances', icon: '💰' },
              { id: 'transactions', label: 'Transactions', icon: '📋' },
              { id: 'configure', label: 'Configure', icon: '⚙️' }
            ].map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveSection(tab.id as any)}
                className={`py-3 px-6 text-sm font-medium border-b-2 ${
                  activeSection === tab.id
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

        <div className="p-6">
          {/* Overview Section */}
          {activeSection === 'overview' && healthReport && (
            <div className="space-y-6">
              {/* Key Metrics */}
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <div className="bg-blue-900/50 p-4 rounded-lg border border-blue-700">
                  <h3 className="text-sm font-medium text-blue-300">Total Treasury Value</h3>
                  <p className="text-2xl font-bold text-white mt-1">
                    {formatCurrency(healthReport.total_usd_value)}
                  </p>
                </div>
                <div className="bg-green-900/50 p-4 rounded-lg border border-green-700">
                  <h3 className="text-sm font-medium text-green-300">Total Assets</h3>
                  <p className="text-2xl font-bold text-white mt-1">
                    {healthReport.total_assets}
                  </p>
                </div>
                <div className="bg-yellow-900/50 p-4 rounded-lg border border-yellow-700">
                  <h3 className="text-sm font-medium text-yellow-300">Hot Wallet Usage</h3>
                  <p className="text-2xl font-bold text-white mt-1">
                    {healthReport.hot_wallet_utilization.toFixed(1)}%
                  </p>
                </div>
                <div className="bg-purple-900/50 p-4 rounded-lg border border-purple-700">
                  <h3 className="text-sm font-medium text-purple-300">Diversification</h3>
                  <p className="text-2xl font-bold text-white mt-1">
                    {(healthReport.diversification_score * 100).toFixed(1)}%
                  </p>
                </div>
              </div>

              {/* Security Alerts */}
              <div className="bg-gray-900/50 p-4 rounded-lg border border-gray-700">
                <h3 className="text-lg font-medium text-white mb-3">Security Status</h3>
                <div className="space-y-2">
                  {healthReport.security_alerts.map((alert, index) => (
                    <div key={index} className={`p-2 rounded text-sm ${
                      alert.includes('✅') ? 'bg-green-900/30 text-green-300' :
                      alert.includes('⚠️') ? 'bg-yellow-900/30 text-yellow-300' :
                      alert.includes('🕐') ? 'bg-blue-900/30 text-blue-300' :
                      'bg-red-900/30 text-red-300'
                    }`}>
                      {alert}
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {/* Balances Section */}
          {activeSection === 'balances' && (
            <div className="space-y-4">
              <h3 className="text-lg font-medium text-white">Treasury Balances</h3>
              <div className="overflow-x-auto">
                <table className="min-w-full divide-y divide-gray-700">
                  <thead className="bg-gray-700">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">
                        Chain
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">
                        Asset
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">
                        Amount
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">
                        USD Value
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">
                        Last Updated
                      </th>
                    </tr>
                  </thead>
                  <tbody className="bg-gray-800 divide-y divide-gray-700">
                    {balances.map((balance, index) => (
                      <tr key={index}>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-white">
                          <span className="capitalize">{balance.chain}</span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-white">
                          <span className="uppercase font-medium">{balance.asset}</span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-white">
                          {balance.amount.toLocaleString()}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-white">
                          {formatCurrency(balance.amount_usd)}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-400">
                          {formatTimestamp(balance.last_updated)}
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
              <h3 className="text-lg font-medium text-white">Recent Transactions</h3>
              <div className="overflow-x-auto">
                <table className="min-w-full divide-y divide-gray-700">
                  <thead className="bg-gray-700">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">
                        Type
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">
                        Asset
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">
                        Amount
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">
                        Status
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">
                        Date
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase tracking-wider">
                        Notes
                      </th>
                    </tr>
                  </thead>
                  <tbody className="bg-gray-800 divide-y divide-gray-700">
                    {transactions.map((transaction, index) => (
                      <tr key={index}>
                        <td className="px-6 py-4 whitespace-nowrap text-sm">
                          <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getTransactionTypeColor(transaction.transaction_type)}`}>
                            {transaction.transaction_type === 'TransactionFeeRevenue' ? 'Fee Revenue (30%)' :
                             transaction.transaction_type.replace(/([A-Z])/g, ' $1').trim()}
                          </span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-white">
                          <span className="uppercase font-medium">{transaction.asset}</span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-white">
                          {formatCurrency(transaction.amount_usd)}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm">
                          <span className={getStatusColor(transaction.status)}>
                            {transaction.status}
                          </span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-400">
                          {formatTimestamp(transaction.timestamp)}
                        </td>
                        <td className="px-6 py-4 text-sm text-gray-400 max-w-xs truncate">
                          {transaction.notes || '-'}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {/* Configure Section */}
          {activeSection === 'configure' && (
            <div className="space-y-6">
              <h3 className="text-lg font-medium text-white">Treasury Configuration</h3>
              <div className="bg-yellow-900/20 border border-yellow-500 rounded-lg p-4">
                <p className="text-yellow-300">
                  🚧 Treasury configuration features coming soon. This will include:
                </p>
                <ul className="text-yellow-200 mt-2 ml-4 space-y-1">
                  <li>• Payment address management</li>
                  <li>• Withdrawal approval settings</li>
                  <li>• Security threshold configuration</li>
                  <li>• Team member permissions</li>
                </ul>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default TreasuryManagement;