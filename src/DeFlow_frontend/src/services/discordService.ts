/**
 * Discord Webhook Service
 * Handles Discord webhook integration for sending messages, embeds, and files
 * Based on the comprehensive DISCORD_INTEGRATION_GUIDE.md
 */

export interface DiscordWebhookConfig {
  webhook_url: string
}

export interface DiscordEmbedField {
  name: string
  value: string
  inline?: boolean
}

export interface DiscordEmbed {
  title?: string
  description?: string
  color?: number
  fields?: DiscordEmbedField[]
  thumbnail?: { url: string }
  image?: { url: string }
  footer?: {
    text: string
    icon_url?: string
  }
  timestamp?: string
}

export interface DiscordMessage {
  message_type: 'text' | 'embed' | 'file'
  content?: string
  username?: string
  avatar_url?: string
  embeds?: DiscordEmbed[]
  file_url?: string
  file_name?: string
  mentions?: 'none' | 'here' | 'everyone' | 'custom'
  custom_mentions?: string
  thread_name?: string
  suppress_embeds?: boolean
}

export interface DiscordResponse {
  id?: string
  channel_id?: string
  guild_id?: string
  webhook_id?: string
  type?: number
  timestamp?: string
  edited_timestamp?: string | null
  content?: string
  embeds?: DiscordEmbed[]
  error?: {
    code: number
    message: string
  }
}

class DiscordService {
  /**
   * Validate webhook URL format
   */
  private isValidWebhookUrl(url: string): boolean {
    const webhookPattern = /^https:\/\/discord\.com\/api\/webhooks\/\d+\/[a-zA-Z0-9_-]+$/
    return webhookPattern.test(url)
  }

  /**
   * Convert hex color to Discord color integer
   */
  private hexToDiscordColor(hex: string): number {
    // Remove # if present
    const cleanHex = hex.replace('#', '')
    
    // Validate hex format
    if (!/^[0-9A-Fa-f]{6}$/.test(cleanHex)) {
      return 5865470 // Default Discord blurple
    }
    
    return parseInt(cleanHex, 16)
  }

  /**
   * Process Discord color selection
   */
  private processEmbedColor(color: string, customColor?: string): number {
    if (color === 'custom' && customColor) {
      return this.hexToDiscordColor(customColor)
    }
    
    // Return preset color as integer
    return parseInt(color, 10)
  }

  /**
   * Parse embed fields from JSON string
   */
  private parseEmbedFields(fieldsJson: string): DiscordEmbedField[] {
    if (!fieldsJson.trim()) return []
    
    try {
      const fields = JSON.parse(fieldsJson)
      
      if (!Array.isArray(fields)) {
        console.warn('Embed fields must be an array')
        return []
      }
      
      return fields.filter(field => 
        field && 
        typeof field.name === 'string' && 
        typeof field.value === 'string'
      ).slice(0, 25) // Discord limit is 25 fields
    } catch (error) {
      console.warn('Invalid embed fields JSON:', error)
      return []
    }
  }

  /**
   * Process mentions for Discord message
   */
  private processMentions(mentions: string, customMentions?: string): any {
    const allowedMentions: any = {
      parse: []
    }

    switch (mentions) {
      case 'here':
        allowedMentions.parse = ['everyone'] // Discord treats @here as part of everyone
        break
      
      case 'everyone':
        allowedMentions.parse = ['everyone']
        break
      
      case 'custom':
        if (customMentions) {
          const mentionIds = customMentions
            .split(',')
            .map(id => id.trim())
            .filter(id => /^\d{17,19}$/.test(id)) // Discord snowflake format
          
          allowedMentions.users = mentionIds
          allowedMentions.parse = ['users']
        }
        break
      
      default:
        // No mentions
        allowedMentions.parse = []
    }

    return allowedMentions
  }

