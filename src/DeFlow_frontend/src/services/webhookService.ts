// Webhook system for external integrations in DeFlow
import { Workflow } from '../types'
import { executionEngine } from './executionEngine'
import { TimestampUtils } from '../utils/timestamp-utils'

export interface WebhookEndpoint {
  id: string
  workflowId: string
  path: string
  method: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE'
  isActive: boolean
  secret?: string
  headers: Record<string, string>
  validation: WebhookValidation
  rateLimiting: RateLimitConfig
  createdAt: string
  lastTriggeredAt?: string
  triggerCount: number
  metadata: WebhookMetadata
}

export interface WebhookValidation {
  enabled: boolean
  signatureHeader?: string
  secretKey?: string
  ipWhitelist?: string[]
  requiredHeaders?: string[]
  bodySchema?: any
}

export interface RateLimitConfig {
  enabled: boolean
  maxRequests: number
  timeWindow: number // seconds
  strategy: 'fixed_window' | 'sliding_window' | 'token_bucket'
}

export interface WebhookMetadata {
  name: string
  description: string
  tags: string[]
  externalService?: string
  documentation?: string
}

export interface WebhookRequest {
  id: string
  endpointId: string
  method: string
  path: string
  headers: Record<string, string>
  body: any
  query: Record<string, string>
  ip: string
  userAgent?: string
  timestamp: string
  processed: boolean
  executionId?: string
  error?: string
  responseStatus?: number
  responseTime?: number
}

export interface WebhookResponse {
  status: number
  headers: Record<string, string>
  body: any
  timestamp: string
}

export interface OutgoingWebhook {
  id: string
  workflowId: string
  name: string
  url: string
  method: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE'
  headers: Record<string, string>
  bodyTemplate: string
  triggers: WebhookTrigger[]
  retryConfig: RetryConfig
  isActive: boolean
  createdAt: string
  lastExecutedAt?: string
  executionCount: number
}

export interface WebhookTrigger {
  event: 'workflow_start' | 'workflow_complete' | 'workflow_fail' | 'node_complete' | 'node_fail'
  conditions?: Record<string, any>
}

export interface RetryConfig {
  maxRetries: number
  backoffStrategy: 'linear' | 'exponential' | 'fixed'
  initialDelay: number
  maxDelay: number
  retryOn: number[] // HTTP status codes to retry on
}

class WebhookService {
  private endpoints: Map<string, WebhookEndpoint> = new Map()
  private requests: Map<string, WebhookRequest> = new Map()
  private outgoingWebhooks: Map<string, OutgoingWebhook> = new Map()
  private rateLimitCache: Map<string, RateLimitState> = new Map()
  private baseUrl: string = import.meta.env.VITE_DFX_NETWORK === 'ic' 
    ? `https://${import.meta.env.VITE_CANISTER_ID_DEFLOW_FRONTEND}.ic0.app`
    : 'http://localhost:4943'

  constructor() {
    this.initializeDemoEndpoints()
    this.setupCleanupJob()
  }

  // Incoming webhook management
  createEndpoint(
    workflowId: string,
    config: Omit<WebhookEndpoint, 'id' | 'createdAt' | 'lastTriggeredAt' | 'triggerCount'>
  ): WebhookEndpoint {
    const endpoint: WebhookEndpoint = {
      ...config,
      id: this.generateId('webhook'),
      workflowId,
      createdAt: TimestampUtils.dateToICPTimestamp(new Date()),
      triggerCount: 0
    }

    // Ensure path is unique
    const existingEndpoint = Array.from(this.endpoints.values()).find(e => 
      e.path === endpoint.path && e.method === endpoint.method && e.isActive
    )
    if (existingEndpoint) {
      throw new Error(`Endpoint ${endpoint.method} ${endpoint.path} already exists`)
    }

    this.endpoints.set(endpoint.id, endpoint)
    return endpoint
  }

  updateEndpoint(endpointId: string, updates: Partial<WebhookEndpoint>): WebhookEndpoint {
    const endpoint = this.endpoints.get(endpointId)
    if (!endpoint) {
      throw new Error('Webhook endpoint not found')
    }

    const updatedEndpoint = { ...endpoint, ...updates }
    this.endpoints.set(endpointId, updatedEndpoint)
    return updatedEndpoint
  }

  deleteEndpoint(endpointId: string): boolean {
    return this.endpoints.delete(endpointId)
  }

  getEndpoint(endpointId: string): WebhookEndpoint | null {
    return this.endpoints.get(endpointId) || null
  }

  getWorkflowEndpoints(workflowId: string): WebhookEndpoint[] {
    return Array.from(this.endpoints.values()).filter(e => e.workflowId === workflowId)
  }

