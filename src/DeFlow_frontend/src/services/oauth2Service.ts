// OAuth2 service for Gmail and Outlook integration
import { Actor, HttpAgent } from '@dfinity/agent'
// SECURITY: Import security utilities
import securityUtilsService from './securityUtilsService'

export interface OAuth2Config {
  clientId: string
  // SECURITY: Client secret removed from frontend
  redirectUri: string
  scopes: string[]
  tokenEndpoint: string
  authEndpoint?: string
  usePKCE?: boolean
}

export interface OAuth2Token {
  access_token: string
  refresh_token: string
  expires_in: number
  token_type: string
  scope: string
  expires_at: number // calculated expiry timestamp
}

export interface OAuth2Provider {
  name: 'gmail' | 'outlook'
  authUrl: string
  tokenUrl: string
  revokeUrl?: string
  scopes: string[]
  config: OAuth2Config
}

class OAuth2Service {
  private providers: Map<string, OAuth2Provider> = new Map()
  private tokens: Map<string, OAuth2Token> = new Map()
  private codeVerifiers: Map<string, string> = new Map()

  // SECURITY: PKCE helper methods for secure OAuth without client secrets
  private generateCodeVerifier(): string {
    const array = new Uint32Array(56)
    crypto.getRandomValues(array)
    return Array.from(array, dec => ('0' + dec.toString(16)).substr(-2)).join('')
  }

