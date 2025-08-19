import { useState, useEffect } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { 
  EyeIcon, 
  EyeSlashIcon,
  LockClosedIcon,
  CheckCircleIcon,
  XCircleIcon
} from '@heroicons/react/24/outline'
import { toast } from 'react-hot-toast'

interface PasswordPolicy {
  min_length: number
  require_uppercase: boolean
  require_lowercase: boolean
  require_numbers: boolean
  require_special_chars: boolean
  max_age_days: number
}

interface PasswordChangeModalProps {
  isOpen: boolean
  onClose: () => void
  isRequired?: boolean
  onSuccess: () => void
}

export default function PasswordChangeModal({ 
  isOpen, 
  onClose, 
  isRequired = false,
  onSuccess 
}: PasswordChangeModalProps) {
  const [oldPassword, setOldPassword] = useState('')
  const [newPassword, setNewPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [showPasswords, setShowPasswords] = useState({
    old: false,
    new: false,
    confirm: false
  })
  const [isLoading, setIsLoading] = useState(false)
  const [policy, setPolicy] = useState<PasswordPolicy | null>(null)
  const [validationErrors, setValidationErrors] = useState<string[]>([])

  useEffect(() => {
    if (isOpen) {
      fetchPasswordPolicy()
    }
  }, [isOpen])

  useEffect(() => {
    if (newPassword) {
      validatePassword(newPassword)
    } else {
      setValidationErrors([])
    }
  }, [newPassword, policy])

  const fetchPasswordPolicy = async () => {
    try {
      const response = await fetch('/api/auth/password-policy')
      if (response.ok) {
        const data = await response.json()
        setPolicy(data.policy)
      }
    } catch (error) {
      console.error('Failed to fetch password policy:', error)
    }
  }

  const validatePassword = (password: string) => {
    if (!policy) return

    const errors: string[] = []

    if (password.length < policy.min_length) {
      errors.push(`Must be at least ${policy.min_length} characters long`)
    }

    if (policy.require_uppercase && !/[A-Z]/.test(password)) {
      errors.push('Must contain at least one uppercase letter')
    }

    if (policy.require_lowercase && !/[a-z]/.test(password)) {
      errors.push('Must contain at least one lowercase letter')
    }

    if (policy.require_numbers && !/[0-9]/.test(password)) {
      errors.push('Must contain at least one number')
    }

    if (policy.require_special_chars && !/[^a-zA-Z0-9]/.test(password)) {
      errors.push('Must contain at least one special character')
    }

    setValidationErrors(errors)
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    if (validationErrors.length > 0) {
      toast.error('Please fix password validation errors')
      return
    }

    if (newPassword !== confirmPassword) {
      toast.error('New passwords do not match')
      return
    }

    if (newPassword === oldPassword) {
      toast.error('New password must be different from current password')
      return
    }

    setIsLoading(true)

    try {
      const response = await fetch('/api/auth/change-password', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('auth_token')}`
        },
        body: JSON.stringify({
          old_password: oldPassword,
          new_password: newPassword
        })
      })

      if (response.ok) {
        toast.success('Password changed successfully!')
        resetForm()
        onSuccess()
        onClose()
      } else {
        const error = await response.json()
        toast.error(error.error || 'Failed to change password')
      }
    } catch (error) {
      toast.error('Failed to change password. Please try again.')
    } finally {
      setIsLoading(false)
    }
  }

  const resetForm = () => {
    setOldPassword('')
    setNewPassword('')
    setConfirmPassword('')
    setValidationErrors([])
    setShowPasswords({ old: false, new: false, confirm: false })
  }

  const togglePasswordVisibility = (field: 'old' | 'new' | 'confirm') => {
    setShowPasswords(prev => ({
      ...prev,
      [field]: !prev[field]
    }))
  }

  const isFormValid = oldPassword && newPassword && confirmPassword && 
                     validationErrors.length === 0 && newPassword === confirmPassword

  return (
    <AnimatePresence>
      {isOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black bg-opacity-50">
          <motion.div
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.95 }}
            className="w-full max-w-md bg-white rounded-xl shadow-xl"
          >
            <div className="px-6 py-4 border-b border-secondary-200">
              <div className="flex items-center space-x-2">
                <div className="w-8 h-8 bg-primary-100 rounded-full flex items-center justify-center">
                  <LockClosedIcon className="w-4 h-4 text-primary-600" />
                </div>
                <div>
                  <h3 className="text-lg font-semibold text-secondary-900">
                    {isRequired ? 'Password Change Required' : 'Change Password'}
                  </h3>
                  {isRequired && (
                    <p className="text-sm text-warning-600">
                      You must change your password before continuing
                    </p>
                  )}
                </div>
              </div>
            </div>

            <form onSubmit={handleSubmit} className="px-6 py-4 space-y-4">
              {/* Current Password */}
              <div>
                <label className="label">Current Password</label>
                <div className="relative">
                  <input
                    type={showPasswords.old ? 'text' : 'password'}
                    value={oldPassword}
                    onChange={(e) => setOldPassword(e.target.value)}
                    className="input pr-10"
                    placeholder="Enter your current password"
                    required
                  />
                  <button
                    type="button"
                    onClick={() => togglePasswordVisibility('old')}
                    className="absolute inset-y-0 right-0 pr-3 flex items-center"
                  >
                    {showPasswords.old ? (
                      <EyeSlashIcon className="h-5 w-5 text-secondary-400" />
                    ) : (
                      <EyeIcon className="h-5 w-5 text-secondary-400" />
                    )}
                  </button>
                </div>
              </div>

              {/* New Password */}
              <div>
                <label className="label">New Password</label>
                <div className="relative">
                  <input
                    type={showPasswords.new ? 'text' : 'password'}
                    value={newPassword}
                    onChange={(e) => setNewPassword(e.target.value)}
                    className="input pr-10"
                    placeholder="Enter your new password"
                    required
                  />
                  <button
                    type="button"
                    onClick={() => togglePasswordVisibility('new')}
                    className="absolute inset-y-0 right-0 pr-3 flex items-center"
                  >
                    {showPasswords.new ? (
                      <EyeSlashIcon className="h-5 w-5 text-secondary-400" />
                    ) : (
                      <EyeIcon className="h-5 w-5 text-secondary-400" />
                    )}
                  </button>
                </div>

                {/* Password Policy Validation */}
                {policy && newPassword && (
                  <div className="mt-2 space-y-1">
                    <PasswordRequirement 
                      met={newPassword.length >= policy.min_length}
                      text={`At least ${policy.min_length} characters`}
                    />
                    {policy.require_uppercase && (
                      <PasswordRequirement 
                        met={/[A-Z]/.test(newPassword)}
                        text="One uppercase letter"
                      />
                    )}
                    {policy.require_lowercase && (
                      <PasswordRequirement 
                        met={/[a-z]/.test(newPassword)}
                        text="One lowercase letter"
                      />
                    )}
                    {policy.require_numbers && (
                      <PasswordRequirement 
                        met={/[0-9]/.test(newPassword)}
                        text="One number"
                      />
                    )}
                    {policy.require_special_chars && (
                      <PasswordRequirement 
                        met={/[^a-zA-Z0-9]/.test(newPassword)}
                        text="One special character"
                      />
                    )}
                  </div>
                )}
              </div>

              {/* Confirm Password */}
              <div>
                <label className="label">Confirm New Password</label>
                <div className="relative">
                  <input
                    type={showPasswords.confirm ? 'text' : 'password'}
                    value={confirmPassword}
                    onChange={(e) => setConfirmPassword(e.target.value)}
                    className="input pr-10"
                    placeholder="Confirm your new password"
                    required
                  />
                  <button
                    type="button"
                    onClick={() => togglePasswordVisibility('confirm')}
                    className="absolute inset-y-0 right-0 pr-3 flex items-center"
                  >
                    {showPasswords.confirm ? (
                      <EyeSlashIcon className="h-5 w-5 text-secondary-400" />
                    ) : (
                      <EyeIcon className="h-5 w-5 text-secondary-400" />
                    )}
                  </button>
                </div>
                {confirmPassword && newPassword !== confirmPassword && (
                  <p className="mt-1 text-sm text-danger-600">
                    Passwords do not match
                  </p>
                )}
              </div>
            </form>

            <div className="px-6 py-4 border-t border-secondary-200 flex space-x-3">
              {!isRequired && (
                <button
                  type="button"
                  onClick={onClose}
                  className="btn-secondary flex-1"
                  disabled={isLoading}
                >
                  Cancel
                </button>
              )}
              <button
                type="submit"
                onClick={handleSubmit}
                disabled={!isFormValid || isLoading}
                className="btn-primary flex-1"
              >
                {isLoading ? 'Changing...' : 'Change Password'}
              </button>
            </div>
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  )
}

interface PasswordRequirementProps {
  met: boolean
  text: string
}

function PasswordRequirement({ met, text }: PasswordRequirementProps) {
  return (
    <div className="flex items-center space-x-2 text-xs">
      {met ? (
        <CheckCircleIcon className="w-4 h-4 text-success-500" />
      ) : (
        <XCircleIcon className="w-4 h-4 text-secondary-400" />
      )}
      <span className={met ? 'text-success-600' : 'text-secondary-500'}>
        {text}
      </span>
    </div>
  )
}