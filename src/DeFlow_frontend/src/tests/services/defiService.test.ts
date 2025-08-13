// DeFi Service Tests - Bitcoin Integration
// Tests for Bitcoin DeFi functionality including portfolio management, transactions, and addresses

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { createMockUser, createMockBitcoinPortfolio, createMockBitcoinAddress } from '../utils/testUtils'

// Mock the ICP service
const mockIcpService = {
  call: vi.fn(),
  query: vi.fn(),
}

vi.mock('../../services/icpService', () => ({
  default: mockIcpService,
}))

// DeFi Service Types (matching backend types)
interface BitcoinPortfolio {
  addresses: BitcoinAddress[]
  total_btc: number
  total_satoshis: number
  total_value_usd: number
  utxos: BitcoinUTXO[]
  last_updated: bigint
}

interface BitcoinAddress {
  address: string
  address_type: 'P2PKH' | 'P2WPKH' | 'P2TR'
  derivation_path: string
  balance_satoshis: number
  utxo_count: number
}

interface BitcoinUTXO {
  txid: string
  vout: number
  value_satoshis: number
  script_pubkey: string
  confirmations: number
}

interface BitcoinSendResult {
  success: boolean
  transaction_id?: string
  from_address: string
  to_address: string
  amount_satoshis: number
  fee_satoshis: number
  change_amount_satoshis: number
  confirmation_time_estimate_minutes: number
  error_message?: string
}

// DeFi Service Implementation
class DeFiService {
  async getBitcoinPortfolio(): Promise<BitcoinPortfolio> {
    const result = await mockIcpService.call('get_bitcoin_portfolio', [])
    if (result.Err) {
      throw new Error(result.Err)
    }
    return result.Ok
  }

  async sendBitcoin(
    to_address: string,
    amount_satoshis: number,
    fee_satoshis?: number,
    from_address_type?: 'P2PKH' | 'P2WPKH' | 'P2TR'
  ): Promise<BitcoinSendResult> {
    const result = await mockIcpService.call('send_bitcoin', [
      to_address,
      amount_satoshis,
      fee_satoshis ? [fee_satoshis] : [],
      from_address_type ? [from_address_type] : []
    ])
    if (result.Err) {
      throw new Error(result.Err)
    }
    return result.Ok
  }

  async getBitcoinAddress(address_type: 'P2PKH' | 'P2WPKH' | 'P2TR'): Promise<BitcoinAddress> {
    const result = await mockIcpService.call('get_bitcoin_address', [address_type])
    if (result.Err) {
      throw new Error(result.Err)
    }
    return result.Ok
  }

  async getAllBitcoinAddresses(): Promise<BitcoinAddress[]> {
    const result = await mockIcpService.call('get_all_bitcoin_addresses', [])
    if (result.Err) {
      throw new Error(result.Err)
    }
    return result.Ok
  }

  validateBitcoinAddress(address: string): 'P2PKH' | 'P2WPKH' | 'P2TR' {
    if (address.startsWith('1')) {
      return 'P2PKH'
    } else if (address.startsWith('bc1q') || address.startsWith('tb1q') || address.startsWith('bcrt1q')) {
      return 'P2WPKH'
    } else if (address.startsWith('bc1p') || address.startsWith('tb1p') || address.startsWith('bcrt1p')) {
      return 'P2TR'
    } else {
      throw new Error(`Invalid Bitcoin address format: ${address}`)
    }
  }

  async estimateBitcoinFee(
    utxo_count: number,
    output_count: number,
    priority: 'Low' | 'Medium' | 'High' | 'Urgent'
  ): Promise<{ total_fee_satoshis: number; sat_per_byte: number; confirmation_blocks: number }> {
    const result = await mockIcpService.query('estimate_bitcoin_fee', [utxo_count, output_count, priority])
    return result
  }
}