  getAllEndpoints(): WebhookEndpoint[] {
    return Array.from(this.endpoints.values())
  }

  // Webhook request processing
  async processWebhookRequest(
    method: string,
    path: string,
    headers: Record<string, string>,
    body: any,
    query: Record<string, string> = {},
    ip: string = '127.0.0.1'
  ): Promise<WebhookResponse> {
    const requestId = this.generateId('req')
    const timestamp = TimestampUtils.dateToICPTimestamp(new Date())

    const request: WebhookRequest = {
      id: requestId,
      endpointId: '',
      method,
      path,
      headers,
      body,
      query,
      ip,
      userAgent: headers['user-agent'],
      timestamp,
      processed: false
    }

    try {
      // Find matching endpoint
      const endpoint = this.findMatchingEndpoint(method, path)
      if (!endpoint) {
        request.error = 'Endpoint not found'
        request.responseStatus = 404
        this.requests.set(requestId, request)
        
        return {
          status: 404,
          headers: { 'content-type': 'application/json' },
          body: { error: 'Webhook endpoint not found' },
          timestamp
        }
      }

      request.endpointId = endpoint.id

      // Check if endpoint is active
      if (!endpoint.isActive) {
        request.error = 'Endpoint disabled'
        request.responseStatus = 410
        this.requests.set(requestId, request)
        
        return {
          status: 410,
          headers: { 'content-type': 'application/json' },
          body: { error: 'Webhook endpoint is disabled' },
          timestamp
        }
      }

      // Rate limiting
      if (endpoint.rateLimiting.enabled) {
        const rateLimitResult = this.checkRateLimit(endpoint, ip)
        if (!rateLimitResult.allowed) {
          request.error = 'Rate limit exceeded'
          request.responseStatus = 429
          this.requests.set(requestId, request)
          
          return {
            status: 429,
            headers: { 
              'content-type': 'application/json',
              'x-ratelimit-limit': endpoint.rateLimiting.maxRequests.toString(),
              'x-ratelimit-remaining': '0',
              'x-ratelimit-reset': rateLimitResult.resetTime.toString()
            },
            body: { error: 'Rate limit exceeded' },
            timestamp
          }
        }
      }

      // Validation
      const validationResult = this.validateRequest(endpoint, request)
      if (!validationResult.valid) {
        request.error = validationResult.error
        request.responseStatus = 400
        this.requests.set(requestId, request)
        
        return {
          status: 400,
          headers: { 'content-type': 'application/json' },
          body: { error: validationResult.error },
          timestamp
        }
      }

      // Process webhook
      const startTime = Date.now()
      const execution = await this.triggerWorkflow(endpoint, request)
      const responseTime = Date.now() - startTime

      // Update request and endpoint
      request.processed = true
      request.executionId = execution.id
      request.responseStatus = 200
      request.responseTime = responseTime
      this.requests.set(requestId, request)

      endpoint.lastTriggeredAt = timestamp
      endpoint.triggerCount++
      this.endpoints.set(endpoint.id, endpoint)

      return {
        status: 200,
        headers: { 
          'content-type': 'application/json',
          'x-execution-id': execution.id
        },
        body: { 
          success: true, 
          executionId: execution.id,
          message: 'Webhook processed successfully'
        },
        timestamp
      }

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error'
      request.error = errorMessage
      request.responseStatus = 500
      this.requests.set(requestId, request)

      return {
        status: 500,
        headers: { 'content-type': 'application/json' },
        body: { error: 'Internal server error' },
        timestamp
      }
    }
  }

  // Outgoing webhook management
  createOutgoingWebhook(config: Omit<OutgoingWebhook, 'id' | 'createdAt' | 'lastExecutedAt' | 'executionCount'>): OutgoingWebhook {
    const webhook: OutgoingWebhook = {
      ...config,
      id: this.generateId('outgoing'),
      createdAt: TimestampUtils.dateToICPTimestamp(new Date()),
      executionCount: 0
    }

    this.outgoingWebhooks.set(webhook.id, webhook)
    return webhook
  }

  updateOutgoingWebhook(webhookId: string, updates: Partial<OutgoingWebhook>): OutgoingWebhook {
    const webhook = this.outgoingWebhooks.get(webhookId)
    if (!webhook) {
      throw new Error('Outgoing webhook not found')
    }

    const updatedWebhook = { ...webhook, ...updates }
    this.outgoingWebhooks.set(webhookId, updatedWebhook)
    return updatedWebhook
  }

  deleteOutgoingWebhook(webhookId: string): boolean {
    return this.outgoingWebhooks.delete(webhookId)
  }

  getOutgoingWebhook(webhookId: string): OutgoingWebhook | null {
    return this.outgoingWebhooks.get(webhookId) || null
  }

