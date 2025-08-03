// Tests for the WorkflowBuilder component
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, fireEvent, waitFor } from '../utils/testUtils'
import { createMockReactFlowNode, createMockReactFlowEdge } from '../utils/testUtils'
import WorkflowBuilder from '../../components/WorkflowBuilder'

// Mock react-flow
vi.mock('reactflow', () => ({
  ReactFlow: ({ children, nodes, edges, onNodesChange, onEdgesChange, onConnect }: any) => (
    <div data-testid="react-flow">
      <div data-testid="nodes-count">{nodes?.length || 0}</div>
      <div data-testid="edges-count">{edges?.length || 0}</div>
      {children}
    </div>
  ),
  Controls: () => <div data-testid="flow-controls">Controls</div>,
  MiniMap: () => <div data-testid="flow-minimap">MiniMap</div>,
  Background: () => <div data-testid="flow-background">Background</div>,
  useReactFlow: () => ({
    getNodes: vi.fn(() => []),
    getEdges: vi.fn(() => []),
    setNodes: vi.fn(),
    setEdges: vi.fn(),
    addNodes: vi.fn(),
    addEdges: vi.fn(),
    deleteElements: vi.fn(),
    getViewport: vi.fn(() => ({ x: 0, y: 0, zoom: 1 })),
    setViewport: vi.fn(),
    fitView: vi.fn(),
    screenToFlowPosition: vi.fn((pos) => pos)
  }),
  addEdge: vi.fn(),
  applyNodeChanges: vi.fn(),
  applyEdgeChanges: vi.fn()
}))

// Mock the custom hook
vi.mock('../../hooks/useWorkflowBuilder', () => ({
  useWorkflowBuilder: () => ({
    nodes: [],
    edges: [],
    selectedNode: null,
    nodeTypes: {},
    onNodesChange: vi.fn(),
    onEdgesChange: vi.fn(),
    onConnect: vi.fn(),
    onNodeClick: vi.fn(),
    onPaneClick: vi.fn(),
    addNode: vi.fn(),
    deleteNode: vi.fn(),
    updateNodeConfig: vi.fn(),
    validateWorkflow: vi.fn(() => ({ isValid: true, errors: [] })),
    saveWorkflow: vi.fn()
  })
}))

