// Load BigInt polyfill FIRST - replaces BigInt with BigNumber.js completely
import './utils/bigint-polyfill'
// Load timestamp utilities (BigNumber.js based, no BigInt)
import './utils/timestamp-utils'

import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import App from './App'
import ErrorBoundary from './components/ErrorBoundary'
import { EnhancedAuthProvider } from './contexts/EnhancedAuthContext'
import './index.css'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ErrorBoundary>
      <EnhancedAuthProvider>
        <App />
      </EnhancedAuthProvider>
    </ErrorBoundary>
  </StrictMode>
)