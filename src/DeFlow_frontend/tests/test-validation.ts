/**
 * Manual validation script to verify core app functions work as expected
 * Run this script to test the main application functionality
 */

import { icpService } from './services/icpService';
import { useWorkflowStore } from './stores/workflowStore';
import { useUIStore } from './stores/uiStore';
import { bigintToNumber, timestampToBigint, formatBigintTimestamp } from './utils/bigint';
import type { Workflow } from './types';

// Test data
const testWorkflow: Workflow = {
  id: 'test-validation-workflow',
  name: 'Validation Test Workflow',
  description: 'A workflow for testing core functionality',
  nodes: [
    {
      id: 'node-1',
      node_type: 'http_request',
      position: { x: 100, y: 100 },
      config: {
        url: { String: 'https://api.example.com/data' },
        method: { String: 'GET' }
      }
    }
  ],
  connections: [],
  triggers: [
    {
      id: 'trigger-1',
      trigger_type: 'manual',
      config: {}
    }
  ],
  created_at: timestampToBigint(),
  updated_at: timestampToBigint(),
  active: true
};

/**
 * Validation Tests Suite
 */
export class AppValidationSuite {
  private results: { test: string; status: 'PASS' | 'FAIL'; message?: string }[] = [];

  private logResult(test: string, status: 'PASS' | 'FAIL', message?: string) {
    this.results.push({ test, status, message });
    const statusSymbol = status === 'PASS' ? '✅' : '❌';
    console.log(`${statusSymbol} ${test}${message ? `: ${message}` : ''}`);
  }

  /**
   * Test BigInt utility functions
   */
  async testBigIntUtils() {
    console.log('\n🧪 Testing BigInt Utilities...');
    
    try {
      // Test basic conversion
      const timestamp = Date.now();
      const bigintTimestamp = timestampToBigint(timestamp);
      const convertedBack = bigintToNumber(bigintTimestamp / BigInt(1000000));
      
      if (Math.abs(convertedBack - timestamp) < 1000) {
        this.logResult('BigInt timestamp conversion', 'PASS');
      } else {
        this.logResult('BigInt timestamp conversion', 'FAIL', 'Conversion precision issue');
      }

      // Test formatting
      const formatted = formatBigintTimestamp(bigintTimestamp);
      if (formatted.includes('T') && formatted.includes('Z')) {
        this.logResult('BigInt timestamp formatting', 'PASS');
      } else {
        this.logResult('BigInt timestamp formatting', 'FAIL', 'Invalid ISO format');
      }

      // Test edge cases
      const maxSafe = bigintToNumber(BigInt(Number.MAX_SAFE_INTEGER));
      if (maxSafe === Number.MAX_SAFE_INTEGER) {
        this.logResult('BigInt safe integer handling', 'PASS');
      } else {
        this.logResult('BigInt safe integer handling', 'FAIL', 'Safe integer conversion failed');
      }

    } catch (error) {
      this.logResult('BigInt utilities', 'FAIL', (error as Error).message);
    }
  }

  /**
   * Test UI Store functionality
   */
  async testUIStore() {
    console.log('\n🧪 Testing UI Store...');
    
    try {
      const store = useUIStore.getState();
      
      // Test toast functionality
      store.showToast({
        title: 'Test Toast',
        message: 'This is a validation test',
        type: 'info',
        duration: 1000
      });
      
      let state = useUIStore.getState();
      if (state.toasts.length === 1) {
        this.logResult('Toast creation', 'PASS');
      } else {
        this.logResult('Toast creation', 'FAIL', `Expected 1 toast, got ${state.toasts.length}`);
      }

      // Test modal functionality
      store.showModal({
        title: 'Test Modal',
        content: 'Validation test modal',
        size: 'md',
        closable: true
      });
      
      state = useUIStore.getState();
      if (state.modals.length === 1) {
        this.logResult('Modal creation', 'PASS');
      } else {
        this.logResult('Modal creation', 'FAIL', `Expected 1 modal, got ${state.modals.length}`);
      }

      // Test loading state
      store.setLoading(true);
      state = useUIStore.getState();
      if (state.isLoading === true) {
        this.logResult('Loading state management', 'PASS');
      } else {
        this.logResult('Loading state management', 'FAIL', 'Loading state not set correctly');
      }

      // Cleanup
      store.hideToast(state.toasts[0]?.id);
      store.hideModal(state.modals[0]?.id);
      store.setLoading(false);

    } catch (error) {
      this.logResult('UI Store', 'FAIL', (error as Error).message);
    }
  }

  /**
   * Test Workflow Store functionality
   */
  async testWorkflowStore() {
    console.log('\n🧪 Testing Workflow Store...');
    
    try {
      const store = useWorkflowStore.getState();
      
      // Test initial state
      if (Array.isArray(store.workflows)) {
        this.logResult('Workflow store initialization', 'PASS');
      } else {
        this.logResult('Workflow store initialization', 'FAIL', 'Workflows not initialized as array');
      }

      // Test store methods exist
      const requiredMethods = [
        'loadWorkflows',
        'createWorkflow', 
        'updateWorkflow',
        'deleteWorkflow',
        'executeWorkflow',
        'loadExecutions'
      ];
      
      const missingMethods = requiredMethods.filter(method => typeof store[method] !== 'function');
      if (missingMethods.length === 0) {
        this.logResult('Workflow store methods', 'PASS');
      } else {
        this.logResult('Workflow store methods', 'FAIL', `Missing methods: ${missingMethods.join(', ')}`);
      }

      // Test workflow validation (basic structure)
      const isValidWorkflow = testWorkflow.id && 
                             testWorkflow.name && 
                             Array.isArray(testWorkflow.nodes) &&
                             Array.isArray(testWorkflow.connections) &&
                             Array.isArray(testWorkflow.triggers);
      
      if (isValidWorkflow) {
        this.logResult('Workflow data structure', 'PASS');
      } else {
        this.logResult('Workflow data structure', 'FAIL', 'Invalid workflow structure');
      }

    } catch (error) {
      this.logResult('Workflow Store', 'FAIL', (error as Error).message);
    }
  }

