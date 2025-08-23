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

interface TokenBalance {
  asset: string;
  amount: number;
  last_updated: bigint;
  usd_value_at_time: number;
}

interface MemberEarnings {
  balances: Record<string, TokenBalance>;
  total_usd_value: number;
  last_distribution_time: bigint;
  withdrawal_addresses: Record<string, string>;
}

interface WithdrawalOption {
  OriginalTokens?: null;
  ConvertToICP?: null;
  Mixed?: {
    original_tokens: string[];
    convert_to_icp: string[];
  };
}

const TreasuryManagement: React.FC = () => {
  const [balances, setBalances] = useState<TreasuryBalance[]>([]);
  const [transactions, setTransactions] = useState<TreasuryTransaction[]>([]);
  const [healthReport, setHealthReport] = useState<TreasuryHealthReport | null>(null);
  const [teamEarnings, setTeamEarnings] = useState<Record<string, MemberEarnings>>({});
  const [myEarnings, setMyEarnings] = useState<MemberEarnings | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeSection, setActiveSection] = useState<'overview' | 'balances' | 'transactions' | 'earnings' | 'configure'>('overview');
  const [showMixedWithdrawal, setShowMixedWithdrawal] = useState(false);
  const [mixedSelection, setMixedSelection] = useState<{
    original_tokens: string[];
    convert_to_icp: string[];
  }>({
    original_tokens: [],
    convert_to_icp: []
  });
  const [withdrawalAddresses, setWithdrawalAddresses] = useState<Record<string, string>>({});
  const [showAddressManager, setShowAddressManager] = useState(false);
  const [newAddress, setNewAddress] = useState('');
  const [selectedChain, setSelectedChain] = useState('Bitcoin');

  useEffect(() => {
    loadTreasuryData();
  }, []);

  const loadTreasuryData = async () => {
    try {
      setLoading(true);
      setError(null);

      const [healthData, balancesData, transactionsData, teamEarningsData, myEarningsData, addressesData] = await Promise.allSettled([
        AdminPoolService.getTreasuryHealthReport(),
        AdminPoolService.getAllTreasuryBalances(),
        AdminPoolService.getTreasuryTransactions(50),
        AdminPoolService.getAllTeamEarnings(),
        AdminPoolService.getMyDetailedEarnings(),
        AdminPoolService.getMyWithdrawalAddresses()
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
      if (teamEarningsData.status === 'fulfilled') {
        setTeamEarnings(teamEarningsData.value);
      }
      if (myEarningsData.status === 'fulfilled') {
        setMyEarnings(myEarningsData.value);
      }
      if (addressesData.status === 'fulfilled') {
        setWithdrawalAddresses(addressesData.value);
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

  const handleWithdrawal = async (option: WithdrawalOption) => {
    try {
      setLoading(true);
      const transfers = await AdminPoolService.withdrawWithOptions(option);
      
      // Show success message
      alert(`Withdrawal initiated! ${transfers.length} token transfers prepared.`);
      
      // Reload data to reflect changes
      await loadTreasuryData();
    } catch (error) {
      console.error('Withdrawal failed:', error);
      alert(`Withdrawal failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setLoading(false);
    }
  };

  const toggleTokenSelection = (asset: string, type: 'original' | 'convert') => {
    setMixedSelection(prev => {
      const newSelection = { ...prev };
      
      if (type === 'original') {
        // Remove from convert list if exists
        newSelection.convert_to_icp = newSelection.convert_to_icp.filter(a => a !== asset);
        
        // Toggle in original list
        if (newSelection.original_tokens.includes(asset)) {
          newSelection.original_tokens = newSelection.original_tokens.filter(a => a !== asset);
        } else {
          newSelection.original_tokens.push(asset);
        }
      } else {
        // Remove from original list if exists
        newSelection.original_tokens = newSelection.original_tokens.filter(a => a !== asset);
        
        // Toggle in convert list
        if (newSelection.convert_to_icp.includes(asset)) {
          newSelection.convert_to_icp = newSelection.convert_to_icp.filter(a => a !== asset);
        } else {
          newSelection.convert_to_icp.push(asset);
        }
      }
      
      return newSelection;
    });
  };

  const handleSetAddress = async () => {
    if (!newAddress.trim()) {
      alert('Please enter a valid address');
      return;
    }

    try {
      setLoading(true);
      await AdminPoolService.setWithdrawalAddress(selectedChain, newAddress.trim());
      
      // Update local state
      setWithdrawalAddresses(prev => ({
        ...prev,
        [selectedChain]: newAddress.trim()
      }));
      
      setNewAddress('');
      alert(`Address set successfully for ${selectedChain}`);
    } catch (error) {
      console.error('Failed to set address:', error);
      alert(`Failed to set address: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setLoading(false);
    }
  };

  const supportedChains = [
    { id: 'Bitcoin', name: 'Bitcoin', asset: 'BTC', example: '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa' },
    { id: 'Ethereum', name: 'Ethereum', asset: 'ETH', example: '0x742d35Cc6465C4F4c22c5e1e3e4AeGef3F2F3b1d' },
    { id: 'Polygon', name: 'Polygon', asset: 'MATIC', example: '0x742d35Cc6465C4F4c22c5e1e3e4AeGef3F2F3b1d' },
    { id: 'Arbitrum', name: 'Arbitrum', asset: 'ETH', example: '0x742d35Cc6465C4F4c22c5e1e3e4AeGef3F2F3b1d' },
    { id: 'Optimism', name: 'Optimism', asset: 'ETH', example: '0x742d35Cc6465C4F4c22c5e1e3e4AeGef3F2F3b1d' },
    { id: 'Base', name: 'Base', asset: 'ETH', example: '0x742d35Cc6465C4F4c22c5e1e3e4AeGef3F2F3b1d' },
    { id: 'Solana', name: 'Solana', asset: 'SOL', example: '11111111111111111111111111111112' },
    { id: 'Avalanche', name: 'Avalanche', asset: 'AVAX', example: '0x742d35Cc6465C4F4c22c5e1e3e4AeGef3F2F3b1d' }
  ];

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
              { id: 'earnings', label: 'Team Earnings', icon: '💎' },
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

          {/* Team Earnings Section */}
          {activeSection === 'earnings' && (
            <div className="space-y-6">
              <h3 className="text-lg font-medium text-white">Team Earnings & Withdrawals</h3>
              
              {/* My Earnings */}
              {myEarnings && (
                <div className="bg-blue-900/30 rounded-lg p-6 border border-blue-700">
                  <h4 className="text-lg font-medium text-blue-300 mb-4">💰 My Earnings</h4>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <div>
                      <p className="text-sm text-gray-400">Total USD Value</p>
                      <p className="text-2xl font-bold text-white">
                        {formatCurrency(myEarnings.total_usd_value)}
                      </p>
                    </div>
                    <div>
                      <p className="text-sm text-gray-400">Last Distribution</p>
                      <p className="text-white">
                        {formatTimestamp(myEarnings.last_distribution_time)}
                      </p>
                    </div>
                  </div>
                  
                  {/* Token Breakdown */}
                  <div className="mt-6">
                    <h5 className="text-sm font-medium text-blue-300 mb-3">Token Breakdown</h5>
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                      {Object.entries(myEarnings.balances).map(([asset, balance]) => (
                        <div key={asset} className="bg-gray-800 p-3 rounded-lg border border-gray-600">
                          <div className="flex justify-between items-center">
                            <span className="text-sm font-medium text-white uppercase">{asset}</span>
                            <span className="text-xs text-gray-400">
                              {formatTimestamp(balance.last_updated)}
                            </span>
                          </div>
                          <p className="text-lg font-bold text-white mt-1">
                            {balance.amount.toLocaleString()}
                          </p>
                          <p className="text-sm text-gray-400">
                            {formatCurrency(balance.usd_value_at_time)}
                          </p>
                        </div>
                      ))}
                    </div>
                  </div>

                  {/* Withdrawal Addresses */}
                  <div className="mt-6">
                    <div className="flex justify-between items-center mb-3">
                      <h5 className="text-sm font-medium text-blue-300">Withdrawal Addresses</h5>
                      <button
                        onClick={() => setShowAddressManager(!showAddressManager)}
                        className="text-xs bg-blue-600 hover:bg-blue-700 text-white px-3 py-1 rounded transition-colors"
                      >
                        {showAddressManager ? 'Hide' : 'Manage Addresses'}
                      </button>
                    </div>
                    
                    {/* Current Addresses */}
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-3 mb-4">
                      {supportedChains.map((chain) => (
                        <div key={chain.id} className="bg-gray-800 p-3 rounded-lg border border-gray-600">
                          <div className="flex justify-between items-center">
                            <div>
                              <span className="text-sm font-medium text-white">{chain.name}</span>
                              <span className="text-xs text-gray-400 ml-2">({chain.asset})</span>
                            </div>
                            <span className={`text-xs px-2 py-1 rounded ${
                              withdrawalAddresses[chain.id] 
                                ? 'bg-green-600 text-white' 
                                : 'bg-red-600 text-white'
                            }`}>
                              {withdrawalAddresses[chain.id] ? 'Set' : 'Not Set'}
                            </span>
                          </div>
                          {withdrawalAddresses[chain.id] && (
                            <p className="text-xs text-gray-400 mt-1 font-mono truncate">
                              {withdrawalAddresses[chain.id]}
                            </p>
                          )}
                        </div>
                      ))}
                    </div>

                    {/* Address Manager */}
                    {showAddressManager && (
                      <div className="bg-gray-800 p-4 rounded-lg border border-gray-600">
                        <h6 className="text-sm font-medium text-white mb-3">Set Withdrawal Addresses</h6>
                        <div className="space-y-4">
                          <div>
                            <label className="block text-sm text-gray-400 mb-2">Select Chain</label>
                            <select
                              value={selectedChain}
                              onChange={(e) => setSelectedChain(e.target.value)}
                              className="w-full bg-gray-700 text-white px-3 py-2 rounded border border-gray-600 focus:border-blue-500"
                            >
                              {supportedChains.map((chain) => (
                                <option key={chain.id} value={chain.id}>
                                  {chain.name} ({chain.asset})
                                </option>
                              ))}
                            </select>
                          </div>
                          <div>
                            <label className="block text-sm text-gray-400 mb-2">
                              Address for {selectedChain}
                            </label>
                            <input
                              type="text"
                              value={newAddress}
                              onChange={(e) => setNewAddress(e.target.value)}
                              placeholder={supportedChains.find(c => c.id === selectedChain)?.example}
                              className="w-full bg-gray-700 text-white px-3 py-2 rounded border border-gray-600 focus:border-blue-500 font-mono text-sm"
                            />
                            <p className="text-xs text-gray-500 mt-1">
                              Current: {withdrawalAddresses[selectedChain] || 'Not set'}
                            </p>
                          </div>
                          <button
                            onClick={handleSetAddress}
                            disabled={!newAddress.trim()}
                            className="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 text-white px-4 py-2 rounded transition-colors"
                          >
                            Set Address
                          </button>
                        </div>
                      </div>
                    )}
                  </div>

                  {/* Withdrawal Options */}
                  <div className="mt-6">
                    <h5 className="text-sm font-medium text-blue-300 mb-3">Withdrawal Options</h5>
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
                      <button 
                        onClick={() => handleWithdrawal({ OriginalTokens: null })}
                        className="bg-green-600 hover:bg-green-700 text-white px-4 py-3 rounded-lg transition-colors"
                      >
                        <div className="text-left">
                          <p className="font-medium">Keep Original Tokens</p>
                          <p className="text-sm opacity-90">Receive BTC, ETH, USDC, etc.</p>
                        </div>
                      </button>
                      <button 
                        onClick={() => handleWithdrawal({ ConvertToICP: null })}
                        className="bg-purple-600 hover:bg-purple-700 text-white px-4 py-3 rounded-lg transition-colors"
                      >
                        <div className="text-left">
                          <p className="font-medium">Convert to ICP</p>
                          <p className="text-sm opacity-90">Convert everything to ICP</p>
                        </div>
                      </button>
                      <button 
                        onClick={() => setShowMixedWithdrawal(!showMixedWithdrawal)}
                        className="bg-orange-600 hover:bg-orange-700 text-white px-4 py-3 rounded-lg transition-colors"
                      >
                        <div className="text-left">
                          <p className="font-medium">Mixed Withdrawal</p>
                          <p className="text-sm opacity-90">Custom per-token selection</p>
                        </div>
                      </button>
                    </div>
                  </div>

                  {/* Mixed Withdrawal Options */}
                  {showMixedWithdrawal && (
                    <div className="mt-4 p-4 bg-gray-800 rounded-lg border border-gray-600">
                      <h6 className="text-sm font-medium text-white mb-3">Select withdrawal method for each token:</h6>
                      <div className="space-y-2">
                        {Object.keys(myEarnings.balances).map((asset) => (
                          <div key={asset} className="flex items-center justify-between">
                            <span className="text-white uppercase font-medium">{asset}</span>
                            <div className="flex space-x-2">
                              <button
                                onClick={() => toggleTokenSelection(asset, 'original')}
                                className={`px-3 py-1 rounded text-sm ${
                                  mixedSelection.original_tokens.includes(asset)
                                    ? 'bg-green-600 text-white' 
                                    : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                                }`}
                              >
                                Keep Original
                              </button>
                              <button
                                onClick={() => toggleTokenSelection(asset, 'convert')}
                                className={`px-3 py-1 rounded text-sm ${
                                  mixedSelection.convert_to_icp.includes(asset)
                                    ? 'bg-purple-600 text-white' 
                                    : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                                }`}
                              >
                                Convert to ICP
                              </button>
                            </div>
                          </div>
                        ))}
                      </div>
                      <button
                        onClick={() => handleWithdrawal({ Mixed: mixedSelection })}
                        className="mt-4 w-full bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg"
                      >
                        Execute Mixed Withdrawal
                      </button>
                    </div>
                  )}
                </div>
              )}

              {/* All Team Earnings (Owner Only) */}
              <div className="bg-gray-900/50 rounded-lg p-6 border border-gray-700">
                <h4 className="text-lg font-medium text-white mb-4">👥 All Team Member Earnings</h4>
                {Object.keys(teamEarnings).length > 0 ? (
                  <div className="space-y-4">
                    {Object.entries(teamEarnings).map(([principal, earnings]) => (
                      <div key={principal} className="bg-gray-800 p-4 rounded-lg border border-gray-600">
                        <div className="flex justify-between items-start mb-3">
                          <div>
                            <p className="text-white font-medium">Team Member</p>
                            <p className="text-xs text-gray-400 font-mono">{principal}</p>
                          </div>
                          <div className="text-right">
                            <p className="text-lg font-bold text-white">
                              {formatCurrency(earnings.total_usd_value)}
                            </p>
                            <p className="text-xs text-gray-400">
                              {Object.keys(earnings.balances).length} tokens
                            </p>
                          </div>
                        </div>
                        <div className="grid grid-cols-2 md:grid-cols-4 gap-2">
                          {Object.entries(earnings.balances).map(([asset, balance]) => (
                            <div key={asset} className="bg-gray-700 p-2 rounded text-center">
                              <p className="text-xs text-gray-300 uppercase">{asset}</p>
                              <p className="text-sm font-medium text-white">{balance.amount.toLocaleString()}</p>
                            </div>
                          ))}
                        </div>
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="text-gray-400">No team earnings data available.</p>
                )}
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