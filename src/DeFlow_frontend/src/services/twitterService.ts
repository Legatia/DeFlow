/**
 * Twitter/X API Service
 * Handles Twitter API v2 integration for posting tweets, replies, and DMs
 * Requires Twitter API v2 Bearer Token or OAuth 1.0a credentials
 */

export interface TwitterCredentials {
  api_key: string
  api_secret: string
  access_token: string
  access_token_secret: string
}

export interface TwitterPostConfig {
  text: string
  reply_to?: string
  media_urls?: string[]
  poll_options?: string[]
  poll_duration_minutes?: number
  hashtags?: string[]
  mentions?: string[]
}

export interface TwitterResponse {
  id?: string
  text?: string
  created_at?: string
  author_id?: string
  public_metrics?: {
    retweet_count: number
    like_count: number
    reply_count: number
    quote_count: number
  }
  error?: {
    code: number
    message: string
    details?: any
  }
}

class TwitterService {
  /**
   * Validate Twitter credentials format
   */
  private isValidCredentials(creds: TwitterCredentials): boolean {
    return !!(creds.api_key && creds.api_secret && creds.access_token && creds.access_token_secret)
  }

  /**
   * Process hashtags for Twitter
   */
  private processHashtags(hashtags: string[]): string {
    return hashtags
      .map(tag => tag.startsWith('#') ? tag : `#${tag}`)
      .join(' ')
  }

  /**
   * Process mentions for Twitter
   */
  private processMentions(mentions: string[]): string {
    return mentions
      .map(mention => mention.startsWith('@') ? mention : `@${mention}`)
      .join(' ')
  }

