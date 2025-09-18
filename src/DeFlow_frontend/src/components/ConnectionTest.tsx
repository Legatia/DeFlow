import React, { useState, useEffect } from 'react';
import simpleDefiTemplateService, { DeFiWorkflowTemplate } from '../services/defiTemplateServiceSimple';

interface ConnectionTestProps {
  onConnectionVerified?: (isConnected: boolean) => void;
}

const ConnectionTest = ({ onConnectionVerified }: ConnectionTestProps) => {
  const [templates, setTemplates] = useState<DeFiWorkflowTemplate[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [connectionStatus, setConnectionStatus] = useState<'testing' | 'connected' | 'failed'>('testing');

  useEffect(() => {
    testConnection();
  }, []);

  const testConnection = async () => {
    setLoading(true);
    setError(null);
    setConnectionStatus('testing');

    try {
      console.log('Testing frontend-backend connection...');
      
      // Initialize the service
      await simpleDefiTemplateService.initialize();
      
      // Try to fetch templates from backend
      const templateList = await simpleDefiTemplateService.listWorkflowTemplates();
      
      setTemplates(templateList);
      setConnectionStatus('connected');
      onConnectionVerified?.(true);
      
      console.log('✅ Connection successful! Retrieved templates:', templateList);
    } catch (err) {
      console.error('❌ Connection failed:', err);
      setError(err instanceof Error ? err.message : 'Connection failed');
      setConnectionStatus('failed');
      onConnectionVerified?.(false);
    } finally {
      setLoading(false);
    }
  };

  const getStatusColor = () => {
    switch (connectionStatus) {
      case 'testing': return 'text-yellow-600 bg-yellow-50 border-yellow-200';
      case 'connected': return 'text-green-600 bg-green-50 border-green-200';
      case 'failed': return 'text-red-600 bg-red-50 border-red-200';
      default: return 'text-gray-600 bg-gray-50 border-gray-200';
    }
  };

  const getStatusIcon = () => {
    switch (connectionStatus) {
      case 'testing': return loading ? '🔄' : '🧪';
      case 'connected': return '✅';
      case 'failed': return '❌';
      default: return '❓';
    }
  };

  return (
    <div className="max-w-4xl mx-auto p-6">
      <h2 className="text-2xl font-bold text-gray-900 mb-6">Frontend-Backend Connection Test</h2>
      
      {/* Connection Status */}
      <div className={`border rounded-lg p-4 mb-6 ${getStatusColor()}`}>
        <div className="flex items-center">
          <span className="text-2xl mr-3">{getStatusIcon()}</span>
          <div>
            <h3 className="font-semibold">
              {connectionStatus === 'testing' && 'Testing Connection...'}
              {connectionStatus === 'connected' && 'Connection Successful'}
              {connectionStatus === 'failed' && 'Connection Failed'}
            </h3>
            <p className="text-sm mt-1">
              {connectionStatus === 'testing' && 'Attempting to connect to DeFlow backend canister...'}
              {connectionStatus === 'connected' && `Successfully connected and retrieved ${templates.length} templates`}
              {connectionStatus === 'failed' && error}
            </p>
          </div>
        </div>
      </div>

      {/* Retry Button */}
      {connectionStatus === 'failed' && (
        <div className="mb-6">
          <button
            onClick={testConnection}
            disabled={loading}
            className="bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 disabled:opacity-50"
          >
            {loading ? 'Testing...' : 'Retry Connection'}
          </button>
        </div>
      )}

      {/* Templates Display */}
      {connectionStatus === 'connected' && templates.length > 0 && (
        <div className="bg-white border border-gray-200 rounded-lg p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">
            Available Strategy Templates ({templates.length})
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {templates.map((template) => (
              <div key={template.id} className="border border-gray-200 rounded-lg p-4">
                <div className="flex items-center justify-between mb-2">
                  <h4 className="font-medium text-gray-900">{template.name}</h4>
                  <span className="px-2 py-1 bg-blue-100 text-blue-800 text-xs rounded-full">
                    {template.category || 'unknown'}
                  </span>
                </div>
                <p className="text-sm text-gray-600 mb-3">{template.description}</p>
                <div className="flex justify-between text-sm">
                  <span className="text-green-600 font-medium">
                    {template.estimated_apy.toFixed(1)}% APY
                  </span>
                  <span className="text-gray-500">
                    Min: ${template.min_capital_usd.toLocaleString()}
                  </span>
                </div>
                <div className="mt-2 flex justify-between text-xs">
                  <span className={`px-2 py-1 rounded-full ${
                    (template.difficulty || 'beginner') === 'Beginner' ? 'bg-green-100 text-green-800' :
                    (template.difficulty || 'beginner') === 'Intermediate' ? 'bg-yellow-100 text-yellow-800' :
                    'bg-red-100 text-red-800'
                  }`}>
                    {template.difficulty || 'beginner'}
                  </span>
                  <span className="text-gray-500">
                    Risk: {template.risk_score}/10
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Debug Information */}
      {process.env.NODE_ENV === 'development' && (
        <div className="mt-6 bg-gray-50 border border-gray-200 rounded-lg p-4">
          <h3 className="font-semibold text-gray-900 mb-2">Debug Information</h3>
          <div className="text-sm space-y-1">
            <p><strong>Backend Canister ID:</strong> {import.meta.env.VITE_CANISTER_ID_DEFLOW_BACKEND || 'Not configured'}</p>
            <p><strong>Network:</strong> {import.meta.env.VITE_DFX_NETWORK || 'local'}</p>
            <p><strong>IC Host:</strong> {import.meta.env.VITE_IC_HOST || 'http://127.0.0.1:4943'}</p>
            <p><strong>Connection Status:</strong> {connectionStatus}</p>
            {error && <p><strong>Error:</strong> {error}</p>}
          </div>
        </div>
      )}
    </div>
  );
};

export default ConnectionTest;