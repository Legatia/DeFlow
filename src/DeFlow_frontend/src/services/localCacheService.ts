// Import BigInt polyfill FIRST to prevent conversion errors
import '../utils/bigint-polyfill'
// SECURITY: Import secure storage for sensitive data
import secureStorageService from './secureStorageService'

// Local cache for guest users and authenticated users
export interface CachedWorkflow {
  id: string
  name: string
  description: string
  nodes: any[]
  connections: any[]
  createdAt: number
  updatedAt: number
  isTemplate?: boolean
  category?: string
}

export interface CachedExecution {
  id: string
  workflowId: string
  status: 'pending' | 'running' | 'completed' | 'failed'
  startedAt: number
  completedAt?: number
  results?: any
  error?: string
}

export interface CachedNotification {
  id: string
  title: string
  message: string
  type: 'info' | 'success' | 'warning' | 'error'
  createdAt: number
  read: boolean
  data?: any
}

export interface UserPreferences {
  theme: 'light' | 'dark'
  notifications: {
    browser: boolean
    sound: boolean
    workflowComplete: boolean
    portfolioAlerts: boolean
  }
  defaultChains: string[]
  autoSave: boolean
}

export interface LinkedInConfig {
  id: string
  name: string
  access_token: string
  post_type: 'person' | 'organization'
  organization_id: string
  createdAt: string
}

export interface FacebookConfig {
  id: string
  name: string
  access_token: string
  page_id: string
  post_type: 'page' | 'group' | 'event'
  createdAt: string
}

class LocalCacheService {
  private readonly STORAGE_KEYS = {
    WORKFLOWS: 'deflow_cached_workflows',
    EXECUTIONS: 'deflow_cached_executions', 
    NOTIFICATIONS: 'deflow_cached_notifications',
    USER_PREFERENCES: 'deflow_user_preferences',
    WALLET_ADDRESSES: 'deflow_cached_wallets',
    LINKEDIN_CONFIGS: 'deflow_linkedin_configs',
    FACEBOOK_CONFIGS: 'deflow_facebook_configs'
  }

  // Workflows
  saveWorkflow(workflow: CachedWorkflow): boolean {
    try {
      const workflows = this.getWorkflows()
      const existingIndex = workflows.findIndex(w => w.id === workflow.id)
      
      if (existingIndex >= 0) {
        workflows[existingIndex] = { ...workflow, updatedAt: Date.now() }
      } else {
        workflows.push(workflow)
      }
      
      localStorage.setItem(this.STORAGE_KEYS.WORKFLOWS, JSON.stringify(workflows))
      
      // Add notification
      this.addNotification({
        id: `workflow_saved_${Date.now()}`,
        title: 'Workflow Saved',
        message: `"${workflow.name}" has been saved locally`,
        type: 'success',
        createdAt: Date.now(),
        read: false
      })
      
      return true
    } catch (error) {
      console.error('Failed to save workflow:', error)
      return false
    }
  }

  getWorkflows(): CachedWorkflow[] {
    try {
      const stored = localStorage.getItem(this.STORAGE_KEYS.WORKFLOWS)
      return stored ? JSON.parse(stored) : []
    } catch (error) {
      console.error('Failed to load workflows:', error)
      return []
    }
  }

  getWorkflow(id: string): CachedWorkflow | null {
    const workflows = this.getWorkflows()
    return workflows.find(w => w.id === id) || null
  }

  deleteWorkflow(id: string): boolean {
    try {
      const workflows = this.getWorkflows()
      const filteredWorkflows = workflows.filter(w => w.id !== id)
      localStorage.setItem(this.STORAGE_KEYS.WORKFLOWS, JSON.stringify(filteredWorkflows))
      
      // Also delete related executions
      this.deleteExecutionsByWorkflow(id)
      
      return true
    } catch (error) {
      console.error('Failed to delete workflow:', error)
      return false
    }
  }

  // Executions
  saveExecution(execution: CachedExecution): boolean {
    try {
      const executions = this.getExecutions()
      const existingIndex = executions.findIndex(e => e.id === execution.id)
      
      if (existingIndex >= 0) {
        executions[existingIndex] = execution
      } else {
        executions.push(execution)
      }
      
      localStorage.setItem(this.STORAGE_KEYS.EXECUTIONS, JSON.stringify(executions))
      
      // Add notification for completed executions
      if (execution.status === 'completed' || execution.status === 'failed') {
        this.addNotification({
          id: `execution_${execution.status}_${Date.now()}`,
          title: `Workflow ${execution.status === 'completed' ? 'Completed' : 'Failed'}`,
          message: `Execution ${execution.id} has ${execution.status}`,
          type: execution.status === 'completed' ? 'success' : 'error',
          createdAt: Date.now(),
          read: false,
          data: { executionId: execution.id, workflowId: execution.workflowId }
        })
      }
      
      return true
    } catch (error) {
      console.error('Failed to save execution:', error)
      return false
    }
  }

  getExecutions(): CachedExecution[] {
    try {
      const stored = localStorage.getItem(this.STORAGE_KEYS.EXECUTIONS)
      return stored ? JSON.parse(stored) : []
    } catch (error) {
      console.error('Failed to load executions:', error)
      return []
    }
  }

