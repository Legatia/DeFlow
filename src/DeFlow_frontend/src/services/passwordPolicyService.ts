// SECURITY: Password policy and validation service
import '../utils/bigint-polyfill'

export interface PasswordPolicy {
  minLength: number
  maxLength: number
  requireUppercase: boolean
  requireLowercase: boolean
  requireNumbers: boolean
  requireSpecialChars: boolean
  minSpecialChars: number
  prohibitCommonPasswords: boolean
  prohibitPersonalInfo: boolean
  prohibitRepeatingChars: boolean
  maxRepeatingChars: number
  requireDifferentFromLast: number // Number of previous passwords to check
  expiryDays?: number // Optional password expiry
}

export interface PasswordValidationResult {
  isValid: boolean
  score: number // 0-100
  errors: string[]
  warnings: string[]
  suggestions: string[]
  strength: 'very-weak' | 'weak' | 'fair' | 'good' | 'strong' | 'very-strong'
}

export interface UserInfo {
  email?: string
  username?: string
  displayName?: string
  firstName?: string
  lastName?: string
}

class PasswordPolicyService {
  // SECURITY: Default strong password policy
  private readonly defaultPolicy: PasswordPolicy = {
    minLength: 12,
    maxLength: 128,
    requireUppercase: true,
    requireLowercase: true,
    requireNumbers: true,
    requireSpecialChars: true,
    minSpecialChars: 1,
    prohibitCommonPasswords: true,
    prohibitPersonalInfo: true,
    prohibitRepeatingChars: true,
    maxRepeatingChars: 2,
    requireDifferentFromLast: 3,
    expiryDays: 90
  }

  // SECURITY: Common passwords list (subset for demo)
  private readonly commonPasswords = new Set([
    'password', 'password123', '123456', '123456789', 'qwerty', 'abc123',
    'password1', 'admin', 'letmein', 'welcome', 'monkey', 'dragon',
    'master', 'shadow', 'password12', 'password1234', 'pass123',
    'welcome123', 'admin123', 'root', 'toor', 'guest', 'test',
    'deflow', 'deflow123', 'crypto', 'bitcoin', 'ethereum', 'blockchain'
  ])

  // SECURITY: Special characters allowed
  private readonly specialChars = '!@#$%^&*()_+-=[]{}|;:,.<>?'

  // SECURITY: Get current password policy
  getPasswordPolicy(): PasswordPolicy {
    // In production, this could be loaded from configuration
    return { ...this.defaultPolicy }
  }

  // SECURITY: Comprehensive password validation
  validatePassword(password: string, userInfo?: UserInfo, previousPasswords?: string[]): PasswordValidationResult {
    const policy = this.getPasswordPolicy()
    const errors: string[] = []
    const warnings: string[] = []
    const suggestions: string[] = []
    let score = 0

    // Length validation
    if (password.length < policy.minLength) {
      errors.push(`Password must be at least ${policy.minLength} characters long`)
    } else if (password.length >= policy.minLength) {
      score += 15
      if (password.length >= 16) score += 10 // Bonus for longer passwords
    }

    if (password.length > policy.maxLength) {
      errors.push(`Password must not exceed ${policy.maxLength} characters`)
    }

    // Character requirements
    const hasUppercase = /[A-Z]/.test(password)
    const hasLowercase = /[a-z]/.test(password)
    const hasNumbers = /[0-9]/.test(password)
    const hasSpecialChars = new RegExp(`[${this.escapeRegExp(this.specialChars)}]`).test(password)
    const specialCharCount = (password.match(new RegExp(`[${this.escapeRegExp(this.specialChars)}]`, 'g')) || []).length

    if (policy.requireUppercase && !hasUppercase) {
      errors.push('Password must contain at least one uppercase letter')
    } else if (hasUppercase) {
      score += 15
    }

    if (policy.requireLowercase && !hasLowercase) {
      errors.push('Password must contain at least one lowercase letter')
    } else if (hasLowercase) {
      score += 15
    }

    if (policy.requireNumbers && !hasNumbers) {
      errors.push('Password must contain at least one number')
    } else if (hasNumbers) {
      score += 15
    }

    if (policy.requireSpecialChars && !hasSpecialChars) {
      errors.push(`Password must contain at least one special character (${this.specialChars})`)
    } else if (hasSpecialChars) {
      score += 15
    }

    if (policy.requireSpecialChars && specialCharCount < policy.minSpecialChars) {
      errors.push(`Password must contain at least ${policy.minSpecialChars} special characters`)
    } else if (specialCharCount >= policy.minSpecialChars) {
      score += 10
    }

    // Common password check
    if (policy.prohibitCommonPasswords && this.isCommonPassword(password)) {
      errors.push('Password is too common. Please choose a more unique password')
    } else if (policy.prohibitCommonPasswords) {
      score += 10
    }

    // Personal information check
    if (policy.prohibitPersonalInfo && userInfo && this.containsPersonalInfo(password, userInfo)) {
      errors.push('Password should not contain personal information (email, username, name)')
    } else if (policy.prohibitPersonalInfo && userInfo) {
      score += 10
    }

    // Repeating characters check
    if (policy.prohibitRepeatingChars && this.hasExcessiveRepeatingChars(password, policy.maxRepeatingChars)) {
      errors.push(`Password should not contain more than ${policy.maxRepeatingChars} consecutive identical characters`)
    } else if (policy.prohibitRepeatingChars) {
      score += 5
    }

    // Previous passwords check
    if (previousPasswords && this.isInPreviousPasswords(password, previousPasswords, policy.requireDifferentFromLast)) {
      errors.push(`Password must be different from your last ${policy.requireDifferentFromLast} passwords`)
    }

    // Pattern detection and warnings
    this.detectPatterns(password, warnings, suggestions)

    // Character diversity bonus
    const charTypes = [hasUppercase, hasLowercase, hasNumbers, hasSpecialChars].filter(Boolean).length
    if (charTypes >= 4) score += 15
    else if (charTypes >= 3) score += 10

    // Entropy estimation
    const entropy = this.estimateEntropy(password)
    if (entropy >= 60) score += 10
    else if (entropy >= 40) score += 5

    // Ensure score is within bounds
    score = Math.min(100, Math.max(0, score))

    // Determine strength
    const strength = this.determineStrength(score, errors.length)

    // Generate suggestions if password is weak
    if (score < 60 || errors.length > 0) {
      this.generateSuggestions(password, policy, suggestions)
    }

    return {
      isValid: errors.length === 0,
      score,
      errors,
      warnings,
      suggestions,
      strength
    }
  }

