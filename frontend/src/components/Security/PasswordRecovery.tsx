import { useState, useEffect } from 'react'
import { motion } from 'framer-motion'
import { 
  LockClosedIcon,
  KeyIcon,
  EyeIcon,
  EyeSlashIcon,
  CheckCircleIcon,
  XCircleIcon,
  ShieldCheckIcon
} from '@heroicons/react/24/outline'
import { toast } from 'react-hot-toast'

interface PasswordRecoveryProps {
  token?: string
  onSuccess?: () => void
  onCancel?: () => void
}

interface PasswordStrength {
  score: number
  feedback: string[]
  requirements: {
    length: boolean
    uppercase: boolean
    lowercase: boolean
    numbers: boolean
    special: boolean
  }
}

export default function PasswordRecovery({ token, onSuccess, onCancel }: PasswordRecoveryProps) {
  const [step, setStep] = useState(token ? 2 : 1)
  const [email, setEmail] = useState('')
  const [isEmailSent, setIsEmailSent] = useState(false)
  const [newPassword, setNewPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [showPassword, setShowPassword] = useState(false)
  const [showConfirmPassword, setShowConfirmPassword] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [tokenValid, setTokenValid] = useState(true)
  const [passwordStrength, setPasswordStrength] = useState<PasswordStrength>({
    score: 0,
    feedback: [],
    requirements: {
      length: false,
      uppercase: false,
      lowercase: false,
      numbers: false,
      special: false
    }
  })

  // Verify token on component mount
  useEffect(() => {
    if (token) {
      verifyResetToken(token)
    }
  }, [token])

  // Update password strength in real-time
  useEffect(() => {
    if (newPassword) {
      setPasswordStrength(analyzePasswordStrength(newPassword))
    } else {
      setPasswordStrength({
        score: 0,
        feedback: [],
        requirements: {
          length: false,
          uppercase: false,
          lowercase: false,
          numbers: false,
          special: false
        }
      })
    }
  }, [newPassword])

  const verifyResetToken = async (token: string) => {
    try {
      // Simulate API call to verify token
      await new Promise(resolve => setTimeout(resolve, 1000))
      
      // In real implementation, call backend to verify token
      const isValid = token.length > 10 // Simple validation for demo
      
      if (!isValid) {
        setTokenValid(false)
        toast.error('Der Reset-Link ist ungültig oder abgelaufen.')
      }
    } catch (error) {
      setTokenValid(false)
      toast.error('Fehler beim Überprüfen des Reset-Links.')
    }
  }

  const analyzePasswordStrength = (password: string): PasswordStrength => {
    const requirements = {
      length: password.length >= 12,
      uppercase: /[A-Z]/.test(password),
      lowercase: /[a-z]/.test(password),
      numbers: /[0-9]/.test(password),
      special: /[^A-Za-z0-9]/.test(password)
    }

    const score = Object.values(requirements).filter(Boolean).length
    const feedback: string[] = []

    if (!requirements.length) feedback.push('Mindestens 12 Zeichen')
    if (!requirements.uppercase) feedback.push('Mindestens 1 Großbuchstabe')
    if (!requirements.lowercase) feedback.push('Mindestens 1 Kleinbuchstabe')
    if (!requirements.numbers) feedback.push('Mindestens 1 Zahl')
    if (!requirements.special) feedback.push('Mindestens 1 Sonderzeichen')

    // Additional security checks
    if (password.length > 0) {
      // Check for common patterns
      if (/123|abc|qwer|pass|admin/i.test(password)) {
        feedback.push('Vermeiden Sie häufige Wörter und Muster')
      }
      
      // Check for repeated characters
      if (/(.)\1{2,}/.test(password)) {
        feedback.push('Vermeiden Sie sich wiederholende Zeichen')
      }

      // Check for keyboard patterns
      if (/qwerty|asdf|zxcv/i.test(password)) {
        feedback.push('Vermeiden Sie Tastaturmuster')
      }
    }

    return { score, feedback, requirements }
  }

  const handleEmailSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!email) {
      toast.error('Bitte geben Sie Ihre E-Mail-Adresse ein.')
      return
    }

    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
    if (!emailRegex.test(email)) {
      toast.error('Bitte geben Sie eine gültige E-Mail-Adresse ein.')
      return
    }

    setIsLoading(true)

    try {
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 2000))
      
      // Log security event
      console.log('Password reset requested for:', email)
      
      setIsEmailSent(true)
      toast.success('Reset-E-Mail wurde gesendet (falls das Konto existiert).')
    } catch (error) {
      toast.error('Fehler beim Senden der Reset-E-Mail.')
    } finally {
      setIsLoading(false)
    }
  }

  const handlePasswordReset = async (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!newPassword || !confirmPassword) {
      toast.error('Bitte füllen Sie alle Felder aus.')
      return
    }

    if (newPassword !== confirmPassword) {
      toast.error('Die Passwörter stimmen nicht überein.')
      return
    }

    if (passwordStrength.score < 5) {
      toast.error('Das Passwort erfüllt nicht alle Sicherheitsanforderungen.')
      return
    }

    setIsLoading(true)

    try {
      // Simulate API call to reset password
      await new Promise(resolve => setTimeout(resolve, 2000))
      
      // Log security event
      console.log('Password reset completed for token:', token)
      
      toast.success('Passwort erfolgreich zurückgesetzt!')
      onSuccess?.()
    } catch (error) {
      toast.error('Fehler beim Zurücksetzen des Passworts.')
    } finally {
      setIsLoading(false)
    }
  }

  const getStrengthColor = (score: number) => {
    if (score <= 1) return 'text-danger-600'
    if (score <= 2) return 'text-warning-600'
    if (score <= 3) return 'text-yellow-600'
    if (score <= 4) return 'text-primary-600'
    return 'text-success-600'
  }

  const getStrengthLabel = (score: number) => {
    if (score <= 1) return 'Sehr schwach'
    if (score <= 2) return 'Schwach'
    if (score <= 3) return 'Mittel'
    if (score <= 4) return 'Stark'
    return 'Sehr stark'
  }

  if (!tokenValid && token) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-primary-50 to-secondary-100 flex items-center justify-center px-4">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          className="max-w-md w-full"
        >
          <div className="card">
            <div className="card-body text-center">
              <div className="mx-auto h-12 w-12 bg-danger-100 rounded-full flex items-center justify-center mb-4">
                <XCircleIcon className="h-6 w-6 text-danger-600" />
              </div>
              <h2 className="text-2xl font-bold text-secondary-900 mb-2">Ungültiger Link</h2>
              <p className="text-secondary-600 mb-6">
                Der Reset-Link ist ungültig oder abgelaufen. Bitte fordern Sie einen neuen an.
              </p>
              <button
                onClick={() => setStep(1)}
                className="btn-primary"
              >
                Neuen Link anfordern
              </button>
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
          <div className="mx-auto h-16 w-16 bg-primary-600 rounded-full flex items-center justify-center mb-4">
            <ShieldCheckIcon className="h-8 w-8 text-white" />
          </div>
          <h1 className="text-3xl font-bold text-secondary-900">SecureGuard</h1>
          <p className="text-secondary-600 mt-2">
            {step === 1 ? 'Passwort zurücksetzen' : 'Neues Passwort festlegen'}
          </p>
        </div>

        <div className="card">
          <div className="card-body">
            {step === 1 && !isEmailSent && (
              <form onSubmit={handleEmailSubmit} className="space-y-6">
                <div className="text-center mb-6">
                  <LockClosedIcon className="mx-auto h-12 w-12 text-secondary-400 mb-4" />
                  <h2 className="text-xl font-semibold text-secondary-900">
                    Passwort vergessen?
                  </h2>
                  <p className="text-secondary-600 mt-2">
                    Geben Sie Ihre E-Mail-Adresse ein und wir senden Ihnen einen Link zum Zurücksetzen.
                  </p>
                </div>

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
                    required
                  />
                </div>

                <div className="flex space-x-3">
                  {onCancel && (
                    <button
                      type="button"
                      onClick={onCancel}
                      className="btn-secondary flex-1"
                    >
                      Abbrechen
                    </button>
                  )}
                  <button
                    type="submit"
                    disabled={isLoading}
                    className="btn-primary flex-1"
                  >
                    {isLoading ? 'Wird gesendet...' : 'Reset-Link senden'}
                  </button>
                </div>
              </form>
            )}

            {step === 1 && isEmailSent && (
              <div className="text-center">
                <CheckCircleIcon className="mx-auto h-12 w-12 text-success-600 mb-4" />
                <h2 className="text-xl font-semibold text-secondary-900 mb-2">
                  E-Mail gesendet!
                </h2>
                <p className="text-secondary-600 mb-6">
                  Falls ein Konto mit dieser E-Mail-Adresse existiert, haben Sie eine E-Mail mit 
                  Anweisungen zum Zurücksetzen Ihres Passworts erhalten.
                </p>
                <p className="text-sm text-secondary-500 mb-6">
                  Überprüfen Sie auch Ihren Spam-Ordner. Der Link ist 1 Stunde gültig.
                </p>
                <button
                  onClick={() => setIsEmailSent(false)}
                  className="btn-secondary"
                >
                  Andere E-Mail verwenden
                </button>
              </div>
            )}

            {step === 2 && (
              <form onSubmit={handlePasswordReset} className="space-y-6">
                <div className="text-center mb-6">
                  <KeyIcon className="mx-auto h-12 w-12 text-secondary-400 mb-4" />
                  <h2 className="text-xl font-semibold text-secondary-900">
                    Neues Passwort festlegen
                  </h2>
                  <p className="text-secondary-600 mt-2">
                    Wählen Sie ein starkes, sicheres Passwort für Ihr Konto.
                  </p>
                </div>

                <div>
                  <label className="block text-sm font-medium text-secondary-700 mb-2">
                    Neues Passwort
                  </label>
                  <div className="relative">
                    <input
                      type={showPassword ? 'text' : 'password'}
                      value={newPassword}
                      onChange={(e) => setNewPassword(e.target.value)}
                      placeholder="Ihr neues sicheres Passwort"
                      className="input pr-10"
                      required
                    />
                    <button
                      type="button"
                      onClick={() => setShowPassword(!showPassword)}
                      className="absolute inset-y-0 right-0 pr-3 flex items-center"
                    >
                      {showPassword ? (
                        <EyeSlashIcon className="h-5 w-5 text-secondary-400" />
                      ) : (
                        <EyeIcon className="h-5 w-5 text-secondary-400" />
                      )}
                    </button>
                  </div>

                  {/* Password Strength Indicator */}
                  {newPassword && (
                    <div className="mt-3">
                      <div className="flex items-center justify-between mb-2">
                        <span className="text-sm text-secondary-600">Passwortstärke:</span>
                        <span className={`text-sm font-medium ${getStrengthColor(passwordStrength.score)}`}>
                          {getStrengthLabel(passwordStrength.score)}
                        </span>
                      </div>
                      
                      <div className="w-full bg-secondary-200 rounded-full h-2 mb-3">
                        <div
                          className={`h-2 rounded-full transition-all duration-300 ${
                            passwordStrength.score <= 1 ? 'bg-danger-500' :
                            passwordStrength.score <= 2 ? 'bg-warning-500' :
                            passwordStrength.score <= 3 ? 'bg-yellow-500' :
                            passwordStrength.score <= 4 ? 'bg-primary-500' :
                            'bg-success-500'
                          }`}
                          style={{ width: `${(passwordStrength.score / 5) * 100}%` }}
                        />
                      </div>

                      <div className="space-y-1">
                        {Object.entries(passwordStrength.requirements).map(([key, met]) => (
                          <div key={key} className="flex items-center space-x-2 text-sm">
                            {met ? (
                              <CheckCircleIcon className="h-4 w-4 text-success-600" />
                            ) : (
                              <XCircleIcon className="h-4 w-4 text-secondary-400" />
                            )}
                            <span className={met ? 'text-success-600' : 'text-secondary-600'}>
                              {key === 'length' && 'Mindestens 12 Zeichen'}
                              {key === 'uppercase' && 'Großbuchstabe'}
                              {key === 'lowercase' && 'Kleinbuchstabe'}
                              {key === 'numbers' && 'Zahl'}
                              {key === 'special' && 'Sonderzeichen'}
                            </span>
                          </div>
                        ))}
                        
                        {passwordStrength.feedback.map((item, index) => (
                          <div key={index} className="flex items-center space-x-2 text-sm">
                            <XCircleIcon className="h-4 w-4 text-warning-600" />
                            <span className="text-warning-600">{item}</span>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </div>

                <div>
                  <label className="block text-sm font-medium text-secondary-700 mb-2">
                    Passwort bestätigen
                  </label>
                  <div className="relative">
                    <input
                      type={showConfirmPassword ? 'text' : 'password'}
                      value={confirmPassword}
                      onChange={(e) => setConfirmPassword(e.target.value)}
                      placeholder="Passwort wiederholen"
                      className="input pr-10"
                      required
                    />
                    <button
                      type="button"
                      onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                      className="absolute inset-y-0 right-0 pr-3 flex items-center"
                    >
                      {showConfirmPassword ? (
                        <EyeSlashIcon className="h-5 w-5 text-secondary-400" />
                      ) : (
                        <EyeIcon className="h-5 w-5 text-secondary-400" />
                      )}
                    </button>
                  </div>
                  
                  {confirmPassword && (
                    <div className="mt-2 flex items-center space-x-2">
                      {newPassword === confirmPassword ? (
                        <>
                          <CheckCircleIcon className="h-4 w-4 text-success-600" />
                          <span className="text-sm text-success-600">Passwörter stimmen überein</span>
                        </>
                      ) : (
                        <>
                          <XCircleIcon className="h-4 w-4 text-danger-600" />
                          <span className="text-sm text-danger-600">Passwörter stimmen nicht überein</span>
                        </>
                      )}
                    </div>
                  )}
                </div>

                <button
                  type="submit"
                  disabled={isLoading || passwordStrength.score < 5 || newPassword !== confirmPassword}
                  className="btn-primary w-full"
                >
                  {isLoading ? 'Wird gespeichert...' : 'Passwort zurücksetzen'}
                </button>

                {onCancel && (
                  <button
                    type="button"
                    onClick={onCancel}
                    className="btn-secondary w-full"
                  >
                    Abbrechen
                  </button>
                )}
              </form>
            )}
          </div>
        </div>
      </motion.div>
    </div>
  )
}