  getExecutionsByWorkflow(workflowId: string): CachedExecution[] {
    return this.getExecutions().filter(e => e.workflowId === workflowId)
  }

  deleteExecutionsByWorkflow(workflowId: string): boolean {
    try {
      const executions = this.getExecutions()
      const filteredExecutions = executions.filter(e => e.workflowId !== workflowId)
      localStorage.setItem(this.STORAGE_KEYS.EXECUTIONS, JSON.stringify(filteredExecutions))
      return true
    } catch (error) {
      console.error('Failed to delete executions for workflow:', error)
      return false
    }
  }

  // Notifications
  addNotification(notification: CachedNotification): boolean {
    try {
      const notifications = this.getNotifications()
      notifications.unshift(notification) // Add to beginning
      
      // Keep only last 100 notifications
      const limitedNotifications = notifications.slice(0, 100)
      
      localStorage.setItem(this.STORAGE_KEYS.NOTIFICATIONS, JSON.stringify(limitedNotifications))
      
      // Show browser notification if enabled
      this.showBrowserNotification(notification)
      
      return true
    } catch (error) {
      console.error('Failed to add notification:', error)
      return false
    }
  }

  getNotifications(): CachedNotification[] {
    try {
      const stored = localStorage.getItem(this.STORAGE_KEYS.NOTIFICATIONS)
      return stored ? JSON.parse(stored) : []
    } catch (error) {
      console.error('Failed to load notifications:', error)
      return []
    }
  }

  markNotificationRead(id: string): boolean {
    try {
      const notifications = this.getNotifications()
      const notification = notifications.find(n => n.id === id)
      
      if (notification) {
        notification.read = true
        localStorage.setItem(this.STORAGE_KEYS.NOTIFICATIONS, JSON.stringify(notifications))
        return true
      }
      
      return false
    } catch (error) {
      console.error('Failed to mark notification as read:', error)
      return false
    }
  }

  markAllNotificationsRead(): boolean {
    try {
      const notifications = this.getNotifications()
      notifications.forEach(n => n.read = true)
      localStorage.setItem(this.STORAGE_KEYS.NOTIFICATIONS, JSON.stringify(notifications))
      return true
    } catch (error) {
      console.error('Failed to mark all notifications as read:', error)
      return false
    }
  }

  getUnreadNotificationCount(): number {
    return this.getNotifications().filter(n => !n.read).length
  }

  // User Preferences
  getUserPreferences(): UserPreferences {
    try {
      const stored = localStorage.getItem(this.STORAGE_KEYS.USER_PREFERENCES)
      return stored ? JSON.parse(stored) : {
        theme: 'light',
        notifications: {
          browser: true,
          sound: true,
          workflowComplete: true,
          portfolioAlerts: true
        },
        defaultChains: ['Ethereum', 'Bitcoin'],
        autoSave: true
      }
    } catch (error) {
      console.error('Failed to load user preferences:', error)
      return {
        theme: 'light',
        notifications: {
          browser: true,
          sound: true,
          workflowComplete: true,
          portfolioAlerts: true
        },
        defaultChains: ['Ethereum', 'Bitcoin'],
        autoSave: true
      }
    }
  }

  saveUserPreferences(preferences: UserPreferences): boolean {
    try {
      localStorage.setItem(this.STORAGE_KEYS.USER_PREFERENCES, JSON.stringify(preferences))
      return true
    } catch (error) {
      console.error('Failed to save user preferences:', error)
      return false
    }
  }

  // SECURITY: Browser notifications with explicit user consent
  private async showBrowserNotification(notification: CachedNotification): Promise<void> {
    const preferences = this.getUserPreferences()
    
    if (!preferences.notifications.browser) return

    // SECURITY: Only show notifications if user has explicitly granted permission
    if ('Notification' in window && Notification.permission === 'granted') {
      const notif = new Notification(notification.title, {
        body: notification.message,
        icon: '/favicon.ico',
        badge: '/favicon.ico',
        tag: notification.id
      })

      // Auto close after 5 seconds
      setTimeout(() => notif.close(), 5000)
    }
    // Note: Permission request is now handled explicitly in user settings
  }

  // SECURITY: Explicit method to request notification permission
  async requestNotificationPermission(): Promise<boolean> {
    if (!('Notification' in window)) {
      console.warn('Browser does not support notifications')
      return false
    }

    if (Notification.permission === 'granted') {
      return true
    }

    if (Notification.permission === 'denied') {
      console.warn('Notification permission has been denied by user')
      return false
    }

    try {
      const permission = await Notification.requestPermission()
      return permission === 'granted'
    } catch (error) {
      console.error('Failed to request notification permission:', error)
      return false
    }
  }

  // SECURITY: Check notification permission status
  getNotificationPermissionStatus(): 'granted' | 'denied' | 'default' | 'unsupported' {
    if (!('Notification' in window)) {
      return 'unsupported'
    }
    return Notification.permission
  }

