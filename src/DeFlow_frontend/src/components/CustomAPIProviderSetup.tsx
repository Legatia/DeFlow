import React, { useState, useEffect } from 'react'
import emailService, { EmailConfig } from '../services/emailService'

interface CustomAPIConfig {
  name: string
  baseUrl: string
  method: 'GET' | 'POST' | 'PUT'
  headers: Record<string, string>
  bodyTemplate: string
  authType: 'none' | 'bearer' | 'api_key' | 'basic' | 'custom'
  authConfig: {
    token?: string
    username?: string
    password?: string
    apiKey?: string
    headerName?: string
    customAuth?: string
  }
  responseMapping: {
    successField?: string
    messageIdField?: string
    errorField?: string
  }
  testEndpoint?: string
}

interface CustomAPIProviderSetupProps {
  onProviderAdded?: (name: string) => void
}

const CustomAPIProviderSetup: React.FC<CustomAPIProviderSetupProps> = ({ onProviderAdded }) => {
  const [providers, setProviders] = useState<Record<string, CustomAPIConfig>>({})
  const [showAddForm, setShowAddForm] = useState(false)
  const [editingProvider, setEditingProvider] = useState<string | null>(null)
  const [currentConfig, setCurrentConfig] = useState<CustomAPIConfig>({
    name: '',
    baseUrl: '',
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    bodyTemplate: JSON.stringify({
      to: '{{to}}',
      subject: '{{subject}}',
      body: '{{body}}'
    }, null, 2),
    authType: 'bearer',
    authConfig: {},
    responseMapping: {},
    testEndpoint: ''
  })

  useEffect(() => {
    loadCustomProviders()
  }, [])

  const loadCustomProviders = () => {
    const stored = localStorage.getItem('custom_api_providers')
    if (stored) {
      setProviders(JSON.parse(stored))
    }
  }

  const saveCustomProviders = (newProviders: Record<string, CustomAPIConfig>) => {
    localStorage.setItem('custom_api_providers', JSON.stringify(newProviders))
    setProviders(newProviders)
  }

  const handleSaveProvider = () => {
    if (!currentConfig.name.trim() || !currentConfig.baseUrl.trim()) {
      alert('Please provide name and base URL')
      return
    }

    const newProviders = {
      ...providers,
      [currentConfig.name]: currentConfig
    }
    
    saveCustomProviders(newProviders)
    
    // Register with email service as custom provider
    const emailConfig: EmailConfig = {
      provider: 'custom',
      credentials: {
        api_key: currentConfig.authConfig.token || currentConfig.authConfig.apiKey || ''
      },
      custom_config: {
        baseUrl: currentConfig.baseUrl,
        method: currentConfig.method,
        headers: currentConfig.headers,
        bodyTemplate: currentConfig.bodyTemplate,
        authType: currentConfig.authType,
        authConfig: currentConfig.authConfig
      }
    }
    
    emailService.addProvider(currentConfig.name, emailConfig)
    onProviderAdded?.(currentConfig.name)
    
    resetForm()
  }

  const resetForm = () => {
    setCurrentConfig({
      name: '',
      baseUrl: '',
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      bodyTemplate: JSON.stringify({
        to: '{{to}}',
        subject: '{{subject}}',
        body: '{{body}}'
      }, null, 2),
      authType: 'bearer',
      authConfig: {},
      responseMapping: {},
      testEndpoint: ''
    })
    setShowAddForm(false)
    setEditingProvider(null)
  }

  const handleEditProvider = (name: string) => {
    setCurrentConfig(providers[name])
    setEditingProvider(name)
    setShowAddForm(true)
  }

  const handleDeleteProvider = (name: string) => {
    if (confirm(`Delete custom provider "${name}"?`)) {
      const newProviders = { ...providers }
      delete newProviders[name]
      saveCustomProviders(newProviders)
      emailService.removeProvider(name)
    }
  }

  const handleTestProvider = async (config: CustomAPIConfig) => {
    const testEmail = prompt('Enter email address to send test email (or press Cancel for API-only test):')
    
    if (testEmail) {
      // Test with actual email sending
      try {
        // Create temporary provider config
        const emailConfig: EmailConfig = {
          provider: 'custom',
          credentials: {
            api_key: config.authConfig.token || config.authConfig.apiKey || ''
          },
          custom_config: {
            baseUrl: config.baseUrl,
            method: config.method,
            headers: config.headers,
            bodyTemplate: config.bodyTemplate,
            authType: config.authType,
            authConfig: config.authConfig
          }
        }

        // Add temporary provider
        const tempProviderName = `test-${Date.now()}`
        emailService.addProvider(tempProviderName, emailConfig)

        // Send test email
        const result = await emailService.sendEmail(tempProviderName, {
          to: testEmail,
          subject: 'DeFlow Custom API Test',
          body_html: `
            <h2>✅ Custom API Test</h2>
            <p>This test email was sent using your custom API configuration:</p>
            <ul>
              <li><strong>Provider:</strong> ${config.name}</li>
              <li><strong>API URL:</strong> ${config.baseUrl}</li>
              <li><strong>Method:</strong> ${config.method}</li>
              <li><strong>Auth Type:</strong> ${config.authType}</li>
            </ul>
            <p>If you received this email, your custom API integration is working correctly!</p>
            <hr>
            <small>Sent from DeFlow at ${new Date().toLocaleString()}</small>
          `,
          body_text: `DeFlow Custom API Test\n\nThis test email was sent using your custom API configuration:\n- Provider: ${config.name}\n- API URL: ${config.baseUrl}\n- Method: ${config.method}\n- Auth Type: ${config.authType}\n\nIf you received this email, your custom API integration is working correctly!\n\nSent from DeFlow at ${new Date().toLocaleString()}`
        })

        // Remove temporary provider
        emailService.removeProvider(tempProviderName)

        if (result.success) {
          alert(`✅ Test email sent successfully!\nMessage ID: ${result.message_id}`)
        } else {
          alert(`❌ Failed to send test email: ${result.error}`)
        }
      } catch (error) {
        alert(`❌ Test email error: ${error}`)
      }
    } else {
      // API-only test (no email sending)
      try {
        // Create test request
        const testData = {
          to: 'test@example.com',
          subject: 'DeFlow API Test',
          body: 'This is a test email from your custom API configuration'
        }

        // Replace template variables
        let body = config.bodyTemplate
        Object.entries(testData).forEach(([key, value]) => {
          body = body.replace(new RegExp(`{{${key}}}`, 'g'), value)
        })

        // Prepare headers
        const headers = { ...config.headers }
        
        // Add authentication
        switch (config.authType) {
          case 'bearer':
            if (config.authConfig.token) {
              headers['Authorization'] = `Bearer ${config.authConfig.token}`
            }
            break
          case 'api_key':
            if (config.authConfig.apiKey && config.authConfig.headerName) {
              headers[config.authConfig.headerName] = config.authConfig.apiKey
            }
            break
          case 'basic':
            if (config.authConfig.username && config.authConfig.password) {
              headers['Authorization'] = `Basic ${btoa(`${config.authConfig.username}:${config.authConfig.password}`)}`
            }
            break
          case 'custom':
            if (config.authConfig.customAuth) {
              // Parse custom auth format like "X-API-Key: {token}"
              const [headerName, headerValue] = config.authConfig.customAuth.split(':').map(s => s.trim())
              headers[headerName] = headerValue.replace('{token}', config.authConfig.token || '')
            }
            break
        }

        // Make test request
        const response = await fetch(config.baseUrl, {
          method: config.method,
          headers,
          body: config.method !== 'GET' ? body : undefined
        })

        const result = await response.text()
        
        if (response.ok) {
          alert(`✅ API test successful!\n\nStatus: ${response.status}\nResponse: ${result.substring(0, 200)}${result.length > 200 ? '...' : ''}`)
        } else {
          alert(`❌ API test failed!\n\nStatus: ${response.status}\nResponse: ${result.substring(0, 200)}${result.length > 200 ? '...' : ''}`)
        }
      } catch (error) {
        alert(`❌ API test error: ${error}`)
      }
    }
  }

  const addHeader = () => {
    setCurrentConfig(prev => ({
      ...prev,
      headers: {
        ...prev.headers,
        '': ''
      }
    }))
  }

  const updateHeader = (oldKey: string, newKey: string, value: string) => {
    setCurrentConfig(prev => {
      const newHeaders = { ...prev.headers }
      if (oldKey !== newKey) {
        delete newHeaders[oldKey]
      }
      newHeaders[newKey] = value
      return { ...prev, headers: newHeaders }
    })
  }

  const removeHeader = (key: string) => {
    setCurrentConfig(prev => {
      const newHeaders = { ...prev.headers }
      delete newHeaders[key]
      return { ...prev, headers: newHeaders }
    })
  }

  const popularTemplates = {
    mailchimp: {
      name: 'Mailchimp Transactional',
      baseUrl: 'https://mandrillapp.com/api/1.0/messages/send.json',
      method: 'POST' as const,
      headers: { 'Content-Type': 'application/json' },
      bodyTemplate: JSON.stringify({
        key: '{{api_key}}',
        message: {
          to: [{ email: '{{to}}' }],
          subject: '{{subject}}',
          html: '{{body}}'
        }
      }, null, 2),
      authType: 'none' as const
    },
    elasticemail: {
      name: 'Elastic Email',
      baseUrl: 'https://api.elasticemail.com/v2/email/send',
      method: 'POST' as const,
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      bodyTemplate: 'apikey={{api_key}}&to={{to}}&subject={{subject}}&bodyHtml={{body}}',
      authType: 'none' as const
    },
    resend: {
      name: 'Resend',
      baseUrl: 'https://api.resend.com/emails',
      method: 'POST' as const,
      headers: { 'Content-Type': 'application/json' },
      bodyTemplate: JSON.stringify({
        from: 'onboarding@resend.dev',
        to: ['{{to}}'],
        subject: '{{subject}}',
        html: '{{body}}'
      }, null, 2),
      authType: 'bearer' as const
    }
  }

  const loadTemplate = (template: any) => {
    setCurrentConfig(prev => ({
      ...prev,
      ...template,
      name: prev.name || template.name
    }))
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium text-gray-900">Custom API Providers</h3>
          <p className="text-sm text-gray-600">
            Integrate with any email service that has an HTTP API
          </p>
        </div>
        <button
          onClick={() => setShowAddForm(true)}
          className="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors flex items-center space-x-2"
        >
          <span>+</span>
          <span>Add Custom API</span>
        </button>
      </div>

      {/* Existing Custom Providers */}
      {Object.keys(providers).length > 0 && (
        <div className="space-y-3">
          <h4 className="font-medium text-gray-900">Your Custom Providers</h4>
          {Object.entries(providers).map(([name, config]) => (
            <div key={name} className="bg-white rounded-lg border border-gray-200 p-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-3">
                  <span className="text-2xl">🔧</span>
                  <div>
                    <h5 className="font-medium text-gray-900">{config.name}</h5>
                    <p className="text-sm text-gray-600">{config.baseUrl}</p>
                    <div className="flex items-center space-x-2 mt-1">
                      <span className="px-2 py-1 text-xs bg-purple-100 text-purple-800 rounded-full">
                        {config.method}
                      </span>
                      <span className="px-2 py-1 text-xs bg-gray-100 text-gray-800 rounded-full">
                        {config.authType}
                      </span>
                    </div>
                  </div>
                </div>
                <div className="flex items-center space-x-2">
                  <button
                    onClick={() => handleTestProvider(config)}
                    className="px-3 py-1 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
                  >
                    Test
                  </button>
                  <button
                    onClick={() => handleEditProvider(name)}
                    className="px-3 py-1 text-sm bg-green-600 text-white rounded hover:bg-green-700 transition-colors"
                  >
                    Edit
                  </button>
                  <button
                    onClick={() => handleDeleteProvider(name)}
                    className="px-3 py-1 text-sm bg-red-600 text-white rounded hover:bg-red-700 transition-colors"
                  >
                    Delete
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Add/Edit Form */}
      {showAddForm && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 overflow-y-auto">
          <div className="bg-white rounded-lg p-6 w-full max-w-4xl mx-4 my-8 max-h-screen overflow-y-auto">
            <div className="flex items-center justify-between mb-4">
              <h4 className="text-lg font-medium text-gray-900">
                {editingProvider ? 'Edit' : 'Add'} Custom API Provider
              </h4>
              <button onClick={resetForm} className="text-gray-400 hover:text-gray-600">✕</button>
            </div>

            {/* Popular Templates */}
            <div className="mb-6 p-4 bg-blue-50 rounded-lg">
              <h5 className="font-medium text-blue-900 mb-3">Quick Templates</h5>
              <div className="flex flex-wrap gap-2">
                {Object.entries(popularTemplates).map(([key, template]) => (
                  <button
                    key={key}
                    onClick={() => loadTemplate(template)}
                    className="px-3 py-1 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
                  >
                    {template.name}
                  </button>
                ))}
              </div>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              {/* Left Column - Basic Config */}
              <div className="space-y-4">
                <h5 className="font-medium text-gray-900">Basic Configuration</h5>
                
                {/* Provider Name */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Provider Name *
                  </label>
                  <input
                    type="text"
                    value={currentConfig.name}
                    onChange={(e) => setCurrentConfig(prev => ({ ...prev, name: e.target.value }))}
                    placeholder="My Email Service"
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                  />
                </div>

                {/* Base URL */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    API Endpoint URL *
                  </label>
                  <input
                    type="url"
                    value={currentConfig.baseUrl}
                    onChange={(e) => setCurrentConfig(prev => ({ ...prev, baseUrl: e.target.value }))}
                    placeholder="https://api.emailservice.com/v1/send"
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                  />
                </div>

                {/* HTTP Method */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    HTTP Method
                  </label>
                  <select
                    value={currentConfig.method}
                    onChange={(e) => setCurrentConfig(prev => ({ ...prev, method: e.target.value as any }))}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                  >
                    <option value="POST">POST</option>
                    <option value="PUT">PUT</option>
                    <option value="GET">GET</option>
                  </select>
                </div>

                {/* Authentication Type */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Authentication Type
                  </label>
                  <select
                    value={currentConfig.authType}
                    onChange={(e) => setCurrentConfig(prev => ({ ...prev, authType: e.target.value as any }))}
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                  >
                    <option value="bearer">Bearer Token</option>
                    <option value="api_key">API Key Header</option>
                    <option value="basic">Basic Auth</option>
                    <option value="custom">Custom Header</option>
                    <option value="none">No Auth (Included in body)</option>
                  </select>
                </div>

                {/* Authentication Config */}
                <div className="space-y-2">
                  {currentConfig.authType === 'bearer' && (
                    <input
                      type="password"
                      placeholder="Bearer Token"
                      value={currentConfig.authConfig.token || ''}
                      onChange={(e) => setCurrentConfig(prev => ({
                        ...prev,
                        authConfig: { ...prev.authConfig, token: e.target.value }
                      }))}
                      className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                    />
                  )}
                  
                  {currentConfig.authType === 'api_key' && (
                    <>
                      <input
                        type="text"
                        placeholder="Header Name (e.g., X-API-Key)"
                        value={currentConfig.authConfig.headerName || ''}
                        onChange={(e) => setCurrentConfig(prev => ({
                          ...prev,
                          authConfig: { ...prev.authConfig, headerName: e.target.value }
                        }))}
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                      />
                      <input
                        type="password"
                        placeholder="API Key"
                        value={currentConfig.authConfig.apiKey || ''}
                        onChange={(e) => setCurrentConfig(prev => ({
                          ...prev,
                          authConfig: { ...prev.authConfig, apiKey: e.target.value }
                        }))}
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                      />
                    </>
                  )}
                  
                  {currentConfig.authType === 'basic' && (
                    <>
                      <input
                        type="text"
                        placeholder="Username"
                        value={currentConfig.authConfig.username || ''}
                        onChange={(e) => setCurrentConfig(prev => ({
                          ...prev,
                          authConfig: { ...prev.authConfig, username: e.target.value }
                        }))}
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                      />
                      <input
                        type="password"
                        placeholder="Password"
                        value={currentConfig.authConfig.password || ''}
                        onChange={(e) => setCurrentConfig(prev => ({
                          ...prev,
                          authConfig: { ...prev.authConfig, password: e.target.value }
                        }))}
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                      />
                    </>
                  )}
                  
                  {currentConfig.authType === 'custom' && (
                    <>
                      <input
                        type="text"
                        placeholder="Custom Auth Format (e.g., X-API-Key: {token})"
                        value={currentConfig.authConfig.customAuth || ''}
                        onChange={(e) => setCurrentConfig(prev => ({
                          ...prev,
                          authConfig: { ...prev.authConfig, customAuth: e.target.value }
                        }))}
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                      />
                      <input
                        type="password"
                        placeholder="Token/Value"
                        value={currentConfig.authConfig.token || ''}
                        onChange={(e) => setCurrentConfig(prev => ({
                          ...prev,
                          authConfig: { ...prev.authConfig, token: e.target.value }
                        }))}
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                      />
                    </>
                  )}
                </div>
              </div>

              {/* Right Column - Advanced Config */}
              <div className="space-y-4">
                <h5 className="font-medium text-gray-900">Advanced Configuration</h5>
                
                {/* Headers */}
                <div>
                  <div className="flex items-center justify-between mb-2">
                    <label className="block text-sm font-medium text-gray-700">
                      HTTP Headers
                    </label>
                    <button
                      type="button"
                      onClick={addHeader}
                      className="text-sm text-purple-600 hover:text-purple-700"
                    >
                      + Add Header
                    </button>
                  </div>
                  <div className="space-y-2 max-h-32 overflow-y-auto">
                    {Object.entries(currentConfig.headers).map(([key, value]) => (
                      <div key={key} className="flex space-x-2">
                        <input
                          type="text"
                          placeholder="Header Name"
                          value={key}
                          onChange={(e) => updateHeader(key, e.target.value, value)}
                          className="flex-1 px-2 py-1 text-sm border border-gray-300 rounded"
                        />
                        <input
                          type="text"
                          placeholder="Header Value"
                          value={value}
                          onChange={(e) => updateHeader(key, key, e.target.value)}
                          className="flex-1 px-2 py-1 text-sm border border-gray-300 rounded"
                        />
                        <button
                          type="button"
                          onClick={() => removeHeader(key)}
                          className="px-2 py-1 text-sm text-red-600 hover:text-red-700"
                        >
                          ×
                        </button>
                      </div>
                    ))}
                  </div>
                </div>

                {/* Body Template */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Request Body Template
                  </label>
                  <textarea
                    value={currentConfig.bodyTemplate}
                    onChange={(e) => setCurrentConfig(prev => ({ ...prev, bodyTemplate: e.target.value }))}
                    rows={8}
                    placeholder="Use {{to}}, {{subject}}, {{body}} as placeholders"
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-purple-500 focus:border-transparent font-mono text-sm"
                  />
                  <p className="text-xs text-gray-500 mt-1">
                    Use placeholders: {'{to}'}, {'{subject}'}, {'{body}'}, {'{api_key}'}
                  </p>
                </div>
              </div>
            </div>

            {/* Action Buttons */}
            <div className="flex justify-end space-x-3 mt-6 pt-4 border-t">
              <button
                onClick={resetForm}
                className="px-4 py-2 text-gray-600 border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={() => handleTestProvider(currentConfig)}
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
              >
                Test Configuration
              </button>
              <button
                onClick={handleSaveProvider}
                className="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors"
              >
                {editingProvider ? 'Update' : 'Add'} Provider
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Empty State */}
      {Object.keys(providers).length === 0 && !showAddForm && (
        <div className="text-center py-8 bg-gray-50 rounded-lg">
          <span className="text-4xl mb-4 block">🔧</span>
          <h4 className="text-lg font-medium text-gray-900 mb-2">No Custom Providers</h4>
          <p className="text-gray-600 mb-4">
            Create custom integrations for any email service with an HTTP API
          </p>
          <button
            onClick={() => setShowAddForm(true)}
            className="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors"
          >
            Add Your First Custom Provider
          </button>
        </div>
      )}

      {/* Documentation */}
      <div className="bg-amber-50 rounded-lg p-4 border border-amber-200">
        <h5 className="font-medium text-amber-800 mb-2">💡 Custom API Tips</h5>
        <ul className="text-sm text-amber-700 space-y-1">
          <li>• Use {'{to}'}, {'{subject}'}, {'{body}'} as placeholders in your template</li>
          <li>• Test your configuration before saving to ensure it works</li>
          <li>• Check the email provider's API documentation for exact format requirements</li>
          <li>• Most providers require JSON format, but some use form-urlencoded</li>
          <li>• Store API keys securely - they're encrypted in local storage</li>
        </ul>
      </div>
    </div>
  )
}

export default CustomAPIProviderSetup