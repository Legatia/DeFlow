// Import BigInt polyfill FIRST to prevent conversion errors
import '../utils/bigint-polyfill'

import { ReactNode, useState, useEffect } from 'react'
import { Link, useLocation } from 'react-router-dom'
import MultiChainWalletComponent from './MultiChainWallet'
import multiChainWalletService, { MultiChainWallet, SUPPORTED_CHAINS } from '../services/multiChainWalletService'
import { useEnhancedAuth } from '../contexts/EnhancedAuthContext'
import localCacheService from '../services/localCacheService'

interface LayoutProps {
  children: ReactNode
}

const Layout = ({ children }: LayoutProps) => {
  const location = useLocation()
  const [isWalletModalOpen, setIsWalletModalOpen] = useState(false)
  const [wallet, setWallet] = useState<MultiChainWallet>(multiChainWalletService.getWallet())
  const [unreadNotifications, setUnreadNotifications] = useState(0)
  const auth = useEnhancedAuth()

  useEffect(() => {
    const handleWalletUpdate = (updatedWallet: MultiChainWallet) => {
      setWallet(updatedWallet)
    }

    multiChainWalletService.addListener(handleWalletUpdate)

    return () => {
      multiChainWalletService.removeListener(handleWalletUpdate)
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

  const isActive = (path: string) => {
    return location.pathname === path || (path !== '/' && location.pathname.startsWith(path))
  }

  const navItems = [
    { path: '/', label: 'DeFi Strategies', icon: '💰' },
    { path: '/dashboard', label: 'General Dashboard', icon: '📊' },
    { path: '/workflows', label: 'Custom Workflows', icon: '⚡' },
    { path: '/executions', label: 'Executions', icon: '📋' },
    { path: '/settings', label: 'Settings', icon: '⚙️' }
  ]

  return (
    <div className="flex h-screen bg-gray-50">
      {/* Sidebar */}
      <div className="w-64 bg-white shadow-lg">
        <div className="p-4 border-b">
          <h1 className="text-xl font-bold text-gray-800">DeFlow</h1>
          <p className="text-sm text-gray-600">DeFi Automation Platform</p>
        </div>
        
        <nav className="mt-4">
          {navItems.map((item) => (
            <Link
              key={item.path}
              to={item.path}
              className={`flex items-center px-4 py-3 text-sm font-medium transition-colors ${
                isActive(item.path)
                  ? 'bg-blue-50 border-r-2 border-blue-500 text-blue-700'
                  : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900'
              }`}
            >
              <span className="mr-3 text-lg">{item.icon}</span>
              {item.label}
            </Link>
          ))}
        </nav>
      </div>

      {/* Main content */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Header */}
        <header className="bg-white shadow-sm border-b px-6 py-4">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-semibold text-gray-800">
              {navItems.find(item => isActive(item.path))?.label || 'Dashboard'}
            </h2>
            
            <div className="flex items-center space-x-4">
              {/* User Mode Indicator */}
              <div className="flex items-center space-x-2">
                <div className={`w-3 h-3 rounded-full ${
                  auth.userMode === 'authenticated' ? 'bg-green-500' : 'bg-orange-500'
                }`}></div>
                <span className="text-sm font-medium text-gray-700">
                  {auth.userMode === 'authenticated' ? 'Premium' : 'Guest Mode'}
                </span>
                {!auth.subscriptionBenefits.reducedFees && (
                  <span className="text-xs bg-orange-100 text-orange-800 px-2 py-1 rounded-full">
                    Standard Fees
                  </span>
                )}
              </div>

              {/* Notifications */}
              {unreadNotifications > 0 && (
                <div className="flex items-center space-x-1">
                  <div className="w-5 h-5 bg-red-500 rounded-full flex items-center justify-center">
                    <span className="text-xs text-white font-bold">{unreadNotifications}</span>
                  </div>
                  <span className="text-sm text-gray-600">notifications</span>
                </div>
              )}

              {/* NFID Authentication Status */}
              {auth.isAuthenticated && (
                <div className="flex items-center space-x-2">
                  <div className="w-6 h-6 rounded-full bg-purple-600 flex items-center justify-center text-xs text-white border-2 border-white">
                    G
                  </div>
                  <div className="text-sm text-gray-600">
                    <div>Google: {auth.principal?.toString().slice(0, 8)}...</div>
                    <div className="text-xs text-gray-500">Cross-device sync enabled</div>
                  </div>
                </div>
              )}

              {/* Multi-Chain Wallet Status */}
              {wallet.addresses.length > 0 && (
                <div className="flex items-center space-x-2">
                  <div className="flex -space-x-1">
                    {wallet.addresses.slice(0, 3).map((addr, index) => {
                      const chainConfig = SUPPORTED_CHAINS[addr.chain]
                      return (
                        <div
                          key={addr.chain}
                          className="w-6 h-6 rounded-full flex items-center justify-center text-xs text-white border-2 border-white"
                          style={{ backgroundColor: chainConfig.color }}
                          title={`${chainConfig.name}: ${addr.address.slice(0, 8)}...`}
                        >
                          {chainConfig.icon}
                        </div>
                      )
                    })}
                    {wallet.addresses.length > 3 && (
                      <div className="w-6 h-6 rounded-full bg-gray-400 flex items-center justify-center text-xs text-white border-2 border-white">
                        +{wallet.addresses.length - 3}
                      </div>
                    )}
                  </div>
                  <span className="text-sm text-gray-600">
                    {wallet.addresses.length} chain{wallet.addresses.length !== 1 ? 's' : ''} connected
                  </span>
                </div>
              )}

              {/* Authentication Actions */}
              {!auth.isAuthenticated ? (
                <div className="flex items-center space-x-2">
                  <button 
                    onClick={() => auth.login()}
                    disabled={auth.isLoading}
                    className="px-4 py-2 text-sm bg-gradient-to-r from-purple-600 to-blue-600 text-white rounded-lg hover:from-purple-700 hover:to-blue-700 transition-colors flex items-center space-x-2 disabled:opacity-50"
                  >
                    <span>🚀</span>
                    <span>{auth.isLoading ? 'Connecting...' : 'Upgrade to Premium'}</span>
                  </button>
                  <button 
                    onClick={() => auth.switchToGuestMode()}
                    className="px-3 py-2 text-sm bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition-colors"
                  >
                    Continue as Guest
                  </button>
                </div>
              ) : (
                <button 
                  onClick={() => auth.logout()}
                  className="px-3 py-2 text-sm bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors flex items-center space-x-2"
                >
                  <span>👋</span>
                  <span>Switch to Guest</span>
                </button>
              )}

              {/* Multi-Chain Wallet Management */}
              <button 
                onClick={() => setIsWalletModalOpen(true)}
                className="px-4 py-2 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors flex items-center space-x-2"
              >
                <span>🔗</span>
                <span>{wallet.addresses.length > 0 ? 'Manage Wallets' : 'Connect Wallets'}</span>
              </button>
            </div>
          </div>
        </header>

        {/* Page content */}
        <main className="flex-1 overflow-auto p-6">
          {children}
        </main>
      </div>

      {/* Multi-Chain Wallet Modal */}
      <MultiChainWalletComponent 
        isOpen={isWalletModalOpen}
        onClose={() => setIsWalletModalOpen(false)}
      />
    </div>
  )
}

export default Layout