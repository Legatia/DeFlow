// Bitcoin DeFi Workflow Nodes Tests
// Tests for Bitcoin node components in the workflow builder

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { createMockWorkflowNode, createMockBitcoinAddress } from '../utils/testUtils'

// Mock Bitcoin DeFi Nodes (these would be actual components in the real app)
interface BitcoinNodeProps {
  nodeId: string
  data: any
  isConnectable: boolean
  selected: boolean
  onNodeDataChange: (nodeId: string, data: any) => void
}

// Bitcoin Portfolio Node Component
const BitcoinPortfolioNode = ({ nodeId, data, onNodeDataChange }: BitcoinNodeProps) => {
  const handleRefresh = () => {
    onNodeDataChange(nodeId, { ...data, lastRefresh: Date.now() })
  }

  return (
    <div data-testid={`bitcoin-portfolio-${nodeId}`} className="bitcoin-node portfolio-node">
      <div className="node-header">
        <span className="node-icon">₿</span>
        <span className="node-title">Bitcoin Portfolio</span>
      </div>
      <div className="node-content">
        <div className="portfolio-summary">
          <div data-testid="total-btc">Total BTC: {data.totalBtc || 0}</div>
          <div data-testid="total-usd">Total USD: ${data.totalValueUsd || 0}</div>
          <div data-testid="address-count">Addresses: {data.addressCount || 0}</div>
        </div>
        <button onClick={handleRefresh} data-testid="refresh-portfolio">
          Refresh Portfolio
        </button>
      </div>
      <div className="node-handles">
        <div className="output-handle" data-testid="portfolio-output">Output</div>
      </div>
    </div>
  )
}

// Bitcoin Send Node Component
const BitcoinSendNode = ({ nodeId, data, onNodeDataChange }: BitcoinNodeProps) => {
  const handleAddressChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onNodeDataChange(nodeId, { ...data, toAddress: e.target.value })
  }

  const handleAmountChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onNodeDataChange(nodeId, { ...data, amountSatoshis: parseInt(e.target.value) || 0 })
  }

  const handleFeeChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    onNodeDataChange(nodeId, { ...data, feePriority: e.target.value })
  }

  return (
    <div data-testid={`bitcoin-send-${nodeId}`} className="bitcoin-node send-node">
      <div className="node-header">
        <span className="node-icon">💸</span>
        <span className="node-title">Send Bitcoin</span>
      </div>
      <div className="node-content">
        <div className="form-group">
          <label htmlFor={`to-address-${nodeId}`}>To Address:</label>
          <input
            id={`to-address-${nodeId}`}
            data-testid="to-address-input"
            type="text"
            value={data.toAddress || ''}
            onChange={handleAddressChange}
            placeholder="Enter Bitcoin address"
          />
        </div>
        <div className="form-group">
          <label htmlFor={`amount-${nodeId}`}>Amount (satoshis):</label>
          <input
            id={`amount-${nodeId}`}
            data-testid="amount-input"
            type="number"
            value={data.amountSatoshis || ''}
            onChange={handleAmountChange}
            placeholder="Amount in satoshis"
          />
        </div>
        <div className="form-group">
          <label htmlFor={`fee-${nodeId}`}>Fee Priority:</label>
          <select
            id={`fee-${nodeId}`}
            data-testid="fee-priority-select"
            value={data.feePriority || 'Medium'}
            onChange={handleFeeChange}
          >
            <option value="Low">Low (~60 min)</option>
            <option value="Medium">Medium (~30 min)</option>
            <option value="High">High (~10 min)</option>
            <option value="Urgent">Urgent (~5 min)</option>
          </select>
        </div>
      </div>
      <div className="node-handles">
        <div className="input-handle" data-testid="send-input">Input</div>
        <div className="output-handle" data-testid="send-output">Output</div>
      </div>
    </div>
  )
}

