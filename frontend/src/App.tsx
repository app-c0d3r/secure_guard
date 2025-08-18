import { Routes, Route } from 'react-router-dom'
import { Toaster } from 'react-hot-toast'
import Layout from '@/components/Layout'
import Dashboard from '@/pages/Dashboard'
import Agents from '@/pages/Agents'
import Security from '@/pages/Security'
import Users from '@/pages/Users'
import Subscriptions from '@/pages/Subscriptions'
import Settings from '@/pages/Settings'
import AssetManagement from '@/pages/AssetManagement'
import Login from '@/pages/Login'
import PasswordRecovery from '@/components/Security/PasswordRecovery'
import { useAuthStore } from '@/stores/authStore'
import { useSecurityMonitoring } from '@/hooks/useSecurityMonitoring'
import { ThemeProvider } from '@/contexts/ThemeContext'

function App() {
  const { isAuthenticated } = useAuthStore()
  
  // Initialize security monitoring for authenticated users
  useSecurityMonitoring({
    rapidClicks: 25,
    rapidNavigation: 15,
    suspiciousKeystrokes: 60,
    devToolsDetection: true,
    consoleInteraction: true,
    networkMonitoring: true
  })

  // Check if this is a password reset route
  const urlParams = new URLSearchParams(window.location.search)
  const resetToken = urlParams.get('reset')
  
  if (resetToken) {
    return (
      <ThemeProvider>
        <PasswordRecovery token={resetToken} />
      </ThemeProvider>
    )
  }

  if (!isAuthenticated) {
    return (
      <ThemeProvider>
        <Login />
      </ThemeProvider>
    )
  }

  return (
    <ThemeProvider>
      <Layout>
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/dashboard" element={<Dashboard />} />
          <Route path="/agents" element={<Agents />} />
          <Route path="/security" element={<Security />} />
          <Route path="/users" element={<Users />} />
          <Route path="/subscriptions" element={<Subscriptions />} />
          <Route path="/assets" element={<AssetManagement />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </Layout>
      <Toaster 
        position="top-right"
        toastOptions={{
          duration: 4000,
          className: 'dark:bg-gray-800 dark:text-white',
          style: {
            background: 'var(--toast-bg, #fff)',
            color: 'var(--toast-color, #1e293b)',
            boxShadow: '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)',
          },
        }}
      />
    </ThemeProvider>
  )
}

export default App