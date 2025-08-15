import React, { useEffect, useState } from 'react'
import { useNavigate, useParams, useLocation } from 'react-router-dom'
import oauth2Service from '../services/oauth2Service'
import emailService from '../services/emailService'

const OAuthCallback: React.FC = () => {
  const [status, setStatus] = useState<'processing' | 'success' | 'error'>('processing')
  const [message, setMessage] = useState('')
  const navigate = useNavigate()
  const { provider } = useParams<{ provider: string }>()
  const location = useLocation()

  useEffect(() => {
    handleCallback()
  }, [])

  const handleCallback = async () => {
    try {
      const urlParams = new URLSearchParams(location.search)
      const code = urlParams.get('code')
      const error = urlParams.get('error')
      const state = urlParams.get('state')

      // Check for OAuth errors
      if (error) {
        throw new Error(`Authentication failed: ${error} - ${urlParams.get('error_description')}`)
      }

      if (!code) {
        throw new Error('No authorization code received')
      }

      if (!provider || (provider !== 'gmail' && provider !== 'outlook')) {
        throw new Error('Invalid provider')
      }

      // Validate state parameter
      const expectedState = sessionStorage.getItem(`oauth_state_${provider}`)
      if (expectedState && state !== expectedState) {
        throw new Error('Invalid state parameter - possible CSRF attack')
      }

      setMessage(`Exchanging authorization code for ${provider === 'gmail' ? 'Gmail' : 'Outlook'} tokens...`)

      // Exchange code for tokens
      const token = await oauth2Service.exchangeCodeForTokens(provider, code, state || undefined)

      setMessage('Configuring email service...')

      // Register with email service
      emailService.addProvider(`${provider}-oauth`, {
        provider,
        credentials: {
          access_token: token.access_token,
          refresh_token: token.refresh_token
        }
      })

      // Test the connection
      setMessage('Testing connection...')
      const isValid = await oauth2Service.testToken(provider)
      
      if (!isValid) {
        throw new Error('Token validation failed')
      }

      // Clean up state
      sessionStorage.removeItem(`oauth_state_${provider}`)

      setStatus('success')
      setMessage(`${provider === 'gmail' ? 'Gmail' : 'Outlook'} connected successfully! You can now send emails through this provider.`)

      // If this is a popup window, close it and notify parent
      if (window.opener) {
        window.opener.postMessage({ 
          type: 'oauth_success', 
          provider,
          message: 'Authentication successful'
        }, window.origin)
        window.close()
        return
      }

      // Otherwise redirect to settings after delay
      setTimeout(() => {
        navigate('/settings')
      }, 3000)

    } catch (error) {
      console.error('OAuth callback error:', error)
      setStatus('error')
      setMessage(error instanceof Error ? error.message : 'Authentication failed')

      // If this is a popup window, close it and notify parent
      if (window.opener) {
        window.opener.postMessage({ 
          type: 'oauth_error', 
          provider,
          error: error instanceof Error ? error.message : 'Authentication failed'
        }, window.origin)
        window.close()
        return
      }

      // Otherwise redirect to settings after delay
      setTimeout(() => {
        navigate('/settings')
      }, 5000)
    }
  }

  const getStatusIcon = () => {
    switch (status) {
      case 'processing': return '⏳'
      case 'success': return '✅'
      case 'error': return '❌'
    }
  }

  const getStatusColor = () => {
    switch (status) {
      case 'processing': return 'text-blue-600'
      case 'success': return 'text-green-600'
      case 'error': return 'text-red-600'
    }
  }

  const getProviderName = () => {
    if (provider === 'gmail') return 'Gmail'
    if (provider === 'outlook') return 'Outlook'
    return 'Email Provider'
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50">
      <div className="max-w-md w-full mx-4">
        <div className="bg-white rounded-lg shadow-lg p-8 text-center">
          <div className="mb-6">
            <div className="text-6xl mb-4">{getStatusIcon()}</div>
            <h1 className="text-2xl font-bold text-gray-900 mb-2">
              {getProviderName()} Authentication
            </h1>
            <p className={`text-lg ${getStatusColor()}`}>
              {status === 'processing' && 'Processing authentication...'}
              {status === 'success' && 'Authentication successful!'}
              {status === 'error' && 'Authentication failed'}
            </p>
          </div>

          <div className="mb-6">
            <div className={`p-4 rounded-lg ${
              status === 'processing' ? 'bg-blue-50 border border-blue-200' :
              status === 'success' ? 'bg-green-50 border border-green-200' :
              'bg-red-50 border border-red-200'
            }`}>
              <p className="text-sm text-gray-700">{message}</p>
            </div>
          </div>

          {status === 'processing' && (
            <div className="flex justify-center">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
            </div>
          )}

          {status === 'success' && (
            <div className="space-y-3">
              <div className="text-sm text-gray-600">
                {window.opener ? (
                  <p>This window will close automatically...</p>
                ) : (
                  <p>Redirecting to settings in a moment...</p>
                )}
              </div>
              {!window.opener && (
                <button
                  onClick={() => navigate('/settings')}
                  className="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
                >
                  Go to Settings Now
                </button>
              )}
            </div>
          )}

          {status === 'error' && (
            <div className="space-y-3">
              <div className="text-sm text-gray-600">
                {window.opener ? (
                  <p>This window will close automatically...</p>
                ) : (
                  <p>You can try again from the settings page.</p>
                )}
              </div>
              {!window.opener && (
                <div className="flex space-x-3 justify-center">
                  <button
                    onClick={() => navigate('/settings')}
                    className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                  >
                    Back to Settings
                  </button>
                  <button
                    onClick={() => window.location.reload()}
                    className="px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors"
                  >
                    Try Again
                  </button>
                </div>
              )}
            </div>
          )}

          {/* Debug Info (only in development) */}
          {process.env.NODE_ENV === 'development' && (
            <div className="mt-6 pt-4 border-t border-gray-200">
              <details className="text-left">
                <summary className="text-sm text-gray-500 cursor-pointer">Debug Info</summary>
                <div className="mt-2 text-xs text-gray-400 space-y-1">
                  <p>Provider: {provider}</p>
                  <p>Status: {status}</p>
                  <p>Is Popup: {window.opener ? 'Yes' : 'No'}</p>
                  <p>URL: {window.location.href}</p>
                </div>
              </details>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

export default OAuthCallback