describe('DeFi Service - Bitcoin Integration', () => {
  let defiService: DeFiService

  beforeEach(() => {
    defiService = new DeFiService()
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  describe('Bitcoin Portfolio Management', () => {
    it('should get Bitcoin portfolio successfully', async () => {
      // Arrange
      const mockPortfolio = createMockBitcoinPortfolio({
        total_btc: 0.5,
        total_value_usd: 22500.0,
        addresses: [
          createMockBitcoinAddress({ address_type: 'P2WPKH', balance_satoshis: 50000000 })
        ]
      })

      mockIcpService.call.mockResolvedValue({ Ok: mockPortfolio })

      // Act
      const portfolio = await defiService.getBitcoinPortfolio()

      // Assert
      expect(mockIcpService.call).toHaveBeenCalledWith('get_bitcoin_portfolio', [])
      expect(portfolio.total_btc).toBe(0.5)
      expect(portfolio.total_value_usd).toBe(22500.0)
      expect(portfolio.addresses).toHaveLength(1)
      expect(portfolio.addresses[0].address_type).toBe('P2WPKH')
    })

    it('should handle portfolio fetch errors', async () => {
      // Arrange
      mockIcpService.call.mockResolvedValue({ Err: 'Bitcoin service not initialized' })

      // Act & Assert
      await expect(defiService.getBitcoinPortfolio()).rejects.toThrow('Bitcoin service not initialized')
    })

    it('should get empty portfolio for new user', async () => {
      // Arrange
      const emptyPortfolio = createMockBitcoinPortfolio({
        total_btc: 0,
        total_value_usd: 0,
        addresses: []
      })

      mockIcpService.call.mockResolvedValue({ Ok: emptyPortfolio })

      // Act
      const portfolio = await defiService.getBitcoinPortfolio()

      // Assert
      expect(portfolio.total_btc).toBe(0)
      expect(portfolio.addresses).toHaveLength(0)
    })
  })

  describe('Bitcoin Address Management', () => {
    it('should generate P2WPKH address', async () => {
      // Arrange
      const mockAddress = createMockBitcoinAddress({
        address: 'bcrt1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh',
        address_type: 'P2WPKH'
      })

      mockIcpService.call.mockResolvedValue({ Ok: mockAddress })

      // Act
      const address = await defiService.getBitcoinAddress('P2WPKH')

      // Assert
      expect(mockIcpService.call).toHaveBeenCalledWith('get_bitcoin_address', ['P2WPKH'])
      expect(address.address_type).toBe('P2WPKH')
      expect(address.address).toMatch(/^bcrt1q/)
    })

    it('should generate P2PKH address', async () => {
      // Arrange
      const mockAddress = createMockBitcoinAddress({
        address: '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa',
        address_type: 'P2PKH'
      })

      mockIcpService.call.mockResolvedValue({ Ok: mockAddress })

      // Act
      const address = await defiService.getBitcoinAddress('P2PKH')

      // Assert
      expect(address.address_type).toBe('P2PKH')
      expect(address.address).toMatch(/^1/)
    })

    it('should generate P2TR address', async () => {
      // Arrange
      const mockAddress = createMockBitcoinAddress({
        address: 'bcrt1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr',
        address_type: 'P2TR'
      })

      mockIcpService.call.mockResolvedValue({ Ok: mockAddress })

      // Act
      const address = await defiService.getBitcoinAddress('P2TR')

      // Assert
      expect(address.address_type).toBe('P2TR')
      expect(address.address).toMatch(/^bcrt1p/)
    })

    it('should get all user addresses', async () => {
      // Arrange
      const mockAddresses = [
        createMockBitcoinAddress({ address_type: 'P2PKH' }),
        createMockBitcoinAddress({ address_type: 'P2WPKH' }),
        createMockBitcoinAddress({ address_type: 'P2TR' })
      ]

      mockIcpService.call.mockResolvedValue({ Ok: mockAddresses })

      // Act
      const addresses = await defiService.getAllBitcoinAddresses()

      // Assert
      expect(addresses).toHaveLength(3)
      expect(addresses.map(a => a.address_type)).toEqual(['P2PKH', 'P2WPKH', 'P2TR'])
    })

    it('should validate Bitcoin addresses correctly', () => {
      // Act & Assert
      expect(defiService.validateBitcoinAddress('1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa')).toBe('P2PKH')
      expect(defiService.validateBitcoinAddress('bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4')).toBe('P2WPKH')
      expect(defiService.validateBitcoinAddress('bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr')).toBe('P2TR')
    })

    it('should throw error for invalid address format', () => {
      // Act & Assert
      expect(() => defiService.validateBitcoinAddress('invalid_address')).toThrow('Invalid Bitcoin address format')
    })
  })

  describe('Bitcoin Transactions', () => {
    it('should send Bitcoin successfully', async () => {
      // Arrange
      const mockSendResult: BitcoinSendResult = {
        success: true,
        transaction_id: 'abc123def456',
        from_address: 'bcrt1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh',
        to_address: '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa',
        amount_satoshis: 100000,
        fee_satoshis: 1000,
        change_amount_satoshis: 49899000,
        confirmation_time_estimate_minutes: 30
      }

      mockIcpService.call.mockResolvedValue({ Ok: mockSendResult })

      // Act
      const result = await defiService.sendBitcoin(
        '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa',
        100000,
        1000,
        'P2WPKH'
      )

      // Assert
      expect(mockIcpService.call).toHaveBeenCalledWith('send_bitcoin', [
        '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa',
        100000,
        [1000],
        ['P2WPKH']
      ])
      expect(result.success).toBe(true)
      expect(result.transaction_id).toBe('abc123def456')
      expect(result.amount_satoshis).toBe(100000)
    })

    it('should handle insufficient funds error', async () => {
      // Arrange
      mockIcpService.call.mockResolvedValue({ 
        Err: 'Insufficient balance: need 101000 satoshis, have 50000 satoshis' 
      })

      // Act & Assert
      await expect(
        defiService.sendBitcoin('1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa', 100000, 1000)
      ).rejects.toThrow('Insufficient balance')
    })

    it('should send Bitcoin with default fee', async () => {
      // Arrange
      const mockSendResult: BitcoinSendResult = {
        success: true,
        from_address: 'bcrt1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh',
        to_address: '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa',
        amount_satoshis: 100000,
        fee_satoshis: 2000, // Default fee
        change_amount_satoshis: 49898000,
        confirmation_time_estimate_minutes: 30
      }

      mockIcpService.call.mockResolvedValue({ Ok: mockSendResult })

      // Act
      const result = await defiService.sendBitcoin('1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa', 100000)

      // Assert
      expect(mockIcpService.call).toHaveBeenCalledWith('send_bitcoin', [
        '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa',
        100000,
        [],
        []
      ])
      expect(result.fee_satoshis).toBe(2000)
    })

    it('should handle transaction broadcast failure', async () => {
      // Arrange
      const mockSendResult: BitcoinSendResult = {
        success: false,
        from_address: 'bcrt1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh',
        to_address: '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa',
        amount_satoshis: 100000,
        fee_satoshis: 1000,
        change_amount_satoshis: 0,
        confirmation_time_estimate_minutes: 0,
        error_message: 'Transaction rejected by network'
      }

      mockIcpService.call.mockResolvedValue({ Ok: mockSendResult })

      // Act
      const result = await defiService.sendBitcoin('1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa', 100000)

      // Assert
      expect(result.success).toBe(false)
      expect(result.error_message).toBe('Transaction rejected by network')
    })
  })

  describe('Fee Estimation', () => {
    it('should estimate fees for different priorities', async () => {
      // Arrange
      const mockFeeEstimate = {
        total_fee_satoshis: 5000,
        sat_per_byte: 20,
        confirmation_blocks: 3
      }

      mockIcpService.query.mockResolvedValue(mockFeeEstimate)

      // Act
      const estimate = await defiService.estimateBitcoinFee(2, 2, 'High')

      // Assert
      expect(mockIcpService.query).toHaveBeenCalledWith('estimate_bitcoin_fee', [2, 2, 'High'])
      expect(estimate.total_fee_satoshis).toBe(5000)
      expect(estimate.sat_per_byte).toBe(20)
      expect(estimate.confirmation_blocks).toBe(3)
    })

    it('should handle different priority levels', async () => {
      // Test each priority level
      const priorities = ['Low', 'Medium', 'High', 'Urgent'] as const
      const expectedFees = [1250, 2500, 5000, 12500]

      for (let i = 0; i < priorities.length; i++) {
        const priority = priorities[i]
        const expectedFee = expectedFees[i]

        mockIcpService.query.mockResolvedValue({
          total_fee_satoshis: expectedFee,
          sat_per_byte: expectedFee / 250, // Assuming 250 byte transaction
          confirmation_blocks: [144, 6, 3, 1][i]
        })

        const estimate = await defiService.estimateBitcoinFee(2, 2, priority)

        expect(estimate.total_fee_satoshis).toBe(expectedFee)
      }
    })
  })

  describe('Edge Cases and Error Handling', () => {
    it('should handle network timeouts gracefully', async () => {
      // Arrange
      mockIcpService.call.mockRejectedValue(new Error('Network timeout'))

      // Act & Assert
      await expect(defiService.getBitcoinPortfolio()).rejects.toThrow('Network timeout')
    })

    it('should handle malformed responses', async () => {
      // Arrange
      mockIcpService.call.mockResolvedValue({ malformed: 'response' })

      // Act & Assert
      await expect(defiService.getBitcoinPortfolio()).rejects.toThrow()
    })

    it('should validate input parameters', async () => {
      // Act & Assert
      await expect(defiService.sendBitcoin('', 100000)).rejects.toThrow()
      await expect(defiService.sendBitcoin('valid_address', 0)).rejects.toThrow()
      await expect(defiService.sendBitcoin('valid_address', -100)).rejects.toThrow()
    })
  })

  describe('Performance Tests', () => {
    it('should handle large portfolios efficiently', async () => {
      // Arrange
      const largePortfolio = createMockBitcoinPortfolio({
        addresses: Array.from({ length: 100 }, (_, i) => 
          createMockBitcoinAddress({ 
            address: `bcrt1q${i.toString().padStart(50, '0')}`,
            balance_satoshis: Math.floor(Math.random() * 100000000)
          })
        )
      })

      mockIcpService.call.mockResolvedValue({ Ok: largePortfolio })

      // Act
      const startTime = Date.now()
      const portfolio = await defiService.getBitcoinPortfolio()
      const endTime = Date.now()

      // Assert
      expect(portfolio.addresses).toHaveLength(100)
      expect(endTime - startTime).toBeLessThan(1000) // Should complete within 1 second
    })

    it('should handle batch address generation', async () => {
      // Arrange
      const batchAddresses = Array.from({ length: 10 }, (_, i) =>
        createMockBitcoinAddress({ 
          address_type: ['P2PKH', 'P2WPKH', 'P2TR'][i % 3] as any
        })
      )

      mockIcpService.call.mockResolvedValue({ Ok: batchAddresses })

      // Act
      const addresses = await defiService.getAllBitcoinAddresses()

      // Assert
      expect(addresses).toHaveLength(10)
      expect(addresses.filter(a => a.address_type === 'P2PKH')).toHaveLength(4)
      expect(addresses.filter(a => a.address_type === 'P2WPKH')).toHaveLength(3)
      expect(addresses.filter(a => a.address_type === 'P2TR')).toHaveLength(3)
    })
  })
})