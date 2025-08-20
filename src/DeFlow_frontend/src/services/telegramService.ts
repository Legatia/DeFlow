/**
 * Telegram Bot API Service
 * Handles all Telegram bot functionality including messages, photos, documents, polls, and interactive keyboards
 * Based on the comprehensive TELEGRAM_BOT_API_GUIDE.md
 */

export interface TelegramConfig {
  bot_token: string
  chat_id: string
}

export interface TelegramMessage {
  message_type: 'text' | 'photo' | 'document' | 'location' | 'poll'
  message: string
  parse_mode?: 'Markdown' | 'MarkdownV2' | 'HTML' | '' | null | undefined
  photo_url?: string
  document_url?: string
  location_lat?: number
  location_lng?: number
  poll_question?: string
  poll_options?: string
  inline_keyboard?: string
  disable_preview?: boolean
  silent?: boolean
  protect_content?: boolean
  reply_to_message_id?: string
  thread_id?: string
}

export interface TelegramResponse {
  ok: boolean
  result?: any
  error_code?: number
  description?: string
}

export interface TelegramUser {
  id: number
  first_name: string
  last_name?: string
  username?: string
  is_bot: boolean
}

export interface TelegramChat {
  id: number
  type: 'private' | 'group' | 'supergroup' | 'channel'
  title?: string
  username?: string
}

class TelegramService {
  private readonly BASE_URL = 'https://api.telegram.org/bot'
  
  /**
   * Validate bot token format
   */
  private isValidToken(token: string): boolean {
    return /^\d+:[A-Za-z0-9_-]{35}$/.test(token)
  }

  /**
   * Validate chat ID format
   */
  private isValidChatId(chatId: string): boolean {
    // Private chats: positive integers
    if (/^\d+$/.test(chatId)) return true
    
    // Groups: negative integers starting with -100
    if (/^-100\d{10,}$/.test(chatId)) return true
    
    // Channels: negative integers or @username
    if (/^-\d+$/.test(chatId) || /^@\w+$/.test(chatId)) return true
    
    return false
  }

