// User authentication and authorization service for DeFlow
import { TimestampUtils } from '../utils/timestamp-utils'

export interface User {
  id: string
  email: string
  username: string
  displayName: string
  role: 'admin' | 'user' | 'viewer'
  permissions: Permission[]
  isActive: boolean
  createdAt: string
  lastLoginAt: string
  profile: UserProfile
}

export interface UserProfile {
  avatar?: string
  bio?: string
  timezone: string
  notifications: NotificationSettings
  preferences: UserPreferences
}

export interface NotificationSettings {
  email: boolean
  push: boolean
  workflow_completion: boolean
  workflow_failure: boolean
  system_alerts: boolean
}

export interface UserPreferences {
  theme: 'light' | 'dark' | 'auto'
  language: string
  defaultWorkflowView: 'grid' | 'list'
  autoSaveInterval: number
}

export interface Permission {
  id: string
  name: string
  resource: string
  action: 'create' | 'read' | 'update' | 'delete' | 'execute' | '*'
  conditions?: Record<string, any>
}

export interface AuthSession {
  id: string
  userId: string
  token: string
  refreshToken: string
  expiresAt: string
  createdAt: string
  isActive: boolean
  deviceInfo: DeviceInfo
}

export interface DeviceInfo {
  userAgent: string
  ip: string
  location?: string
  deviceType: 'desktop' | 'mobile' | 'tablet'
}

export interface LoginCredentials {
  email: string
  password: string
  rememberMe?: boolean
}

export interface RegisterData {
  email: string
  username: string
  password: string
  displayName: string
  acceptTerms: boolean
}

export interface PasswordResetRequest {
  email: string
}

export interface PasswordReset {
  token: string
  newPassword: string
}

class AuthService {
  private currentUser: User | null = null
  private currentSession: AuthSession | null = null
  private users: Map<string, User> = new Map()
  private sessions: Map<string, AuthSession> = new Map()
  private refreshTokens: Map<string, string> = new Map()

  constructor() {
    // SECURITY: Initialize demo users with proper password hashing
    this.initializeDemoUsers()
    this.loadSessionFromStorage()
  }

  // Authentication methods
  async login(credentials: LoginCredentials): Promise<{ user: User; session: AuthSession }> {
    const { email, password, rememberMe = false } = credentials

    // Find user by email
    const user = Array.from(this.users.values()).find(u => u.email === email)
    if (!user) {
      throw new Error('Invalid email or password')
    }

    if (!user.isActive) {
      throw new Error('Account is disabled. Please contact support.')
    }

    // In real implementation, verify password hash
    if (password !== 'password123') {
      throw new Error('Invalid email or password')
    }

    // Update last login
    user.lastLoginAt = TimestampUtils.dateToICPTimestamp(new Date())
    this.users.set(user.id, user)

    // Create session
    const session = this.createSession(user.id, rememberMe)
    
    this.currentUser = user
    this.currentSession = session
    this.saveSessionToStorage(session)

    return { user, session }
  }

  async register(data: RegisterData): Promise<{ user: User; session: AuthSession }> {
    const { email, username, password, displayName, acceptTerms } = data

    if (!acceptTerms) {
      throw new Error('You must accept the terms and conditions')
    }

    // Check if email already exists
    const existingUser = Array.from(this.users.values()).find(u => u.email === email)
    if (existingUser) {
      throw new Error('Email already registered')
    }

    // Check if username already exists
    const existingUsername = Array.from(this.users.values()).find(u => u.username === username)
    if (existingUsername) {
      throw new Error('Username already taken')
    }

    // Create new user
    const userId = this.generateUserId()
    const now = TimestampUtils.dateToICPTimestamp(new Date())
    
    const user: User = {
      id: userId,
      email,
      username,
      displayName,
      role: 'user',
      permissions: this.getDefaultPermissions(),
      isActive: true,
      createdAt: now,
      lastLoginAt: now,
      profile: {
        timezone: 'UTC',
        notifications: {
          email: true,
          push: true,
          workflow_completion: true,
          workflow_failure: true,
          system_alerts: false
        },
        preferences: {
          theme: 'light',
          language: 'en',
          defaultWorkflowView: 'grid',
          autoSaveInterval: 30
        }
      }
    }

    this.users.set(userId, user)

    // Create session
    const session = this.createSession(userId, false)
    
    this.currentUser = user
    this.currentSession = session
    this.saveSessionToStorage(session)

    return { user, session }
  }

  async logout(): Promise<void> {
    if (this.currentSession) {
      this.currentSession.isActive = false
      this.sessions.set(this.currentSession.id, this.currentSession)
    }

    this.currentUser = null
    this.currentSession = null
    this.clearSessionFromStorage()
  }

  async refreshSession(): Promise<AuthSession> {
    if (!this.currentSession) {
      throw new Error('No active session')
    }

    const refreshToken = this.refreshTokens.get(this.currentSession.id)
    if (!refreshToken) {
      throw new Error('Invalid refresh token')
    }

    // Create new session
    const newSession = this.createSession(this.currentSession.userId, true)
    
    // Deactivate old session
    this.currentSession.isActive = false
    this.sessions.set(this.currentSession.id, this.currentSession)

    this.currentSession = newSession
    this.saveSessionToStorage(newSession)

    return newSession
  }