  /**
   * Build Discord message payload
   */
  private buildMessagePayload(config: DiscordWebhookConfig, message: DiscordMessage, templateVariables: Record<string, any> = {}): any {
    const payload: any = {}

    // Process template variables in content
    let processedContent = message.content || ''
    for (const [key, value] of Object.entries(templateVariables)) {
      const regex = new RegExp(`{{\\s*${key}\\s*}}`, 'g')
      processedContent = processedContent.replace(regex, String(value))
    }

    // Add basic webhook properties
    if (processedContent.trim()) {
      payload.content = processedContent
    }

    if (message.username) {
      payload.username = message.username
    }

    if (message.avatar_url) {
      payload.avatar_url = message.avatar_url
    }

    // Add mentions
    if (message.mentions && message.mentions !== 'none') {
      payload.allowed_mentions = this.processMentions(message.mentions, message.custom_mentions)
      
      // Add mention prefixes to content if needed
      if (message.mentions === 'here' && !processedContent.includes('@here')) {
        payload.content = `@here ${processedContent}`
      } else if (message.mentions === 'everyone' && !processedContent.includes('@everyone')) {
        payload.content = `@everyone ${processedContent}`
      }
    }

    // Handle message type specific properties
    if (message.message_type === 'embed' && message.embeds && message.embeds.length > 0) {
      payload.embeds = message.embeds.map(embed => ({
        ...embed,
        timestamp: embed.timestamp || new Date().toISOString()
      }))
    }

    // Add flags if needed
    if (message.suppress_embeds) {
      payload.flags = 1 << 2 // SUPPRESS_EMBEDS flag
    }

    // Thread creation (for forum channels)
    if (message.thread_name) {
      payload.thread_name = message.thread_name
    }

    return payload
  }

