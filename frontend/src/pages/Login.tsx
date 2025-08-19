import { useState, useEffect } from 'react'
import { motion } from 'framer-motion'
import { 
  EyeIcon, 
  EyeSlashIcon,
  ShieldCheckIcon,
  ExclamationTriangleIcon,
  ClockIcon,
  LockClosedIcon
} from '@heroicons/react/24/outline'
import { useAuthStore } from '@/stores/authStore'
import { useLoginSecurity } from '@/hooks/useLoginSecurity'
import CaptchaComponent from '@/components/Security/CaptchaComponent'
import { toast } from 'react-hot-toast'

export default function Login() {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [showPassword, setShowPassword] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [captchaToken, setCaptchaToken] = useState<string | null>(null)
  const [showForgotPassword, setShowForgotPassword] = useState(false)
  const [resetEmail, setResetEmail] = useState('')
  const [isResetLoading, setIsResetLoading] = useState(false)

  const { login } = useAuthStore()
  const {
    securityState,
    recordFailedAttempt,
    recordSuccessfulLogin,
    canAttemptLogin,
    formatTimeRemaining
  } = useLoginSecurity(email)

  // Auto-update countdown timer
  useEffect(() => {
    if (securityState.isBlocked) {
      const interval = setInterval(() => {
        // Force re-render to update countdown
      }, 1000)
      return () => clearInterval(interval)
    }
  }, [securityState.isBlocked])

  // Additional frontend security measures
  useEffect(() => {
    // Disable right-click context menu on login page
    const handleContextMenu = (e: MouseEvent) => e.preventDefault()
    document.addEventListener('contextmenu', handleContextMenu)

    // Detect developer tools
    const detectDevTools = () => {
      const threshold = 160
      if (window.outerHeight - window.innerHeight > threshold || 
          window.outerWidth - window.innerWidth > threshold) {
        console.clear()
        console.warn('Developer tools detected. Security monitoring active.')
      }
    }

    const devToolsInterval = setInterval(detectDevTools, 1000)

    // Disable common keyboard shortcuts
    const handleKeyDown = (e: KeyboardEvent) => {
      // Disable F12, Ctrl+Shift+I, Ctrl+U, Ctrl+S
      if (e.key === 'F12' || 
          (e.ctrlKey && e.shiftKey && e.key === 'I') ||
          (e.ctrlKey && e.key === 'u') ||
          (e.ctrlKey && e.key === 's')) {
        e.preventDefault()
        toast.error('Diese Aktion ist aus Sicherheitsgründen deaktiviert.')
      }
    }

    document.addEventListener('keydown', handleKeyDown)

    return () => {
      document.removeEventListener('contextmenu', handleContextMenu)
      document.removeEventListener('keydown', handleKeyDown)
      clearInterval(devToolsInterval)
    }
  }, [])

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!canAttemptLogin()) {
      toast.error('Login temporarily blocked due to security measures.')
      return
    }

    if (securityState.requiresCaptcha && !captchaToken) {
      toast.error('Bitte lösen Sie das CAPTCHA zur Sicherheitsverifizierung.')
      return
    }

    if (!email || !password) {
      toast.error('Bitte füllen Sie alle Felder aus.')
      return
    }

    // Email validation
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
    if (!emailRegex.test(email)) {
      toast.error('Bitte geben Sie eine gültige E-Mail-Adresse ein.')
      return
    }

    // Password strength check
    if (password.length < 8) {
      toast.error('Das Passwort muss mindestens 8 Zeichen lang sein.')
      return
    }

    setIsLoading(true)

    try {
      // In real implementation, this would be an API call with security headers
      // including email, password, captchaToken, and browser fingerprint
      await new Promise(resolve => setTimeout(resolve, 1500))
      
      // Simulate login success/failure
      const isValidLogin = email === 'admin@company.com' && password === 'SecurePass123!'
      
      if (isValidLogin) {
        recordSuccessfulLogin()
        
        const token = 'mock_jwt_token_' + Date.now()
        const user = {
          id: '1',
          username: 'admin',
          name: 'Admin User',
          email,
          role: 'admin' as const,
          permissions: ['read', 'write', 'admin', 'control_agents', 'view_assets', 'assets.view', 'users.read', 'secrets.read', 'subscriptions.read', 'security.incidents'],
          canAccessSecrets: true,
          canManageUsers: true,
          canAdminSystem: true,
          canControlAgents: true,
          canViewAssets: true
        }
        
        login(token, user)
        toast.success('Anmeldung erfolgreich!')
      } else {
        recordFailedAttempt()
        throw new Error('Invalid credentials')
      }
    } catch (error) {
      // Don't reveal specific error details for security
      toast.error('Anmeldung fehlgeschlagen. Bitte prüfen Sie Ihre Eingaben.')
    } finally {
      setIsLoading(false)
      setCaptchaToken(null) // Reset CAPTCHA
    }
  }

  const handleForgotPassword = async (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!resetEmail) {
      toast.error('Bitte geben Sie Ihre E-Mail-Adresse ein.')
      return
    }

    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
    if (!emailRegex.test(resetEmail)) {
      toast.error('Bitte geben Sie eine gültige E-Mail-Adresse ein.')
      return
    }

    setIsResetLoading(true)

    try {
      // Simulate password reset API call
      await new Promise(resolve => setTimeout(resolve, 2000))
      
      toast.success(
        'Falls ein Konto mit dieser E-Mail-Adresse existiert, ' +
        'erhalten Sie eine E-Mail mit Anweisungen zum Zurücksetzen Ihres Passworts.'
      )
      
      setShowForgotPassword(false)
      setResetEmail('')
    } catch (error) {
      toast.error('Ein Fehler ist aufgetreten. Bitte versuchen Sie es später erneut.')
    } finally {
      setIsResetLoading(false)
    }
  }

  if (showForgotPassword) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-primary-50 to-secondary-100 flex items-center justify-center px-4">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          className="max-w-md w-full"
        >
          <div className="card">
            <div className="card-body">
              <div className="text-center mb-8">
                <div className="mx-auto h-12 w-12 bg-primary-100 rounded-full flex items-center justify-center mb-4">
                  <LockClosedIcon className="icon-lg text-primary-600" />
                </div>
                <h2 className="text-2xl font-bold text-secondary-900">Passwort zurücksetzen</h2>
                <p className="text-secondary-600 mt-2">
                  Geben Sie Ihre E-Mail-Adresse ein, um ein neues Passwort zu erhalten.
                </p>
              </div>

              <form onSubmit={handleForgotPassword} className="space-y-6">
                <div>
                  <label className="block text-sm font-medium text-secondary-700 mb-2">
                    E-Mail-Adresse
                  </label>
                  <input
                    type="email"
                    value={resetEmail}
                    onChange={(e) => setResetEmail(e.target.value)}
                    placeholder="ihre.email@unternehmen.com"
                    className="input"
                    required
                  />
                </div>

                <div className="flex space-x-3">
                  <button
                    type="button"
                    onClick={() => setShowForgotPassword(false)}
                    className="btn-secondary flex-1"
                  >
                    Zurück
                  </button>
                  <button
                    type="submit"
                    disabled={isResetLoading}
                    className="btn-primary flex-1"
                  >
                    {isResetLoading ? 'Wird gesendet...' : 'Zurücksetzen'}
                  </button>
                </div>
              </form>
            </div>
          </div>
        </motion.div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-primary-50 to-secondary-100 flex items-center justify-center px-4">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className="max-w-md w-full"
      >
        <div className="text-center mb-8">
          <div className="mx-auto h-12 w-12 bg-primary-600 rounded-full flex items-center justify-center mb-4">
            <ShieldCheckIcon className="icon-lg text-white" />
          </div>
          <h1 className="text-3xl font-bold text-secondary-900">SecureGuard</h1>
          <p className="text-secondary-600 mt-2">Sicheres Anmelden in Ihr Konto</p>
        </div>

        <div className="card">
          <div className="card-body">
            {/* Security Warning if blocked */}
            {securityState.isBlocked && (
              <motion.div
                initial={{ opacity: 0, scale: 0.95 }}
                animate={{ opacity: 1, scale: 1 }}
                className="bg-danger-50 border border-danger-200 rounded-lg p-4 mb-6"
              >
                <div className="flex items-center space-x-2 text-danger-800">
                  <ExclamationTriangleIcon className="h-5 w-5" />
                  <span className="font-medium">Konto temporär gesperrt</span>
                </div>
                <p className="text-sm text-danger-700 mt-1">
                  Zu viele fehlgeschlagene Anmeldeversuche. 
                  Versuchen Sie es in {formatTimeRemaining()} erneut.
                </p>
                <div className="flex items-center space-x-1 mt-2 text-sm text-danger-600">
                  <ClockIcon className="h-4 w-4" />
                  <span>Automatische Entsperrung läuft...</span>
                </div>
              </motion.div>
            )}

            {/* Security Info */}
            {securityState.attemptCount > 0 && !securityState.isBlocked && (
              <motion.div
                initial={{ opacity: 0, scale: 0.95 }}
                animate={{ opacity: 1, scale: 1 }}
                className="bg-warning-50 border border-warning-200 rounded-lg p-4 mb-6"
              >
                <div className="flex items-center space-x-2 text-warning-800">
                  <ExclamationTriangleIcon className="h-5 w-5" />
                  <span className="font-medium">Sicherheitshinweis</span>
                </div>
                <p className="text-sm text-warning-700 mt-1">
                  {5 - securityState.attemptCount} Anmeldeversuche verbleibend
                </p>
              </motion.div>
            )}

            <form onSubmit={handleLogin} className="space-y-6">
              <div>
                <label className="block text-sm font-medium text-secondary-700 mb-2">
                  E-Mail-Adresse
                </label>
                <input
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="ihre.email@unternehmen.com"
                  className="input"
                  disabled={securityState.isBlocked}
                  required
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-secondary-700 mb-2">
                  Passwort
                </label>
                <div className="relative">
                  <input
                    type={showPassword ? 'text' : 'password'}
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    placeholder="Ihr sicheres Passwort"
                    className="input pr-10"
                    disabled={securityState.isBlocked}
                    required
                  />
                  <button
                    type="button"
                    onClick={() => setShowPassword(!showPassword)}
                    className="absolute inset-y-0 right-0 pr-3 flex items-center"
                    disabled={securityState.isBlocked}
                  >
                    {showPassword ? (
                      <EyeSlashIcon className="h-5 w-5 text-secondary-400" />
                    ) : (
                      <EyeIcon className="h-5 w-5 text-secondary-400" />
                    )}
                  </button>
                </div>
              </div>

              {/* CAPTCHA when required */}
              {securityState.requiresCaptcha && !securityState.isBlocked && (
                <CaptchaComponent
                  onVerify={setCaptchaToken}
                  onError={(error) => toast.error(error)}
                  difficulty="medium"
                />
              )}

              <button
                type="submit"
                disabled={isLoading || securityState.isBlocked || (securityState.requiresCaptcha && !captchaToken)}
                className="btn-primary w-full"
              >
                {isLoading ? 'Wird angemeldet...' : 'Anmelden'}
              </button>
            </form>

            <div className="mt-6 text-center">
              <button
                onClick={() => setShowForgotPassword(true)}
                className="text-sm text-primary-600 hover:text-primary-500 transition-colors"
                disabled={securityState.isBlocked}
              >
                Passwort vergessen?
              </button>
            </div>

            {/* Security Footer */}
            <div className="mt-6 pt-6 border-t border-secondary-100">
              <div className="text-xs text-secondary-500 text-center space-y-1">
                <p>🔒 Diese Verbindung ist verschlüsselt und sicher</p>
                <p>🛡️ Brute-Force-Schutz aktiv</p>
                <p>📊 Sicherheitsereignisse werden protokolliert</p>
              </div>
            </div>

            {/* Production warning - remove demo credentials in production */}
            {process.env.NODE_ENV === 'development' && (
              <div className="mt-4 p-3 bg-blue-50 border border-blue-200 rounded-lg">
                <p className="text-xs text-blue-800 text-center">
                  <strong>Demo:</strong> admin@company.com / SecurePass123!
                </p>
              </div>
            )}
          </div>
        </div>
      </motion.div>
    </div>
  )
}