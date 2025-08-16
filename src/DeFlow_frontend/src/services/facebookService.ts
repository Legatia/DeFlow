// Facebook API service for DeFlow
export interface FacebookCredentials {
  access_token: string
  page_id?: string
  app_id?: string
  app_secret?: string
}

export interface FacebookPostConfig {
  message: string
  link?: string
  picture?: string
  name?: string
  caption?: string
  description?: string
  place?: string
  tags?: string[]
  scheduled_publish_time?: number
  published?: boolean
  post_type?: 'page' | 'group' | 'event'
  target_id?: string // page_id, group_id, or event_id
}

export interface FacebookResponse {
  id?: string
  post_id?: string
  created_time?: string
  message?: string
  success?: boolean
  error?: {
    code: number
    message: string
    type?: string
    fbtrace_id?: string
    details?: string
  }
}

export interface FacebookPage {
  id: string
  name: string
  category: string
  access_token: string
  followers_count?: number
  picture?: string
}

class FacebookService {
  private readonly API_BASE = 'https://graph.facebook.com/v18.0'
  private readonly CHAR_LIMIT = 63206

  /**
   * Post content to Facebook page, group, or event
   */
  async postToFacebook(
    credentials: FacebookCredentials,
    config: FacebookPostConfig,
    templateVariables: Record<string, any> = {}
  ): Promise<FacebookResponse> {
    try {
      // Validate credentials
      const tokenValidation = await this.validateCredentials(credentials, config.target_id)
      if (!tokenValidation.valid) {
        return {
          success: false,
          error: {
            code: 401,
            message: tokenValidation.error || 'Invalid Facebook credentials'
          }
        }
      }

      // Process template variables in message
      const processedMessage = this.processTemplate(config.message, templateVariables)
      
      // Validate message length
      if (processedMessage.length > this.CHAR_LIMIT) {
        return {
          success: false,
          error: {
            code: 422,
            message: `Message exceeds Facebook's ${this.CHAR_LIMIT} character limit`
          }
        }
      }

      // Check rate limits
      if (!this.checkRateLimit(config.post_type || 'page', config.target_id || credentials.page_id || '')) {
        return {
          success: false,
          error: {
            code: 429,
            message: 'Rate limit exceeded. Please wait before posting again.'
          }
        }
      }

      // Determine endpoint and build payload
      const { endpoint, payload } = this.buildRequestData(credentials, config, processedMessage)

      // Make API request
      const response = await fetch(endpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(payload)
      })

      const responseData = await response.json()

      if (!response.ok) {
        return {
          success: false,
          error: {
            code: response.status,
            message: responseData.error?.message || 'Facebook API error',
            type: responseData.error?.type,
            fbtrace_id: responseData.error?.fbtrace_id,
            details: JSON.stringify(responseData)
          }
        }
      }

      return {
        success: true,
        id: responseData.id,
        post_id: responseData.post_id || responseData.id,
        created_time: new Date().toISOString(),
        message: processedMessage
      }

    } catch (error) {
      return {
        success: false,
        error: {
          code: 500,
          message: error instanceof Error ? error.message : 'Unknown error occurred'
        }
      }
    }
  }

  /**
   * Validate Facebook credentials and permissions
   */
  async validateCredentials(credentials: FacebookCredentials, targetId?: string): Promise<{ valid: boolean; error?: string }> {
    try {
      const pageId = targetId || credentials.page_id
      if (!pageId) {
        return { valid: false, error: 'Page ID is required for Facebook posting' }
      }

      const response = await fetch(`${this.API_BASE}/${pageId}?access_token=${credentials.access_token}`, {
        headers: {
          'Content-Type': 'application/json'
        }
      })

      if (response.ok) {
        return { valid: true }
      } else {
        const errorData = await response.json()
        return { 
          valid: false, 
          error: errorData.error?.message || `HTTP ${response.status}: Invalid token or permissions`
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
   * Get page information
   */
  async getPageInfo(credentials: FacebookCredentials, pageId?: string): Promise<FacebookPage | null> {
    try {
      const targetPageId = pageId || credentials.page_id
      if (!targetPageId) return null

      const response = await fetch(
        `${this.API_BASE}/${targetPageId}?fields=id,name,category,followers_count,picture&access_token=${credentials.access_token}`,
        {
          headers: {
            'Content-Type': 'application/json'
          }
        }
      )

      if (response.ok) {
        const data = await response.json()
        return {
          id: data.id,
          name: data.name,
          category: data.category,
          access_token: credentials.access_token,
          followers_count: data.followers_count,
          picture: data.picture?.data?.url
        }
      }
      
      return null
    } catch (error) {
      console.error('Error fetching Facebook page info:', error)
      return null
    }
  }

  /**
   * Get user's pages they can post to
   */
  async getUserPages(userAccessToken: string): Promise<FacebookPage[]> {
    try {
      const response = await fetch(
        `${this.API_BASE}/me/accounts?fields=id,name,category,access_token,followers_count&access_token=${userAccessToken}`,
        {
          headers: {
            'Content-Type': 'application/json'
          }
        }
      )

      if (response.ok) {
        const data = await response.json()
        return data.data?.map((page: any) => ({
          id: page.id,
          name: page.name,
          category: page.category,
          access_token: page.access_token,
          followers_count: page.followers_count
        })) || []
      }
      
      return []
    } catch (error) {
      console.error('Error fetching Facebook pages:', error)
      return []
    }
  }

  /**
   * Upload media to Facebook
   */
  async uploadMedia(credentials: FacebookCredentials, mediaUrl: string, mediaType: 'photo' | 'video' = 'photo'): Promise<string | null> {
    try {
      const pageId = credentials.page_id
      if (!pageId) return null

      const endpoint = mediaType === 'photo' 
        ? `${this.API_BASE}/${pageId}/photos`
        : `${this.API_BASE}/${pageId}/videos`

      const response = await fetch(endpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          url: mediaUrl,
          published: false, // Upload without publishing
          access_token: credentials.access_token
        })
      })

      if (response.ok) {
        const data = await response.json()
        return data.id
      }
      
      return null
    } catch (error) {
      console.error('Error uploading media to Facebook:', error)
      return null
    }
  }

  /**
   * Build request data based on post type
   */
  private buildRequestData(credentials: FacebookCredentials, config: FacebookPostConfig, message: string) {
    const targetId = config.target_id || credentials.page_id
    const postType = config.post_type || 'page'
    
    let endpoint: string
    let payload: any = {
      message: message,
      access_token: credentials.access_token
    }

    // Set endpoint based on post type
    switch (postType) {
      case 'group':
        endpoint = `${this.API_BASE}/${targetId}/feed`
        break
      case 'event':
        endpoint = `${this.API_BASE}/${targetId}/feed`
        break
      case 'page':
      default:
        endpoint = `${this.API_BASE}/${targetId}/feed`
        break
    }

    // Add optional fields
    if (config.link) payload.link = config.link
    if (config.picture) payload.picture = config.picture
    if (config.name) payload.name = config.name
    if (config.caption) payload.caption = config.caption
    if (config.description) payload.description = config.description
    if (config.place) payload.place = config.place
    if (config.scheduled_publish_time) {
      payload.scheduled_publish_time = config.scheduled_publish_time
      payload.published = false
    } else {
      payload.published = config.published !== false
    }

    return { endpoint, payload }
  }

  /**
   * Process template variables in text
   */
  private processTemplate(text: string, variables: Record<string, any>): string {
    let processedText = text

    // Replace template variables like {{variable_name}}
    Object.entries(variables).forEach(([key, value]) => {
      const regex = new RegExp(`{{\\s*${key}\\s*}}`, 'g')
      processedText = processedText.replace(regex, String(value))
    })

    // Replace common date/time variables
    const now = new Date()
    processedText = processedText.replace(/{{date}}/g, now.toLocaleDateString())
    processedText = processedText.replace(/{{time}}/g, now.toLocaleTimeString())
    processedText = processedText.replace(/{{datetime}}/g, now.toLocaleString())

    return processedText
  }

  /**
   * Check rate limits for Facebook API
   */
  private checkRateLimit(postType: string, targetId: string): boolean {
    // Facebook rate limits:
    // - Page posts: 200 per hour per page
    // - Group posts: 50 per hour per group
    // - API calls: 200 per hour per user per app
    
    // For now, just return true - implement proper rate limiting later
    return true
  }

  /**
   * Get Facebook authorization URL for OAuth flow
   */
  getAuthorizationUrl(appId: string, redirectUri: string, scopes: string[] = ['pages_manage_posts', 'pages_read_engagement']): string {
    const scope = scopes.join(',')
    const state = Math.random().toString(36).substring(2, 15)
    
    return `https://www.facebook.com/v18.0/dialog/oauth?` +
      `client_id=${encodeURIComponent(appId)}&` +
      `redirect_uri=${encodeURIComponent(redirectUri)}&` +
      `scope=${encodeURIComponent(scope)}&` +
      `response_type=code&` +
      `state=${encodeURIComponent(state)}`
  }

  /**
   * Exchange authorization code for access token
   */
  async exchangeCodeForToken(
    appId: string,
    appSecret: string,
    redirectUri: string,
    code: string
  ): Promise<FacebookCredentials | null> {
    try {
      const response = await fetch(
        `${this.API_BASE}/oauth/access_token?` +
        `client_id=${appId}&` +
        `client_secret=${appSecret}&` +
        `redirect_uri=${encodeURIComponent(redirectUri)}&` +
        `code=${code}`,
        {
          headers: {
            'Content-Type': 'application/json'
          }
        }
      )

      if (response.ok) {
        const data = await response.json()
        return {
          access_token: data.access_token,
          app_id: appId,
          app_secret: appSecret
        }
      }

      return null
    } catch (error) {
      console.error('Error exchanging Facebook code for token:', error)
      return null
    }
  }

  /**
   * Get long-lived access token from short-lived token
   */
  async getLongLivedToken(
    appId: string,
    appSecret: string,
    shortLivedToken: string
  ): Promise<string | null> {
    try {
      const response = await fetch(
        `${this.API_BASE}/oauth/access_token?` +
        `grant_type=fb_exchange_token&` +
        `client_id=${appId}&` +
        `client_secret=${appSecret}&` +
        `fb_exchange_token=${shortLivedToken}`,
        {
          headers: {
            'Content-Type': 'application/json'
          }
        }
      )

      if (response.ok) {
        const data = await response.json()
        return data.access_token
      }

      return null
    } catch (error) {
      console.error('Error getting long-lived Facebook token:', error)
      return null
    }
  }

  /**
   * Debug access token to check validity and permissions
   */
  async debugToken(accessToken: string, appAccessToken: string): Promise<any> {
    try {
      const response = await fetch(
        `${this.API_BASE}/debug_token?input_token=${accessToken}&access_token=${appAccessToken}`,
        {
          headers: {
            'Content-Type': 'application/json'
          }
        }
      )

      if (response.ok) {
        const data = await response.json()
        return data.data
      }

      return null
    } catch (error) {
      console.error('Error debugging Facebook token:', error)
      return null
    }
  }
}

// Export singleton instance
export const facebookService = new FacebookService()
export default facebookService