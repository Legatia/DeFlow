/**
 * Twitter/X API Setup Component
 * Provides Twitter API credential configuration with testing and validation
 */

import React, { useState, useEffect } from 'react'
import twitterService, { TwitterCredentials } from '../services/twitterService'
// SECURITY: Import input validation service
import inputValidationService from '../services/inputValidationService'

interface TwitterAPIConfig extends TwitterCredentials {
  id: string
  name: string
  account_name?: string
  is_connected: boolean
  created_at: string
  last_tested?: string
}

const TwitterAPISetup: React.FC = () => {
  const [configs, setConfigs] = useState<TwitterAPIConfig[]>([])
  const [isAddingConfig, setIsAddingConfig] = useState(false)
  const [testingConfig, setTestingConfig] = useState<string | null>(null)
  const [newConfig, setNewConfig] = useState({
    name: '',
    api_key: '',
    api_secret: '',
    access_token: '',
    access_token_secret: ''
  })
  const [validationErrors, setValidationErrors] = useState<Record<string, string>>({})
  const [testResults, setTestResults] = useState<Record<string, { success: boolean; message: string }>>({})
  // SECURITY: Add rate limiting state
  const [lastSubmitTime, setLastSubmitTime] = useState(0)

  // Load saved configs from localStorage
  useEffect(() => {
    loadConfigs()
  }, [])

  const loadConfigs = () => {
    try {
      const savedConfigs = localStorage.getItem('deflow_twitter_configs')
      if (savedConfigs) {
        setConfigs(JSON.parse(savedConfigs))
      }
    } catch (error) {
      console.error('Error loading Twitter configs:', error)
    }
  }

  const saveConfigs = (updatedConfigs: TwitterAPIConfig[]) => {
    try {
      localStorage.setItem('deflow_twitter_configs', JSON.stringify(updatedConfigs))
      setConfigs(updatedConfigs)
    } catch (error) {
      console.error('Error saving Twitter configs:', error)
    }
  }

  // SECURITY: Use comprehensive validation service
  const validateConfig = (config: typeof newConfig): { isValid: boolean; errors: Record<string, string>; sanitizedData?: any } => {
    return inputValidationService.validateTwitterCredentials({
      name: config.name,
      api_key: config.api_key,
      api_secret: config.api_secret,
      access_token: config.access_token,
      access_token_secret: config.access_token_secret
    })
  }

  const handleAddConfig = async () => {
    // SECURITY: Rate limiting check
    if (!inputValidationService.validateRateLimiting(lastSubmitTime, 2000)) {
      setValidationErrors({ general: 'Please wait before submitting again' })
      return
    }

    const validation = validateConfig(newConfig)
    setValidationErrors(validation.errors)

    if (!validation.isValid) {
      return
    }

    // Check for duplicate names
    if (configs.some(config => config.name === validation.sanitizedData!.name)) {
      setValidationErrors({ name: 'A configuration with this name already exists' })
      return
    }

    const apiConfig: TwitterAPIConfig = {
      id: Date.now().toString(),
      name: validation.sanitizedData!.name,
      api_key: validation.sanitizedData!.api_key,
      api_secret: validation.sanitizedData!.api_secret,
      access_token: validation.sanitizedData!.access_token,
      access_token_secret: validation.sanitizedData!.access_token_secret,
      is_connected: false,
      created_at: new Date().toISOString()
    }

    // Test the credentials
    try {
      const testResult = await twitterService.validateCredentials({
        api_key: newConfig.api_key,
        api_secret: newConfig.api_secret,
        access_token: newConfig.access_token,
        access_token_secret: newConfig.access_token_secret
      })
      
      if (testResult.valid) {
        apiConfig.is_connected = true
      }

      const updatedConfigs = [...configs, apiConfig]
      saveConfigs(updatedConfigs)

      // Reset form
      setNewConfig({ name: '', api_key: '', api_secret: '', access_token: '', access_token_secret: '' })
      setValidationErrors({})
      setIsAddingConfig(false)
      setLastSubmitTime(Date.now())

    } catch (error) {
      setValidationErrors({ general: 'Failed to validate credentials. Check your internet connection.' })
    }
  }

  const handleTestConfig = async (config: TwitterAPIConfig) => {
    setTestingConfig(config.id)
    setTestResults(prev => ({ ...prev, [config.id]: { success: false, message: 'Testing credentials...' } }))

    try {
      const testResult = await twitterService.validateCredentials({
        api_key: config.api_key,
        api_secret: config.api_secret,
        access_token: config.access_token,
        access_token_secret: config.access_token_secret
      })
      
      if (testResult.valid) {
        setTestResults(prev => ({
          ...prev,
          [config.id]: { success: true, message: '✅ Credentials validated successfully! Ready to post tweets.' }
        }))

        // Update config connection status
        const updatedConfigs = configs.map(c => 
          c.id === config.id 
            ? { 
                ...c, 
                is_connected: true, 
                last_tested: new Date().toISOString() 
              }
            : c
        )
        saveConfigs(updatedConfigs)

      } else {
        setTestResults(prev => ({
          ...prev,
          [config.id]: { success: false, message: `❌ ${testResult.error}` }
        }))
      }

    } catch (error) {
      setTestResults(prev => ({
        ...prev,
        [config.id]: { 
          success: false, 
          message: `❌ Network error: ${error instanceof Error ? error.message : 'Unknown error'}` 
        }
      }))
    } finally {
      setTestingConfig(null)
    }
  }

  const handleDeleteConfig = (configId: string) => {
    if (confirm('Delete this Twitter API configuration? This cannot be undone.')) {
      const updatedConfigs = configs.filter(config => config.id !== configId)
      saveConfigs(updatedConfigs)
      // Clean up test results
      setTestResults(prev => {
        const newResults = { ...prev }
        delete newResults[configId]
        return newResults
      })
    }
  }

  const maskCredential = (credential: string): string => {
    if (!credential || credential.length < 8) return '••••••••'
    return credential.slice(0, 4) + '••••••••' + credential.slice(-4)
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium text-gray-900">Twitter/X API</h3>
          <p className="text-sm text-gray-600">
            Configure Twitter API credentials for posting tweets and social media automation.
            <span className="text-amber-600 font-medium ml-1">Premium Feature</span>
          </p>
        </div>
        
        <button
          onClick={() => setIsAddingConfig(true)}
          className="px-4 py-2 bg-black text-white rounded-lg hover:bg-gray-800 transition-colors flex items-center space-x-2"
        >
          <span>+</span>
          <span>Add API Config</span>
        </button>
      </div>

      {/* Setup Instructions */}
      {configs.length === 0 && !isAddingConfig && (
        <div className="bg-black/5 border border-black/20 rounded-lg p-4">
          <h4 className="font-medium text-gray-900 mb-2">🚀 Get Started with Twitter API</h4>
          <ol className="text-sm text-gray-700 space-y-1 list-decimal list-inside">
            <li>Apply for Twitter API access at <a href="https://developer.twitter.com" target="_blank" className="text-blue-600 hover:underline">developer.twitter.com</a></li>
            <li>Create a new App in your Twitter Developer Dashboard</li>
            <li>Generate API Keys and Access Tokens</li>
            <li>Copy your credentials and add them below</li>
            <li>Test the connection to verify everything works</li>
          </ol>
          <div className="mt-3 p-3 bg-amber-50 border border-amber-200 rounded-lg">
            <p className="text-sm text-amber-800">
              <strong>⭐ Premium Feature:</strong> Twitter integration requires a DeFlow Premium subscription.
            </p>
          </div>
        </div>
      )}

      {/* Add Config Form */}
      {isAddingConfig && (
        <div className="bg-gray-50 border border-gray-200 rounded-lg p-6">
          <h4 className="font-medium text-gray-900 mb-4">Add Twitter API Configuration</h4>
          
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Configuration Name *
              </label>
              <input
                type="text"
                value={newConfig.name}
                onChange={(e) => setNewConfig({ ...newConfig, name: e.target.value })}
                placeholder="Main Account"
                className={`w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                  validationErrors.name ? 'border-red-500' : 'border-gray-300'
                }`}
              />
              {validationErrors.name && (
                <p className="text-red-600 text-sm mt-1">{validationErrors.name}</p>
              )}
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  API Key *
                </label>
                <input
                  type="password"
                  value={newConfig.api_key}
                  onChange={(e) => setNewConfig({ ...newConfig, api_key: e.target.value })}
                  placeholder="Your Twitter API Key"
                  className={`w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                    validationErrors.api_key ? 'border-red-500' : 'border-gray-300'
                  }`}
                />
                {validationErrors.api_key && (
                  <p className="text-red-600 text-sm mt-1">{validationErrors.api_key}</p>
                )}
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  API Secret *
                </label>
                <input
                  type="password"
                  value={newConfig.api_secret}
                  onChange={(e) => setNewConfig({ ...newConfig, api_secret: e.target.value })}
                  placeholder="Your Twitter API Secret"
                  className={`w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                    validationErrors.api_secret ? 'border-red-500' : 'border-gray-300'
                  }`}
                />
                {validationErrors.api_secret && (
                  <p className="text-red-600 text-sm mt-1">{validationErrors.api_secret}</p>
                )}
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Access Token *
                </label>
                <input
                  type="password"
                  value={newConfig.access_token}
                  onChange={(e) => setNewConfig({ ...newConfig, access_token: e.target.value })}
                  placeholder="Your Access Token"
                  className={`w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                    validationErrors.access_token ? 'border-red-500' : 'border-gray-300'
                  }`}
                />
                {validationErrors.access_token && (
                  <p className="text-red-600 text-sm mt-1">{validationErrors.access_token}</p>
                )}
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Access Token Secret *
                </label>
                <input
                  type="password"
                  value={newConfig.access_token_secret}
                  onChange={(e) => setNewConfig({ ...newConfig, access_token_secret: e.target.value })}
                  placeholder="Your Access Token Secret"
                  className={`w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                    validationErrors.access_token_secret ? 'border-red-500' : 'border-gray-300'
                  }`}
                />
                {validationErrors.access_token_secret && (
                  <p className="text-red-600 text-sm mt-1">{validationErrors.access_token_secret}</p>
                )}
              </div>
            </div>
          </div>

          <div className="mt-6 flex space-x-3">
            <button
              onClick={handleAddConfig}
              className="px-4 py-2 bg-black text-white rounded-lg hover:bg-gray-800 transition-colors"
            >
              Add Configuration
            </button>
            <button
              onClick={() => {
                setIsAddingConfig(false)
                setValidationErrors({})
                setNewConfig({ name: '', api_key: '', api_secret: '', access_token: '', access_token_secret: '' })
              }}
              className="px-4 py-2 bg-gray-300 text-gray-700 rounded-lg hover:bg-gray-400 transition-colors"
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {/* Configured APIs */}
      {configs.length > 0 && (
        <div className="space-y-4">
          {configs.map((config) => (
            <div key={config.id} className="bg-white border border-gray-200 rounded-lg p-6">
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center space-x-2 mb-2">
                    <h4 className="font-medium text-gray-900">{config.name}</h4>
                    <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                      config.is_connected 
                        ? 'bg-green-100 text-green-800'
                        : 'bg-yellow-100 text-yellow-800'
                    }`}>
                      {config.is_connected ? '✅ Connected' : '⚠️ Not Tested'}
                    </span>
                  </div>
                  
                  <div className="space-y-1 text-sm text-gray-600">
                    {config.account_name && (
                      <p>👤 Account: @{config.account_name}</p>
                    )}
                    <p>🔑 API Key: {maskCredential(config.api_key)}</p>
                    <p>🎫 Access Token: {maskCredential(config.access_token)}</p>
                    <p>📅 Added: {new Date(config.created_at).toLocaleDateString()}</p>
                    {config.last_tested && (
                      <p>🔍 Last tested: {new Date(config.last_tested).toLocaleString()}</p>
                    )}
                  </div>

                  {/* Test Results */}
                  {testResults[config.id] && (
                    <div className={`mt-3 p-3 rounded-lg text-sm ${
                      testResults[config.id].success 
                        ? 'bg-green-50 text-green-800 border border-green-200'
                        : 'bg-red-50 text-red-800 border border-red-200'
                    }`}>
                      {testResults[config.id].message}
                    </div>
                  )}
                </div>

                <div className="flex space-x-2 ml-4">
                  <button
                    onClick={() => handleTestConfig(config)}
                    disabled={testingConfig === config.id}
                    className="px-3 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {testingConfig === config.id ? '⏳ Testing...' : '🧪 Test'}
                  </button>
                  
                  <button
                    onClick={() => handleDeleteConfig(config.id)}
                    className="px-3 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
                  >
                    🗑️ Delete
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Usage Instructions */}
      {configs.length > 0 && (
        <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
          <h4 className="font-medium text-gray-900 mb-2">📝 Usage in Workflows</h4>
          <p className="text-sm text-gray-600 mb-2">
            Use your Twitter API configurations in workflow nodes:
          </p>
          <ol className="text-sm text-gray-600 space-y-1 list-decimal list-inside">
            <li>Create social media content with "Social Media Text" or "Social Media with Image" nodes</li>
            <li>Add a "Twitter/X Post" node to your workflow</li>
            <li>Enter your API credentials from the configurations above</li>
            <li>Connect your content formatter to the Twitter node</li>
            <li>Use template variables like {'{{portfolio_value}}'} for dynamic content</li>
            <li>Test your workflow to ensure tweets are posted successfully</li>
          </ol>
        </div>
      )}

      {/* API Information */}
      <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
        <h4 className="font-medium text-blue-900 mb-2">💡 Twitter API Information</h4>
        <div className="text-sm text-blue-800 space-y-1">
          <p><strong>API Version:</strong> Twitter API v2 with OAuth 1.0a authentication</p>
          <p><strong>Rate Limits:</strong> 300 tweets per 15 minutes, 15,000 DMs per day</p>
          <p><strong>Character Limit:</strong> 280 characters per tweet (automatically enforced)</p>
          <p><strong>Best for:</strong> Automated trading updates, portfolio alerts, community engagement</p>
        </div>
      </div>
    </div>
  )
}

export default TwitterAPISetup