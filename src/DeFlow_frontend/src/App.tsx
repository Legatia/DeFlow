import { useState, useEffect } from 'react'
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom'
import Layout from './components/Layout'
import LoadingScreen from './components/LoadingScreen'
import Dashboard from './pages/Dashboard'
import WorkflowList from './pages/WorkflowList'
import WorkflowEditor from './pages/WorkflowEditor'
import ExecutionHistory from './pages/ExecutionHistory'
import Settings from './pages/Settings'
import DeFiDashboard from './pages/DeFiDashboard'

function App() {
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    // Small delay to ensure polyfills are loaded and UI can render
    const timer = setTimeout(() => {
      setIsLoading(false)
    }, 100)

    return () => clearTimeout(timer)
  }, [])

  if (isLoading) {
    return <LoadingScreen />
  }

  return (
    <Router>
      <Layout>
        <Routes>
          <Route path="/" element={<DeFiDashboard />} />
          <Route path="/dashboard" element={<Dashboard />} />
          <Route path="/defi" element={<DeFiDashboard />} />
          <Route path="/workflows" element={<WorkflowList />} />
          <Route path="/workflows/new" element={<WorkflowEditor />} />
          <Route path="/workflows/:id" element={<WorkflowEditor />} />
          <Route path="/executions" element={<ExecutionHistory />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </Layout>
    </Router>
  )
}

export default App