# DeFlow Frontend Testing Documentation

This document outlines the comprehensive testing strategy for the DeFlow frontend application.

## Testing Framework Overview

### Unit Tests (Vitest + Testing Library)
- **Framework**: Vitest with React Testing Library
- **Purpose**: Test individual components, utilities, and stores
- **Location**: `src/**/__tests__/*.test.ts(x)`

### Integration Tests (Vitest + Testing Library)
- **Framework**: Vitest with React Testing Library
- **Purpose**: Test component interactions and store integrations
- **Location**: `src/**/__tests__/*.test.ts(x)`

### End-to-End Tests (Playwright)
- **Framework**: Playwright
- **Purpose**: Test complete user workflows and app functionality
- **Location**: `e2e/*.spec.ts`

## Test Categories

### 1. Utility Function Tests
**Location**: `src/utils/__tests__/bigint.test.ts`

Tests the BigInt utility functions that handle safe conversions between BigInt and number types:
- `bigintToNumber()` - Converts BigInt to number with safety checks
- `numberToBigint()` - Converts number to BigInt with proper flooring
- `timestampToBigint()` - Converts timestamps to ICP-compatible nanoseconds
- `bigintToTimestamp()` - Converts nanoseconds back to milliseconds
- `formatBigintTimestamp()` - Formats BigInt timestamps as ISO strings

### 2. Store Tests
**Location**: `src/stores/__tests__/`

#### Workflow Store Tests (`workflowStore.test.ts`)
Tests the Zustand workflow state management:
- Loading workflows from ICP backend
- Creating new workflows with proper timestamp handling
- Updating existing workflows
- Deleting workflows
- Executing workflows with trigger data
- Loading workflow executions
- Error handling for all operations

#### UI Store Tests (`uiStore.test.ts`)
Tests the UI state management for toasts, modals, and loading states:
- Toast creation with auto-removal timers
- Manual toast removal
- Modal creation and management
- Loading state management
- Multiple toast/modal handling

### 3. Service Tests
**Location**: `src/services/__tests__/icpService.test.ts`

Integration tests for the ICP service layer:
- Service initialization and actor creation
- Workflow CRUD operations
- Execution management
- Node definition operations
- Authentication flow
- Error handling and network failures

### 4. Component Tests
**Location**: `src/components/__tests__/`

Tests individual React components:
- Button component variants and interactions
- Form validation components
- Layout components
- UI component accessibility

### 5. Application Integration Tests
**Location**: `src/__tests__/App.test.tsx`

Tests the main application integration:
- Router configuration
- Provider context setup
- Page navigation
- Error boundary handling

### 6. End-to-End Tests
**Location**: `e2e/app-functionality.spec.ts`

Comprehensive user workflow testing:

#### Core Application Functionality
- Application loading and initialization
- Navigation between pages
- Responsive design testing
- Loading and error state handling

#### Workflow Management
- Workflow list display
- Workflow creation and editing
- Form validation
- Workflow deletion with confirmation

#### Workflow Execution
- Manual workflow execution
- Execution history viewing
- Execution details and timeline
- Failed execution retry

#### Authentication
- Internet Identity integration
- Login/logout flow
- User session management
- Protected route access

#### UI Interactions
- Toast notification system
- Modal dialogs
- Form validation
- Drag and drop functionality
- Search and filtering

#### Performance and Accessibility
- Page load performance
- Accessibility compliance
- Keyboard navigation
- Screen reader compatibility

## Running Tests

### Unit and Integration Tests
```bash
# Run all unit/integration tests
npm run test

# Run tests with UI
npm run test:ui

# Run tests once (CI mode)
npm run test:run

# Run tests with coverage
npm run test:coverage
```

### End-to-End Tests
```bash
# Run all E2E tests
npm run test:e2e

# Run E2E tests with UI
npm run test:e2e:ui

# Run E2E tests with browser visible
npm run test:e2e:headed

# Install Playwright browsers (first time)
npx playwright install
```

### Run All Tests
```bash
# Run complete test suite
npm run test:all
```

## Test Data and Mocking

### ICP Service Mocking
Tests use comprehensive mocks for:
- `@dfinity/agent` - HTTP agent and actor creation
- `@dfinity/auth-client` - Authentication client
- `declarations/DeFlow_backend` - Generated canister interfaces

### Test Data
Mock objects include:
- Sample workflows with proper BigInt timestamps
- Workflow executions with various states
- Node definitions with schema validation
- User authentication states

## Test Environment Setup

### Vitest Configuration
- **Environment**: jsdom for DOM testing
- **Setup**: Automatic test setup with global mocks
- **Coverage**: V8 coverage provider
- **Globals**: Vitest globals enabled for convenient testing

### Playwright Configuration
- **Browsers**: Chromium, Firefox, WebKit
- **Mobile**: Pixel 5, iPhone 12 testing
- **Base URL**: ICP canister localhost URL
- **Retries**: 2 retries in CI environment
- **Screenshots**: On failure only
- **Tracing**: On first retry

## Continuous Integration

### GitHub Actions Integration
Tests are designed to run in CI environments with:
- Parallel test execution
- Artifact collection for failures
- Coverage reporting
- Cross-browser testing

### Test Reliability
- Deterministic test data
- Proper async/await handling
- Mock cleanup between tests
- Timeout configurations
- Retry strategies for flaky tests

## Coverage Goals

### Target Coverage
- **Unit Tests**: 90%+ code coverage
- **Integration Tests**: 80%+ feature coverage
- **E2E Tests**: 100% critical user journey coverage

### Coverage Areas
- All utility functions
- Store state management
- Component interactions
- ICP service integration
- Error handling paths
- Authentication flows

## Debugging Tests

### Local Debugging
```bash
# Debug specific test
npm run test -- --reporter=verbose utils/bigint

# Debug with browser DevTools (E2E)
npm run test:e2e:headed -- --debug

# View test UI
npm run test:ui
```

### CI Debugging
- Playwright trace files
- Screenshot artifacts
- Test logs and timing
- Coverage reports

## Best Practices

### Test Writing Guidelines
1. **Arrange, Act, Assert** pattern
2. **Descriptive test names** that explain the scenario
3. **Single responsibility** per test
4. **Proper mocking** to isolate units under test
5. **Cleanup** after each test
6. **Async/await** for all async operations

### Test Data Management
1. **Factory functions** for creating test data
2. **Minimal viable data** for each test
3. **Realistic edge cases** including BigInt boundaries
4. **Error scenarios** for comprehensive coverage

### Maintenance
1. **Update tests** with feature changes
2. **Refactor common patterns** into helpers
3. **Monitor test performance** and optimize slow tests
4. **Review test coverage** regularly

## Known Limitations

### Current Test Scope
- Internet Identity integration requires manual testing
- ICP canister deployment testing requires local replica
- Real blockchain interactions are mocked
- File upload/download functionality testing limited

### Future Improvements
- Visual regression testing
- Performance benchmarking
- Load testing for heavy workflows
- Real ICP testnet integration tests

## Troubleshooting

### Common Issues
1. **BigInt serialization**: Ensure proper BigInt handling in tests
2. **Async timing**: Use proper waiting strategies
3. **Mock conflicts**: Clear mocks between tests
4. **Browser differences**: Test across all target browsers

### Debug Commands
```bash
# Verbose test output
npm run test -- --reporter=verbose

# Run specific test file
npm run test -- src/utils/__tests__/bigint.test.ts

# Debug E2E with browser
npm run test:e2e -- --headed --debug
```

This comprehensive testing strategy ensures the DeFlow frontend is reliable, maintainable, and provides a high-quality user experience across all supported platforms and browsers.