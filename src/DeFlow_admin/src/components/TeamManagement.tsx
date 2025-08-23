import React, { useState, useEffect } from 'react';
import { AdminAuthService } from '../services/adminAuthService';

interface TeamMember {
  principal: string;
  addedBy: string;
  addedAt: number;
  role: 'admin' | 'member';
  status: 'active' | 'inactive';
  earningPercentage: number;
}

interface PendingApproval {
  id: string;
  candidatePrincipal: string;
  requestedBy: string;
  requestedAt: number;
  role: 'admin' | 'member';
  status: 'pending' | 'approved' | 'rejected';
}

interface TeamManagementProps {
  isOwner: boolean;
  currentPrincipal: string;
}

const TeamManagement: React.FC<TeamManagementProps> = ({ isOwner, currentPrincipal }) => {
  const [teamMembers, setTeamMembers] = useState<TeamMember[]>([]);
  const [pendingApprovals, setPendingApprovals] = useState<PendingApproval[]>([]);
  const [newMemberPrincipal, setNewMemberPrincipal] = useState('');
  const [newMemberRole, setNewMemberRole] = useState<'admin' | 'member'>('member');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [editingEarnings, setEditingEarnings] = useState<{[key: string]: number | undefined}>({});

  useEffect(() => {
    loadData();
  }, []);

  const loadData = () => {
    setTeamMembers(AdminAuthService.getTeamMembers());
    setPendingApprovals(AdminAuthService.getPendingApprovals().filter(p => p.status === 'pending'));
  };

  const handleAddTeamMember = async () => {
    if (!newMemberPrincipal.trim()) {
      setError('Please enter a principal');
      return;
    }

    try {
      setIsLoading(true);
      setError(null);
      
      await AdminAuthService.requestTeamMemberAddition(newMemberPrincipal.trim(), newMemberRole);
      
      setSuccess(isOwner ? 'Team member request created (auto-approved as owner)' : 'Team member request submitted for owner approval');
      setNewMemberPrincipal('');
      loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to add team member');
    } finally {
      setIsLoading(false);
    }
  };

  const handleApproveRequest = async (approvalId: string) => {
    try {
      setIsLoading(true);
      await AdminAuthService.approveTeamMember(approvalId);
      setSuccess('Team member approved successfully');
      loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to approve team member');
    } finally {
      setIsLoading(false);
    }
  };

  const handleRejectRequest = async (approvalId: string) => {
    try {
      setIsLoading(true);
      await AdminAuthService.rejectTeamMember(approvalId);
      setSuccess('Team member request rejected');
      loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to reject team member');
    } finally {
      setIsLoading(false);
    }
  };

  const handleRemoveTeamMember = async (principal: string) => {
    if (!confirm('Are you sure you want to remove this team member?')) return;

    try {
      setIsLoading(true);
      await AdminAuthService.removeTeamMember(principal);
      setSuccess('Team member removed successfully');
      loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to remove team member');
    } finally {
      setIsLoading(false);
    }
  };

  const handleUpdateEarning = async (principal: string, percentage: number) => {
    try {
      setIsLoading(true);
      await AdminAuthService.updateTeamMemberEarning(principal, percentage);
      setSuccess(`Earning percentage updated to ${percentage}%`);
      const newState = { ...editingEarnings };
      delete newState[principal];
      setEditingEarnings(newState);
      loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update earning percentage');
    } finally {
      setIsLoading(false);
    }
  };

  const getTotalEarningPercentage = () => {
    return AdminAuthService.getTotalEarningPercentage();
  };

  const startEditingEarning = (principal: string, currentPercentage: number) => {
    setEditingEarnings(prev => ({ ...prev, [principal]: currentPercentage }));
  };

  const cancelEditingEarning = (principal: string) => {
    const newState = { ...editingEarnings };
    delete newState[principal];
    setEditingEarnings(newState);
  };

  const formatPrincipal = (principal: string) => {
    if (principal.length <= 20) return principal;
    return `${principal.slice(0, 10)}...${principal.slice(-10)}`;
  };

  const formatDate = (timestamp: number) => {
    return new Date(timestamp).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  };

  // Clear messages after 3 seconds
  useEffect(() => {
    if (error || success) {
      const timer = setTimeout(() => {
        setError(null);
        setSuccess(null);
      }, 3000);
      return () => clearTimeout(timer);
    }
  }, [error, success]);

  return (
    <div className="space-y-8">
      <div className="bg-white rounded-lg shadow-lg">
        <div className="px-6 py-4 border-b border-gray-200">
          <h2 className="text-xl font-semibold text-gray-900">Team Management</h2>
          <p className="text-gray-600 text-sm mt-1">
            Manage admin team members and approval requests
          </p>
        </div>

        <div className="p-6 space-y-6">
          {/* Status Messages */}
          {error && (
            <div className="bg-red-50 border border-red-300 text-red-700 px-4 py-3 rounded-lg">
              {error}
            </div>
          )}
          
          {success && (
            <div className="bg-green-50 border border-green-300 text-green-700 px-4 py-3 rounded-lg">
              {success}
            </div>
          )}

          {/* Add New Team Member */}
          <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
            <h3 className="font-semibold text-blue-900 mb-4">Add Team Member</h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Principal ID
                </label>
                <input
                  type="text"
                  value={newMemberPrincipal}
                  onChange={(e) => setNewMemberPrincipal(e.target.value)}
                  placeholder="Enter team member's principal ID"
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono text-sm"
                />
              </div>
              
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Role
                </label>
                <select
                  value={newMemberRole}
                  onChange={(e) => setNewMemberRole(e.target.value as 'admin' | 'member')}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                >
                  <option value="member">Member</option>
                  <option value="admin">Admin</option>
                </select>
              </div>

              <button
                onClick={handleAddTeamMember}
                disabled={isLoading || !newMemberPrincipal.trim()}
                className="w-full px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
              >
                {isLoading ? 'Processing...' : (isOwner ? 'Add Team Member (Auto-Approve)' : 'Request Team Member Addition')}
              </button>

              <p className="text-xs text-gray-600">
                {isOwner 
                  ? 'As the owner, team members will be added immediately.'
                  : 'Your request will need approval from the project owner.'
                }
              </p>
            </div>
          </div>

          {/* Pending Approvals (Owner Only) */}
          {isOwner && pendingApprovals.length > 0 && (
            <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
              <h3 className="font-semibold text-yellow-900 mb-4">
                Pending Approvals ({pendingApprovals.length})
              </h3>
              <div className="space-y-3">
                {pendingApprovals.map((approval) => (
                  <div key={approval.id} className="flex items-center justify-between bg-white p-3 rounded border">
                    <div className="flex-1">
                      <p className="font-mono text-sm text-gray-800">
                        {formatPrincipal(approval.candidatePrincipal)}
                      </p>
                      <p className="text-xs text-gray-600">
                        Role: {approval.role} • Requested by: {formatPrincipal(approval.requestedBy)} • {formatDate(approval.requestedAt)}
                      </p>
                    </div>
                    <div className="flex space-x-2">
                      <button
                        onClick={() => handleApproveRequest(approval.id)}
                        disabled={isLoading}
                        className="px-3 py-1 bg-green-600 text-white text-sm rounded hover:bg-green-700 disabled:bg-gray-400"
                      >
                        Approve
                      </button>
                      <button
                        onClick={() => handleRejectRequest(approval.id)}
                        disabled={isLoading}
                        className="px-3 py-1 bg-red-600 text-white text-sm rounded hover:bg-red-700 disabled:bg-gray-400"
                      >
                        Reject
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Earning Distribution Summary (Owner Only) */}
          {isOwner && teamMembers.length > 0 && (
            <div className="bg-purple-50 border border-purple-200 rounded-lg p-4">
              <h3 className="font-semibold text-purple-900 mb-4">Earning Distribution Summary</h3>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div className="bg-white p-3 rounded border">
                  <p className="text-sm text-gray-600">Total Allocated</p>
                  <p className="text-2xl font-bold text-purple-600">{getTotalEarningPercentage()}%</p>
                </div>
                <div className="bg-white p-3 rounded border">
                  <p className="text-sm text-gray-600">Owner Share</p>
                  <p className="text-2xl font-bold text-blue-600">{100 - getTotalEarningPercentage()}%</p>
                </div>
                <div className="bg-white p-3 rounded border">
                  <p className="text-sm text-gray-600">Team Members</p>
                  <p className="text-2xl font-bold text-green-600">{teamMembers.filter(m => m.status === 'active').length}</p>
                </div>
              </div>
              {getTotalEarningPercentage() > 100 && (
                <div className="mt-3 bg-red-100 border border-red-300 text-red-700 px-3 py-2 rounded text-sm">
                  ⚠️ Warning: Total allocation exceeds 100%. Please adjust percentages.
                </div>
              )}
            </div>
          )}

          {/* Current Team Members */}
          <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
            <h3 className="font-semibold text-gray-900 mb-4">
              Current Team Members ({teamMembers.length + 1})
            </h3>
            
            <div className="space-y-3">
              {/* Owner */}
              <div className="flex items-center justify-between bg-white p-3 rounded border border-blue-200">
                <div className="flex-1">
                  <p className="font-mono text-sm text-gray-800">
                    {formatPrincipal(currentPrincipal)}
                    <span className="ml-2 text-xs text-blue-600 font-medium">(You)</span>
                  </p>
                  <p className="text-xs text-gray-600">
                    Role: Owner • Status: Active • Earning: {100 - getTotalEarningPercentage()}%
                  </p>
                </div>
                <span className="px-2 py-1 bg-blue-100 text-blue-800 text-xs rounded-full">
                  Owner
                </span>
              </div>

              {/* Team Members */}
              {teamMembers.map((member) => (
                <div key={member.principal} className="bg-white p-3 rounded border">
                  <div className="flex items-center justify-between">
                    <div className="flex-1">
                      <p className="font-mono text-sm text-gray-800">
                        {formatPrincipal(member.principal)}
                        {member.principal === currentPrincipal && <span className="ml-2 text-xs text-green-600 font-medium">(You)</span>}
                      </p>
                      <p className="text-xs text-gray-600">
                        Role: {member.role} • Added: {formatDate(member.addedAt)} • Status: {member.status}
                      </p>
                    </div>
                    <div className="flex items-center space-x-2">
                      <span className={`px-2 py-1 text-xs rounded-full ${
                        member.role === 'admin' 
                          ? 'bg-purple-100 text-purple-800' 
                          : 'bg-green-100 text-green-800'
                      }`}>
                        {member.role}
                      </span>
                      {isOwner && (
                        <button
                          onClick={() => handleRemoveTeamMember(member.principal)}
                          disabled={isLoading}
                          className="px-2 py-1 bg-red-600 text-white text-xs rounded hover:bg-red-700 disabled:bg-gray-400"
                        >
                          Remove
                        </button>
                      )}
                    </div>
                  </div>
                  
                  {/* Earning Percentage Section (Owner Only) */}
                  {isOwner && (
                    <div className="mt-3 pt-3 border-t border-gray-200">
                      <div className="flex items-center justify-between">
                        <div className="flex items-center space-x-2">
                          <span className="text-sm font-medium text-gray-700">Earning Share:</span>
                          {editingEarnings[member.principal] !== undefined ? (
                            <div className="flex items-center space-x-2">
                              <input
                                type="number"
                                min="0"
                                max="100"
                                step="0.1"
                                value={editingEarnings[member.principal] || 0}
                                onChange={(e) => setEditingEarnings(prev => ({ ...prev, [member.principal]: parseFloat(e.target.value) || 0 }))}
                                className="w-16 px-2 py-1 text-sm border border-gray-300 rounded focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                              />
                              <span className="text-sm text-gray-600">%</span>
                            </div>
                          ) : (
                            <span className="text-sm font-semibold text-blue-600">{member.earningPercentage}%</span>
                          )}
                        </div>
                        
                        <div className="flex items-center space-x-1">
                          {editingEarnings[member.principal] !== undefined ? (
                            <>
                              <button
                                onClick={() => handleUpdateEarning(member.principal, editingEarnings[member.principal] || 0)}
                                disabled={isLoading}
                                className="px-2 py-1 bg-green-600 text-white text-xs rounded hover:bg-green-700 disabled:bg-gray-400"
                              >
                                Save
                              </button>
                              <button
                                onClick={() => cancelEditingEarning(member.principal)}
                                disabled={isLoading}
                                className="px-2 py-1 bg-gray-600 text-white text-xs rounded hover:bg-gray-700 disabled:bg-gray-400"
                              >
                                Cancel
                              </button>
                            </>
                          ) : (
                            <button
                              onClick={() => startEditingEarning(member.principal, member.earningPercentage)}
                              disabled={isLoading}
                              className="px-2 py-1 bg-blue-600 text-white text-xs rounded hover:bg-blue-700 disabled:bg-gray-400"
                            >
                              Edit
                            </button>
                          )}
                        </div>
                      </div>
                    </div>
                  )}
                </div>
              ))}
            </div>

            {teamMembers.length === 0 && (
              <p className="text-gray-500 text-center py-4">
                No team members yet. Add some team members to collaborate!
              </p>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default TeamManagement;