// Bitcoin Address Node Component
const BitcoinAddressNode = ({ nodeId, data, onNodeDataChange }: BitcoinNodeProps) => {
  const handleAddressTypeChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    onNodeDataChange(nodeId, { ...data, addressType: e.target.value })
  }

  const handleGenerate = () => {
    // Simulate address generation
    const mockAddress = createMockBitcoinAddress({ 
      address_type: data.addressType || 'P2WPKH' 
    })
    onNodeDataChange(nodeId, { 
      ...data, 
      generatedAddress: mockAddress.address,
      generated: true
    })
  }

  return (
    <div data-testid={`bitcoin-address-${nodeId}`} className="bitcoin-node address-node">
      <div className="node-header">
        <span className="node-icon">🏠</span>
        <span className="node-title">Bitcoin Address</span>
      </div>
      <div className="node-content">
        <div className="form-group">
          <label htmlFor={`address-type-${nodeId}`}>Address Type:</label>
          <select
            id={`address-type-${nodeId}`}
            data-testid="address-type-select"
            value={data.addressType || 'P2WPKH'}
            onChange={handleAddressTypeChange}
          >
            <option value="P2PKH">P2PKH (Legacy)</option>
            <option value="P2WPKH">P2WPKH (SegWit)</option>
            <option value="P2TR">P2TR (Taproot)</option>
          </select>
        </div>
        <button onClick={handleGenerate} data-testid="generate-address">
          Generate Address
        </button>
        {data.generated && (
          <div className="generated-address" data-testid="generated-address">
            <strong>Address:</strong> {data.generatedAddress}
          </div>
        )}
      </div>
      <div className="node-handles">
        <div className="output-handle" data-testid="address-output">Output</div>
      </div>
    </div>
  )
}

// Bitcoin Balance Node Component  
const BitcoinBalanceNode = ({ nodeId, data, onNodeDataChange }: BitcoinNodeProps) => {
  const handleAddressChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onNodeDataChange(nodeId, { ...data, checkAddress: e.target.value })
  }

  const handleCheck = () => {
    // Simulate balance check
    const mockBalance = Math.floor(Math.random() * 100000000) // Random balance in satoshis
    onNodeDataChange(nodeId, { 
      ...data, 
      balance: mockBalance,
      balanceBtc: mockBalance / 100000000,
      checked: true
    })
  }

  return (
    <div data-testid={`bitcoin-balance-${nodeId}`} className="bitcoin-node balance-node">
      <div className="node-header">
        <span className="node-icon">💰</span>
        <span className="node-title">Bitcoin Balance</span>
      </div>
      <div className="node-content">
        <div className="form-group">
          <label htmlFor={`check-address-${nodeId}`}>Address to Check:</label>
          <input
            id={`check-address-${nodeId}`}
            data-testid="check-address-input"
            type="text"
            value={data.checkAddress || ''}
            onChange={handleAddressChange}
            placeholder="Enter Bitcoin address"
          />
        </div>
        <button onClick={handleCheck} data-testid="check-balance">
          Check Balance
        </button>
        {data.checked && (
          <div className="balance-result" data-testid="balance-result">
            <div>Balance: {data.balance} satoshis</div>
            <div>BTC: {data.balanceBtc}</div>
          </div>
        )}
      </div>
      <div className="node-handles">
        <div className="input-handle" data-testid="balance-input">Input</div>
        <div className="output-handle" data-testid="balance-output">Output</div>
      </div>
    </div>
  )
}

