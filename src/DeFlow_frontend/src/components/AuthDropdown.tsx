// Import BigInt polyfill FIRST to prevent conversion errors
import '../utils/bigint-polyfill'

import React, { useState, useRef, useEffect } from 'react'
import { useEnhancedAuth } from '../contexts/EnhancedAuthContext'

interface AuthDropdownProps {
  onClose?: () => void
}

export const AuthDropdown = ({ onClose }: AuthDropdownProps) => {
  const [isOpen, setIsOpen] = useState(false)
  const dropdownRef = useRef<HTMLDivElement>(null)
  const auth = useEnhancedAuth()

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false)
        onClose?.()
      }
    }

    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [onClose])

  const handleNFIDLogin = async () => {
    setIsOpen(false)
    onClose?.()
    await auth.loginWithNFID()
  }

  const handleInternetIdentityLogin = async () => {
    setIsOpen(false)
    onClose?.()
    await auth.loginWithInternetIdentity()
  }

  const handleContinueAsGuest = () => {
    setIsOpen(false)
    onClose?.()
    auth.switchToGuestMode()
  }

  return (
    <div className="relative" ref={dropdownRef}>
      {/* Login/Register Button */}
      <button
        onClick={() => setIsOpen(!isOpen)}
        disabled={auth.isLoading}
        className="px-4 py-2 text-sm bg-gradient-to-r from-purple-600 to-blue-600 text-white rounded-lg hover:from-purple-700 hover:to-blue-700 transition-colors flex items-center space-x-2 disabled:opacity-50"
      >
        <span>{auth.isLoading ? 'Connecting...' : 'Register / Login'}</span>
        <span className={`transition-transform ${isOpen ? 'rotate-180' : ''}`}>
          ▼
        </span>
      </button>

      {/* Dropdown Menu */}
      {isOpen && (
        <div className="absolute right-0 mt-2 w-64 bg-white rounded-lg shadow-xl border border-gray-200 z-50">
          <div className="p-4">
            <h3 className="text-lg font-semibold text-gray-800 mb-3">Choose Authentication Method</h3>
            
            {/* NFID Option */}
            <button
              onClick={handleNFIDLogin}
              disabled={auth.isLoading}
              className="w-full p-3 text-left rounded-lg hover:bg-purple-50 transition-colors border border-purple-200 mb-3 disabled:opacity-50"
            >
              <div className="flex items-center space-x-3">
                <div className="w-8 h-8 bg-purple-600 rounded-full flex items-center justify-center">
                  <span className="text-white font-bold text-sm">G</span>
                </div>
                <div>
                  <div className="font-medium text-gray-800">NFID (Google)</div>
                  <div className="text-sm text-gray-600">Login with your Google account</div>
                </div>
              </div>
            </button>

            {/* Internet Identity Option */}
            <button
              onClick={handleInternetIdentityLogin}
              disabled={auth.isLoading}
              className="w-full p-3 text-left rounded-lg hover:bg-blue-50 transition-colors border border-blue-200 mb-3 disabled:opacity-50"
            >
              <div className="flex items-center space-x-3">
                <div className="w-8 h-8 bg-blue-600 rounded-full flex items-center justify-center">
                  <span className="text-white font-bold text-sm">∞</span>
                </div>
                <div>
                  <div className="font-medium text-gray-800">Internet Identity</div>
                  <div className="text-sm text-gray-600">ICP's native identity system</div>
                </div>
              </div>
            </button>

            {/* Divider */}
            <div className="border-t border-gray-200 my-3"></div>

            {/* Continue as Guest */}
            <button
              onClick={handleContinueAsGuest}
              className="w-full p-3 text-left rounded-lg hover:bg-gray-50 transition-colors"
            >
              <div className="flex items-center space-x-3">
                <div className="w-8 h-8 bg-gray-400 rounded-full flex items-center justify-center">
                  <span className="text-white font-bold text-sm">👤</span>
                </div>
                <div>
                  <div className="font-medium text-gray-800">Continue as Guest</div>
                  <div className="text-sm text-gray-600">Use app without account (data saved locally)</div>
                </div>
              </div>
            </button>

            {/* Benefits Note */}
            <div className="mt-3 p-2 bg-blue-50 rounded-lg">
              <div className="text-xs text-blue-800">
                <strong>Premium Benefits:</strong> Reduced fees, cross-device sync, unlimited workflows, priority execution
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}