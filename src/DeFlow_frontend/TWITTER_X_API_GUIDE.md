# Twitter/X API Integration Guide for DeFlow

This comprehensive guide covers everything you need to know about integrating Twitter/X API with DeFlow for automated social media posting and monitoring.

## Table of Contents
1. [Overview](#overview)
2. [Twitter/X API Basics](#twitterx-api-basics)
3. [Developer Account Setup](#developer-account-setup)
4. [App Creation & Configuration](#app-creation--configuration)
5. [Authentication Methods](#authentication-methods)
6. [DeFlow Integration](#deflow-integration)
7. [Use Cases & Examples](#use-cases--examples)
8. [Rate Limits & Best Practices](#rate-limits--best-practices)
9. [Troubleshooting](#troubleshooting)
10. [Advanced Features](#advanced-features)

## Overview

Twitter/X API integration in DeFlow enables automated social media workflows for:
- **Portfolio alerts**: Tweet when DeFi strategies perform well
- **Market updates**: Share insights about crypto markets
- **Community engagement**: Automated responses and interactions
- **Content scheduling**: Post updates at optimal times
- **Monitoring**: Track mentions, hashtags, and sentiment

### Why Twitter/X Integration Matters
- **Real-time communication**: Instant updates to your audience
- **Community building**: Engage with DeFi and crypto communities
- **Marketing automation**: Promote your strategies and insights
- **Social proof**: Share successful trades and performance
- **Network effects**: Build followers through consistent posting

## Twitter/X API Basics

### API Versions
Twitter/X offers multiple API versions:

**1. API v2 (Recommended)**
- Modern, feature-rich API
- Better rate limits
- Improved data models
- OAuth 2.0 support

**2. API v1.1 (Legacy)**
- Older but stable
- Some unique endpoints
- OAuth 1.0a authentication
- Being phased out

### Access Tiers

**Free Tier**
- 1,500 posts per month
- Basic read access
- App-only authentication
- Good for personal use

**Basic Tier ($100/month)**
- 3,000 posts per month
- Enhanced read access
- User authentication
- Small business use

**Pro Tier ($5,000/month)**
- 300,000 posts per month
- Full v2 access
- Advanced features
- Commercial use

**Enterprise**
- Custom limits
- Priority support
- Advanced analytics
- Large-scale operations

## Developer Account Setup

### Step 1: Apply for Developer Account

1. **Visit Developer Portal**
   - Go to [developer.twitter.com](https://developer.twitter.com)
   - Sign in with your Twitter/X account

2. **Apply for Access**
   - Click "Apply for a developer account"
   - Choose your use case:
     - "Making a bot" (for automation)
     - "Exploring the API" (for learning)
     - "Academic research" (for research)

3. **Provide Details**
   - **Primary reason**: Automation and notifications
   - **Use case**: DeFi portfolio management and alerts
   - **Description**: 
     ```
     I'm building automated workflows for DeFi portfolio management 
     that will post updates about portfolio performance, market insights, 
     and trading alerts. The bot will help users stay informed about 
     their investments and share valuable insights with the community.
     ```

4. **Answer Questions**
   - Be specific about your intended use
   - Mention it's for personal/business automation
   - Explain the value to the Twitter community

5. **Wait for Approval**
   - Usually takes 1-7 days
   - Check email for approval/rejection
   - May require additional information

### Step 2: Developer Account Verification

**Email Verification**
- Check your email for verification link
- Click to confirm your developer account

**Phone Verification**
- Add phone number to your Twitter account
- Required for API access

**Account Requirements**
- Account must be in good standing
- No recent violations
- Complete profile (bio, profile picture)

## App Creation & Configuration

### Step 1: Create Twitter App

1. **Access Developer Dashboard**
   - Go to [developer.twitter.com/en/portal/dashboard](https://developer.twitter.com/en/portal/dashboard)
   - Click "Create Project" or "Standalone App"

2. **Project Setup** (if using projects)
   - **Project name**: "DeFlow Integration"
   - **Use case**: "Making a bot"
   - **Description**: "Automated DeFi portfolio alerts and social engagement"

3. **App Details**
   - **App name**: "DeFlow Bot" (must be unique globally)
   - **Description**: "Automated portfolio management and DeFi alerts"
   - **Website**: Your website or GitHub repo
   - **Terms of Service**: Your terms URL (optional)
   - **Privacy Policy**: Your privacy policy URL (optional)

### Step 2: Configure App Permissions

**App Permissions** (in App Settings):
- ✅ **Read**: View tweets, users, lists
- ✅ **Write**: Post tweets, retweets, likes
- ✅ **Direct Messages**: Send/receive DMs (optional)

**User Authentication Settings**:
- ✅ **3-legged OAuth**: For user-specific actions
- **Callback URL**: `http://localhost:4943/oauth/callback/twitter`
- **Website URL**: Your application website
- **Terms of Service**: (optional)
- **Privacy Policy**: (optional)

### Step 3: Generate API Keys

**Essential Keys**:
```
API Key (Consumer Key): abc123...
API Secret (Consumer Secret): def456...
Bearer Token: AAAAAAAAAA...
```

**OAuth 1.0a Keys** (for user authentication):
```
Access Token: 123456789-abc123...
Access Token Secret: def456...
```

**OAuth 2.0 Keys** (recommended):
```
Client ID: abc123...
Client Secret: def456...
```

### Step 4: App Settings Configuration

**Basic Information**:
- App icon (1400x1400px recommended)
- App description (detailed)
- Website URL
- Terms and privacy links

**Authentication**:
- **Type of App**: Web App, Automated App, or Bot
- **Callback URLs**: 
  ```
  http://localhost:4943/oauth/callback/twitter
  https://your-canister-id.ic0.app/oauth/callback/twitter
  ```

**Permissions**:
- Read: ✅ (view tweets, profiles)
- Write: ✅ (post tweets, retweets)
- Direct Messages: ⚪ (optional)

## Authentication Methods

### OAuth 2.0 (Recommended)

**Advantages**:
- Modern standard
- Better security
- Easier implementation
- Supports PKCE

**Flow**:
1. **Authorization URL**:
   ```
   https://twitter.com/i/oauth2/authorize?
   response_type=code&
   client_id=YOUR_CLIENT_ID&
   redirect_uri=YOUR_CALLBACK&
   scope=tweet.read tweet.write users.read&
   state=STATE&
   code_challenge=CHALLENGE&
   code_challenge_method=S256
   ```

2. **Token Exchange**:
   ```javascript
   const tokenResponse = await fetch('https://api.twitter.com/2/oauth2/token', {
     method: 'POST',
     headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
     body: new URLSearchParams({
       code: authCode,
       grant_type: 'authorization_code',
       client_id: CLIENT_ID,
       redirect_uri: CALLBACK_URL,
       code_verifier: CODE_VERIFIER
     })
   })
   ```

### OAuth 1.0a (Legacy)

**Advantages**:
- Access to all v1.1 endpoints
- Some exclusive features
- Well-established

**Implementation**:
More complex, requires signature generation and nonce handling.

### App-Only Authentication

**Use Case**: Read-only access without user context
**Method**: Bearer token authentication
**Limitations**: Cannot post on behalf of users

## DeFlow Integration

### Configuration in DeFlow

**1. Custom API Provider Setup**:
```json
{
  "name": "Twitter Bot",
  "baseUrl": "https://api.twitter.com/2/tweets",
  "method": "POST",
  "authType": "bearer",
  "token": "YOUR_BEARER_TOKEN",
  "headers": {
    "Content-Type": "application/json"
  },
  "bodyTemplate": {
    "text": "{{message}}"
  }
}
```

**2. OAuth 2.0 Setup** (Future Feature):
- Client ID configuration
- Automatic token refresh
- User-specific posting

### Implementation Examples

**Basic Tweet Posting**:
```typescript
// Using custom API provider
await emailService.sendEmail('twitter-api', {
  to: '', // Not used for Twitter
  subject: '', // Not used
  body: JSON.stringify({
    text: "🚀 My DeFi portfolio just gained 15% today! Thanks to automated rebalancing with #DeFlow. #DeFi #Crypto"
  })
})
```

**Advanced Tweet with Media**:
```typescript
const tweetData = {
  text: "📊 Weekly portfolio performance report:",
  media: {
    media_ids: ["1234567890"] // Uploaded media ID
  }
}
```

**Thread Creation**:
```typescript
// Tweet 1
const firstTweet = await postTweet("🧵 Let me share my DeFi strategy performance this week... 1/3")

// Tweet 2 (reply to first)
const secondTweet = await postTweet("The automated rebalancing saved me from a 20% loss during yesterday's market dip... 2/3", {
  reply: { in_reply_to_tweet_id: firstTweet.id }
})

// Tweet 3 (reply to second)
await postTweet("Key lesson: Automation beats emotions in volatile markets. #DeFi #Automation 3/3", {
  reply: { in_reply_to_tweet_id: secondTweet.id }
})
```

## Use Cases & Examples

### 1. Portfolio Performance Alerts

**Trigger**: Portfolio gains/losses exceed threshold
**Action**: Tweet performance update

```typescript
// Portfolio Alert Workflow
if (portfolioChange > 10) {
  const message = `🚀 Portfolio Alert: +${portfolioChange.toFixed(1)}% gain today!
  
💰 Total Value: $${portfolioValue.toLocaleString()}
📈 Best Performer: ${topStrategy.name} (+${topStrategy.gain}%)
⚡ Powered by @DeFlow automation

#DeFi #CryptoGains #Automation`

  await twitterService.postTweet(message)
}
```

### 2. Market Insights Sharing

**Trigger**: Significant market movement detected
**Action**: Share analysis and strategy adjustments

```typescript
// Market Analysis Tweet
const insight = `📊 Market Analysis Alert:

${asset} just ${direction} ${percentage}% in the last hour.

My automated strategy response:
• ${action1}
• ${action2}
• ${action3}

Risk-adjusted returns remain strong 💪

#CryptoTrading #DeFi #MarketAnalysis`
```

### 3. Strategy Success Stories

**Trigger**: Strategy reaches milestone
**Action**: Share success metrics

```typescript
// Milestone Tweet
const celebration = `🎉 Milestone achieved!

My ${strategyName} strategy just:
✅ Reached $${milestone.toLocaleString()} AUM
✅ Maintained ${apy}% APY for 30 days
✅ Zero manual interventions needed

The power of automation! 🤖

Built with @DeFlow
#DeFi #Automation #Yield`
```

### 4. Educational Content

**Trigger**: Weekly/daily schedule
**Action**: Share educational insights

```typescript
// Educational Thread
const educationalContent = `🧵 DeFi Strategy Breakdown: Yield Farming Automation

Today I'll explain how I automated my yield farming to:
• Maximize returns
• Minimize gas fees  
• Reduce emotional decisions
• Save 10+ hours/week

Let's dive in... 1/7 👇

#DeFiEducation #Automation`
```

### 5. Community Engagement

**Trigger**: Mentions or hashtag monitoring
**Action**: Automated responses

```typescript
// Mention Response
if (mention.includes('#DeFlow') && mention.includes('question')) {
  const response = `Thanks for the question! 🙏 
  
DeFlow helps automate DeFi strategies through:
• Portfolio rebalancing
• Yield optimization
• Risk management
• Social notifications

Check out our docs: [link]

Happy to help! 💪`

  await twitterService.replyToTweet(mention.id, response)
}
```

## Rate Limits & Best Practices

### Rate Limits by Tier

**Free Tier**:
- **Posts**: 1,500 tweets per month (~50/day)
- **Reads**: 10,000 requests per month
- **Users**: 300 requests per 15 minutes

**Basic Tier ($100/month)**:
- **Posts**: 3,000 tweets per month (~100/day)
- **Reads**: 50,000 requests per month
- **Users**: 500 requests per 15 minutes

**Pro Tier ($5,000/month)**:
- **Posts**: 300,000 tweets per month (~10,000/day)
- **Reads**: 2,000,000 requests per month
- **Users**: 10,000 requests per 15 minutes

### Best Practices

**Content Guidelines**:
- ✅ **Authentic**: Share real experiences and insights
- ✅ **Valuable**: Provide useful information to followers
- ✅ **Engaging**: Ask questions, start conversations
- ❌ **Spammy**: Avoid repetitive or low-value content
- ❌ **Misleading**: Never post false information
- ❌ **Too frequent**: Respect your audience's attention

**Technical Best Practices**:
- **Rate limiting**: Implement exponential backoff
- **Error handling**: Gracefully handle API errors
- **Content variation**: Avoid duplicate posts
- **Timing**: Post when your audience is active
- **Monitoring**: Track engagement and adjust strategy

**Compliance**:
- Follow Twitter's Terms of Service
- Respect user privacy
- Don't manipulate trending topics
- Avoid coordinated inauthentic behavior
- Disclose automation when appropriate

### Rate Limiting Implementation

```typescript
class TwitterRateLimiter {
  private queue: Array<() => Promise<any>> = []
  private processing = false
  private readonly DELAY_MS = 1000 // 1 second between requests

  async addRequest<T>(request: () => Promise<T>): Promise<T> {
    return new Promise((resolve, reject) => {
      this.queue.push(async () => {
        try {
          const result = await request()
          resolve(result)
        } catch (error) {
          reject(error)
        }
      })

      this.processQueue()
    })
  }

  private async processQueue() {
    if (this.processing || this.queue.length === 0) return

    this.processing = true

    while (this.queue.length > 0) {
      const request = this.queue.shift()!
      await request()
      
      if (this.queue.length > 0) {
        await new Promise(resolve => setTimeout(resolve, this.DELAY_MS))
      }
    }

    this.processing = false
  }
}
```

## Troubleshooting

### Common Issues

**"Unauthorized" (401 Error)**:
- ❌ Invalid API keys
- ❌ Expired access token
- ❌ Incorrect authentication method
- ✅ **Fix**: Regenerate keys, check authentication flow

**"Forbidden" (403 Error)**:
- ❌ App lacks required permissions
- ❌ User hasn't granted access
- ❌ Attempting restricted action
- ✅ **Fix**: Update app permissions, re-authenticate user

**"Rate Limited" (429 Error)**:
- ❌ Exceeded rate limits
- ❌ Too many requests too quickly
- ✅ **Fix**: Implement rate limiting, reduce frequency

**"Duplicate Content" (403 Error)**:
- ❌ Posted identical tweet recently
- ❌ Very similar content detected
- ✅ **Fix**: Add timestamps, vary content, wait longer between similar posts

**"Media Upload Failed"**:
- ❌ File too large (>5MB for images, >512MB for videos)
- ❌ Unsupported format
- ❌ Invalid media ID
- ✅ **Fix**: Compress media, use supported formats (PNG, JPG, GIF, MP4)

### Debugging Tips

**Enable Debug Logging**:
```typescript
const twitterClient = new TwitterApi({
  appKey: API_KEY,
  appSecret: API_SECRET,
  accessToken: ACCESS_TOKEN,
  accessSecret: ACCESS_SECRET,
}, { debug: true })
```

**Test with Simple Requests**:
```typescript
// Test authentication
const user = await twitterClient.currentUser()
console.log('Authenticated as:', user.screen_name)

// Test posting
const tweet = await twitterClient.v2.tweet('Test tweet from DeFlow!')
console.log('Posted tweet:', tweet.data.id)
```

**Monitor Rate Limits**:
```typescript
const rateLimits = await twitterClient.v1.getRateLimitStatus()
console.log('Remaining tweet posts:', rateLimits.resources.statuses['/statuses/update'].remaining)
```

## Advanced Features

### 1. Media Handling

**Image Upload**:
```typescript
// Upload image
const mediaId = await twitterClient.v1.uploadMedia('./chart.png')

// Tweet with image
await twitterClient.v2.tweet({
  text: "📊 My portfolio performance this week:",
  media: { media_ids: [mediaId] }
})
```

**Video Upload**:
```typescript
// Upload video (chunked for large files)
const mediaId = await twitterClient.v1.uploadMedia('./performance-video.mp4', {
  type: 'video/mp4'
})

await twitterClient.v2.tweet({
  text: "🎥 Watch my DeFi strategy in action:",
  media: { media_ids: [mediaId] }
})
```

### 2. Polls and Interactive Content

**Create Poll**:
```typescript
await twitterClient.v2.tweet({
  text: "Which DeFi strategy interests you most?",
  poll: {
    options: ['Yield Farming', 'Liquidity Mining', 'Arbitrage', 'Lending'],
    duration_minutes: 1440 // 24 hours
  }
})
```

### 3. Scheduled Posting

**Queue System**:
```typescript
class TweetScheduler {
  private schedule: Array<{tweet: string, time: Date}> = []

  scheduletweet(content: string, scheduledTime: Date) {
    this.schedule.push({
      tweet: content,
      time: scheduledTime
    })
  }

  async processPendingTweets() {
    const now = new Date()
    const readyTweets = this.schedule.filter(item => item.time <= now)

    for (const item of readyTweets) {
      await twitterClient.v2.tweet(item.tweet)
      this.schedule = this.schedule.filter(s => s !== item)
    }
  }
}
```

### 4. Analytics and Monitoring

**Track Performance**:
```typescript
// Get tweet analytics
const tweetId = 'your_tweet_id'
const analytics = await twitterClient.v2.tweets(tweetId, {
  'tweet.fields': ['public_metrics']
})

console.log('Tweet performance:', {
  retweets: analytics.data[0].public_metrics.retweet_count,
  likes: analytics.data[0].public_metrics.like_count,
  replies: analytics.data[0].public_metrics.reply_count,
  quotes: analytics.data[0].public_metrics.quote_count
})
```

**Monitor Mentions**:
```typescript
// Search for mentions
const mentions = await twitterClient.v2.search('@your_username', {
  'tweet.fields': ['created_at', 'author_id'],
  'user.fields': ['username']
})

for (const mention of mentions.data) {
  console.log(`Mentioned by @${mention.author_id}: ${mention.text}`)
}
```

### 5. Sentiment Analysis Integration

**Basic Sentiment Detection**:
```typescript
function analyzeSentiment(text: string): 'positive' | 'negative' | 'neutral' {
  const positiveWords = ['gain', 'profit', 'up', 'bullish', 'moon', 'pump']
  const negativeWords = ['loss', 'down', 'bearish', 'crash', 'dump', 'rekt']
  
  const lowerText = text.toLowerCase()
  const positiveCount = positiveWords.filter(word => lowerText.includes(word)).length
  const negativeCount = negativeWords.filter(word => lowerText.includes(word)).length
  
  if (positiveCount > negativeCount) return 'positive'
  if (negativeCount > positiveCount) return 'negative'
  return 'neutral'
}

// Adjust posting strategy based on market sentiment
const marketSentiment = analyzeSentiment(recentTweets.join(' '))
if (marketSentiment === 'negative') {
  // Post more educational/supportive content
  await postEducationalThread()
} else {
  // Share performance updates
  await postPerformanceUpdate()
}
```

## Security Considerations

### Key Management
- **Never commit API keys** to version control
- **Use environment variables** for sensitive data
- **Rotate keys regularly** (every 90 days recommended)
- **Use least privilege principle** (minimal required permissions)

### User Data Protection
- **Don't store user credentials** unnecessarily
- **Encrypt stored tokens** if caching is required
- **Respect user privacy** and data retention policies
- **Provide clear opt-out mechanisms**

### Automation Ethics
- **Be transparent** about automation
- **Add value** to conversations
- **Respect community guidelines**
- **Monitor and moderate** automated content

## Compliance and Legal

### Twitter Terms of Service
- Review and comply with current ToS
- Understand automation policies
- Respect rate limits and fair use
- Report and address violations promptly

### Financial Regulations
- **Investment advice disclaimers**: "Not financial advice"
- **Performance disclaimers**: "Past performance doesn't guarantee future results"
- **Risk warnings**: Highlight risks of DeFi and crypto
- **Compliance with local laws**: Financial promotion rules

### GDPR and Privacy
- **Data minimization**: Collect only necessary data
- **User consent**: Clear consent for data processing
- **Right to deletion**: Allow users to remove their data
- **Privacy policy**: Document data handling practices

## Getting Help

### Resources
- **Twitter Developer Docs**: [developer.twitter.com](https://developer.twitter.com)
- **API Reference**: Detailed endpoint documentation
- **Community Forums**: Developer community support
- **Status Page**: API status and incidents

### Support Channels
- **Developer Support**: For API-related issues
- **Twitter Help**: General platform support
- **Community Forums**: Peer-to-peer help
- **GitHub**: Open-source libraries and examples

### Best Practices for Support
- **Provide specific error messages**
- **Include relevant code snippets** (without API keys)
- **Describe expected vs actual behavior**
- **Share your app ID** (not API keys)

---

*This guide provides a comprehensive foundation for Twitter/X API integration with DeFlow. Always refer to the latest Twitter Developer documentation for the most current information, as APIs and policies can change.*

**Last Updated**: August 2025
**API Version**: v2
**DeFlow Compatibility**: v0.1.0+