// Comprehensive functionality test for DeFlow services
import { executionEngine } from './services/executionEngine'
import { authService } from './services/authService'
import { monitoringService } from './services/monitoringService'
import { webhookService } from './services/webhookService'
import { collaborationService } from './services/collaborationService'
import { realTimeService } from './services/realTimeService'
import { TimestampUtils } from './utils/timestamp-utils'

// Test workflow for functionality testing
const testWorkflow = {
  id: 'test_workflow_001',
  name: 'Functionality Test Workflow',
  description: 'Testing all core functionality',
  nodes: [
    {
      id: 'trigger-1',
      node_type: 'manual-trigger',
      position: { x: 100, y: 100 },
      configuration: { parameters: { name: 'Test Start' } },
      metadata: {
        label: 'Start Test',
        description: 'Manual trigger for testing',
        tags: ['trigger'],
        icon: '▶️',
        color: '#3b82f6'
      }
    },
    {
      id: 'email-1',
      node_type: 'send-email',
      position: { x: 400, y: 100 },
      configuration: {
        parameters: {
          to: 'test@deflow.com',
          subject: 'Test Email',
          body: 'This is a test email from DeFlow functionality test',
          useTemplate: false
        }
      },
      metadata: {
        label: 'Send Test Email',
        description: 'Test email sending',
        tags: ['email'],
        icon: '📧',
        color: '#ef4444'
      }
    }
  ],
  connections: [
    {
      id: 'conn-1',
      source_node_id: 'trigger-1',
      target_node_id: 'email-1',
      source_output: 'trigger',
      target_input: 'data'
    }
  ],
  triggers: [{ type: 'manual' as const }],
  created_at: TimestampUtils.dateToICPTimestamp(new Date()),
  updated_at: TimestampUtils.dateToICPTimestamp(new Date()),
  active: true,
  owner: 'test_user',
  tags: ['test'],
  version: '1.0.0'
}

