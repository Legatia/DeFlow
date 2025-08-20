/**
 * Telegram Bot Setup Component
 * Provides comprehensive Telegram Bot API configuration with testing and validation
 * Based on TELEGRAM_BOT_API_GUIDE.md
 */

import React, { useState, useEffect } from 'react'
import telegramService, { TelegramConfig, TelegramUser } from '../services/telegramService'

interface TelegramBot extends TelegramConfig {
  id: string
  name: string
  bot_info?: TelegramUser
  is_connected: boolean
  created_at: string
  last_tested?: string
}

const TelegramBotSetup: React.FC = () => {
  const [bots, setBots] = useState<TelegramBot[]>([])
  const [isAddingBot, setIsAddingBot] = useState(false)
  const [testingBot, setTestingBot] = useState<string | null>(null)
  const [newBot, setNewBot] = useState({
    name: '',
    bot_token: '',
    chat_id: ''
  })
  const [validationErrors, setValidationErrors] = useState<Record<string, string>>({})
  const [testResults, setTestResults] = useState<Record<string, { success: boolean; message: string }>>({})

  // Load saved bots from localStorage
  useEffect(() => {
    loadBots()
  }, [])

  const loadBots = () => {
    try {
      const savedBots = localStorage.getItem('deflow_telegram_bots')
      if (savedBots) {
        setBots(JSON.parse(savedBots))
      }
    } catch (error) {
      console.error('Error loading Telegram bots:', error)
    }
  }

  const saveBots = (updatedBots: TelegramBot[]) => {
    try {
      localStorage.setItem('deflow_telegram_bots', JSON.stringify(updatedBots))
      setBots(updatedBots)
    } catch (error) {
      console.error('Error saving Telegram bots:', error)
    }
  }

  const validateBotConfig = (config: typeof newBot): Record<string, string> => {
    const errors: Record<string, string> = {}

    if (!config.name.trim()) {
      errors.name = 'Bot name is required'
    }

    if (!config.bot_token.trim()) {
      errors.bot_token = 'Bot token is required'
    } else if (!/^\d+:[A-Za-z0-9_-]{35}$/.test(config.bot_token)) {
      errors.bot_token = 'Invalid bot token format (should be: 123456789:ABC-DEF...)'
    }

    if (!config.chat_id.trim()) {
      errors.chat_id = 'Chat ID is required'
    } else {
      const chatId = config.chat_id.trim()
      const isValidFormat = /^\d+$/.test(chatId) || // Private chats
                           /^-100\d{10,}$/.test(chatId) || // Groups
                           /^-\d+$/.test(chatId) || // Other chats
                           /^@\w+$/.test(chatId) // Channel usernames

      if (!isValidFormat) {
        errors.chat_id = 'Invalid Chat ID format. Use: positive number (user), -100xxxxx (group), or @username (channel)'
      }
    }

    return errors
  }

  const handleAddBot = async () => {
    const errors = validateBotConfig(newBot)
    setValidationErrors(errors)

    if (Object.keys(errors).length > 0) {
      return
    }

    // Check for duplicate names
    if (bots.some(bot => bot.name === newBot.name)) {
      setValidationErrors({ name: 'A bot with this name already exists' })
      return
    }

    const botConfig: TelegramBot = {
      id: Date.now().toString(),
      name: newBot.name,
      bot_token: newBot.bot_token,
      chat_id: newBot.chat_id,
      is_connected: false,
      created_at: new Date().toISOString()
    }

    // Test the bot token
    try {
      const testResult = await telegramService.testBotToken(newBot.bot_token)
      
      if (testResult.valid && testResult.bot_info) {
        botConfig.bot_info = testResult.bot_info
        botConfig.is_connected = true
      }

      const updatedBots = [...bots, botConfig]
      saveBots(updatedBots)

      // Reset form
      setNewBot({ name: '', bot_token: '', chat_id: '' })
      setValidationErrors({})
      setIsAddingBot(false)

    } catch (error) {
      setValidationErrors({ bot_token: 'Failed to connect to Telegram API. Check your internet connection.' })
    }
  }

  const handleTestBot = async (bot: TelegramBot) => {
    setTestingBot(bot.id)
    setTestResults(prev => ({ ...prev, [bot.id]: { success: false, message: 'Testing...' } }))

    try {
      // Test bot token
      const tokenTest = await telegramService.testBotToken(bot.bot_token)
      
      if (!tokenTest.valid) {
        setTestResults(prev => ({
          ...prev,
          [bot.id]: { success: false, message: tokenTest.error || 'Invalid bot token' }
        }))
        return
      }

      // Test sending a message
      const testMessage = `🤖 **Telegram Bot Test**

Hello! This is a test message from DeFlow.

✅ Your bot is connected successfully!
📊 Bot: ${tokenTest.bot_info?.first_name}
🆔 Chat ID: ${bot.chat_id}
⏰ Time: ${new Date().toLocaleTimeString()}

You can now use this bot in your DeFlow workflows.`

      const messageResult = await telegramService.sendTelegramMessage(
        { bot_token: bot.bot_token, chat_id: bot.chat_id },
        {
          message_type: 'text',
          message: testMessage,
          parse_mode: 'Markdown'
        }
      )

      if (messageResult.ok) {
        setTestResults(prev => ({
          ...prev,
          [bot.id]: { success: true, message: '✅ Test message sent successfully! Check your Telegram.' }
        }))

        // Update bot info and connection status
        const updatedBots = bots.map(b => 
          b.id === bot.id 
            ? { 
                ...b, 
                bot_info: tokenTest.bot_info, 
                is_connected: true, 
                last_tested: new Date().toISOString() 
              }
            : b
        )
        saveBots(updatedBots)

      } else {
        const errorHelp = telegramService.getErrorHelp(messageResult)
        setTestResults(prev => ({
          ...prev,
          [bot.id]: { success: false, message: `❌ ${errorHelp}` }
        }))
      }

    } catch (error) {
      setTestResults(prev => ({
        ...prev,
        [bot.id]: { 
          success: false, 
          message: `❌ Network error: ${error instanceof Error ? error.message : 'Unknown error'}` 
        }
      }))
    } finally {
      setTestingBot(null)
    }
  }

  const handleDeleteBot = (botId: string) => {
    if (confirm('Delete this Telegram bot configuration? This cannot be undone.')) {
      const updatedBots = bots.filter(bot => bot.id !== botId)
      saveBots(updatedBots)
      // Clean up test results
      setTestResults(prev => {
        const newResults = { ...prev }
        delete newResults[botId]
        return newResults
      })
    }
  }

  const formatChatId = (chatId: string): string => {
    if (chatId.startsWith('-100')) {
      return `${chatId} (Group)`
    } else if (chatId.startsWith('-')) {
      return `${chatId} (Channel)`
    } else if (chatId.startsWith('@')) {
      return `${chatId} (Username)`
    } else {
      return `${chatId} (User)`
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium text-gray-900">Telegram Bots</h3>
          <p className="text-sm text-gray-600">
            Configure Telegram bots for sending notifications and alerts.
            <a 
              href="/TELEGRAM_BOT_API_GUIDE.md" 
              target="_blank" 
              className="text-blue-600 hover:text-blue-800 ml-1"
            >
              See setup guide →
            </a>
          </p>
        </div>
        
        <button
          onClick={() => setIsAddingBot(true)}
          className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors flex items-center space-x-2"
        >
          <span>+</span>
          <span>Add Bot</span>
        </button>
      </div>

      {/* Setup Instructions */}
      {bots.length === 0 && !isAddingBot && (
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <h4 className="font-medium text-blue-900 mb-2">🚀 Get Started with Telegram Bots</h4>
          <ol className="text-sm text-blue-800 space-y-1 list-decimal list-inside">
            <li>Message @BotFather on Telegram</li>
            <li>Create a new bot with /newbot command</li>
            <li>Copy the bot token (keep it secure!)</li>
            <li>Get your Chat ID by messaging the bot</li>
            <li>Add the bot configuration below</li>
          </ol>
        </div>
      )}

      {/* Add Bot Form */}
      {isAddingBot && (
        <div className="bg-gray-50 border border-gray-200 rounded-lg p-6">
          <h4 className="font-medium text-gray-900 mb-4">Add New Telegram Bot</h4>
          
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Bot Name *
              </label>
              <input
                type="text"
                value={newBot.name}
                onChange={(e) => setNewBot({ ...newBot, name: e.target.value })}
                placeholder="My Portfolio Bot"
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
                Chat ID *
              </label>
              <input
                type="text"
                value={newBot.chat_id}
                onChange={(e) => setNewBot({ ...newBot, chat_id: e.target.value })}
                placeholder="123456789 or -1001234567890"
                className={`w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                  validationErrors.chat_id ? 'border-red-500' : 'border-gray-300'
                }`}
              />
              {validationErrors.chat_id && (
                <p className="text-red-600 text-sm mt-1">{validationErrors.chat_id}</p>
              )}
              <p className="text-xs text-gray-500 mt-1">
                Get your Chat ID by messaging @userinfobot
              </p>
            </div>
          </div>

          <div className="mt-4">
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Bot Token *
            </label>
            <input
              type="password"
              value={newBot.bot_token}
              onChange={(e) => setNewBot({ ...newBot, bot_token: e.target.value })}
              placeholder="1234567890:ABCdefGHIjklMNOpqrsTUVwxyz"
              className={`w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                validationErrors.bot_token ? 'border-red-500' : 'border-gray-300'
              }`}
            />
            {validationErrors.bot_token && (
              <p className="text-red-600 text-sm mt-1">{validationErrors.bot_token}</p>
            )}
            <p className="text-xs text-gray-500 mt-1">
              Get bot token from @BotFather on Telegram
            </p>
          </div>

          <div className="mt-6 flex space-x-3">
            <button
              onClick={handleAddBot}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
            >
              Add Bot
            </button>
            <button
              onClick={() => {
                setIsAddingBot(false)
                setValidationErrors({})
                setNewBot({ name: '', bot_token: '', chat_id: '' })
              }}
              className="px-4 py-2 bg-gray-300 text-gray-700 rounded-lg hover:bg-gray-400 transition-colors"
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {/* Configured Bots */}
      {bots.length > 0 && (
        <div className="space-y-4">
          {bots.map((bot) => (
            <div key={bot.id} className="bg-white border border-gray-200 rounded-lg p-6">
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center space-x-2 mb-2">
                    <h4 className="font-medium text-gray-900">{bot.name}</h4>
                    <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                      bot.is_connected 
                        ? 'bg-green-100 text-green-800'
                        : 'bg-yellow-100 text-yellow-800'
                    }`}>
                      {bot.is_connected ? '✅ Connected' : '⚠️ Not Tested'}
                    </span>
                  </div>
                  
                  <div className="space-y-1 text-sm text-gray-600">
                    {bot.bot_info && (
                      <p>🤖 Bot: {bot.bot_info.first_name} (@{bot.bot_info.username || 'unknown'})</p>
                    )}
                    <p>💬 Chat: {formatChatId(bot.chat_id)}</p>
                    <p>📅 Added: {new Date(bot.created_at).toLocaleDateString()}</p>
                    {bot.last_tested && (
                      <p>🔍 Last tested: {new Date(bot.last_tested).toLocaleString()}</p>
                    )}
                  </div>

                  {/* Test Results */}
                  {testResults[bot.id] && (
                    <div className={`mt-3 p-3 rounded-lg text-sm ${
                      testResults[bot.id].success 
                        ? 'bg-green-50 text-green-800 border border-green-200'
                        : 'bg-red-50 text-red-800 border border-red-200'
                    }`}>
                      {testResults[bot.id].message}
                    </div>
                  )}
                </div>

                <div className="flex space-x-2 ml-4">
                  <button
                    onClick={() => handleTestBot(bot)}
                    disabled={testingBot === bot.id}
                    className="px-3 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {testingBot === bot.id ? '⏳ Testing...' : '🧪 Test'}
                  </button>
                  
                  <button
                    onClick={() => handleDeleteBot(bot.id)}
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
      {bots.length > 0 && (
        <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
          <h4 className="font-medium text-gray-900 mb-2">📝 Usage in Workflows</h4>
          <p className="text-sm text-gray-600 mb-2">
            Use your configured Telegram bots in workflow nodes:
          </p>
          <ol className="text-sm text-gray-600 space-y-1 list-decimal list-inside">
            <li>Add a "Telegram Message" node to your workflow</li>
            <li>Enter your bot token and chat ID from the configurations above</li>
            <li>Choose message type (text, photo, document, poll)</li>
            <li>Configure your message with template variables like {'{{portfolio_value}}'}</li>
            <li>Test your workflow to ensure messages are delivered</li>
          </ol>
        </div>
      )}
    </div>
  )
}

export default TelegramBotSetup