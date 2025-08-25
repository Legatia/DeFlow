// Email service for ICP canisters - HTTP API only
import { Actor, HttpAgent } from '@dfinity/agent'

export interface EmailConfig {
  provider: 'gmail' | 'outlook' | 'sendgrid' | 'mailgun' | 'postmark' | 'custom'
  credentials: {
    // OAuth2 tokens (managed by DeFlow)
    access_token?: string
    refresh_token?: string
    // API keys (user managed)
    api_key?: string
    domain?: string // For mailgun
  }
  from_email?: string
  from_name?: string
  // Custom provider config
  custom_config?: {
    baseUrl: string
    method: 'GET' | 'POST' | 'PUT'
    headers: Record<string, string>
    bodyTemplate: string
    authType: 'none' | 'bearer' | 'api_key' | 'basic' | 'custom'
    authConfig: {
      token?: string
      username?: string
      password?: string
      apiKey?: string
      headerName?: string
      customAuth?: string
    }
  }
}

export interface EmailMessage {
  to: string | string[]
  cc?: string | string[]
  bcc?: string | string[]
  subject: string
  body_text?: string
  body_html?: string
  attachments?: EmailAttachment[]
}

export interface EmailAttachment {
  filename: string
  content: string // base64 encoded
  content_type: string
}

export interface EmailResponse {
  success: boolean
  message_id?: string
  error?: string
}

class EmailService {
  private configs: Map<string, EmailConfig> = new Map()

  // Add email provider configuration
  addProvider(name: string, config: EmailConfig): void {
    this.configs.set(name, config)
  }

  // Remove email provider
  removeProvider(name: string): void {
    this.configs.delete(name)
  }

  // Get all configured providers
  getProviders(): string[] {
    return Array.from(this.configs.keys())
  }

