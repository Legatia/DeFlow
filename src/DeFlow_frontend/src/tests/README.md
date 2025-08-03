# DeFlow Testing Framework

This directory contains the comprehensive testing framework for the DeFlow workflow automation platform.

## Overview

The testing framework includes:
- **Unit Tests**: Individual component and service testing
- **Integration Tests**: End-to-end workflow execution testing
- **Utilities**: Shared testing helpers and mock factories
- **Setup**: Test environment configuration

## Test Structure

```
src/tests/
├── components/           # Component tests
│   └── WorkflowBuilder.test.tsx
├── services/            # Service layer tests
│   ├── executionEngine.test.ts
│   └── authService.test.ts
├── integration/         # Integration tests
│   └── workflow-execution.test.ts
├── utils/              # Testing utilities
│   └── testUtils.tsx
├── setup.ts            # Test setup configuration
└── README.md           # This file
```

## Running Tests

### All Tests
```bash
npm test
```

### Watch Mode
```bash
npm run test:watch
```

### Coverage Report
```bash
npm run test:coverage
```

### Specific Test Files
```bash
# Run component tests
npm test -- --testPathPattern=components

# Run service tests
npm test -- --testPathPattern=services

# Run integration tests
npm test -- --testPathPattern=integration
```

## Test Categories

### 1. Component Tests
- **WorkflowBuilder**: Visual workflow editor testing
- **NodePalette**: Drag-and-drop node palette
- **NodeConfigPanel**: Node configuration interface
- **WorkflowTemplates**: Pre-built workflow templates

### 2. Service Tests
- **ExecutionEngine**: Workflow execution logic
- **AuthService**: Authentication and authorization
- **MonitoringService**: Performance and health monitoring
- **WebhookService**: External integration webhooks
- **CollaborationService**: Workflow sharing features

### 3. Integration Tests
- **Workflow Execution**: End-to-end execution flows
- **Real-time Updates**: WebSocket communication
- **Webhook Integration**: External trigger handling
- **Multi-user Scenarios**: Collaboration workflows

## Mock Data Factories

The testing framework provides factory functions for creating mock data:

```typescript
// Create mock workflow
const workflow = createMockWorkflow({
  name: 'Test Workflow',
  nodes: [...],
  connections: [...]
})

// Create mock user
const user = createMockUser({
  email: 'test@example.com',
  role: 'admin'
})

// Create mock execution
const execution = createMockWorkflowExecution({
  status: 'completed'
})
```

## Testing Utilities

### Custom Render
```typescript
import { render } from '@tests/utils/testUtils'

render(<MyComponent />) // Includes all providers
```

### Form Testing
```typescript
import { fillForm, submitForm } from '@tests/utils/testUtils'

fillForm(container, {
  email: 'test@example.com',
  password: 'password123'
})
submitForm(container)
```

### API Mocking
```typescript
import { mockApiResponse, mockApiError } from '@tests/utils/testUtils'

// Mock successful response
const data = await mockApiResponse({ success: true })

// Mock error response
await mockApiError('API Error', 1000) // 1 second delay
```

## Test Environment Setup

The test environment includes:
- **JSDOM**: Browser environment simulation
- **React Testing Library**: Component testing utilities
- **Vitest**: Fast test runner with TypeScript support
- **Mock Services**: Isolated service testing
- **Coverage Reports**: Code coverage analysis

## Coverage Requirements

Minimum coverage thresholds:
- **Branches**: 80%
- **Functions**: 80%
- **Lines**: 80%
- **Statements**: 80%

## Best Practices

### 1. Test Naming
```typescript
describe('ComponentName', () => {
  describe('Feature Group', () => {
    it('should do something specific', () => {
      // Test implementation
    })
  })
})
```

### 2. Arrange-Act-Assert Pattern
```typescript
it('should calculate workflow duration', () => {
  // Arrange
  const startTime = '2024-01-01T10:00:00Z'
  const endTime = '2024-01-01T10:01:00Z'
  
  // Act
  const duration = calculateDuration(startTime, endTime)
  
  // Assert
  expect(duration).toBe(60000000000) // 60 seconds in nanoseconds
})
```

### 3. Mock Management
```typescript
beforeEach(() => {
  vi.clearAllMocks() // Clear all mocks before each test
})
```

### 4. Async Testing
```typescript
it('should handle async operations', async () => {
  const result = await asyncOperation()
  
  await waitFor(() => {
    expect(screen.getByText('Success')).toBeInTheDocument()
  })
})
```

## Performance Testing

The framework includes performance testing utilities:

```typescript
import { measurePerformance } from '@tests/utils/testUtils'

it('should render large workflows efficiently', async () => {
  const renderTime = await measurePerformance(() => {
    render(<WorkflowBuilder nodes={manyNodes} />)
  })
  
  expect(renderTime).toBeLessThan(1000) // Less than 1 second
})
```

## Accessibility Testing

```typescript
import { checkA11y } from '@tests/utils/testUtils'

it('should be accessible', async () => {
  const { container } = render(<MyComponent />)
  const violations = await checkA11y(container)
  
  expect(violations).toHaveLength(0)
})
```

## Debugging Tests

### Visual Debugging
```typescript
import { screen } from '@testing-library/react'

// Print current DOM state
screen.debug()

// Print specific element
screen.debug(screen.getByRole('button'))
```

### Test Data Inspection
```typescript
// Log test data for debugging
console.log('Execution result:', JSON.stringify(execution, null, 2))
```

## Continuous Integration

The testing framework integrates with CI/CD pipelines:
- **Automated Test Runs**: On every pull request
- **Coverage Reports**: Uploaded to coverage services
- **Test Results**: JUnit XML format for CI integration
- **Performance Monitoring**: Track test execution times

## Common Issues and Solutions

### 1. React Flow Testing
React Flow components require special mocking due to their complexity. Use the provided mocks in the test setup.

### 2. Timing Issues
Use `waitFor` for async operations and avoid `act` warnings:
```typescript
await waitFor(() => {
  expect(screen.getByText('Loaded')).toBeInTheDocument()
})
```

### 3. Mock Cleanup
Always clear mocks between tests to avoid state leakage:
```typescript
afterEach(() => {
  vi.clearAllMocks()
  cleanup()
})
```

## Contributing

When adding new tests:
1. Follow the existing naming conventions
2. Include both positive and negative test cases
3. Mock external dependencies
4. Maintain good test coverage
5. Update this documentation if needed

## Test Scripts

Available npm scripts:
- `npm test` - Run all tests
- `npm run test:watch` - Run tests in watch mode
- `npm run test:coverage` - Generate coverage report
- `npm run test:ui` - Open Vitest UI for interactive testing
- `npm run test:run` - Run tests once (CI mode)