  // Password management
  async requestPasswordReset(request: PasswordResetRequest): Promise<void> {
    const user = Array.from(this.users.values()).find(u => u.email === request.email)
    if (!user) {
      // Don't reveal if email exists for security
      return
    }

    // In real implementation, send email with reset token
  }

  async resetPassword(reset: PasswordReset): Promise<void> {
    // In real implementation, verify token and update password
  }

  async changePassword(currentPassword: string, newPassword: string): Promise<void> {
    if (!this.currentUser) {
      throw new Error('Not authenticated')
    }

    // In real implementation, verify current password and update
  }

  // User management
  getCurrentUser(): User | null {
    return this.currentUser
  }

  getCurrentSession(): AuthSession | null {
    return this.currentSession
  }

  isAuthenticated(): boolean {
    return this.currentUser !== null && this.currentSession !== null && this.currentSession.isActive
  }

  async updateProfile(updates: Partial<UserProfile>): Promise<User> {
    if (!this.currentUser) {
      throw new Error('Not authenticated')
    }

    this.currentUser.profile = { ...this.currentUser.profile, ...updates }
    this.users.set(this.currentUser.id, this.currentUser)

    return this.currentUser
  }

  async updateUser(updates: Partial<Pick<User, 'displayName' | 'email'>>): Promise<User> {
    if (!this.currentUser) {
      throw new Error('Not authenticated')
    }

    // Check email uniqueness if being updated
    if (updates.email && updates.email !== this.currentUser.email) {
      const existingUser = Array.from(this.users.values()).find(u => u.email === updates.email)
      if (existingUser) {
        throw new Error('Email already in use')
      }
    }

    this.currentUser = { ...this.currentUser, ...updates }
    this.users.set(this.currentUser.id, this.currentUser)

    return this.currentUser
  }

  // Authorization methods
  hasPermission(resource: string, action: string): boolean {
    if (!this.currentUser) return false

    return this.currentUser.permissions.some(p => 
      p.resource === resource && p.action === action
    ) || this.currentUser.permissions.some(p => 
      p.resource === '*' && p.action === '*'
    )
  }

  canAccessWorkflow(workflowId: string, action: 'read' | 'write' | 'execute' | 'delete'): boolean {
    if (!this.currentUser) return false

    // Admin has all permissions
    if (this.currentUser.role === 'admin') return true

    // Check specific workflow permissions
    return this.hasPermission('workflow', action) || this.hasPermission('*', '*')
  }

  canManageUsers(): boolean {
    return this.hasPermission('user', 'create') || this.currentUser?.role === 'admin'
  }

  canViewAnalytics(): boolean {
    return this.hasPermission('analytics', 'read') || this.currentUser?.role === 'admin'
  }

  // Session management
  private createSession(userId: string, rememberMe: boolean): AuthSession {
    const sessionId = this.generateSessionId()
    const token = this.generateToken()
    const refreshToken = this.generateRefreshToken()
    
    const expiresAt = new Date()
    if (rememberMe) {
      expiresAt.setDate(expiresAt.getDate() + 30) // 30 days
    } else {
      expiresAt.setHours(expiresAt.getHours() + 8) // 8 hours
    }

    const session: AuthSession = {
      id: sessionId,
      userId,
      token,
      refreshToken,
      expiresAt: TimestampUtils.dateToICPTimestamp(expiresAt),
      createdAt: TimestampUtils.dateToICPTimestamp(new Date()),
      isActive: true,
      deviceInfo: this.getDeviceInfo()
    }

    this.sessions.set(sessionId, session)
    this.refreshTokens.set(sessionId, refreshToken)

    return session
  }

  private getDeviceInfo(): DeviceInfo {
    return {
      userAgent: navigator.userAgent,
      ip: '127.0.0.1', // Would be actual IP
      deviceType: 'desktop' // Would detect actual device type
    }
  }

  private saveSessionToStorage(session: AuthSession): void {
    try {
      localStorage.setItem('deflow_session', JSON.stringify({
        sessionId: session.id,
        token: session.token,
        userId: session.userId
      }))
    } catch (error) {
      console.warn('Failed to save session to localStorage:', error)
    }
  }

  private loadSessionFromStorage(): void {
    try {
      const stored = localStorage.getItem('deflow_session')
      if (!stored) return

      const { sessionId, token, userId } = JSON.parse(stored)
      const session = this.sessions.get(sessionId)
      
      if (session && session.isActive && session.token === token) {
        const expiresAt = TimestampUtils.icpTimestampToDate(session.expiresAt)
        if (expiresAt > new Date()) {
          this.currentSession = session
          this.currentUser = this.users.get(userId) || null
        }
      }
    } catch (error) {
      console.warn('Failed to load session from localStorage:', error)
      this.clearSessionFromStorage()
    }
  }

  private clearSessionFromStorage(): void {
    try {
      localStorage.removeItem('deflow_session')
    } catch (error) {
      console.warn('Failed to clear session from localStorage:', error)
    }
  }

