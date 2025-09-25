/**
 * Tests for DepositAddressManager component
 * Tests deposit address generation, QR code display, and user interactions
 */

import React from 'react'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest'
import DepositAddressManager from '../DepositAddressManager'
import { EnhancedAuthProvider } from '../../contexts/EnhancedAuthContext'
import QRCode from 'qrcode'

// Mock QRCode library
vi.mock('qrcode', () => ({
  default: {
    toDataURL: vi.fn()
  }
}))

// Mock enhanced auth context
const mockAuthContext = {
  subscriptionTier: 'premium',
  user: { principal: 'test-principal' },
  isAuthenticated: true
}

// Mock fetch for API calls
const mockFetch = vi.fn()
global.fetch = mockFetch

// Mock localStorage
const mockLocalStorage = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn()
}
Object.defineProperty(window, 'localStorage', {
  value: mockLocalStorage
})

// Mock clipboard API
Object.assign(navigator, {
  clipboard: {
    writeText: vi.fn().mockResolvedValue(undefined)
  }
})

// Wrapper component with context
const TestWrapper: React.FC<{ children: React.ReactNode }> = ({ children }) => (
  <EnhancedAuthProvider value={mockAuthContext as any}>
    {children}
  </EnhancedAuthProvider>
)

describe('DepositAddressManager', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockLocalStorage.getItem.mockReturnValue(null)
    mockFetch.mockClear()
    ;(QRCode.toDataURL as any).mockResolvedValue('data:image/png;base64,mockqrcode')
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('renders without crashing', () => {
    render(
      <TestWrapper>
        <DepositAddressManager />
      </TestWrapper>
    )

    expect(screen.getByText('Deposit Addresses')).toBeInTheDocument()
    expect(screen.getByText(/Generate unique deposit addresses/)).toBeInTheDocument()
  })

  it('displays welcome message when no addresses exist', () => {
    render(
      <TestWrapper>
        <DepositAddressManager />
      </TestWrapper>
    )

    expect(screen.getByText('🚀 Get Started with DeFi Automation')).toBeInTheDocument()
    expect(screen.getByText(/DeFlow generates unique deposit addresses/)).toBeInTheDocument()
  })

  it('displays supported chains with generate buttons', () => {
    render(
      <TestWrapper>
        <DepositAddressManager />
      </TestWrapper>
    )

    // Check for supported chains
    expect(screen.getByText('Ethereum Ecosystem')).toBeInTheDocument()
    expect(screen.getByText('Bitcoin')).toBeInTheDocument()
    expect(screen.getByText('Solana')).toBeInTheDocument()
    expect(screen.getByText('Internet Computer')).toBeInTheDocument()

    // Check for generate buttons
    const generateButtons = screen.getAllByText(/Generate.*Address/)
    expect(generateButtons).toHaveLength(4)
  })

  it('generates address when button is clicked', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ address: '0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e' })
    })

    render(
      <TestWrapper>
        <DepositAddressManager />
      </TestWrapper>
    )

    const generateButton = screen.getByText('Generate Ethereum Ecosystem Address')
    fireEvent.click(generateButton)

    // Check loading state
    await waitFor(() => {
      expect(screen.getByText('Generating Address...')).toBeInTheDocument()
    })

    // Wait for address generation to complete
    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledWith('/api/defi/generate_deposit_address', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          chain_type: 'ethereum'
        })
      })
    }, { timeout: 3000 })

    await waitFor(() => {
      expect(screen.getByText('0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e')).toBeInTheDocument()
    })

    expect(QRCode.toDataURL).toHaveBeenCalledWith('ethereum:0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e', expect.any(Object))
  })

  it('falls back to mock address if API fails', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: false,
      status: 500
    })

    const consoleWarnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {})

    render(
      <TestWrapper>
        <DepositAddressManager />
      </TestWrapper>
    )

    const generateButton = screen.getByText('Generate Bitcoin Address')
    fireEvent.click(generateButton)

    await waitFor(() => {
      expect(consoleWarnSpy).toHaveBeenCalledWith('Backend API failed, using mock address')
    })

    await waitFor(() => {
      expect(screen.getByText('bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh')).toBeInTheDocument()
    })

    consoleWarnSpy.mockRestore()
  })

  it('displays QR code when show QR button is clicked', async () => {
    // First generate an address
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ address: '0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e' })
    })

    render(
      <TestWrapper>
        <DepositAddressManager />
      </TestWrapper>
    )

    const generateButton = screen.getByText('Generate Ethereum Ecosystem Address')
    fireEvent.click(generateButton)

    await waitFor(() => {
      expect(screen.getByText('0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e')).toBeInTheDocument()
    })

    // Click Show QR button
    const showQRButton = screen.getByText('Show QR')
    fireEvent.click(showQRButton)

    await waitFor(() => {
      expect(screen.getByAltText('QR Code for deposit address')).toBeInTheDocument()
      expect(screen.getByText('📱 Scan with wallet app to copy address')).toBeInTheDocument()
      expect(screen.getByText('💾 Download QR Code')).toBeInTheDocument()
    })

    // Button should change to "Hide QR"
    expect(screen.getByText('Hide QR')).toBeInTheDocument()
  })

  it('copies address to clipboard when copy button is clicked', async () => {
    // Generate address first
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ address: '0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e' })
    })

    render(
      <TestWrapper>
        <DepositAddressManager />
      </TestWrapper>
    )

    const generateButton = screen.getByText('Generate Ethereum Ecosystem Address')
    fireEvent.click(generateButton)

    await waitFor(() => {
      expect(screen.getByText('0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e')).toBeInTheDocument()
    })

    // Click copy button
    const copyButton = screen.getByText('📋 Copy')
    fireEvent.click(copyButton)

    expect(navigator.clipboard.writeText).toHaveBeenCalledWith('0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e')

    await waitFor(() => {
      expect(screen.getByText('✓ Copied')).toBeInTheDocument()
    })
  })

  it('displays multi-network support info for Ethereum', async () => {
    // Generate Ethereum address
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ address: '0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e' })
    })

    render(
      <TestWrapper>
        <DepositAddressManager />
      </TestWrapper>
    )

    const generateButton = screen.getByText('Generate Ethereum Ecosystem Address')
    fireEvent.click(generateButton)

    await waitFor(() => {
      expect(screen.getByText('💡 Multi-Network Support')).toBeInTheDocument()
      expect(screen.getByText('This address works on all Ethereum-compatible networks:')).toBeInTheDocument()
      expect(screen.getByText('• Ethereum (ETH)')).toBeInTheDocument()
      expect(screen.getByText('• Polygon (MATIC)')).toBeInTheDocument()
      expect(screen.getByText('• Arbitrum (ETH)')).toBeInTheDocument()
      expect(screen.getByText('💰 Tip: Use Polygon or Arbitrum for lower fees!')).toBeInTheDocument()
    })
  })

  it('checks balance when refresh button is clicked', async () => {
    // Generate address first
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ address: '0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e' })
    })

    render(
      <TestWrapper>
        <DepositAddressManager />
      </TestWrapper>
    )

    const generateButton = screen.getByText('Generate Ethereum Ecosystem Address')
    fireEvent.click(generateButton)

    await waitFor(() => {
      expect(screen.getByText('0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e')).toBeInTheDocument()
    })

    // Click refresh button
    const refreshButton = screen.getByText('🔄')
    fireEvent.click(refreshButton)

    await waitFor(() => {
      expect(screen.getByText('⏳')).toBeInTheDocument()
      expect(screen.getByText('Checking...')).toBeInTheDocument()
    })
  })

  it('removes address when delete button is clicked', async () => {
    // Mock confirm dialog
    const mockConfirm = vi.spyOn(window, 'confirm').mockReturnValue(true)

    // Generate address first
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ address: '0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e' })
    })

    render(
      <TestWrapper>
        <DepositAddressManager />
      </TestWrapper>
    )

    const generateButton = screen.getByText('Generate Ethereum Ecosystem Address')
    fireEvent.click(generateButton)

    await waitFor(() => {
      expect(screen.getByText('0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e')).toBeInTheDocument()
    })

    // Click delete button
    const deleteButton = screen.getByText('🗑️')
    fireEvent.click(deleteButton)

    expect(mockConfirm).toHaveBeenCalledWith('Remove this deposit address? This cannot be undone.')

    await waitFor(() => {
      expect(screen.queryByText('0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e')).not.toBeInTheDocument()
    })

    mockConfirm.mockRestore()
  })

  it('saves and loads addresses from localStorage', async () => {
    const mockSavedAddresses = JSON.stringify([
      {
        id: '1',
        chainType: 'bitcoin',
        chainName: 'Bitcoin',
        address: 'bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh',
        qrCode: 'data:image/png;base64,mockqrcode',
        balance: 0.1,
        balanceUSD: 4300,
        isGenerated: true,
        createdAt: '2023-01-01T00:00:00.000Z',
        transactions: []
      }
    ])

    mockLocalStorage.getItem.mockReturnValue(mockSavedAddresses)

    render(
      <TestWrapper>
        <DepositAddressManager />
      </TestWrapper>
    )

    expect(mockLocalStorage.getItem).toHaveBeenCalledWith('deflow_deposit_addresses')

    // Should display saved address
    await waitFor(() => {
      expect(screen.getByText('bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh')).toBeInTheDocument()
      expect(screen.getByText('0.100000 BTC ($4300.00)')).toBeInTheDocument()
    })
  })

  it('displays usage instructions when addresses exist', async () => {
    const mockSavedAddresses = JSON.stringify([
      {
        id: '1',
        chainType: 'ethereum',
        chainName: 'Ethereum Ecosystem',
        address: '0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e',
        qrCode: 'data:image/png;base64,mockqrcode',
        balance: 0,
        balanceUSD: 0,
        isGenerated: true,
        createdAt: '2023-01-01T00:00:00.000Z',
        transactions: []
      }
    ])

    mockLocalStorage.getItem.mockReturnValue(mockSavedAddresses)

    render(
      <TestWrapper>
        <DepositAddressManager />
      </TestWrapper>
    )

    await waitFor(() => {
      expect(screen.getByText('💡 Using Your Deposit Addresses')).toBeInTheDocument()
      expect(screen.getByText('Your deposit addresses are ready for automated DeFi strategies:')).toBeInTheDocument()
      expect(screen.getByText('Send tokens: Transfer any amount to your deposit addresses')).toBeInTheDocument()
      expect(screen.getByText('Automatic detection: We\'ll detect deposits within minutes')).toBeInTheDocument()
      expect(screen.getByText('⚡ Pro Tip: Start with small amounts to test the system before depositing larger sums.')).toBeInTheDocument()
    })
  })

  it('handles QR code generation errors gracefully', async () => {
    // Mock QR code generation failure
    ;(QRCode.toDataURL as any).mockRejectedValue(new Error('QR generation failed'))

    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ address: '0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e' })
    })

    render(
      <TestWrapper>
        <DepositAddressManager />
      </TestWrapper>
    )

    const generateButton = screen.getByText('Generate Ethereum Ecosystem Address')
    fireEvent.click(generateButton)

    await waitFor(() => {
      expect(consoleErrorSpy).toHaveBeenCalledWith('Failed to generate QR code:', expect.any(Error))
      // Should fall back to external API URL
      expect(screen.getByText('0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e')).toBeInTheDocument()
    })

    consoleErrorSpy.mockRestore()
  })

  it('shows loading state during address generation', async () => {
    let resolvePromise: (value: any) => void
    const pendingPromise = new Promise((resolve) => {
      resolvePromise = resolve
    })

    mockFetch.mockReturnValue(pendingPromise)

    render(
      <TestWrapper>
        <DepositAddressManager />
      </TestWrapper>
    )

    const generateButton = screen.getByText('Generate Bitcoin Address')
    fireEvent.click(generateButton)

    // Should show loading state
    expect(screen.getByText('Generating Address...')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /generating address/i })).toBeDisabled()

    // Resolve the promise
    resolvePromise!({
      ok: true,
      json: async () => ({ address: 'bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh' })
    })

    await waitFor(() => {
      expect(screen.getByText('bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh')).toBeInTheDocument()
    })
  })

  it('prevents generating duplicate addresses for same chain', async () => {
    // Generate first address
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ address: '0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e' })
    })

    render(
      <TestWrapper>
        <DepositAddressManager />
      </TestWrapper>
    )

    const generateButton = screen.getByText('Generate Ethereum Ecosystem Address')
    fireEvent.click(generateButton)

    await waitFor(() => {
      expect(screen.getByText('0x742d35Cc6634C0532925a3b8D0Ff6e5fD8fE9A3e')).toBeInTheDocument()
    })

    // Generate button should no longer be available for Ethereum
    expect(screen.queryByText('Generate Ethereum Ecosystem Address')).not.toBeInTheDocument()

    // But should still be available for other chains
    expect(screen.getByText('Generate Bitcoin Address')).toBeInTheDocument()
    expect(screen.getByText('Generate Solana Address')).toBeInTheDocument()
  })
})