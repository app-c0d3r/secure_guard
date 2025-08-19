import { Routes, Route } from 'react-router-dom'
import { Toaster } from 'react-hot-toast'
import { useState, useEffect } from 'react'
import Layout from '@/components/Layout'
import ProtectedRoute from '@/components/ProtectedRoute'
import Dashboard from '@/pages/Dashboard'
import Agents from '@/pages/Agents'
import Security from '@/pages/Security'
import Users from '@/pages/Users'
import Subscriptions from '@/pages/Subscriptions'
import Settings from '@/pages/Settings'
import AssetManagement from '@/pages/AssetManagement'
import Login from '@/pages/Login'
import PasswordRecovery from '@/components/Security/PasswordRecovery'
import PasswordChangeModal from '@/components/Security/PasswordChangeModal'
import { useAuthStore } from '@/stores/authStore'
import { useSecurityMonitoring } from '@/hooks/useSecurityMonitoring'
import { ThemeProvider } from '@/contexts/ThemeContext'

function App() {
  const { isAuthenticated, token } = useAuthStore()
  const [mustChangePassword, setMustChangePassword] = useState(false)
  const [isCheckingAuth, setIsCheckingAuth] = useState(true)
  
  // Initialize security monitoring for authenticated users
  useSecurityMonitoring({
    rapidClicks: 25,
    rapidNavigation: 15,
    suspiciousKeystrokes: 60,
    devToolsDetection: true,
    consoleInteraction: true,
    networkMonitoring: true
  })

  // Check authentication status and password change requirement
  useEffect(() => {
    const checkAuthStatus = async () => {
      if (isAuthenticated && token) {
        try {
          const response = await fetch('/api/auth/status', {
            headers: {
              'Authorization': `Bearer ${token}`
            }
          })
          
          if (response.ok) {
            const data = await response.json()
            setMustChangePassword(data.must_change_password)
          }
        } catch (error) {
          console.error('Failed to check auth status:', error)
        }
      }
      setIsCheckingAuth(false)
    }

    checkAuthStatus()
  }, [isAuthenticated, token])

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

  // Show loading while checking auth status
  if (isCheckingAuth) {
    return (
      <ThemeProvider>
        <div className="min-h-screen bg-gradient-to-br from-primary-50 to-secondary-100 flex items-center justify-center">
          <div className="text-center">
            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600 mx-auto mb-4"></div>
            <p className="text-secondary-600">Loading...</p>
          </div>
        </div>
      </ThemeProvider>
    )
  }

  return (
    <ThemeProvider>
      <Routes>
        {/* Public routes */}
        <Route path="/login" element={<Login />} />
        
        {/* Protected routes */}
        <Route path="/" element={
          <ProtectedRoute>
            <Layout>
              <Dashboard />
            </Layout>
          </ProtectedRoute>
        } />
        
        <Route path="/dashboard" element={
          <ProtectedRoute>
            <Layout>
              <Dashboard />
            </Layout>
          </ProtectedRoute>
        } />
        
        <Route path="/agents" element={
          <ProtectedRoute>
            <Layout>
              <Agents />
            </Layout>
          </ProtectedRoute>
        } />
        
        <Route path="/security" element={
          <ProtectedRoute>
            <Layout>
              <Security />
            </Layout>
          </ProtectedRoute>
        } />
        
        <Route path="/users" element={
          <ProtectedRoute requiredRoles={['system_admin', 'admin', 'manager']} requiredPermissions={['users.read']}>
            <Layout>
              <Users />
            </Layout>
          </ProtectedRoute>
        } />
        
        <Route path="/subscriptions" element={
          <ProtectedRoute>
            <Layout>
              <Subscriptions />
            </Layout>
          </ProtectedRoute>
        } />
        
        <Route path="/assets" element={
          <ProtectedRoute requiredRoles={['system_admin', 'admin', 'manager']} requiredPermissions={['assets.view']}>
            <Layout>
              <AssetManagement />
            </Layout>
          </ProtectedRoute>
        } />
        
        <Route path="/settings" element={
          <ProtectedRoute>
            <Layout>
              <Settings />
            </Layout>
          </ProtectedRoute>
        } />
      </Routes>
      
      {/* Password Change Modal */}
      {isAuthenticated && (
        <PasswordChangeModal
          isOpen={mustChangePassword}
          onClose={() => {}} // Cannot close if required
          isRequired={true}
          onSuccess={() => setMustChangePassword(false)}
        />
      )}
      
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