  // Cache management
  getCacheSize(): string {
    let totalSize = 0
    
    Object.values(this.STORAGE_KEYS).forEach(key => {
      const value = localStorage.getItem(key)
      if (value) {
        totalSize += new Blob([value]).size
      }
    })
    
    return this.formatBytes(totalSize)
  }

  clearCache(): boolean {
    try {
      Object.values(this.STORAGE_KEYS).forEach(key => {
        localStorage.removeItem(key)
      })
      
      this.addNotification({
        id: `cache_cleared_${Date.now()}`,
        title: 'Cache Cleared',
        message: 'All local data has been cleared',
        type: 'info',
        createdAt: Date.now(),
        read: false
      })
      
      return true
    } catch (error) {
      console.error('Failed to clear cache:', error)
      return false
    }
  }

  // Export data for authenticated users
  exportData(): string {
    const data = {
      workflows: this.getWorkflows(),
      executions: this.getExecutions(),
      preferences: this.getUserPreferences(),
      exportedAt: Date.now()
    }
    
    return JSON.stringify(data, null, 2)
  }

  // Import data (for authenticated users syncing from cloud)
  importData(jsonData: string): boolean {
    try {
      const data = JSON.parse(jsonData)
      
      if (data.workflows) {
        localStorage.setItem(this.STORAGE_KEYS.WORKFLOWS, JSON.stringify(data.workflows))
      }
      
      if (data.executions) {
        localStorage.setItem(this.STORAGE_KEYS.EXECUTIONS, JSON.stringify(data.executions))
      }
      
      if (data.preferences) {
        localStorage.setItem(this.STORAGE_KEYS.USER_PREFERENCES, JSON.stringify(data.preferences))
      }
      
      this.addNotification({
        id: `data_imported_${Date.now()}`,
        title: 'Data Imported',
        message: 'Your data has been imported successfully',
        type: 'success',
        createdAt: Date.now(),
        read: false
      })
      
      return true
    } catch (error) {
      console.error('Failed to import data:', error)
      return false
    }
  }

  // SECURITY: LinkedIn Configurations with encryption
  async getLinkedInConfigs(): Promise<LinkedInConfig[]> {
    try {
      if (!secureStorageService.isEncryptionReady()) {
        await secureStorageService.initializeEncryption()
      }
      
      const configs = await secureStorageService.getSecureItem<LinkedInConfig[]>('linkedin_configs')
      return configs || []
    } catch (error) {
      console.error('SECURITY: Failed to load encrypted LinkedIn configs:', error)
      // Fallback to unencrypted (migration)
      try {
        const stored = localStorage.getItem(this.STORAGE_KEYS.LINKEDIN_CONFIGS)
        return stored ? JSON.parse(stored) : []
      } catch (fallbackError) {
        console.error('Failed to load LinkedIn configs (fallback):', fallbackError)
        return []
      }
    }
  }

  async saveLinkedInConfigs(configs: LinkedInConfig[]): Promise<boolean> {
    try {
      if (!secureStorageService.isEncryptionReady()) {
        await secureStorageService.initializeEncryption()
      }
      
      // SECURITY: Store encrypted
      const success = await secureStorageService.setSecureItem('linkedin_configs', configs)
      
      if (success) {
        // Remove unencrypted version
        localStorage.removeItem(this.STORAGE_KEYS.LINKEDIN_CONFIGS)
      }
      
      return success
    } catch (error) {
      console.error('SECURITY: Failed to save encrypted LinkedIn configs:', error)
      return false
    }
  }

  // SECURITY: Facebook Configurations with encryption
  async getFacebookConfigs(): Promise<FacebookConfig[]> {
    try {
      if (!secureStorageService.isEncryptionReady()) {
        await secureStorageService.initializeEncryption()
      }
      
      const configs = await secureStorageService.getSecureItem<FacebookConfig[]>('facebook_configs')
      return configs || []
    } catch (error) {
      console.error('SECURITY: Failed to load encrypted Facebook configs:', error)
      // Fallback to unencrypted (migration)
      try {
        const stored = localStorage.getItem(this.STORAGE_KEYS.FACEBOOK_CONFIGS)
        return stored ? JSON.parse(stored) : []
      } catch (fallbackError) {
        console.error('Failed to load Facebook configs (fallback):', fallbackError)
        return []
      }
    }
  }

  async saveFacebookConfigs(configs: FacebookConfig[]): Promise<boolean> {
    try {
      if (!secureStorageService.isEncryptionReady()) {
        await secureStorageService.initializeEncryption()
      }
      
      // SECURITY: Store encrypted
      const success = await secureStorageService.setSecureItem('facebook_configs', configs)
      
      if (success) {
        // Remove unencrypted version
        localStorage.removeItem(this.STORAGE_KEYS.FACEBOOK_CONFIGS)
      }
      
      return success
    } catch (error) {
      console.error('SECURITY: Failed to save encrypted Facebook configs:', error)
      return false
    }
  }

  private formatBytes(bytes: number): string {
    if (bytes === 0) return '0 Bytes'
    
    const k = 1024
    const sizes = ['Bytes', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }
}

export default new LocalCacheService()