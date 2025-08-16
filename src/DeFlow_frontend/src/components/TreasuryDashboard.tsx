/**
 * Treasury Dashboard Component
 * Provides comprehensive treasury monitoring for team members
 */

import React, { useState, useEffect } from 'react';
import TreasuryService, { 
  TreasuryHealthReport, 
  TreasuryBalance, 
  TreasuryTransaction, 
  PaymentAddress 
} from '../services/treasuryService';

interface TreasuryDashboardProps {
  userRole?: string;
}

const TreasuryDashboard: React.FC<TreasuryDashboardProps> = ({ userRole }) => {
  const [healthReport, setHealthReport] = useState<TreasuryHealthReport | null>(null);
  const [balances, setBalances] = useState<TreasuryBalance[]>([]);
  const [transactions, setTransactions] = useState<TreasuryTransaction[]>([]);
  const [addresses, setAddresses] = useState<PaymentAddress[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'overview' | 'balances' | 'transactions' | 'addresses'>('overview');

  useEffect(() => {
    loadTreasuryData();
  }, []);

  const loadTreasuryData = async () => {
    try {
      setLoading(true);
      setError(null);

      const [healthData, balancesData, transactionsData, addressesData] = await Promise.allSettled([
        TreasuryService.getTreasuryHealthReport(),
        TreasuryService.getAllTreasuryBalances(),
        TreasuryService.getTreasuryTransactions(20),
        TreasuryService.getAllPaymentAddresses()
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
      if (addressesData.status === 'fulfilled') {
        setAddresses(addressesData.value);
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

  const formatAddress = (address: string) => {
    if (address.length > 20) {
      return `${address.slice(0, 8)}...${address.slice(-8)}`;
    }
    return address;
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Confirmed': return 'text-green-600';
      case 'Pending': return 'text-yellow-600';
      case 'Failed': return 'text-red-600';
      default: return 'text-gray-600';
    }
  };

  const getAddressTypeColor = (type: string) => {
    switch (type) {
      case 'Hot': return 'bg-orange-100 text-orange-800';
      case 'Warm': return 'bg-yellow-100 text-yellow-800';
      case 'Cold': return 'bg-blue-100 text-blue-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  if (loading) {
    return (
      <div className="bg-white rounded-lg shadow p-6">
        <div className="animate-pulse">
          <div className="h-4 bg-gray-200 rounded w-1/4 mb-4"></div>
          <div className="space-y-3">
            <div className="h-3 bg-gray-200 rounded"></div>
            <div className="h-3 bg-gray-200 rounded w-5/6"></div>
            <div className="h-3 bg-gray-200 rounded w-4/6"></div>
          </div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-4">
        <h3 className="text-red-800 font-medium">Error Loading Treasury Data</h3>
        <p className="text-red-600 mt-1">{error}</p>
        <button 
          onClick={loadTreasuryData}
          className="mt-3 bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700"
        >
          Retry
        </button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-white rounded-lg shadow p-6">
        <div className="flex justify-between items-center">
          <div>
            <h2 className="text-2xl font-bold text-gray-900">Treasury Dashboard</h2>
            <p className="text-gray-600 mt-1">Monitor DeFlow payment system and treasury health</p>
          </div>
          <button 
            onClick={loadTreasuryData}
            className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700"
          >
            Refresh Data
          </button>
        </div>
      </div>

      {/* Navigation Tabs */}
      <div className="bg-white rounded-lg shadow">
        <div className="border-b border-gray-200">
          <nav className="-mb-px flex">
            {[
              { id: 'overview', label: 'Overview' },
              { id: 'balances', label: 'Balances' },
              { id: 'transactions', label: 'Transactions' },
              { id: 'addresses', label: 'Addresses' }
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
                {tab.label}
              </button>
            ))}
          </nav>
        </div>

        <div className="p-6">
          {/* Overview Tab */}
          {activeTab === 'overview' && healthReport && (
            <div className="space-y-6">
              {/* Key Metrics */}
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <div className="bg-blue-50 p-4 rounded-lg">
                  <h3 className="text-sm font-medium text-blue-600">Total Treasury Value</h3>
                  <p className="text-2xl font-bold text-blue-900 mt-1">
                    {formatCurrency(healthReport.total_usd_value)}
                  </p>
                </div>
                <div className="bg-green-50 p-4 rounded-lg">
                  <h3 className="text-sm font-medium text-green-600">Total Assets</h3>
                  <p className="text-2xl font-bold text-green-900 mt-1">
                    {healthReport.total_assets}
                  </p>
                </div>
                <div className="bg-yellow-50 p-4 rounded-lg">
                  <h3 className="text-sm font-medium text-yellow-600">Hot Wallet Usage</h3>
                  <p className="text-2xl font-bold text-yellow-900 mt-1">
                    {healthReport.hot_wallet_utilization.toFixed(1)}%
                  </p>
                </div>
                <div className="bg-purple-50 p-4 rounded-lg">
                  <h3 className="text-sm font-medium text-purple-600">Diversification</h3>
                  <p className="text-2xl font-bold text-purple-900 mt-1">
                    {(healthReport.diversification_score * 100).toFixed(1)}%
                  </p>
                </div>
              </div>

              {/* Security Alerts */}
              <div className="bg-gray-50 p-4 rounded-lg">
                <h3 className="text-lg font-medium text-gray-900 mb-3">Security Alerts</h3>
                <div className="space-y-2">
                  {healthReport.security_alerts.map((alert, index) => (
                    <div key={index} className={`p-2 rounded text-sm ${
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

              {/* Balances Over Limit */}
              {healthReport.balances_over_limit.length > 0 && (
                <div className="bg-red-50 p-4 rounded-lg">
                  <h3 className="text-lg font-medium text-red-800 mb-3">Balances Over Limit</h3>
                  <div className="space-y-2">
                    {healthReport.balances_over_limit.map((balance, index) => (
                      <div key={index} className="text-red-700 text-sm">
                        {balance}
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}

          {/* Balances Tab */}
          {activeTab === 'balances' && (
            <div className="space-y-4">
              <h3 className="text-lg font-medium text-gray-900">Treasury Balances</h3>
              <div className="overflow-x-auto">
                <table className="min-w-full divide-y divide-gray-200">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Chain
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Asset
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Amount
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        USD Value
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Last Updated
                      </th>
                    </tr>
                  </thead>
                  <tbody className="bg-white divide-y divide-gray-200">
                    {balances.map((balance, index) => (
                      <tr key={index}>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          <span className="capitalize">{balance.chain}</span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          <span className="uppercase font-medium">{balance.asset}</span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {balance.amount.toLocaleString()}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {formatCurrency(balance.amount_usd)}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                          {new Date(Number(balance.last_updated) / 1000000).toLocaleDateString()}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {/* Transactions Tab */}
          {activeTab === 'transactions' && (
            <div className="space-y-4">
              <h3 className="text-lg font-medium text-gray-900">Recent Transactions</h3>
              <div className="overflow-x-auto">
                <table className="min-w-full divide-y divide-gray-200">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Type
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Asset
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Amount
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Status
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Date
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        TX Hash
                      </th>
                    </tr>
                  </thead>
                  <tbody className="bg-white divide-y divide-gray-200">
                    {transactions.map((transaction, index) => (
                      <tr key={index}>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                            transaction.transaction_type === 'TransactionFeeRevenue' ? 'bg-green-100 text-green-800' :
                            transaction.transaction_type === 'SubscriptionPayment' ? 'bg-blue-100 text-blue-800' :
                            transaction.transaction_type === 'WithdrawalToTeam' ? 'bg-purple-100 text-purple-800' :
                            'bg-gray-100 text-gray-800'
                          }`}>
                            {transaction.transaction_type === 'TransactionFeeRevenue' ? 'Fee Revenue (30%)' :
                             transaction.transaction_type === 'SubscriptionPayment' ? 'Subscription' :
                             transaction.transaction_type.replace(/([A-Z])/g, ' $1').trim()}
                          </span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          <span className="uppercase font-medium">{transaction.asset}</span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {formatCurrency(transaction.amount_usd)}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm">
                          <span className={getStatusColor(transaction.status)}>
                            {transaction.status}
                          </span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                          {new Date(Number(transaction.timestamp) / 1000000).toLocaleDateString()}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                          {transaction.tx_hash ? (
                            <span className="font-mono">{formatAddress(transaction.tx_hash)}</span>
                          ) : (
                            '-'
                          )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {/* Addresses Tab */}
          {activeTab === 'addresses' && (
            <div className="space-y-4">
              <h3 className="text-lg font-medium text-gray-900">Payment Addresses</h3>
              <div className="grid gap-4">
                {addresses.map((address, index) => (
                  <div key={index} className="border border-gray-200 rounded-lg p-4">
                    <div className="flex justify-between items-start">
                      <div className="flex-1">
                        <div className="flex items-center space-x-2">
                          <span className="font-medium capitalize">{address.chain}</span>
                          <span className="uppercase text-sm font-medium text-gray-600">{address.asset}</span>
                          <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getAddressTypeColor(address.address_type)}`}>
                            {address.address_type}
                          </span>
                        </div>
                        <div className="mt-2">
                          <p className="text-sm text-gray-600">Address:</p>
                          <p className="font-mono text-sm text-gray-900 break-all">{address.address}</p>
                        </div>
                        {address.max_balance_usd && (
                          <div className="mt-2">
                            <p className="text-sm text-gray-600">
                              Max Balance: {formatCurrency(address.max_balance_usd)}
                            </p>
                          </div>
                        )}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default TreasuryDashboard;