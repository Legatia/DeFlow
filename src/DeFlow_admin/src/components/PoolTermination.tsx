import React, { useState, useEffect } from 'react';
import { AdminPoolService } from '../services/adminPoolService';

interface PoolTerminationRequest {
  id: string;
  initiated_by: string;
  reason: string;
  asset_distribution_plan: AssetDistribution[];
  owner_approval: TerminationApproval | null;
  cofounder_approval: TerminationApproval | null;
  created_at: bigint;
  expires_at: bigint;
  emergency_termination: boolean;
}

interface AssetDistribution {
  chain: string;
  asset: string;
  total_amount: number;
  destination_address: string;
  estimated_usd_value: number;
  status: string;
  tx_hash: string | null;
  executed_at: bigint | null;
}

interface TerminationApproval {
  approver: string;
  approved_at: bigint;
  signature_confirmation: string;
  notes: string | null;
}

interface TerminationSummary {
  total_assets_distributed: number;
  chains_processed: string[];
  successful_distributions: number;
  failed_distributions: number;
  termination_initiated_at: bigint;
  termination_completed_at: bigint | null;
  final_state_hash: string;
}

const PoolTermination: React.FC = () => {
  const [activeRequest, setActiveRequest] = useState<PoolTerminationRequest | null>(null);
  const [cofounder, setCofounder] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showInitiateModal, setShowInitiateModal] = useState(false);
  const [showApprovalModal, setShowApprovalModal] = useState(false);
  const [showConfirmExecution, setShowConfirmExecution] = useState(false);

  useEffect(() => {
    loadTerminationData();
  }, []);

  const loadTerminationData = async () => {
    try {
      setLoading(true);
      setError(null);
      
      // Load active termination request
      const request = await AdminPoolService.getActiveTerminationRequest();
      setActiveRequest(request);
      
      // Load cofounder info
      const cofounderPrincipal = await AdminPoolService.getCofounder();
      setCofounder(cofounderPrincipal);
      
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load termination data');
    } finally {
      setLoading(false);
    }
  };

  const formatTimeRemaining = (expiresAt: bigint) => {
    const now = BigInt(Date.now() * 1_000_000); // Convert to nanoseconds
    const remaining = Number(expiresAt - now) / 1_000_000_000; // Convert to seconds
    
    if (remaining <= 0) return 'EXPIRED';
    
    const hours = Math.floor(remaining / 3600);
    const minutes = Math.floor((remaining % 3600) / 60);
    return `${hours}h ${minutes}m remaining`;
  };

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 0,
      maximumFractionDigits: 0
    }).format(amount);
  };

  if (loading) {
    return (
      <div className="text-center py-12">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-red-500 mx-auto"></div>
        <p className="text-gray-400 mt-4">Loading termination data...</p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-red-900/20 border border-red-500 rounded-lg p-6">
        <div className="flex justify-between items-start">
          <div>
            <h2 className="text-2xl font-bold text-red-400">🔥 Pool Termination Control</h2>
            <p className="text-red-300 mt-1">
              ⚠️ CRITICAL: This will permanently shut down the pool and distribute all assets
            </p>
          </div>
          <div className="flex items-center space-x-2">
            <span className="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-red-100 text-red-800">
              {activeRequest ? 'TERMINATION IN PROGRESS' : 'ACTIVE POOL'}
            </span>
          </div>
        </div>
      </div>

      {/* Cofounder Status */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h3 className="text-lg font-medium text-white mb-4">Multi-Signature Setup</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="bg-blue-900/30 p-4 rounded-lg border border-blue-700">
            <h4 className="text-sm font-medium text-blue-300">Owner (You)</h4>
            <p className="text-sm text-blue-200 mt-1">Can initiate, approve, and execute termination</p>
            <span className="inline-flex items-center mt-2 px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800">
              ✓ Ready
            </span>
          </div>
          
          <div className="bg-purple-900/30 p-4 rounded-lg border border-purple-700">
            <h4 className="text-sm font-medium text-purple-300">Cofounder</h4>
            {cofounder ? (
              <>
                <p className="text-xs text-purple-200 mt-1 font-mono">{cofounder}</p>
                <span className="inline-flex items-center mt-2 px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800">
                  ✓ Set
                </span>
              </>
            ) : (
              <>
                <p className="text-sm text-purple-200 mt-1">Not set - required for non-emergency termination</p>
                <button className="mt-2 text-xs bg-purple-600 text-white px-3 py-1 rounded hover:bg-purple-700">
                  Set Cofounder
                </button>
              </>
            )}
          </div>
        </div>
      </div>

      {/* Active Termination Request */}
      {activeRequest ? (
        <ActiveTerminationRequest 
          request={activeRequest}
          onApprove={() => setShowApprovalModal(true)}
          onExecute={() => setShowConfirmExecution(true)}
          onCancel={async () => {
            if (confirm('Are you sure you want to cancel this termination request?')) {
              try {
                await AdminPoolService.cancelPoolTermination(activeRequest.id, 'Cancelled by owner');
                await loadTerminationData();
              } catch (err) {
                setError(err instanceof Error ? err.message : 'Failed to cancel termination');
              }
            }
          }}
          formatTimeRemaining={formatTimeRemaining}
          formatCurrency={formatCurrency}
        />
      ) : (
        <NoActiveTermination onInitiate={() => setShowInitiateModal(true)} />
      )}

      {/* Error Display */}
      {error && (
        <div className="bg-red-900/20 border border-red-500 rounded-lg p-4">
          <h4 className="text-red-400 font-medium">Error</h4>
          <p className="text-red-300 mt-1">{error}</p>
          <button 
            onClick={() => setError(null)}
            className="mt-3 text-xs bg-red-600 text-white px-3 py-1 rounded hover:bg-red-700"
          >
            Dismiss
          </button>
        </div>
      )}

      {/* Modals */}
      {showInitiateModal && (
        <InitiateTerminationModal 
          onClose={() => setShowInitiateModal(false)}
          onSubmit={async (data) => {
            try {
              await AdminPoolService.initiatePoolTermination(
                data.reason, 
                data.distributions, 
                data.emergency
              );
              setShowInitiateModal(false);
              await loadTerminationData();
            } catch (err) {
              setError(err instanceof Error ? err.message : 'Failed to initiate termination');
            }
          }}
        />
      )}

      {showApprovalModal && activeRequest && (
        <ApprovalModal
          terminationId={activeRequest.id}
          onClose={() => setShowApprovalModal(false)}
          onSubmit={async (confirmationPhrase, notes) => {
            try {
              await AdminPoolService.approvePoolTermination(activeRequest.id, confirmationPhrase, notes);
              setShowApprovalModal(false);
              await loadTerminationData();
            } catch (err) {
              setError(err instanceof Error ? err.message : 'Failed to approve termination');
            }
          }}
        />
      )}

      {showConfirmExecution && activeRequest && (
        <ExecutionConfirmModal
          terminationId={activeRequest.id}
          onClose={() => setShowConfirmExecution(false)}
          onConfirm={async () => {
            try {
              const result = await AdminPoolService.executePoolTermination(activeRequest.id);
              setShowConfirmExecution(false);
              alert(`Pool terminated successfully! ${result.successful_distributions} distributions completed.`);
              await loadTerminationData();
            } catch (err) {
              setError(err instanceof Error ? err.message : 'Failed to execute termination');
            }
          }}
        />
      )}
    </div>
  );
};

// Sub-components
const ActiveTerminationRequest: React.FC<{
  request: PoolTerminationRequest;
  onApprove: () => void;
  onExecute: () => void;
  onCancel: () => void;
  formatTimeRemaining: (expires: bigint) => string;
  formatCurrency: (amount: number) => string;
}> = ({ request, onApprove, onExecute, onCancel, formatTimeRemaining, formatCurrency }) => {
  const totalUSD = request.asset_distribution_plan.reduce((sum, dist) => sum + dist.estimated_usd_value, 0);
  const hasOwnerApproval = request.owner_approval !== null;
  const hasCofounderApproval = request.cofounder_approval !== null;
  const canExecute = request.emergency_termination ? 
    (hasOwnerApproval || hasCofounderApproval) : 
    (hasOwnerApproval && hasCofounderApproval);

  return (
    <div className="bg-red-900/30 border border-red-500 rounded-lg p-6">
      <div className="flex justify-between items-start mb-6">
        <div>
          <h3 className="text-xl font-bold text-red-400">Active Termination Request</h3>
          <p className="text-red-300 mt-1">ID: {request.id}</p>
          <p className="text-red-300 text-sm mt-1">
            {request.emergency_termination ? '🚨 EMERGENCY TERMINATION' : '📋 Standard Termination'}
          </p>
        </div>
        <div className="text-right">
          <p className="text-red-400 font-mono">{formatTimeRemaining(request.expires_at)}</p>
          <p className="text-red-300 text-sm mt-1">Total: {formatCurrency(totalUSD)}</p>
        </div>
      </div>

      {/* Approval Status */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
        <div className={`p-4 rounded-lg border ${hasOwnerApproval ? 'bg-green-900/30 border-green-500' : 'bg-gray-700 border-gray-600'}`}>
          <h4 className="text-sm font-medium text-white">Owner Approval</h4>
          {hasOwnerApproval ? (
            <p className="text-green-400 text-sm mt-1">✓ Approved</p>
          ) : (
            <p className="text-gray-400 text-sm mt-1">⏳ Pending</p>
          )}
        </div>

        <div className={`p-4 rounded-lg border ${hasCofounderApproval ? 'bg-green-900/30 border-green-500' : 'bg-gray-700 border-gray-600'}`}>
          <h4 className="text-sm font-medium text-white">Cofounder Approval</h4>
          {hasCofounderApproval ? (
            <p className="text-green-400 text-sm mt-1">✓ Approved</p>
          ) : (
            <p className="text-gray-400 text-sm mt-1">⏳ Pending</p>
          )}
        </div>
      </div>

      {/* Asset Distribution Plan */}
      <div className="mb-6">
        <h4 className="text-lg font-medium text-white mb-3">Asset Distribution Plan</h4>
        <div className="space-y-2">
          {request.asset_distribution_plan.map((dist, index) => (
            <div key={index} className="bg-gray-800 p-3 rounded-lg flex justify-between items-center">
              <div>
                <span className="text-white font-medium">{dist.asset}</span>
                <span className="text-gray-400 ml-2">on {dist.chain}</span>
              </div>
              <div className="text-right">
                <p className="text-white">{dist.total_amount.toFixed(6)}</p>
                <p className="text-gray-400 text-sm">{formatCurrency(dist.estimated_usd_value)}</p>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Action Buttons */}
      <div className="flex space-x-4">
        {!hasOwnerApproval && (
          <button
            onClick={onApprove}
            className="bg-yellow-600 text-white px-6 py-2 rounded-lg hover:bg-yellow-700"
          >
            Approve Termination
          </button>
        )}
        
        {canExecute && (
          <button
            onClick={onExecute}
            className="bg-red-600 text-white px-6 py-2 rounded-lg hover:bg-red-700 font-bold"
          >
            🔥 EXECUTE TERMINATION
          </button>
        )}
        
        <button
          onClick={onCancel}
          className="bg-gray-600 text-white px-6 py-2 rounded-lg hover:bg-gray-700"
        >
          Cancel Request
        </button>
      </div>
    </div>
  );
};

const NoActiveTermination: React.FC<{ onInitiate: () => void }> = ({ onInitiate }) => (
  <div className="bg-gray-800 rounded-lg p-6 text-center">
    <h3 className="text-xl font-medium text-white mb-4">No Active Termination Request</h3>
    <p className="text-gray-400 mb-6">
      Pool is currently active. Only initiate termination in case of emergency or planned shutdown.
    </p>
    <button
      onClick={onInitiate}
      className="bg-red-600 text-white px-6 py-3 rounded-lg hover:bg-red-700 font-bold"
    >
      🚨 Initiate Pool Termination
    </button>
  </div>
);

// Modal Components
const InitiateTerminationModal: React.FC<{
  onClose: () => void;
  onSubmit: (data: { reason: string; distributions: Array<[string, string, string]>; emergency: boolean }) => void;
}> = ({ onClose, onSubmit }) => {
  const [reason, setReason] = useState('');
  const [emergency, setEmergency] = useState(false);
  const [distributions, setDistributions] = useState<Array<{ chain: string; asset: string; address: string }>>([
    { chain: 'ethereum', asset: 'USDC', address: '' },
    { chain: 'polygon', asset: 'USDT', address: '' }
  ]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const distributionTuples: Array<[string, string, string]> = distributions
      .filter(d => d.chain && d.asset && d.address)
      .map(d => [d.chain, d.asset, d.address]);
    
    onSubmit({ reason, distributions: distributionTuples, emergency });
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg p-6 max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto">
        <div className="flex justify-between items-center mb-6">
          <h3 className="text-xl font-bold text-red-400">🔥 Initiate Pool Termination</h3>
          <button onClick={onClose} className="text-gray-400 hover:text-white">✕</button>
        </div>

        <form onSubmit={handleSubmit} className="space-y-6">
          <div>
            <label className="block text-sm font-medium text-white mb-2">Termination Reason</label>
            <textarea
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              required
              minLength={10}
              className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white"
              rows={3}
              placeholder="Explain why the pool needs to be terminated..."
            />
          </div>

          <div>
            <div className="flex items-center space-x-2 mb-4">
              <input
                type="checkbox"
                id="emergency"
                checked={emergency}
                onChange={(e) => setEmergency(e.target.checked)}
                className="rounded"
              />
              <label htmlFor="emergency" className="text-sm text-white">
                🚨 Emergency Termination (requires only single approval)
              </label>
            </div>
          </div>

          <div>
            <h4 className="text-lg font-medium text-white mb-3">Asset Distribution Addresses</h4>
            {distributions.map((dist, index) => (
              <div key={index} className="grid grid-cols-3 gap-3 mb-3">
                <input
                  type="text"
                  placeholder="Chain (e.g. ethereum)"
                  value={dist.chain}
                  onChange={(e) => {
                    const newDist = [...distributions];
                    newDist[index].chain = e.target.value;
                    setDistributions(newDist);
                  }}
                  className="bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white text-sm"
                />
                <input
                  type="text"
                  placeholder="Asset (e.g. USDC)"
                  value={dist.asset}
                  onChange={(e) => {
                    const newDist = [...distributions];
                    newDist[index].asset = e.target.value;
                    setDistributions(newDist);
                  }}
                  className="bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white text-sm"
                />
                <input
                  type="text"
                  placeholder="Destination address"
                  value={dist.address}
                  onChange={(e) => {
                    const newDist = [...distributions];
                    newDist[index].address = e.target.value;
                    setDistributions(newDist);
                  }}
                  className="bg-gray-700 border border-gray-600 rounded px-3 py-2 text-white text-sm"
                />
              </div>
            ))}
            <button
              type="button"
              onClick={() => setDistributions([...distributions, { chain: '', asset: '', address: '' }])}
              className="text-blue-400 text-sm hover:text-blue-300"
            >
              + Add Distribution
            </button>
          </div>

          <div className="flex space-x-4">
            <button
              type="submit"
              className="bg-red-600 text-white px-6 py-2 rounded-lg hover:bg-red-700"
            >
              Initiate Termination
            </button>
            <button
              type="button"
              onClick={onClose}
              className="bg-gray-600 text-white px-6 py-2 rounded-lg hover:bg-gray-700"
            >
              Cancel
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

const ApprovalModal: React.FC<{
  terminationId: string;
  onClose: () => void;
  onSubmit: (confirmationPhrase: string, notes?: string) => void;
}> = ({ terminationId, onClose, onSubmit }) => {
  const [confirmationPhrase, setConfirmationPhrase] = useState('');
  const [notes, setNotes] = useState('');
  const expectedPhrase = `TERMINATE_POOL_${terminationId}`;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSubmit(confirmationPhrase, notes || undefined);
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg p-6 max-w-md w-full mx-4">
        <div className="flex justify-between items-center mb-6">
          <h3 className="text-xl font-bold text-yellow-400">⚠️ Approve Termination</h3>
          <button onClick={onClose} className="text-gray-400 hover:text-white">✕</button>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-white mb-2">
              Confirmation Phrase
            </label>
            <p className="text-xs text-gray-400 mb-2">
              Type exactly: <span className="font-mono text-yellow-400">{expectedPhrase}</span>
            </p>
            <input
              type="text"
              value={confirmationPhrase}
              onChange={(e) => setConfirmationPhrase(e.target.value)}
              className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white font-mono"
              placeholder={expectedPhrase}
              required
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-white mb-2">Approval Notes (Optional)</label>
            <textarea
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
              className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-white"
              rows={2}
              placeholder="Add any additional notes..."
            />
          </div>

          <div className="flex space-x-4">
            <button
              type="submit"
              disabled={confirmationPhrase !== expectedPhrase}
              className="bg-yellow-600 text-white px-6 py-2 rounded-lg hover:bg-yellow-700 disabled:bg-gray-600 disabled:cursor-not-allowed"
            >
              Approve Termination
            </button>
            <button
              type="button"
              onClick={onClose}
              className="bg-gray-600 text-white px-6 py-2 rounded-lg hover:bg-gray-700"
            >
              Cancel
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

const ExecutionConfirmModal: React.FC<{
  terminationId: string;
  onClose: () => void;
  onConfirm: () => void;
}> = ({ terminationId, onClose, onConfirm }) => (
  <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div className="bg-red-900/50 border border-red-500 rounded-lg p-6 max-w-md w-full mx-4">
      <h3 className="text-xl font-bold text-red-400 mb-4">🔥 FINAL CONFIRMATION</h3>
      <div className="space-y-4 text-red-300">
        <p>You are about to <strong>PERMANENTLY TERMINATE</strong> the DeFlow pool.</p>
        <p>This action will:</p>
        <ul className="list-disc list-inside space-y-1 text-sm">
          <li>Distribute all pool assets to specified addresses</li>
          <li>Shut down the pool permanently</li>
          <li>Make the pool non-recoverable</li>
        </ul>
        <p className="font-bold">This action CANNOT be undone!</p>
      </div>

      <div className="flex space-x-4 mt-6">
        <button
          onClick={onConfirm}
          className="bg-red-600 text-white px-6 py-2 rounded-lg hover:bg-red-700 font-bold"
        >
          🔥 EXECUTE TERMINATION
        </button>
        <button
          onClick={onClose}
          className="bg-gray-600 text-white px-6 py-2 rounded-lg hover:bg-gray-700"
        >
          Cancel
        </button>
      </div>
    </div>
  </div>
);

export default PoolTermination;