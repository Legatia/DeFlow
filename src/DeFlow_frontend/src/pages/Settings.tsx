// Import BigInt polyfill FIRST to prevent conversion errors
import '../utils/bigint-polyfill'

import React, { useState, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { useEnhancedAuth } from '../contexts/EnhancedAuthContext'
import localCacheService, { UserPreferences } from '../services/localCacheService'

const Settings = () => {
  const navigate = useNavigate()
  const auth = useEnhancedAuth()
  const [preferences, setPreferences] = useState<UserPreferences>(localCacheService.getUserPreferences())
  const [cacheSize, setCacheSize] = useState('0 Bytes')

  useEffect(() => {
    setCacheSize(auth.getCacheSize())
  }, [auth])

  const handlePreferenceChange = (key: keyof UserPreferences, value: any) => {
    const updatedPreferences = { ...preferences, [key]: value }
    setPreferences(updatedPreferences)
    localCacheService.saveUserPreferences(updatedPreferences)
  }

  const handleNotificationChange = (key: string, value: boolean) => {
    const updatedNotifications = { ...preferences.notifications, [key]: value }
    handlePreferenceChange('notifications', updatedNotifications)
  }

  const handleExportData = () => {
    const data = auth.exportLocalData()
    const blob = new Blob([data], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = `deflow-data-${new Date().toISOString().split('T')[0]}.json`
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    URL.revokeObjectURL(url)
  }

  const handleClearCache = () => {
    if (confirm('Clear all local data? This will remove workflows, executions, and preferences. This action cannot be undone.')) {
      auth.clearLocalData()
      setCacheSize('0 Bytes')
      setPreferences(localCacheService.getUserPreferences()) // Reload defaults
    }
  }

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      <h1 className="text-2xl font-bold text-gray-900">Settings</h1>

      {/* Account & Subscription Section */}
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-medium text-gray-900 mb-4">Account & Subscription</h2>
        
        <div className="space-y-4">
          {/* Authentication Status */}
          <div className="flex items-center justify-between p-4 border rounded-lg">
            <div>
              <h3 className="font-medium text-gray-900">
                {auth.isAuthenticated ? `${auth.authMethod === 'nfid' ? 'Google (NFID)' : 'Internet Identity'}` : 'Not Authenticated'}
              </h3>
              <p className="text-sm text-gray-600">
                {auth.isAuthenticated 
                  ? `Connected • ${auth.userMode === 'authenticated' ? 'Premium User' : 'Guest User'}` 
                  : 'Login to access premium features'}
              </p>
              {auth.principal && (
                <p className="text-xs text-gray-500 mt-1">
                  Principal: {auth.principal.toString().slice(0, 30)}...
                </p>
              )}
            </div>
            <div className="flex space-x-2">
              {auth.isAuthenticated ? (
                <>
                  <button
                    onClick={() => navigate('/premium')}
                    className="px-4 py-2 bg-gradient-to-r from-purple-600 to-blue-600 text-white rounded-lg hover:from-purple-700 hover:to-blue-700 transition-colors flex items-center space-x-2"
                  >
                    <span>⭐</span>
                    <span>Manage Subscription</span>
                  </button>
                  <button
                    onClick={() => auth.logout()}
                    className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
                  >
                    Logout
                  </button>
                </>
              ) : (
                <button
                  onClick={() => navigate('/premium')}
                  className="px-4 py-2 bg-gradient-to-r from-purple-600 to-blue-600 text-white rounded-lg hover:from-purple-700 hover:to-blue-700 transition-colors"
                >
                  Upgrade to Premium
                </button>
              )}
            </div>
          </div>

          {/* Subscription Benefits */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="p-4 border rounded-lg">
              <div className="flex items-center justify-between mb-2">
                <h3 className="font-medium text-gray-900">Unlimited Workflows</h3>
                <span className={auth.subscriptionBenefits.unlimitedWorkflows ? 'text-green-600' : 'text-gray-400'}>
                  {auth.subscriptionBenefits.unlimitedWorkflows ? '✅' : '❌'}
                </span>
              </div>
              <p className="text-sm text-gray-600">Create unlimited automation workflows</p>
            </div>

            <div className="p-4 border rounded-lg">
              <div className="flex items-center justify-between mb-2">
                <h3 className="font-medium text-gray-900">Reduced Fees</h3>
                <span className={auth.subscriptionBenefits.reducedFees ? 'text-green-600' : 'text-gray-400'}>
                  {auth.subscriptionBenefits.reducedFees ? '✅' : '❌'}
                </span>
              </div>
              <p className="text-sm text-gray-600">Save 50% on transaction fees</p>
            </div>

            <div className="p-4 border rounded-lg">
              <div className="flex items-center justify-between mb-2">
                <h3 className="font-medium text-gray-900">Cross-Device Sync</h3>
                <span className={auth.subscriptionBenefits.crossDeviceSync ? 'text-green-600' : 'text-gray-400'}>
                  {auth.subscriptionBenefits.crossDeviceSync ? '✅' : '❌'}
                </span>
              </div>
              <p className="text-sm text-gray-600">Access workflows from any device</p>
            </div>

            <div className="p-4 border rounded-lg">
              <div className="flex items-center justify-between mb-2">
                <h3 className="font-medium text-gray-900">Priority Execution</h3>
                <span className={auth.subscriptionBenefits.priorityExecution ? 'text-green-600' : 'text-gray-400'}>
                  {auth.subscriptionBenefits.priorityExecution ? '✅' : '❌'}
                </span>
              </div>
              <p className="text-sm text-gray-600">Your workflows run first in queue</p>
            </div>
          </div>
        </div>
      </div>

      {/* Notification Preferences */}
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-medium text-gray-900 mb-4">Notifications</h2>
        
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="font-medium text-gray-900">Browser Notifications</h3>
              <p className="text-sm text-gray-600">Show desktop notifications</p>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={preferences.notifications.browser}
                onChange={(e) => handleNotificationChange('browser', e.target.checked)}
                className="sr-only peer"
              />
              <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
            </label>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <h3 className="font-medium text-gray-900">Sound Alerts</h3>
              <p className="text-sm text-gray-600">Play sound for notifications</p>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={preferences.notifications.sound}
                onChange={(e) => handleNotificationChange('sound', e.target.checked)}
                className="sr-only peer"
              />
              <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
            </label>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <h3 className="font-medium text-gray-900">Workflow Completions</h3>
              <p className="text-sm text-gray-600">Notify when workflows finish</p>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={preferences.notifications.workflowComplete}
                onChange={(e) => handleNotificationChange('workflowComplete', e.target.checked)}
                className="sr-only peer"
              />
              <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
            </label>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <h3 className="font-medium text-gray-900">Portfolio Alerts</h3>
              <p className="text-sm text-gray-600">Notify about portfolio changes</p>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={preferences.notifications.portfolioAlerts}
                onChange={(e) => handleNotificationChange('portfolioAlerts', e.target.checked)}
                className="sr-only peer"
              />
              <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
            </label>
          </div>
        </div>
      </div>

      {/* Data Management */}
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-medium text-gray-900 mb-4">Data Management</h2>
        
        <div className="space-y-4">
          <div className="flex items-center justify-between p-4 border rounded-lg">
            <div>
              <h3 className="font-medium text-gray-900">Local Storage</h3>
              <p className="text-sm text-gray-600">Cache size: {cacheSize}</p>
            </div>
            <div className="space-x-2">
              <button
                onClick={handleExportData}
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
              >
                Export Data
              </button>
              <button
                onClick={handleClearCache}
                className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
              >
                Clear Cache
              </button>
            </div>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <h3 className="font-medium text-gray-900">Auto-save</h3>
              <p className="text-sm text-gray-600">Automatically save workflow changes</p>
            </div>
            <label className="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={preferences.autoSave}
                onChange={(e) => handlePreferenceChange('autoSave', e.target.checked)}
                className="sr-only peer"
              />
              <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
            </label>
          </div>
        </div>
      </div>

      {/* Application Information */}
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-medium text-gray-900 mb-4">Application Information</h2>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="p-4 border rounded-lg">
            <h3 className="font-medium text-gray-900">Version</h3>
            <p className="text-sm text-gray-600">DeFlow v0.1.0</p>
          </div>
          
          <div className="p-4 border rounded-lg">
            <h3 className="font-medium text-gray-900">Network</h3>
            <p className="text-sm text-gray-600">
              {window.location.hostname === 'localhost' ? 'Local Development' : 'Internet Computer'}
            </p>
          </div>
          
          <div className="p-4 border rounded-lg">
            <h3 className="font-medium text-gray-900">Auth Method</h3>
            <p className="text-sm text-gray-600">
              {auth.authMethod ? auth.authMethod.replace('-', ' ').replace(/\b\w/g, l => l.toUpperCase()) : 'Guest Mode'}
            </p>
          </div>
          
          <div className="p-4 border rounded-lg">
            <h3 className="font-medium text-gray-900">User Mode</h3>
            <p className="text-sm text-gray-600 capitalize">{auth.userMode}</p>
          </div>
        </div>
      </div>

      {/* Debug Information */}
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-medium text-gray-900 mb-4">Debug Information</h2>
        
        <div className="space-y-2 text-sm">
          <div className="flex justify-between">
            <span className="text-gray-600">BigInt Support:</span>
            <span className="text-gray-900">{typeof BigInt !== 'undefined' ? '✅ Available (Polyfilled)' : '❌ Not Available'}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-600">WebAssembly:</span>
            <span className="text-gray-900">{typeof WebAssembly !== 'undefined' ? '✅ Available' : '❌ Not Available'}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-600">Local Storage:</span>
            <span className="text-gray-900">{typeof localStorage !== 'undefined' ? '✅ Available' : '❌ Not Available'}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-600">Notifications:</span>
            <span className="text-gray-900">{
              'Notification' in window 
                ? Notification.permission === 'granted' ? '✅ Granted' : '⚠️ ' + Notification.permission
                : '❌ Not Supported'
            }</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-600">User Agent:</span>
            <span className="text-gray-900 font-mono text-xs">{navigator.userAgent.slice(0, 60)}...</span>
          </div>
        </div>
      </div>
    </div>
  )
}

export default Settings