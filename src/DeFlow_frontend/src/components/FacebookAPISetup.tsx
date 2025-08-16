import React, { useState, useEffect } from 'react'
import localCacheService, { FacebookConfig } from '../services/localCacheService'
import facebookService, { FacebookCredentials } from '../services/facebookService'

const FacebookAPISetup: React.FC = () => {
  const [configs, setConfigs] = useState<FacebookConfig[]>([])
  const [showAddForm, setShowAddForm] = useState(false)
  const [newConfig, setNewConfig] = useState<Partial<FacebookConfig>>({
    name: '',
    access_token: '',
    page_id: '',
    post_type: 'page'
  })
  const [testing, setTesting] = useState<string | null>(null)
  const [testResults, setTestResults] = useState<Record<string, { status: string; message: string }>>({})

  useEffect(() => {
    loadConfigs()
  }, [])

  const loadConfigs = () => {
    const savedConfigs = localCacheService.getFacebookConfigs()
    setConfigs(savedConfigs)
  }

  const handleAddConfig = async () => {
    if (!newConfig.name || !newConfig.access_token || !newConfig.page_id) {
      alert('Please fill in all required fields')
      return
    }

    const config: FacebookConfig = {
      id: Date.now().toString(),
      name: newConfig.name,
      access_token: newConfig.access_token,
      page_id: newConfig.page_id,
      post_type: newConfig.post_type || 'page',
      createdAt: new Date().toISOString()
    }

    const updatedConfigs = [...configs, config]
    setConfigs(updatedConfigs)
    localCacheService.saveFacebookConfigs(updatedConfigs)
    
    setNewConfig({
      name: '',
      access_token: '',
      page_id: '',
      post_type: 'page'
    })
    setShowAddForm(false)
  }

  const handleDeleteConfig = (id: string) => {
    if (confirm('Delete this Facebook configuration?')) {
      const updatedConfigs = configs.filter(config => config.id !== id)
      setConfigs(updatedConfigs)
      localCacheService.saveFacebookConfigs(updatedConfigs)
    }
  }

  const testConnection = async (config: FacebookConfig) => {
    setTesting(config.id)
    setTestResults(prev => ({ ...prev, [config.id]: { status: 'testing', message: 'Testing connection...' }}))

    try {
      const credentials: FacebookCredentials = {
        access_token: config.access_token,
        page_id: config.page_id
      }

      const validation = await facebookService.validateCredentials(credentials, config.page_id)
      
      if (validation.valid) {
        // Get page info for additional details
        const pageInfo = await facebookService.getPageInfo(credentials, config.page_id)
        const displayName = pageInfo ? pageInfo.name : 'Unknown Page'
        const followers = pageInfo ? ` (${pageInfo.followers_count || 0} followers)` : ''
        
        setTestResults(prev => ({ 
          ...prev, 
          [config.id]: { 
            status: 'success', 
            message: `✅ Connected to ${displayName}${followers}` 
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

  const getFacebookAuthUrl = () => {
    const appId = 'your-facebook-app-id' // This should come from env or config
    const redirectUri = `${window.location.origin}/auth/facebook/callback`
    return facebookService.getAuthorizationUrl(appId, redirectUri)
  }

  const openFacebookAuth = () => {
    const authUrl = getFacebookAuthUrl()
    window.open(authUrl, '_blank', 'width=600,height=700')
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium text-gray-900">Facebook API Integration</h3>
          <p className="text-sm text-gray-600">
            Connect your Facebook business pages to post content and engage with your community
          </p>
        </div>
        <button
          onClick={() => setShowAddForm(!showAddForm)}
          className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        >
          Add Facebook Page
        </button>
      </div>

      {/* Facebook App Setup Instructions */}
      <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
        <h4 className="font-medium text-blue-900 mb-2">Setup Instructions</h4>
        <ol className="text-sm text-blue-800 space-y-1 list-decimal list-inside">
          <li>Visit <a href="https://developers.facebook.com/" target="_blank" rel="noopener noreferrer" className="underline">Meta for Developers</a></li>
          <li>Create a new app with "Business" type</li>
          <li>Add "Facebook Login" and "Pages API" products</li>
          <li>Set redirect URI: <code className="bg-blue-100 px-1 rounded">{window.location.origin}/auth/facebook/callback</code></li>
          <li>Submit app for review to get full permissions</li>
          <li>Use Graph API Explorer to get page access tokens</li>
        </ol>
      </div>

      {/* App Review Notice */}
      <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
        <h4 className="font-medium text-yellow-900 mb-2">⚠️ App Review Required</h4>
        <div className="text-sm text-yellow-800 space-y-1">
          <p>Facebook requires app review for posting permissions:</p>
          <ul className="list-disc list-inside ml-4 space-y-1">
            <li><code>pages_manage_posts</code> - Required to post to pages</li>
            <li><code>pages_read_engagement</code> - Read page insights</li>
            <li><code>publish_to_groups</code> - Post to groups (if needed)</li>
          </ul>
          <p className="mt-2">During development, you can test with pages you admin without approval.</p>
        </div>
      </div>

      {/* OAuth Authorization Button */}
      <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
        <div className="flex items-center justify-between">
          <div>
            <h4 className="font-medium text-gray-900">OAuth Authorization</h4>
            <p className="text-sm text-gray-600">Recommended: Use OAuth to safely connect your Facebook page</p>
          </div>
          <button
            onClick={openFacebookAuth}
            className="px-4 py-2 bg-[#1877F2] text-white rounded-lg hover:bg-[#166FE5] transition-colors flex items-center space-x-2"
          >
            <span>📘</span>
            <span>Connect Facebook</span>
          </button>
        </div>
      </div>

      {/* Add Configuration Form */}
      {showAddForm && (
        <div className="bg-gray-50 border border-gray-200 rounded-lg p-6">
          <h4 className="font-medium text-gray-900 mb-4">Add Facebook Configuration</h4>
          
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Configuration Name *
              </label>
              <input
                type="text"
                value={newConfig.name || ''}
                onChange={(e) => setNewConfig({ ...newConfig, name: e.target.value })}
                placeholder="My Business Page"
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Page Access Token *
              </label>
              <input
                type="password"
                value={newConfig.access_token || ''}
                onChange={(e) => setNewConfig({ ...newConfig, access_token: e.target.value })}
                placeholder="Facebook Page access token (long-lived)"
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              />
              <p className="text-xs text-gray-500 mt-1">
                Long-lived page access token from Graph API Explorer
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Page ID *
              </label>
              <input
                type="text"
                value={newConfig.page_id || ''}
                onChange={(e) => setNewConfig({ ...newConfig, page_id: e.target.value })}
                placeholder="1234567890"
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              />
              <p className="text-xs text-gray-500 mt-1">
                Facebook Page ID (found in Page settings)
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Post Type
              </label>
              <select
                value={newConfig.post_type || 'page'}
                onChange={(e) => setNewConfig({ ...newConfig, post_type: e.target.value as 'page' | 'group' | 'event' })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              >
                <option value="page">Page Posts</option>
                <option value="group">Group Posts</option>
                <option value="event">Event Updates</option>
              </select>
            </div>
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
          <h4 className="font-medium text-gray-900">Facebook Configurations</h4>
          
          {configs.map((config) => (
            <div key={config.id} className="bg-white border border-gray-200 rounded-lg p-4">
              <div className="flex items-center justify-between">
                <div className="flex-1">
                  <div className="flex items-center space-x-3">
                    <h5 className="font-medium text-gray-900">{config.name}</h5>
                    <span className="text-xs px-2 py-1 bg-blue-100 text-blue-800 rounded-full">
                      {config.post_type.charAt(0).toUpperCase() + config.post_type.slice(1)}
                    </span>
                  </div>
                  
                  <div className="mt-1 text-sm text-gray-600">
                    <p>Token: {config.access_token.slice(0, 10)}...{config.access_token.slice(-6)}</p>
                    <p>Page ID: {config.page_id}</p>
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
          <p>1. Create workflows with <strong>Social Media Text</strong> → <strong>Facebook Post</strong></p>
          <p>2. Set platform to "Facebook (63,206 chars)" in Social Media Text node</p>
          <p>3. Use engaging content with calls-to-action and relevant hashtags</p>
          <p>4. Templates support variables like {"{{business_update}}, {{date}}, {{metrics}}"}</p>
          <p>5. Consider posting timing for maximum engagement</p>
        </div>
      </div>

      {/* Troubleshooting */}
      <div className="bg-red-50 border border-red-200 rounded-lg p-4">
        <h4 className="font-medium text-red-900 mb-2">Common Issues</h4>
        <div className="text-sm text-red-800 space-y-1">
          <p><strong>Token Invalid:</strong> Tokens expire after 60 days. Refresh in Graph API Explorer.</p>
          <p><strong>Insufficient Permissions:</strong> App needs review for <code>pages_manage_posts</code>.</p>
          <p><strong>Content Blocked:</strong> Avoid spam-like behavior and follow Community Standards.</p>
          <p><strong>Rate Limited:</strong> Reduce posting frequency (max 200/hour per page).</p>
        </div>
      </div>
    </div>
  )
}

export default FacebookAPISetup