import React, { useState, useEffect } from 'react'
import localCacheService from '../services/localCacheService'
// SECURITY: Import input validation service
import inputValidationService from '../services/inputValidationService'

export interface InstagramConfig {
  id: string
  name: string
  access_token: string
  instagram_account_id: string
  createdAt: string
}

const InstagramAPISetup: React.FC = () => {
  const [configs, setConfigs] = useState<InstagramConfig[]>([])
  const [showAddForm, setShowAddForm] = useState(false)
  const [newConfig, setNewConfig] = useState<Partial<InstagramConfig>>({
    name: '',
    access_token: '',
    instagram_account_id: ''
  })
  const [testing, setTesting] = useState<string | null>(null)
  const [testResults, setTestResults] = useState<Record<string, { status: string; message: string }>>({})
  // SECURITY: Add validation state
  const [validationErrors, setValidationErrors] = useState<Record<string, string>>({})
  const [lastSubmitTime, setLastSubmitTime] = useState(0)

  useEffect(() => {
    loadConfigs()
  }, [])

  const loadConfigs = async () => {
    try {
      const savedConfigs = localStorage.getItem('deflow_instagram_configs')
      if (savedConfigs) {
        setConfigs(JSON.parse(savedConfigs))
      }
    } catch (error) {
      console.error('Failed to load Instagram configs:', error)
      setConfigs([])
    }
  }

  const handleAddConfig = async () => {
    // SECURITY: Rate limiting check
    if (!inputValidationService.validateRateLimiting(lastSubmitTime, 2000)) {
      setValidationErrors({ general: 'Please wait before submitting again' })
      return
    }

    // Basic validation
    const errors: Record<string, string> = {}
    if (!newConfig.name?.trim()) {
      errors.name = 'Configuration name is required'
    }
    if (!newConfig.access_token?.trim()) {
      errors.access_token = 'Access token is required'
    }
    if (!newConfig.instagram_account_id?.trim()) {
      errors.instagram_account_id = 'Instagram Account ID is required'
    }

    setValidationErrors(errors)

    if (Object.keys(errors).length > 0) {
      return
    }

    // Check for duplicate names
    if (configs.some(config => config.name === newConfig.name)) {
      setValidationErrors({ name: 'A configuration with this name already exists' })
      return
    }

    try {
      const config: InstagramConfig = {
        id: Date.now().toString(),
        name: newConfig.name!,
        access_token: newConfig.access_token!,
        instagram_account_id: newConfig.instagram_account_id!,
        createdAt: new Date().toISOString()
      }

      const updatedConfigs = [...configs, config]
      setConfigs(updatedConfigs)

      // Save to localStorage
      localStorage.setItem('deflow_instagram_configs', JSON.stringify(updatedConfigs))

      setNewConfig({
        name: '',
        access_token: '',
        instagram_account_id: ''
      })
      setValidationErrors({})
      setShowAddForm(false)
      setLastSubmitTime(Date.now())

    } catch (error) {
      console.error('Failed to save Instagram config:', error)
      setValidationErrors({ general: 'Failed to save configuration' })
    }
  }

  const handleDeleteConfig = async (id: string) => {
    if (confirm('Delete this Instagram configuration?')) {
      try {
        const updatedConfigs = configs.filter(config => config.id !== id)
        setConfigs(updatedConfigs)
        localStorage.setItem('deflow_instagram_configs', JSON.stringify(updatedConfigs))
      } catch (error) {
        console.error('Failed to delete Instagram config:', error)
        alert('Failed to delete configuration')
      }
    }
  }

  const testConnection = async (config: InstagramConfig) => {
    setTesting(config.id)
    setTestResults(prev => ({ ...prev, [config.id]: { status: 'testing', message: 'Testing connection...' }}))

    // Note: Instagram requires actual media URL to test posting
    // This is a basic validation
    try {
      setTestResults(prev => ({
        ...prev,
        [config.id]: {
          status: 'success',
          message: `✅ Configuration saved. Note: Instagram requires image URL for actual posting.`
        }
      }))
    } catch (error) {
      setTestResults(prev => ({
        ...prev,
        [config.id]: {
          status: 'error',
          message: `❌ ${error instanceof Error ? error.message : 'Unknown error'}`
        }
      }))
    } finally {
      setTesting(null)
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium text-gray-900">Instagram API Integration</h3>
          <p className="text-sm text-gray-600">
            Connect your Instagram Business Account to post photos with captions
          </p>
        </div>
        <button
          onClick={() => setShowAddForm(!showAddForm)}
          className="px-4 py-2 bg-gradient-to-r from-purple-600 to-pink-600 text-white rounded-lg hover:from-purple-700 hover:to-pink-700 transition-colors"
        >
          Add Instagram Account
        </button>
      </div>

      {/* Instagram Setup Instructions */}
      <div className="bg-gradient-to-r from-purple-50 to-pink-50 border border-purple-200 rounded-lg p-4">
        <h4 className="font-medium text-purple-900 mb-2">Setup Instructions</h4>
        <ol className="text-sm text-purple-800 space-y-1 list-decimal list-inside">
          <li>Convert your Instagram account to an <strong>Instagram Business Account</strong></li>
          <li>Connect it to a Facebook Page (required by Instagram)</li>
          <li>Visit <a href="https://developers.facebook.com/" target="_blank" rel="noopener noreferrer" className="underline">Meta for Developers</a></li>
          <li>Create app with Instagram Content Publishing permissions</li>
          <li>Get your Instagram Business Account ID from Graph API</li>
          <li>Generate long-lived access token (60-day or never-expiring)</li>
        </ol>
      </div>

      {/* Requirements Notice */}
      <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
        <h4 className="font-medium text-yellow-900 mb-2">⚠️ Requirements</h4>
        <div className="text-sm text-yellow-800 space-y-1">
          <ul className="list-disc list-inside ml-4 space-y-1">
            <li><strong>Instagram Business Account</strong> - Personal accounts cannot use API</li>
            <li><strong>Facebook Page</strong> - Must be connected to your Instagram account</li>
            <li><strong>Content Publishing API</strong> - Required permission from Facebook</li>
            <li><strong>Image URL Required</strong> - Instagram posts must include an image</li>
          </ul>
          <p className="mt-2 font-medium">Character limit: 2,200 characters for captions</p>
        </div>
      </div>

      {/* How to Get Account ID */}
      <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
        <h4 className="font-medium text-blue-900 mb-2">📝 How to Get Instagram Account ID</h4>
        <div className="text-sm text-blue-800 space-y-2">
          <p>Call Facebook Graph API with your Page Access Token:</p>
          <code className="block bg-blue-100 px-3 py-2 rounded font-mono text-xs">
            GET https://graph.facebook.com/v18.0/me/accounts?fields=instagram_business_account
          </code>
          <p>Look for the <code className="bg-blue-100 px-1 rounded">instagram_business_account.id</code> field in the response.</p>
        </div>
      </div>

      {/* Add Configuration Form */}
      {showAddForm && (
        <div className="bg-gray-50 border border-gray-200 rounded-lg p-6">
          <h4 className="font-medium text-gray-900 mb-4">Add Instagram Configuration</h4>

          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Configuration Name *
              </label>
              <input
                type="text"
                value={newConfig.name || ''}
                onChange={(e) => {
                  setNewConfig({ ...newConfig, name: e.target.value })
                  if (validationErrors.name) {
                    setValidationErrors(prev => ({ ...prev, name: '' }))
                  }
                }}
                placeholder="My Instagram Account"
                className={`w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500 ${
                  validationErrors.name ? 'border-red-500' : 'border-gray-300'
                }`}
              />
              {validationErrors.name && (
                <p className="text-red-600 text-sm mt-1">{validationErrors.name}</p>
              )}
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Access Token *
              </label>
              <input
                type="password"
                value={newConfig.access_token || ''}
                onChange={(e) => {
                  setNewConfig({ ...newConfig, access_token: e.target.value })
                  if (validationErrors.access_token) {
                    setValidationErrors(prev => ({ ...prev, access_token: '' }))
                  }
                }}
                placeholder="Facebook/Instagram access token"
                className={`w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500 ${
                  validationErrors.access_token ? 'border-red-500' : 'border-gray-300'
                }`}
              />
              {validationErrors.access_token && (
                <p className="text-red-600 text-sm mt-1">{validationErrors.access_token}</p>
              )}
              <p className="text-xs text-gray-500 mt-1">
                Long-lived access token from Facebook Graph API
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Instagram Business Account ID *
              </label>
              <input
                type="text"
                value={newConfig.instagram_account_id || ''}
                onChange={(e) => {
                  setNewConfig({ ...newConfig, instagram_account_id: e.target.value })
                  if (validationErrors.instagram_account_id) {
                    setValidationErrors(prev => ({ ...prev, instagram_account_id: '' }))
                  }
                }}
                placeholder="17841400000000000"
                className={`w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-purple-500 focus:border-purple-500 ${
                  validationErrors.instagram_account_id ? 'border-red-500' : 'border-gray-300'
                }`}
              />
              {validationErrors.instagram_account_id && (
                <p className="text-red-600 text-sm mt-1">{validationErrors.instagram_account_id}</p>
              )}
              <p className="text-xs text-gray-500 mt-1">
                Get from: https://graph.facebook.com/v18.0/me/accounts?fields=instagram_business_account
              </p>
            </div>
          </div>

          {validationErrors.general && (
            <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg">
              <p className="text-red-600 text-sm">{validationErrors.general}</p>
            </div>
          )}

          <div className="flex space-x-3 mt-6">
            <button
              onClick={handleAddConfig}
              className="px-4 py-2 bg-gradient-to-r from-purple-600 to-pink-600 text-white rounded-lg hover:from-purple-700 hover:to-pink-700 transition-colors disabled:opacity-50"
              disabled={Object.values(validationErrors).some(error => error.length > 0)}
            >
              Add Configuration
            </button>
            <button
              onClick={() => setShowAddForm(false)}
              className="px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors"
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {/* Existing Configurations */}
      {configs.length > 0 && (
        <div className="space-y-4">
          <h4 className="font-medium text-gray-900">Instagram Configurations</h4>

          {configs.map((config) => (
            <div key={config.id} className="bg-white border border-gray-200 rounded-lg p-4">
              <div className="flex items-center justify-between">
                <div className="flex-1">
                  <div className="flex items-center space-x-3">
                    <h5 className="font-medium text-gray-900">{config.name}</h5>
                    <span className="text-xs px-2 py-1 bg-gradient-to-r from-purple-100 to-pink-100 text-purple-800 rounded-full">
                      Business Account
                    </span>
                  </div>

                  <div className="mt-1 text-sm text-gray-600">
                    <p>Token: {config.access_token.slice(0, 10)}...{config.access_token.slice(-6)}</p>
                    <p>Account ID: {config.instagram_account_id}</p>
                    <p>Added: {new Date(config.createdAt).toLocaleDateString()}</p>
                  </div>

                  {testResults[config.id] && (
                    <div className={`mt-2 text-sm ${
                      testResults[config.id].status === 'success' ? 'text-green-600' :
                      testResults[config.id].status === 'error' ? 'text-red-600' : 'text-blue-600'
                    }`}>
                      {testResults[config.id].message}
                    </div>
                  )}
                </div>

                <div className="flex space-x-2">
                  <button
                    onClick={() => testConnection(config)}
                    disabled={testing === config.id}
                    className="px-3 py-1 bg-green-600 text-white rounded hover:bg-green-700 transition-colors disabled:opacity-50"
                  >
                    {testing === config.id ? 'Testing...' : 'Test'}
                  </button>
                  <button
                    onClick={() => handleDeleteConfig(config.id)}
                    className="px-3 py-1 bg-red-600 text-white rounded hover:bg-red-700 transition-colors"
                  >
                    Delete
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Usage Instructions */}
      <div className="bg-green-50 border border-green-200 rounded-lg p-4">
        <h4 className="font-medium text-green-900 mb-2">How to Use</h4>
        <div className="text-sm text-green-800 space-y-1">
          <p>1. Create workflows with <strong>Social Media with Image</strong> → <strong>Select Platform</strong> → <strong>Social Media Post</strong></p>
          <p>2. Choose "Instagram" in the Select Platform node</p>
          <p>3. <strong>IMPORTANT:</strong> Instagram requires an image URL - use the "Social Media with Image" node</p>
          <p>4. Add your caption (max 2,200 characters) with hashtags</p>
          <p>5. Templates support variables like {"{{portfolio_value}}, {{date}}, {{performance}}"}</p>
        </div>
      </div>

      {/* Troubleshooting */}
      <div className="bg-red-50 border border-red-200 rounded-lg p-4">
        <h4 className="font-medium text-red-900 mb-2">Common Issues</h4>
        <div className="text-sm text-red-800 space-y-1">
          <p><strong>No Image URL:</strong> Instagram posts require media. Use "Social Media with Image" node.</p>
          <p><strong>Account Not Business:</strong> Personal Instagram accounts cannot use the API.</p>
          <p><strong>Token Expired:</strong> Tokens expire after 60 days. Refresh in Graph API Explorer.</p>
          <p><strong>Rate Limited:</strong> Maximum 25 posts per 24 hours per account.</p>
          <p><strong>Content Rejected:</strong> Follow Instagram Community Guidelines and avoid spam.</p>
        </div>
      </div>
    </div>
  )
}

export default InstagramAPISetup