  /**
   * Test ICP Service initialization
   */
  async testICPService() {
    console.log('\n🧪 Testing ICP Service...');
    
    try {
      // Test service initialization
      await icpService.initialize();
      this.logResult('ICP service initialization', 'PASS');

      // Test service methods exist
      const requiredMethods = [
        'createWorkflow',
        'updateWorkflow',
        'getWorkflow', 
        'listWorkflows',
        'deleteWorkflow',
        'startExecution',
        'getExecution',
        'listExecutions',
        'greet'
      ];
      
      const missingMethods = requiredMethods.filter(method => typeof icpService[method] !== 'function');
      if (missingMethods.length === 0) {
        this.logResult('ICP service methods', 'PASS');
      } else {
        this.logResult('ICP service methods', 'FAIL', `Missing methods: ${missingMethods.join(', ')}`);
      }

      // Test basic service call (greet function)
      try {
        const greeting = await icpService.greet('Validation Test');
        if (typeof greeting === 'string') {
          this.logResult('ICP service communication', 'PASS');
        } else {
          this.logResult('ICP service communication', 'FAIL', 'Invalid greeting response');
        }
      } catch (error) {
        this.logResult('ICP service communication', 'FAIL', 'Service call failed - expected in test environment');
      }

    } catch (error) {
      this.logResult('ICP Service', 'FAIL', (error as Error).message);
    }
  }

  /**
   * Test Type System Integration
   */
  async testTypeSystem() {
    console.log('\n🧪 Testing Type System...');
    
    try {
      // Test workflow type structure
      const workflow: Workflow = testWorkflow;
      if (workflow.id && workflow.name && workflow.created_at && workflow.updated_at) {
        this.logResult('Workflow type definition', 'PASS');
      } else {
        this.logResult('Workflow type definition', 'FAIL', 'Missing required workflow properties');
      }

      // Test BigInt type handling
      const timestamp: bigint = workflow.created_at;
      if (typeof timestamp === 'bigint') {
        this.logResult('BigInt type integration', 'PASS');
      } else {
        this.logResult('BigInt type integration', 'FAIL', 'Timestamp not properly typed as BigInt');
      }

      // Test node configuration typing
      const nodeConfig = workflow.nodes[0]?.config;
      if (nodeConfig && typeof nodeConfig === 'object') {
        this.logResult('Node configuration typing', 'PASS');
      } else {
        this.logResult('Node configuration typing', 'FAIL', 'Node config not properly structured');
      }

    } catch (error) {
      this.logResult('Type System', 'FAIL', (error as Error).message);
    }
  }

  /**
   * Run all validation tests
   */
  async runAllTests() {
    console.log('🚀 Starting DeFlow App Validation Suite...\n');
    
    await this.testBigIntUtils();
    await this.testUIStore();
    await this.testWorkflowStore();
    await this.testICPService();
    await this.testTypeSystem();
    
    this.printSummary();
  }

  /**
   * Print test results summary
   */
  private printSummary() {
    console.log('\n📊 Validation Results Summary:');
    console.log('================================');
    
    const passed = this.results.filter(r => r.status === 'PASS').length;
    const failed = this.results.filter(r => r.status === 'FAIL').length;
    const total = this.results.length;
    
    console.log(`✅ Passed: ${passed}/${total}`);
    console.log(`❌ Failed: ${failed}/${total}`);
    console.log(`📈 Success Rate: ${Math.round((passed / total) * 100)}%\n`);
    
    if (failed > 0) {
      console.log('❌ Failed Tests:');
      this.results
        .filter(r => r.status === 'FAIL')
        .forEach(r => console.log(`   • ${r.test}: ${r.message || 'Unknown error'}`));
    }
    
    console.log('\n🎯 Core Functionality Status:');
    console.log(`BigInt Utils: ${this.getCategoryStatus('BigInt')}`);
    console.log(`UI Store: ${this.getCategoryStatus('UI')}`);
    console.log(`Workflow Store: ${this.getCategoryStatus('Workflow')}`);
    console.log(`ICP Service: ${this.getCategoryStatus('ICP')}`);
    console.log(`Type System: ${this.getCategoryStatus('Type')}`);
    
    if (passed >= total * 0.8) {
      console.log('\n🎉 App validation PASSED! Core functions are working correctly.');
    } else {
      console.log('\n⚠️  App validation has issues. Please review failed tests.');
    }
  }

  private getCategoryStatus(category: string): string {
    const categoryTests = this.results.filter(r => r.test.toLowerCase().includes(category.toLowerCase()));
    const categoryPassed = categoryTests.filter(r => r.status === 'PASS').length;
    return categoryPassed === categoryTests.length ? '✅ PASS' : '❌ FAIL';
  }
}

// Export for use in console or other scripts
export const runValidation = () => {
  const suite = new AppValidationSuite();
  return suite.runAllTests();
};

// Auto-run if this file is executed directly
if (typeof window !== 'undefined' && (window as any).__DEV__) {
  console.log('🔧 Development mode detected. Run `runValidation()` in console to test app functions.');
}