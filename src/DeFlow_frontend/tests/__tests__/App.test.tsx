import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import App from '../App';

// Mock the ICP service
vi.mock('../services/icpService', () => ({
  icpService: {
    initialize: vi.fn().mockResolvedValue(undefined),
    listWorkflows: vi.fn().mockResolvedValue([]),
    listNodeTypes: vi.fn().mockResolvedValue(['http_request', 'data_transform']),
    getNodeDefinition: vi.fn().mockResolvedValue({
      node_type: 'http_request',
      name: 'HTTP Request',
      description: 'Makes HTTP requests',
      category: 'network',
      input_schema: {},
      output_schema: {},
      config_schema: {},
    }),
  }
}));

// Mock authentication context
vi.mock('../contexts/AuthContext', () => ({
  AuthProvider: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="auth-provider">{children}</div>
  ),
  useAuth: () => ({
    isAuthenticated: false,
    user: null,
    login: vi.fn(),
    logout: vi.fn(),
    isLoading: false,
  })
}));

// Mock workflow context
vi.mock('../contexts/WorkflowContext', () => ({
  WorkflowProvider: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="workflow-provider">{children}</div>
  ),
  useWorkflow: () => ({
    workflows: [],
    currentWorkflow: null,
    isLoading: false,
    error: null,
    loadWorkflows: vi.fn(),
    createWorkflow: vi.fn(),
    updateWorkflow: vi.fn(),
    deleteWorkflow: vi.fn(),
  })
}));

// Mock components to avoid complex rendering
vi.mock('../components/layout/Layout', () => ({
  default: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="layout">
      <header data-testid="header">DeFlow Header</header>
      <main data-testid="main">{children}</main>
    </div>
  )
}));

vi.mock('../pages/Dashboard', () => ({
  default: () => <div data-testid="dashboard">Dashboard Page</div>
}));

vi.mock('../pages/WorkflowEditor', () => ({
  default: () => <div data-testid="workflow-editor">Workflow Editor Page</div>
}));

vi.mock('../pages/ExecutionHistory', () => ({
  default: () => <div data-testid="execution-history">Execution History Page</div>
}));

describe('App Integration Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should render the app with providers', () => {
    render(
      <MemoryRouter initialEntries={['/']}>
        <App />
      </MemoryRouter>
    );

    expect(screen.getByTestId('auth-provider')).toBeInTheDocument();
    expect(screen.getByTestId('workflow-provider')).toBeInTheDocument();
    expect(screen.getByTestId('layout')).toBeInTheDocument();
  });

  it('should render dashboard on root route', () => {
    render(
      <MemoryRouter initialEntries={['/']}>
        <App />
      </MemoryRouter>
    );

    expect(screen.getByTestId('dashboard')).toBeInTheDocument();
  });

  it('should render workflow editor on /workflow route', () => {
    render(
      <MemoryRouter initialEntries={['/workflow']}>
        <App />
      </MemoryRouter>
    );

    expect(screen.getByTestId('workflow-editor')).toBeInTheDocument();
  });

  it('should render workflow editor with ID on /workflow/:id route', () => {
    render(
      <MemoryRouter initialEntries={['/workflow/123']}>
        <App />
      </MemoryRouter>
    );

    expect(screen.getByTestId('workflow-editor')).toBeInTheDocument();
  });

  it('should render execution history on /executions route', () => {
    render(
      <MemoryRouter initialEntries={['/executions']}>
        <App />
      </MemoryRouter>
    );

    expect(screen.getByTestId('execution-history')).toBeInTheDocument();
  });

  it('should render layout with header and main content', () => {
    render(
      <MemoryRouter initialEntries={['/']}>
        <App />
      </MemoryRouter>
    );

    expect(screen.getByTestId('header')).toBeInTheDocument();
    expect(screen.getByTestId('main')).toBeInTheDocument();
    expect(screen.getByText('DeFlow Header')).toBeInTheDocument();
  });

  it('should handle navigation between routes', async () => {
    const { rerender } = render(
      <MemoryRouter initialEntries={['/']}>
        <App />
      </MemoryRouter>
    );

    expect(screen.getByTestId('dashboard')).toBeInTheDocument();

    rerender(
      <MemoryRouter initialEntries={['/workflow']}>
        <App />
      </MemoryRouter>
    );

    expect(screen.getByTestId('workflow-editor')).toBeInTheDocument();
    expect(screen.queryByTestId('dashboard')).not.toBeInTheDocument();
  });

  it('should maintain provider context across navigation', () => {
    render(
      <MemoryRouter initialEntries={['/workflow']}>
        <App />
      </MemoryRouter>
    );

    // Providers should always be present
    expect(screen.getByTestId('auth-provider')).toBeInTheDocument();
    expect(screen.getByTestId('workflow-provider')).toBeInTheDocument();
  });

  it('should handle invalid routes gracefully', () => {
    render(
      <MemoryRouter initialEntries={['/invalid-route']}>
        <App />
      </MemoryRouter>
    );

    // Should still render layout
    expect(screen.getByTestId('layout')).toBeInTheDocument();
    
    // But no specific page content
    expect(screen.queryByTestId('dashboard')).not.toBeInTheDocument();
    expect(screen.queryByTestId('workflow-editor')).not.toBeInTheDocument();
    expect(screen.queryByTestId('execution-history')).not.toBeInTheDocument();
  });
});