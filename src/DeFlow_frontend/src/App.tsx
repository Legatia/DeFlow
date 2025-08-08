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
    // Initialize app without waiting for ICP services
    const initializeApp = async () => {
      try {
        console.log('Starting app initialization...')
        console.log('Current hostname:', window.location.hostname)
        
        // Skip ICP service initialization in development and local canister
        const hostname = window.location.hostname
        const isDevelopment = 
          hostname === 'localhost' || 
          hostname === '127.0.0.1' ||
          hostname.endsWith('.localhost')  // Catch canister URLs like u6s2n-gx777-77774-qaaba-cai.localhost
        
        if (isDevelopment) {
          console.log('Development/Local mode - skipping ICP initialization for hostname:', hostname)
          setIsLoading(false)
          return
        }
        
        // For production, add a timeout
        console.log('Production mode - setting timeout for initialization')
        const timer = setTimeout(() => {
          console.log('App initialization timeout reached, loading anyway')
          setIsLoading(false)
        }, 3000)
        
        // Clean up timer if component unmounts
        return () => {
          console.log('Cleaning up initialization timer')
          clearTimeout(timer)
        }
      } catch (error) {
        console.error('App initialization error:', error)
        setIsLoading(false)
      }
    }
    
    let cleanup: (() => void) | undefined
    
    initializeApp().then((result) => {
      cleanup = result
    })
    
    // Return cleanup function for effect
    return () => {
      if (cleanup) {
        cleanup()
      }
    }
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