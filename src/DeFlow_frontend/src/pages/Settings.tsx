import { useState, useEffect } from 'react'
import { simpleService } from '../services/simpleService'

const Settings = () => {
  const [isAuthenticated, setIsAuthenticated] = useState(false)
  const [identity, setIdentity] = useState<any>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [greeting, setGreeting] = useState('')

  useEffect(() => {
    checkAuthStatus()
  }, [])

  const checkAuthStatus = async () => {
    try {
      const authenticated = await simpleService.isAuthenticated()
      setIsAuthenticated(authenticated)
      
      if (authenticated) {
        const userIdentity = await simpleService.getIdentity()
        setIdentity(userIdentity)
      }
    } catch (error) {
      console.error('Failed to check auth status:', error)
    } finally {
      setIsLoading(false)
    }
  }

  const handleLogin = async () => {
    try {
      setIsLoading(true)
      const success = await simpleService.login()
      if (success) {
        await checkAuthStatus()
      }
    } catch (error) {
      console.error('Login failed:', error)
      alert('Login failed. Please try again.')
    } finally {
      setIsLoading(false)
    }
  }

  const handleLogout = async () => {
    try {
      setIsLoading(true)
      await simpleService.logout()
      setIsAuthenticated(false)
      setIdentity(null)
      setGreeting('')
    } catch (error) {
      console.error('Logout failed:', error)
    } finally {
      setIsLoading(false)
    }
  }

  const testGreeting = async () => {
    try {
      const response = await simpleService.greet('DeFlow User')
      setGreeting(response)
    } catch (error) {
      console.error('Greeting test failed:', error)
      setGreeting('Failed to connect to backend')
    }
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>
    )
  }

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      <h1 className="text-2xl font-bold text-gray-900">Settings</h1>

      {/* Authentication Section */}
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-medium text-gray-900 mb-4">Authentication</h2>
        
        <div className="space-y-4">
          <div className="flex items-center justify-between p-4 border rounded-lg">
            <div>
              <h3 className="font-medium text-gray-900">Internet Identity</h3>
              <p className="text-sm text-gray-600">
                {isAuthenticated ? 'Connected to Internet Identity' : 'Not connected'}
              </p>
              {identity && (
                <p className="text-xs text-gray-500 mt-1">
                  Principal: {identity.getPrincipal().toString().slice(0, 20)}...
                </p>
              )}
            </div>
            <div>
              {isAuthenticated ? (
                <button
                  onClick={handleLogout}
                  className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
                >
                  Disconnect
                </button>
              ) : (
                <button
                  onClick={handleLogin}
                  className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                >
                  Connect
                </button>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Backend Connection Test */}
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-medium text-gray-900 mb-4">Backend Connection</h2>
        
        <div className="space-y-4">
          <div className="flex items-center justify-between p-4 border rounded-lg">
            <div>
              <h3 className="font-medium text-gray-900">Canister Status</h3>
              <p className="text-sm text-gray-600">Test connection to DeFlow backend</p>
              {greeting && (
                <p className="text-sm text-green-600 mt-1">{greeting}</p>
              )}
            </div>
            <button
              onClick={testGreeting}
              className="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
            >
              Test Connection
            </button>
          </div>
        </div>
      </div>

      {/* Application Info */}
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-medium text-gray-900 mb-4">Application Information</h2>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="p-4 border rounded-lg">
            <h3 className="font-medium text-gray-900">Version</h3>
            <p className="text-sm text-gray-600">DeFlow Frontend v0.1.0</p>
          </div>
          
          <div className="p-4 border rounded-lg">
            <h3 className="font-medium text-gray-900">Network</h3>
            <p className="text-sm text-gray-600">
              {window.location.hostname === 'localhost' ? 'Local Development' : 'Internet Computer'}
            </p>
          </div>
          
          <div className="p-4 border rounded-lg">
            <h3 className="font-medium text-gray-900">Build</h3>
            <p className="text-sm text-gray-600">React + TypeScript + Vite</p>
          </div>
          
          <div className="p-4 border rounded-lg">
            <h3 className="font-medium text-gray-900">ICP Integration</h3>
            <p className="text-sm text-gray-600">@dfinity/agent v2.1.3</p>
          </div>
        </div>
      </div>

      {/* Debug Information */}
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-medium text-gray-900 mb-4">Debug Information</h2>
        
        <div className="space-y-2 text-sm">
          <div className="flex justify-between">
            <span className="text-gray-600">User Agent:</span>
            <span className="text-gray-900 font-mono text-xs">{navigator.userAgent.slice(0, 50)}...</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-600">BigInt Support:</span>
            <span className="text-gray-900">{typeof BigInt !== 'undefined' ? '✅ Available' : '❌ Not Available'}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-600">WebAssembly:</span>
            <span className="text-gray-900">{typeof WebAssembly !== 'undefined' ? '✅ Available' : '❌ Not Available'}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-600">Local Storage:</span>
            <span className="text-gray-900">{typeof localStorage !== 'undefined' ? '✅ Available' : '❌ Not Available'}</span>
          </div>
        </div>
      </div>
    </div>
  )
}

export default Settings