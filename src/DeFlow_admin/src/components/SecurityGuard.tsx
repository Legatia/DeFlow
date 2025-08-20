import React, { useEffect, useState } from 'react';
import { AdminAuthService } from '../services/adminAuthService';

interface SecurityGuardProps {
  children: React.ReactNode;
}

const SecurityGuard: React.FC<SecurityGuardProps> = ({ children }) => {
  const [isSecure, setIsSecure] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    checkSecurity();
  }, []);

  const checkSecurity = async () => {
    try {
      // SECURITY: Validate environment is properly configured
      if (!process.env.VITE_OWNER_PRINCIPAL) {
        throw new Error('SECURITY: Owner principal not configured. Admin dashboard cannot start.');
      }

      if (!process.env.VITE_CANISTER_ID_DEFLOW_POOL) {
        throw new Error('SECURITY: Pool canister ID not configured.');
      }

      // SECURITY: Check for development environment leaks
      if (process.env.NODE_ENV === 'production' && process.env.DFX_NETWORK === 'local') {
        console.warn('WARNING: Production build with local network configuration detected');
      }

      // SECURITY: Validate current session
      const session = await AdminAuthService.getCurrentSession();
      if (session && !await AdminAuthService.isOwner()) {
        await AdminAuthService.logout();
        throw new Error('SECURITY: Invalid session detected, cleared for safety');
      }

      setIsSecure(true);
    } catch (error) {
      setError(error instanceof Error ? error.message : 'Security validation failed');
    }
  };

  if (error) {
    return (
      <div className="min-h-screen bg-red-900 flex items-center justify-center">
        <div className="max-w-md w-full p-8 bg-red-800/50 rounded-xl border border-red-600">
          <div className="text-center">
            <div className="mx-auto h-12 w-12 bg-red-500 rounded-lg flex items-center justify-center mb-4">
              <svg className="h-8 w-8 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
                      d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-2.694-.833-3.464 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z" />
              </svg>
            </div>
            <h2 className="text-2xl font-bold text-white mb-4">Security Error</h2>
            <p className="text-red-200 mb-6">{error}</p>
            <button 
              onClick={checkSecurity}
              className="bg-red-600 hover:bg-red-700 text-white font-medium py-2 px-4 rounded-lg transition-colors"
            >
              Retry Security Check
            </button>
          </div>
        </div>
      </div>
    );
  }

  if (!isSecure) {
    return (
      <div className="min-h-screen bg-gray-900 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
          <p className="text-gray-300">Performing security validation...</p>
        </div>
      </div>
    );
  }

  return <>{children}</>;
};

export default SecurityGuard;