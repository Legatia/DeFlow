// SECURITY: Comprehensive input validation service for API configurations
import '../utils/bigint-polyfill'

export interface ValidationResult {
  isValid: boolean
  errors: Record<string, string>
  sanitizedData?: any
}

export interface APIValidationRules {
  required: boolean
  minLength?: number
  maxLength?: number
  pattern?: RegExp
  customValidator?: (value: string) => string | null
}

class InputValidationService {
  // Common validation patterns
  private readonly patterns = {
    // Twitter API patterns
    twitterApiKey: /^[A-Za-z0-9]{25}$/,
    twitterApiSecret: /^[A-Za-z0-9]{50}$/,
    twitterAccessToken: /^[0-9]+-[A-Za-z0-9]{40}$/,
    twitterAccessTokenSecret: /^[A-Za-z0-9]{45}$/,
    
    // Facebook API patterns
    facebookAccessToken: /^[A-Za-z0-9_-]{100,}$/,
    facebookPageId: /^[0-9]{10,20}$/,
    facebookAppId: /^[0-9]{15,20}$/,
    
    // LinkedIn API patterns
    linkedinAccessToken: /^[A-Za-z0-9]{60,100}$/,
    linkedinOrganizationId: /^[0-9]{8,15}$/,
    
    // Google/OAuth patterns
    googleClientId: /^[0-9]+-[a-z0-9]{32}\.apps\.googleusercontent\.com$/,
    oauthAccessToken: /^[A-Za-z0-9_-]{40,200}$/,
    
    // General patterns
    configName: /^[a-zA-Z0-9\s\-_]{1,50}$/,
    url: /^https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)$/
  }

  // SECURITY: Sanitize input to prevent XSS and injection
  private sanitizeInput(input: string): string {
    return input
      .trim()
      .replace(/[<>\"']/g, '') // Remove potentially dangerous characters
      .replace(/javascript:/gi, '') // Remove javascript: protocols
      .replace(/data:/gi, '') // Remove data: protocols
      .replace(/vbscript:/gi, '') // Remove vbscript: protocols
  }

  // SECURITY: Validate configuration name
  validateConfigName(name: string): ValidationResult {
    const sanitized = this.sanitizeInput(name)
    const errors: Record<string, string> = {}

    if (!sanitized) {
      errors.name = 'Configuration name is required'
    } else if (sanitized.length < 2) {
      errors.name = 'Configuration name must be at least 2 characters'
    } else if (sanitized.length > 50) {
      errors.name = 'Configuration name must not exceed 50 characters'
    } else if (!this.patterns.configName.test(sanitized)) {
      errors.name = 'Configuration name contains invalid characters. Use only letters, numbers, spaces, hyphens, and underscores'
    }

    return {
      isValid: Object.keys(errors).length === 0,
      errors,
      sanitizedData: sanitized
    }
  }

  // SECURITY: Validate Twitter API credentials
  validateTwitterCredentials(credentials: {
    name: string
    api_key: string
    api_secret: string
    access_token: string
    access_token_secret: string
  }): ValidationResult {
    const errors: Record<string, string> = {}
    const sanitizedData: any = {}

    // Validate name
    const nameValidation = this.validateConfigName(credentials.name)
    if (!nameValidation.isValid) {
      Object.assign(errors, nameValidation.errors)
    } else {
      sanitizedData.name = nameValidation.sanitizedData
    }

    // Validate API Key
    const apiKey = credentials.api_key.trim()
    if (!apiKey) {
      errors.api_key = 'API Key is required'
    } else if (!this.patterns.twitterApiKey.test(apiKey)) {
      errors.api_key = 'Invalid API Key format. Expected 25 alphanumeric characters'
    } else {
      sanitizedData.api_key = apiKey
    }

    // Validate API Secret
    const apiSecret = credentials.api_secret.trim()
    if (!apiSecret) {
      errors.api_secret = 'API Secret is required'
    } else if (!this.patterns.twitterApiSecret.test(apiSecret)) {
      errors.api_secret = 'Invalid API Secret format. Expected 50 alphanumeric characters'
    } else {
      sanitizedData.api_secret = apiSecret
    }

    // Validate Access Token
    const accessToken = credentials.access_token.trim()
    if (!accessToken) {
      errors.access_token = 'Access Token is required'
    } else if (!this.patterns.twitterAccessToken.test(accessToken)) {
      errors.access_token = 'Invalid Access Token format. Expected format: numbers-40_alphanumeric_chars'
    } else {
      sanitizedData.access_token = accessToken
    }

    // Validate Access Token Secret
    const accessTokenSecret = credentials.access_token_secret.trim()
    if (!accessTokenSecret) {
      errors.access_token_secret = 'Access Token Secret is required'
    } else if (!this.patterns.twitterAccessTokenSecret.test(accessTokenSecret)) {
      errors.access_token_secret = 'Invalid Access Token Secret format. Expected 45 alphanumeric characters'
    } else {
      sanitizedData.access_token_secret = accessTokenSecret
    }

    return {
      isValid: Object.keys(errors).length === 0,
      errors,
      sanitizedData: Object.keys(errors).length === 0 ? sanitizedData : undefined
    }
  }

  // SECURITY: Validate Facebook API credentials
  validateFacebookCredentials(credentials: {
    name: string
    access_token: string
    page_id: string
    post_type: string
  }): ValidationResult {
    const errors: Record<string, string> = {}
    const sanitizedData: any = {}

    // Validate name
    const nameValidation = this.validateConfigName(credentials.name)
    if (!nameValidation.isValid) {
      Object.assign(errors, nameValidation.errors)
    } else {
      sanitizedData.name = nameValidation.sanitizedData
    }

    // Validate Access Token
    const accessToken = credentials.access_token.trim()
    if (!accessToken) {
      errors.access_token = 'Access Token is required'
    } else if (accessToken.length < 50) {
      errors.access_token = 'Access Token appears too short. Facebook tokens are typically 100+ characters'
    } else if (!this.patterns.facebookAccessToken.test(accessToken)) {
      errors.access_token = 'Invalid Access Token format. Use only alphanumeric characters, hyphens, and underscores'
    } else {
      sanitizedData.access_token = accessToken
    }

    // Validate Page ID
    const pageId = credentials.page_id.trim()
    if (!pageId) {
      errors.page_id = 'Page ID is required'
    } else if (!this.patterns.facebookPageId.test(pageId)) {
      errors.page_id = 'Invalid Page ID format. Expected 10-20 digit number'
    } else {
      sanitizedData.page_id = pageId
    }

    // Validate Post Type
    const validPostTypes = ['page', 'group', 'event']
    if (!validPostTypes.includes(credentials.post_type)) {
      errors.post_type = 'Invalid post type. Must be: page, group, or event'
    } else {
      sanitizedData.post_type = credentials.post_type
    }

    return {
      isValid: Object.keys(errors).length === 0,
      errors,
      sanitizedData: Object.keys(errors).length === 0 ? sanitizedData : undefined
    }
  }

  // SECURITY: Validate LinkedIn API credentials
  validateLinkedInCredentials(credentials: {
    name: string
    access_token: string
    post_type: string
    organization_id?: string
  }): ValidationResult {
    const errors: Record<string, string> = {}
    const sanitizedData: any = {}

    // Validate name
    const nameValidation = this.validateConfigName(credentials.name)
    if (!nameValidation.isValid) {
      Object.assign(errors, nameValidation.errors)
    } else {
      sanitizedData.name = nameValidation.sanitizedData
    }

    // Validate Access Token
    const accessToken = credentials.access_token.trim()
    if (!accessToken) {
      errors.access_token = 'Access Token is required'
    } else if (accessToken.length < 50) {
      errors.access_token = 'Access Token appears too short. LinkedIn tokens are typically 60+ characters'
    } else if (!this.patterns.linkedinAccessToken.test(accessToken)) {
      errors.access_token = 'Invalid Access Token format. Use only alphanumeric characters'
    } else {
      sanitizedData.access_token = accessToken
    }

    // Validate Post Type
    const validPostTypes = ['person', 'organization']
    if (!validPostTypes.includes(credentials.post_type)) {
      errors.post_type = 'Invalid post type. Must be: person or organization'
    } else {
      sanitizedData.post_type = credentials.post_type
    }

    // Validate Organization ID (if required)
    if (credentials.post_type === 'organization') {
      const orgId = credentials.organization_id?.trim() || ''
      if (!orgId) {
        errors.organization_id = 'Organization ID is required for organization posts'
      } else if (!this.patterns.linkedinOrganizationId.test(orgId)) {
        errors.organization_id = 'Invalid Organization ID format. Expected 8-15 digit number'
      } else {
        sanitizedData.organization_id = orgId
      }
    }

    return {
      isValid: Object.keys(errors).length === 0,
      errors,
      sanitizedData: Object.keys(errors).length === 0 ? sanitizedData : undefined
    }
  }

  // SECURITY: Validate OAuth configuration
  validateOAuthConfig(config: {
    clientId: string
    redirectUri: string
    scopes: string[]
  }): ValidationResult {
    const errors: Record<string, string> = {}
    const sanitizedData: any = {}

    // Validate Client ID
    const clientId = config.clientId.trim()
    if (!clientId) {
      errors.clientId = 'Client ID is required'
    } else if (clientId.includes('your-') || clientId.includes('example')) {
      errors.clientId = 'Please replace placeholder Client ID with actual value'
    } else {
      sanitizedData.clientId = clientId
    }

    // Validate Redirect URI
    const redirectUri = config.redirectUri.trim()
    if (!redirectUri) {
      errors.redirectUri = 'Redirect URI is required'
    } else if (!this.patterns.url.test(redirectUri)) {
      errors.redirectUri = 'Invalid Redirect URI format. Must be a valid HTTPS URL'
    } else if (!redirectUri.startsWith('https://') && !redirectUri.includes('localhost')) {
      errors.redirectUri = 'Redirect URI must use HTTPS in production'
    } else {
      sanitizedData.redirectUri = redirectUri
    }

    // Validate Scopes
    if (!config.scopes || config.scopes.length === 0) {
      errors.scopes = 'At least one scope is required'
    } else {
      // Sanitize scopes
      const sanitizedScopes = config.scopes
        .map(scope => this.sanitizeInput(scope))
        .filter(scope => scope.length > 0)
      
      if (sanitizedScopes.length === 0) {
        errors.scopes = 'Valid scopes are required'
      } else {
        sanitizedData.scopes = sanitizedScopes
      }
    }

    return {
      isValid: Object.keys(errors).length === 0,
      errors,
      sanitizedData: Object.keys(errors).length === 0 ? sanitizedData : undefined
    }
  }

  // SECURITY: Check for common security issues in API keys
  validateAPIKeySecurity(key: string, keyType: string): string[] {
    const warnings: string[] = []

    // Check for common insecure patterns
    if (key.includes('test') || key.includes('demo') || key.includes('example')) {
      warnings.push(`${keyType} appears to be a test/demo key`)
    }

    if (key.includes('your-') || key.includes('replace-')) {
      warnings.push(`${keyType} appears to be a placeholder`)
    }

    if (key.length < 20) {
      warnings.push(`${keyType} is unusually short for production use`)
    }

    // Check for repeated patterns
    const hasRepeatedChars = /(.)\1{5,}/.test(key)
    if (hasRepeatedChars) {
      warnings.push(`${keyType} contains suspicious repeated characters`)
    }

    return warnings
  }

  // SECURITY: Rate limiting validation
  validateRateLimiting(lastCallTime: number, minInterval: number = 1000): boolean {
    const now = Date.now()
    return (now - lastCallTime) >= minInterval
  }

  // SECURITY: Comprehensive validation for any API configuration
  validateGenericAPIConfig(config: Record<string, any>, rules: Record<string, APIValidationRules>): ValidationResult {
    const errors: Record<string, string> = {}
    const sanitizedData: Record<string, any> = {}

    for (const [field, value] of Object.entries(config)) {
      const rule = rules[field]
      if (!rule) continue

      const stringValue = String(value || '').trim()
      const sanitized = this.sanitizeInput(stringValue)

      // Required field check
      if (rule.required && !sanitized) {
        errors[field] = `${field} is required`
        continue
      }

      // Skip other validations if field is not required and empty
      if (!rule.required && !sanitized) {
        continue
      }

      // Length validation
      if (rule.minLength && sanitized.length < rule.minLength) {
        errors[field] = `${field} must be at least ${rule.minLength} characters`
        continue
      }

      if (rule.maxLength && sanitized.length > rule.maxLength) {
        errors[field] = `${field} must not exceed ${rule.maxLength} characters`
        continue
      }

      // Pattern validation
      if (rule.pattern && !rule.pattern.test(sanitized)) {
        errors[field] = `${field} format is invalid`
        continue
      }

      // Custom validation
      if (rule.customValidator) {
        const customError = rule.customValidator(sanitized)
        if (customError) {
          errors[field] = customError
          continue
        }
      }

      // If all validations pass, add to sanitized data
      sanitizedData[field] = sanitized
    }

    return {
      isValid: Object.keys(errors).length === 0,
      errors,
      sanitizedData: Object.keys(errors).length === 0 ? sanitizedData : undefined
    }
  }
}

// Export singleton instance
export const inputValidationService = new InputValidationService()
export default inputValidationService