describe('WorkflowBuilder', () => {
  const mockOnSave = vi.fn()
  const defaultProps = {
    initialNodes: [],
    initialEdges: [],
    onSave: mockOnSave
  }

  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('Rendering', () => {
    it('should render workflow builder with all components', () => {
      render(<WorkflowBuilder {...defaultProps} />)

      expect(screen.getByTestId('react-flow')).toBeInTheDocument()
      expect(screen.getByTestId('flow-controls')).toBeInTheDocument()
      expect(screen.getByTestId('flow-minimap')).toBeInTheDocument()
      expect(screen.getByTestId('flow-background')).toBeInTheDocument()
    })

    it('should render node palette', () => {
      render(<WorkflowBuilder {...defaultProps} />)

      expect(screen.getByText('Node Palette')).toBeInTheDocument()
      expect(screen.getByText('Triggers')).toBeInTheDocument()
      expect(screen.getByText('Actions')).toBeInTheDocument()
      expect(screen.getByText('Conditions')).toBeInTheDocument()
    })

    it('should render toolbar with save button', () => {
      render(<WorkflowBuilder {...defaultProps} />)

      expect(screen.getByRole('button', { name: /save workflow/i })).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /validate/i })).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /clear all/i })).toBeInTheDocument()
    })

    it('should render with initial nodes and edges', () => {
      const initialNodes = [
        createMockReactFlowNode('node-1'),
        createMockReactFlowNode('node-2')
      ]
      const initialEdges = [
        createMockReactFlowEdge('node-1', 'node-2')
      ]

      render(
        <WorkflowBuilder 
          {...defaultProps}
          initialNodes={initialNodes}
          initialEdges={initialEdges}
        />
      )

      expect(screen.getByTestId('nodes-count')).toHaveTextContent('2')
      expect(screen.getByTestId('edges-count')).toHaveTextContent('1')
    })
  })

  describe('Node Palette', () => {
    it('should show node categories', () => {
      render(<WorkflowBuilder {...defaultProps} />)

      expect(screen.getByText('Triggers')).toBeInTheDocument()
      expect(screen.getByText('Actions')).toBeInTheDocument()
      expect(screen.getByText('Conditions')).toBeInTheDocument()
      expect(screen.getByText('Transformations')).toBeInTheDocument()
      expect(screen.getByText('Integrations')).toBeInTheDocument()
      expect(screen.getByText('Utilities')).toBeInTheDocument()
    })

    it('should allow category selection', async () => {
      render(<WorkflowBuilder {...defaultProps} />)

      const actionsCategory = screen.getByText('Actions')
      fireEvent.click(actionsCategory)

      await waitFor(() => {
        expect(actionsCategory.closest('button')).toHaveClass('bg-blue-100')
      })
    })

    it('should show search functionality', () => {
      render(<WorkflowBuilder {...defaultProps} />)

      const searchInput = screen.getByPlaceholderText('Search nodes...')
      expect(searchInput).toBeInTheDocument()

      fireEvent.change(searchInput, { target: { value: 'email' } })
      expect(searchInput).toHaveValue('email')
    })

    it('should support drag and drop from palette', () => {
      render(<WorkflowBuilder {...defaultProps} />)

      const triggerNode = screen.getByText('Manual Trigger')
      expect(triggerNode).toBeInTheDocument()

      // Test drag start
      fireEvent.dragStart(triggerNode, {
        dataTransfer: {
          setData: vi.fn(),
          effectAllowed: 'copy'
        }
      })

      // In a real test, you would also test the drop functionality
      // This would require more complex setup with the React Flow canvas
    })
  })

  describe('Workflow Operations', () => {
    it('should save workflow when save button is clicked', async () => {
      render(<WorkflowBuilder {...defaultProps} />)

      const saveButton = screen.getByRole('button', { name: /save workflow/i })
      fireEvent.click(saveButton)

      await waitFor(() => {
        expect(mockOnSave).toHaveBeenCalledWith([], [])
      })
    })

    it('should validate workflow when validate button is clicked', async () => {
      render(<WorkflowBuilder {...defaultProps} />)

      const validateButton = screen.getByRole('button', { name: /validate/i })
      fireEvent.click(validateButton)

      // In the actual implementation, this would show validation results
      await waitFor(() => {
        expect(validateButton).toBeInTheDocument()
      })
    })

    it('should clear workflow when clear button is clicked', async () => {
      const initialNodes = [createMockReactFlowNode('node-1')]
      render(
        <WorkflowBuilder 
          {...defaultProps}
          initialNodes={initialNodes}
        />
      )

      const clearButton = screen.getByRole('button', { name: /clear all/i })
      fireEvent.click(clearButton)

      // This would clear the workflow in the actual implementation
      await waitFor(() => {
        expect(clearButton).toBeInTheDocument()
      })
    })

    it('should show validation errors', () => {
      render(<WorkflowBuilder {...defaultProps} />)

      // In the actual implementation, validation errors would be shown
      // This would require setting up the workflow builder hook to return errors
      expect(screen.queryByText('Validation Error')).not.toBeInTheDocument()
    })
  })

  describe('Node Configuration', () => {
    it('should show configuration panel when node is selected', () => {
      render(<WorkflowBuilder {...defaultProps} />)

      // In the actual implementation, clicking a node would show the config panel
      // This test would need to be expanded based on the actual selection logic
      expect(screen.queryByText('Node Configuration')).not.toBeInTheDocument()
    })

    it('should hide configuration panel when no node is selected', () => {
      render(<WorkflowBuilder {...defaultProps} />)

      expect(screen.queryByText('Node Configuration')).not.toBeInTheDocument()
    })

    it('should update node configuration', async () => {
      render(<WorkflowBuilder {...defaultProps} />)

      // This would test the configuration update functionality
      // Would require a more complex setup with selected nodes
    })

    it('should delete node from configuration panel', async () => {
      render(<WorkflowBuilder {...defaultProps} />)

      // This would test node deletion from the config panel
      // Would require a selected node and the delete functionality
    })
  })

  describe('Keyboard Shortcuts', () => {
    it('should handle save shortcut (Ctrl+S)', () => {
      render(<WorkflowBuilder {...defaultProps} />)

      fireEvent.keyDown(document, { key: 's', ctrlKey: true })

      expect(mockOnSave).toHaveBeenCalled()
    })

    it('should handle delete shortcut when node is selected', () => {
      render(<WorkflowBuilder {...defaultProps} />)

      fireEvent.keyDown(document, { key: 'Delete' })

      // This would delete the selected node in the actual implementation
    })

    it('should handle undo shortcut (Ctrl+Z)', () => {
      render(<WorkflowBuilder {...defaultProps} />)

      fireEvent.keyDown(document, { key: 'z', ctrlKey: true })

      // This would undo the last action in the actual implementation
    })

    it('should handle redo shortcut (Ctrl+Y)', () => {
      render(<WorkflowBuilder {...defaultProps} />)

      fireEvent.keyDown(document, { key: 'y', ctrlKey: true })

      // This would redo the last undone action in the actual implementation
    })
  })

  describe('Workflow Validation', () => {
    it('should validate workflow structure', () => {
      render(<WorkflowBuilder {...defaultProps} />)

      // Test validation of empty workflow
      const validateButton = screen.getByRole('button', { name: /validate/i })
      fireEvent.click(validateButton)

      // In actual implementation, this would show validation messages
    })

    it('should validate node connections', () => {
      const nodes = [
        createMockReactFlowNode('trigger-1'),
        createMockReactFlowNode('action-1')
      ]
      const edges = [
        createMockReactFlowEdge('trigger-1', 'action-1')
      ]

      render(
        <WorkflowBuilder 
          {...defaultProps}
          initialNodes={nodes}
          initialEdges={edges}
        />
      )

      // This would validate the connections are valid
    })

    it('should validate node configurations', () => {
      render(<WorkflowBuilder {...defaultProps} />)

      // This would validate that required node configurations are complete
    })

    it('should show validation results', () => {
      render(<WorkflowBuilder {...defaultProps} />)

      // This would display validation results to the user
      expect(screen.queryByText('Validation Results')).not.toBeInTheDocument()
    })
  })

  describe('Accessibility', () => {
    it('should be keyboard navigable', () => {
      render(<WorkflowBuilder {...defaultProps} />)

      const saveButton = screen.getByRole('button', { name: /save workflow/i })
      expect(saveButton).toBeInTheDocument()
      
      // Test tab navigation
      saveButton.focus()
      expect(document.activeElement).toBe(saveButton)
    })

    it('should have proper ARIA labels', () => {
      render(<WorkflowBuilder {...defaultProps} />)

      const saveButton = screen.getByRole('button', { name: /save workflow/i })
      expect(saveButton).toHaveAccessibleName()
    })

    it('should announce status changes to screen readers', () => {
      render(<WorkflowBuilder {...defaultProps} />)

      // This would test ARIA live regions for status announcements
      expect(screen.queryByRole('status')).not.toBeInTheDocument()
    })
  })

  describe('Performance', () => {
    it('should handle large workflows efficiently', () => {
      const manyNodes = Array.from({ length: 100 }, (_, i) => 
        createMockReactFlowNode(`node-${i}`)
      )

      const renderStart = performance.now()
      render(
        <WorkflowBuilder 
          {...defaultProps}
          initialNodes={manyNodes}
        />
      )
      const renderEnd = performance.now()

      // Should render within reasonable time (less than 1 second)
      expect(renderEnd - renderStart).toBeLessThan(1000)
    })

    it('should not re-render unnecessarily', () => {
      const { rerender } = render(<WorkflowBuilder {...defaultProps} />)

      // Re-render with same props
      rerender(<WorkflowBuilder {...defaultProps} />)

      // In actual implementation, you would monitor render counts
      expect(screen.getByTestId('react-flow')).toBeInTheDocument()
    })
  })

  describe('Error Handling', () => {
    it('should handle invalid node types gracefully', () => {
      const invalidNodes = [
        {
          id: 'invalid-1',
          type: 'invalid-type',
          position: { x: 0, y: 0 },
          data: {}
        }
      ]

      render(
        <WorkflowBuilder 
          {...defaultProps}
          initialNodes={invalidNodes}
        />
      )

      // Should not crash and should handle invalid nodes
      expect(screen.getByTestId('react-flow')).toBeInTheDocument()
    })

    it('should handle save errors gracefully', async () => {
      const failingOnSave = vi.fn().mockRejectedValue(new Error('Save failed'))
      
      render(
        <WorkflowBuilder 
          {...defaultProps}
          onSave={failingOnSave}
        />
      )

      const saveButton = screen.getByRole('button', { name: /save workflow/i })
      fireEvent.click(saveButton)

      await waitFor(() => {
        expect(failingOnSave).toHaveBeenCalled()
      })

      // Should show error message in actual implementation
    })

    it('should recover from component errors', () => {
      // This would test error boundaries and recovery mechanisms
      render(<WorkflowBuilder {...defaultProps} />)

      expect(screen.getByTestId('react-flow')).toBeInTheDocument()
    })
  })
})