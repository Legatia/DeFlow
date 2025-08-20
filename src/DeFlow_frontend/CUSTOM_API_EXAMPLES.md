# Custom API Examples for DeFlow Email Integration

This guide provides real-world examples of how to configure various email service providers using DeFlow's Custom API feature.

## Overview

DeFlow's Custom API feature allows you to integrate with any email service that provides an HTTP API. This is perfect for:
- Email services not directly supported
- Regional email providers
- Enterprise email solutions
- Cost-effective email services

## Popular Email Service Configurations

### 1. Resend (Developer-Friendly)

**Configuration:**
```
Provider Name: Resend
API Endpoint: https://api.resend.com/emails
Method: POST
Auth Type: Bearer Token
Bearer Token: re_xxxxxxxxxxxxxxxxx
```

**Headers:**
```
Content-Type: application/json
```

**Body Template:**
```json
{
  "from": "alerts@yourdomain.com",
  "to": ["{{to}}"],
  "subject": "{{subject}}",
  "html": "{{body}}"
}
```

### 2. Mailgun (Powerful & Reliable)

**Configuration:**
```
Provider Name: Mailgun
API Endpoint: https://api.mailgun.net/v3/yourdomain.com/messages
Method: POST
Auth Type: Basic Auth
Username: api
Password: key-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

**Headers:**
```
Content-Type: application/x-www-form-urlencoded
```

**Body Template:**
```
from=DeFlow Alerts <alerts@yourdomain.com>&to={{to}}&subject={{subject}}&html={{body}}
```

### 3. SendinBlue (Brevo)

**Configuration:**
```
Provider Name: SendinBlue
API Endpoint: https://api.sendinblue.com/v3/smtp/email
Method: POST
Auth Type: API Key Header
Header Name: api-key
API Key: xkeysib-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

**Headers:**
```
Content-Type: application/json
```

**Body Template:**
```json
{
  "sender": {
    "name": "DeFlow",
    "email": "alerts@yourdomain.com"
  },
  "to": [
    {"email": "{{to}}"}
  ],
  "subject": "{{subject}}",
  "htmlContent": "{{body}}"
}
```

### 4. Elastic Email (Cost-Effective)

**Configuration:**
```
Provider Name: Elastic Email
API Endpoint: https://api.elasticemail.com/v2/email/send
Method: POST
Auth Type: None (API key in body)
```

**Headers:**
```
Content-Type: application/x-www-form-urlencoded
```

**Body Template:**
```
apikey={{api_key}}&from=alerts@yourdomain.com&to={{to}}&subject={{subject}}&bodyHtml={{body}}
```

**Note:** Add your API key as `{{api_key}}` in the template.

### 5. Mailchimp Transactional (Mandrill)

**Configuration:**
```
Provider Name: Mailchimp Transactional
API Endpoint: https://mandrillapp.com/api/1.0/messages/send.json
Method: POST
Auth Type: None (API key in body)
```

**Headers:**
```
Content-Type: application/json
```

**Body Template:**
```json
{
  "key": "{{api_key}}",
  "message": {
    "from_email": "alerts@yourdomain.com",
    "from_name": "DeFlow",
    "to": [
      {
        "email": "{{to}}"
      }
    ],
    "subject": "{{subject}}",
    "html": "{{body}}"
  }
}
```

### 6. Postmark (Fast Delivery)

**Configuration:**
```
Provider Name: Postmark
API Endpoint: https://api.postmarkapp.com/email
Method: POST
Auth Type: API Key Header
Header Name: X-Postmark-Server-Token
API Key: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
```

**Headers:**
```
Content-Type: application/json
```

**Body Template:**
```json
{
  "From": "alerts@yourdomain.com",
  "To": "{{to}}",
  "Subject": "{{subject}}",
  "HtmlBody": "{{body}}"
}
```

### 7. Amazon SES (AWS Simple Email Service)

**Configuration:**
```
Provider Name: Amazon SES
API Endpoint: https://email.us-east-1.amazonaws.com/
Method: POST
Auth Type: Custom Header
Custom Auth: Authorization: AWS4-HMAC-SHA256 {token}
Token: [Your AWS signature - complex, consider using official SDK]
```

**Note:** AWS SES requires complex signature calculation. Consider using a simpler service or AWS SDK integration.

### 8. SparkPost

**Configuration:**
```
Provider Name: SparkPost
API Endpoint: https://api.sparkpost.com/api/v1/transmissions
Method: POST
Auth Type: API Key Header
Header Name: Authorization
API Key: [Your SparkPost API key]
```

**Headers:**
```
Content-Type: application/json
```

**Body Template:**
```json
{
  "content": {
    "from": "alerts@yourdomain.com",
    "subject": "{{subject}}",
    "html": "{{body}}"
  },
  "recipients": [
    {"address": "{{to}}"}
  ]
}
```

## Enterprise Solutions

### Microsoft 365 SMTP (Alternative to Graph API)

**Note:** While we recommend using Microsoft Graph API directly, you can use SMTP through an HTTP-to-SMTP bridge:

**Configuration:**
```
Provider Name: O365 SMTP Bridge
API Endpoint: https://your-smtp-bridge.com/send
Method: POST
Auth Type: None (credentials in body)
```

