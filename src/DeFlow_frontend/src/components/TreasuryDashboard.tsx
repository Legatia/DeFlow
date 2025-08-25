/**
 * Treasury Dashboard Component
 * Provides comprehensive treasury monitoring for team members
 */

import React from 'react';

interface TreasuryDashboardProps {
  userRole?: string;
}

// Temporarily disabled due to missing treasury service
const TreasuryDashboard: React.FC<TreasuryDashboardProps> = ({ userRole }) => {
  return (
    <div className="p-6 bg-white rounded-lg shadow-sm">
      <h2 className="text-2xl font-bold mb-4">Treasury Dashboard</h2>
      <p className="text-gray-600">Treasury functionality temporarily disabled.</p>
      <p className="text-sm text-gray-500 mt-2">
        This feature will be available in the next release.
      </p>
    </div>
  );
};

export default TreasuryDashboard;