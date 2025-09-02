import React, { useState, useEffect } from 'react';
import { AdminPoolService } from '../services/adminPoolService';

interface EarningsConfig {
  principal: string;
  allocation: { type: string; amount: number };
  role: string;
  isActive: boolean;
  vestingCliffMonths: number;
  vestingPeriodMonths: number;
  joinedTimestamp: number;
  lastModifiedBy: string;
  lastModifiedTime: number;
}

interface EarningsManagementProps {
  isOwner: boolean;
  currentPrincipal: string;
}

const EarningsManagement: React.FC<EarningsManagementProps> = ({ isOwner, currentPrincipal }) => {
  const [earningsConfigs, setEarningsConfigs] = useState<EarningsConfig[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  // Form states
  const [newMemberPrincipal, setNewMemberPrincipal] = useState('');
  const [newAllocationType, setNewAllocationType] = useState<'percentage' | 'fixedMonthly' | 'perTransaction'>('percentage');
  const [newAmount, setNewAmount] = useState<number>(0);

  useEffect(() => {
    loadEarningsConfig();
  }, []);

  const loadEarningsConfig = async () => {
    try {
      setIsLoading(true);
      const configs = await AdminPoolService.getAllEarningsConfig();
      setEarningsConfigs(configs);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load earnings configuration');
    } finally {
      setIsLoading(false);
    }
  };

  const handleSetMemberEarnings = async () => {
    if (!newMemberPrincipal.trim()) {
      setError('Please enter a member principal');
      return;
    }

    if (newAmount <= 0) {
      setError('Amount must be greater than 0');
      return;
    }

    try {
      setIsLoading(true);
      const result = await AdminPoolService.setMemberEarnings(
        newMemberPrincipal.trim(),
        newAllocationType,
        newAmount
      );
      
      setSuccess(result);
      setError(null);
      setNewMemberPrincipal('');
      setNewAmount(0);
      await loadEarningsConfig();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to set member earnings');
    } finally {
      setIsLoading(false);
    }
  };

  const handleToggleActive = async (memberPrincipal: string, currentActive: boolean) => {
    try {
      setIsLoading(true);
      const result = await AdminPoolService.activateMemberEarnings(
        memberPrincipal,
        !currentActive
      );
      
      setSuccess(result);
      setError(null);
      await loadEarningsConfig();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to toggle member status');
    } finally {
      setIsLoading(false);
    }
  };

  const formatAmount = (allocation: { type: string; amount: number }) => {
    switch (allocation.type) {
      case 'percentage':
        return `${allocation.amount}%`;
      case 'fixedMonthly':
        return `$${allocation.amount.toLocaleString()}/month`;
      case 'perTransaction':
        return `$${allocation.amount}/tx`;
      default:
        return `${allocation.amount}`;
    }
  };

  const getTotalPercentage = () => {
    return earningsConfigs
      .filter(config => config.allocation.type === 'percentage' && config.isActive)
      .reduce((sum, config) => sum + config.allocation.amount, 0);
  };

  const clearMessages = () => {
    setError(null);
    setSuccess(null);
  };

  return (
    <div className="space-y-6">
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-xl font-semibold text-gray-900 mb-4">Team Earnings Management</h2>
        
        {error && (
          <div className="mb-4 p-4 bg-red-50 border border-red-200 text-red-700 rounded-md">
            {error}
            <button onClick={clearMessages} className="ml-2 text-red-500 hover:text-red-700">×</button>
          </div>
        )}
        
        {success && (
          <div className="mb-4 p-4 bg-green-50 border border-green-200 text-green-700 rounded-md">
            {success}
            <button onClick={clearMessages} className="ml-2 text-green-500 hover:text-green-700">×</button>
          </div>
        )}

        {/* Current Earnings Overview */}
        <div className="mb-6 p-4 bg-blue-50 border border-blue-200 rounded-md">
          <h3 className="text-lg font-medium text-blue-900 mb-2">Earnings Overview</h3>
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <span className="font-medium">Total Percentage Allocated:</span> {getTotalPercentage()}%
            </div>
            <div>
              <span className="font-medium">Active Members:</span> {earningsConfigs.filter(c => c.isActive).length}
            </div>
          </div>
          {getTotalPercentage() > 100 && (
            <div className="mt-2 text-red-600 text-sm font-medium">
              ⚠️ Warning: Total percentage exceeds 100%
            </div>
          )}
        </div>

        {/* Add New Member Earnings */}
        {isOwner && (
          <div className="mb-6 p-4 border border-gray-200 rounded-md">
            <h3 className="text-lg font-medium text-gray-900 mb-4">Set Member Earnings</h3>
            
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Member Principal
                </label>
                <input
                  type="text"
                  value={newMemberPrincipal}
                  onChange={(e) => setNewMemberPrincipal(e.target.value)}
                  placeholder="Enter principal ID..."
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Allocation Type
                </label>
                <select
                  value={newAllocationType}
                  onChange={(e) => setNewAllocationType(e.target.value as any)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <option value="percentage">Percentage (%)</option>
                  <option value="fixedMonthly">Fixed Monthly ($)</option>
                  <option value="perTransaction">Per Transaction ($)</option>
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Amount
                </label>
                <input
                  type="number"
                  value={newAmount}
                  onChange={(e) => setNewAmount(Number(e.target.value))}
                  min="0"
                  max={newAllocationType === 'percentage' ? 100 : newAllocationType === 'fixedMonthly' ? 50000 : 1000}
                  step={newAllocationType === 'percentage' ? 0.1 : 1}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
            </div>

            <button
              onClick={handleSetMemberEarnings}
              disabled={isLoading || !newMemberPrincipal.trim() || newAmount <= 0}
              className="mt-4 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isLoading ? 'Setting...' : 'Set Member Earnings'}
            </button>
          </div>
        )}

        {/* Current Team Members */}
        <div>
          <h3 className="text-lg font-medium text-gray-900 mb-4">Current Team Earnings</h3>
          
          {isLoading && earningsConfigs.length === 0 ? (
            <div className="text-gray-500">Loading earnings configuration...</div>
          ) : earningsConfigs.length === 0 ? (
            <div className="text-gray-500">No team members configured yet</div>
          ) : (
            <div className="overflow-x-auto">
              <table className="min-w-full bg-white border border-gray-200">
                <thead className="bg-gray-50">
                  <tr>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Principal
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Role
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Earnings
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Status
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Vesting
                    </th>
                    {isOwner && (
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Actions
                      </th>
                    )}
                  </tr>
                </thead>
                <tbody className="bg-white divide-y divide-gray-200">
                  {earningsConfigs.map((config) => (
                    <tr key={config.principal} className={config.isActive ? '' : 'bg-gray-50 opacity-60'}>
                      <td className="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-900">
                        {config.principal.slice(0, 8)}...{config.principal.slice(-8)}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                        <span className={`px-2 py-1 text-xs rounded-full ${
                          config.role === 'Owner' ? 'bg-purple-100 text-purple-800' :
                          config.role === 'SeniorManager' ? 'bg-blue-100 text-blue-800' :
                          config.role === 'OperationsManager' ? 'bg-green-100 text-green-800' :
                          config.role === 'TechManager' ? 'bg-yellow-100 text-yellow-800' :
                          'bg-gray-100 text-gray-800'
                        }`}>
                          {config.role}
                        </span>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                        <span className="font-medium">{formatAmount(config.allocation)}</span>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                        <span className={`px-2 py-1 text-xs rounded-full ${
                          config.isActive ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
                        }`}>
                          {config.isActive ? 'Active' : 'Inactive'}
                        </span>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                        {config.vestingCliffMonths > 0 && (
                          <div className="text-xs text-gray-500">
                            Cliff: {config.vestingCliffMonths}mo
                          </div>
                        )}
                        <div className="text-xs text-gray-500">
                          Period: {config.vestingPeriodMonths}mo
                        </div>
                      </td>
                      {isOwner && (
                        <td className="px-6 py-4 whitespace-nowrap text-sm font-medium">
                          <button
                            onClick={() => handleToggleActive(config.principal, config.isActive)}
                            disabled={isLoading}
                            className={`px-3 py-1 text-xs rounded-md ${
                              config.isActive 
                                ? 'bg-red-100 text-red-700 hover:bg-red-200' 
                                : 'bg-green-100 text-green-700 hover:bg-green-200'
                            } disabled:opacity-50`}
                          >
                            {config.isActive ? 'Deactivate' : 'Activate'}
                          </button>
                        </td>
                      )}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>

        {/* Quick Examples */}
        <div className="mt-6 p-4 bg-gray-50 border border-gray-200 rounded-md">
          <h4 className="text-sm font-medium text-gray-900 mb-2">Allocation Examples</h4>
          <div className="text-xs text-gray-600 space-y-1">
            <div><strong>Percentage:</strong> 25.0 = 25% of monthly profits</div>
            <div><strong>Fixed Monthly:</strong> 5000.0 = $5,000 guaranteed per month</div>
            <div><strong>Per Transaction:</strong> 10.0 = $10 per platform transaction</div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default EarningsManagement;