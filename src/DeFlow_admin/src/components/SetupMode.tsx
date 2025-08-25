import React, { useState } from 'react';

interface SetupModeProps {
  userPrincipal: string;
  onSetupComplete: () => void;
}

const SetupMode: React.FC<SetupModeProps> = ({ userPrincipal, onSetupComplete }) => {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const handleCopyPrincipal = async () => {
    try {
      await navigator.clipboard.writeText(userPrincipal);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy principal:', err);
    }
  };

  const handleSetOwner = async () => {
    try {
      setIsLoading(true);
      setError(null);

      // TODO: Call backend method to set this principal as owner
      // For now, we'll simulate this by showing instructions
      
      // In a real implementation, you would:
      // 1. Call your backend canister's set_owner method
      // 2. Pass the userPrincipal as the owner
      // 3. Update the frontend environment or refetch config
      
      console.log('Setting owner principal:', userPrincipal);
      
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      // For now, show instructions to manually update .env.production
      alert(`SETUP INSTRUCTIONS:\n\n1. Copy your principal: ${userPrincipal}\n\n2. Update .env.production:\nVITE_OWNER_PRINCIPAL=${userPrincipal}\n\n3. Redeploy the admin canister\n\n4. Refresh this page`);
      
      onSetupComplete();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Setup failed');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-gray-900 via-blue-900 to-purple-900 flex items-center justify-center p-4">
      <div className="max-w-2xl w-full space-y-8 p-8 bg-white/10 backdrop-blur-lg rounded-xl border border-white/20">
        <div className="text-center">
          <div className="mx-auto h-16 w-16 bg-yellow-500 rounded-lg flex items-center justify-center">
            <svg className="h-10 w-10 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 100 4m0-4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 100 4m0-4v2m0-6V4" />
            </svg>
          </div>
          <h2 className="mt-6 text-3xl font-bold text-white">
            Initial Admin Setup
          </h2>
          <p className="mt-2 text-gray-300">
            Configure the owner principal for this DeFlow admin dashboard
          </p>
        </div>

        <div className="space-y-6">
          <div className="bg-blue-500/20 border border-blue-500 rounded-lg p-4">
            <h3 className="text-blue-200 font-semibold mb-2">🎉 You're the First Admin!</h3>
            <p className="text-blue-100 text-sm">
              Since no owner is configured yet, you can set yourself as the admin owner. 
              After setup, only authorized principals will be able to access this dashboard.
            </p>
          </div>

          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Your Internet Identity Principal
              </label>
              <div className="flex rounded-lg border border-gray-600">
                <input
                  type="text"
                  value={userPrincipal}
                  readOnly
                  className="flex-1 px-3 py-2 bg-gray-800 text-white rounded-l-lg text-sm font-mono"
                />
                <button
                  onClick={handleCopyPrincipal}
                  className="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-r-lg transition-colors"
                >
                  {copied ? '✅ Copied' : '📋 Copy'}
                </button>
              </div>
              <p className="mt-1 text-xs text-gray-400">
                This is your unique Internet Identity principal that will become the admin owner
              </p>
            </div>

            {error && (
              <div className="bg-red-500/20 border border-red-500 text-red-200 px-4 py-3 rounded-lg">
                {error}
              </div>
            )}

            <div className="bg-yellow-500/20 border border-yellow-500 rounded-lg p-4">
              <h4 className="text-yellow-200 font-semibold mb-2">📝 Setup Steps:</h4>
              <ol className="text-yellow-100 text-sm space-y-1 list-decimal list-inside">
                <li>Copy your principal above</li>
                <li>Update your .env.production file with VITE_OWNER_PRINCIPAL</li>
                <li>Redeploy the admin canister</li>
                <li>Refresh this page - you'll then have full admin access</li>
              </ol>
            </div>

            <button
              onClick={handleSetOwner}
              disabled={isLoading}
              className="w-full py-3 px-4 bg-green-600 hover:bg-green-700 text-white font-medium rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isLoading ? (
                <div className="flex items-center justify-center">
                  <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
                  Setting up admin owner...
                </div>
              ) : (
                'Set Me as Admin Owner'
              )}
            </button>
          </div>
        </div>

        <div className="text-center">
          <p className="text-xs text-gray-400">
            One-time setup • Your principal will become the permanent admin owner
          </p>
        </div>
      </div>
    </div>
  );
};

export default SetupMode;