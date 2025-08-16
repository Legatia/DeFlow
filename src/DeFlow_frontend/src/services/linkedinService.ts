// LinkedIn API service for DeFlow
export interface LinkedInCredentials {
  access_token: string
  token_type?: string
  expires_in?: number
  scope?: string
}

export interface LinkedInPostConfig {
  text: string
  media_urls?: string[]
  article_url?: string
  article_title?: string
  article_description?: string
  hashtags?: string[]
  mentions?: string[]
  post_type?: 'person' | 'organization'
  organization_id?: string
}

export interface LinkedInResponse {
  id?: string
  urn?: string
  created_at?: string
  text?: string
  success?: boolean
  error?: {
    code: number
    message: string
    details?: string
  }
}

export interface LinkedInProfile {
  id: string
  firstName: string
  lastName: string
  profilePicture?: string
  emailAddress?: string
}

class LinkedInService {
  private readonly API_BASE = 'https://api.linkedin.com/v2'
  private readonly CHAR_LIMIT = 3000

  /**
   * Post content to LinkedIn personal profile or organization page
   */
  async postToLinkedIn(
    credentials: LinkedInCredentials,
    config: LinkedInPostConfig,
    templateVariables: Record<string, any> = {}
  ): Promise<LinkedInResponse> {
    try {
      // Validate credentials
      const tokenValidation = await this.validateCredentials(credentials)
      if (!tokenValidation.valid) {
        return {
          success: false,
          error: {
            code: 401,
            message: tokenValidation.error || 'Invalid LinkedIn credentials'
          }
        }
      }

      // Process template variables in text
      const processedText = this.processTemplate(config.text, templateVariables)
      
      // Validate text length
      if (processedText.length > this.CHAR_LIMIT) {
        return {
          success: false,
          error: {
            code: 422,
            message: `Text exceeds LinkedIn's ${this.CHAR_LIMIT} character limit`
          }
        }
      }

      // Determine author URN
      let authorUrn: string
      if (config.post_type === 'organization' && config.organization_id) {
        authorUrn = `urn:li:organization:${config.organization_id}`
      } else {
        // Get user URN for personal posts
        const profile = await this.getUserProfile(credentials)
        if (!profile) {
          return {
            success: false,
            error: {
              code: 400,
              message: 'Unable to retrieve user profile'
            }
          }
        }
        authorUrn = `urn:li:person:${profile.id}`
      }

      // Build post payload
      const payload = this.buildPostPayload(authorUrn, processedText, config)

      // Make API request
      const response = await fetch(`${this.API_BASE}/ugcPosts`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${credentials.access_token}`,
          'Content-Type': 'application/json',
          'X-Restli-Protocol-Version': '2.0.0'
        },
        body: JSON.stringify(payload)
      })

      const responseData = await response.json()

      if (!response.ok) {
        return {
          success: false,
          error: {
            code: response.status,
            message: responseData.message || 'LinkedIn API error',
            details: JSON.stringify(responseData)
          }
        }
      }

      return {
        success: true,
        id: responseData.id,
        urn: responseData.id,
        created_at: new Date().toISOString(),
        text: processedText
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
   * Validate LinkedIn credentials
   */
  async validateCredentials(credentials: LinkedInCredentials): Promise<{ valid: boolean; error?: string }> {
    try {
      const response = await fetch(`${this.API_BASE}/people/~`, {
        headers: {
          'Authorization': `Bearer ${credentials.access_token}`,
          'Content-Type': 'application/json'
        }
      })

      if (response.ok) {
        return { valid: true }
      } else {
        const errorData = await response.json()
        return { 
          valid: false, 
          error: errorData.message || `HTTP ${response.status}: Invalid token`
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
   * Get user profile information
   */
  async getUserProfile(credentials: LinkedInCredentials): Promise<LinkedInProfile | null> {
    try {
      const response = await fetch(`${this.API_BASE}/people/~?projection=(id,firstName,lastName,profilePicture(displayImage~:playableStreams))`, {
        headers: {
          'Authorization': `Bearer ${credentials.access_token}`,
          'Content-Type': 'application/json'
        }
      })

      if (response.ok) {
        const data = await response.json()
        return {
          id: data.id,
          firstName: data.firstName?.localized?.en_US || '',
          lastName: data.lastName?.localized?.en_US || '',
          profilePicture: data.profilePicture?.displayImage?.elements?.[0]?.identifiers?.[0]?.identifier
        }
      }
      
      return null
    } catch (error) {
      console.error('Error fetching LinkedIn profile:', error)
      return null
    }
  }

  /**
   * Get organization pages the user can post to
   */
  async getOrganizations(credentials: LinkedInCredentials): Promise<Array<{id: string, name: string}>> {
    try {
      const response = await fetch(`${this.API_BASE}/organizationAcls?q=roleAssignee&projection=(elements*(organization~(id,name)))`, {
        headers: {
          'Authorization': `Bearer ${credentials.access_token}`,
          'Content-Type': 'application/json'
        }
      })

      if (response.ok) {
        const data = await response.json()
        return data.elements?.map((element: any) => ({
          id: element.organization?.id,
          name: element.organization?.name
        })).filter((org: any) => org.id && org.name) || []
      }
      
      return []
    } catch (error) {
      console.error('Error fetching LinkedIn organizations:', error)
      return []
    }
  }

  /**
   * Build LinkedIn post payload
   */
  private buildPostPayload(authorUrn: string, text: string, config: LinkedInPostConfig) {
    const payload: any = {
      author: authorUrn,
      lifecycleState: 'PUBLISHED',
      specificContent: {
        'com.linkedin.ugc.ShareContent': {
          shareCommentary: {
            text: text
          },
          shareMediaCategory: 'NONE'
        }
      },
      visibility: {
        'com.linkedin.ugc.MemberNetworkVisibility': 'PUBLIC'
      }
    }

    // Add article link if provided
    if (config.article_url) {
      payload.specificContent['com.linkedin.ugc.ShareContent'].shareMediaCategory = 'ARTICLE'
      payload.specificContent['com.linkedin.ugc.ShareContent'].media = [{
        status: 'READY',
        originalUrl: config.article_url,
        title: {
          text: config.article_title || config.article_url
        },
        description: {
          text: config.article_description || ''
        }
      }]
    }

    // Add image media if provided
    if (config.media_urls && config.media_urls.length > 0) {
      payload.specificContent['com.linkedin.ugc.ShareContent'].shareMediaCategory = 'IMAGE'
      payload.specificContent['com.linkedin.ugc.ShareContent'].media = config.media_urls.map(url => ({
        status: 'READY',
        originalUrl: url
      }))
    }

    return payload
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
   * Check if we're hitting rate limits
   */
  checkRateLimit(): boolean {
    // LinkedIn allows 100 posts per day for personal profiles
    // 25 posts per day for organization pages
    // For now, just return true - implement proper rate limiting later
    return true
  }

  /**
   * Get LinkedIn authorization URL for OAuth flow
   */
  getAuthorizationUrl(clientId: string, redirectUri: string, scopes: string[] = ['r_liteprofile', 'r_emailaddress', 'w_member_social']): string {
    const scope = scopes.join(' ')
    const state = Math.random().toString(36).substring(2, 15)
    
    return `https://www.linkedin.com/oauth/v2/authorization?` +
      `response_type=code&` +
      `client_id=${encodeURIComponent(clientId)}&` +
      `redirect_uri=${encodeURIComponent(redirectUri)}&` +
      `scope=${encodeURIComponent(scope)}&` +
      `state=${encodeURIComponent(state)}`
  }

  /**
   * Exchange authorization code for access token
   */
  async exchangeCodeForToken(
    clientId: string,
    clientSecret: string,
    redirectUri: string,
    code: string
  ): Promise<LinkedInCredentials | null> {
    try {
      const response = await fetch('https://www.linkedin.com/oauth/v2/accessToken', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded'
        },
        body: new URLSearchParams({
          grant_type: 'authorization_code',
          code: code,
          client_id: clientId,
          client_secret: clientSecret,
          redirect_uri: redirectUri
        })
      })

      if (response.ok) {
        const data = await response.json()
        return {
          access_token: data.access_token,
          token_type: data.token_type,
          expires_in: data.expires_in,
          scope: data.scope
        }
      }

      return null
    } catch (error) {
      console.error('Error exchanging LinkedIn code for token:', error)
      return null
    }
  }
}

// Export singleton instance
export const linkedinService = new LinkedInService()
export default linkedinService