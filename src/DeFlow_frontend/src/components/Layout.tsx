// Import BigInt polyfill FIRST to prevent conversion errors
import '../utils/bigint-polyfill'

import { ReactNode, useState, useEffect } from 'react'
import { Link, useLocation } from 'react-router-dom'
import WalletConfiguration from './WalletConfiguration'
import { useEnhancedAuth } from '../contexts/EnhancedAuthContext'
import { AuthDropdown } from './AuthDropdown'
import { NotificationDropdown } from './NotificationDropdown'
import localCacheService from '../services/localCacheService'
import SubscriptionService, { UserSubscription } from '../services/subscriptionService'
import { useNavigate } from 'react-router-dom'

interface LayoutProps {
  children: ReactNode
}

const Layout = ({ children }: LayoutProps) => {
  const location = useLocation()
  const navigate = useNavigate()
  const [isWalletModalOpen, setIsWalletModalOpen] = useState(false)
  const [walletCount, setWalletCount] = useState(0)
  const [unreadNotifications, setUnreadNotifications] = useState(0)
  const [subscription, setSubscription] = useState<UserSubscription>(SubscriptionService.getCurrentSubscription())
  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState(() => {
    // Load sidebar state from localStorage
    const savedState = localStorage.getItem('deflow_sidebar_collapsed')
    return savedState === 'true'
  })
  const [showToggleHint, setShowToggleHint] = useState(() => {
    // Show hint if user hasn't seen it before
    return !localStorage.getItem('deflow_sidebar_hint_seen')
  })
  const auth = useEnhancedAuth()

  // Load wallet count from localStorage
  useEffect(() => {
    const loadWalletCount = () => {
      try {
        const savedWallets = localStorage.getItem('deflow_wallets')
        if (savedWallets) {
          const wallets = JSON.parse(savedWallets)
          setWalletCount(wallets.length || 0)
        }
      } catch (error) {
        console.error('Failed to load wallet count:', error)
      }
    }
    
    loadWalletCount()
    
    // Listen for storage changes to update wallet count
    const handleStorageChange = () => {
      loadWalletCount()
    }
    
    window.addEventListener('storage', handleStorageChange)
    return () => window.removeEventListener('storage', handleStorageChange)
  }, [])

  // Save sidebar state to localStorage
  useEffect(() => {
    localStorage.setItem('deflow_sidebar_collapsed', isSidebarCollapsed.toString())
  }, [isSidebarCollapsed])

  // Hide toggle hint after 5 seconds
  useEffect(() => {
    if (showToggleHint) {
      const timer = setTimeout(() => {
        setShowToggleHint(false)
        localStorage.setItem('deflow_sidebar_hint_seen', 'true')
      }, 5000)
      return () => clearTimeout(timer)
    }
  }, [showToggleHint])

  // Keyboard shortcut to toggle sidebar (Ctrl+B)
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.ctrlKey && event.key === 'b') {
        event.preventDefault()
        setIsSidebarCollapsed(prev => !prev)
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [])

  useEffect(() => {
    const handleSubscriptionUpdate = (updatedSubscription: UserSubscription) => {
      setSubscription(updatedSubscription)
    }

    SubscriptionService.addSubscriptionListener(handleSubscriptionUpdate)

    return () => {
      SubscriptionService.removeSubscriptionListener(handleSubscriptionUpdate)
    }
  }, [])

  // Update notification count
  useEffect(() => {
    const updateNotificationCount = () => {
      setUnreadNotifications(localCacheService.getUnreadNotificationCount())
    }
    
    // Initial count
    updateNotificationCount()
    
    // Update every 5 seconds
    const interval = setInterval(updateNotificationCount, 5000)
    
    return () => clearInterval(interval)
  }, [])

  // Handle subscribe now logic
  const handleSubscribeNow = () => {
    navigate('/premium')
  }

  const isActive = (path: string) => {
    return location.pathname === path || (path !== '/' && location.pathname.startsWith(path))
  }

  const navItems = [
    { path: '/dashboard', label: 'Dashboard', icon: '📊' },
    { path: '/workflows', label: 'Custom Workflows', icon: '⚡' },
    { path: '/executions', label: 'Executions', icon: '📋' },
    { path: '/settings', label: 'Settings', icon: '⚙️' }
  ]

  return (
    <div className="flex h-screen bg-gray-50">
      {/* Sidebar */}
      <div className={`${isSidebarCollapsed ? 'w-16' : 'w-64'} bg-white shadow-lg transition-all duration-300 ease-in-out relative`}>
        {/* Sidebar Border Indicator */}
        <div className="absolute top-0 right-0 w-px h-full bg-gradient-to-b from-purple-200 via-blue-200 to-purple-200"></div>
        
        <div className={`p-4 border-b ${isSidebarCollapsed ? 'text-center' : ''}`}>
          {isSidebarCollapsed ? (
            <div className="w-8 h-8 bg-gradient-to-r from-purple-600 to-blue-600 rounded-lg flex items-center justify-center mx-auto hover:shadow-md transition-shadow cursor-pointer">
              <span className="text-white text-xl font-bold">D</span>
            </div>
          ) : (
            <>
              <h1 className="text-xl font-bold bg-gradient-to-r from-purple-600 to-blue-600 bg-clip-text text-transparent">DeFlow</h1>
              <p className="text-sm text-gray-600">DeFi Automation Platform</p>
            </>
          )}
        </div>
        
        <nav className="mt-4">
          {navItems.map((item) => (
            <Link
              key={item.path}
              to={item.path}
              className={`flex items-center ${isSidebarCollapsed ? 'justify-center px-2' : 'px-4'} py-3 text-sm font-medium transition-all duration-200 group ${
                isActive(item.path)
                  ? 'bg-gradient-to-r from-blue-50 to-purple-50 border-r-2 border-blue-500 text-blue-700'
                  : 'text-gray-600 hover:bg-gradient-to-r hover:from-gray-50 hover:to-blue-25 hover:text-gray-900'
              }`}
              title={isSidebarCollapsed ? item.label : ''}
            >
              <span className={`${isSidebarCollapsed ? '' : 'mr-3'} text-lg transition-transform group-hover:scale-110`}>{item.icon}</span>
              {!isSidebarCollapsed && (
                <span className="transition-all duration-300 ease-in-out">{item.label}</span>
              )}
            </Link>
          ))}
        </nav>
        
        {/* Collapsed State Helper */}
        {isSidebarCollapsed && (
          <div className="absolute bottom-4 left-1/2 transform -translate-x-1/2">
            <div className="text-xs text-gray-400 text-center">
              <div className="w-6 h-px bg-gray-300 mx-auto mb-1"></div>
              <span>Menu</span>
            </div>
          </div>
        )}
      </div>

      {/* Main content */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Header */}
        <header className="bg-white shadow-sm border-b px-6 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-4">
              {/* Sidebar Toggle Button */}
              <div className="relative">
                <button
                  onClick={() => {
                    setIsSidebarCollapsed(!isSidebarCollapsed)
                    if (showToggleHint) {
                      setShowToggleHint(false)
                      localStorage.setItem('deflow_sidebar_hint_seen', 'true')
                    }
                  }}
                  className={`flex items-center space-x-2 p-2 text-gray-600 hover:text-blue-700 hover:bg-blue-50 rounded-lg transition-all duration-200 border border-gray-200 hover:border-blue-300 ${
                    showToggleHint ? 'animate-pulse border-blue-300 bg-blue-25' : ''
                  }`}
                  title={`${isSidebarCollapsed ? 'Expand' : 'Collapse'} sidebar (Ctrl+B)`}
                >
                  <svg 
                    className="w-4 h-4" 
                    fill="none" 
                    stroke="currentColor" 
                    viewBox="0 0 24 24"
                  >
                    {isSidebarCollapsed ? (
                      // Expand icon (menu with arrows right)
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
                    ) : (
                      // Collapse icon (sidebar with arrow left)
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 19l-7-7 7-7M19 5v14" />
                    )}
                  </svg>
                  <span className="text-xs font-medium hidden sm:inline">
                    {isSidebarCollapsed ? 'Show' : 'Hide'} Menu
                  </span>
                </button>
                
                {/* First-time user hint */}
                {showToggleHint && (
                  <div className="absolute top-full left-0 mt-2 p-2 bg-blue-600 text-white text-xs rounded-lg shadow-lg z-50 whitespace-nowrap animate-fade-in">
                    <div className="flex items-center space-x-1">
                      <span>💡</span>
                      <span>Click here to hide menu and get more space!</span>
                    </div>
                    <div className="absolute -top-1 left-4 w-2 h-2 bg-blue-600 rotate-45"></div>
                  </div>
                )}
              </div>
              
              <h2 className="text-lg font-semibold text-gray-800">
                {navItems.find(item => isActive(item.path))?.label || 'Dashboard'}
              </h2>
            </div>
            
            <div className="flex items-center space-x-4">
              {/* User Plan Indicator */}
              <div className="flex items-center space-x-2">
                <div className={`w-3 h-3 rounded-full ${SubscriptionService.getSubscriptionColor()}`}></div>
                <span className="text-sm font-medium text-gray-700">
                  {SubscriptionService.getSubscriptionDisplayText()}
                </span>
                {SubscriptionService.isExpiringSoon() && (
                  <span className="text-xs bg-yellow-100 text-yellow-800 px-2 py-1 rounded-full">
                    {SubscriptionService.getDaysRemaining()} days left
                  </span>
                )}
              </div>

              {/* Notifications */}
              <NotificationDropdown 
                unreadCount={unreadNotifications} 
                onMarkAllRead={() => setUnreadNotifications(0)}
              />

              {/* Authentication Status */}
              {auth.isAuthenticated && (
                <div className="flex items-center space-x-2">
                  <div className={`w-6 h-6 rounded-full flex items-center justify-center text-xs text-white border-2 border-white ${
                    auth.authMethod === 'nfid' ? 'bg-purple-600' : 'bg-blue-600'
                  }`}>
                    {auth.authMethod === 'nfid' ? 'G' : '∞'}
                  </div>
                  <div className="text-sm text-gray-600">
                    <div>
                      {auth.authMethod === 'nfid' ? 'Google' : 'Internet ID'}: {auth.principal?.toString().slice(0, 8)}...
                    </div>
                    <div className="text-xs text-gray-500">Cross-device sync enabled</div>
                  </div>
                </div>
              )}

              {/* Wallet Status */}
              {walletCount > 0 && (
                <div className="flex items-center space-x-2">
                  <div className="flex items-center space-x-1">
                    <div className="w-6 h-6 rounded-full bg-gradient-to-r from-purple-600 to-blue-600 flex items-center justify-center text-xs text-white border-2 border-white">
                      💰
                    </div>
                    <span className="text-sm text-gray-600">
                      {walletCount} wallet{walletCount !== 1 ? 's' : ''} configured
                    </span>
                  </div>
                </div>
              )}

              {/* Authentication Actions */}
              {!auth.isAuthenticated ? (
                <div className="flex items-center space-x-2">
                  <button 
                    onClick={handleSubscribeNow}
                    className="px-4 py-2 text-sm bg-gradient-to-r from-purple-600 to-blue-600 text-white rounded-lg hover:from-purple-700 hover:to-blue-700 transition-colors flex items-center space-x-2"
                  >
                    <span>Subscribe Now</span>
                  </button>
                  <AuthDropdown />
                </div>
              ) : (
                <div className="flex items-center space-x-2">
                  <button 
                    onClick={() => navigate('/premium')}
                    className="px-3 py-2 text-sm bg-gradient-to-r from-purple-600 to-blue-600 text-white rounded-lg hover:from-purple-700 hover:to-blue-700 transition-colors flex items-center space-x-2"
                  >
                    <span>⭐</span>
                    <span>Manage Subscription</span>
                  </button>
                  <button 
                    onClick={() => auth.logout()}
                    className="px-3 py-2 text-sm bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors flex items-center space-x-2"
                  >
                    <span>Switch to Guest</span>
                  </button>
                </div>
              )}

              {/* Multi-Chain Wallet Management */}
              <button 
                onClick={() => setIsWalletModalOpen(true)}
                className="px-4 py-2 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors flex items-center space-x-2"
              >
                <span>{walletCount > 0 ? `${walletCount} Wallet${walletCount !== 1 ? 's' : ''}` : 'Add Wallet'}</span>
              </button>
            </div>
          </div>
        </header>

        {/* Page content */}
        <main className="flex-1 overflow-auto p-6">
          {children}
        </main>
      </div>

      {/* Wallet Configuration Modal */}
      {isWalletModalOpen && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl shadow-xl max-w-4xl w-full mx-4 max-h-[90vh] overflow-hidden">
            {/* Header */}
            <div className="px-6 py-4 border-b border-gray-200 flex items-center justify-between bg-gradient-to-r from-purple-600 to-blue-600">
              <div>
                <h2 className="text-xl font-semibold text-white">Wallet Configuration</h2>
                <p className="text-purple-100 text-sm mt-1">
                  Add seed phrases to enable automated DeFi strategies
                </p>
              </div>
              <button
                onClick={() => {
                  setIsWalletModalOpen(false)
                  // Reload wallet count when modal closes
                  const savedWallets = localStorage.getItem('deflow_wallets')
                  if (savedWallets) {
                    const wallets = JSON.parse(savedWallets)
                    setWalletCount(wallets.length || 0)
                  }
                }}
                className="text-white hover:text-purple-200 text-2xl font-light"
              >
                ×
              </button>
            </div>

            {/* Content */}
            <div className="p-6 overflow-y-auto max-h-[75vh]">
              <WalletConfiguration />
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default Layout