async function runComprehensiveTest() {
  console.log('🧪 Starting DeFlow Comprehensive Functionality Test...\n')
  
  const results = {
    authService: false,
    executionEngine: false,
    monitoringService: false,
    webhookService: false,
    collaborationService: false,
    realTimeService: false,
    overallHealth: false
  }

  try {
    // Test 1: Authentication Service
    console.log('1️⃣ Testing Authentication Service...')
    try {
      const loginResult = await authService.login({
        email: 'admin@deflow.com',
        password: 'password123'
      })
      
      if (loginResult.user && loginResult.session && authService.isAuthenticated()) {
        console.log('   ✅ Authentication: Login successful')
        console.log(`   ✅ User: ${loginResult.user.displayName} (${loginResult.user.role})`)
        
        // Test permissions
        const hasWorkflowPerm = authService.canAccessWorkflow('test', 'read')
        console.log(`   ✅ Permissions: Can access workflows = ${hasWorkflowPerm}`)
        
        results.authService = true
      } else {
        console.log('   ❌ Authentication: Login failed')
      }
    } catch (error) {
      console.log(`   ❌ Authentication error: ${error}`)
    }

    // Test 2: Execution Engine
    console.log('\n2️⃣ Testing Workflow Execution Engine...')
    try {
      const execution = await executionEngine.executeWorkflow(
        testWorkflow,
        { test: 'functionality_test' },
        'test_user'
      )

      if (execution && execution.status === 'completed') {
        console.log('   ✅ Execution: Workflow executed successfully')
        console.log(`   ✅ Status: ${execution.status}`)
        console.log(`   ✅ Node executions: ${execution.node_executions.length}`)
        
        // Check if all nodes completed
        const allCompleted = execution.node_executions.every(ne => ne.status === 'completed')
        console.log(`   ✅ All nodes completed: ${allCompleted}`)
        
        results.executionEngine = true
      } else {
        console.log(`   ❌ Execution failed: ${execution?.error_message || 'Unknown error'}`)
      }
    } catch (error) {
      console.log(`   ❌ Execution engine error: ${error}`)
    }

    // Test 3: Monitoring Service
    console.log('\n3️⃣ Testing Monitoring Service...')
    try {
      const metrics = monitoringService.getMetrics()
      const health = monitoringService.getSystemHealth()
      
      console.log(`   ✅ Metrics: ${metrics.totalExecutions} total executions`)
      console.log(`   ✅ Success rate: ${metrics.successRate.toFixed(1)}%`)
      console.log(`   ✅ System health: ${health.status}`)
      console.log(`   ✅ Average execution time: ${metrics.averageExecutionTime.toFixed(2)}ms`)
      
      results.monitoringService = true
    } catch (error) {
      console.log(`   ❌ Monitoring service error: ${error}`)
    }

    // Test 4: Webhook Service
    console.log('\n4️⃣ Testing Webhook Service...')
    try {
      // Create webhook endpoint
      const endpoint = webhookService.createEndpoint(testWorkflow.id, {
        path: '/test/webhook',
        method: 'POST',
        isActive: true,
        headers: {},
        validation: { enabled: false },
        rateLimiting: { enabled: false, maxRequests: 100, timeWindow: 60, strategy: 'fixed_window' },
        metadata: {
          name: 'Test Webhook',
          description: 'Test webhook endpoint',
          tags: ['test']
        }
      })

      // Process test webhook request
      const response = await webhookService.processWebhookRequest(
        'POST',
        '/test/webhook',
        { 'content-type': 'application/json' },
        { test: 'webhook_data' },
        {},
        '127.0.0.1'
      )

      if (response.status === 200) {
        console.log('   ✅ Webhook: Endpoint created and processed successfully')
        console.log(`   ✅ Response: ${response.status} - ${response.body.message}`)
        
        const analytics = webhookService.getEndpointAnalytics(endpoint.id)
        console.log(`   ✅ Analytics: ${analytics.totalRequests} requests processed`)
        
        results.webhookService = true
      } else {
        console.log(`   ❌ Webhook failed: ${response.status} - ${response.body.error}`)
      }
    } catch (error) {
      console.log(`   ❌ Webhook service error: ${error}`)
    }

    // Test 5: Collaboration Service
    console.log('\n5️⃣ Testing Collaboration Service...')
    try {
      // Share workflow
      const shared = collaborationService.shareWorkflow(
        testWorkflow.id,
        'test_user',
        {
          visibility: 'team',
          allowComments: true,
          allowFork: true,
          allowExport: false
        },
        {
          view: true,
          edit: true,
          execute: true,
          share: true,
          delete: false,
          comment: true,
          analytics: true
        }
      )

      // Add comment
      const comment = collaborationService.addComment(
        testWorkflow.id,
        'test_user',
        'This is a test comment for functionality testing',
        'general'
      )

      console.log('   ✅ Collaboration: Workflow shared successfully')
      console.log(`   ✅ Shared workflow ID: ${shared.id}`)
      console.log(`   ✅ Comment added: ${comment.id}`)
      console.log(`   ✅ Collaborators: ${shared.collaborators.length}`)
      
      results.collaborationService = true
    } catch (error) {
      console.log(`   ❌ Collaboration service error: ${error}`)
    }

    // Test 6: Real-time Service
    console.log('\n6️⃣ Testing Real-time Service...')
    try {
      const connectionId = 'test_connection'
      
      // Add connection
      realTimeService.addConnection(connectionId, 'test_user', [testWorkflow.id])
      
      // Broadcast test message
      realTimeService.broadcastToWorkflow(testWorkflow.id, {
        type: 'test_message',
        message: 'Testing real-time functionality'
      })

      // Get messages
      const messages = realTimeService.getMessages(connectionId)
      const stats = realTimeService.getConnectionStats()

      console.log('   ✅ Real-time: Connection established')
      console.log(`   ✅ Messages: ${messages.length} messages received`)
      console.log(`   ✅ Active connections: ${stats.activeConnections}`)
      
      // Cleanup
      realTimeService.removeConnection(connectionId)
      
      results.realTimeService = true
    } catch (error) {
      console.log(`   ❌ Real-time service error: ${error}`)
    }

    // Overall Health Check
    console.log('\n🔍 Overall System Health Check...')
    const totalTests = Object.keys(results).length - 1 // Exclude overallHealth
    const passedTests = Object.values(results).filter(r => r === true).length
    const healthPercentage = (passedTests / totalTests) * 100

    results.overallHealth = healthPercentage >= 80 // 80% pass rate required

    console.log(`\n📊 Test Results Summary:`)
    console.log(`   Total Tests: ${totalTests}`)
    console.log(`   Passed: ${passedTests}`)
    console.log(`   Failed: ${totalTests - passedTests}`)
    console.log(`   Success Rate: ${healthPercentage.toFixed(1)}%`)
    
    if (results.overallHealth) {
      console.log(`\n🎉 Overall Status: HEALTHY ✅`)
      console.log(`   DeFlow platform is functioning correctly!`)
    } else {
      console.log(`\n⚠️  Overall Status: NEEDS ATTENTION ❌`)
      console.log(`   Some services require investigation.`)
    }

    // Detailed breakdown
    console.log(`\n📋 Detailed Service Status:`)
    Object.entries(results).forEach(([service, status]) => {
      if (service !== 'overallHealth') {
        const icon = status ? '✅' : '❌'
        console.log(`   ${icon} ${service}: ${status ? 'PASS' : 'FAIL'}`)
      }
    })

    return results

  } catch (error) {
    console.error(`\n💥 Critical error during testing: ${error}`)
    return results
  }
}

// Export for use in testing
export { runComprehensiveTest, testWorkflow }

// Run if executed directly
if (typeof window === 'undefined') {
  runComprehensiveTest().then(results => {
    const exitCode = results.overallHealth ? 0 : 1
    process.exit(exitCode)
  }).catch(error => {
    console.error('Test execution failed:', error)
    process.exit(1)
  })
}