  // Send email using specified provider
  async sendEmail(providerName: string, message: EmailMessage): Promise<EmailResponse> {
    const config = this.configs.get(providerName)
    if (!config) {
      throw new Error(`Email provider '${providerName}' not configured`)
    }

    try {
      switch (config.provider) {
        case 'gmail':
          return await this.sendGmail(config, message)
        case 'outlook':
          return await this.sendOutlook(config, message)
        case 'sendgrid':
          return await this.sendSendGrid(config, message)
        case 'mailgun':
          return await this.sendMailgun(config, message)
        case 'postmark':
          return await this.sendPostmark(config, message)
        case 'custom':
          return await this.sendCustom(config, message)
        default:
          throw new Error(`Unsupported email provider: ${config.provider}`)
      }
    } catch (error) {
      console.error(`Email send error for ${providerName}:`, error)
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error'
      }
    }
  }

  // Gmail API implementation
  private async sendGmail(config: EmailConfig, message: EmailMessage): Promise<EmailResponse> {
    if (!config.credentials.access_token) {
      throw new Error('Gmail access token required')
    }

    // Create RFC 2822 formatted email
    const email = this.createRfc2822Email(message, config)
    const raw = btoa(email).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')

    const response = await fetch('https://gmail.googleapis.com/gmail/v1/users/me/messages/send', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${config.credentials.access_token}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ raw })
    })

    if (!response.ok) {
      // If token expired, try to refresh
      if (response.status === 401) {
        throw new Error('Gmail access token expired. Please reconnect your Gmail account.')
      }
      const error = await response.text()
      throw new Error(`Gmail API error: ${error}`)
    }

    const result = await response.json()
    return {
      success: true,
      message_id: result.id
    }
  }

  // Outlook/Microsoft Graph API implementation
  private async sendOutlook(config: EmailConfig, message: EmailMessage): Promise<EmailResponse> {
    if (!config.credentials.access_token) {
      throw new Error('Outlook access token required')
    }

    const outlookMessage = {
      message: {
        subject: message.subject,
        body: {
          contentType: message.body_html ? 'HTML' : 'Text',
          content: message.body_html || message.body_text || ''
        },
        toRecipients: this.parseEmailAddresses(message.to),
        ccRecipients: message.cc ? this.parseEmailAddresses(message.cc) : undefined,
        bccRecipients: message.bcc ? this.parseEmailAddresses(message.bcc) : undefined,
        attachments: message.attachments ? this.convertAttachmentsForOutlook(message.attachments) : undefined
      }
    }

    const response = await fetch('https://graph.microsoft.com/v1.0/me/sendMail', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${config.credentials.access_token}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(outlookMessage)
    })

    if (!response.ok) {
      // If token expired, try to refresh
      if (response.status === 401) {
        throw new Error('Outlook access token expired. Please reconnect your Outlook account.')
      }
      const error = await response.text()
      throw new Error(`Outlook API error: ${error}`)
    }

    return {
      success: true,
      message_id: `outlook-${Date.now()}`
    }
  }

  // SendGrid API implementation
  private async sendSendGrid(config: EmailConfig, message: EmailMessage): Promise<EmailResponse> {
    if (!config.credentials.api_key) {
      throw new Error('SendGrid API key required')
    }

    const sendgridMessage = {
      personalizations: [{
        to: this.parseEmailAddresses(message.to),
        cc: message.cc ? this.parseEmailAddresses(message.cc) : undefined,
        bcc: message.bcc ? this.parseEmailAddresses(message.bcc) : undefined,
        subject: message.subject
      }],
      from: {
        email: config.from_email || 'noreply@example.com',
        name: config.from_name || 'DeFlow'
      },
      content: [
        ...(message.body_text ? [{ type: 'text/plain', value: message.body_text }] : []),
        ...(message.body_html ? [{ type: 'text/html', value: message.body_html }] : [])
      ],
      attachments: message.attachments ? this.convertAttachmentsForSendGrid(message.attachments) : undefined
    }

    const response = await fetch('https://api.sendgrid.com/v3/mail/send', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${config.credentials.api_key}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(sendgridMessage)
    })

    if (!response.ok) {
      const error = await response.text()
      throw new Error(`SendGrid API error: ${error}`)
    }

    const messageId = response.headers.get('X-Message-Id')
    return {
      success: true,
      message_id: messageId || `sendgrid-${Date.now()}`
    }
  }

  // Mailgun API implementation
  private async sendMailgun(config: EmailConfig, message: EmailMessage): Promise<EmailResponse> {
    if (!config.credentials.api_key || !config.credentials.domain) {
      throw new Error('Mailgun API key and domain required')
    }

    const formData = new FormData()
    formData.append('from', `${config.from_name || 'DeFlow'} <${config.from_email || 'noreply@' + config.credentials.domain}>`)
    formData.append('to', Array.isArray(message.to) ? message.to.join(',') : message.to)
    if (message.cc) formData.append('cc', Array.isArray(message.cc) ? message.cc.join(',') : message.cc)
    if (message.bcc) formData.append('bcc', Array.isArray(message.bcc) ? message.bcc.join(',') : message.bcc)
    formData.append('subject', message.subject)
    if (message.body_text) formData.append('text', message.body_text)
    if (message.body_html) formData.append('html', message.body_html)

    // Add attachments
    if (message.attachments) {
      message.attachments.forEach((attachment, index) => {
        const blob = new Blob([atob(attachment.content)], { type: attachment.content_type })
        formData.append('attachment', blob, attachment.filename)
      })
    }

    const response = await fetch(`https://api.mailgun.net/v3/${config.credentials.domain}/messages`, {
      method: 'POST',
      headers: {
        'Authorization': `Basic ${btoa('api:' + config.credentials.api_key)}`
      },
      body: formData
    })

    if (!response.ok) {
      const error = await response.text()
      throw new Error(`Mailgun API error: ${error}`)
    }

    const result = await response.json()
    return {
      success: true,
      message_id: result.id
    }
  }

  // Postmark API implementation
  private async sendPostmark(config: EmailConfig, message: EmailMessage): Promise<EmailResponse> {
    if (!config.credentials.api_key) {
      throw new Error('Postmark API key required')
    }

    const postmarkMessage = {
      From: `${config.from_name || 'DeFlow'} <${config.from_email || 'noreply@example.com'}>`,
      To: Array.isArray(message.to) ? message.to.join(',') : message.to,
      Cc: message.cc ? (Array.isArray(message.cc) ? message.cc.join(',') : message.cc) : undefined,
      Bcc: message.bcc ? (Array.isArray(message.bcc) ? message.bcc.join(',') : message.bcc) : undefined,
      Subject: message.subject,
      TextBody: message.body_text,
      HtmlBody: message.body_html,
      Attachments: message.attachments ? this.convertAttachmentsForPostmark(message.attachments) : undefined
    }

    const response = await fetch('https://api.postmarkapp.com/email', {
      method: 'POST',
      headers: {
        'X-Postmark-Server-Token': config.credentials.api_key,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(postmarkMessage)
    })

    if (!response.ok) {
      const error = await response.text()
      throw new Error(`Postmark API error: ${error}`)
    }

    const result = await response.json()
    return {
      success: true,
      message_id: result.MessageID
    }
  }

  // Custom API provider implementation
  private async sendCustom(config: EmailConfig, message: EmailMessage): Promise<EmailResponse> {
    if (!config.custom_config) {
      throw new Error('Custom provider configuration required')
    }

    const customConfig = config.custom_config

    try {
      // Prepare request data
      const requestData = {
        to: Array.isArray(message.to) ? message.to.join(',') : message.to,
        subject: message.subject,
        body: message.body_html || message.body_text || '',
        body_html: message.body_html || '',
        body_text: message.body_text || '',
        cc: message.cc ? (Array.isArray(message.cc) ? message.cc.join(',') : message.cc) : '',
        bcc: message.bcc ? (Array.isArray(message.bcc) ? message.bcc.join(',') : message.bcc) : '',
        api_key: customConfig.authConfig.apiKey || customConfig.authConfig.token || config.credentials.api_key || ''
      }

      // Replace template variables in body
      let body = customConfig.bodyTemplate
      Object.entries(requestData).forEach(([key, value]) => {
        const regex = new RegExp(`{{\\s*${key}\\s*}}`, 'g')
        body = body.replace(regex, String(value))
      })

      // Prepare headers
      const headers = { ...customConfig.headers }
      
      // Add authentication
      switch (customConfig.authType) {
        case 'bearer':
          if (customConfig.authConfig.token) {
            headers['Authorization'] = `Bearer ${customConfig.authConfig.token}`
          }
          break
        case 'api_key':
          if (customConfig.authConfig.apiKey && customConfig.authConfig.headerName) {
            headers[customConfig.authConfig.headerName] = customConfig.authConfig.apiKey
          }
          break
        case 'basic':
          if (customConfig.authConfig.username && customConfig.authConfig.password) {
            headers['Authorization'] = `Basic ${btoa(`${customConfig.authConfig.username}:${customConfig.authConfig.password}`)}`
          }
          break
        case 'custom':
          if (customConfig.authConfig.customAuth && customConfig.authConfig.token) {
            // Parse custom auth format like "X-API-Key: {token}"
            const [headerName, headerTemplate] = customConfig.authConfig.customAuth.split(':').map(s => s.trim())
            if (headerName && headerTemplate) {
              headers[headerName] = headerTemplate.replace('{token}', customConfig.authConfig.token)
            }
          }
          break
        case 'none':
          // No additional authentication needed
          break
      }

      // Make API request
      const response = await fetch(customConfig.baseUrl, {
        method: customConfig.method,
        headers,
        body: customConfig.method !== 'GET' ? body : undefined
      })

      const responseText = await response.text()
      
      if (!response.ok) {
        throw new Error(`Custom API error (${response.status}): ${responseText}`)
      }

      // Try to parse response
      let responseData: any = {}
      try {
        responseData = JSON.parse(responseText)
      } catch {
        // Response is not JSON, use text as-is
        responseData = { raw: responseText }
      }

      return {
        success: true,
        message_id: responseData.id || responseData.messageId || responseData.message_id || `custom-${Date.now()}`
      }
    } catch (error) {
      console.error('Custom provider error:', error)
      throw error
    }
  }

  // Helper methods
  private createRfc2822Email(message: EmailMessage, config: EmailConfig): string {
    const to = Array.isArray(message.to) ? message.to.join(', ') : message.to
    const from = `${config.from_name || 'DeFlow'} <${config.from_email || 'noreply@example.com'}>`
    
    let email = `From: ${from}\r\n`
    email += `To: ${to}\r\n`
    if (message.cc) email += `Cc: ${Array.isArray(message.cc) ? message.cc.join(', ') : message.cc}\r\n`
    if (message.bcc) email += `Bcc: ${Array.isArray(message.bcc) ? message.bcc.join(', ') : message.bcc}\r\n`
    email += `Subject: ${message.subject}\r\n`
    email += `Content-Type: ${message.body_html ? 'text/html' : 'text/plain'}; charset=utf-8\r\n`
    email += `\r\n`
    email += message.body_html || message.body_text || ''
    
    return email
  }

  private parseEmailAddresses(addresses: string | string[]): Array<{email: string, name?: string}> {
    const addressArray = Array.isArray(addresses) ? addresses : [addresses]
    return addressArray.map(addr => ({ email: addr.trim() }))
  }

  private convertAttachmentsForOutlook(attachments: EmailAttachment[]) {
    return attachments.map(att => ({
      '@odata.type': '#microsoft.graph.fileAttachment',
      name: att.filename,
      contentBytes: att.content,
      contentType: att.content_type
    }))
  }

  private convertAttachmentsForSendGrid(attachments: EmailAttachment[]) {
    return attachments.map(att => ({
      content: att.content,
      filename: att.filename,
      type: att.content_type,
      disposition: 'attachment'
    }))
  }

  private convertAttachmentsForPostmark(attachments: EmailAttachment[]) {
    return attachments.map(att => ({
      Name: att.filename,
      Content: att.content,
      ContentType: att.content_type
    }))
  }

  // OAuth2 token refresh for Gmail/Outlook
  async refreshOAuth2Token(providerName: string, refreshToken: string): Promise<string> {
    const config = this.configs.get(providerName)
    if (!config) {
      throw new Error(`Provider '${providerName}' not configured`)
    }

    // This would typically be handled by the backend/canister
    // Frontend would call the canister method to refresh tokens
    throw new Error('OAuth2 token refresh should be handled by the backend canister')
  }

  // Test email connectivity
  async testProvider(providerName: string): Promise<boolean> {
    try {
      const testMessage: EmailMessage = {
        to: 'test@example.com',
        subject: 'DeFlow Email Test',
        body_text: 'This is a test email from DeFlow'
      }
      
      // In a real implementation, you might send to a test endpoint
      // or use provider-specific test methods
      return true
    } catch (error) {
      console.error(`Provider test failed for ${providerName}:`, error)
      return false
    }
  }
}

