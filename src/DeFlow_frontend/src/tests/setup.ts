// Test setup and configuration for DeFlow
import { vi } from 'vitest'
import { expect, afterEach } from 'vitest'
import { cleanup } from '@testing-library/react'
import * as matchers from '@testing-library/jest-dom/matchers'

// Extend Vitest's expect with jest-dom matchers
expect.extend(matchers)

// Mock localStorage
const localStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
}
Object.defineProperty(window, 'localStorage', {
  value: localStorageMock
})

// Mock sessionStorage
const sessionStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
}
Object.defineProperty(window, 'sessionStorage', {
  value: sessionStorageMock
})

// Mock ResizeObserver
global.ResizeObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}))

// Mock IntersectionObserver
global.IntersectionObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}))

// Mock matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(), // deprecated
    removeListener: vi.fn(), // deprecated
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
})

// Mock crypto for testing
Object.defineProperty(global, 'crypto', {
  value: {
    randomUUID: vi.fn(() => 'mock-uuid-' + Math.random().toString(36).substr(2, 9)),
    getRandomValues: vi.fn(() => new Uint32Array(10))
  }
})

// Mock fetch
global.fetch = vi.fn()

// Mock React Flow
vi.mock('reactflow', () => ({
  useNodesState: vi.fn(() => [[], vi.fn(), vi.fn()]),
  useEdgesState: vi.fn(() => [[], vi.fn(), vi.fn()]),
  useReactFlow: vi.fn(() => ({
    getNodes: vi.fn(() => []),
    getEdges: vi.fn(() => []),
    setNodes: vi.fn(),
    setEdges: vi.fn(),
    addNodes: vi.fn(),
    addEdges: vi.fn(),
    getNode: vi.fn(),
    getEdge: vi.fn(),
    project: vi.fn(),
    screenToFlowPosition: vi.fn(),
    flowToScreenPosition: vi.fn(),
    fitView: vi.fn(),
    zoomIn: vi.fn(),
    zoomOut: vi.fn(),
    zoomTo: vi.fn(),
    getZoom: vi.fn(() => 1),
    setViewport: vi.fn(),
    getViewport: vi.fn(() => ({ x: 0, y: 0, zoom: 1 })),
  })),
  ReactFlow: vi.fn(({ children, ...props }) => 
    React.createElement('div', { 'data-testid': 'react-flow', ...props }, children)
  ),
  Controls: vi.fn(() => React.createElement('div', { 'data-testid': 'react-flow-controls' })),
  Background: vi.fn(() => React.createElement('div', { 'data-testid': 'react-flow-background' })),
  MiniMap: vi.fn(() => React.createElement('div', { 'data-testid': 'react-flow-minimap' })),
  Handle: vi.fn(({ type, position, ...props }) => 
    React.createElement('div', { 
      'data-testid': `react-flow-handle-${type}-${position}`, 
      className: `react-flow__handle react-flow__handle-${type} react-flow__handle-${position}`,
      ...props 
    })
  ),
  Position: {
    Top: 'top',
    Right: 'right', 
    Bottom: 'bottom',
    Left: 'left'
  },
  MarkerType: {
    Arrow: 'arrow',
    ArrowClosed: 'arrowclosed'
  }
}))

// Import React for the mocks
import React from 'react'

// Mock console methods to reduce noise in tests
global.console = {
  ...console,
  // Uncomment to suppress console logs in tests
  // log:  vi.fn(),
  // debug: vi.fn(),
  // info: vi.fn(),
  warn: vi.fn(),
  error: vi.fn(),
}

// Cleanup after each test
afterEach(() => {
  cleanup()
  vi.clearAllMocks()
  localStorageMock.clear()
  sessionStorageMock.clear()
})

// Set up test environment variables
process.env.NODE_ENV = 'test'