  // SECURITY: Hash password for storage (in production, use bcrypt or similar)
  async hashPassword(password: string): Promise<string> {
    // DEMO: In production, use proper password hashing like bcrypt
    // This is just for demonstration - NEVER use this in production
    const encoder = new TextEncoder()
    const data = encoder.encode(password + 'deflow_salt_2024')
    const hashBuffer = await crypto.subtle.digest('SHA-256', data)
    const hashArray = Array.from(new Uint8Array(hashBuffer))
    return hashArray.map(b => b.toString(16).padStart(2, '0')).join('')
  }

  // SECURITY: Verify password against hash
  async verifyPassword(password: string, hash: string): Promise<boolean> {
    const computedHash = await this.hashPassword(password)
    return computedHash === hash
  }

  // SECURITY: Generate secure random password
  generateSecurePassword(length: number = 16): string {
    const policy = this.getPasswordPolicy()
    const uppercase = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'
    const lowercase = 'abcdefghijklmnopqrstuvwxyz'
    const numbers = '0123456789'
    const special = this.specialChars

    let allChars = ''
    let password = ''

    // Ensure at least one character from each required type
    if (policy.requireUppercase) {
      password += this.getRandomChar(uppercase)
      allChars += uppercase
    }
    if (policy.requireLowercase) {
      password += this.getRandomChar(lowercase)
      allChars += lowercase
    }
    if (policy.requireNumbers) {
      password += this.getRandomChar(numbers)
      allChars += numbers
    }
    if (policy.requireSpecialChars) {
      password += this.getRandomChar(special)
      allChars += special
    }

    // Fill remaining length with random characters
    for (let i = password.length; i < length; i++) {
      password += this.getRandomChar(allChars)
    }

    // Shuffle the password to avoid predictable patterns
    return this.shuffleString(password)
  }

  // SECURITY: Check password expiry
  isPasswordExpired(lastChangedDate: Date): boolean {
    const policy = this.getPasswordPolicy()
    if (!policy.expiryDays) return false

    const expiryDate = new Date(lastChangedDate)
    expiryDate.setDate(expiryDate.getDate() + policy.expiryDays)

    return new Date() > expiryDate
  }

  // SECURITY: Get days until password expires
  getDaysUntilExpiry(lastChangedDate: Date): number {
    const policy = this.getPasswordPolicy()
    if (!policy.expiryDays) return Infinity

    const expiryDate = new Date(lastChangedDate)
    expiryDate.setDate(expiryDate.getDate() + policy.expiryDays)

    const msUntilExpiry = expiryDate.getTime() - new Date().getTime()
    return Math.ceil(msUntilExpiry / (1000 * 60 * 60 * 24))
  }