  /**
   * Build tweet text with hashtags and mentions
   */
  private buildTweetText(config: TwitterPostConfig): string {
    let text = config.text

    // Add mentions at the beginning if any
    if (config.mentions && config.mentions.length > 0) {
      const mentionText = this.processMentions(config.mentions)
      text = `${mentionText} ${text}`
    }

    // Add hashtags at the end if any
    if (config.hashtags && config.hashtags.length > 0) {
      const hashtagText = this.processHashtags(config.hashtags)
      text = `${text} ${hashtagText}`
    }

    // Ensure we don't exceed Twitter's character limit
    if (text.length > 280) {
      console.warn('Tweet text exceeds 280 characters, will be truncated')
      text = text.substring(0, 277) + '...'
    }

    return text
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

  /**
   * Validate Twitter API credentials
   */
  async validateCredentials(credentials: TwitterCredentials): Promise<{ valid: boolean; error?: string }> {
    if (!this.isValidCredentials(credentials)) {
      return {
        valid: false,
        error: 'All Twitter API credentials are required (API Key, API Secret, Access Token, Access Token Secret)'
      }
    }

    try {
      // Test credentials by attempting to get user info
      const response = await this.makeTwitterRequest(
        'GET',
        'https://api.twitter.com/2/users/me',
        credentials
      )

      if (response.error) {
        return {
          valid: false,
          error: response.error.message
        }
      }

      return { valid: true }
    } catch (error) {
      return {
        valid: false,
        error: error instanceof Error ? error.message : 'Network error'
      }
    }
  }

  /**
   * Post a tweet
   */
  async postTweet(
    credentials: TwitterCredentials, 
    config: TwitterPostConfig, 
    templateVariables: Record<string, any> = {}
  ): Promise<TwitterResponse> {
    
    if (!this.isValidCredentials(credentials)) {
      return {
        error: {
          code: 400,
          message: 'Invalid Twitter credentials'
        }
      }
    }

    // Process template variables
    const processedConfig = {
      ...config,
      text: this.processTemplate(config.text, templateVariables)
    }

    // Build final tweet text
    const tweetText = this.buildTweetText(processedConfig)

    // Prepare tweet payload
    const payload: any = {
      text: tweetText
    }

    // Add reply configuration
    if (config.reply_to) {
      payload.reply = {
        in_reply_to_tweet_id: config.reply_to
      }
    }

    // Add poll configuration
    if (config.poll_options && config.poll_options.length > 0) {
      payload.poll = {
        options: config.poll_options.slice(0, 4), // Twitter allows max 4 poll options
        duration_minutes: config.poll_duration_minutes || 1440 // 24 hours default
      }
    }

    return this.makeTwitterRequest('POST', 'https://api.twitter.com/2/tweets', credentials, payload)
  }

  /**
   * Send a direct message
   */
  async sendDirectMessage(
    credentials: TwitterCredentials,
    recipientId: string,
    text: string,
    templateVariables: Record<string, any> = {}
  ): Promise<TwitterResponse> {
    
    if (!this.isValidCredentials(credentials)) {
      return {
        error: {
          code: 400,
          message: 'Invalid Twitter credentials'
        }
      }
    }

    const processedText = this.processTemplate(text, templateVariables)

    const payload = {
      type: 'MessageCreate',
      message_create: {
        target: {
          recipient_id: recipientId
        },
        message_data: {
          text: processedText
        }
      }
    }

    return this.makeTwitterRequest('POST', 'https://api.twitter.com/1.1/direct_messages/events/new.json', credentials, payload)
  }

  /**
   * Make authenticated request to Twitter API
   */
  private async makeTwitterRequest(
    method: string, 
    url: string, 
    credentials: TwitterCredentials, 
    payload?: any
  ): Promise<TwitterResponse> {
    
    try {
      // Generate OAuth 1.0a signature (simplified version)
      const headers = this.generateOAuthHeaders(method, url, credentials, payload)

      const options: RequestInit = {
        method,
        headers: {
          'Content-Type': 'application/json',
          ...headers
        }
      }

      if (payload && (method === 'POST' || method === 'PUT')) {
        options.body = JSON.stringify(payload)
      }

      const response = await fetch(url, options)

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({ message: response.statusText }))
        return {
          error: {
            code: response.status,
            message: errorData.message || `HTTP ${response.status}: ${response.statusText}`,
            details: errorData
          }
        }
      }

      const data = await response.json()
      return data.data || data

    } catch (error) {
      console.error('Twitter API request failed:', error)
      return {
        error: {
          code: 0,
          message: error instanceof Error ? error.message : 'Network error'
        }
      }
    }
  }

  /**
   * Generate OAuth 1.0a headers (simplified implementation)
   * Note: In a real implementation, you'd want to use a proper OAuth library
   */
  private generateOAuthHeaders(
    method: string, 
    url: string, 
    credentials: TwitterCredentials, 
    payload?: any
  ): Record<string, string> {
    
    // This is a simplified implementation
    // In production, use a proper OAuth 1.0a library like 'oauth-1.0a'
    
    const timestamp = Math.floor(Date.now() / 1000).toString()
    const nonce = Math.random().toString(36).substring(2, 15)

    // For simplicity, we'll use Bearer Token authentication for read operations
    // and OAuth 1.0a for write operations (posting tweets)
    
    return {
      'Authorization': `OAuth oauth_consumer_key="${credentials.api_key}", oauth_token="${credentials.access_token}", oauth_signature_method="HMAC-SHA1", oauth_timestamp="${timestamp}", oauth_nonce="${nonce}", oauth_version="1.0"`
    }
  }

  /**
   * Rate limiting helper
   */
  private rateLimitQueue: Array<{ timestamp: number; endpoint: string }> = []
  
  checkRateLimit(endpoint: string): boolean {
    const now = Date.now()
    const fifteenMinutes = 15 * 60 * 1000

    // Clean old requests
    this.rateLimitQueue = this.rateLimitQueue.filter(item => now - item.timestamp < fifteenMinutes)

    // Count requests for this endpoint in the last 15 minutes
    const recentRequests = this.rateLimitQueue.filter(item => item.endpoint === endpoint).length

    // Twitter API v2 limits vary by endpoint
    const limits: Record<string, number> = {
      'tweets': 300,      // 300 tweets per 15 minutes
      'users/me': 75,     // 75 requests per 15 minutes
      'dm': 15000         // 15,000 DMs per 24 hours (simplified to 15 min window)
    }

    const limit = limits[endpoint] || 100 // Default limit

    if (recentRequests >= limit) {
      return false // Rate limited
    }

    // Add this request to queue
    this.rateLimitQueue.push({ timestamp: now, endpoint })
    return true // OK to proceed
  }

  /**
   * Get helpful error messages for common Twitter API issues
   */
  getErrorHelp(error: TwitterResponse['error']): string {
    if (!error) return 'Unknown Twitter error'

    switch (error.code) {
      case 400:
        return 'Bad request - Check your tweet content and parameters'
      
      case 401:
        return 'Unauthorized - Check your API credentials and permissions'
      
      case 403:
        return 'Forbidden - Your account may be suspended or lack permissions'
      
      case 429:
        return 'Rate limited - You are posting too frequently. Please wait before trying again'
      
      case 422:
        return 'Unprocessable entity - Tweet content may be invalid or duplicate'
      
      default:
        return `Twitter API Error ${error.code}: ${error.message}`
    }
  }
}

// Export singleton instance
export const twitterService = new TwitterService()
export default twitterService