describe('Bitcoin DeFi Workflow Nodes', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  describe('Bitcoin Portfolio Node', () => {
    it('should render portfolio node correctly', () => {
      // Arrange
      const mockData = {
        totalBtc: 0.5,
        totalValueUsd: 22500,
        addressCount: 3
      }
      const mockOnChange = vi.fn()

      // Act
      render(
        <BitcoinPortfolioNode
          nodeId="portfolio-1"
          data={mockData}
          isConnectable={true}
          selected={false}
          onNodeDataChange={mockOnChange}
        />
      )

      // Assert
      expect(screen.getByTestId('bitcoin-portfolio-portfolio-1')).toBeInTheDocument()
      expect(screen.getByTestId('total-btc')).toHaveTextContent('Total BTC: 0.5')
      expect(screen.getByTestId('total-usd')).toHaveTextContent('Total USD: $22500')
      expect(screen.getByTestId('address-count')).toHaveTextContent('Addresses: 3')
    })

    it('should handle portfolio refresh', () => {
      // Arrange
      const mockData = { totalBtc: 0.5 }
      const mockOnChange = vi.fn()

      render(
        <BitcoinPortfolioNode
          nodeId="portfolio-1"
          data={mockData}
          isConnectable={true}
          selected={false}
          onNodeDataChange={mockOnChange}
        />
      )

      // Act
      fireEvent.click(screen.getByTestId('refresh-portfolio'))

      // Assert
      expect(mockOnChange).toHaveBeenCalledWith('portfolio-1', {
        ...mockData,
        lastRefresh: expect.any(Number)
      })
    })

    it('should display zero values for empty portfolio', () => {
      // Arrange
      render(
        <BitcoinPortfolioNode
          nodeId="portfolio-1"
          data={{}}
          isConnectable={true}
          selected={false}
          onNodeDataChange={vi.fn()}
        />
      )

      // Assert
      expect(screen.getByTestId('total-btc')).toHaveTextContent('Total BTC: 0')
      expect(screen.getByTestId('total-usd')).toHaveTextContent('Total USD: $0')
      expect(screen.getByTestId('address-count')).toHaveTextContent('Addresses: 0')
    })
  })

  describe('Bitcoin Send Node', () => {
    it('should render send node correctly', () => {
      // Arrange
      const mockData = {
        toAddress: '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa',
        amountSatoshis: 100000,
        feePriority: 'High'
      }

      // Act
      render(
        <BitcoinSendNode
          nodeId="send-1"
          data={mockData}
          isConnectable={true}
          selected={false}
          onNodeDataChange={vi.fn()}
        />
      )

      // Assert
      expect(screen.getByTestId('bitcoin-send-send-1')).toBeInTheDocument()
      expect(screen.getByTestId('to-address-input')).toHaveValue('1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa')
      expect(screen.getByTestId('amount-input')).toHaveValue(100000)
      expect(screen.getByTestId('fee-priority-select')).toHaveValue('High')
    })

    it('should handle address input changes', () => {
      // Arrange
      const mockOnChange = vi.fn()
      
      render(
        <BitcoinSendNode
          nodeId="send-1"
          data={{}}
          isConnectable={true}
          selected={false}
          onNodeDataChange={mockOnChange}
        />
      )

      // Act
      fireEvent.change(screen.getByTestId('to-address-input'), {
        target: { value: '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa' }
      })

      // Assert
      expect(mockOnChange).toHaveBeenCalledWith('send-1', {
        toAddress: '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa'
      })
    })

    it('should handle amount input changes', () => {
      // Arrange
      const mockOnChange = vi.fn()
      
      render(
        <BitcoinSendNode
          nodeId="send-1"
          data={{}}
          isConnectable={true}
          selected={false}
          onNodeDataChange={mockOnChange}
        />
      )

      // Act
      fireEvent.change(screen.getByTestId('amount-input'), {
        target: { value: '50000' }
      })

      // Assert
      expect(mockOnChange).toHaveBeenCalledWith('send-1', {
        amountSatoshis: 50000
      })
    })

    it('should handle fee priority selection', () => {
      // Arrange
      const mockOnChange = vi.fn()
      
      render(
        <BitcoinSendNode
          nodeId="send-1"
          data={{}}
          isConnectable={true}
          selected={false}
          onNodeDataChange={mockOnChange}
        />
      )

      // Act
      fireEvent.change(screen.getByTestId('fee-priority-select'), {
        target: { value: 'Urgent' }
      })

      // Assert
      expect(mockOnChange).toHaveBeenCalledWith('send-1', {
        feePriority: 'Urgent'
      })
    })

    it('should handle invalid amount input gracefully', () => {
      // Arrange
      const mockOnChange = vi.fn()
      
      render(
        <BitcoinSendNode
          nodeId="send-1"
          data={{}}
          isConnectable={true}
          selected={false}
          onNodeDataChange={mockOnChange}
        />
      )

      // Act
      fireEvent.change(screen.getByTestId('amount-input'), {
        target: { value: 'invalid' }
      })

      // Assert
      expect(mockOnChange).toHaveBeenCalledWith('send-1', {
        amountSatoshis: 0
      })
    })
  })

  describe('Bitcoin Address Node', () => {
    it('should render address node correctly', () => {
      // Arrange
      render(
        <BitcoinAddressNode
          nodeId="address-1"
          data={{ addressType: 'P2WPKH' }}
          isConnectable={true}
          selected={false}
          onNodeDataChange={vi.fn()}
        />
      )

      // Assert
      expect(screen.getByTestId('bitcoin-address-address-1')).toBeInTheDocument()
      expect(screen.getByTestId('address-type-select')).toHaveValue('P2WPKH')
      expect(screen.getByTestId('generate-address')).toBeInTheDocument()
    })

    it('should handle address type selection', () => {
      // Arrange
      const mockOnChange = vi.fn()
      
      render(
        <BitcoinAddressNode
          nodeId="address-1"
          data={{}}
          isConnectable={true}
          selected={false}
          onNodeDataChange={mockOnChange}
        />
      )

      // Act
      fireEvent.change(screen.getByTestId('address-type-select'), {
        target: { value: 'P2TR' }
      })

      // Assert
      expect(mockOnChange).toHaveBeenCalledWith('address-1', {
        addressType: 'P2TR'
      })
    })

    it('should generate address when button clicked', () => {
      // Arrange
      const mockOnChange = vi.fn()
      
      render(
        <BitcoinAddressNode
          nodeId="address-1"
          data={{ addressType: 'P2WPKH' }}
          isConnectable={true}
          selected={false}
          onNodeDataChange={mockOnChange}
        />
      )

      // Act
      fireEvent.click(screen.getByTestId('generate-address'))

      // Assert
      expect(mockOnChange).toHaveBeenCalledWith('address-1', {
        addressType: 'P2WPKH',
        generatedAddress: expect.stringMatching(/^bcrt1q/),
        generated: true
      })
    })

    it('should display generated address', () => {
      // Arrange
      const mockData = {
        addressType: 'P2WPKH',
        generatedAddress: 'bcrt1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh',
        generated: true
      }
      
      render(
        <BitcoinAddressNode
          nodeId="address-1"
          data={mockData}
          isConnectable={true}
          selected={false}
          onNodeDataChange={vi.fn()}
        />
      )

      // Assert
      expect(screen.getByTestId('generated-address')).toBeInTheDocument()
      expect(screen.getByTestId('generated-address')).toHaveTextContent(
        'bcrt1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh'
      )
    })
  })

  describe('Bitcoin Balance Node', () => {
    it('should render balance node correctly', () => {
      // Arrange
      render(
        <BitcoinBalanceNode
          nodeId="balance-1"
          data={{}}
          isConnectable={true}
          selected={false}
          onNodeDataChange={vi.fn()}
        />
      )

      // Assert
      expect(screen.getByTestId('bitcoin-balance-balance-1')).toBeInTheDocument()
      expect(screen.getByTestId('check-address-input')).toBeInTheDocument()
      expect(screen.getByTestId('check-balance')).toBeInTheDocument()
    })

    it('should handle address input for balance check', () => {
      // Arrange
      const mockOnChange = vi.fn()
      
      render(
        <BitcoinBalanceNode
          nodeId="balance-1"
          data={{}}
          isConnectable={true}
          selected={false}
          onNodeDataChange={mockOnChange}
        />
      )

      // Act
      fireEvent.change(screen.getByTestId('check-address-input'), {
        target: { value: '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa' }
      })

      // Assert
      expect(mockOnChange).toHaveBeenCalledWith('balance-1', {
        checkAddress: '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa'
      })
    })

    it('should check balance when button clicked', () => {
      // Arrange
      const mockOnChange = vi.fn()
      
      render(
        <BitcoinBalanceNode
          nodeId="balance-1"
          data={{ checkAddress: '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa' }}
          isConnectable={true}
          selected={false}
          onNodeDataChange={mockOnChange}
        />
      )

      // Act
      fireEvent.click(screen.getByTestId('check-balance'))

      // Assert
      expect(mockOnChange).toHaveBeenCalledWith('balance-1', {
        checkAddress: '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa',
        balance: expect.any(Number),
        balanceBtc: expect.any(Number),
        checked: true
      })
    })

    it('should display balance results', () => {
      // Arrange
      const mockData = {
        checkAddress: '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa',
        balance: 50000000,
        balanceBtc: 0.5,
        checked: true
      }
      
      render(
        <BitcoinBalanceNode
          nodeId="balance-1"
          data={mockData}
          isConnectable={true}
          selected={false}
          onNodeDataChange={vi.fn()}
        />
      )

      // Assert
      expect(screen.getByTestId('balance-result')).toBeInTheDocument()
      expect(screen.getByTestId('balance-result')).toHaveTextContent('Balance: 50000000 satoshis')
      expect(screen.getByTestId('balance-result')).toHaveTextContent('BTC: 0.5')
    })
  })

  describe('Node Integration', () => {
    it('should handle node data updates correctly', () => {
      // Arrange
      const mockOnChange = vi.fn()
      
      render(
        <BitcoinSendNode
          nodeId="send-1"
          data={{}}
          isConnectable={true}
          selected={false}
          onNodeDataChange={mockOnChange}
        />
      )

      // Act - Multiple field updates
      fireEvent.change(screen.getByTestId('to-address-input'), {
        target: { value: '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa' }
      })
      fireEvent.change(screen.getByTestId('amount-input'), {
        target: { value: '100000' }
      })
      fireEvent.change(screen.getByTestId('fee-priority-select'), {
        target: { value: 'High' }
      })

      // Assert
      expect(mockOnChange).toHaveBeenCalledTimes(3)
      expect(mockOnChange).toHaveBeenNthCalledWith(1, 'send-1', {
        toAddress: '1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa'
      })
      expect(mockOnChange).toHaveBeenNthCalledWith(2, 'send-1', {
        amountSatoshis: 100000
      })
      expect(mockOnChange).toHaveBeenNthCalledWith(3, 'send-1', {
        feePriority: 'High'
      })
    })

    it('should maintain node state between renders', () => {
      // Arrange
      let nodeData = { toAddress: 'initial' }
      const { rerender } = render(
        <BitcoinSendNode
          nodeId="send-1"
          data={nodeData}
          isConnectable={true}
          selected={false}
          onNodeDataChange={(id, data) => { nodeData = data }}
        />
      )

      // Act - Update data and rerender
      fireEvent.change(screen.getByTestId('to-address-input'), {
        target: { value: 'updated_address' }
      })
      
      rerender(
        <BitcoinSendNode
          nodeId="send-1"
          data={nodeData}
          isConnectable={true}
          selected={false}
          onNodeDataChange={(id, data) => { nodeData = data }}
        />
      )

      // Assert
      expect(screen.getByTestId('to-address-input')).toHaveValue('updated_address')
    })
  })

  describe('Accessibility', () => {
    it('should have proper ARIA labels and roles', () => {
      // Arrange & Act
      render(
        <BitcoinSendNode
          nodeId="send-1"
          data={{}}
          isConnectable={true}
          selected={false}
          onNodeDataChange={vi.fn()}
        />
      )

      // Assert
      expect(screen.getByLabelText('To Address:')).toBeInTheDocument()
      expect(screen.getByLabelText('Amount (satoshis):')).toBeInTheDocument()
      expect(screen.getByLabelText('Fee Priority:')).toBeInTheDocument()
    })

    it('should support keyboard navigation', () => {
      // Arrange
      render(
        <BitcoinAddressNode
          nodeId="address-1"
          data={{}}
          isConnectable={true}
          selected={false}
          onNodeDataChange={vi.fn()}
        />
      )

      // Act & Assert
      const generateButton = screen.getByTestId('generate-address')
      generateButton.focus()
      expect(generateButton).toHaveFocus()
    })
  })
})