# Twitter/X Integration

{% hint style="info" %}
**Twitter/X integration requires Premium tier or higher** to access social media nodes beyond Discord and Telegram.
{% endhint %}

## Overview

Twitter/X integration in DeFlow enables powerful automated social media workflows for DeFi users, content creators, and businesses. Whether you're sharing portfolio updates, market insights, or community engagement, DeFlow makes it seamless.

### Common Use Cases

* **📈 Portfolio Alerts** - Tweet when DeFi strategies perform well
* **📊 Market Updates** - Share insights about crypto markets  
* **💬 Community Engagement** - Automated responses and interactions
* **⏰ Content Scheduling** - Post updates at optimal times
* **👀 Social Monitoring** - Track mentions, hashtags, and sentiment

## 🚀 Quick Setup

### Prerequisites

{% hint style="warning" %}
**Required**: Twitter/X Developer Account with API access
{% endhint %}

1. **Developer Account**: Apply at [developer.twitter.com](https://developer.twitter.com)
2. **API Tier**: Free tier (1,500 posts/month) or paid tiers for higher limits
3. **DeFlow Subscription**: Premium ($19/month) or higher for social media nodes

### Step 1: Twitter Developer Account

1. **Visit Developer Portal**
   ```
   🌐 https://developer.twitter.com
   ```
   
2. **Apply for Access**
   - Choose "Making a bot" for automation
   - Describe your use case:
   ```
   Building automated workflows for DeFi portfolio management 
   that will post updates about portfolio performance, market insights, 
   and trading alerts to help users stay informed about their investments.
   ```

3. **Account Approval**
   - Usually takes 1-2 business days
   - Check email for approval notification
   - Complete any additional verification steps

### Step 2: Create Twitter App

1. **Navigate to App Creation**
   ```
   Developer Portal → Apps → Create App
   ```

2. **App Configuration**
   - **App Name**: `DeFlow-Bot-[YourUsername]`
   - **Description**: `Automated DeFi portfolio alerts and social media management`
   - **Website**: Your website or `https://deflow.app`
   - **Callback URL**: `https://app.deflow.app/auth/twitter/callback`

3. **App Permissions**
   - ✅ **Read**: Required for monitoring
   - ✅ **Write**: Required for posting
   - ✅ **Direct Messages**: Optional for DM automation

### Step 3: Get API Credentials

After app creation, collect these credentials:

```javascript
API Configuration
├── API Key (Consumer Key)
├── API Secret (Consumer Secret) 
├── Bearer Token
└── Access Token & Secret (after OAuth)
```

{% hint style="danger" %}
**Security**: Never share your API keys publicly. Store them securely in DeFlow.
{% endhint %}

## 🔧 DeFlow Configuration

### Adding Twitter Integration

1. **Open Node Palette**
   - Navigate to your workflow builder
   - Find "Social Media" section
   - Select "Twitter Post" node

2. **Authentication Setup**
   ```yaml
   Connection Type: OAuth 2.0
   API Version: v2 (Recommended)
   Credentials: [Add New Connection]
   ```

3. **OAuth Flow**
   - Click "Authenticate with Twitter"
   - Authorize DeFlow app access
   - Connection established automatically

### Node Configuration

#### Tweet Content Node
```yaml
Content Options:
  - Text: Up to 280 characters
  - Media: Images, videos, GIFs
  - Links: Auto-shortened URLs
  - Hashtags: Automatic or manual
  - Mentions: @username format
```

#### Tweet Scheduling Node
```yaml
Timing Options:
  - Immediate: Post right away
  - Scheduled: Set specific time
  - Optimal: AI-suggested best times
  - Recurring: Daily/weekly patterns
```

#### Monitoring Node
```yaml
Track Options:
  - Keywords: Specific terms
  - Hashtags: #DeFi, #Bitcoin, etc.
  - Mentions: @YourUsername
  - Sentiment: Positive/negative analysis
```

## 📱 Node Types

### 1. Tweet Post Node

**Purpose**: Send tweets with text, media, and links

**Configuration**:
```yaml
tweet_content:
  text: "🚀 Portfolio update: ${portfolio_value} (+${gain_percentage}%)"
  media: ${chart_image}
  hashtags: ["#DeFi", "#Portfolio", "#Crypto"]
  
conditional_posting:
  only_if: portfolio_gain > 5%
  max_frequency: "1 per hour"
```

**Example Workflow**:
```
Portfolio Check → If Gain > 5% → Tweet Post → Community Notification
```

### 2. Twitter Monitor Node

**Purpose**: Listen for mentions, keywords, or hashtags

**Configuration**:
```yaml
monitoring:
  keywords: ["DeFi", "yield farming", "@YourBot"]
  hashtags: ["#DeFlowAlert", "#CryptoGains"]
  mentions: true
  
response_triggers:
  auto_reply: true
  sentiment_filter: "positive_only"
  rate_limit: "5 replies per hour"
```

### 3. Thread Creation Node

**Purpose**: Create Twitter threads for longer content

**Configuration**:
```yaml
thread_content:
  intro: "🧵 Weekly DeFi Performance Thread"
  tweets: [
    "1/ This week's portfolio performance: ${weekly_summary}",
    "2/ Top performing strategy: ${best_strategy}",
    "3/ Risk analysis: ${risk_metrics}"
  ]
  conclusion: "That's a wrap! Questions? Drop them below 👇"
```

### 4. Social Listening Node

**Purpose**: Analyze social sentiment and trends

**Configuration**:
```yaml
analysis:
  sentiment_tracking: true
  trend_detection: true
  competitor_monitoring: ["@competitor1", "@competitor2"]
  
outputs:
  sentiment_score: number
  trending_topics: array
  engagement_rate: percentage
```

## 🎯 Workflow Examples

### Example 1: Portfolio Performance Alerts

```yaml
Workflow: "DeFi Performance Tracker"
Trigger: Daily at 9 AM
Steps:
  1. Fetch Portfolio Data
  2. Calculate 24h Performance
  3. If Gain > 3%:
     - Generate Chart
     - Tweet: "📈 Portfolio gained ${gain}% today! 
              Strategy: ${top_strategy}
              #DeFi #CryptoGains"
  4. If Loss > 5%:
     - Tweet: "⚠️ Portfolio adjustment needed.
              Risk management activated.
              #RiskManagement #DeFi"
```

### Example 2: Market News Amplifier

```yaml
Workflow: "Crypto News Amplifier"
Trigger: RSS Feed Update
Steps:
  1. Monitor Crypto News Sources
  2. Filter for Relevant Articles
  3. Summarize with AI
  4. Tweet: "📰 Breaking: ${news_summary}
            Link: ${article_url}
            #CryptoNews #DeFi"
  5. Schedule Follow-up Analysis
```

### Example 3: Community Engagement Bot

```yaml
Workflow: "Community Engagement"
Trigger: Mention Detection
Steps:
  1. Monitor @mentions
  2. Analyze Sentiment
  3. If Positive Question:
     - Generate Helpful Response
     - Reply with Portfolio Tip
  4. If Support Request:
     - Tag Human Support
     - Provide Initial Resources
```

## 📊 Rate Limits & Best Practices

### API Rate Limits

| Tier | Monthly Posts | Per Hour | Special Limits |
|------|---------------|----------|----------------|
| **Free** | 1,500 | ~50 | App-only auth |
| **Basic** | 3,000 | ~100 | User auth available |
| **Pro** | 300,000 | ~10,000 | Full feature access |

### Best Practices

{% hint style="success" %}
**Engagement Tips**
- Post during peak hours (typically 9 AM - 3 PM in your audience's timezone)
- Use relevant hashtags (2-3 per tweet)
- Include visuals when possible (images, charts, GIFs)
- Engage authentically - avoid pure automation
{% endhint %}

1. **Content Quality**
   - Provide value in every tweet
   - Mix automated and manual content
   - Include relevant hashtags and mentions
   - Use emojis for visual appeal

2. **Timing Strategy**
   - Post during audience active hours
   - Spread posts throughout the day
   - Avoid spam-like posting patterns
   - Respect rate limits

3. **Engagement Focus**
   - Respond to replies and mentions
   - Retweet and comment on community content
   - Share others' valuable insights
   - Build genuine relationships

## 🚨 Troubleshooting

### Common Issues

#### Authentication Errors
```yaml
Error: "Unauthorized access"
Solution:
  - Verify API credentials
  - Check app permissions
  - Regenerate access tokens
  - Confirm callback URL
```

#### Rate Limit Exceeded
```yaml
Error: "Too many requests"
Solution:
  - Reduce posting frequency
  - Implement backoff strategy
  - Upgrade API tier if needed
  - Monitor usage dashboard
```

#### Content Rejected
```yaml
Error: "Tweet content violation"
Solution:
  - Check character limits (280)
  - Remove duplicate content
  - Avoid spam triggers
  - Review Twitter rules
```

### Getting Help

- **Twitter Developer Forums**: [twittercommunity.com](https://twittercommunity.com)
- **DeFlow Support**: help@deflow.app
- **API Documentation**: [developer.twitter.com/docs](https://developer.twitter.com/docs)

## 🔐 Security & Compliance

### API Security
- Store credentials securely in DeFlow
- Use OAuth 2.0 when possible  
- Regularly rotate access tokens
- Monitor for unauthorized access

### Content Compliance
- Follow Twitter's Terms of Service
- Avoid automated spam behavior
- Respect user privacy
- Include proper disclosures for promotional content

### GDPR & Privacy
- Only collect necessary data
- Provide opt-out mechanisms
- Respect user data requests
- Maintain audit logs

## 📈 Advanced Features

### Analytics Integration
```yaml
Metrics Tracking:
  - Tweet impressions
  - Engagement rates
  - Follower growth
  - Click-through rates
  
Integration Options:
  - Google Analytics
  - Twitter Analytics API
  - Custom dashboard
```

### AI-Powered Content
```yaml
AI Features:
  - Sentiment analysis
  - Content optimization
  - Trend prediction
  - Automated replies
  
Implementation:
  - OpenAI GPT integration
  - Custom training data
  - Content moderation
```

### Multi-Account Management
```yaml
Account Management:
  - Multiple Twitter accounts
  - Cross-platform posting
  - Centralized scheduling
  - Performance comparison
```

## 💡 Pro Tips

1. **Content Mix**: 80% value, 20% promotion
2. **Visual Appeal**: Include charts, graphs, and images
3. **Community First**: Engage before promoting
4. **Consistency**: Regular posting schedule builds audience
5. **Authenticity**: Mix automation with genuine interaction

---

Ready to start automating your Twitter presence? [Set up your first Twitter workflow →](../../workflows/creating-workflows.md)