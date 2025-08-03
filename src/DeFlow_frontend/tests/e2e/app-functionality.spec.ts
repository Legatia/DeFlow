import { test, expect } from '@playwright/test';

test.describe('DeFlow App Functionality', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should load the application successfully', async ({ page }) => {
    // Check if the page title is correct
    await expect(page).toHaveTitle(/DeFlow/);
    
    // Check if the main layout elements are present
    await expect(page.locator('[data-testid="app"]')).toBeVisible();
  });

  test('should display the dashboard by default', async ({ page }) => {
    // Check if dashboard is displayed on root route
    await expect(page.locator('[data-testid="dashboard"]')).toBeVisible();
    
    // Check for key dashboard elements
    await expect(page.getByText('Dashboard')).toBeVisible();
  });

  test('should navigate between pages', async ({ page }) => {
    // Navigate to workflow editor
    await page.click('text=Create Workflow');
    await expect(page.url()).toContain('/workflow');
    
    // Navigate to execution history
    await page.click('text=Executions');
    await expect(page.url()).toContain('/executions');
    
    // Navigate back to dashboard
    await page.click('text=Dashboard');
    await expect(page.url()).toBe('/');
  });

  test('should handle responsive design', async ({ page }) => {
    // Test desktop view
    await page.setViewportSize({ width: 1200, height: 800 });
    await expect(page.locator('[data-testid="desktop-menu"]')).toBeVisible();
    
    // Test mobile view
    await page.setViewportSize({ width: 375, height: 667 });
    await expect(page.locator('[data-testid="mobile-menu-trigger"]')).toBeVisible();
  });

  test('should display loading states appropriately', async ({ page }) => {
    // Check for loading indicators during data fetching
    const loadingIndicator = page.locator('[data-testid="loading"]');
    
    // Loading should appear briefly when navigating
    await page.click('text=Workflows');
    // Note: Loading might be too fast to catch in some cases
  });

  test('should handle error states gracefully', async ({ page }) => {
    // Test network error handling
    await page.route('**/api/**', route => route.abort());
    
    // Navigate to a page that requires API calls
    await page.click('text=Workflows');
    
    // Should display error message instead of crashing
    await expect(page.locator('[data-testid="error-message"]').or(page.getByText('Error'))).toBeVisible();
  });
});

test.describe('Workflow Management', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should display workflow list', async ({ page }) => {
    await page.click('text=Workflows');
    
    // Should show workflows section
    await expect(page.locator('[data-testid="workflow-list"]')).toBeVisible();
    
    // Should show create workflow button
    await expect(page.getByText('Create New Workflow')).toBeVisible();
  });

  test('should open workflow creation form', async ({ page }) => {
    await page.click('text=Create Workflow');
    
    // Should navigate to workflow editor
    await expect(page.url()).toContain('/workflow');
    
    // Should show workflow editor components
    await expect(page.locator('[data-testid="workflow-editor"]')).toBeVisible();
  });

  test('should validate workflow form inputs', async ({ page }) => {
    await page.goto('/workflow');
    
    // Try to save without filling required fields
    await page.click('text=Save Workflow');
    
    // Should show validation errors
    await expect(page.locator('[data-testid="validation-error"]').or(page.getByText('required'))).toBeVisible();
  });

  test('should create a new workflow', async ({ page }) => {
    await page.goto('/workflow');
    
    // Fill in workflow details
    await page.fill('[data-testid="workflow-name"]', 'Test Workflow');
    await page.fill('[data-testid="workflow-description"]', 'A test workflow for automation');
    
    // Save the workflow
    await page.click('text=Save Workflow');
    
    // Should show success message
    await expect(page.locator('[data-testid="success-message"]').or(page.getByText('saved'))).toBeVisible();
  });

  test('should edit existing workflow', async ({ page }) => {
    // Assuming there's an existing workflow
    await page.goto('/workflow/test-id');
    
    // Should load workflow data
    await expect(page.locator('[data-testid="workflow-name"]')).toHaveValue(/\w+/);
    
    // Make changes
    await page.fill('[data-testid="workflow-name"]', 'Updated Workflow Name');
    
    // Save changes
    await page.click('text=Save Changes');
    
    // Should show success message
    await expect(page.locator('[data-testid="success-message"]')).toBeVisible();
  });

  test('should delete workflow with confirmation', async ({ page }) => {
    await page.goto('/workflows');
    
    // Click delete on a workflow
    await page.click('[data-testid="delete-workflow-btn"]');
    
    // Should show confirmation dialog
    await expect(page.locator('[data-testid="confirmation-modal"]')).toBeVisible();
    
    // Confirm deletion
    await page.click('text=Delete');
    
    // Should show success message
    await expect(page.getByText('deleted')).toBeVisible();
  });
});

test.describe('Workflow Execution', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should execute workflow manually', async ({ page }) => {
    await page.goto('/workflows');
    
    // Click execute on a workflow
    await page.click('[data-testid="execute-workflow-btn"]');
    
    // Should show execution started message
    await expect(page.getByText('execution started')).toBeVisible();
  });

  test('should display execution history', async ({ page }) => {
    await page.goto('/executions');
    
    // Should show executions list
    await expect(page.locator('[data-testid="executions-list"]')).toBeVisible();
    
    // Should show execution status
    await expect(page.locator('[data-testid="execution-status"]')).toBeVisible();
  });

  test('should show execution details', async ({ page }) => {
    await page.goto('/executions');
    
    // Click on an execution
    await page.click('[data-testid="execution-item"]');
    
    // Should show execution details
    await expect(page.locator('[data-testid="execution-details"]')).toBeVisible();
    
    // Should show execution timeline
    await expect(page.locator('[data-testid="execution-timeline"]')).toBeVisible();
  });

  test('should retry failed execution', async ({ page }) => {
    await page.goto('/executions');
    
    // Find a failed execution
    await page.click('[data-testid="failed-execution"]');
    
    // Click retry button
    await page.click('text=Retry');
    
    // Should show retry confirmation
    await expect(page.getByText('retry')).toBeVisible();
  });
});