  /**
   * Make API request to Telegram Bot API
   */
  private async makeRequest(
    token: string, 
    method: string, 
    data: Record<string, any> = {}
  ): Promise<TelegramResponse> {
    if (!this.isValidToken(token)) {
      throw new Error('Invalid bot token format')
    }

    const url = `${this.BASE_URL}${token}/${method}`
    
    try {
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data)
      })

      const result = await response.json()
      
      if (!result.ok) {
        console.error(`Telegram API error (${method}):`, result.description)
      }

      return result as TelegramResponse
    } catch (error) {
      console.error(`Telegram network error (${method}):`, error)
      throw new Error(`Failed to call Telegram API: ${error instanceof Error ? error.message : 'Unknown error'}`)
    }
  }

  /**
   * Test bot token validity
   */
  async testBotToken(token: string): Promise<{ valid: boolean; bot_info?: TelegramUser; error?: string }> {
    try {
      const response = await this.makeRequest(token, 'getMe')
      
      if (response.ok) {
        return {
          valid: true,
          bot_info: response.result as TelegramUser
        }
      } else {
        return {
          valid: false,
          error: response.description || 'Invalid bot token'
        }
      }
    } catch (error) {
      return {
        valid: false,
        error: error instanceof Error ? error.message : 'Network error'
      }
    }
  }

  /**
   * Send text message
   */
  async sendMessage(config: TelegramConfig, messageData: TelegramMessage): Promise<TelegramResponse> {
    if (!this.isValidChatId(config.chat_id)) {
      throw new Error('Invalid chat ID format')
    }

    const requestData: Record<string, any> = {
      chat_id: config.chat_id,
      text: messageData.message
    }

    // Add optional parameters
    if (messageData.parse_mode && messageData.parse_mode.trim() !== '') {
      requestData.parse_mode = messageData.parse_mode
    }

    if (messageData.disable_preview) {
      requestData.disable_web_page_preview = true
    }

    if (messageData.silent) {
      requestData.disable_notification = true
    }

    if (messageData.protect_content) {
      requestData.protect_content = true
    }

    if (messageData.reply_to_message_id) {
      requestData.reply_to_message_id = parseInt(messageData.reply_to_message_id)
    }

    if (messageData.thread_id) {
      requestData.message_thread_id = parseInt(messageData.thread_id)
    }

    // Add inline keyboard if provided
    if (messageData.inline_keyboard) {
      try {
        const keyboards = JSON.parse(messageData.inline_keyboard)
        if (Array.isArray(keyboards)) {
          // Convert flat array to keyboard rows
          const rows = []
          for (let i = 0; i < keyboards.length; i += 2) {
            const row = keyboards.slice(i, i + 2)
            rows.push(row)
          }
          requestData.reply_markup = { inline_keyboard: rows }
        }
      } catch (error) {
        console.warn('Invalid inline keyboard JSON, ignoring:', error)
      }
    }

    return this.makeRequest(config.bot_token, 'sendMessage', requestData)
  }

  /**
   * Send photo with caption
   */
  async sendPhoto(config: TelegramConfig, messageData: TelegramMessage): Promise<TelegramResponse> {
    if (!messageData.photo_url) {
      throw new Error('Photo URL is required for photo messages')
    }

    if (!this.isValidChatId(config.chat_id)) {
      throw new Error('Invalid chat ID format')
    }

    const requestData: Record<string, any> = {
      chat_id: config.chat_id,
      photo: messageData.photo_url,
      caption: messageData.message
    }

    // Add optional parameters
    if (messageData.parse_mode && messageData.parse_mode.trim() !== '') {
      requestData.parse_mode = messageData.parse_mode
    }

    if (messageData.silent) {
      requestData.disable_notification = true
    }

    if (messageData.protect_content) {
      requestData.protect_content = true
    }

    if (messageData.reply_to_message_id) {
      requestData.reply_to_message_id = parseInt(messageData.reply_to_message_id)
    }

    if (messageData.thread_id) {
      requestData.message_thread_id = parseInt(messageData.thread_id)
    }

    // Add inline keyboard if provided
    if (messageData.inline_keyboard) {
      try {
        const keyboards = JSON.parse(messageData.inline_keyboard)
        if (Array.isArray(keyboards)) {
          const rows = []
          for (let i = 0; i < keyboards.length; i += 2) {
            const row = keyboards.slice(i, i + 2)
            rows.push(row)
          }
          requestData.reply_markup = { inline_keyboard: rows }
        }
      } catch (error) {
        console.warn('Invalid inline keyboard JSON, ignoring:', error)
      }
    }

    return this.makeRequest(config.bot_token, 'sendPhoto', requestData)
  }

  /**
   * Send document/file
   */
  async sendDocument(config: TelegramConfig, messageData: TelegramMessage): Promise<TelegramResponse> {
    if (!messageData.document_url) {
      throw new Error('Document URL is required for document messages')
    }

    if (!this.isValidChatId(config.chat_id)) {
      throw new Error('Invalid chat ID format')
    }

    const requestData: Record<string, any> = {
      chat_id: config.chat_id,
      document: messageData.document_url,
      caption: messageData.message
    }

    // Add optional parameters
    if (messageData.parse_mode && messageData.parse_mode.trim() !== '') {
      requestData.parse_mode = messageData.parse_mode
    }

    if (messageData.silent) {
      requestData.disable_notification = true
    }

    if (messageData.protect_content) {
      requestData.protect_content = true
    }

    if (messageData.reply_to_message_id) {
      requestData.reply_to_message_id = parseInt(messageData.reply_to_message_id)
    }

    if (messageData.thread_id) {
      requestData.message_thread_id = parseInt(messageData.thread_id)
    }

    return this.makeRequest(config.bot_token, 'sendDocument', requestData)
  }

  /**
   * Send location
   */
  async sendLocation(config: TelegramConfig, messageData: TelegramMessage): Promise<TelegramResponse> {
    if (messageData.location_lat === undefined || messageData.location_lng === undefined) {
      throw new Error('Latitude and longitude are required for location messages')
    }

    if (!this.isValidChatId(config.chat_id)) {
      throw new Error('Invalid chat ID format')
    }

    const requestData: Record<string, any> = {
      chat_id: config.chat_id,
      latitude: messageData.location_lat,
      longitude: messageData.location_lng
    }

    if (messageData.silent) {
      requestData.disable_notification = true
    }

    if (messageData.protect_content) {
      requestData.protect_content = true
    }

    if (messageData.reply_to_message_id) {
      requestData.reply_to_message_id = parseInt(messageData.reply_to_message_id)
    }

    if (messageData.thread_id) {
      requestData.message_thread_id = parseInt(messageData.thread_id)
    }

    return this.makeRequest(config.bot_token, 'sendLocation', requestData)
  }

  /**
   * Send poll
   */
  async sendPoll(config: TelegramConfig, messageData: TelegramMessage): Promise<TelegramResponse> {
    if (!messageData.poll_question || !messageData.poll_options) {
      throw new Error('Poll question and options are required for poll messages')
    }

    if (!this.isValidChatId(config.chat_id)) {
      throw new Error('Invalid chat ID format')
    }

    // Parse poll options (one per line)
    const options = messageData.poll_options.split('\n').filter(option => option.trim())
    
    if (options.length < 2) {
      throw new Error('Poll must have at least 2 options')
    }

    if (options.length > 10) {
      throw new Error('Poll cannot have more than 10 options')
    }

    const requestData: Record<string, any> = {
      chat_id: config.chat_id,
      question: messageData.poll_question,
      options: options,
      is_anonymous: true,
      type: 'regular',
      allows_multiple_answers: false
    }

    if (messageData.silent) {
      requestData.disable_notification = true
    }

    if (messageData.protect_content) {
      requestData.protect_content = true
    }

    if (messageData.reply_to_message_id) {
      requestData.reply_to_message_id = parseInt(messageData.reply_to_message_id)
    }

    if (messageData.thread_id) {
      requestData.message_thread_id = parseInt(messageData.thread_id)
    }

    return this.makeRequest(config.bot_token, 'sendPoll', requestData)
  }

  /**
   * Get chat information
   */
  async getChat(config: TelegramConfig): Promise<TelegramResponse> {
    if (!this.isValidChatId(config.chat_id)) {
      throw new Error('Invalid chat ID format')
    }

    return this.makeRequest(config.bot_token, 'getChat', {
      chat_id: config.chat_id
    })
  }

  /**
   * Get chat member count (for groups)
   */
  async getChatMemberCount(config: TelegramConfig): Promise<TelegramResponse> {
    if (!this.isValidChatId(config.chat_id)) {
      throw new Error('Invalid chat ID format')
    }

    return this.makeRequest(config.bot_token, 'getChatMemberCount', {
      chat_id: config.chat_id
    })
  }

  /**
   * Set webhook for receiving updates
   */
  async setWebhook(token: string, webhookUrl: string, secretToken?: string): Promise<TelegramResponse> {
    const requestData: Record<string, any> = {
      url: webhookUrl,
      allowed_updates: ['message', 'callback_query', 'poll_answer'],
      drop_pending_updates: true
    }

    if (secretToken) {
      requestData.secret_token = secretToken
    }

    return this.makeRequest(token, 'setWebhook', requestData)
  }

  /**
   * Remove webhook
   */
  async deleteWebhook(token: string): Promise<TelegramResponse> {
    return this.makeRequest(token, 'deleteWebhook', {
      drop_pending_updates: true
    })
  }

  /**
   * Process template variables in message content
   */
  processTemplate(template: string, variables: Record<string, any>): string {
    let processed = template
    
    for (const [key, value] of Object.entries(variables)) {
      const regex = new RegExp(`{{\\s*${key}\\s*}}`, 'g')
      processed = processed.replace(regex, String(value))
    }

    return processed
  }

  /**
   * Main method to send any type of message
   */
  async sendTelegramMessage(
    config: TelegramConfig, 
    messageData: TelegramMessage, 
    templateVariables: Record<string, any> = {}
  ): Promise<TelegramResponse> {
    
    // Process template variables in message content
    const processedMessage = {
      ...messageData,
      message: this.processTemplate(messageData.message, templateVariables)
    }

    // Process template variables in other fields that might contain them
    if (processedMessage.photo_url) {
      processedMessage.photo_url = this.processTemplate(processedMessage.photo_url, templateVariables)
    }

    if (processedMessage.document_url) {
      processedMessage.document_url = this.processTemplate(processedMessage.document_url, templateVariables)
    }

    if (processedMessage.poll_question) {
      processedMessage.poll_question = this.processTemplate(processedMessage.poll_question, templateVariables)
    }

    // Send based on message type
    switch (processedMessage.message_type) {
      case 'text':
        return this.sendMessage(config, processedMessage)
      
      case 'photo':
        return this.sendPhoto(config, processedMessage)
      
      case 'document':
        return this.sendDocument(config, processedMessage)
      
      case 'location':
        return this.sendLocation(config, processedMessage)
      
      case 'poll':
        return this.sendPoll(config, processedMessage)
      
      default:
        throw new Error(`Unsupported message type: ${processedMessage.message_type}`)
    }
  }

  /**
   * Rate limiting helper - simple implementation
   */
  private rateLimitQueue: Array<{ timestamp: number; chatId: string }> = []
  
  checkRateLimit(chatId: string): boolean {
    const now = Date.now()
    const oneMinute = 60 * 1000
    
    // Clean old requests
    this.rateLimitQueue = this.rateLimitQueue.filter(item => now - item.timestamp < oneMinute)
    
    // Count requests for this chat in the last minute
    const recentRequests = this.rateLimitQueue.filter(item => item.chatId === chatId).length
    
    // Check if it's a group (starts with -100) - different limits
    const isGroup = chatId.startsWith('-100')
    const limit = isGroup ? 20 : 30 // 20 for groups, 30 for private chats per minute
    
    if (recentRequests >= limit) {
      return false // Rate limited
    }
    
    // Add this request to queue
    this.rateLimitQueue.push({ timestamp: now, chatId })
    return true // OK to proceed
  }

  /**
   * Helper method to escape MarkdownV2 special characters
   */
  escapeMarkdownV2(text: string): string {
    const specialChars = ['_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!']
    let escaped = text
    
    for (const char of specialChars) {
      escaped = escaped.replace(new RegExp('\\' + char, 'g'), '\\' + char)
    }
    
    return escaped
  }

  /**
   * Generate helpful error messages for common issues
   */
  getErrorHelp(error: TelegramResponse): string {
    if (!error.description) return 'Unknown Telegram API error'

    const description = error.description.toLowerCase()

    if (description.includes('bot was blocked')) {
      return 'The user has blocked this bot. Remove them from your notification list.'
    }

    if (description.includes('chat not found')) {
      return 'Chat ID not found. Verify the chat ID is correct and the bot has access to this chat.'
    }

    if (description.includes('message is too long')) {
      return 'Message exceeds 4096 characters. Split into multiple messages or use document attachment.'
    }

    if (description.includes('too many requests')) {
      return 'Rate limit exceeded. Reduce message frequency or implement proper rate limiting.'
    }

    if (description.includes('unauthorized')) {
      return 'Invalid bot token. Check your bot token from @BotFather.'
    }

    if (description.includes('forbidden')) {
      return 'Bot lacks required permissions. Check bot settings and group permissions.'
    }

    if (description.includes('bad request')) {
      return 'Invalid request parameters. Check message format, chat ID, and other parameters.'
    }

    return `Telegram API Error: ${error.description}`
  }
}

// Export singleton instance
export const telegramService = new TelegramService()
export default telegramService