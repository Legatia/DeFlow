import React, { useState } from 'react';
import { AuthClient } from '@dfinity/auth-client';

interface AdminAuthProps {
  onLogin: (principal: string) => Promise<void>;
}

const AdminAuth: React.FC<AdminAuthProps> = ({ onLogin }) => {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleInternetIdentityLogin = async () => {
    try {
      setIsLoading(true);
      setError(null);

      // DEVELOPMENT: Only allow development bypass for local network
      if (process.env.DFX_NETWORK === "local" && !process.env.VITE_INTERNET_IDENTITY_CANISTER_ID) {
        console.warn('DEVELOPMENT: Using development principal for local testing');
        const devPrincipal = process.env.VITE_OWNER_PRINCIPAL || prompt('Enter your owner principal for development testing:');
        if (devPrincipal && devPrincipal.length > 20) {
          await onLogin(devPrincipal);
          return;
        } else {
          throw new Error('Invalid development principal provided');
        }
      }

      // PRODUCTION: Ensure Internet Identity is properly configured
      if (process.env.DFX_NETWORK === "ic" && !process.env.VITE_INTERNET_IDENTITY_CANISTER_ID) {
        throw new Error('PRODUCTION: Internet Identity canister ID not configured for mainnet deployment');
      }

      // SECURITY: Real Internet Identity authentication for production
      const authClient = await AuthClient.create({
        idleOptions: {
          disableIdle: true,
          disableDefaultIdleCallback: true
        }
      });

      // Check if already authenticated
      if (await authClient.isAuthenticated()) {
        const identity = authClient.getIdentity();
        const principal = identity.getPrincipal().toString();
        await onLogin(principal);
        return;
      }

      // Start Internet Identity login flow
      await authClient.login({
        identityProvider: process.env.DFX_NETWORK === "local" 
          ? `http://localhost:4943/?canisterId=${process.env.VITE_INTERNET_IDENTITY_CANISTER_ID}`
          : `https://identity.ic0.app`,
        maxTimeToLive: BigInt(7 * 24 * 60 * 60 * 1000 * 1000 * 1000), // 7 days in nanoseconds
        onSuccess: async () => {
          const identity = authClient.getIdentity();
          const principal = identity.getPrincipal().toString();
          
          // SECURITY: Validate principal before login
          if (principal === '2vxsx-fae' || principal.length < 20) {
            throw new Error('SECURITY: Invalid principal detected');
          }
          
          await onLogin(principal);
        },
        onError: (error) => {
          if (error === 'UserInterrupt') {
            throw new Error('Login was canceled. Please try again.');
          }
          throw new Error(`Internet Identity authentication failed: ${error}`);
        }
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Authentication failed');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-gray-900 via-blue-900 to-purple-900 flex items-center justify-center">
      <div className="max-w-md w-full space-y-8 p-8 bg-white/10 backdrop-blur-lg rounded-xl border border-white/20">
        <div className="text-center">
          <div className="mx-auto h-12 w-12 bg-blue-500 rounded-lg flex items-center justify-center">
            <svg className="h-8 w-8 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
            </svg>
          </div>
          <h2 className="mt-6 text-3xl font-bold text-white">
            DeFlow Admin Access
          </h2>
          <p className="mt-2 text-sm text-gray-300">
            Secure treasury and pool management dashboard
          </p>
        </div>

        <div className="space-y-4">
          {error && (
            <div className="bg-red-500/20 border border-red-500 text-red-200 px-4 py-3 rounded-lg">
              {error}
            </div>
          )}

          <button
            onClick={handleInternetIdentityLogin}
            disabled={isLoading}
            className="group relative w-full flex justify-center py-3 px-4 border border-transparent text-sm font-medium rounded-lg text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {isLoading ? (
              <div className="flex items-center">
                <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
                Authenticating...
              </div>
            ) : (
              <div className="flex items-center">
                <svg className="h-5 w-5 mr-2" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
                </svg>
                Login with Internet Identity
              </div>
            )}
          </button>
        </div>

        <div className="text-center">
          <p className="text-xs text-gray-400">
            Owner-only access • Secure treasury management
          </p>
        </div>
      </div>
    </div>
  );
};

export default AdminAuth;