  /**
   * Make request to Discord webhook
   */
  private async makeWebhookRequest(webhookUrl: string, payload: any): Promise<DiscordResponse> {
    try {
      const response = await fetch(webhookUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(payload)
      })

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({ message: response.statusText }))
        return {
          error: {
            code: response.status,
            message: errorData.message || `HTTP ${response.status}: ${response.statusText}`
          }
        }
      }

      // Discord webhook returns 204 No Content on success, or message data
      if (response.status === 204) {
        return { id: 'success' }
      }

      return await response.json()
    } catch (error) {
      console.error('Discord webhook request failed:', error)
      return {
        error: {
          code: 0,
          message: error instanceof Error ? error.message : 'Network error'
        }
      }
    }
  }

  /**
   * Test webhook connectivity
   */
  async testWebhook(webhookUrl: string): Promise<{ valid: boolean; error?: string }> {
    if (!this.isValidWebhookUrl(webhookUrl)) {
      return {
        valid: false,
        error: 'Invalid webhook URL format. Should be: https://discord.com/api/webhooks/ID/TOKEN'
      }
    }

    const testPayload = {
      content: '🧪 **Discord Webhook Test**\n\n✅ Your webhook is working correctly!\n\nYou can now use this webhook in your DeFlow workflows.',
      username: 'DeFlow Test',
      embeds: [{
        title: 'Webhook Connection Successful',
        description: 'This is a test message from DeFlow to verify your Discord webhook integration.',
        color: 5865470,
        fields: [{
          name: '📊 Status',
          value: 'Connected',
          inline: true
        }, {
          name: '⏰ Time',
          value: new Date().toLocaleString(),
          inline: true
        }],
        footer: {
          text: 'DeFlow • Discord Integration Test'
        },
        timestamp: new Date().toISOString()
      }]
    }

    const response = await this.makeWebhookRequest(webhookUrl, testPayload)
    
    if (response.error) {
      return {
        valid: false,
        error: response.error.message
      }
    }

    return { valid: true }
  }

  /**
   * Send text message
   */
  async sendTextMessage(config: DiscordWebhookConfig, message: DiscordMessage, templateVariables: Record<string, any> = {}): Promise<DiscordResponse> {
    if (!this.isValidWebhookUrl(config.webhook_url)) {
      return {
        error: {
          code: 400,
          message: 'Invalid webhook URL format'
        }
      }
    }

    const payload = this.buildMessagePayload(config, message, templateVariables)
    return this.makeWebhookRequest(config.webhook_url, payload)
  }

  /**
   * Send rich embed message
   */
  async sendEmbedMessage(config: DiscordWebhookConfig, message: DiscordMessage, embedConfig: any, templateVariables: Record<string, any> = {}): Promise<DiscordResponse> {
    if (!this.isValidWebhookUrl(config.webhook_url)) {
      return {
        error: {
          code: 400,
          message: 'Invalid webhook URL format'
        }
      }
    }

    // Build embed from configuration
    const embed: DiscordEmbed = {}

    // Process template variables in embed fields
    const processTemplate = (text: string): string => {
      let processed = text
      for (const [key, value] of Object.entries(templateVariables)) {
        const regex = new RegExp(`{{\\s*${key}\\s*}}`, 'g')
        processed = processed.replace(regex, String(value))
      }
      return processed
    }

    if (embedConfig.embed_title) {
      embed.title = processTemplate(embedConfig.embed_title)
    }

    if (embedConfig.embed_description) {
      embed.description = processTemplate(embedConfig.embed_description)
    }

    // Process color
    embed.color = this.processEmbedColor(embedConfig.embed_color, embedConfig.embed_color_custom)

    // Add fields
    if (embedConfig.embed_fields) {
      const fields = this.parseEmbedFields(embedConfig.embed_fields)
      if (fields.length > 0) {
        embed.fields = fields.map(field => ({
          name: processTemplate(field.name),
          value: processTemplate(field.value),
          inline: field.inline || false
        }))
      }
    }

    // Add thumbnail
    if (embedConfig.embed_thumbnail) {
      embed.thumbnail = { url: processTemplate(embedConfig.embed_thumbnail) }
    }

    // Add image
    if (embedConfig.embed_image) {
      embed.image = { url: processTemplate(embedConfig.embed_image) }
    }

    // Add footer
    if (embedConfig.embed_footer) {
      embed.footer = {
        text: processTemplate(embedConfig.embed_footer)
      }
      
      if (embedConfig.embed_footer_icon) {
        embed.footer.icon_url = processTemplate(embedConfig.embed_footer_icon)
      }
    }

    // Add timestamp
    embed.timestamp = new Date().toISOString()

    // Create message with embed
    const embedMessage: DiscordMessage = {
      ...message,
      message_type: 'embed',
      embeds: [embed]
    }

    const payload = this.buildMessagePayload(config, embedMessage, templateVariables)
    return this.makeWebhookRequest(config.webhook_url, payload)
  }

  /**
   * Send file attachment
   */
  async sendFileMessage(config: DiscordWebhookConfig, message: DiscordMessage, templateVariables: Record<string, any> = {}): Promise<DiscordResponse> {
    if (!this.isValidWebhookUrl(config.webhook_url)) {
      return {
        error: {
          code: 400,
          message: 'Invalid webhook URL format'
        }
      }
    }

    if (!message.file_url) {
      return {
        error: {
          code: 400,
          message: 'File URL is required for file messages'
        }
      }
    }

    try {
      // Fetch the file
      const fileResponse = await fetch(message.file_url)
      if (!fileResponse.ok) {
        return {
          error: {
            code: 400,
            message: `Failed to fetch file: ${fileResponse.statusText}`
          }
        }
      }

      const fileBlob = await fileResponse.blob()
      const fileName = message.file_name || message.file_url.split('/').pop() || 'attachment'

      // Create form data for file upload
      const formData = new FormData()
      formData.append('file', fileBlob, fileName)

      // Build payload
      const payload = this.buildMessagePayload(config, message, templateVariables)
      formData.append('payload_json', JSON.stringify(payload))

      // Send with multipart form data
      const response = await fetch(config.webhook_url, {
        method: 'POST',
        body: formData
      })

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({ message: response.statusText }))
        return {
          error: {
            code: response.status,
            message: errorData.message || `HTTP ${response.status}: ${response.statusText}`
          }
        }
      }

      if (response.status === 204) {
        return { id: 'success' }
      }

      return await response.json()
    } catch (error) {
      console.error('Discord file upload failed:', error)
      return {
        error: {
          code: 0,
          message: error instanceof Error ? error.message : 'File upload failed'
        }
      }
    }
  }

  /**
   * Main method to send Discord message
   */
  async sendDiscordMessage(
    config: DiscordWebhookConfig,
    messageConfig: any,
    templateVariables: Record<string, any> = {}
  ): Promise<DiscordResponse> {
    
    const message: DiscordMessage = {
      message_type: messageConfig.message_type || 'text',
      content: messageConfig.message,
      username: messageConfig.username,
      avatar_url: messageConfig.avatar_url,
      mentions: messageConfig.mentions,
      custom_mentions: messageConfig.custom_mentions,
      thread_name: messageConfig.thread_name,
      suppress_embeds: messageConfig.suppress_embeds,
      file_url: messageConfig.file_url,
      file_name: messageConfig.file_name
    }

    switch (message.message_type) {
      case 'text':
        return this.sendTextMessage(config, message, templateVariables)
      
      case 'embed':
        return this.sendEmbedMessage(config, message, messageConfig, templateVariables)
      
      case 'file':
        return this.sendFileMessage(config, message, templateVariables)
      
      default:
        return {
          error: {
            code: 400,
            message: `Unsupported message type: ${message.message_type}`
          }
        }
    }
  }

  /**
   * Rate limiting helper - simple implementation
   */
  private rateLimitQueue: Array<{ timestamp: number; webhookUrl: string }> = []
  
  checkRateLimit(webhookUrl: string): boolean {
    const now = Date.now()
    const oneMinute = 60 * 1000
    
    // Clean old requests
    this.rateLimitQueue = this.rateLimitQueue.filter(item => now - item.timestamp < oneMinute)
    
    // Count requests for this webhook in the last minute
    const recentRequests = this.rateLimitQueue.filter(item => item.webhookUrl === webhookUrl).length
    
    // Discord webhook limit is 30 requests per minute
    if (recentRequests >= 30) {
      return false // Rate limited
    }
    
    // Add this request to queue
    this.rateLimitQueue.push({ timestamp: now, webhookUrl })
    return true // OK to proceed
  }

  /**
   * Generate helpful error messages for common Discord issues
   */
  getErrorHelp(error: DiscordResponse['error']): string {
    if (!error) return 'Unknown Discord error'

    switch (error.code) {
      case 400:
        return 'Bad request - Check your message format and webhook URL'
      
      case 401:
        return 'Unauthorized - Your webhook URL may be invalid or expired'
      
      case 403:
        return 'Forbidden - The webhook lacks permissions to post to this channel'
      
      case 404:
        return 'Webhook not found - The webhook may have been deleted or URL is incorrect'
      
      case 429:
        return 'Rate limited - You are sending messages too quickly. Please wait before trying again'
      
      case 50027:
        return 'Invalid webhook token - Please regenerate your webhook URL in Discord'
      
      case 50035:
        return 'Invalid form body - Check your embed structure and field limits'
      
      default:
        return `Discord API Error ${error.code}: ${error.message}`
    }
  }

  /**
   * Process template variables in text
   */
  processTemplate(template: string, variables: Record<string, any>): string {
    let processed = template
    
    for (const [key, value] of Object.entries(variables)) {
      const regex = new RegExp(`{{\\s*${key}\\s*}}`, 'g')
      processed = processed.replace(regex, String(value))
    }

    return processed
  }
}

// Export singleton instance
export const discordService = new DiscordService()
export default discordService