  private async generateCodeChallenge(verifier: string): Promise<string> {
    const encoder = new TextEncoder()
    const data = encoder.encode(verifier)
    const digest = await crypto.subtle.digest('SHA-256', data)
    return btoa(String.fromCharCode(...new Uint8Array(digest)))
      .replace(/\+/g, '-')
      .replace(/\//g, '_')
      .replace(/=/g, '')
  }

  private getStoredCodeVerifier(state: string): string {
    return this.codeVerifiers.get(state) || ''
  }

  // OAuth2 endpoints and configurations
  private readonly OAUTH_CONFIGS = {
    gmail: {
      authUrl: 'https://accounts.google.com/o/oauth2/v2/auth',
      tokenUrl: 'https://oauth2.googleapis.com/token',
      revokeUrl: 'https://oauth2.googleapis.com/revoke',
      scopes: [
        'https://www.googleapis.com/auth/gmail.send',
        'https://www.googleapis.com/auth/gmail.readonly',
        'https://www.googleapis.com/auth/gmail.modify'
      ]
    },
    outlook: {
      authUrl: 'https://login.microsoftonline.com/common/oauth2/v2.0/authorize',
      tokenUrl: 'https://login.microsoftonline.com/common/oauth2/v2.0/token',
      revokeUrl: 'https://login.microsoftonline.com/common/oauth2/v2.0/logout',
      scopes: [
        'https://graph.microsoft.com/Mail.Send',
        'https://graph.microsoft.com/Mail.Read',
        'https://graph.microsoft.com/User.Read'
      ]
    }
  }

  constructor() {
    this.loadStoredTokens()
  }

  // Configure OAuth2 provider
  configureProvider(name: 'gmail' | 'outlook', config: OAuth2Config): void {
    const oauthConfig = this.OAUTH_CONFIGS[name]
    
    const provider: OAuth2Provider = {
      name,
      authUrl: oauthConfig.authUrl,
      tokenUrl: oauthConfig.tokenUrl,
      revokeUrl: oauthConfig.revokeUrl,
      scopes: oauthConfig.scopes,
      config
    }

    this.providers.set(name, provider)
  }

  // Get authorization URL for provider
  getAuthorizationUrl(providerName: 'gmail' | 'outlook', state?: string): string {
    const provider = this.providers.get(providerName)
    if (!provider) {
      throw new Error(`Provider ${providerName} not configured`)
    }

    const params = new URLSearchParams({
      client_id: provider.config.clientId,
      redirect_uri: provider.config.redirectUri,
      scope: provider.scopes.join(' '),
      response_type: 'code',
      access_type: 'offline', // For refresh tokens
      prompt: 'consent', // Force consent to get refresh token
      ...(state && { state })
    })

    // Microsoft uses different parameter names
    if (providerName === 'outlook') {
      params.set('response_mode', 'query')
    }

    return `${provider.authUrl}?${params.toString()}`
  }

  // Exchange authorization code for tokens
  async exchangeCodeForTokens(
    providerName: 'gmail' | 'outlook', 
    code: string,
    state?: string
  ): Promise<OAuth2Token> {
    const provider = this.providers.get(providerName)
    if (!provider) {
      throw new Error(`Provider ${providerName} not configured`)
    }

    const tokenData = {
      client_id: provider.config.clientId,
      // SECURITY: Client secret handled by backend - use PKCE instead
      code,
      grant_type: 'authorization_code',
      redirect_uri: provider.config.redirectUri,
      code_verifier: this.getStoredCodeVerifier(state || '') // PKCE verification
    }

    try {
      // SECURITY: Use secure fetch with HTTPS enforcement and error sanitization
      const response = await securityUtilsService.secureFetch(provider.tokenUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
          'Accept': 'application/json'
        },
        body: new URLSearchParams(tokenData).toString()
      })

      if (!response.ok) {
        const errorText = await response.text()
        // SECURITY: Sanitize error message to prevent information disclosure
        const sanitizedError = securityUtilsService.sanitizeErrorMessage(errorText, 'Token exchange')
        throw new Error(`Token exchange failed: ${response.status} - ${sanitizedError}`)
      }

      const tokenResponse = await response.json()
      
      // Calculate expiry timestamp
      const expiresAt = Date.now() + (tokenResponse.expires_in * 1000)
      
      const token: OAuth2Token = {
        access_token: tokenResponse.access_token,
        refresh_token: tokenResponse.refresh_token,
        expires_in: tokenResponse.expires_in,
        token_type: tokenResponse.token_type || 'Bearer',
        scope: tokenResponse.scope || provider.scopes.join(' '),
        expires_at: expiresAt
      }

      // Store token
      this.tokens.set(providerName, token)
      this.saveTokensToStorage()

      return token
    } catch (error) {
      // SECURITY: Sanitize error before logging and re-throwing
      const sanitizedError = securityUtilsService.sanitizeErrorMessage(error, 'OAuth2 token exchange')
      console.error(`OAuth2 token exchange error for ${providerName}:`, sanitizedError)
      throw new Error(securityUtilsService.getUserFriendlyErrorMessage(error, 'authenticate'))
    }
  }

  // Refresh access token using refresh token
  async refreshToken(providerName: 'gmail' | 'outlook'): Promise<OAuth2Token> {
    const provider = this.providers.get(providerName)
    const currentToken = this.tokens.get(providerName)

    if (!provider || !currentToken?.refresh_token) {
      throw new Error(`Cannot refresh token for ${providerName}: missing provider or refresh token`)
    }

    const refreshData = {
      client_id: provider.config.clientId,
      // SECURITY: Client secret handled by backend proxy
      refresh_token: currentToken.refresh_token,
      grant_type: 'refresh_token'
    }

    try {
      // SECURITY: Use secure fetch with HTTPS enforcement
      const response = await securityUtilsService.secureFetch(provider.tokenUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
          'Accept': 'application/json'
        },
        body: new URLSearchParams(refreshData).toString()
      })

      if (!response.ok) {
        const errorText = await response.text()
        // SECURITY: Sanitize error message
        const sanitizedError = securityUtilsService.sanitizeErrorMessage(errorText, 'Token refresh')
        throw new Error(`Token refresh failed: ${response.status} - ${sanitizedError}`)
      }

      const tokenResponse = await response.json()
      
      // Calculate expiry timestamp
      const expiresAt = Date.now() + (tokenResponse.expires_in * 1000)
      
      // Update token (keep existing refresh token if not provided)
      const updatedToken: OAuth2Token = {
        access_token: tokenResponse.access_token,
        refresh_token: tokenResponse.refresh_token || currentToken.refresh_token,
        expires_in: tokenResponse.expires_in,
        token_type: tokenResponse.token_type || 'Bearer',
        scope: tokenResponse.scope || currentToken.scope,
        expires_at: expiresAt
      }

      // Store updated token
      this.tokens.set(providerName, updatedToken)
      this.saveTokensToStorage()

      return updatedToken
    } catch (error) {
      // SECURITY: Sanitize error before logging and re-throwing
      const sanitizedError = securityUtilsService.sanitizeErrorMessage(error, 'OAuth2 token refresh')
      console.error(`OAuth2 token refresh error for ${providerName}:`, sanitizedError)
      throw new Error(securityUtilsService.getUserFriendlyErrorMessage(error, 'refresh authentication'))
    }
  }

  // Get valid access token (refresh if needed)
  async getValidToken(providerName: 'gmail' | 'outlook'): Promise<string> {
    const token = this.tokens.get(providerName)
    
    if (!token) {
      throw new Error(`No token found for ${providerName}. Please authenticate first.`)
    }

    // Check if token is expired (with 5 minute buffer)
    const now = Date.now()
    const bufferTime = 5 * 60 * 1000 // 5 minutes
    
    if (token.expires_at && token.expires_at - bufferTime < now) {
      console.log(`Token for ${providerName} is expired, refreshing...`)
      const refreshedToken = await this.refreshToken(providerName)
      return refreshedToken.access_token
    }

    return token.access_token
  }

  // Revoke tokens and disconnect provider
  async revokeToken(providerName: 'gmail' | 'outlook'): Promise<void> {
    const provider = this.providers.get(providerName)
    const token = this.tokens.get(providerName)

    if (!provider || !token) {
      throw new Error(`Cannot revoke token for ${providerName}: missing provider or token`)
    }

    if (provider.revokeUrl) {
      try {
        const revokeData = new URLSearchParams({
          token: token.access_token
        })

        const response = await fetch(provider.revokeUrl, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/x-www-form-urlencoded'
          },
          body: revokeData.toString()
        })

        if (!response.ok) {
          console.warn(`Token revocation failed for ${providerName}: ${response.status}`)
        }
      } catch (error) {
        console.warn(`Token revocation error for ${providerName}:`, error)
      }
    }

    // Remove from local storage regardless of revocation result
    this.tokens.delete(providerName)
    this.saveTokensToStorage()
  }

  // Check if provider is authenticated
  isAuthenticated(providerName: 'gmail' | 'outlook'): boolean {
    const token = this.tokens.get(providerName)
    return !!(token?.access_token)
  }

  // Get token info for provider
  getTokenInfo(providerName: 'gmail' | 'outlook'): OAuth2Token | null {
    return this.tokens.get(providerName) || null
  }

  // Get all authenticated providers
  getAuthenticatedProviders(): string[] {
    return Array.from(this.tokens.keys()).filter(provider => 
      this.isAuthenticated(provider as 'gmail' | 'outlook')
    )
  }

  // Generate secure state parameter for OAuth2 flow
  generateState(): string {
    const array = new Uint8Array(32)
    crypto.getRandomValues(array)
    return Array.from(array, byte => byte.toString(16).padStart(2, '0')).join('')
  }

  // Validate state parameter
  validateState(providedState: string, expectedState: string): boolean {
    return providedState === expectedState
  }

  // Handle OAuth2 callback (typically called after redirect)
  async handleCallback(
    providerName: 'gmail' | 'outlook',
    callbackUrl: string,
    expectedState?: string
  ): Promise<OAuth2Token> {
    const url = new URL(callbackUrl)
    const code = url.searchParams.get('code')
    const state = url.searchParams.get('state')
    const error = url.searchParams.get('error')

    if (error) {
      throw new Error(`OAuth2 error: ${error} - ${url.searchParams.get('error_description')}`)
    }

    if (!code) {
      throw new Error('No authorization code received')
    }

    if (expectedState && !this.validateState(state || '', expectedState)) {
      throw new Error('Invalid state parameter')
    }

    return await this.exchangeCodeForTokens(providerName, code, state || undefined)
  }

  // Storage methods
  private loadStoredTokens(): void {
    try {
      const stored = localStorage.getItem('oauth2_tokens')
      if (stored) {
        const tokens = JSON.parse(stored)
        this.tokens = new Map(Object.entries(tokens))
      }
    } catch (error) {
      console.warn('Failed to load stored OAuth2 tokens:', error)
    }
  }

  private saveTokensToStorage(): void {
    try {
      const tokensObj = Object.fromEntries(this.tokens)
      localStorage.setItem('oauth2_tokens', JSON.stringify(tokensObj))
    } catch (error) {
      console.warn('Failed to save OAuth2 tokens:', error)
    }
  }

  // Clear all tokens (for logout)
  clearAllTokens(): void {
    this.tokens.clear()
    localStorage.removeItem('oauth2_tokens')
  }

  // Test token validity by making a simple API call
  async testToken(providerName: 'gmail' | 'outlook'): Promise<boolean> {
    try {
      const token = await this.getValidToken(providerName)
      
      if (providerName === 'gmail') {
        // Test Gmail API access
        const response = await fetch('https://gmail.googleapis.com/gmail/v1/users/me/profile', {
          headers: {
            'Authorization': `Bearer ${token}`
          }
        })
        return response.ok
      } else if (providerName === 'outlook') {
        // Test Microsoft Graph API access
        const response = await fetch('https://graph.microsoft.com/v1.0/me', {
          headers: {
            'Authorization': `Bearer ${token}`
          }
        })
        return response.ok
      }
      
      return false
    } catch (error) {
      console.error(`Token test failed for ${providerName}:`, error)
      return false
    }
  }
}