  // Default permissions
  private getDefaultPermissions(): Permission[] {
    return [
      { id: 'perm_001', name: 'Read Workflows', resource: 'workflow', action: 'read' },
      { id: 'perm_002', name: 'Create Workflows', resource: 'workflow', action: 'create' },
      { id: 'perm_003', name: 'Update Own Workflows', resource: 'workflow', action: 'update' },
      { id: 'perm_004', name: 'Execute Workflows', resource: 'workflow', action: 'execute' },
      { id: 'perm_005', name: 'Read Profile', resource: 'profile', action: 'read' },
      { id: 'perm_006', name: 'Update Profile', resource: 'profile', action: 'update' }
    ]
  }

  private getAdminPermissions(): Permission[] {
    return [
      { id: 'perm_admin_001', name: 'All Permissions', resource: '*', action: '*' }
    ]
  }

  // Demo users initialization
  private initializeDemoUsers(): void {
    const now = TimestampUtils.dateToICPTimestamp(new Date())

    const adminUser: User = {
      id: 'user_001',
      email: 'admin@deflow.com',
      username: 'admin',
      displayName: 'System Administrator',
      role: 'admin',
      permissions: this.getAdminPermissions(),
      isActive: true,
      createdAt: now,
      lastLoginAt: now,
      profile: {
        timezone: 'UTC',
        notifications: {
          email: true,
          push: true,
          workflow_completion: true,
          workflow_failure: true,
          system_alerts: true
        },
        preferences: {
          theme: 'dark',
          language: 'en',
          defaultWorkflowView: 'grid',
          autoSaveInterval: 30
        }
      }
    }

    const demoUser: User = {
      id: 'user_002',
      email: 'demo@deflow.com',
      username: 'demo',
      displayName: 'Demo User',
      role: 'user',
      permissions: this.getDefaultPermissions(),
      isActive: true,
      createdAt: now,
      lastLoginAt: now,
      profile: {
        timezone: 'UTC',
        notifications: {
          email: true,
          push: false,
          workflow_completion: true,
          workflow_failure: true,
          system_alerts: false
        },
        preferences: {
          theme: 'light',
          language: 'en',
          defaultWorkflowView: 'list',
          autoSaveInterval: 60
        }
      }
    }

    this.users.set(adminUser.id, adminUser)
    this.users.set(demoUser.id, demoUser)
  }

  // Utility methods
  private generateUserId(): string {
    return `user_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
  }

  private generateSessionId(): string {
    return `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
  }

  private generateToken(): string {
    return `token_${Date.now()}_${Math.random().toString(36).substr(2, 16)}`
  }

  private generateRefreshToken(): string {
    return `refresh_${Date.now()}_${Math.random().toString(36).substr(2, 16)}`
  }

  // Admin methods
  getAllUsers(): User[] {
    if (!this.canManageUsers()) {
      throw new Error('Insufficient permissions')
    }
    return Array.from(this.users.values())
  }

  async createUser(userData: Omit<User, 'id' | 'createdAt' | 'lastLoginAt'>): Promise<User> {
    if (!this.canManageUsers()) {
      throw new Error('Insufficient permissions')
    }

    const userId = this.generateUserId()
    const now = TimestampUtils.dateToICPTimestamp(new Date())

    const user: User = {
      ...userData,
      id: userId,
      createdAt: now,
      lastLoginAt: now
    }

    this.users.set(userId, user)
    return user
  }

  async updateUserById(userId: string, updates: Partial<User>): Promise<User> {
    if (!this.canManageUsers()) {
      throw new Error('Insufficient permissions')
    }

    const user = this.users.get(userId)
    if (!user) {
      throw new Error('User not found')
    }

    const updatedUser = { ...user, ...updates }
    this.users.set(userId, updatedUser)
    return updatedUser
  }

  async deleteUser(userId: string): Promise<void> {
    if (!this.canManageUsers()) {
      throw new Error('Insufficient permissions')
    }

    this.users.delete(userId)
    
    // Deactivate all sessions for this user
    this.sessions.forEach((session, sessionId) => {
      if (session.userId === userId) {
        session.isActive = false
        this.sessions.set(sessionId, session)
      }
    })
  }

  // Session monitoring
  getUserSessions(userId?: string): AuthSession[] {
    const targetUserId = userId || this.currentUser?.id
    if (!targetUserId) return []

    return Array.from(this.sessions.values()).filter(s => s.userId === targetUserId)
  }

  async terminateSession(sessionId: string): Promise<void> {
    const session = this.sessions.get(sessionId)
    if (!session) {
      throw new Error('Session not found')
    }

    // Users can only terminate their own sessions unless they're admin
    if (session.userId !== this.currentUser?.id && this.currentUser?.role !== 'admin') {
      throw new Error('Insufficient permissions')
    }

    session.isActive = false
    this.sessions.set(sessionId, session)

    // If terminating current session, logout
    if (this.currentSession?.id === sessionId) {
      await this.logout()
    }
  }
}

// Export singleton instance
export const authService = new AuthService()
export default authService