// Import BigInt polyfill before any other imports to prevent conversion errors
import './utils/bigint-polyfill'

import { useState, useEffect } from 'react'
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom'
import Layout from './components/Layout'
import LoadingScreen from './components/LoadingScreen'
import Dashboard from './pages/Dashboard'
import WorkflowList from './pages/WorkflowList'
import WorkflowEditor from './pages/WorkflowEditor'
import ExecutionHistory from './pages/ExecutionHistory'
import Settings from './pages/Settings'
import DeFiDashboard from './pages/DeFiDashboard'
import PaymentPage from './pages/PaymentPage'

function App() {
  const [isInitialLoading, setIsInitialLoading] = useState(true)

  useEffect(() => {
    // Small delay to ensure polyfills are loaded
    const timer = setTimeout(() => {
      setIsInitialLoading(false)
    }, 500)

    return () => clearTimeout(timer)
  }, [])

  // Show loading screen during initial app load
  if (isInitialLoading) {
    return <LoadingScreen />
  }

  return (
    <Router>
        <Layout>
          <Routes>
            <Route path="/" element={<Navigate to="/dashboard" replace />} />
            <Route path="/dashboard" element={<Dashboard />} />
            <Route path="/workflows" element={<WorkflowList />} />
            <Route path="/workflows/new" element={<WorkflowEditor />} />
            <Route path="/workflows/:id" element={<WorkflowEditor />} />
            <Route path="/executions" element={<ExecutionHistory />} />
            <Route path="/settings" element={<Settings />} />
            <Route path="/payment" element={<PaymentPage />} />
            <Route path="/premium" element={<PaymentPage />} />
          </Routes>
        </Layout>
    </Router>
  )
}

export default App