/**
 * Discord Webhook Setup Component
 * Provides comprehensive Discord webhook configuration with testing and validation
 * Based on DISCORD_INTEGRATION_GUIDE.md
 */

import React, { useState, useEffect } from 'react'
import discordService, { DiscordWebhookConfig } from '../services/discordService'

interface DiscordWebhook extends DiscordWebhookConfig {
  id: string
  name: string
  channel_name?: string
  guild_name?: string
  is_connected: boolean
  created_at: string
  last_tested?: string
}

const DiscordWebhookSetup: React.FC = () => {
  const [webhooks, setWebhooks] = useState<DiscordWebhook[]>([])
  const [isAddingWebhook, setIsAddingWebhook] = useState(false)
  const [testingWebhook, setTestingWebhook] = useState<string | null>(null)
  const [newWebhook, setNewWebhook] = useState({
    name: '',
    webhook_url: ''
  })
  const [validationErrors, setValidationErrors] = useState<Record<string, string>>({})
  const [testResults, setTestResults] = useState<Record<string, { success: boolean; message: string }>>({})

  // Load saved webhooks from localStorage
  useEffect(() => {
    loadWebhooks()
  }, [])

  const loadWebhooks = () => {
    try {
      const savedWebhooks = localStorage.getItem('deflow_discord_webhooks')
      if (savedWebhooks) {
        setWebhooks(JSON.parse(savedWebhooks))
      }
    } catch (error) {
      console.error('Error loading Discord webhooks:', error)
    }
  }

  const saveWebhooks = (updatedWebhooks: DiscordWebhook[]) => {
    try {
      localStorage.setItem('deflow_discord_webhooks', JSON.stringify(updatedWebhooks))
      setWebhooks(updatedWebhooks)
    } catch (error) {
      console.error('Error saving Discord webhooks:', error)
    }
  }

  const validateWebhookConfig = (config: typeof newWebhook): Record<string, string> => {
    const errors: Record<string, string> = {}

    if (!config.name.trim()) {
      errors.name = 'Webhook name is required'
    }

    if (!config.webhook_url.trim()) {
      errors.webhook_url = 'Webhook URL is required'
    } else {
      const webhookPattern = /^https:\/\/discord\.com\/api\/webhooks\/\d+\/[a-zA-Z0-9_-]+$/
      if (!webhookPattern.test(config.webhook_url)) {
        errors.webhook_url = 'Invalid webhook URL format. Should be: https://discord.com/api/webhooks/ID/TOKEN'
      }
    }

    return errors
  }

  const parseWebhookUrl = (url: string): { server?: string; channel?: string } => {
    try {
      // Extract webhook ID and token from URL
      const match = url.match(/https:\/\/discord\.com\/api\/webhooks\/(\d+)\/([a-zA-Z0-9_-]+)/)
      if (match) {
        return {
          server: 'Discord Server', // We can't get server name from URL alone
          channel: `Webhook ${match[1].slice(-4)}...` // Show last 4 digits of webhook ID
        }
      }
    } catch (error) {
      console.warn('Could not parse webhook URL:', error)
    }
    return {}
  }

  const handleAddWebhook = async () => {
    const errors = validateWebhookConfig(newWebhook)
    setValidationErrors(errors)

    if (Object.keys(errors).length > 0) {
      return
    }

    // Check for duplicate names
    if (webhooks.some(webhook => webhook.name === newWebhook.name)) {
      setValidationErrors({ name: 'A webhook with this name already exists' })
      return
    }

    // Check for duplicate URLs
    if (webhooks.some(webhook => webhook.webhook_url === newWebhook.webhook_url)) {
      setValidationErrors({ webhook_url: 'This webhook URL is already configured' })
      return
    }

    const urlInfo = parseWebhookUrl(newWebhook.webhook_url)

    const webhookConfig: DiscordWebhook = {
      id: Date.now().toString(),
      name: newWebhook.name,
      webhook_url: newWebhook.webhook_url,
      channel_name: urlInfo.channel,
      guild_name: urlInfo.server,
      is_connected: false,
      created_at: new Date().toISOString()
    }

    // Test the webhook
    try {
      const testResult = await discordService.testWebhook(newWebhook.webhook_url)
      
      if (testResult.valid) {
        webhookConfig.is_connected = true
      }

      const updatedWebhooks = [...webhooks, webhookConfig]
      saveWebhooks(updatedWebhooks)

      // Reset form
      setNewWebhook({ name: '', webhook_url: '' })
      setValidationErrors({})
      setIsAddingWebhook(false)

    } catch (error) {
      setValidationErrors({ webhook_url: 'Failed to test webhook. Check your internet connection.' })
    }
  }

  const handleTestWebhook = async (webhook: DiscordWebhook) => {
    setTestingWebhook(webhook.id)
    setTestResults(prev => ({ ...prev, [webhook.id]: { success: false, message: 'Testing...' } }))

    try {
      const testResult = await discordService.testWebhook(webhook.webhook_url)
      
      if (testResult.valid) {
        setTestResults(prev => ({
          ...prev,
          [webhook.id]: { success: true, message: '✅ Test message sent successfully! Check your Discord channel.' }
        }))

        // Update webhook connection status
        const updatedWebhooks = webhooks.map(w => 
          w.id === webhook.id 
            ? { 
                ...w, 
                is_connected: true, 
                last_tested: new Date().toISOString() 
              }
            : w
        )
        saveWebhooks(updatedWebhooks)

      } else {
        setTestResults(prev => ({
          ...prev,
          [webhook.id]: { success: false, message: `❌ ${testResult.error}` }
        }))
      }

    } catch (error) {
      setTestResults(prev => ({
        ...prev,
        [webhook.id]: { 
          success: false, 
          message: `❌ Network error: ${error instanceof Error ? error.message : 'Unknown error'}` 
        }
      }))
    } finally {
      setTestingWebhook(null)
    }
  }

  const handleDeleteWebhook = (webhookId: string) => {
    if (confirm('Delete this Discord webhook configuration? This cannot be undone.')) {
      const updatedWebhooks = webhooks.filter(webhook => webhook.id !== webhookId)
      saveWebhooks(updatedWebhooks)
      // Clean up test results
      setTestResults(prev => {
        const newResults = { ...prev }
        delete newResults[webhookId]
        return newResults
      })
    }
  }

  const formatWebhookUrl = (url: string): string => {
    try {
      const match = url.match(/https:\/\/discord\.com\/api\/webhooks\/(\d+)\/([a-zA-Z0-9_-]+)/)
      if (match) {
        const id = match[1]
        const token = match[2]
        return `.../${id.slice(-4)}/${token.slice(0, 8)}...`
      }
    } catch (error) {
      // Ignore
    }
    return 'Invalid URL'
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium text-gray-900">Discord Webhooks</h3>
          <p className="text-sm text-gray-600">
            Configure Discord webhooks for sending notifications and alerts.
            <a 
              href="/DISCORD_INTEGRATION_GUIDE.md" 
              target="_blank" 
              className="text-blue-600 hover:text-blue-800 ml-1"
            >
              See setup guide →
            </a>
          </p>
        </div>
        
        <button
          onClick={() => setIsAddingWebhook(true)}
          className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors flex items-center space-x-2"
        >
          <span>+</span>
          <span>Add Webhook</span>
        </button>
      </div>

      {/* Setup Instructions */}
      {webhooks.length === 0 && !isAddingWebhook && (
        <div className="bg-purple-50 border border-purple-200 rounded-lg p-4">
          <h4 className="font-medium text-purple-900 mb-2">🚀 Get Started with Discord Webhooks</h4>
          <ol className="text-sm text-purple-800 space-y-1 list-decimal list-inside">
            <li>Right-click on your Discord channel</li>
            <li>Select "Edit Channel" → "Integrations"</li>
            <li>Click "Create Webhook"</li>
            <li>Copy the webhook URL</li>
            <li>Add it to DeFlow below</li>
          </ol>
        </div>
      )}

      {/* Add Webhook Form */}
      {isAddingWebhook && (
        <div className="bg-gray-50 border border-gray-200 rounded-lg p-6">
          <h4 className="font-medium text-gray-900 mb-4">Add New Discord Webhook</h4>
          
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Webhook Name *
              </label>
              <input
                type="text"
                value={newWebhook.name}
                onChange={(e) => setNewWebhook({ ...newWebhook, name: e.target.value })}
                placeholder="Portfolio Alerts"
                className={`w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                  validationErrors.name ? 'border-red-500' : 'border-gray-300'
                }`}
              />
              {validationErrors.name && (
                <p className="text-red-600 text-sm mt-1">{validationErrors.name}</p>
              )}
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Discord Webhook URL *
              </label>
              <input
                type="url"
                value={newWebhook.webhook_url}
                onChange={(e) => setNewWebhook({ ...newWebhook, webhook_url: e.target.value })}
                placeholder="https://discord.com/api/webhooks/123456789/abcdefghijklmnop"
                className={`w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                  validationErrors.webhook_url ? 'border-red-500' : 'border-gray-300'
                }`}
              />
              {validationErrors.webhook_url && (
                <p className="text-red-600 text-sm mt-1">{validationErrors.webhook_url}</p>
              )}
              <p className="text-xs text-gray-500 mt-1">
                Get webhook URL from Discord channel settings → Integrations → Create Webhook
              </p>
            </div>
          </div>

          <div className="mt-6 flex space-x-3">
            <button
              onClick={handleAddWebhook}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
            >
              Add Webhook
            </button>
            <button
              onClick={() => {
                setIsAddingWebhook(false)
                setValidationErrors({})
                setNewWebhook({ name: '', webhook_url: '' })
              }}
              className="px-4 py-2 bg-gray-300 text-gray-700 rounded-lg hover:bg-gray-400 transition-colors"
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {/* Configured Webhooks */}
      {webhooks.length > 0 && (
        <div className="space-y-4">
          {webhooks.map((webhook) => (
            <div key={webhook.id} className="bg-white border border-gray-200 rounded-lg p-6">
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center space-x-2 mb-2">
                    <h4 className="font-medium text-gray-900">{webhook.name}</h4>
                    <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                      webhook.is_connected 
                        ? 'bg-green-100 text-green-800'
                        : 'bg-yellow-100 text-yellow-800'
                    }`}>
                      {webhook.is_connected ? '✅ Connected' : '⚠️ Not Tested'}
                    </span>
                  </div>
                  
                  <div className="space-y-1 text-sm text-gray-600">
                    {webhook.guild_name && (
                      <p>🏰 Server: {webhook.guild_name}</p>
                    )}
                    {webhook.channel_name && (
                      <p>📢 Channel: #{webhook.channel_name}</p>
                    )}
                    <p>🔗 URL: {formatWebhookUrl(webhook.webhook_url)}</p>
                    <p>📅 Added: {new Date(webhook.created_at).toLocaleDateString()}</p>
                    {webhook.last_tested && (
                      <p>🔍 Last tested: {new Date(webhook.last_tested).toLocaleString()}</p>
                    )}
                  </div>

                  {/* Test Results */}
                  {testResults[webhook.id] && (
                    <div className={`mt-3 p-3 rounded-lg text-sm ${
                      testResults[webhook.id].success 
                        ? 'bg-green-50 text-green-800 border border-green-200'
                        : 'bg-red-50 text-red-800 border border-red-200'
                    }`}>
                      {testResults[webhook.id].message}
                    </div>
                  )}
                </div>

                <div className="flex space-x-2 ml-4">
                  <button
                    onClick={() => handleTestWebhook(webhook)}
                    disabled={testingWebhook === webhook.id}
                    className="px-3 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {testingWebhook === webhook.id ? '⏳ Testing...' : '🧪 Test'}
                  </button>
                  
                  <button
                    onClick={() => handleDeleteWebhook(webhook.id)}
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
      {webhooks.length > 0 && (
        <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
          <h4 className="font-medium text-gray-900 mb-2">📝 Usage in Workflows</h4>
          <p className="text-sm text-gray-600 mb-2">
            Use your configured Discord webhooks in workflow nodes:
          </p>
          <ol className="text-sm text-gray-600 space-y-1 list-decimal list-inside">
            <li>Add a "Discord Message" node to your workflow</li>
            <li>Paste your webhook URL from the configurations above</li>
            <li>Choose message type (text, rich embed, or file attachment)</li>
            <li>Configure your message with template variables like {'{{portfolio_value}}'}</li>
            <li>For rich embeds, set colors, fields, images, and thumbnails</li>
            <li>Test your workflow to ensure messages appear in Discord</li>
          </ol>
        </div>
      )}

      {/* Feature Comparison */}
      <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
        <h4 className="font-medium text-blue-900 mb-2">💡 Discord vs Other Platforms</h4>
        <div className="text-sm text-blue-800 space-y-1">
          <p><strong>Discord Strengths:</strong> Rich embeds, community-focused, excellent for DeFi/crypto communities</p>
          <p><strong>When to use:</strong> Community alerts, trading signals, group notifications, rich formatted data</p>
          <p><strong>Best for:</strong> Public announcements, portfolio summaries with charts, community engagement</p>
        </div>
      </div>
    </div>
  )
}

export default DiscordWebhookSetup