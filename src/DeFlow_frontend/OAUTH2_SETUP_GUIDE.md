# OAuth2 Setup Guide for DeFlow

This guide shows you how to set up OAuth2 authentication for Gmail and Outlook email integration in DeFlow.

## Overview

DeFlow now supports OAuth2 authentication for:
- **Gmail API** (Google Workspace)
- **Outlook API** (Microsoft 365)

This provides secure, token-based authentication without storing passwords.

## Gmail/Google Setup

### 1. Create Google Cloud Project

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select existing one
3. Name it something like "DeFlow Email Integration"

### 2. Enable Gmail API

1. Go to **APIs & Services** → **Library**
2. Search for "Gmail API"
3. Click **Enable**

### 3. Configure OAuth Consent Screen

1. Go to **APIs & Services** → **OAuth consent screen**
2. Choose **External** (unless using Google Workspace)
3. Fill required fields:
   - App name: "DeFlow"
   - User support email: your email
   - Developer contact: your email
4. Add scopes:
   - `https://www.googleapis.com/auth/gmail.send`
   - `https://www.googleapis.com/auth/gmail.readonly`
5. Add test users (your email) if app is not published

### 4. Create OAuth2 Credentials

1. Go to **APIs & Services** → **Credentials**
2. Click **Create Credentials** → **OAuth 2.0 Client IDs**
3. Application type: **Web application**
4. Name: "DeFlow Web Client"
5. **Authorized redirect URIs**:
   ```
   http://localhost:4943/oauth/callback/gmail
   https://your-canister-id.ic0.app/oauth/callback/gmail
   ```
6. Save **Client ID** and **Client Secret**

## Outlook/Microsoft Setup

### 1. Register Application

1. Go to [Azure Portal](https://portal.azure.com/)
2. Navigate to **Azure Active Directory** → **App registrations**
3. Click **New registration**
4. Fill details:
   - Name: "DeFlow Email Integration"
   - Supported account types: **Accounts in any organizational directory (Any Azure AD directory - Multitenant) and personal Microsoft accounts**
   - Redirect URI: **Web** → `http://localhost:4943/oauth/callback/outlook`

### 2. Add Redirect URIs

1. Go to **Authentication**
2. Add redirect URIs:
   ```
   http://localhost:4943/oauth/callback/outlook
   https://your-canister-id.ic0.app/oauth/callback/outlook
   ```
3. Enable **Access tokens** and **ID tokens**

### 3. Create Client Secret

1. Go to **Certificates & secrets**
2. Click **New client secret**
3. Description: "DeFlow OAuth Secret"
4. Expires: **24 months**
5. Save the **Value** (not the ID)

### 4. Configure API Permissions

1. Go to **API permissions**
2. Click **Add a permission** → **Microsoft Graph**
3. Choose **Delegated permissions**
4. Add permissions:
   - `Mail.Send`
   - `Mail.Read`
   - `User.Read`
5. Click **Grant admin consent** (if you're admin)

## DeFlow Configuration

### 1. Access Settings

1. Open DeFlow application
2. Go to **Settings** → **External Integrations**
3. Find **OAuth2 Providers** section
4. Click **Configure**

### 2. Configure Gmail

1. Enable **Gmail API Configuration**
2. Enter **Google Client ID** (from step 4 above)
3. Enter **Google Client Secret** (from step 4 above)
4. Save configuration

### 3. Configure Outlook

1. Enable **Outlook API Configuration**
2. Enter **Microsoft Application (Client) ID** (from App registration overview)
3. Enter **Microsoft Client Secret** (from step 3 above)
4. Save configuration

### 4. Connect Accounts

1. Click **Connect Gmail** or **Connect Outlook**
2. Complete OAuth flow in popup window
3. Grant permissions when prompted
4. Test connection to verify

## Using OAuth2 Providers

### In Workflows

```typescript
// Send email via OAuth2 provider
await emailService.sendEmail('gmail-oauth', {
  to: 'user@example.com',
  subject: 'Automated Alert',
  body_html: '<h1>Your portfolio gained 5%!</h1>'
})

await emailService.sendEmail('outlook-oauth', {
  to: 'team@company.com',
  subject: 'Daily Report',
  body_html: '<p>Today\'s metrics...</p>'
})
```

### Provider Names

After connecting, use these provider names:
- Gmail: `gmail-oauth`
- Outlook: `outlook-oauth`

## Security Features

### Token Management
- **Automatic refresh**: Tokens refresh automatically before expiry
- **Secure storage**: Tokens encrypted in browser localStorage
- **Scope limitation**: Only email sending/reading permissions
- **Revocation**: Users can disconnect anytime

### Best Practices
- **Rotate secrets**: Change client secrets periodically
- **Monitor usage**: Check API quotas regularly
- **Test permissions**: Verify minimal required scopes
- **Update redirects**: Keep redirect URIs current

## Troubleshooting

### Common Issues

**"redirect_uri_mismatch"**
- Check redirect URI matches exactly in provider settings
- Include both localhost and production URLs

**"invalid_client"**
- Verify Client ID and Secret are correct
- Check they're from the same OAuth application

**"insufficient_scope"**
- Ensure required scopes are granted
- Re-authenticate if scopes changed

**"token_expired"**
- DeFlow automatically refreshes tokens
- Reconnect if refresh token is invalid

### Testing

1. **Connection Test**: Use "Test Connection" button in settings
2. **Send Test Email**: Use "Send Test" for actual email sending
3. **Check Logs**: Browser console shows detailed error messages

### Development vs Production

**Development** (localhost):
```
http://localhost:4943/oauth/callback/gmail
http://localhost:4943/oauth/callback/outlook
```

**Production** (ICP):
```
https://your-canister-id.ic0.app/oauth/callback/gmail
https://your-canister-id.ic0.app/oauth/callback/outlook
```

## Rate Limits

### Gmail API
- **Daily quota**: 1 billion quota units
- **Requests per second**: ~250 requests/second
- **Messages per day**: ~250,000 emails

### Microsoft Graph
- **Requests per second**: 10,000 requests per 10 minutes
- **Messages per day**: No specific limit
- **Throttling**: Automatic retry with backoff

## Support

For issues:
1. Check browser console for detailed errors
2. Verify OAuth2 configuration in provider settings
3. Test with minimal example email
4. Check API status pages for service outages

---

*This OAuth2 integration provides enterprise-grade security for email automation while maintaining user control over their accounts.*