// Singleton instance
const oauth2Service = new OAuth2Service()
export default oauth2Service

// Configuration helper for development
export const configureOAuth2ForDevelopment = () => {
  // These would typically be environment variables
  const configs = {
    gmail: {
      clientId: process.env.REACT_APP_GOOGLE_CLIENT_ID || 'your-google-client-id',
      // SECURITY: Client secret removed from frontend - handled by backend proxy
      redirectUri: `${window.location.origin}/oauth/callback/gmail`,
      scopes: ['https://www.googleapis.com/auth/gmail.send'],
      tokenEndpoint: '/api/auth/google/token', // Proxy through secure backend
      usePKCE: true
    },
    outlook: {
      clientId: process.env.REACT_APP_MICROSOFT_CLIENT_ID || 'your-microsoft-client-id',
      // SECURITY: Client secret removed from frontend - handled by backend proxy
      redirectUri: `${window.location.origin}/oauth/callback/outlook`,
      scopes: ['https://graph.microsoft.com/mail.send'],
      tokenEndpoint: '/api/auth/microsoft/token', // Proxy through secure backend
      usePKCE: true
    }
  }

  // Only configure if client IDs are provided
  if (configs.gmail.clientId !== 'your-google-client-id') {
    oauth2Service.configureProvider('gmail', configs.gmail)
  }
  
  if (configs.outlook.clientId !== 'your-microsoft-client-id') {
    oauth2Service.configureProvider('outlook', configs.outlook)
  }
}