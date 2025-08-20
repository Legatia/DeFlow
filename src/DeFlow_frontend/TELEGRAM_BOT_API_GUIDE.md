# Telegram Bot API Integration Guide for DeFlow

This comprehensive guide covers everything you need to know about integrating Telegram Bot API with DeFlow for automated messaging, notifications, and community engagement.

## Table of Contents
1. [Overview](#overview)
2. [Telegram Bot Basics](#telegram-bot-basics)
3. [Developer Setup](#developer-setup)
4. [Bot Creation & Configuration](#bot-creation--configuration)
5. [Authentication & Security](#authentication--security)
6. [DeFlow Integration](#deflow-integration)
7. [User Setup Guide](#user-setup-guide)
8. [Use Cases & Examples](#use-cases--examples)
9. [Rate Limits & Best Practices](#rate-limits--best-practices)
10. [Advanced Features](#advanced-features)
11. [Troubleshooting](#troubleshooting)
12. [Security Considerations](#security-considerations)

## Overview

Telegram Bot API integration in DeFlow enables automated messaging workflows for:
- **Portfolio alerts**: Send DeFi performance notifications to users or groups
- **Market updates**: Share real-time crypto market insights
- **Community management**: Automated responses and member engagement
- **Trading signals**: Send buy/sell alerts to trading groups
- **Personal notifications**: Private alerts for portfolio changes
- **Channel broadcasting**: Automated content posting to Telegram channels

### Why Telegram Integration Matters
- **High engagement**: 95% message open rates
- **Real-time delivery**: Instant notifications to mobile devices
- **Group messaging**: Reach entire communities simultaneously
- **Rich media support**: Send charts, images, and documents
- **Bot interactions**: Create interactive experiences with inline keyboards
- **Global reach**: 800M+ active users worldwide

## Telegram Bot Basics

### Bot Types
**1. Regular Bots**
- Interact in groups and private chats
- Respond to commands and messages
- Can be added to groups by users
- Perfect for community engagement

**2. Inline Bots**
- Work in any chat via @botname query
- Provide inline results and suggestions
- Advanced feature for content discovery

### API Capabilities
**Messaging**:
- Send text messages with formatting
- Send photos, videos, documents
- Send location and contact information
- Forward and reply to messages

**Interactive Elements**:
- Inline keyboards with buttons
- Custom keyboards for easy commands
- Polls and quizzes
- Game integration

**Group Management**:
- Moderate chat members
- Pin/unpin messages
- Set chat descriptions and photos
- Manage permissions

## Developer Setup

### Step 1: Create Telegram Bot

1. **Start BotFather Conversation**
   - Open Telegram app
   - Search for "@BotFather" (official bot creation bot)
   - Start conversation with `/start`

2. **Create New Bot**
   ```
   /newbot
   ```
   - Choose bot name (display name): "DeFlow Portfolio Bot"
   - Choose username (must end with 'bot'): "DeFlowPortfolioBot"
   - Must be unique across all Telegram

3. **Receive Bot Token**
   ```
   Congratulations! Your bot is ready.
   Token: 123456789:ABCdefGHIjklMNOpqrsTUVwxyz
   Keep your token secure and store it safely.
   ```

4. **Configure Bot Settings**
   ```
   /mybots → Select your bot → Bot Settings
   ```
   - **Description**: "Automated DeFi portfolio alerts and market insights"
   - **About**: "Get real-time notifications about your DeFi investments"
   - **Profile Photo**: Upload bot avatar
   - **Commands**: Set up command menu

### Step 2: Bot Configuration

**Set Bot Commands**:
```
/setcommands

Select your bot → Send this list:

start - Initialize portfolio tracking
help - Show available commands
portfolio - View current portfolio status
alerts - Configure alert settings
subscribe - Subscribe to market updates
unsubscribe - Stop market updates
support - Get help and support
```

**Bot Privacy Settings**:
```
/setprivacy

Select your bot → Choose privacy level:
- Disable: Bot receives all group messages
- Enable: Bot only receives commands and replies
```

**Group Settings**:
```
/setjoingroups

Select your bot:
- Enable: Bot can be added to groups
- Disable: Bot only works in private chats
```

### Step 3: Advanced Configuration

**Inline Mode** (Optional):
```
/setinline

Select your bot → Enable inline mode
Placeholder text: "Search DeFi insights..."
```

**Domain Whitelisting** (for web apps):
```
/setdomain

Select your bot → Add domains:
- localhost:4943
- your-canister-id.ic0.app
```

## Bot Creation & Configuration

### Basic Bot Information

**Essential Settings**:
```json
{
  "name": "DeFlow Portfolio Bot",
  "username": "DeFlowPortfolioBot", 
  "description": "Automated DeFi portfolio management and alerts",
  "about": "Stay updated with your DeFi investments through real-time notifications",
  "profile_photo": "bot-avatar.jpg"
}
```

**Commands Setup**:
```json
{
  "commands": [
    {"command": "start", "description": "Initialize portfolio tracking"},
    {"command": "portfolio", "description": "View current portfolio status"},
    {"command": "alerts", "description": "Configure alert settings"},
    {"command": "performance", "description": "Show performance metrics"},
    {"command": "strategies", "description": "List active strategies"},
    {"command": "market", "description": "Get market insights"},
    {"command": "help", "description": "Show help information"},
    {"command": "settings", "description": "Manage bot settings"}
  ]
}
```

### Permission Configuration

**Required Permissions**:
- ✅ Send messages
- ✅ Send photos (for charts)
- ✅ Send documents (for reports)
- ✅ Edit messages (for live updates)
- ✅ Delete messages (for cleanup)

**Optional Permissions**:
- Pin messages (for important alerts)
- Manage chat (for group administration)
- Add new members (for referrals)

## Authentication & Security

### Bot Token Security

**Token Storage**:
```typescript
// NEVER hardcode tokens in source code
const BOT_TOKEN = process.env.TELEGRAM_BOT_TOKEN // ✅ Good
const BOT_TOKEN = "123456:ABC-DEF..." // ❌ Never do this
```

**Environment Configuration**:
```bash
# .env file (never commit this)
TELEGRAM_BOT_TOKEN=your_bot_token_here
TELEGRAM_WEBHOOK_SECRET=random_secret_string
```

### Webhook Security

**Webhook URL**:
```
https://api.telegram.org/bot{BOT_TOKEN}/setWebhook
```

**Request Body**:
```json
{
  "url": "https://your-canister-id.ic0.app/telegram/webhook",
  "secret_token": "your_secret_token",
  "allowed_updates": ["message", "callback_query"],
  "drop_pending_updates": true
}
```

**Webhook Verification**:
```typescript
function verifyTelegramWebhook(request: any): boolean {
  const secret = process.env.TELEGRAM_WEBHOOK_SECRET
  const hash = crypto.createHmac('sha256', secret)
    .update(JSON.stringify(request.body))
    .digest('hex')
  
  const expectedHash = `sha256=${hash}`
  return expectedHash === request.headers['x-telegram-bot-api-secret-token']
}
```

## DeFlow Integration

### Custom API Provider Setup

**Telegram Provider Configuration**:
```json
{
  "name": "Telegram Bot",
  "baseUrl": "https://api.telegram.org/bot{{token}}/sendMessage",
  "method": "POST",
  "authType": "custom",
  "headers": {
    "Content-Type": "application/json"
  },
  "bodyTemplate": {
    "chat_id": "{{chat_id}}",
    "text": "{{message}}",
    "parse_mode": "Markdown"
  },
  "variables": {
    "token": "YOUR_BOT_TOKEN_HERE",
    "chat_id": "YOUR_CHAT_ID_HERE"
  }
}
```

### Advanced Provider with Rich Formatting

**Rich Message Provider**:
```json
{
  "name": "Telegram Rich Message",
  "baseUrl": "https://api.telegram.org/bot{{token}}/sendMessage",
  "method": "POST",
  "authType": "custom",
  "headers": {
    "Content-Type": "application/json"
  },
  "bodyTemplate": {
    "chat_id": "{{chat_id}}",
    "text": "{{message}}",
    "parse_mode": "MarkdownV2",
    "reply_markup": {
      "inline_keyboard": [[
        {
          "text": "📊 View Portfolio",
          "url": "{{portfolio_url}}"
        },
        {
          "text": "⚙️ Settings",
          "callback_data": "settings"
        }
      ]]
    }
  }
}
```

### Photo/Chart Sending Provider

**Image Provider**:
```json
{
  "name": "Telegram Photo",
  "baseUrl": "https://api.telegram.org/bot{{token}}/sendPhoto",
  "method": "POST",
  "authType": "custom",
  "headers": {
    "Content-Type": "application/json"
  },
  "bodyTemplate": {
    "chat_id": "{{chat_id}}",
    "photo": "{{image_url}}",
    "caption": "{{caption}}",
    "parse_mode": "Markdown"
  }
}
```

## User Setup Guide

### Step 1: Get Your Chat ID

**Method 1: Direct Message Bot**
1. Start conversation with your bot
2. Send any message (e.g., "/start")
3. Bot will reply with your Chat ID
4. Copy the Chat ID for DeFlow configuration

**Method 2: Use @userinfobot**
1. Start conversation with @userinfobot
2. Send any message
3. It will reply with your user information including Chat ID

**Method 3: Add Bot to Group**
1. Add your bot to a Telegram group
2. Send a message mentioning the bot
3. Bot will reply with the group Chat ID

### Step 2: Configure DeFlow

**In DeFlow Settings**:
1. Go to **Settings** → **External Integrations**
2. Find **Custom API Providers** section
3. Click **Add Provider**
4. Select **Telegram Bot** template
5. Enter your Bot Token and Chat ID
6. Test the connection

**Configuration Form**:
```
Provider Name: My Telegram Alerts
Bot Token: 123456789:ABCdefGHIjklMNOpqrsTUVwxyz
Chat ID: 123456789 (for private messages)
         -100123456789 (for groups, starts with -100)
```

### Step 3: Test Integration

**Send Test Message**:
1. Click **Test Connection** button
2. Check your Telegram for test message
3. Verify formatting and delivery
4. Save configuration if successful

### Step 4: Set Up Workflows

**Create Notification Workflow**:
```typescript
{
  "trigger": "portfolio_change",
  "condition": "change > 5%",
  "action": {
    "type": "telegram_message",
    "provider": "My Telegram Alerts",
    "message": "🚀 Portfolio Alert!\n\nGain: +{{change}}%\nValue: ${{value}}\nTop performer: {{best_strategy}}"
  }
}
```

## Use Cases & Examples

### 1. Portfolio Performance Alerts

**Daily Summary**:
```markdown
📊 **Daily Portfolio Summary**

💰 **Total Value**: $12,450.32
📈 **24h Change**: +5.67% (+$707.23)
🏆 **Best Performer**: Uniswap V3 Strategy (+12.3%)
📉 **Underperformer**: Compound Lending (-2.1%)

**Top 3 Strategies:**
1. 🦄 Uniswap V3: $4,200 (+12.3%)
2. ⚡ Aave Lending: $3,800 (+3.2%)
3. 🔄 Curve Farming: $2,950 (+1.8%)

🤖 *Automated by DeFlow*
[View Details](https://deflow.app/dashboard)
```

**Threshold Alerts**:
```markdown
🚨 **ALERT: Significant Portfolio Movement**

Your portfolio just **gained 15.2%** in the last hour!

💰 **Current Value**: $15,847.56
📊 **Change**: +$2,095.34
⚡ **Trigger**: ETH price surge + automated rebalancing

**Action Taken:**
✅ Rebalanced to maintain target allocation
✅ Harvested rewards from 3 strategies
✅ Compound profits automatically

*Time to celebrate! 🎉*
```

### 2. Market Insights

**Market Movement Alert**:
```markdown
📈 **Crypto Market Alert**

🔴 **BTC**: $45,250 (+8.2% in 1h)
🟢 **ETH**: $3,150 (+12.5% in 1h)
🔵 **DeFi Index**: +15.3%

**Your Portfolio Impact:**
• Estimated gain: +$1,200
• Strategy adjustments triggered
• Rebalancing in progress

**Market Sentiment**: 🚀 Extremely Bullish
**Fear & Greed**: 82 (Extreme Greed)

[View Full Analysis](https://deflow.app/market)
```

### 3. Trading Signals

**Buy/Sell Signal**:
```markdown
⚡ **TRADING SIGNAL**

**Signal**: BUY ETH
**Strategy**: DCA + Momentum
**Confidence**: 87%

**Analysis:**
• RSI oversold (28)
• Support level confirmed
• Volume increasing
• On-chain metrics bullish

**Suggested Action:**
💰 Amount: $500
⏰ Timing: Next 2 hours
🎯 Target: $3,300 (+4.8%)
🛡️ Stop Loss: $2,950 (-6.3%)

*Auto-executed via DeFlow strategy*
```

### 4. DeFi Protocol Updates

**Yield Farming Alert**:
```markdown
🌾 **Yield Farming Opportunity**

**New Pool Alert**: USDC-ETH on Uniswap V3
📊 **APY**: 24.6% (↑15% from yesterday)
💰 **TVL**: $12.5M
⚡ **Rewards**: UNI tokens + fees

**Your Current Allocation:**
• Available: $2,500 USDC
• Auto-compound: Enabled
• Strategy match: 95%

**Action Required:**
Should I deploy capital to this pool?
👍 Yes, deploy $2,500
👎 No, keep current allocation

*React to this message to confirm*
```

### 5. Risk Management

**Risk Alert**:
```markdown
⚠️ **RISK MANAGEMENT ALERT**

**Impermanent Loss Warning**
Pool: USDC-WBTC (Curve)
Current IL: -3.2%
Threshold: -5.0%

**Portfolio Status:**
🔴 High Risk: 2 positions
🟡 Medium Risk: 3 positions  
🟢 Low Risk: 8 positions

**Suggested Actions:**
1. Reduce exposure to volatile pairs
2. Increase stablecoin allocation
3. Enable stop-loss on WBTC position

[Review Risk Settings](https://deflow.app/risk)
```

### 6. Community Engagement

**Group Broadcasting**:
```markdown
🎯 **DeFlow Community Update**

**This Week's Performance:**
• Total Volume: $2.4M processed
• Average User Gain: +8.3%
• Strategies Deployed: 147
• Gas Saved: $23,450

**Top Performing Strategies:**
1. 🥇 ETH Momentum Trading: +23.4%
2. 🥈 Stablecoin Arbitrage: +12.1%
3. 🥉 DeFi Index Fund: +9.8%

**New Features:**
✅ Telegram notifications (you're here!)
✅ Advanced stop-loss options
✅ Cross-chain bridge automation

Join our alpha testing: @DeFlowAlpha
```

## Rate Limits & Best Practices

### Telegram API Rate Limits

**Message Limits**:
- **Private chats**: 30 messages per second
- **Groups**: 20 messages per minute per group
- **Channels**: 30 messages per second
- **Broadcast**: 1 message per second

**Other Limits**:
- **File uploads**: 50MB per file
- **Photo resolution**: 10MB maximum
- **Video duration**: No specific limit
- **Audio duration**: No specific limit

### Best Practices

**Message Frequency**:
- ✅ **Personalized alerts**: Based on user-defined thresholds
- ✅ **Valuable content**: Market insights, performance data
- ✅ **Interactive elements**: Buttons for quick actions
- ❌ **Spam**: Avoid too frequent notifications
- ❌ **Repetitive content**: Vary message formats
- ❌ **Generic messages**: Personalize based on user portfolio

**Content Guidelines**:
```typescript
// Good: Personalized and actionable
const goodMessage = `🚀 Your ETH strategy gained 8.2%!
Current value: $${portfolioValue}
Action: Auto-compounding enabled
Next rebalance: In 4 hours`

// Bad: Generic and spammy
const badMessage = `Buy crypto now! Prices going up!
Don't miss out! Click here now!`
```

**Message Formatting**:
- Use **bold** for important numbers
- Use *italics* for context
- Use `code` for technical details
- Use emojis sparingly but effectively
- Structure with line breaks and sections
- Include relevant action buttons

### Rate Limiting Implementation

```typescript
class TelegramRateLimiter {
  private messageQueue: Array<{
    chatId: string
    message: string
    timestamp: number
  }> = []
  
  private readonly MESSAGES_PER_SECOND = 30
  private readonly GROUP_MESSAGES_PER_MINUTE = 20
  
  async sendMessage(chatId: string, message: string) {
    if (this.isGroup(chatId)) {
      return this.sendGroupMessage(chatId, message)
    }
    return this.sendPrivateMessage(chatId, message)
  }
  
  private async sendPrivateMessage(chatId: string, message: string) {
    // Check if we can send immediately
    const now = Date.now()
    const recentMessages = this.messageQueue.filter(
      msg => now - msg.timestamp < 1000 && msg.chatId === chatId
    )
    
    if (recentMessages.length < this.MESSAGES_PER_SECOND) {
      return this.sendImmediate(chatId, message)
    }
    
    // Queue for later sending
    return this.queueMessage(chatId, message)
  }
  
  private isGroup(chatId: string): boolean {
    return chatId.startsWith('-100') // Telegram group chat IDs start with -100
  }
}
```

## Advanced Features

### 1. Inline Keyboards

**Interactive Portfolio Dashboard**:
```typescript
const inlineKeyboard = {
  inline_keyboard: [
    [
      { text: "📊 Portfolio", callback_data: "portfolio" },
      { text: "📈 Performance", callback_data: "performance" }
    ],
    [
      { text: "⚙️ Strategies", callback_data: "strategies" },
      { text: "🔔 Alerts", callback_data: "alerts" }
    ],
    [
      { text: "💰 Deposit", url: "https://deflow.app/deposit" },
      { text: "📤 Withdraw", url: "https://deflow.app/withdraw" }
    ]
  ]
}

const message = {
  chat_id: chatId,
  text: "🎯 **DeFlow Dashboard**\n\nChoose an option below:",
  parse_mode: "Markdown",
  reply_markup: inlineKeyboard
}
```

### 2. Custom Keyboards

**Quick Commands Keyboard**:
```typescript
const customKeyboard = {
  keyboard: [
    ["/portfolio", "/performance"],
    ["/alerts", "/strategies"],
    ["/help", "/settings"]
  ],
  resize_keyboard: true,
  one_time_keyboard: false,
  selective: true
}
```

### 3. File and Media Handling

**Send Portfolio Chart**:
```typescript
// Generate chart and send as photo
const chartData = generatePortfolioChart(portfolio)
const chartUrl = uploadChart(chartData)

const photoMessage = {
  chat_id: chatId,
  photo: chartUrl,
  caption: `📊 **Portfolio Performance Chart**
  
📈 **7-day gain**: +${weeklyGain}%
💰 **Current value**: $${currentValue}
🎯 **Target allocation**: Maintained

*Updated every 15 minutes*`,
  parse_mode: "Markdown"
}
```

**Send PDF Report**:
```typescript
const reportData = generateMonthlyReport(portfolio)
const reportUrl = uploadPDF(reportData)

const documentMessage = {
  chat_id: chatId,
  document: reportUrl,
  caption: "📋 Your monthly DeFlow report is ready!",
  reply_markup: {
    inline_keyboard: [[
      { text: "📊 View Online", url: "https://deflow.app/reports" }
    ]]
  }
}
```

### 4. Polling and Surveys

**Strategy Performance Poll**:
```typescript
const pollMessage = {
  chat_id: chatId,
  question: "Which DeFi strategy performed best this week?",
  options: [
    "🦄 Uniswap V3 Farming",
    "⚡ Aave Lending", 
    "🔄 Curve Strategies",
    "🏦 Compound Auto-compound"
  ],
  is_anonymous: false,
  allows_multiple_answers: false,
  type: "regular"
}
```

### 5. Group Management

**Welcome New Members**:
```typescript
// Detect new member joining group
if (update.message?.new_chat_members) {
  const welcomeMessage = `👋 Welcome to DeFlow Community!

🎯 **Get Started:**
• Send /start to connect your portfolio
• Use /help to see all commands  
• Join discussions and share insights

📊 **Community Stats:**
• Members: ${memberCount}
• Active portfolios: ${activePortfolios}
• Total volume: $${totalVolume}

Happy trading! 🚀`

  await sendMessage(chatId, welcomeMessage)
}
```

## Troubleshooting

### Common Issues

**"Bot was blocked by the user"**
```json
{
  "ok": false,
  "error_code": 403,
  "description": "Forbidden: bot was blocked by the user"
}
```
- **Cause**: User blocked the bot
- **Fix**: Remove user from notification list, provide opt-out instructions

**"Chat not found"**
```json
{
  "ok": false,
  "error_code": 400,
  "description": "Bad Request: chat not found"
}
```
- **Cause**: Invalid Chat ID or chat doesn't exist
- **Fix**: Verify Chat ID format, check if user started conversation with bot

**"Message is too long"**
```json
{
  "ok": false,
  "error_code": 400,
  "description": "Bad Request: message is too long"
}
```
- **Cause**: Message exceeds 4096 characters
- **Fix**: Split long messages, use file attachments for large content

**"Too Many Requests"**
```json
{
  "ok": false,
  "error_code": 429,
  "description": "Too Many Requests: retry after 30"
}
```
- **Cause**: Hit rate limits
- **Fix**: Implement exponential backoff, reduce message frequency

### Debugging Tips

**Enable Debug Logging**:
```typescript
const telegramService = {
  async sendMessage(chatId: string, text: string) {
    console.log(`Sending to ${chatId}: ${text.substring(0, 100)}...`)
    
    try {
      const response = await fetch(this.apiUrl, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ chat_id: chatId, text })
      })
      
      const result = await response.json()
      console.log('Telegram API response:', result)
      
      if (!result.ok) {
        console.error('Telegram API error:', result.description)
      }
      
      return result
    } catch (error) {
      console.error('Network error:', error)
      throw error
    }
  }
}
```

**Test Bot Connectivity**:
```typescript
// Test bot token validity
const testResponse = await fetch(
  `https://api.telegram.org/bot${BOT_TOKEN}/getMe`
)
const botInfo = await testResponse.json()
console.log('Bot info:', botInfo)
```

**Validate Chat IDs**:
```typescript
function validateChatId(chatId: string): boolean {
  // Private chats: positive integers
  if (/^\d+$/.test(chatId)) return true
  
  // Groups: negative integers starting with -100
  if (/^-100\d{10,}$/.test(chatId)) return true
  
  // Channels: negative integers or @username
  if (/^-\d+$/.test(chatId) || /^@\w+$/.test(chatId)) return true
  
  return false
}
```

## Security Considerations

### Bot Token Protection

**Secure Storage**:
```typescript
// ✅ Good: Environment variables
const BOT_TOKEN = process.env.TELEGRAM_BOT_TOKEN

// ✅ Good: Encrypted configuration
const BOT_TOKEN = decrypt(encryptedToken, secretKey)

// ❌ Bad: Hardcoded in source
const BOT_TOKEN = "123456789:ABC-DEF..." 

// ❌ Bad: Plain text files
const BOT_TOKEN = fs.readFileSync('token.txt', 'utf8')
```

**Token Rotation**:
- Regenerate bot token every 90 days
- Use BotFather → /revoke to invalidate old tokens
- Update all systems simultaneously
- Monitor for unauthorized usage

### User Privacy

**Data Minimization**:
```typescript
// Only store necessary user data
interface TelegramUser {
  chat_id: string         // Required for messaging
  first_name?: string     // Optional for personalization
  username?: string       // Optional for support
  // Don't store: phone_number, profile_photos, etc.
}
```

**Message Privacy**:
- Don't log sensitive portfolio data
- Use encryption for stored preferences
- Provide clear data deletion options
- Regular data cleanup for inactive users

### Webhook Security

**Validate Requests**:
```typescript
function validateTelegramUpdate(update: any, secretToken: string): boolean {
  // Verify request comes from Telegram
  if (!update.update_id || typeof update.update_id !== 'number') {
    return false
  }
  
  // Check webhook secret if configured
  if (secretToken && update.webhook_secret !== secretToken) {
    return false
  }
  
  // Validate message structure
  if (update.message && !update.message.message_id) {
    return false
  }
  
  return true
}
```

**Rate Limiting Protection**:
```typescript
const rateLimiter = new Map<string, number[]>()

function isRateLimited(chatId: string): boolean {
  const now = Date.now()
  const userRequests = rateLimiter.get(chatId) || []
  
  // Remove requests older than 1 minute
  const recentRequests = userRequests.filter(time => now - time < 60000)
  rateLimiter.set(chatId, recentRequests)
  
  // Allow max 30 requests per minute
  return recentRequests.length >= 30
}
```

### Content Security

**Input Validation**:
```typescript
function sanitizeUserInput(text: string): string {
  // Remove potential XSS attempts
  return text
    .replace(/<script[^>]*>[\s\S]*?<\/script>/gi, '')
    .replace(/<[^>]*>/g, '')
    .substring(0, 4000) // Limit length
}
```

**Prevent Command Injection**:
```typescript
const ALLOWED_COMMANDS = [
  'start', 'help', 'portfolio', 'alerts', 
  'performance', 'strategies', 'settings'
]

function isValidCommand(command: string): boolean {
  return ALLOWED_COMMANDS.includes(command.toLowerCase())
}
```

## Getting Help

### Official Resources
- **Telegram Bot API Docs**: [core.telegram.org/bots/api](https://core.telegram.org/bots/api)
- **BotFather Help**: @BotFather in Telegram
- **Bot Support**: @BotSupport in Telegram
- **Developer Chat**: @BotTalk in Telegram

### Community Resources
- **Bot Development Groups**: Search "telegram bot development"
- **GitHub Examples**: Telegram bot repositories
- **Stack Overflow**: "telegram-bot" tag
- **Reddit**: r/TelegramBots community

### DeFlow Support

For DeFlow-specific integration issues:
1. Check browser console for API errors
2. Verify bot token and Chat ID format
3. Test with simple message first
4. Check Telegram API status page

### Best Practices for Support

**When Asking for Help**:
- Provide specific error messages (without bot token)
- Include relevant code snippets
- Describe expected vs actual behavior
- Mention your bot username (not token)
- Share configuration details (sanitized)

---

*This guide provides a comprehensive foundation for Telegram Bot API integration with DeFlow. Always refer to the latest Telegram Bot API documentation for the most current information, as APIs and policies can change.*

**Last Updated**: August 2025
**Bot API Version**: 7.0+
**DeFlow Compatibility**: v0.1.0+