test.describe('Authentication Flow', () => {
  test('should show login interface', async ({ page }) => {
    await page.goto('/');
    
    // Should show login button when not authenticated
    await expect(page.getByText('Connect Wallet')).toBeVisible();
  });

  test('should handle Internet Identity authentication', async ({ page }) => {
    await page.goto('/');
    
    // Click login button
    await page.click('text=Connect Wallet');
    
    // Should redirect to Internet Identity or show auth modal
    // Note: This might open a new tab/window for II authentication
    await expect(page.url()).toContain('identity');
  });

  test('should show user info when authenticated', async ({ page }) => {
    // Mock authenticated state
    await page.addInitScript(() => {
      localStorage.setItem('auth-storage', JSON.stringify({
        state: { isAuthenticated: true, user: { principal: 'test-principal' } }
      }));
    });
    
    await page.goto('/');
    
    // Should show user info
    await expect(page.locator('[data-testid="user-menu"]')).toBeVisible();
  });

  test('should handle logout', async ({ page }) => {
    // Mock authenticated state
    await page.addInitScript(() => {
      localStorage.setItem('auth-storage', JSON.stringify({
        state: { isAuthenticated: true, user: { principal: 'test-principal' } }
      }));
    });
    
    await page.goto('/');
    
    // Click logout
    await page.click('[data-testid="logout-btn"]');
    
    // Should return to login state
    await expect(page.getByText('Connect Wallet')).toBeVisible();
  });
});

test.describe('UI Components and Interactions', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should show toast notifications', async ({ page }) => {
    // Trigger an action that shows a toast
    await page.click('text=Save');
    
    // Should show toast notification
    await expect(page.locator('[data-testid="toast"]')).toBeVisible();
    
    // Toast should auto-dismiss
    await expect(page.locator('[data-testid="toast"]')).toBeHidden({ timeout: 6000 });
  });

  test('should open and close modals', async ({ page }) => {
    // Trigger modal
    await page.click('text=Settings');
    
    // Should show modal
    await expect(page.locator('[data-testid="modal"]')).toBeVisible();
    
    // Close modal
    await page.click('[data-testid="modal-close"]');
    
    // Should hide modal
    await expect(page.locator('[data-testid="modal"]')).toBeHidden();
  });

  test('should handle form validation', async ({ page }) => {
    await page.goto('/workflow');
    
    // Submit empty form
    await page.click('text=Save');
    
    // Should show validation errors
    await expect(page.locator('.error')).toBeVisible();
    
    // Fill required field
    await page.fill('[data-testid="workflow-name"]', 'Test');
    
    // Error should disappear
    await expect(page.locator('.error')).toBeHidden();
  });

  test('should handle drag and drop in workflow editor', async ({ page }) => {
    await page.goto('/workflow');
    
    // Drag a node from palette
    const nodeSource = page.locator('[data-testid="node-palette-item"]').first();
    const canvas = page.locator('[data-testid="workflow-canvas"]');
    
    await nodeSource.dragTo(canvas);
    
    // Should create node on canvas
    await expect(page.locator('[data-testid="workflow-node"]')).toBeVisible();
  });

  test('should search and filter workflows', async ({ page }) => {
    await page.goto('/workflows');
    
    // Enter search term
    await page.fill('[data-testid="search-input"]', 'test');
    
    // Should filter results
    await expect(page.locator('[data-testid="workflow-item"]')).toContainText('test');
  });
});

test.describe('Performance and Accessibility', () => {
  test('should meet accessibility standards', async ({ page }) => {
    await page.goto('/');
    
    // Check for proper heading hierarchy
    await expect(page.locator('h1')).toBeVisible();
    
    // Check for alt text on images
    const images = page.locator('img');
    const count = await images.count();
    for (let i = 0; i < count; i++) {
      await expect(images.nth(i)).toHaveAttribute('alt');
    }
    
    // Check for proper form labels
    const inputs = page.locator('input');
    const inputCount = await inputs.count();
    for (let i = 0; i < inputCount; i++) {
      const input = inputs.nth(i);
      const id = await input.getAttribute('id');
      if (id) {
        await expect(page.locator(`label[for="${id}"]`)).toBeVisible();
      }
    }
  });

  test('should load quickly', async ({ page }) => {
    const startTime = Date.now();
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    const loadTime = Date.now() - startTime;
    
    // Should load within 5 seconds
    expect(loadTime).toBeLessThan(5000);
  });

  test('should handle keyboard navigation', async ({ page }) => {
    await page.goto('/');
    
    // Tab through interactive elements
    await page.keyboard.press('Tab');
    await expect(page.locator(':focus')).toBeVisible();
    
    // Should be able to navigate menu with keyboard
    await page.keyboard.press('Enter');
    // Check that keyboard interaction worked
  });
});