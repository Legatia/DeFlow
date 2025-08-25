# Facebook Integration Guide for DeFlow

This guide covers implementing and using Facebook integration in DeFlow workflows for social media automation, page management, and community engagement.

## Table of Contents

1. [Developer Implementation Guide](#developer-implementation-guide)
2. [User Setup Guide](#user-setup-guide)
3. [API Reference](#api-reference)
4. [Workflow Examples](#workflow-examples)
5. [Best Practices](#best-practices)
6. [Troubleshooting](#troubleshooting)

---

## Developer Implementation Guide

### 1. Facebook API Overview

Facebook provides several APIs through the Meta for Developers platform:

- **Graph API**: Primary API for posting content, managing pages
- **Instagram Graph API**: For Instagram business accounts
- **WhatsApp Business API**: For WhatsApp automation
- **Messenger Platform**: For chatbot integration

### 2. Authentication Methods

#### OAuth 2.0 Flow with App Review
```typescript
interface FacebookOAuthConfig {
  app_id: string
  app_secret: string
  redirect_uri: string
  scope: string[]
}

// Required permissions for basic posting
const DEFAULT_PERMISSIONS = [
  'pages_manage_posts',      // Post to pages
  'pages_read_engagement',   // Read page insights
  'publish_to_groups',       // Post to groups (if approved)
  'user_posts'              // Post to user timeline (deprecated)
]
```

#### Access Token Types
```typescript
interface FacebookTokens {
  user_access_token: string      // Short-lived (1-2 hours)
  page_access_token: string      // Long-lived (60 days)
  app_access_token: string       // App-level access
  long_lived_token: string       // Extended user token (60 days)
}
```

### 3. Node Architecture Design

Following DeFlow's modular approach:

#### A. Facebook API Sender Node
```typescript
{
  id: 'facebook-post',
  name: 'Facebook Post',
  description: 'Post content to Facebook pages or groups - accepts text or JSON data',
  category: 'integrations',
  icon: '📘',
  color: '#1877F2',
  inputs: [
    { id: 'message', name: 'Message Data', type: 'data', required: true }
  ],
  outputs: [
    { id: 'result', name: 'Facebook Result', type: 'data', required: true }
  ],
  configSchema: [
    {
      key: 'access_token',
      name: 'Page Access Token',
      type: 'password',
      required: true,
      description: 'Facebook Page access token (long-lived)'
    },
    {
      key: 'page_id',
      name: 'Page ID',
      type: 'text',
      required: true,
      placeholder: '1234567890',
      description: 'Facebook Page ID to post to'
    },
    {
      key: 'post_type',
      name: 'Post Type',
      type: 'select',
      options: [
        { label: 'Page Post', value: 'page' },
        { label: 'Group Post', value: 'group' },
        { label: 'Event Update', value: 'event' }
      ],
      defaultValue: 'page'
    }
  ]
}
```

#### B. Enhanced Social Media Formatters
Update existing nodes to support Facebook:

```typescript
// Add Facebook to platform options
{
  key: 'platform',
  options: [
    { label: 'Twitter/X (280 chars)', value: 'twitter' },
    { label: 'Discord (2000 chars)', value: 'discord' },
    { label: 'LinkedIn (3000 chars)', value: 'linkedin' },
    { label: 'Facebook (63,206 chars)', value: 'facebook' }, // Add this
    { label: 'General (no limit)', value: 'general' }
  ]
}
```

### 4. Service Implementation

#### Facebook Service Structure
```typescript
// src/services/facebookService.ts
export interface FacebookPostConfig {
  message: string
  link?: string
  picture?: string
  name?: string
  caption?: string
  description?: string
  place?: string
  tags?: string[]
  scheduled_publish_time?: number
  published?: boolean
}

export interface FacebookResponse {
  id?: string
  post_id?: string
  created_time?: string
  message?: string
  error?: {
    code: number
    message: string
    type: string
    fbtrace_id: string
  }
}

class FacebookService {
  async postToPage(
    pageAccessToken: string,
    pageId: string,
    config: FacebookPostConfig,
    templateVariables: Record<string, any> = {}
  ): Promise<FacebookResponse>

  async postToGroup(
    userAccessToken: string,
    groupId: string,
    config: FacebookPostConfig,
    templateVariables: Record<string, any> = {}
  ): Promise<FacebookResponse>

  async validatePageToken(
    pageAccessToken: string,
    pageId: string
  ): Promise<{ valid: boolean; error?: string }>

  async getPageInfo(
    pageAccessToken: string,
    pageId: string
  ): Promise<{ name: string; category: string; followers_count: number }>
}
```

#### Key API Endpoints
```typescript
const FACEBOOK_ENDPOINTS = {
  PAGE_POSTS: (pageId: string) => `https://graph.facebook.com/v18.0/${pageId}/feed`,
  GROUP_POSTS: (groupId: string) => `https://graph.facebook.com/v18.0/${groupId}/feed`,
  PAGE_INFO: (pageId: string) => `https://graph.facebook.com/v18.0/${pageId}`,
  MEDIA_UPLOAD: 'https://graph.facebook.com/v18.0/me/photos',
  TOKEN_DEBUG: 'https://graph.facebook.com/debug_token'
}
```

### 5. Rate Limiting

Facebook API limits are complex and vary by app type:

```typescript
// Rate limits (per app, per page)
const FACEBOOK_LIMITS = {
  PAGE_POSTS: 200,        // per hour per page
  GROUP_POSTS: 50,        // per hour per group
  MEDIA_UPLOADS: 100,     // per hour per page
  API_CALLS: 200          // per hour per user per app
}

checkRateLimit(endpoint: string, targetId: string): boolean {
  // Implementation with sliding window
  // Track per-page and per-app limits
}
```

### 6. Media Handling

Facebook supports rich media:
```typescript
interface FacebookMedia {
  type: 'photo' | 'video' | 'link' | 'album'
  url: string
  caption?: string
  alt_text?: string
  published?: boolean
}

// Photo upload flow
async uploadPhoto(pageToken: string, pageId: string, photoUrl: string): Promise<string> {
  // 1. Upload photo to Facebook
  // 2. Get photo ID
  // 3. Use photo ID in post
}
```

---

## User Setup Guide

### 1. Facebook Developer App Setup

#### Step 1: Create Facebook App
1. Go to [Meta for Developers](https://developers.facebook.com/)
2. Click "Create App"
3. Select "Business" app type
4. Fill in app details:
   - **App name**: "DeFlow Social Automation"
   - **Contact email**: Your email
   - **Business account**: Select or create

#### Step 2: Add Facebook Login Product
1. In app dashboard, click "Add Product"
2. Select "Facebook Login"
3. Configure settings:
   - **Valid OAuth Redirect URIs**: `http://localhost:3000/auth/facebook/callback`
   - **Client OAuth Settings**: Enable relevant options

#### Step 3: Add Pages API Product
1. Click "Add Product" again
2. Select "Pages API"
3. This allows posting to Facebook pages

#### Step 4: App Review (Required for Live Usage)
1. Add app details (Privacy Policy, Terms of Service)
2. Submit for review for required permissions:
   - `pages_manage_posts`
   - `pages_read_engagement`
   - `publish_to_groups` (if needed)

### 2. Facebook Page Setup

#### Step 1: Create/Access Business Page
1. Ensure you have a Facebook Business Page
2. Make sure you're an admin of the page
3. Note the Page ID (found in Page settings)

#### Step 2: Generate Page Access Token
1. Go to [Graph API Explorer](https://developers.facebook.com/tools/explorer/)
2. Select your app and page
3. Request `pages_manage_posts` permission
4. Generate long-lived page access token

### 3. DeFlow Configuration

#### Step 1: Add to Settings
1. Open DeFlow Settings page
2. Go to "External Integrations" section
3. Click "Add Facebook Configuration"
4. Enter:
   - **Configuration Name**: "Main Business Page"
   - **Page Access Token**: Your long-lived token
   - **Page ID**: Your Facebook page ID

#### Step 2: Test Connection
1. Click "Test Connection"
2. Verify it shows page name and follower count
3. Check connection status shows "✅ Connected"

### 4. Creating Facebook Workflows

#### Basic Page Update
```
Schedule Trigger → Social Media Text → Facebook Post
```

**Configuration:**
- **Social Media Text**: Set platform to "Facebook (63,206 chars)"
- **Content**: Engaging content with call-to-action
- **Facebook Post**: Select your configured page

#### Event Promotion
```
Webhook → Social Media with Image → Facebook Post
```

**Use Case**: Promote DeFi events, webinars, product launches

#### Cross-Platform Posting
```
Content Source → Social Media Text → Facebook Post
                                   → Twitter Post  
                                   → LinkedIn Post
```

---

## API Reference

### Authentication Flow

#### 1. User Authorization
```
https://www.facebook.com/v18.0/dialog/oauth?
  client_id={app-id}&
  redirect_uri={redirect-uri}&
  scope={permissions}&
  response_type=code
```

#### 2. Exchange Code for Token
```bash
curl -X GET "https://graph.facebook.com/v18.0/oauth/access_token?
  client_id={app-id}&
  client_secret={app-secret}&
  redirect_uri={redirect-uri}&
  code={authorization-code}"
```

#### 3. Get Long-Lived Token
```bash
curl -X GET "https://graph.facebook.com/v18.0/oauth/access_token?
  grant_type=fb_exchange_token&
  client_id={app-id}&
  client_secret={app-secret}&
  fb_exchange_token={short-lived-token}"
```

### Posting Endpoints

#### Page Post
```bash
POST https://graph.facebook.com/v18.0/{page-id}/feed
Content-Type: application/json
Authorization: Bearer {page-access-token}

{
  "message": "Your post content here #hashtags",
  "link": "https://example.com",
  "published": true
}
```

#### Photo Post
```bash
POST https://graph.facebook.com/v18.0/{page-id}/photos
Content-Type: application/json

{
  "caption": "Photo caption with #hashtags",
  "url": "https://example.com/image.jpg",
  "published": true
}
```

### Error Codes

| Code | Type | Meaning | Solution |
|------|------|---------|----------|
| 100 | OAuthException | Invalid access token | Refresh token or re-authenticate |
| 190 | OAuthException | Token expired | Get new long-lived token |
| 200 | PermissionsError | Missing permissions | Update app permissions |
| 341 | ApplicationThrottledException | Rate limited | Implement backoff strategy |
| 506 | DuplicatePostException | Duplicate content | Add unique content or timestamps |

---

## Workflow Examples

### 1. Daily Business Updates
**Use Case**: Share daily business metrics and insights

**Template**:
```
📈 Daily Business Update - {{date}}

🎯 Key Metrics:
• Customer Growth: {{customer_growth}}%
• Revenue: ${{daily_revenue}}
• Platform Usage: {{active_users}} users

💡 Today's Insight: {{business_insight}}

What's your biggest business challenge today? 
Let's discuss in the comments! 👇

#Business #DeFi #Growth #Entrepreneurship
```

### 2. Product Launch Campaign
**Use Case**: Coordinate product launches across Facebook and other platforms

**Nodes**:
1. **Manual Trigger**: Launch button
2. **Social Media with Image**: Product announcement with hero image
3. **Facebook Post**: Main announcement
4. **Delay Node**: Wait 2 hours
5. **Social Media Text**: Follow-up reminder
6. **Facebook Post**: Reminder post

### 3. Community Engagement
**Use Case**: Foster community discussion and engagement

**Template**:
```
🤔 Community Question of the Day

{{question_text}}

📊 Poll in comments:
A) {{option_a}}
B) {{option_b}}  
C) {{option_c}}

Your thoughts? Drop a comment below! 💬

#CommunityEngagement #DeFi #Discussion
```

---

## Best Practices

### Content Strategy

#### Engagement-Focused
- Ask questions to encourage comments
- Use polls and interactive content
- Respond promptly to comments
- Share user-generated content

#### Visual Content
- Include images or videos when possible
- Use consistent brand colors and fonts
- Create shareable infographics
- Add captions for accessibility

### Posting Guidelines

#### Frequency
- **Business Pages**: 1-2 posts per day maximum
- **Groups**: Follow group rules (varies)
- **Stories**: 3-5 stories per day acceptable

#### Timing
- **Best times**: 9-10 AM and 1-3 PM weekdays
- **Avoid**: Late nights and early mornings
- **Test**: Use Facebook Insights to find your audience's peak times

### Compliance

#### Facebook Policies
- Follow Community Standards
- Respect intellectual property
- No misleading information
- Comply with advertising policies

#### Data Protection
- Implement proper consent mechanisms
- Follow GDPR for EU users
- Respect user privacy settings
- Secure token storage

---

## Security Considerations

### App Security
```typescript
// Secure token management
interface FacebookSecurityConfig {
  token_encryption: boolean
  token_rotation: number    // days
  rate_limiting: boolean
  content_filtering: boolean
}
```

### Content Validation
```typescript
const validateFacebookContent = (content: string): boolean => {
  // Check for policy violations
  // Validate URLs and links
  // Scan for spam indicators
  // Verify character limits
}
```

### Privacy Protection
- Never store user personal data
- Use minimal required permissions
- Implement data retention policies
- Provide clear privacy disclosures

---

## Implementation Challenges

### 1. App Review Process
**Challenge**: Facebook requires app review for most useful permissions
**Solution**: 
- Plan 2-4 weeks for review process
- Provide detailed use case documentation
- Create privacy policy and terms of service
- Submit video demonstrations

### 2. Token Management
**Challenge**: Complex token types and expiration
**Solution**:
```typescript
class FacebookTokenManager {
  async refreshLongLivedToken(shortToken: string): Promise<string>
  async getPageTokens(userToken: string): Promise<PageToken[]>
  async validateToken(token: string): Promise<TokenInfo>
}
```

### 3. Rate Limiting Complexity
**Challenge**: Different limits for different endpoints
**Solution**: Implement sophisticated rate limiting:

```typescript
const FACEBOOK_RATE_LIMITS = {
  PAGE_POSTS: { calls: 200, window: 3600 },      // 200 per hour
  GRAPH_API: { calls: 100, window: 3600 },       // 100 per hour per user
  BATCH_API: { calls: 50, window: 3600 }         // 50 batch requests per hour
}
```

---

## User Setup Guide

### 1. Prerequisites

#### Business Requirements
- Facebook Business Page (required)
- Meta Business Account (recommended)
- Verified business information
- Valid privacy policy and terms of service

#### App Approval Process
1. **Development Phase**: Use test mode with limited features
2. **Submission Phase**: Submit app for review with required documentation
3. **Review Phase**: Meta reviews app (2-7 business days)
4. **Live Phase**: Full API access once approved

### 2. Step-by-Step Setup

#### Step 1: Create Facebook App
1. Visit [Meta for Developers](https://developers.facebook.com/)
2. Create new app with "Business" type
3. Add "Facebook Login" and "Pages API" products
4. Configure OAuth settings

#### Step 2: Get Page Access Token
1. Use [Graph API Explorer](https://developers.facebook.com/tools/explorer/)
2. Select your app
3. Get User Access Token with `pages_manage_posts` permission
4. Use `/me/accounts` endpoint to get page tokens
5. Exchange for long-lived page token

#### Step 3: DeFlow Configuration
1. In DeFlow Settings → External Integrations
2. Click "Add Facebook Configuration"
3. Enter:
   - **Configuration Name**: "Business Page"
   - **Page Access Token**: Your long-lived token
   - **Page ID**: Your page ID
4. Test connection

### 3. Advanced Configuration

#### Multiple Pages
```typescript
// Support multiple business pages
interface FacebookPageConfig {
  id: string
  name: string
  page_id: string
  access_token: string
  category: string
  followers_count: number
}
```

#### Scheduled Posts
```typescript
// Facebook supports native scheduled posting
{
  scheduled_publish_time: unixTimestamp,
  published: false  // Creates unpublished scheduled post
}
```

---

## Workflow Examples

### 1. Business Page Content Calendar
**Use Case**: Automated content calendar for business page

**Schedule**:
- **Monday**: Industry news and insights
- **Wednesday**: Product updates and features
- **Friday**: Community highlights and engagement

**Implementation**:
```
Schedule Trigger (Multiple) → Content Router → Social Media Text → Facebook Post
```

### 2. Customer Success Stories
**Use Case**: Share customer wins and case studies

**Template**:
```
🎉 Customer Success Story

Meet {{customer_name}}, who achieved {{achievement}} using DeFlow!

📊 Results:
• {{metric_1}}: {{value_1}}
• {{metric_2}}: {{value_2}}
• {{metric_3}}: {{value_3}}

💡 "{{customer_quote}}" - {{customer_name}}, {{customer_title}}

Want similar results? DM us to get started! 💬

#CustomerSuccess #DeFi #Automation #Results
```

### 3. Event Promotion Funnel
**Use Case**: Multi-stage event promotion campaign

**Flow**:
```
Event Created → Announcement Post → Reminder Posts → Live Updates → Follow-up
```

**Stages**:
1. **Week before**: Event announcement with registration link
2. **Day before**: Reminder with agenda highlights  
3. **Day of**: Live updates and key takeaways
4. **Day after**: Thank you post with recording link

---

## Best Practices

### Content Guidelines

#### Engaging Content
- Use questions to drive comments
- Include calls-to-action
- Share behind-the-scenes content
- Celebrate community achievements

#### Visual Standards
- Maintain consistent brand visuals
- Use high-quality images (1200x628px recommended)
- Include alt text for accessibility
- Test on mobile devices

### Growth Strategies

#### Organic Reach
- Post when your audience is most active
- Use relevant hashtags (max 2-3 on Facebook)
- Encourage shares and saves
- Respond quickly to comments

#### Community Building
- Create Facebook Groups for deeper engagement
- Host live sessions and Q&As
- Share user-generated content
- Collaborate with other businesses

---

## Troubleshooting

### Common Issues

#### 1. "Token Invalid" Error
**Causes**:
- Token expired (page tokens last 60 days)
- Insufficient permissions
- App not approved for production

**Solutions**:
```typescript
// Implement token refresh
async refreshPageToken(userToken: string, pageId: string): Promise<string> {
  const response = await fetch(
    `https://graph.facebook.com/v18.0/${pageId}?fields=access_token&access_token=${userToken}`
  )
  return response.data.access_token
}
```

#### 2. "Publishing Limit Reached"
**Cause**: Exceeded posting rate limits
**Solution**: 
- Implement intelligent scheduling
- Spread posts across time
- Use post insights to optimize timing

#### 3. "Content Blocked"
**Cause**: Content violates Facebook policies
**Solution**:
- Review Community Standards
- Avoid spam-like behavior
- Use original, valuable content
- Test content before automation

#### 4. App Review Rejection
**Common Reasons**:
- Insufficient use case documentation
- Missing privacy policy
- Unclear business purpose
- Technical implementation issues

**Solutions**:
- Provide detailed use case scenarios
- Create comprehensive documentation
- Submit video demonstrations
- Ensure app follows best practices

### Debug Tools

#### Token Debugger
```bash
curl "https://graph.facebook.com/debug_token?
  input_token={token-to-debug}&
  access_token={app-token}"
```

#### API Explorer
Use Facebook's Graph API Explorer to:
- Test API calls
- Explore available fields
- Generate code samples
- Debug permissions

---

## Facebook vs Other Platforms

| Feature | Facebook | Instagram | LinkedIn | Twitter |
|---------|----------|-----------|----------|---------|
| Character Limit | 63,206 | 2,200 | 3,000 | 280 |
| Business Focus | ✅ Mixed | ❌ Visual | ✅ Professional | ❌ News |
| Visual Content | ✅ Important | ✅ Primary | ❌ Optional | ✅ Helpful |
| Community Features | ✅ Groups | ❌ Limited | ❌ Limited | ❌ Spaces |
| Ad Integration | ✅ Extensive | ✅ Extensive | ✅ Good | ✅ Good |
| App Review Required | ✅ Yes | ✅ Yes | ❌ Basic Only | ❌ No |

### When to Use Facebook
- ✅ Community building and engagement
- ✅ Visual content and storytelling
- ✅ Event promotion and management
- ✅ Customer support and communication
- ✅ Brand awareness campaigns
- ❌ Real-time news and updates
- ❌ Professional networking
- ❌ Technical developer content

---

## Implementation Roadmap

### Phase 1: Foundation (2 weeks)
- [ ] Create FacebookService class
- [ ] Implement OAuth 2.0 flow
- [ ] Add basic page posting
- [ ] Create Settings component
- [ ] Submit app for basic review

### Phase 2: Rich Features (2 weeks)
- [ ] Add image/video posting
- [ ] Implement scheduled posts
- [ ] Add Facebook Groups support
- [ ] Create workflow templates

### Phase 3: Advanced Integration (2 weeks)
- [ ] Add Facebook Insights analytics
- [ ] Implement comment management
- [ ] Add Instagram Business API
- [ ] Create advanced templates

### Phase 4: Enterprise Features (2 weeks)
- [ ] Add Facebook Ads API integration
- [ ] Implement multi-page management
- [ ] Add WhatsApp Business API
- [ ] Create enterprise templates

---

## Required Documentation for App Review

### 1. Use Case Documentation
```markdown
## DeFlow Facebook Integration Use Case

DeFlow is a workflow automation platform that helps users:
- Schedule and publish content to their business pages
- Automate customer engagement responses
- Share portfolio updates and business metrics
- Coordinate marketing campaigns across platforms

### Specific Use Cases:
1. **Business Page Management**: Auto-post daily updates
2. **Customer Engagement**: Automated responses to common questions  
3. **Marketing Automation**: Coordinate campaigns across platforms
4. **Analytics Sharing**: Post performance metrics and insights
```

### 2. Privacy Policy Requirements
- Data collection practices
- Token storage and security
- User consent mechanisms
- Data retention policies
- Third-party integrations

### 3. Permissions Justification

| Permission | Justification |
|------------|---------------|
| `pages_manage_posts` | Required to post content to user's business pages |
| `pages_read_engagement` | Needed to read post performance and optimize timing |
| `publish_to_groups` | Allow posting to business groups for community engagement |

---

This comprehensive guide provides everything needed to implement robust Facebook integration in DeFlow, following the same modular architecture principles established with Discord and Twitter.