import React, { useState } from 'react';
import TreasuryManagement from '../components/TreasuryManagement';
import PoolManagement from '../components/PoolManagement';
import SystemHealth from '../components/SystemHealth';
import TeamManagement from '../components/TeamManagement';
import EarningsManagement from '../components/EarningsManagement';

interface AdminSession {
  principal: string;
  isOwner: boolean;
  isTeamMember: boolean;
  sessionStart: number;
}

interface AdminDashboardProps {
  adminSession: AdminSession;
  onLogout: () => void;
}

const AdminDashboard: React.FC<AdminDashboardProps> = ({ adminSession, onLogout }) => {
  const [activeTab, setActiveTab] = useState<'treasury' | 'pool' | 'system' | 'team' | 'earnings'>('treasury');

  const formatSessionTime = (timestamp: number) => {
    const hours = Math.floor((Date.now() - timestamp) / (1000 * 60 * 60));
    const minutes = Math.floor(((Date.now() - timestamp) % (1000 * 60 * 60)) / (1000 * 60));
    return `${hours}h ${minutes}m ago`;
  };

  return (
    <div className="min-h-screen bg-gray-900">
      {/* Header */}
      <header className="bg-gray-800 border-b border-gray-700">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center py-4">
            <div className="flex items-center">
              <div className="flex items-center">
                <div className="h-8 w-8 bg-blue-500 rounded-lg flex items-center justify-center mr-3">
                  <svg className="h-5 w-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                  </svg>
                </div>
                <div>
                  <h1 className="text-xl font-bold text-white">DeFlow Admin</h1>
                  <p className="text-xs text-gray-400">Treasury & Pool Management</p>
                </div>
              </div>
            </div>

            <div className="flex items-center space-x-4">
              {/* Session Info */}
              <div className="text-right">
                <p className="text-sm text-white">
                  {adminSession.isOwner ? 'Owner Session' : adminSession.isTeamMember ? 'Team Member' : 'Setup Mode'}
                  <span className={`inline-flex items-center ml-2 px-2 py-1 rounded-full text-xs font-medium ${
                    adminSession.isOwner ? 'bg-green-100 text-green-800' : 
                    adminSession.isTeamMember ? 'bg-blue-100 text-blue-800' : 
                    'bg-yellow-100 text-yellow-800'
                  }`}>
                    {adminSession.isOwner ? 'Owner' : adminSession.isTeamMember ? 'Member' : 'Setup'}
                  </span>
                </p>
                <p className="text-xs text-gray-400">
                  {(adminSession.isOwner || adminSession.isTeamMember) ? `Started ${formatSessionTime(adminSession.sessionStart)}` : `Principal: ${adminSession.principal.slice(0, 8)}...${adminSession.principal.slice(-8)}`}
                </p>
              </div>

              {/* Logout Button */}
              <button
                onClick={onLogout}
                className="inline-flex items-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-white bg-red-600 hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 transition-colors"
              >
                <svg className="h-4 w-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
                </svg>
                Logout
              </button>
            </div>
          </div>
        </div>
      </header>

      {/* Navigation */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="border-b border-gray-700">
          <nav className="-mb-px flex space-x-8">
            {[
              { id: 'treasury', label: 'Treasury Management', icon: '💰' },
              { id: 'pool', label: 'Pool Management', icon: '🏊' },
              { id: 'system', label: 'System Health', icon: '📊' },
              { id: 'team', label: 'Team Management', icon: '👥' },
              { id: 'earnings', label: 'Earnings Management', icon: '💼' }
            ].map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id as any)}
                className={`py-4 px-1 border-b-2 font-medium text-sm ${
                  activeTab === tab.id
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
      </div>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {activeTab === 'treasury' && <TreasuryManagement />}
        {activeTab === 'pool' && <PoolManagement />}
        {activeTab === 'system' && <SystemHealth />}
        {activeTab === 'team' && (
          <TeamManagement 
            isOwner={adminSession.isOwner}
            currentPrincipal={adminSession.principal}
          />
        )}
        {activeTab === 'earnings' && (
          <EarningsManagement 
            isOwner={adminSession.isOwner}
            currentPrincipal={adminSession.principal}
          />
        )}
      </main>
    </div>
  );
};

export default AdminDashboard;