  getWorkflowOutgoingWebhooks(workflowId: string): OutgoingWebhook[] {
    return Array.from(this.outgoingWebhooks.values()).filter(w => w.workflowId === workflowId)
  }

  // Outgoing webhook execution
  async executeOutgoingWebhooks(
    event: WebhookTrigger['event'],
    workflowId: string,
    data: any
  ): Promise<void> {
    const webhooks = this.getWorkflowOutgoingWebhooks(workflowId).filter(w => 
      w.isActive && w.triggers.some(t => t.event === event)
    )

    await Promise.all(webhooks.map(webhook => this.executeOutgoingWebhook(webhook, data)))
  }

  private async executeOutgoingWebhook(webhook: OutgoingWebhook, data: any): Promise<void> {
    try {
      const processedBody = this.processTemplate(webhook.bodyTemplate, data)
      const processedHeaders = this.processHeaders(webhook.headers, data)

      // Simulate HTTP request
      await new Promise(resolve => setTimeout(resolve, 100 + Math.random() * 400))

      webhook.lastExecutedAt = TimestampUtils.dateToICPTimestamp(new Date())
      webhook.executionCount++
      this.outgoingWebhooks.set(webhook.id, webhook)


    } catch (error) {
      console.error(`[Outgoing Webhook Error] ${webhook.name}:`, error)
      
      // Implement retry logic here based on webhook.retryConfig
      // For now, just log the error
    }
  }

  // Helper methods
  private findMatchingEndpoint(method: string, path: string): WebhookEndpoint | null {
    return Array.from(this.endpoints.values()).find(e => 
      e.method === method && e.path === path
    ) || null
  }

  private checkRateLimit(endpoint: WebhookEndpoint, ip: string): { allowed: boolean; resetTime: number } {
    const key = `${endpoint.id}:${ip}`
    const now = Date.now()
    const windowMs = endpoint.rateLimiting.timeWindow * 1000

    let state = this.rateLimitCache.get(key)
    if (!state) {
      state = {
        requests: 0,
        windowStart: now,
        lastRequest: 0
      }
    }

    // Reset window if expired
    if (now - state.windowStart >= windowMs) {
      state.requests = 0
      state.windowStart = now
    }

    state.requests++
    state.lastRequest = now
    this.rateLimitCache.set(key, state)

    const allowed = state.requests <= endpoint.rateLimiting.maxRequests
    const resetTime = state.windowStart + windowMs

    return { allowed, resetTime }
  }

  private validateRequest(endpoint: WebhookEndpoint, request: WebhookRequest): { valid: boolean; error?: string } {
    if (!endpoint.validation.enabled) {
      return { valid: true }
    }

    // IP whitelist validation
    if (endpoint.validation.ipWhitelist && endpoint.validation.ipWhitelist.length > 0) {
      if (!endpoint.validation.ipWhitelist.includes(request.ip)) {
        return { valid: false, error: 'IP not whitelisted' }
      }
    }

    // Required headers validation
    if (endpoint.validation.requiredHeaders) {
      for (const header of endpoint.validation.requiredHeaders) {
        if (!request.headers[header.toLowerCase()]) {
          return { valid: false, error: `Missing required header: ${header}` }
        }
      }
    }

    // Signature validation (simplified)
    if (endpoint.validation.signatureHeader && endpoint.validation.secretKey) {
      const providedSignature = request.headers[endpoint.validation.signatureHeader.toLowerCase()]
      if (!providedSignature) {
        return { valid: false, error: 'Missing signature header' }
      }
      // In real implementation, verify HMAC signature
    }

    return { valid: true }
  }

  private async triggerWorkflow(endpoint: WebhookEndpoint, request: WebhookRequest): Promise<any> {
    // Find workflow (in real implementation, this would fetch from database)
    // For now, create a mock execution
    const triggerData = {
      webhook: {
        path: request.path,
        method: request.method,
        headers: request.headers,
        body: request.body,
        query: request.query,
        ip: request.ip
      },
      timestamp: request.timestamp
    }

    // This would integrate with the workflow execution system
    const execution = await executionEngine.executeWorkflow(
      { id: endpoint.workflowId } as any, // Mock workflow
      triggerData,
      'webhook-system'
    )

    return execution
  }

