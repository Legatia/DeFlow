import React, { useState, useEffect } from 'react'
import oauth2Service, { OAuth2Token } from '../services/oauth2Service'
import emailService from '../services/emailService'

interface OAuth2SetupProps {
  onProviderConnected?: (provider: string) => void
  onProviderDisconnected?: (provider: string) => void
}

const OAuth2Setup: React.FC<OAuth2SetupProps> = ({ 
  onProviderConnected, 
  onProviderDisconnected 
}) => {
  const [authenticatedProviders, setAuthenticatedProviders] = useState<string[]>([])
  const [pendingAuth, setPendingAuth] = useState<string | null>(null)
  const [testResults, setTestResults] = useState<Record<string, boolean>>({})
  const [showConfigForm, setShowConfigForm] = useState(false)
  const [configs, setConfigs] = useState({
    gmail: {
      clientId: '',
      enabled: false
    },
    outlook: {
      clientId: '',
      enabled: false
    }
  })

  useEffect(() => {
    loadAuthenticatedProviders()
    loadStoredConfigs()
    
    // Handle OAuth callback if present in URL
    handleOAuthCallback()
  }, [])

  const loadAuthenticatedProviders = () => {
    const providers = oauth2Service.getAuthenticatedProviders()
    setAuthenticatedProviders(providers)
  }

  const loadStoredConfigs = () => {
    try {
      const stored = localStorage.getItem('oauth2_configs')
      if (stored) {
        const parsedConfigs = JSON.parse(stored)
        setConfigs(parsedConfigs)
        
        // Configure OAuth2 service with stored configs
        if (parsedConfigs.gmail.enabled && parsedConfigs.gmail.clientId) {
          oauth2Service.configureProvider('gmail', {
            clientId: parsedConfigs.gmail.clientId,
            redirectUri: `${window.location.origin}/oauth/callback/gmail`,
            scopes: [],
            tokenEndpoint: '/api/auth/google/token',
            usePKCE: true
          })
        }
        
        if (parsedConfigs.outlook.enabled && parsedConfigs.outlook.clientId) {
          oauth2Service.configureProvider('outlook', {
            clientId: parsedConfigs.outlook.clientId,
            redirectUri: `${window.location.origin}/oauth/callback/outlook`,
            scopes: [],
            tokenEndpoint: '/api/auth/microsoft/token',
            usePKCE: true
          })
        }
      }
    } catch (error) {
      console.warn('Failed to load OAuth2 configs:', error)
    }
  }

  const saveConfigs = () => {
    try {
      localStorage.setItem('oauth2_configs', JSON.stringify(configs))
      
      // Configure OAuth2 service
      if (configs.gmail.enabled && configs.gmail.clientId) {
        oauth2Service.configureProvider('gmail', {
          clientId: configs.gmail.clientId,
          redirectUri: `${window.location.origin}/oauth/callback/gmail`,
          scopes: [],
          tokenEndpoint: '/api/auth/google/token',
          usePKCE: true
        })
      }
      
      if (configs.outlook.enabled && configs.outlook.clientId) {
        oauth2Service.configureProvider('outlook', {
          clientId: configs.outlook.clientId,
          redirectUri: `${window.location.origin}/oauth/callback/outlook`,
          scopes: [],
          tokenEndpoint: '/api/auth/microsoft/token',
          usePKCE: true
        })
      }
      
      setShowConfigForm(false)
      alert('OAuth2 configurations saved successfully!')
    } catch (error) {
      alert('Failed to save configurations')
      console.error('Save error:', error)
    }
  }

  const handleOAuthCallback = async () => {
    const urlParams = new URLSearchParams(window.location.search)
    const provider = urlParams.get('provider')
    const code = urlParams.get('code')
    const error = urlParams.get('error')

    if (error) {
      alert(`Authentication failed: ${error}`)
      setPendingAuth(null)
      return
    }

    if (provider && code && (provider === 'gmail' || provider === 'outlook')) {
      try {
        setPendingAuth(provider)
        const token = await oauth2Service.exchangeCodeForTokens(provider, code)
        
        // Register with email service
        emailService.addProvider(`${provider}-oauth`, {
          provider,
          credentials: {
            access_token: token.access_token,
            refresh_token: token.refresh_token
          }
        })
        
        loadAuthenticatedProviders()
        onProviderConnected?.(provider)
        
        // Clear URL parameters
        window.history.replaceState({}, document.title, window.location.pathname)
        
        alert(`${provider === 'gmail' ? 'Gmail' : 'Outlook'} connected successfully!`)
      } catch (error) {
        alert(`Failed to connect ${provider}: ${error}`)
        console.error('OAuth callback error:', error)
      } finally {
        setPendingAuth(null)
      }
    }
  }

  const handleConnect = async (provider: 'gmail' | 'outlook') => {
    try {
      const config = configs[provider]
      if (!config.enabled || !config.clientId) {
        alert(`Please configure ${provider === 'gmail' ? 'Gmail' : 'Outlook'} OAuth2 settings first`)
        setShowConfigForm(true)
        return
      }

      const state = oauth2Service.generateState()
      sessionStorage.setItem(`oauth_state_${provider}`, state)
      
      const authUrl = oauth2Service.getAuthorizationUrl(provider, state)
      
      // For development, we'll use popup window
      // In production, you might want to redirect the whole page
      const popup = window.open(
        authUrl,
        `oauth_${provider}`,
        'width=500,height=600,scrollbars=yes,resizable=yes'
      )

      // Monitor popup for completion
      const checkClosed = setInterval(() => {
        if (popup?.closed) {
          clearInterval(checkClosed)
          // Reload to check for authentication
          setTimeout(() => {
            loadAuthenticatedProviders()
          }, 1000)
        }
      }, 1000)
      
    } catch (error) {
      alert(`Failed to start authentication: ${error}`)
      console.error('OAuth connect error:', error)
    }
  }

  const handleDisconnect = async (provider: 'gmail' | 'outlook') => {
    try {
      await oauth2Service.revokeToken(provider)
      emailService.removeProvider(`${provider}-oauth`)
      loadAuthenticatedProviders()
      onProviderDisconnected?.(provider)
      alert(`${provider === 'gmail' ? 'Gmail' : 'Outlook'} disconnected successfully`)
    } catch (error) {
      alert(`Failed to disconnect ${provider}: ${error}`)
      console.error('OAuth disconnect error:', error)
    }
  }

  const handleTestConnection = async (provider: 'gmail' | 'outlook') => {
    try {
      const isValid = await oauth2Service.testToken(provider)
      setTestResults(prev => ({ ...prev, [provider]: isValid }))
      
      if (isValid) {
        alert(`${provider === 'gmail' ? 'Gmail' : 'Outlook'} connection is working!`)
      } else {
        alert(`${provider === 'gmail' ? 'Gmail' : 'Outlook'} connection failed. Please reconnect.`)
      }
    } catch (error) {
      setTestResults(prev => ({ ...prev, [provider]: false }))
      alert(`Connection test failed: ${error}`)
    }
  }

  const getProviderIcon = (provider: string) => {
    switch (provider) {
      case 'gmail': return '🟩'
      case 'outlook': return '🟦'
      default: return '📧'
    }
  }

  const getProviderColor = (provider: string) => {
    switch (provider) {
      case 'gmail': return 'border-green-300 bg-green-50'
      case 'outlook': return 'border-blue-300 bg-blue-50'
      default: return 'border-gray-300 bg-gray-50'
    }
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium text-gray-900">OAuth2 Providers</h3>
          <p className="text-sm text-gray-600">
            Secure authentication for Gmail and Outlook
          </p>
        </div>
        <button
          onClick={() => setShowConfigForm(true)}
          className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors flex items-center space-x-2"
        >
          <span>⚙️</span>
          <span>Configure</span>
        </button>
      </div>

      {/* Provider Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* Gmail */}
        <div className={`p-4 border-2 rounded-lg ${getProviderColor('gmail')}`}>
          <div className="flex items-center justify-between mb-3">
            <div className="flex items-center space-x-3">
              <span className="text-2xl">{getProviderIcon('gmail')}</span>
              <div>
                <h4 className="font-medium text-gray-900">Gmail API</h4>
                <p className="text-sm text-gray-600">Google Workspace integration</p>
              </div>
            </div>
            <div className="flex items-center space-x-2">
              {authenticatedProviders.includes('gmail') && (
                <>
                  {testResults.gmail !== undefined && (
                    <span className={`w-3 h-3 rounded-full ${
                      testResults.gmail ? 'bg-green-500' : 'bg-red-500'
                    }`} title={testResults.gmail ? 'Connected' : 'Connection Failed'}></span>
                  )}
                  <span className="px-2 py-1 text-xs bg-green-100 text-green-800 rounded-full">
                    Connected
                  </span>
                </>
              )}
              {!configs.gmail.enabled && (
                <span className="px-2 py-1 text-xs bg-gray-100 text-gray-600 rounded-full">
                  Not Configured
                </span>
              )}
            </div>
          </div>
          
          <div className="space-y-2">
            {authenticatedProviders.includes('gmail') ? (
              <div className="flex space-x-2">
                <button
                  onClick={() => handleTestConnection('gmail')}
                  className="flex-1 px-3 py-2 text-sm bg-green-600 text-white rounded hover:bg-green-700 transition-colors"
                  disabled={pendingAuth === 'gmail'}
                >
                  Test Connection
                </button>
                <button
                  onClick={() => handleDisconnect('gmail')}
                  className="flex-1 px-3 py-2 text-sm bg-red-600 text-white rounded hover:bg-red-700 transition-colors"
                  disabled={pendingAuth === 'gmail'}
                >
                  Disconnect
                </button>
              </div>
            ) : (
              <button
                onClick={() => handleConnect('gmail')}
                className="w-full px-3 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
                disabled={pendingAuth === 'gmail' || !configs.gmail.enabled}
              >
                {pendingAuth === 'gmail' ? 'Connecting...' : 'Connect Gmail'}
              </button>
            )}
            
            {configs.gmail.enabled && (
              <div className="text-xs text-gray-500">
                <p>✅ Client ID configured</p>
                <p>🔄 Redirect: {window.location.origin}/oauth/callback/gmail</p>
              </div>
            )}
          </div>
        </div>

        {/* Outlook */}
        <div className={`p-4 border-2 rounded-lg ${getProviderColor('outlook')}`}>
          <div className="flex items-center justify-between mb-3">
            <div className="flex items-center space-x-3">
              <span className="text-2xl">{getProviderIcon('outlook')}</span>
              <div>
                <h4 className="font-medium text-gray-900">Outlook API</h4>
                <p className="text-sm text-gray-600">Microsoft 365 integration</p>
              </div>
            </div>
            <div className="flex items-center space-x-2">
              {authenticatedProviders.includes('outlook') && (
                <>
                  {testResults.outlook !== undefined && (
                    <span className={`w-3 h-3 rounded-full ${
                      testResults.outlook ? 'bg-green-500' : 'bg-red-500'
                    }`} title={testResults.outlook ? 'Connected' : 'Connection Failed'}></span>
                  )}
                  <span className="px-2 py-1 text-xs bg-blue-100 text-blue-800 rounded-full">
                    Connected
                  </span>
                </>
              )}
              {!configs.outlook.enabled && (
                <span className="px-2 py-1 text-xs bg-gray-100 text-gray-600 rounded-full">
                  Not Configured
                </span>
              )}
            </div>
          </div>
          
          <div className="space-y-2">
            {authenticatedProviders.includes('outlook') ? (
              <div className="flex space-x-2">
                <button
                  onClick={() => handleTestConnection('outlook')}
                  className="flex-1 px-3 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
                  disabled={pendingAuth === 'outlook'}
                >
                  Test Connection
                </button>
                <button
                  onClick={() => handleDisconnect('outlook')}
                  className="flex-1 px-3 py-2 text-sm bg-red-600 text-white rounded hover:bg-red-700 transition-colors"
                  disabled={pendingAuth === 'outlook'}
                >
                  Disconnect
                </button>
              </div>
            ) : (
              <button
                onClick={() => handleConnect('outlook')}
                className="w-full px-3 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
                disabled={pendingAuth === 'outlook' || !configs.outlook.enabled}
              >
                {pendingAuth === 'outlook' ? 'Connecting...' : 'Connect Outlook'}
              </button>
            )}
            
            {configs.outlook.enabled && (
              <div className="text-xs text-gray-500">
                <p>✅ Client ID configured</p>
                <p>🔄 Redirect: {window.location.origin}/oauth/callback/outlook</p>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Configuration Modal */}
      {showConfigForm && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-full max-w-2xl mx-4 max-h-screen overflow-y-auto">
            <div className="flex items-center justify-between mb-4">
              <h4 className="text-lg font-medium text-gray-900">OAuth2 Configuration</h4>
              <button
                onClick={() => setShowConfigForm(false)}
                className="text-gray-400 hover:text-gray-600"
              >
                ✕
              </button>
            </div>

            <div className="space-y-6">
              {/* Gmail Configuration */}
              <div className="border rounded-lg p-4">
                <div className="flex items-center justify-between mb-3">
                  <h5 className="font-medium text-gray-900">Gmail API Configuration</h5>
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input
                      type="checkbox"
                      checked={configs.gmail.enabled}
                      onChange={(e) => setConfigs(prev => ({
                        ...prev,
                        gmail: { ...prev.gmail, enabled: e.target.checked }
                      }))}
                      className="sr-only peer"
                    />
                    <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
                  </label>
                </div>
                
                {configs.gmail.enabled && (
                  <div className="space-y-3">
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">
                        Google Client ID
                      </label>
                      <input
                        type="text"
                        value={configs.gmail.clientId}
                        onChange={(e) => setConfigs(prev => ({
                          ...prev,
                          gmail: { ...prev.gmail, clientId: e.target.value }
                        }))}
                        placeholder="your-app.googleusercontent.com"
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                      />
                    </div>
                    <div className="text-sm text-green-600 bg-green-50 p-3 rounded">
                      <p className="font-medium">🔒 SECURITY: Client secrets are handled securely by the backend using PKCE flow.</p>
                      <p>No client secret needed in the frontend for enhanced security.</p>
                    </div>
                    <div className="text-sm text-gray-600 bg-blue-50 p-3 rounded">
                      <p className="font-medium mb-1">Setup Instructions:</p>
                      <ol className="list-decimal list-inside space-y-1 text-xs">
                        <li>Go to Google Cloud Console → APIs & Services → Credentials</li>
                        <li>Create OAuth 2.0 Client ID for Web Application</li>
                        <li>Add redirect URI: <code className="bg-white px-1 rounded">{window.location.origin}/oauth/callback/gmail</code></li>
                        <li>Enable Gmail API in APIs & Services → Library</li>
                      </ol>
                    </div>
                  </div>
                )}
              </div>

              {/* Outlook Configuration */}
              <div className="border rounded-lg p-4">
                <div className="flex items-center justify-between mb-3">
                  <h5 className="font-medium text-gray-900">Outlook API Configuration</h5>
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input
                      type="checkbox"
                      checked={configs.outlook.enabled}
                      onChange={(e) => setConfigs(prev => ({
                        ...prev,
                        outlook: { ...prev.outlook, enabled: e.target.checked }
                      }))}
                      className="sr-only peer"
                    />
                    <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
                  </label>
                </div>
                
                {configs.outlook.enabled && (
                  <div className="space-y-3">
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">
                        Microsoft Application (Client) ID
                      </label>
                      <input
                        type="text"
                        value={configs.outlook.clientId}
                        onChange={(e) => setConfigs(prev => ({
                          ...prev,
                          outlook: { ...prev.outlook, clientId: e.target.value }
                        }))}
                        placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                      />
                    </div>
                    <div className="text-sm text-green-600 bg-green-50 p-3 rounded">
                      <p className="font-medium">🔒 SECURITY: Client secrets are handled securely by the backend using PKCE flow.</p>
                      <p>No client secret needed in the frontend for enhanced security.</p>
                    </div>
                    <div className="text-sm text-gray-600 bg-blue-50 p-3 rounded">
                      <p className="font-medium mb-1">Setup Instructions:</p>
                      <ol className="list-decimal list-inside space-y-1 text-xs">
                        <li>Go to Azure Portal → App Registrations → New registration</li>
                        <li>Add redirect URI: <code className="bg-white px-1 rounded">{window.location.origin}/oauth/callback/outlook</code></li>
                        <li>Go to Certificates & secrets → New client secret</li>
                        <li>Add API permissions: Mail.Send, Mail.Read, User.Read</li>
                      </ol>
                    </div>
                  </div>
                )}
              </div>
            </div>

            <div className="flex justify-end space-x-3 mt-6 pt-4 border-t">
              <button
                onClick={() => setShowConfigForm(false)}
                className="px-4 py-2 text-gray-600 border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={saveConfigs}
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
              >
                Save Configuration
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Help Section */}
      <div className="bg-amber-50 rounded-lg p-4 border border-amber-200">
        <h5 className="font-medium text-amber-800 mb-2">🔐 OAuth2 Security Benefits</h5>
        <ul className="text-sm text-amber-700 space-y-1">
          <li>• No password storage - uses secure tokens</li>
          <li>• Automatic token refresh for seamless operation</li>
          <li>• Granular permissions for email sending only</li>
          <li>• Users can revoke access anytime from their account</li>
          <li>• Works with 2FA and enterprise accounts</li>
        </ul>
      </div>
    </div>
  )
}

export default OAuth2Setup