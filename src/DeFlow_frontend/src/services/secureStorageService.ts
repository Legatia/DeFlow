// SECURITY: Encrypted local storage service for sensitive data
import '../utils/bigint-polyfill'

export interface EncryptionKey {
  key: CryptoKey
  salt: Uint8Array
}

class SecureStorageService {
  private encryptionKey: CryptoKey | null = null
  private keyDerivationSalt: Uint8Array | null = null

  // SECURITY: Initialize encryption with user-derived key
  async initializeEncryption(userPassword?: string): Promise<void> {
    try {
      if (userPassword) {
        // Derive key from user password
        await this.deriveKeyFromPassword(userPassword)
      } else {
        // Generate random key for anonymous users
        await this.generateRandomKey()
      }
    } catch (error) {
      console.error('SECURITY: Failed to initialize encryption:', error)
      throw new Error('Failed to initialize secure storage')
    }
  }

  // SECURITY: Derive encryption key from user password using PBKDF2
  private async deriveKeyFromPassword(password: string): Promise<void> {
    const encoder = new TextEncoder()
    const passwordBuffer = encoder.encode(password)
    
    // Generate or retrieve salt
    this.keyDerivationSalt = this.getSavedSalt() || crypto.getRandomValues(new Uint8Array(16))
    
    // Import password as key material
    const keyMaterial = await crypto.subtle.importKey(
      'raw',
      passwordBuffer,
      'PBKDF2',
      false,
      ['deriveBits', 'deriveKey']
    )
    
    // Derive AES key
    this.encryptionKey = await crypto.subtle.deriveKey(
      {
        name: 'PBKDF2',
        salt: this.keyDerivationSalt,
        iterations: 100000, // High iteration count for security
        hash: 'SHA-256'
      },
      keyMaterial,
      { name: 'AES-GCM', length: 256 },
      false,
      ['encrypt', 'decrypt']
    )
    
    // Save salt (not the key!)
    this.saveSalt(this.keyDerivationSalt)
  }

  // SECURITY: Generate random key for anonymous users
  private async generateRandomKey(): Promise<void> {
    this.encryptionKey = await crypto.subtle.generateKey(
      { name: 'AES-GCM', length: 256 },
      true, // extractable for anonymous users
      ['encrypt', 'decrypt']
    )
    
    // Export and store key for anonymous users (base64 encoded)
    const exportedKey = await crypto.subtle.exportKey('raw', this.encryptionKey)
    const keyArray = new Uint8Array(exportedKey)
    const keyBase64 = btoa(String.fromCharCode(...keyArray))
    localStorage.setItem('deflow_secure_key', keyBase64)
  }

  // SECURITY: Encrypt data using AES-GCM
  private async encryptData(data: string): Promise<string> {
    if (!this.encryptionKey) {
      throw new Error('Encryption not initialized')
    }

    const encoder = new TextEncoder()
    const dataBuffer = encoder.encode(data)
    
    // Generate random IV for each encryption
    const iv = crypto.getRandomValues(new Uint8Array(12))
    
    // Encrypt the data
    const encryptedBuffer = await crypto.subtle.encrypt(
      { name: 'AES-GCM', iv },
      this.encryptionKey,
      dataBuffer
    )
    
    // Combine IV and encrypted data
    const combined = new Uint8Array(iv.length + encryptedBuffer.byteLength)
    combined.set(iv, 0)
    combined.set(new Uint8Array(encryptedBuffer), iv.length)
    
    // Return base64 encoded result
    return btoa(String.fromCharCode(...combined))
  }

  // SECURITY: Decrypt data using AES-GCM
  private async decryptData(encryptedData: string): Promise<string> {
    if (!this.encryptionKey) {
      throw new Error('Encryption not initialized')
    }

    try {
      // Decode base64
      const combined = new Uint8Array(atob(encryptedData).split('').map(c => c.charCodeAt(0)))
      
      // Extract IV and encrypted data
      const iv = combined.slice(0, 12)
      const encrypted = combined.slice(12)
      
      // Decrypt
      const decryptedBuffer = await crypto.subtle.decrypt(
        { name: 'AES-GCM', iv },
        this.encryptionKey,
        encrypted
      )
      
      // Convert to string
      const decoder = new TextDecoder()
      return decoder.decode(decryptedBuffer)
    } catch (error) {
      console.error('SECURITY: Decryption failed:', error)
      throw new Error('Failed to decrypt data')
    }
  }

  // SECURITY: Securely store encrypted data
  async setSecureItem(key: string, value: any): Promise<boolean> {
    try {
      const jsonString = JSON.stringify(value)
      const encryptedData = await this.encryptData(jsonString)
      localStorage.setItem(`secure_${key}`, encryptedData)
      return true
    } catch (error) {
      console.error('SECURITY: Failed to store encrypted data:', error)
      return false
    }
  }

  // SECURITY: Retrieve and decrypt data
  async getSecureItem<T>(key: string): Promise<T | null> {
    try {
      const encryptedData = localStorage.getItem(`secure_${key}`)
      if (!encryptedData) return null
      
      const decryptedString = await this.decryptData(encryptedData)
      return JSON.parse(decryptedString) as T
    } catch (error) {
      console.error('SECURITY: Failed to retrieve encrypted data:', error)
      return null
    }
  }

  // SECURITY: Remove encrypted data
  removeSecureItem(key: string): boolean {
    try {
      localStorage.removeItem(`secure_${key}`)
      return true
    } catch (error) {
      console.error('SECURITY: Failed to remove encrypted data:', error)
      return false
    }
  }

  // SECURITY: Clear all secure data
  clearAllSecureData(): boolean {
    try {
      const keys = Object.keys(localStorage).filter(key => key.startsWith('secure_'))
      keys.forEach(key => localStorage.removeItem(key))
      return true
    } catch (error) {
      console.error('SECURITY: Failed to clear secure data:', error)
      return false
    }
  }

  // SECURITY: Check if encryption is initialized
  isEncryptionReady(): boolean {
    return this.encryptionKey !== null
  }

  // Helper methods for salt management
  private getSavedSalt(): Uint8Array | null {
    try {
      const saltString = localStorage.getItem('deflow_encryption_salt')
      if (!saltString) return null
      
      const saltArray = atob(saltString).split('').map(c => c.charCodeAt(0))
      return new Uint8Array(saltArray)
    } catch (error) {
      return null
    }
  }

  private saveSalt(salt: Uint8Array): void {
    try {
      const saltString = btoa(String.fromCharCode(...salt))
      localStorage.setItem('deflow_encryption_salt', saltString)
    } catch (error) {
      console.error('SECURITY: Failed to save encryption salt:', error)
    }
  }

  // SECURITY: Load existing key for anonymous users
  async loadExistingKey(): Promise<boolean> {
    try {
      const keyBase64 = localStorage.getItem('deflow_secure_key')
      if (!keyBase64) return false
      
      const keyArray = new Uint8Array(atob(keyBase64).split('').map(c => c.charCodeAt(0)))
      
      this.encryptionKey = await crypto.subtle.importKey(
        'raw',
        keyArray,
        { name: 'AES-GCM' },
        false,
        ['encrypt', 'decrypt']
      )
      
      return true
    } catch (error) {
      console.error('SECURITY: Failed to load existing key:', error)
      return false
    }
  }

  // SECURITY: Validate browser crypto support
  static isCryptoSupported(): boolean {
    return !!(crypto && crypto.subtle && crypto.getRandomValues)
  }
}

// Export singleton instance
export const secureStorageService = new SecureStorageService()
export default secureStorageService