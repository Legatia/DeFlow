# External Authentication Guide

This guide covers how to set up external service authentication for DeFlow workflow automation. DeFlow supports integration with email providers, social media platforms, and messaging services to enable comprehensive automation workflows.

## Overview

DeFlow uses a hybrid authentication approach optimized for ICP canister constraints:
- **HTTP API Integration**: For providers with REST APIs (Gmail, Outlook, Twitter/X) - managed by DeFlow
- **User-Managed API Keys**: For messaging platforms (Telegram, Discord) and email service providers
- **Bridge Services**: For traditional protocols (SMTP) that require TCP connections

## ICP Canister Limitations

**Important**: ICP canisters can only make HTTP(S) outcalls, not direct TCP connections. This affects:
- ❌ **Direct SMTP**: Not supported (requires TCP on ports 25/465/587)
- ✅ **Email APIs**: Gmail API, Outlook API, SendGrid work perfectly
- ✅ **Messaging APIs**: Telegram, Discord use HTTP APIs
- ✅ **Social APIs**: Twitter/X uses HTTP REST API

## Supported Services

### 📧 Email Providers

#### Gmail (HTTP API - Managed) ✅ ICP Compatible
**Setup**: OAuth2 through DeFlow interface
**API**: Gmail API v1 (HTTP REST)
**Capabilities**:
- Send automated emails
- Read inbox for triggers
- Manage labels and filters
- Access Google Workspace features

**How to Connect**:
1. Go to Settings → External Integrations
2. Click "Connect Gmail"
3. Authorize DeFlow in Google OAuth flow
4. API tokens managed automatically

**Technical**: Uses `https://gmail.googleapis.com/gmail/v1/` endpoints

#### Outlook/Microsoft 365 (HTTP API - Managed) ✅ ICP Compatible
**Setup**: OAuth2 through DeFlow interface
**API**: Microsoft Graph API (HTTP REST)
**Capabilities**:
- Send emails via Microsoft Graph API
- Read mailbox and folders
- Access calendar events
- Integration with Microsoft 365 suite

**How to Connect**:
1. Go to Settings → External Integrations  
2. Click "Connect Outlook"
3. Sign in with Microsoft account
4. Grant permissions for email access

**Technical**: Uses `https://graph.microsoft.com/v1.0/` endpoints

#### Email Service Providers (HTTP API - User Managed) ✅ ICP Compatible
**Recommended for ICP**: Use HTTP-based email services instead of SMTP
**Supported Services**:
- **SendGrid**: Professional email delivery
- **Mailgun**: Developer-focused email API  
- **Postmark**: Transactional email service
- **Amazon SES**: AWS email service

**SendGrid Example Setup**:
```json
{
  "provider": "sendgrid",
  "api_key": "SG.xxxxxxxxxxxxxxxx",
  "from_email": "noreply@yourapp.com",
  "from_name": "DeFlow Alerts"
}
```

#### SMTP Bridge Service (For Legacy Email) ⚠️ Requires Bridge
**Use Case**: When you must use traditional SMTP
**Solution**: HTTP-to-SMTP bridge service

**Bridge Service Requirements**:
```typescript
// Bridge API endpoint
POST https://your-bridge.com/api/send-email
{
  "smtp_config": {
    "host": "smtp.gmail.com",
    "port": 587,
    "username": "alerts@domain.com",
    "password": "app-password"
  },
  "email": {
    "to": "user@example.com",
    "subject": "Alert",
    "body": "Your DeFi strategy update"
  }
}
```

**Security Note**: Bridge service should validate requests and encrypt SMTP credentials.

### 🐦 Social Media

#### Twitter/X (OAuth2 - Managed)
**Setup**: Automatic through DeFlow interface
**Capabilities**:
- Post tweets and threads
- Send direct messages
- Monitor mentions and hashtags
- Access analytics data

**How to Connect**:
1. Go to Settings → External Integrations
2. Click "Connect Twitter/X"
3. Authorize DeFlow app on Twitter
4. Select permission level (read/write)

**API Limits**: Subject to Twitter API rate limits
- Basic tier: 1,500 posts per month
- Pro tier: 50,000 posts per month

### 💬 Messaging Platforms

#### Telegram (Bot API - User Managed)
**Setup**: Create your own Telegram bot
**Capabilities**:
- Send messages to chats/channels
- Create polls and keyboards
- File uploads and media sharing
- Real-time message webhooks

**Step-by-Step Setup**:

1. **Create Telegram Bot**:
   - Message @BotFather on Telegram
   - Send `/newbot` command
   - Choose bot name and username
   - Save the bot token (format: `123456789:ABCdefGHIjklMNOpqrsTUVwxyz`)

2. **Get Chat/Channel IDs**:
   - Add bot to your chat/channel
   - For channels: Make bot an admin
   - Send a message, then visit: `https://api.telegram.org/bot<TOKEN>/getUpdates`
   - Look for `chat.id` in the response

3. **Configure in DeFlow**:
   ```json
   {
     "provider": "telegram",
     "botToken": "123456789:ABCdefGHIjklMNOpqrsTUVwxyz",
     "defaultChatId": "-1001234567890"
   }
   ```