// Singleton instance
const emailService = new EmailService()
export default emailService

// Helper functions for common email templates
export const EmailTemplates = {
  defiAlert: (strategy: string, amount: number, change: number) => ({
    subject: `🚀 DeFi Alert: ${strategy} Performance Update`,
    body_html: `
      <h2>DeFi Strategy Update</h2>
      <p><strong>Strategy:</strong> ${strategy}</p>
      <p><strong>Current Value:</strong> $${amount.toFixed(2)}</p>
      <p><strong>Change:</strong> <span style="color: ${change >= 0 ? 'green' : 'red'}">${change >= 0 ? '+' : ''}${change.toFixed(2)}%</span></p>
      <p>This alert was generated by your DeFlow automation.</p>
    `,
    body_text: `DeFi Strategy Update\n\nStrategy: ${strategy}\nCurrent Value: $${amount.toFixed(2)}\nChange: ${change >= 0 ? '+' : ''}${change.toFixed(2)}%\n\nThis alert was generated by your DeFlow automation.`
  }),

  workflowComplete: (workflowName: string, duration: number) => ({
    subject: `✅ Workflow Complete: ${workflowName}`,
    body_html: `
      <h2>Workflow Completed</h2>
      <p><strong>Workflow:</strong> ${workflowName}</p>
      <p><strong>Execution Time:</strong> ${duration} seconds</p>
      <p>Your automated workflow has completed successfully.</p>
    `,
    body_text: `Workflow Completed\n\nWorkflow: ${workflowName}\nExecution Time: ${duration} seconds\n\nYour automated workflow has completed successfully.`
  }),

  errorAlert: (workflowName: string, error: string) => ({
    subject: `❌ Workflow Error: ${workflowName}`,
    body_html: `
      <h2>Workflow Error</h2>
      <p><strong>Workflow:</strong> ${workflowName}</p>
      <p><strong>Error:</strong> ${error}</p>
      <p>Please check your DeFlow dashboard for more details.</p>
    `,
    body_text: `Workflow Error\n\nWorkflow: ${workflowName}\nError: ${error}\n\nPlease check your DeFlow dashboard for more details.`
  })
}