  // Private helper methods
  private isCommonPassword(password: string): boolean {
    const lower = password.toLowerCase()
    
    // Check exact matches
    if (this.commonPasswords.has(lower)) return true
    
    // Check common patterns
    if (/^password\d*$/i.test(password)) return true
    if (/^\d{4,}$/.test(password)) return true // All numbers
    if (/^[a-z]+\d*$/i.test(password) && password.length < 8) return true // Simple words
    
    return false
  }

  private containsPersonalInfo(password: string, userInfo: UserInfo): boolean {
    const lower = password.toLowerCase()
    const { email, username, displayName, firstName, lastName } = userInfo

    const infoToCheck = [
      email?.split('@')[0], // Username part of email
      username,
      firstName,
      lastName,
      displayName
    ].filter(Boolean).map(info => info!.toLowerCase())

    return infoToCheck.some(info => {
      if (info.length < 3) return false // Skip very short info
      return lower.includes(info) || info.includes(lower)
    })
  }

  private hasExcessiveRepeatingChars(password: string, maxRepeating: number): boolean {
    for (let i = 0; i < password.length - maxRepeating; i++) {
      const char = password[i]
      let count = 1
      
      for (let j = i + 1; j < password.length && password[j] === char; j++) {
        count++
      }
      
      if (count > maxRepeating) return true
    }
    
    return false
  }

  private isInPreviousPasswords(password: string, previousPasswords: string[], count: number): boolean {
    return previousPasswords.slice(0, count).includes(password)
  }

  private detectPatterns(password: string, warnings: string[], suggestions: string[]): void {
    // Sequential characters
    if (/123|abc|qwe|789/i.test(password)) {
      warnings.push('Password contains sequential characters')
      suggestions.push('Avoid sequential characters like "123" or "abc"')
    }

    // Keyboard patterns
    if (/qwerty|asdf|zxcv/i.test(password)) {
      warnings.push('Password contains keyboard patterns')
      suggestions.push('Avoid keyboard patterns like "qwerty" or "asdf"')
    }

    // Years
    if (/19\d{2}|20\d{2}/.test(password)) {
      warnings.push('Password contains year patterns')
      suggestions.push('Avoid including years in your password')
    }

    // Simple substitutions
    if (/[@a4@]|[3e3]|[1i!]|[0o0]|[5s$]/i.test(password)) {
      warnings.push('Password uses simple character substitutions')
      suggestions.push('Use more complex character combinations instead of simple substitutions')
    }
  }

  private generateSuggestions(password: string, policy: PasswordPolicy, suggestions: string[]): void {
    if (password.length < policy.minLength) {
      suggestions.push(`Make your password at least ${policy.minLength} characters long`)
    }

    if (policy.requireUppercase && !/[A-Z]/.test(password)) {
      suggestions.push('Add uppercase letters (A-Z)')
    }

    if (policy.requireLowercase && !/[a-z]/.test(password)) {
      suggestions.push('Add lowercase letters (a-z)')
    }

    if (policy.requireNumbers && !/[0-9]/.test(password)) {
      suggestions.push('Add numbers (0-9)')
    }

    if (policy.requireSpecialChars && !new RegExp(`[${this.escapeRegExp(this.specialChars)}]`).test(password)) {
      suggestions.push(`Add special characters (${this.specialChars})`)
    }

    suggestions.push('Consider using a passphrase with multiple words')
    suggestions.push('Use a password manager to generate and store complex passwords')
  }

  private estimateEntropy(password: string): number {
    const charset = this.getCharsetSize(password)
    return Math.log2(Math.pow(charset, password.length))
  }

  private getCharsetSize(password: string): number {
    let size = 0
    if (/[a-z]/.test(password)) size += 26
    if (/[A-Z]/.test(password)) size += 26
    if (/[0-9]/.test(password)) size += 10
    if (new RegExp(`[${this.escapeRegExp(this.specialChars)}]`).test(password)) size += this.specialChars.length
    return size
  }

  private determineStrength(score: number, errorCount: number): PasswordValidationResult['strength'] {
    if (errorCount > 0) return 'very-weak'
    if (score >= 90) return 'very-strong'
    if (score >= 75) return 'strong'
    if (score >= 60) return 'good'
    if (score >= 40) return 'fair'
    if (score >= 20) return 'weak'
    return 'very-weak'
  }

  private getRandomChar(charset: string): string {
    const randomIndex = crypto.getRandomValues(new Uint32Array(1))[0] % charset.length
    return charset[randomIndex]
  }

  private shuffleString(str: string): string {
    const array = str.split('')
    for (let i = array.length - 1; i > 0; i--) {
      const j = crypto.getRandomValues(new Uint32Array(1))[0] % (i + 1)
      ;[array[i], array[j]] = [array[j], array[i]]
    }
    return array.join('')
  }

  private escapeRegExp(string: string): string {
    return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  }
}

// Export singleton instance
export const passwordPolicyService = new PasswordPolicyService()
export default passwordPolicyService