4. **Test Connection**:
   - Use DeFlow test message feature
   - Verify bot can send to your chat/channel

**Security**: Keep bot token secure, never share publicly

#### Discord (Bot API - User Managed)
**Setup**: Create Discord application and bot
**Capabilities**:
- Send messages to channels
- Create/manage channels and roles
- Moderate servers
- Embed rich content and files

**Step-by-Step Setup**:

1. **Create Discord Application**:
   - Go to https://discord.com/developers/applications
   - Click "New Application"
   - Name your application (e.g., "DeFlow Bot")

2. **Create Bot**:
   - Go to "Bot" section
   - Click "Add Bot"
   - Copy bot token (format: `MTAx...`)
   - Enable necessary permissions

3. **Add Bot to Server**:
   - Go to "OAuth2" → "URL Generator"
   - Select "bot" scope
   - Choose permissions:
     - Send Messages
     - Manage Channels (if needed)
     - Read Message History
   - Copy generated URL and open in browser
   - Select server and authorize

4. **Get Channel IDs**:
   - Enable Developer Mode in Discord
   - Right-click channel → "Copy ID"

5. **Configure in DeFlow**:
   ```json
   {
     "provider": "discord",
     "botToken": "MTAx...your-bot-token",
     "defaultChannelId": "123456789012345678"
   }
   ```

**Permission Requirements**:
- `Send Messages`: Basic messaging
- `Embed Links`: Rich content
- `Attach Files`: File uploads
- `Manage Channels`: Create/modify channels

## Configuration Examples

### Email Workflow Trigger
```typescript
// Gmail OAuth2 (Managed)
{
  trigger: "gmail_received",
  filter: {
    from: "important@client.com",
    subject_contains: "urgent"
  },
  action: "send_telegram_alert"
}

// SMTP Email Send (User Managed)
{
  action: "send_email",
  provider: "smtp",
  config: {
    server: "smtp.gmail.com",
    port: 587,
    username: "alerts@mycompany.com",
    password: "app-specific-password"
  }
}
```

### Social Media Automation
```typescript
// Twitter OAuth2 (Managed)
{
  trigger: "defi_profit_threshold",
  action: "post_tweet",
  content: "🚀 Portfolio just hit ${{amount}} profit! #DeFi #DeFlow"
}
```

### Messaging Notifications
```typescript
// Telegram Bot (User Managed)
{
  trigger: "workflow_completion",
  action: "send_telegram",
  config: {
    botToken: "{{USER_TELEGRAM_TOKEN}}",
    chatId: "{{USER_CHAT_ID}}",
    message: "✅ Workflow '{{workflow_name}}' completed successfully"
  }
}

// Discord Bot (User Managed)
{
  trigger: "error_occurred",
  action: "send_discord",
  config: {
    botToken: "{{USER_DISCORD_TOKEN}}",
    channelId: "{{USER_CHANNEL_ID}}",
    embed: {
      title: "❌ Workflow Error",
      description: "{{error_message}}",
      color: "#ff0000"
    }
  }
}
```

## Security Best Practices

### OAuth2 Services (Managed by DeFlow)
- ✅ Tokens encrypted and stored securely
- ✅ Automatic token refresh
- ✅ Minimal permissions requested
- ✅ Revocation support

### User-Managed Services
- 🔒 **Use App-Specific Passwords**: Never use main account passwords
- 🔒 **Rotate Tokens Regularly**: Change bot tokens periodically
- 🔒 **Limit Permissions**: Grant minimum required permissions
- 🔒 **Monitor Usage**: Check for unauthorized API calls

### General Security
- Store sensitive credentials in DeFlow's encrypted settings
- Use environment variables for development
- Never commit tokens to version control
- Monitor API usage and set up alerts

## Rate Limits and Quotas

| Service | Limit | Notes |
|---------|-------|-------|
| Gmail API | 1B quota units/day | ~250 requests/user/day |
| Outlook API | 10,000 requests/10min | Per user |
| Twitter API | 1,500 posts/month (Basic) | Paid tiers available |
| Telegram Bot | 30 messages/second | Per bot |
| Discord Bot | 5 requests/second | Per bot, per endpoint |

## Troubleshooting

### Common Issues

**Gmail OAuth2**:
- ❌ "Access blocked" → Enable less secure apps or use OAuth2
- ❌ "Invalid credentials" → Regenerate OAuth tokens

**Twitter API**:
- ❌ "Rate limit exceeded" → Wait for reset or upgrade plan
- ❌ "App not approved" → Apply for elevated access

**Telegram Bot**:
- ❌ "Unauthorized" → Check bot token format
- ❌ "Chat not found" → Verify chat ID and bot membership

**Discord Bot**:
- ❌ "Missing permissions" → Check bot permissions in server
- ❌ "Unknown channel" → Verify channel ID and bot access

### Debug Mode
Enable debug logging in DeFlow settings to see detailed API request/response information.

## Support

For integration issues:
1. Check service status pages
2. Verify API credentials in provider console
3. Test with minimal example in DeFlow
4. Contact DeFlow support with error logs

---

*Last updated: August 2025*
*For the latest API documentation, refer to each provider's official docs*