  private processTemplate(template: string, data: any): string {
    if (!template) return ''
    
    return template.replace(/\{\{([^}]+)\}\}/g, (match, path) => {
      const value = this.getValueByPath(data, path.trim())
      return value !== undefined ? JSON.stringify(value) : match
    })
  }

  private processHeaders(headers: Record<string, string>, data: any): Record<string, string> {
    const processed: Record<string, string> = {}
    
    for (const [key, value] of Object.entries(headers)) {
      processed[key] = this.processTemplate(value, data)
    }
    
    return processed
  }

  private getValueByPath(obj: any, path: string): any {
    return path.split('.').reduce((current, key) => current?.[key], obj)
  }

  private generateId(prefix: string): string {
    return `${prefix}_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
  }

  // Analytics and monitoring
  getEndpointAnalytics(endpointId: string): {
    totalRequests: number
    successfulRequests: number
    failedRequests: number
    averageResponseTime: number
    lastTriggered?: string
    recentRequests: WebhookRequest[]
  } {
    const requests = Array.from(this.requests.values()).filter(r => r.endpointId === endpointId)
    const successful = requests.filter(r => r.responseStatus && r.responseStatus < 400)
    const failed = requests.filter(r => r.responseStatus && r.responseStatus >= 400)
    
    const avgResponseTime = requests
      .filter(r => r.responseTime)
      .reduce((sum, r) => sum + r.responseTime!, 0) / requests.length || 0

    const endpoint = this.endpoints.get(endpointId)

    return {
      totalRequests: requests.length,
      successfulRequests: successful.length,
      failedRequests: failed.length,
      averageResponseTime: avgResponseTime,
      lastTriggered: endpoint?.lastTriggeredAt,
      recentRequests: requests
        .sort((a, b) => b.timestamp.localeCompare(a.timestamp))
        .slice(0, 10)
    }
  }

  getSystemAnalytics(): {
    totalEndpoints: number
    activeEndpoints: number
    totalRequests: number
    requestsToday: number
    averageResponseTime: number
    topEndpoints: Array<{ endpointId: string; requests: number }>
  } {
    const endpoints = Array.from(this.endpoints.values())
    const requests = Array.from(this.requests.values())
    
    const today = new Date()
    today.setHours(0, 0, 0, 0)
    const todayRequests = requests.filter(r => {
      const requestDate = TimestampUtils.icpTimestampToDate(r.timestamp)
      return requestDate >= today
    })

    const avgResponseTime = requests
      .filter(r => r.responseTime)
      .reduce((sum, r) => sum + r.responseTime!, 0) / requests.length || 0

    // Calculate top endpoints
    const endpointCounts = new Map<string, number>()
    requests.forEach(r => {
      if (r.endpointId) {
        endpointCounts.set(r.endpointId, (endpointCounts.get(r.endpointId) || 0) + 1)
      }
    })

    const topEndpoints = Array.from(endpointCounts.entries())
      .map(([endpointId, count]) => ({ endpointId, requests: count }))
      .sort((a, b) => b.requests - a.requests)
      .slice(0, 10)

    return {
      totalEndpoints: endpoints.length,
      activeEndpoints: endpoints.filter(e => e.isActive).length,
      totalRequests: requests.length,
      requestsToday: todayRequests.length,
      averageResponseTime: avgResponseTime,
      topEndpoints
    }
  }

  // Utility methods
  generateWebhookUrl(endpoint: WebhookEndpoint): string {
    return `${this.baseUrl}/webhook${endpoint.path}`
  }

  private initializeDemoEndpoints(): void {
    // Create some demo webhook endpoints
    const demoEndpoint: WebhookEndpoint = {
      id: 'webhook_001',
      workflowId: 'workflow_001',
      path: '/webhook/demo',
      method: 'POST',
      isActive: true,
      headers: {},
      validation: {
        enabled: false
      },
      rateLimiting: {
        enabled: true,
        maxRequests: 100,
        timeWindow: 60,
        strategy: 'fixed_window'
      },
      createdAt: TimestampUtils.dateToICPTimestamp(new Date()),
      triggerCount: 0,
      metadata: {
        name: 'Demo Webhook',
        description: 'Demo webhook endpoint for testing',
        tags: ['demo', 'test'],
        externalService: 'Demo Service'
      }
    }

    this.endpoints.set(demoEndpoint.id, demoEndpoint)
  }

  private setupCleanupJob(): void {
    // Clean up old requests every hour
    setInterval(() => {
      const oneWeekAgo = new Date(Date.now() - 7 * 24 * 60 * 60 * 1000)
      
      this.requests.forEach((request, id) => {
        const requestDate = TimestampUtils.icpTimestampToDate(request.timestamp)
        if (requestDate < oneWeekAgo) {
          this.requests.delete(id)
        }
      })

      // Clean up rate limit cache
      this.rateLimitCache.clear()
    }, 60 * 60 * 1000) // 1 hour
  }
}

interface RateLimitState {
  requests: number
  windowStart: number
  lastRequest: number
}

// Export singleton instance
export const webhookService = new WebhookService()
export default webhookService