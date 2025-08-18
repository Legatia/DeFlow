import React, { useState, useEffect } from 'react'
import localCacheService, { LinkedInConfig } from '../services/localCacheService'
import linkedinService, { LinkedInCredentials } from '../services/linkedinService'
// SECURITY: Import input validation service
import inputValidationService from '../services/inputValidationService'

const LinkedInAPISetup: React.FC = () => {
  const [configs, setConfigs] = useState<LinkedInConfig[]>([])
  const [showAddForm, setShowAddForm] = useState(false)
  const [newConfig, setNewConfig] = useState<Partial<LinkedInConfig>>({
    name: '',
    access_token: '',
    post_type: 'person',
    organization_id: ''
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
      // SECURITY: Use encrypted storage for sensitive configs
      const savedConfigs = await localCacheService.getLinkedInConfigs()
      setConfigs(savedConfigs)
    } catch (error) {
      console.error('SECURITY: Failed to load LinkedIn configs:', error)
      setConfigs([])
    }
  }

  const handleAddConfig = async () => {
    // SECURITY: Rate limiting check
    if (!inputValidationService.validateRateLimiting(lastSubmitTime, 2000)) {
      setValidationErrors({ general: 'Please wait before submitting again' })
      return
    }

    // SECURITY: Comprehensive validation
    const validation = inputValidationService.validateLinkedInCredentials({
      name: newConfig.name || '',
      access_token: newConfig.access_token || '',
      post_type: newConfig.post_type || 'person',
      organization_id: newConfig.organization_id
    })

    setValidationErrors(validation.errors)

    if (!validation.isValid) {
      return
    }

    // Check for duplicate names
    if (configs.some(config => config.name === validation.sanitizedData!.name)) {
      setValidationErrors({ name: 'A configuration with this name already exists' })
      return
    }

    try {
      const config: LinkedInConfig = {
        id: Date.now().toString(),
        name: validation.sanitizedData!.name,
        access_token: validation.sanitizedData!.access_token,
        post_type: validation.sanitizedData!.post_type,
        organization_id: validation.sanitizedData!.organization_id || '',
        createdAt: new Date().toISOString()
      }

      const updatedConfigs = [...configs, config]
      setConfigs(updatedConfigs)
      
      // SECURITY: Save to encrypted storage
      const success = await localCacheService.saveLinkedInConfigs(updatedConfigs)
      if (!success) {
        setValidationErrors({ general: 'Failed to save configuration securely' })
        return
      }
      
      setNewConfig({
        name: '',
        access_token: '',
        post_type: 'person',
        organization_id: ''
      })
      setValidationErrors({})
      setShowAddForm(false)
      setLastSubmitTime(Date.now())
      
    } catch (error) {
      console.error('SECURITY: Failed to save LinkedIn config:', error)
      setValidationErrors({ general: 'Failed to save configuration' })
    }
  }

  const handleDeleteConfig = async (id: string) => {
    if (confirm('Delete this LinkedIn configuration?')) {
      try {
        const updatedConfigs = configs.filter(config => config.id !== id)
        setConfigs(updatedConfigs)
        // SECURITY: Save to encrypted storage
        await localCacheService.saveLinkedInConfigs(updatedConfigs)
      } catch (error) {
        console.error('SECURITY: Failed to delete LinkedIn config:', error)
        alert('Failed to delete configuration')
      }
    }
  }

  const testConnection = async (config: LinkedInConfig) => {
    setTesting(config.id)
    setTestResults(prev => ({ ...prev, [config.id]: { status: 'testing', message: 'Testing connection...' }}))

    try {
      const credentials: LinkedInCredentials = {
        access_token: config.access_token
      }

      const validation = await linkedinService.validateCredentials(credentials)
      
      if (validation.valid) {
        // Get user profile for additional info
        const profile = await linkedinService.getUserProfile(credentials)
        const displayName = profile ? `${profile.firstName} ${profile.lastName}` : 'Unknown User'
        
        setTestResults(prev => ({ 
          ...prev, 
          [config.id]: { 
            status: 'success', 
            message: `✅ Connected as ${displayName}` 
          }
        }))
      } else {
        setTestResults(prev => ({ 
          ...prev, 
          [config.id]: { 
            status: 'error', 
            message: `❌ ${validation.error || 'Connection failed'}` 
          }
        }))
      }
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

  const getLinkedInAuthUrl = () => {
    const clientId = 'your-linkedin-client-id' // This should come from env or config
    const redirectUri = `${window.location.origin}/auth/linkedin/callback`
    return linkedinService.getAuthorizationUrl(clientId, redirectUri)
  }

  const openLinkedInAuth = () => {
    const authUrl = getLinkedInAuthUrl()
    window.open(authUrl, '_blank', 'width=600,height=700')
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium text-gray-900">LinkedIn API Integration</h3>
          <p className="text-sm text-gray-600">
            Connect your LinkedIn account to post professional content and updates
          </p>
        </div>
        <button
          onClick={() => setShowAddForm(!showAddForm)}
          className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        >
          Add LinkedIn Account
        </button>
      </div>

      {/* LinkedIn OAuth Instructions */}
      <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
        <h4 className="font-medium text-blue-900 mb-2">Quick Setup Instructions</h4>
        <ol className="text-sm text-blue-800 space-y-1 list-decimal list-inside">
          <li>Visit <a href="https://developer.linkedin.com/" target="_blank" rel="noopener noreferrer" className="underline">LinkedIn Developer Portal</a></li>
          <li>Create a new app or use existing one</li>
          <li>Add "Share on LinkedIn" and "Sign In with LinkedIn" products</li>
          <li>Set redirect URI: <code className="bg-blue-100 px-1 rounded">{window.location.origin}/auth/linkedin/callback</code></li>
          <li>Use the OAuth flow below or manually enter your access token</li>
        </ol>
      </div>

      {/* OAuth Authorization Button */}
      <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
        <div className="flex items-center justify-between">
          <div>
            <h4 className="font-medium text-gray-900">OAuth Authorization</h4>
            <p className="text-sm text-gray-600">Recommended: Use OAuth to safely connect your LinkedIn account</p>
          </div>
          <button
            onClick={openLinkedInAuth}
            className="px-4 py-2 bg-[#0077B5] text-white rounded-lg hover:bg-[#005582] transition-colors flex items-center space-x-2"
          >
            <span>💼</span>
            <span>Connect LinkedIn</span>
          </button>
        </div>
      </div>

      {/* Add Configuration Form */}
      {showAddForm && (
        <div className="bg-gray-50 border border-gray-200 rounded-lg p-6">
          <h4 className="font-medium text-gray-900 mb-4">Add LinkedIn Configuration</h4>
          
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Configuration Name *
              </label>
              <input
                type="text"
                value={newConfig.name || ''}
                onChange={(e) => setNewConfig({ ...newConfig, name: e.target.value })}
                placeholder="My LinkedIn Account"
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Access Token *
              </label>
              <input
                type="password"
                value={newConfig.access_token || ''}
                onChange={(e) => setNewConfig({ ...newConfig, access_token: e.target.value })}
                placeholder="LinkedIn OAuth 2.0 access token"
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              />
              <p className="text-xs text-gray-500 mt-1">
                Long-lived access token from LinkedIn OAuth flow
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Post Type
              </label>
              <select
                value={newConfig.post_type || 'person'}
                onChange={(e) => setNewConfig({ ...newConfig, post_type: e.target.value as 'person' | 'organization' })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              >
                <option value="person">Personal Posts</option>
                <option value="organization">Company Page Posts</option>
              </select>
            </div>

            {newConfig.post_type === 'organization' && (
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Organization ID
                </label>
                <input
                  type="text"
                  value={newConfig.organization_id || ''}
                  onChange={(e) => setNewConfig({ ...newConfig, organization_id: e.target.value })}
                  placeholder="LinkedIn organization ID"
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                />
                <p className="text-xs text-gray-500 mt-1">
                  Required for company page posts
                </p>
              </div>
            )}
          </div>

          <div className="flex space-x-3 mt-6">
            <button
              onClick={handleAddConfig}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
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
          <h4 className="font-medium text-gray-900">LinkedIn Configurations</h4>
          
          {configs.map((config) => (
            <div key={config.id} className="bg-white border border-gray-200 rounded-lg p-4">
              <div className="flex items-center justify-between">
                <div className="flex-1">
                  <div className="flex items-center space-x-3">
                    <h5 className="font-medium text-gray-900">{config.name}</h5>
                    <span className="text-xs px-2 py-1 bg-blue-100 text-blue-800 rounded-full">
                      {config.post_type === 'person' ? 'Personal' : 'Company'}
                    </span>
                  </div>
                  
                  <div className="mt-1 text-sm text-gray-600">
                    <p>Token: {config.access_token.slice(0, 10)}...{config.access_token.slice(-6)}</p>
                    {config.organization_id && (
                      <p>Organization ID: {config.organization_id}</p>
                    )}
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
          <p>1. Create workflows with <strong>Social Media Text</strong> → <strong>LinkedIn Post</strong></p>
          <p>2. Set platform to "LinkedIn (3000 chars)" in Social Media Text node</p>
          <p>3. Use professional tone and relevant hashtags (#DeFi #Business)</p>
          <p>4. Templates support variables like {"{{portfolio_value}}, {{date}}, {{strategy}}"}</p>
        </div>
      </div>
    </div>
  )
}

export default LinkedInAPISetup