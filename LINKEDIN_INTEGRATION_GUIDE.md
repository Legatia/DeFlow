# LinkedIn Integration Guide for DeFlow

This guide covers implementing and using LinkedIn integration in DeFlow workflows for professional networking, content sharing, and business automation.

## Table of Contents

1. [Developer Implementation Guide](#developer-implementation-guide)
2. [User Setup Guide](#user-setup-guide)
3. [API Reference](#api-reference)
4. [Workflow Examples](#workflow-examples)
5. [Best Practices](#best-practices)
6. [Troubleshooting](#troubleshooting)

---

## Developer Implementation Guide

### 1. LinkedIn API Overview

LinkedIn provides several APIs for different use cases:

- **LinkedIn API v2**: Primary API for posting content, managing profiles
- **Marketing Developer Platform**: For advertising and analytics
- **Partner Program**: For advanced integrations

### 2. Authentication Methods

#### OAuth 2.0 Flow (Recommended)
```typescript
interface LinkedInOAuthConfig {
  client_id: string
  client_secret: string
  redirect_uri: string
  scope: string[]
}

// Required scopes for basic posting
const DEFAULT_SCOPES = [
  'r_liteprofile',
  'r_emailaddress', 
  'w_member_social'
]
```

#### Access Token Management
```typescript
interface LinkedInCredentials {
  access_token: string
  token_type: 'Bearer'
  expires_in: number
  refresh_token?: string
}
```

### 3. Node Architecture Design

Following DeFlow's modular approach, implement these nodes:

#### A. LinkedIn API Sender Node
```typescript
{
  id: 'linkedin-post',
  name: 'LinkedIn Post',
  description: 'Post content to LinkedIn - accepts text or JSON message data',
  category: 'integrations',
  icon: '💼',
  color: '#0077B5',
  inputs: [
    { id: 'message', name: 'Message Data', type: 'data', required: true }
  ],
  outputs: [
    { id: 'result', name: 'LinkedIn Result', type: 'data', required: true }
  ],
  configSchema: [
    {
      key: 'access_token',
      name: 'Access Token',
      type: 'password',
      required: true,
      description: 'LinkedIn OAuth 2.0 access token'
    },
    {
      key: 'post_type',
      name: 'Post Type',
      type: 'select',
      options: [
        { label: 'Personal Post', value: 'person' },
        { label: 'Company Page Post', value: 'organization' }
      ],
      defaultValue: 'person'
    },
    {
      key: 'organization_id',
      name: 'Organization ID',
      type: 'text',
      required: false,
      description: 'LinkedIn organization ID (for company posts)'
    }
  ]
}
```

#### B. Reuse Existing Social Media Formatters
The existing `social-media-text` and `social-media-with-image` nodes work perfectly for LinkedIn:

```typescript
// Platform-specific formatting
{
  key: 'platform',
  options: [
    { label: 'Twitter/X (280 chars)', value: 'twitter' },
    { label: 'Discord (2000 chars)', value: 'discord' },
    { label: 'LinkedIn (3000 chars)', value: 'linkedin' }, // Add this
    { label: 'General (no limit)', value: 'general' }
  ]
}
```

### 4. Service Implementation

#### LinkedIn Service Structure
```typescript
// src/services/linkedinService.ts
export interface LinkedInPostConfig {
  text: string
  media_urls?: string[]
  article_url?: string
  hashtags?: string[]
  mentions?: string[]
}

export interface LinkedInResponse {
  id?: string
  created_at?: string
  urn?: string
  error?: {
    code: number
    message: string
  }
}

class LinkedInService {
  async postToLinkedIn(
    accessToken: string,
    config: LinkedInPostConfig,
    templateVariables: Record<string, any> = {}
  ): Promise<LinkedInResponse>

  async postToCompanyPage(
    accessToken: string,
    organizationId: string,
    config: LinkedInPostConfig,
    templateVariables: Record<string, any> = {}
  ): Promise<LinkedInResponse>

  async validateToken(accessToken: string): Promise<{ valid: boolean; error?: string }>
}
```

#### Key API Endpoints
```typescript
const LINKEDIN_ENDPOINTS = {
  POST_PERSON: 'https://api.linkedin.com/v2/ugcPosts',
  POST_ORGANIZATION: 'https://api.linkedin.com/v2/ugcPosts',
  PROFILE: 'https://api.linkedin.com/v2/people/~',
  MEDIA_UPLOAD: 'https://api.linkedin.com/v2/assets?action=registerUpload'
}
```

### 5. Rate Limiting

LinkedIn API limits:
- **Personal Posts**: 100 posts per day
- **Company Posts**: 25 posts per day per organization
- **Profile API**: 500 requests per day

```typescript
checkRateLimit(endpoint: string, isCompany: boolean = false): boolean {
  const limits = {
    'ugcPosts_person': 100,     // per day
    'ugcPosts_company': 25,     // per day per org
    'profile': 500              // per day
  }
  // Implementation...
}
```

### 6. Media Handling

LinkedIn supports various media types:
```typescript
interface LinkedInMedia {
  type: 'IMAGE' | 'VIDEO' | 'ARTICLE'
  url: string
  title?: string
  description?: string
}
```

---

## User Setup Guide

### 1. LinkedIn Developer App Setup

#### Step 1: Create LinkedIn App
1. Go to [LinkedIn Developer Portal](https://developer.linkedin.com/)
2. Sign in with your LinkedIn account
3. Click "Create App"
4. Fill in app details:
   - **App name**: "DeFlow Integration"
   - **LinkedIn Page**: Your company page (or personal)
   - **Privacy policy URL**: Your privacy policy
   - **App logo**: Upload a logo (optional)

#### Step 2: Configure App Settings
1. Go to "Products" tab
2. Request access to:
   - **Share on LinkedIn** (for posting)
   - **Sign In with LinkedIn** (for authentication)
3. Wait for approval (usually instant for basic features)

#### Step 3: Get API Credentials
1. Go to "Auth" tab
2. Copy your **Client ID** and **Client Secret**
3. Add redirect URL: `http://localhost:3000/auth/linkedin/callback`

### 2. DeFlow Configuration

#### Step 1: Add to Settings
1. Open DeFlow Settings page
2. Go to "External Integrations" section
3. Click "Add LinkedIn Configuration"
4. Enter your Client ID and Client Secret

#### Step 2: Authorize Account
1. Click "Connect LinkedIn Account"
2. Sign in to LinkedIn when prompted
3. Grant permissions to DeFlow
4. Verify connection shows "✅ Connected"

### 3. Creating LinkedIn Workflows

#### Basic Portfolio Update
```
Schedule Trigger → Social Media Text → LinkedIn Post
```

**Configuration:**
- **Social Media Text**: Set platform to "LinkedIn (3000 chars)"
- **Content**: Professional tone with business hashtags
- **LinkedIn Post**: Select "Personal Post" or "Company Page Post"

#### Professional Article Sharing
```
RSS Feed → Social Media Text → LinkedIn Post
```

**Use Case**: Automatically share your blog posts or industry articles

#### Company Updates
```
Webhook → Social Media with Image → LinkedIn Post
```

**Use Case**: Share company milestones, product updates, partnerships

---

## API Reference

### Authentication Scopes

| Scope | Description | Use Case |
|-------|-------------|----------|
| `r_liteprofile` | Basic profile info | User identification |
| `r_emailaddress` | Email address | Account linking |
| `w_member_social` | Post on behalf of user | Personal posts |
| `w_organization_social` | Post to company pages | Business posts |

### Post Types

#### Text Post
```json
{
  "author": "urn:li:person:123456789",
  "lifecycleState": "PUBLISHED",
  "specificContent": {
    "com.linkedin.ugc.ShareContent": {
      "shareCommentary": {
        "text": "Your post content here #hashtags"
      },
      "shareMediaCategory": "NONE"
    }
  }
}
```

#### Image Post
```json
{
  "author": "urn:li:person:123456789",
  "lifecycleState": "PUBLISHED",
  "specificContent": {
    "com.linkedin.ugc.ShareContent": {
      "shareCommentary": {
        "text": "Post with image"
      },
      "shareMediaCategory": "IMAGE",
      "media": [
        {
          "status": "READY",
          "media": "urn:li:digitalmediaAsset:123456789"
        }
      ]
    }
  }
}
```

### Error Codes

| Code | Meaning | Solution |
|------|---------|----------|
| 401 | Unauthorized | Check access token validity |
| 403 | Forbidden | Verify app permissions and scopes |
| 422 | Invalid content | Check post format and length |
| 429 | Rate limited | Implement exponential backoff |

---

## Workflow Examples

### 1. Daily Professional Update
**Use Case**: Share daily DeFi market insights professionally

**Nodes**:
1. **Schedule Trigger**: Daily at 9 AM
2. **Social Media Text**: 
   ```
   📊 DeFi Market Update - {{date}}
   
   Key insights from today's analysis:
   • Total Value Locked: ${{tvl}}
   • Top performing protocol: {{top_protocol}}
   • Market sentiment: {{sentiment}}
   
   What trends are you watching? 
   
   #DeFi #Blockchain #Finance #MarketAnalysis
   ```
3. **LinkedIn Post**: Personal post

### 2. Company Product Updates
**Use Case**: Announce new DeFlow features to company network

**Nodes**:
1. **Webhook Trigger**: Manual trigger for announcements
2. **Social Media with Image**:
   ```
   🚀 Exciting DeFlow Update!
   
   {{feature_name}} is now live!
   
   {{feature_description}}
   
   Try it now: {{feature_url}}
   
   #DeFlow #ProductUpdate #Innovation
   ```
3. **LinkedIn Post**: Company page post

### 3. Thought Leadership Content
**Use Case**: Share industry insights and establish thought leadership

**Nodes**:
1. **RSS Feed Trigger**: Monitor industry news
2. **AI Content Generator**: Create insightful commentary
3. **Social Media Text**: Format for LinkedIn
4. **LinkedIn Post**: Personal post

---

## Best Practices

### Content Guidelines

#### Professional Tone
- Use business-appropriate language
- Include relevant hashtags (max 3-5)
- Add value to professional network
- Credit sources and collaborators

#### Character Limits
- **LinkedIn Posts**: 3,000 characters max
- **Headlines**: Keep under 150 characters
- **Summaries**: 200-300 characters optimal

### Posting Strategy

#### Timing
- **Best times**: Tuesday-Thursday, 8-10 AM and 12-2 PM
- **Avoid**: Weekends and late evenings
- **Frequency**: 1-2 posts per day maximum

#### Content Mix
- 40% Industry insights and trends
- 30% Company/personal updates
- 20% Educational content
- 10% Promotional content

### Compliance

#### LinkedIn Terms
- No automated bulk messaging
- Respect connection limits
- Don't scrape user data
- Follow community guidelines

#### Professional Standards
- Disclose automation when required
- Maintain authentic voice
- Respect audience preferences
- Follow financial advice regulations for DeFi content

---

## Troubleshooting

### Common Issues

#### 1. "Invalid Access Token" Error
**Cause**: Token expired or invalid
**Solution**: 
- Refresh OAuth token
- Re-authenticate in DeFlow Settings
- Check token scopes

#### 2. "Insufficient Permissions" Error
**Cause**: Missing required scopes
**Solution**:
- Update app permissions in LinkedIn Developer Portal
- Re-authorize with new scopes
- Wait for LinkedIn approval if required

#### 3. Rate Limiting
**Cause**: Too many API calls
**Solution**:
- Implement exponential backoff
- Reduce posting frequency
- Use scheduling to spread posts

#### 4. Content Rejected
**Cause**: Violates LinkedIn policies
**Solution**:
- Review content guidelines
- Avoid spam-like behavior
- Use authentic, valuable content

### Debug Tools

#### Test OAuth Flow
```bash
curl -X POST https://api.linkedin.com/v2/people/~ \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json"
```

#### Validate Post Format
```javascript
// Use LinkedIn's content validation
const validateLinkedInPost = (postData) => {
  // Check character limits
  // Validate media URLs
  // Verify required fields
}
```

### Support Resources

- [LinkedIn API Documentation](https://docs.microsoft.com/en-us/linkedin/)
- [LinkedIn Developer Community](https://www.linkedin.com/groups/3722681/)
- [OAuth 2.0 Specification](https://tools.ietf.org/html/rfc6749)
- [LinkedIn Business Guidelines](https://www.linkedin.com/help/linkedin/answer/129562)

---

## Implementation Timeline

### Phase 1: Basic Integration (Week 1)
- [ ] Create LinkedInService class
- [ ] Implement OAuth 2.0 flow
- [ ] Add basic text posting
- [ ] Create Settings component

### Phase 2: Rich Content (Week 2)  
- [ ] Add image/media support
- [ ] Implement article sharing
- [ ] Add company page posting
- [ ] Create workflow templates

### Phase 3: Advanced Features (Week 3)
- [ ] Add analytics integration
- [ ] Implement connection management
- [ ] Add direct messaging (if approved)
- [ ] Create advanced templates

### Phase 4: Professional Features (Week 4)
- [ ] Add Sales Navigator integration
- [ ] Implement lead generation workflows
- [ ] Add recruitment posting
- [ ] Create B2B automation templates

---

## Security Considerations

### Token Storage
- Store access tokens securely in encrypted local storage
- Implement token refresh mechanism
- Never log or expose tokens in client-side code

### Rate Limiting
- Implement client-side rate limiting
- Use exponential backoff for retries
- Monitor API usage to avoid limits

### Content Validation
- Sanitize user input
- Validate URLs and media
- Check content length limits
- Filter inappropriate content

### Privacy
- Respect user privacy settings
- Don't access private profile data
- Follow GDPR and privacy regulations
- Implement data retention policies

---

## LinkedIn vs Other Platforms

| Feature | LinkedIn | Twitter | Discord |
|---------|----------|---------|---------|
| Character Limit | 3,000 | 280 | 2,000 |
| Media Support | Images, Videos, Articles | Images, Videos, GIFs | Images, Videos, Files |
| Professional Focus | ✅ High | ❌ Mixed | ❌ Gaming/Tech |
| B2B Audience | ✅ Primary | ❌ Mixed | ❌ Communities |
| Rate Limits | Moderate | Strict | Generous |
| Setup Complexity | Medium | High | Low |

### When to Use LinkedIn
- ✅ Professional announcements
- ✅ Industry insights and analysis  
- ✅ Company updates and milestones
- ✅ Thought leadership content
- ✅ B2B networking and lead generation
- ❌ Casual conversations
- ❌ Real-time trading signals
- ❌ Community gaming content

---

## Example Implementation

### linkedinService.ts
```typescript
export class LinkedInService {
  async postContent(
    accessToken: string,
    config: LinkedInPostConfig,
    templateVariables: Record<string, any> = {}
  ): Promise<LinkedInResponse> {
    
    // Process template variables
    const processedText = this.processTemplate(config.text, templateVariables)
    
    // Build LinkedIn post payload
    const payload = {
      author: config.post_type === 'organization' 
        ? `urn:li:organization:${config.organization_id}`
        : await this.getUserUrn(accessToken),
      lifecycleState: 'PUBLISHED',
      specificContent: {
        'com.linkedin.ugc.ShareContent': {
          shareCommentary: {
            text: processedText
          },
          shareMediaCategory: config.media_urls?.length > 0 ? 'IMAGE' : 'NONE'
        }
      }
    }

    return this.makeLinkedInRequest('POST', '/v2/ugcPosts', accessToken, payload)
  }
}
```

### Settings Component
```typescript
// LinkedInAPISetup.tsx
const LinkedInAPISetup: React.FC = () => {
  const handleOAuthFlow = async () => {
    const authUrl = `https://www.linkedin.com/oauth/v2/authorization?` +
      `response_type=code&` +
      `client_id=${clientId}&` +
      `redirect_uri=${redirectUri}&` +
      `scope=${scopes.join(' ')}`
    
    window.open(authUrl, '_blank')
  }

  return (
    <div className="space-y-6">
      <h3>LinkedIn API Integration</h3>
      <button onClick={handleOAuthFlow}>
        Connect LinkedIn Account
      </button>
    </div>
  )
}
```

---

This guide provides a complete foundation for implementing LinkedIn integration in DeFlow, following the same modular architecture pattern established with Discord and Twitter integrations.