import React, { useState, useEffect } from 'react'
import emailService, { EmailConfig, EmailMessage, EmailTemplates } from '../services/emailService'
import OAuth2Setup from './OAuth2Setup'

interface EmailProviderSetupProps {
  onProviderAdded?: (name: string) => void
  onProviderRemoved?: (name: string) => void
}

const EmailProviderSetup: React.FC<EmailProviderSetupProps> = ({ 
  onProviderAdded, 
  onProviderRemoved 
}) => {
  const [providers, setProviders] = useState<string[]>([])
  const [showAddForm, setShowAddForm] = useState(false)
  const [selectedProvider, setSelectedProvider] = useState<'sendgrid' | 'mailgun' | 'postmark'>('sendgrid')
  const [providerName, setProviderName] = useState('')
  const [apiKey, setApiKey] = useState('')
  const [domain, setDomain] = useState('') // For Mailgun
  const [fromEmail, setFromEmail] = useState('')
  const [fromName, setFromName] = useState('')
  const [testResults, setTestResults] = useState<Record<string, boolean>>({})

  useEffect(() => {
    loadProviders()
  }, [])

  const loadProviders = () => {
    setProviders(emailService.getProviders())
  }

  const handleAddProvider = () => {
    if (!providerName.trim() || !apiKey.trim()) {
      alert('Please fill in required fields')
      return
    }

    const config: EmailConfig = {
      provider: selectedProvider,
      credentials: {
        api_key: apiKey,
        ...(selectedProvider === 'mailgun' && domain ? { domain } : {})
      },
      from_email: fromEmail || undefined,
      from_name: fromName || undefined
    }

    emailService.addProvider(providerName, config)
    loadProviders()
    onProviderAdded?.(providerName)
    
    // Reset form
    setProviderName('')
    setApiKey('')
    setDomain('')
    setFromEmail('')
    setFromName('')
    setShowAddForm(false)
  }

  const handleRemoveProvider = (name: string) => {
    if (confirm(`Remove email provider "${name}"?`)) {
      emailService.removeProvider(name)
      loadProviders()
      onProviderRemoved?.(name)
    }
  }

  const handleTestProvider = async (name: string) => {
    try {
      const success = await emailService.testProvider(name)
      setTestResults(prev => ({ ...prev, [name]: success }))
    } catch (error) {
      setTestResults(prev => ({ ...prev, [name]: false }))
      console.error('Provider test failed:', error)
    }
  }

  const handleSendTestEmail = async (providerName: string) => {
    const testEmail = prompt('Enter email address to send test email:')
    if (!testEmail) return

    try {
      const testMessage: EmailMessage = {
        to: testEmail,
        subject: 'DeFlow Email Test',
        body_html: `
          <h2>✅ Email Service Test</h2>
          <p>This is a test email from DeFlow using the <strong>${providerName}</strong> provider.</p>
          <p>If you received this email, your email integration is working correctly!</p>
          <hr>
          <small>Sent from DeFlow at ${new Date().toLocaleString()}</small>
        `,
        body_text: `DeFlow Email Test\n\nThis is a test email from DeFlow using the ${providerName} provider.\n\nIf you received this email, your email integration is working correctly!\n\nSent from DeFlow at ${new Date().toLocaleString()}`
      }

      const result = await emailService.sendEmail(providerName, testMessage)
      
      if (result.success) {
        alert(`✅ Test email sent successfully!\nMessage ID: ${result.message_id}`)
      } else {
        alert(`❌ Failed to send test email: ${result.error}`)
      }
    } catch (error) {
      alert(`❌ Error sending test email: ${error}`)
    }
  }

  const getProviderIcon = (provider: string) => {
    switch (provider) {
      case 'sendgrid': return '📧'
      case 'mailgun': return '🔫'
      case 'postmark': return '📮'
      default: return '✉️'
    }
  }

  const getProviderColor = (provider: string) => {
    switch (provider) {
      case 'sendgrid': return 'bg-blue-100 text-blue-800'
      case 'mailgun': return 'bg-orange-100 text-orange-800'
      case 'postmark': return 'bg-green-100 text-green-800'
      default: return 'bg-gray-100 text-gray-800'
    }
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium text-gray-900">Email Providers</h3>
          <p className="text-sm text-gray-600">
            Configure HTTP-based email services (ICP compatible)
          </p>
        </div>
        <button
          onClick={() => setShowAddForm(true)}
          className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors flex items-center space-x-2"
        >
          <span>+</span>
          <span>Add Provider</span>
        </button>
      </div>

      {/* OAuth2 Providers */}
      <div className="mb-8">
        <OAuth2Setup 
          onProviderConnected={onProviderAdded}
          onProviderDisconnected={onProviderRemoved}
        />
      </div>

      {/* Configured Providers */}
      {providers.length > 0 && (
        <div className="space-y-3">
          <h4 className="font-medium text-gray-900">Configured Providers</h4>
          {providers.map((name) => (
            <div key={name} className="bg-white rounded-lg border border-gray-200 p-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-3">
                  <span className="text-2xl">{getProviderIcon(name)}</span>
                  <div>
                    <h5 className="font-medium text-gray-900">{name}</h5>
                    <div className="flex items-center space-x-2 mt-1">
                      <span className={`px-2 py-1 text-xs rounded-full ${getProviderColor(name)}`}>
                        {name}
                      </span>
                      {testResults[name] !== undefined && (
                        <span className={`px-2 py-1 text-xs rounded-full ${
                          testResults[name] 
                            ? 'bg-green-100 text-green-800' 
                            : 'bg-red-100 text-red-800'
                        }`}>
                          {testResults[name] ? '✅ Tested' : '❌ Failed'}
                        </span>
                      )}
                    </div>
                  </div>
                </div>
                <div className="flex items-center space-x-2">
                  <button
                    onClick={() => handleSendTestEmail(name)}
                    className="px-3 py-1 text-sm bg-green-600 text-white rounded hover:bg-green-700 transition-colors"
                  >
                    Send Test
                  </button>
                  <button
                    onClick={() => handleTestProvider(name)}
                    className="px-3 py-1 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
                  >
                    Test Config
                  </button>
                  <button
                    onClick={() => handleRemoveProvider(name)}
                    className="px-3 py-1 text-sm bg-red-600 text-white rounded hover:bg-red-700 transition-colors"
                  >
                    Remove
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Add Provider Form */}
      {showAddForm && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-full max-w-lg mx-4">
            <div className="flex items-center justify-between mb-4">
              <h4 className="text-lg font-medium text-gray-900">Add Email Provider</h4>
              <button
                onClick={() => setShowAddForm(false)}
                className="text-gray-400 hover:text-gray-600"
              >
                ✕
              </button>
            </div>

            <div className="space-y-4">
              {/* Provider Type */}
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Provider Type
                </label>
                <select
                  value={selectedProvider}
                  onChange={(e) => setSelectedProvider(e.target.value as any)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                >
                  <option value="sendgrid">SendGrid</option>
                  <option value="mailgun">Mailgun</option>
                  <option value="postmark">Postmark</option>
                </select>
              </div>

              {/* Provider Name */}
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Provider Name *
                </label>
                <input
                  type="text"
                  value={providerName}
                  onChange={(e) => setProviderName(e.target.value)}
                  placeholder="e.g., main-alerts, billing-emails"
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
              </div>

              {/* API Key */}
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  API Key *
                </label>
                <input
                  type="password"
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  placeholder={
                    selectedProvider === 'sendgrid' ? 'SG.xxxxxxxxxxxxxxxx' :
                    selectedProvider === 'mailgun' ? 'key-xxxxxxxxxxxxxxxx' :
                    'xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx'
                  }
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
              </div>

              {/* Domain (Mailgun only) */}
              {selectedProvider === 'mailgun' && (
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    Domain *
                  </label>
                  <input
                    type="text"
                    value={domain}
                    onChange={(e) => setDomain(e.target.value)}
                    placeholder="mg.yourdomain.com"
                    className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                  />
                </div>
              )}

              {/* From Email */}
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  From Email
                </label>
                <input
                  type="email"
                  value={fromEmail}
                  onChange={(e) => setFromEmail(e.target.value)}
                  placeholder="noreply@yourdomain.com"
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
              </div>

              {/* From Name */}
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  From Name
                </label>
                <input
                  type="text"
                  value={fromName}
                  onChange={(e) => setFromName(e.target.value)}
                  placeholder="DeFlow Alerts"
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
              </div>

              {/* Provider Info */}
              <div className="bg-gray-50 rounded p-3 text-sm">
                <h5 className="font-medium text-gray-900 mb-1">
                  {selectedProvider === 'sendgrid' && 'SendGrid Setup'}
                  {selectedProvider === 'mailgun' && 'Mailgun Setup'}
                  {selectedProvider === 'postmark' && 'Postmark Setup'}
                </h5>
                <p className="text-gray-600">
                  {selectedProvider === 'sendgrid' && 'Get your API key from SendGrid → Settings → API Keys'}
                  {selectedProvider === 'mailgun' && 'Get your API key and domain from Mailgun → Settings → API Keys'}
                  {selectedProvider === 'postmark' && 'Get your server token from Postmark → Servers → API Tokens'}
                </p>
              </div>
            </div>

            <div className="flex justify-end space-x-3 mt-6">
              <button
                onClick={() => setShowAddForm(false)}
                className="px-4 py-2 text-gray-600 border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleAddProvider}
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
              >
                Add Provider
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Empty State */}
      {providers.length === 0 && !showAddForm && (
        <div className="text-center py-8 bg-gray-50 rounded-lg">
          <span className="text-4xl mb-4 block">📧</span>
          <h4 className="text-lg font-medium text-gray-900 mb-2">No Email Providers</h4>
          <p className="text-gray-600 mb-4">
            Add an email provider to enable automated notifications
          </p>
          <button
            onClick={() => setShowAddForm(true)}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            Add Your First Provider
          </button>
        </div>
      )}
    </div>
  )
}

export default EmailProviderSetup