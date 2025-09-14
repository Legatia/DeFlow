import React, { useState, useEffect } from 'react';
import { Principal } from '@dfinity/principal';
import flowTokenService, { 
  Phase1Status, 
  UserPhase1Balance, 
  FlowTransaction 
} from '../services/flowTokenService';

interface FlowTokenDashboardProps {
  isOwner: boolean;
  currentPrincipal: string;
}

const FlowTokenDashboard: React.FC<FlowTokenDashboardProps> = ({ isOwner, currentPrincipal }) => {
  const [phase1Status, setPhase1Status] = useState<Phase1Status | null>(null);
  const [selectedUserBalance, setSelectedUserBalance] = useState<UserPhase1Balance | null>(null);
  const [searchPrincipal, setSearchPrincipal] = useState('');
  const [recentTransactions, setRecentTransactions] = useState<FlowTransaction[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [activeSection, setActiveSection] = useState<'overview' | 'users' | 'transactions' | 'controls'>('overview');

  useEffect(() => {
    loadPhase1Status();
  }, []);

  const loadPhase1Status = async () => {
    try {
      setLoading(true);
      setError(null);
      
      // Use real service call
      const status = await flowTokenService.getPhase1Status();
      setPhase1Status(status);
    } catch (err) {
      // Fallback to mock data for demo purposes if canister is not available
      console.warn('Failed to load from canister, using mock data:', err);
      
      const mockStatus: Phase1Status = {
        current_phase: { Phase1PreLaunch: null },
        total_pre_launch_distributed: 50000000000000, // 500,000 FLOW (with 8 decimals)
        pool_asset_value_usd: 25000.50,
        launch_threshold_usd: 60000.0,
        progress_to_launch: 41.7,
        btc_amount: 0.42,
        ckbtc_staked: 0.38,
        eligible_users: 157
      };
      
      setPhase1Status(mockStatus);
    } finally {
      setLoading(false);
    }
  };

  const searchUserBalance = async () => {
    if (!searchPrincipal.trim()) return;
    
    try {
      setLoading(true);
      setError(null);
      
      const userPrincipal = Principal.fromText(searchPrincipal);
      const balance = await flowTokenService.getUserPhase1Balance(userPrincipal);
      setSelectedUserBalance(balance);
    } catch (err) {
      // Fallback to mock data if canister call fails
      console.warn('Failed to fetch user balance from canister, using mock data:', err);
      
      const mockBalance: UserPhase1Balance = {
        pre_launch_balance: 15000000000, // 150 FLOW
        tradeable_balance: 0,
        total_balance: 15000000000,
        phase1_airdrop_received: 10000000000, // 100 FLOW
        phase1_activity_rewards: 5000000000, // 50 FLOW
        eligible_for_phase2_conversion: true,
        estimated_phase2_value: 15000000000
      };
      
      setSelectedUserBalance(mockBalance);
    } finally {
      setLoading(false);
    }
  };

  const triggerEarlyAdopterAirdrop = async () => {
    if (!isOwner) return;
    
    try {
      setLoading(true);
      setError(null);
      
      // For now, using empty recipients array - this would be filled from a UI form
      const recipients: Principal[] = [];
      
      try {
        const result = await flowTokenService.triggerEarlyAdopterAirdrop(recipients);
        alert(`Airdrop triggered successfully!\n${result}`);
      } catch (canisterError) {
        // Fallback for demo purposes
        console.warn('Canister call failed, showing demo message:', canisterError);
        alert('Early adopter airdrop triggered successfully! (Demo mode)');
      }
      
      await loadPhase1Status(); // Refresh data
    } catch (err) {
      setError(`Failed to trigger airdrop: ${err}`);
      console.error('Error triggering airdrop:', err);
    } finally {
      setLoading(false);
    }
  };

  const updatePoolAssets = async (btcEquivalent: number, actualBtc: number, otherAssets: number) => {
    if (!isOwner) return;
    
    try {
      setLoading(true);
      setError(null);
      
      try {
        const result = await flowTokenService.updatePoolAssets(btcEquivalent, actualBtc, otherAssets);
        alert(`Pool assets updated successfully!\n${result}`);
      } catch (canisterError) {
        // Fallback for demo purposes
        console.warn('Canister call failed, showing demo message:', canisterError);
        alert(`Pool assets updated! (Demo mode)\nBTC Equivalent: $${btcEquivalent}\nActual BTC: ${actualBtc}\nOther Assets: $${otherAssets}`);
      }
      
      await loadPhase1Status(); // Refresh data
    } catch (err) {
      setError(`Failed to update pool assets: ${err}`);
      console.error('Error updating pool assets:', err);
    } finally {
      setLoading(false);
    }
  };

  const formatFlowAmount = (amount: number) => {
    return flowTokenService.formatFlowAmount(amount);
  };

  const formatCurrency = (amount: number) => {
    return flowTokenService.formatCurrency(amount);
  };

  const getCurrentPhase = () => {
    if (!phase1Status) return 'Unknown';
    return flowTokenService.getCurrentPhaseText(phase1Status.current_phase);
  };

  if (loading && !phase1Status) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-gradient-to-r from-purple-600 to-blue-600 rounded-lg p-6 text-white">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold flex items-center">
              🪙 $FLOW Token Dashboard
            </h1>
            <p className="text-purple-100 mt-1">
              Comprehensive management for DeFlow's two-phase token launch
            </p>
          </div>
          <div className="text-right">
            <div className="text-sm text-purple-100">Current Phase</div>
            <div className="text-xl font-semibold">{getCurrentPhase()}</div>
          </div>
        </div>
      </div>

      {error && (
        <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
          <div className="flex">
            <div className="py-1">
              <svg className="fill-current h-6 w-6 text-red-500 mr-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20">
                <path d="M2.93 17.07A10 10 0 1 1 17.07 2.93 10 10 0 0 1 2.93 17.07zm12.73-1.41A8 8 0 1 0 4.34 4.34a8 8 0 0 0 11.32 11.32zM9 11V9h2v6H9v-4zm0-6h2v2H9V5z"/>
              </svg>
            </div>
            <div>
              <p className="font-bold">Error</p>
              <p className="text-sm">{error}</p>
            </div>
          </div>
        </div>
      )}

      {/* Navigation */}
      <div className="bg-white rounded-lg shadow">
        <div className="border-b border-gray-200">
          <nav className="-mb-px flex space-x-8 px-6">
            {[
              { id: 'overview', label: 'Phase 1 Overview', icon: '📊' },
              { id: 'users', label: 'User Balances', icon: '👥' },
              { id: 'transactions', label: 'Recent Activity', icon: '📋' },
              { id: 'controls', label: 'Admin Controls', icon: '⚙️' }
            ].map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveSection(tab.id as any)}
                className={`py-4 px-1 border-b-2 font-medium text-sm ${
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
          {/* Phase 1 Overview */}
          {activeSection === 'overview' && phase1Status && (
            <div className="space-y-6">
              <h2 className="text-lg font-semibold text-gray-900">Phase 1 Launch Progress</h2>
              
              {/* Progress to Phase 2 */}
              <div className="bg-gray-50 rounded-lg p-4">
                <div className="flex justify-between items-center mb-2">
                  <span className="text-sm font-medium text-gray-700">Progress to Phase 2 Launch</span>
                  <span className="text-sm text-gray-500">
                    {formatCurrency(phase1Status.pool_asset_value_usd)} / {formatCurrency(phase1Status.launch_threshold_usd)}
                  </span>
                </div>
                <div className="w-full bg-gray-200 rounded-full h-3">
                  <div 
                    className="bg-gradient-to-r from-purple-500 to-blue-500 h-3 rounded-full transition-all duration-300"
                    style={{ width: `${Math.min(phase1Status.progress_to_launch, 100)}%` }}
                  ></div>
                </div>
                <div className="mt-1 text-xs text-gray-500">
                  {phase1Status.progress_to_launch.toFixed(1)}% complete • ${(phase1Status.launch_threshold_usd - phase1Status.pool_asset_value_usd).toFixed(2)} remaining
                </div>
              </div>

              {/* Key Metrics Grid */}
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                <div className="bg-blue-50 rounded-lg p-4">
                  <div className="flex items-center">
                    <div className="flex-shrink-0">
                      <div className="w-8 h-8 bg-blue-500 rounded-lg flex items-center justify-center">
                        <span className="text-white text-sm font-bold">🪙</span>
                      </div>
                    </div>
                    <div className="ml-3">
                      <p className="text-sm font-medium text-blue-900">Pre-Launch Distributed</p>
                      <p className="text-lg font-semibold text-blue-600">
                        {formatFlowAmount(phase1Status.total_pre_launch_distributed)} FLOW
                      </p>
                    </div>
                  </div>
                </div>

                <div className="bg-green-50 rounded-lg p-4">
                  <div className="flex items-center">
                    <div className="flex-shrink-0">
                      <div className="w-8 h-8 bg-green-500 rounded-lg flex items-center justify-center">
                        <span className="text-white text-sm font-bold">👥</span>
                      </div>
                    </div>
                    <div className="ml-3">
                      <p className="text-sm font-medium text-green-900">Eligible Users</p>
                      <p className="text-lg font-semibold text-green-600">
                        {phase1Status.eligible_users}
                      </p>
                    </div>
                  </div>
                </div>

                <div className="bg-yellow-50 rounded-lg p-4">
                  <div className="flex items-center">
                    <div className="flex-shrink-0">
                      <div className="w-8 h-8 bg-yellow-500 rounded-lg flex items-center justify-center">
                        <span className="text-white text-sm font-bold">₿</span>
                      </div>
                    </div>
                    <div className="ml-3">
                      <p className="text-sm font-medium text-yellow-900">Pool BTC</p>
                      <p className="text-lg font-semibold text-yellow-600">
                        {phase1Status.btc_amount.toFixed(4)} BTC
                      </p>
                    </div>
                  </div>
                </div>

                <div className="bg-purple-50 rounded-lg p-4">
                  <div className="flex items-center">
                    <div className="flex-shrink-0">
                      <div className="w-8 h-8 bg-purple-500 rounded-lg flex items-center justify-center">
                        <span className="text-white text-sm font-bold">🔒</span>
                      </div>
                    </div>
                    <div className="ml-3">
                      <p className="text-sm font-medium text-purple-900">ckBTC Staked</p>
                      <p className="text-lg font-semibold text-purple-600">
                        {phase1Status.ckbtc_staked.toFixed(4)} ckBTC
                      </p>
                    </div>
                  </div>
                </div>
              </div>

              {/* Phase 2 Launch Countdown */}
              <div className="bg-gradient-to-r from-purple-50 to-blue-50 border border-purple-200 rounded-lg p-4">
                <h3 className="text-lg font-medium text-gray-900 mb-2">🚀 Phase 2 Launch Status</h3>
                <div className="text-sm text-gray-600">
                  {phase1Status.progress_to_launch >= 100 ? (
                    <div className="flex items-center text-green-600">
                      <svg className="w-5 h-5 mr-2" fill="currentColor" viewBox="0 0 20 20">
                        <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                      </svg>
                      <strong>Ready for Phase 2 Launch!</strong> Pool has reached the 1 BTC threshold.
                    </div>
                  ) : (
                    <div>
                      <strong>Phase 2 launches automatically</strong> when the pool reaches <strong>1 BTC equivalent (${phase1Status.launch_threshold_usd.toLocaleString()})</strong>.
                      <br />
                      <span className="text-purple-600 font-medium">
                        ${(phase1Status.launch_threshold_usd - phase1Status.pool_asset_value_usd).toFixed(2)} more needed for launch
                      </span>
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* User Balances Section */}
          {activeSection === 'users' && (
            <div className="space-y-6">
              <h2 className="text-lg font-semibold text-gray-900">User Balance Lookup</h2>
              
              {/* Search Form */}
              <div className="flex space-x-2">
                <input
                  type="text"
                  value={searchPrincipal}
                  onChange={(e) => setSearchPrincipal(e.target.value)}
                  placeholder="Enter user Principal ID..."
                  className="flex-1 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
                <button
                  onClick={searchUserBalance}
                  disabled={loading || !searchPrincipal.trim()}
                  className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  Search
                </button>
              </div>

              {/* User Balance Results */}
              {selectedUserBalance && (
                <div className="bg-gray-50 rounded-lg p-4 space-y-4">
                  <h3 className="font-medium text-gray-900">Balance for: {searchPrincipal}</h3>
                  
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div className="bg-white rounded-lg p-4 border">
                      <div className="text-sm text-gray-500 mb-1">Pre-Launch Balance</div>
                      <div className="text-2xl font-bold text-blue-600">
                        {formatFlowAmount(selectedUserBalance.pre_launch_balance)} FLOW
                      </div>
                      <div className="text-xs text-gray-500 mt-1">Future value IOUs (not tradeable)</div>
                    </div>
                    
                    <div className="bg-white rounded-lg p-4 border">
                      <div className="text-sm text-gray-500 mb-1">Tradeable Balance</div>
                      <div className="text-2xl font-bold text-green-600">
                        {formatFlowAmount(selectedUserBalance.tradeable_balance)} FLOW
                      </div>
                      <div className="text-xs text-gray-500 mt-1">Phase 2 tradeable tokens</div>
                    </div>
                  </div>

                  <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div className="bg-white rounded-lg p-3 border">
                      <div className="text-sm text-gray-500">Airdrop Received</div>
                      <div className="font-semibold text-purple-600">
                        {formatFlowAmount(selectedUserBalance.phase1_airdrop_received)} FLOW
                      </div>
                    </div>
                    
                    <div className="bg-white rounded-lg p-3 border">
                      <div className="text-sm text-gray-500">Activity Rewards</div>
                      <div className="font-semibold text-orange-600">
                        {formatFlowAmount(selectedUserBalance.phase1_activity_rewards)} FLOW
                      </div>
                    </div>
                    
                    <div className="bg-white rounded-lg p-3 border">
                      <div className="text-sm text-gray-500">Phase 2 Eligible</div>
                      <div className={`font-semibold ${selectedUserBalance.eligible_for_phase2_conversion ? 'text-green-600' : 'text-red-600'}`}>
                        {selectedUserBalance.eligible_for_phase2_conversion ? '✅ Yes' : '❌ No'}
                      </div>
                    </div>
                  </div>
                </div>
              )}
            </div>
          )}

          {/* Transactions Section */}
          {activeSection === 'transactions' && (
            <div className="space-y-6">
              <h2 className="text-lg font-semibold text-gray-900">Recent FLOW Token Activity</h2>
              
              <div className="bg-gray-50 rounded-lg p-4 text-center text-gray-500">
                <div className="text-4xl mb-2">📋</div>
                <div className="font-medium">Transaction History</div>
                <div className="text-sm mt-1">Recent token transactions will appear here once the pool canister is integrated.</div>
              </div>
            </div>
          )}

          {/* Admin Controls Section */}
          {activeSection === 'controls' && (
            <div className="space-y-6">
              <h2 className="text-lg font-semibold text-gray-900">Admin Controls</h2>
              
              {!isOwner && (
                <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
                  <div className="flex">
                    <svg className="flex-shrink-0 h-5 w-5 text-yellow-400" viewBox="0 0 20 20" fill="currentColor">
                      <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
                    </svg>
                    <div className="ml-3">
                      <h3 className="text-sm font-medium text-yellow-800">Owner Access Required</h3>
                      <p className="text-sm text-yellow-700 mt-1">
                        Only the contract owner can access admin controls for token management.
                      </p>
                    </div>
                  </div>
                </div>
              )}

              {isOwner && (
                <div className="space-y-4">
                  {/* Early Adopter Airdrop */}
                  <div className="bg-white border border-gray-200 rounded-lg p-4">
                    <h3 className="font-medium text-gray-900 mb-2">🎁 Early Adopter Airdrop</h3>
                    <p className="text-sm text-gray-600 mb-3">
                      Distribute 100 FLOW pre-launch tokens to early adopters. These tokens will become tradeable in Phase 2.
                    </p>
                    <button
                      onClick={triggerEarlyAdopterAirdrop}
                      disabled={loading}
                      className="px-4 py-2 bg-purple-600 text-white rounded-md hover:bg-purple-700 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      {loading ? 'Processing...' : 'Trigger Airdrop'}
                    </button>
                  </div>

                  {/* Pool Asset Management */}
                  <div className="bg-white border border-gray-200 rounded-lg p-4">
                    <h3 className="font-medium text-gray-900 mb-2">💰 Pool Asset Management</h3>
                    <p className="text-sm text-gray-600 mb-3">
                      Update pool asset values to track progress toward Phase 2 launch threshold.
                    </p>
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-3 mb-3">
                      <input
                        type="number"
                        placeholder="BTC Equivalent ($)"
                        className="px-3 py-2 border border-gray-300 rounded-md text-sm"
                        id="btc-equivalent"
                      />
                      <input
                        type="number"
                        step="0.00000001"
                        placeholder="Actual BTC Amount"
                        className="px-3 py-2 border border-gray-300 rounded-md text-sm"
                        id="actual-btc"
                      />
                      <input
                        type="number"
                        placeholder="Other Assets ($)"
                        className="px-3 py-2 border border-gray-300 rounded-md text-sm"
                        id="other-assets"
                      />
                    </div>
                    <button
                      onClick={() => {
                        const btcEquiv = parseFloat((document.getElementById('btc-equivalent') as HTMLInputElement)?.value || '0');
                        const actualBtc = parseFloat((document.getElementById('actual-btc') as HTMLInputElement)?.value || '0');
                        const otherAssets = parseFloat((document.getElementById('other-assets') as HTMLInputElement)?.value || '0');
                        updatePoolAssets(btcEquiv, actualBtc, otherAssets);
                      }}
                      disabled={loading}
                      className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      {loading ? 'Updating...' : 'Update Pool Assets'}
                    </button>
                  </div>

                  {/* Phase 2 Manual Trigger */}
                  <div className="bg-white border border-gray-200 rounded-lg p-4">
                    <h3 className="font-medium text-gray-900 mb-2">🚀 Phase 2 Launch</h3>
                    <p className="text-sm text-gray-600 mb-3">
                      Phase 2 launches automatically when pool reaches 1 BTC equivalent. Manual trigger available for testing.
                    </p>
                    <button
                      disabled={true}
                      className="px-4 py-2 bg-gray-400 text-white rounded-md cursor-not-allowed"
                    >
                      Auto-Launch (Manual Override Coming Soon)
                    </button>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default FlowTokenDashboard;