**Body Template:**
```json
{
  "smtp_host": "smtp.office365.com",
  "smtp_port": 587,
  "username": "your-email@company.com",
  "password": "{{api_key}}",
  "from": "alerts@company.com",
  "to": "{{to}}",
  "subject": "{{subject}}",
  "html": "{{body}}"
}
```

### Custom Enterprise Email Gateway

**Configuration:**
```
Provider Name: Enterprise Gateway
API Endpoint: https://email-gateway.company.com/api/send
Method: POST
Auth Type: Bearer Token
Bearer Token: [Your enterprise token]
```

**Headers:**
```
Content-Type: application/json
X-Company-ID: your-company-id
```

**Body Template:**
```json
{
  "recipient": "{{to}}",
  "subject": "{{subject}}",
  "content": "{{body}}",
  "priority": "normal",
  "department": "automation"
}
```

## Regional Providers

### Email.cz (Czech Republic)

**Configuration:**
```
Provider Name: Email.cz
API Endpoint: https://api.email.cz/v2/send
Method: POST
Auth Type: API Key Header
Header Name: X-API-Key
API Key: [Your Email.cz API key]
```

### Mail.ru (Russia)

**Configuration:**
```
Provider Name: Mail.ru API
API Endpoint: https://api.mail.ru/platform/api
Method: POST
Auth Type: Custom Header
Custom Auth: sig: {token}
```

## Testing Your Configuration

### Test Steps:
1. **Save Configuration**: Add your provider in DeFlow Settings
2. **API Test**: Click "Test" → Cancel email prompt → Tests API connectivity only
3. **Email Test**: Click "Test" → Enter your email → Sends actual test email
4. **Workflow Test**: Create a simple workflow that sends an email using your provider

### Common Issues:

**Authentication Errors:**
- Double-check API key format
- Verify header names (case-sensitive)
- Check if API key has send permissions

**Template Errors:**
- Ensure JSON is valid (use online JSON validator)
- Check placeholder names match exactly: `{{to}}`, `{{subject}}`, `{{body}}`
- Verify required fields for your provider

**CORS Errors:**
- Some providers block browser requests (works in production canisters)
- Test with different endpoint if available

## Advanced Features

### Multiple Recipients
For providers supporting multiple recipients, modify the template:

```json
{
  "to": ["{{to}}", "backup@yourdomain.com"],
  "subject": "{{subject}}",
  "html": "{{body}}"
}
```

### Custom Fields
Add your own placeholders:

```json
{
  "to": "{{to}}",
  "subject": "{{subject}}",
  "html": "{{body}}",
  "tags": ["deflow", "automation"],
  "metadata": {
    "source": "deflow",
    "user_id": "{{user_id}}"
  }
}
```

Then use in workflows:
```typescript
await emailService.sendEmail('my-provider', {
  to: 'user@example.com',
  subject: 'Alert',
  body_html: 'Your custom email content',
  // Add custom fields
  user_id: 'user123'
})
```

### Webhook Integration
Some providers support webhooks for delivery status:

```json
{
  "to": "{{to}}",
  "subject": "{{subject}}",
  "html": "{{body}}",
  "webhook_url": "https://your-app.com/email-webhook"
}
```

## Security Best Practices

1. **API Key Security**:
   - Use environment-specific API keys
   - Rotate keys regularly
   - Never commit keys to version control

2. **Rate Limiting**:
   - Check provider rate limits
   - Implement exponential backoff
   - Use multiple providers for high volume

3. **Monitoring**:
   - Log email sending results
   - Monitor bounce rates
   - Set up alerts for failures

4. **Compliance**:
   - Follow CAN-SPAM, GDPR rules
   - Include unsubscribe links
   - Maintain email reputation

## Cost Optimization

### Free Tiers (Monthly):
- **Resend**: 3,000 emails
- **Elastic Email**: 100 emails/day
- **Mailgun**: 5,000 emails (first 3 months)
- **SendinBlue**: 300 emails/day

### Pay-as-you-go:
- **Amazon SES**: $0.10 per 1,000 emails
- **Postmark**: $1.25 per 1,000 emails
- **SparkPost**: $0.20 per 1,000 emails

### Enterprise:
- **Mailgun**: $35/month for 50K emails
- **SendGrid**: Custom pricing
- **Mailchimp**: $10-$300/month

## Integration with DeFlow Workflows

### Portfolio Alert Example:
```typescript
// In your DeFi workflow
if (portfolioChange > 5) {
  await emailService.sendEmail('resend-provider', {
    to: userEmail,
    subject: '🚀 Portfolio Alert: +5% Gain!',
    body_html: `Your ${strategy} strategy gained ${portfolioChange}%`,
  })
}
```

### Error Notification:
```typescript
// In error handler
try {
  await executeWorkflow()
} catch (error) {
  await emailService.sendEmail('reliable-provider', {
    to: 'admin@yourapp.com',
    subject: '❌ Workflow Failed',
    body_html: `Error: ${error.message}`,
  })
}
```

This custom API feature gives you unlimited flexibility to integrate with any email service, making DeFlow compatible with your preferred providers and business requirements.