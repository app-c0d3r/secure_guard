import { useState, useEffect, useCallback } from 'react'
import { toast } from 'react-hot-toast'

interface LoginAttempt {
  email: string
  timestamp: number
  ip?: string
  userAgent?: string
}

interface SecurityState {
  isBlocked: boolean
  blockUntil: number | null
  attemptCount: number
  lastAttempt: number | null
  requiresCaptcha: boolean
  lockoutLevel: number
}

const MAX_ATTEMPTS = 5
const INITIAL_LOCKOUT = 5 * 60 * 1000 // 5 minutes
const CAPTCHA_THRESHOLD = 3
const PROGRESSIVE_LOCKOUT_MULTIPLIER = 2

export function useLoginSecurity(email: string) {
  const [securityState, setSecurityState] = useState<SecurityState>({
    isBlocked: false,
    blockUntil: null,
    attemptCount: 0,
    lastAttempt: null,
    requiresCaptcha: false,
    lockoutLevel: 0
  })

  const getStorageKey = (email: string) => `login_security_${email.toLowerCase()}`
  const getGlobalKey = () => 'global_login_security'

  // Load security state from localStorage
  useEffect(() => {
    if (!email) return

    const key = getStorageKey(email)
    const stored = localStorage.getItem(key)
    const globalStored = localStorage.getItem(getGlobalKey())
    
    if (stored) {
      const data = JSON.parse(stored)
      const now = Date.now()
      
      // Check if lockout has expired
      if (data.blockUntil && now < data.blockUntil) {
        setSecurityState(data)
      } else if (data.blockUntil && now >= data.blockUntil) {
        // Lockout expired, reset but keep attempt history for progressive lockout
        const resetState = {
          ...data,
          isBlocked: false,
          blockUntil: null,
          requiresCaptcha: data.attemptCount >= CAPTCHA_THRESHOLD
        }
        setSecurityState(resetState)
        localStorage.setItem(key, JSON.stringify(resetState))
      } else {
        setSecurityState(data)
      }
    }

    // Check global rate limiting (per IP/browser)
    if (globalStored) {
      const globalData = JSON.parse(globalStored)
      const now = Date.now()
      
      if (globalData.blockUntil && now < globalData.blockUntil) {
        setSecurityState(prev => ({
          ...prev,
          isBlocked: true,
          blockUntil: globalData.blockUntil
        }))
      }
    }
  }, [email])

  // Record failed login attempt
  const recordFailedAttempt = useCallback(() => {
    if (!email) return

    const now = Date.now()
    const key = getStorageKey(email)
    const globalKey = getGlobalKey()

    setSecurityState(prev => {
      const newAttemptCount = prev.attemptCount + 1
      const newLockoutLevel = newAttemptCount >= MAX_ATTEMPTS ? prev.lockoutLevel + 1 : prev.lockoutLevel
      
      let newState: SecurityState = {
        ...prev,
        attemptCount: newAttemptCount,
        lastAttempt: now,
        requiresCaptcha: newAttemptCount >= CAPTCHA_THRESHOLD
      }

      // Apply progressive lockout
      if (newAttemptCount >= MAX_ATTEMPTS) {
        const lockoutDuration = INITIAL_LOCKOUT * Math.pow(PROGRESSIVE_LOCKOUT_MULTIPLIER, newLockoutLevel)
        const blockUntil = now + lockoutDuration

        newState = {
          ...newState,
          isBlocked: true,
          blockUntil,
          lockoutLevel: newLockoutLevel
        }

        // Also set global lockout for this browser/IP
        const globalData = {
          blockUntil,
          attempts: newAttemptCount,
          timestamp: now
        }
        localStorage.setItem(globalKey, JSON.stringify(globalData))

        toast.error(
          `Konto temporär gesperrt für ${Math.ceil(lockoutDuration / 60000)} Minuten. ` +
          `Zu viele fehlgeschlagene Anmeldeversuche.`
        )
      } else {
        const remainingAttempts = MAX_ATTEMPTS - newAttemptCount
        toast.error(
          `Anmeldung fehlgeschlagen. ${remainingAttempts} Versuche verbleibend.`
        )
      }

      localStorage.setItem(key, JSON.stringify(newState))
      return newState
    })

    // Log security event for monitoring
    logSecurityEvent('failed_login_attempt', {
      email,
      attemptCount: securityState.attemptCount + 1,
      timestamp: now,
      userAgent: navigator.userAgent,
      ip: 'client_side' // Server should log actual IP
    })
  }, [email, securityState.attemptCount])

  // Record successful login (reset counters)
  const recordSuccessfulLogin = useCallback(() => {
    if (!email) return

    const key = getStorageKey(email)
    const globalKey = getGlobalKey()
    
    // Reset user-specific state but keep lockout level for future progressive lockout
    const resetState: SecurityState = {
      isBlocked: false,
      blockUntil: null,
      attemptCount: 0,
      lastAttempt: null,
      requiresCaptcha: false,
      lockoutLevel: securityState.lockoutLevel // Keep for progressive lockout
    }

    setSecurityState(resetState)
    localStorage.setItem(key, JSON.stringify(resetState))
    localStorage.removeItem(globalKey)

    logSecurityEvent('successful_login', {
      email,
      timestamp: Date.now()
    })
  }, [email, securityState.lockoutLevel])

  // Check if current time is within suspicious pattern
  const detectSuspiciousPatterns = useCallback(() => {
    const now = Date.now()
    const recentAttempts = getRecentAttempts(60000) // Last minute
    
    // Detect rapid-fire attempts
    if (recentAttempts.length >= 10) {
      logSecurityEvent('rapid_fire_attempts', {
        email,
        attemptCount: recentAttempts.length,
        timeWindow: '1_minute'
      })
      return true
    }

    // Detect distributed attacks (multiple emails from same browser)
    const allEmails = getAllRecentEmails(300000) // Last 5 minutes
    if (allEmails.length >= 5) {
      logSecurityEvent('distributed_attack_pattern', {
        emails: allEmails,
        timeWindow: '5_minutes'
      })
      return true
    }

    return false
  }, [email])

  // Get recent attempts for pattern analysis
  const getRecentAttempts = (timeWindow: number): LoginAttempt[] => {
    const now = Date.now()
    const attempts: LoginAttempt[] = []
    
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i)
      if (key?.startsWith('login_security_')) {
        const data = JSON.parse(localStorage.getItem(key) || '{}')
        if (data.lastAttempt && (now - data.lastAttempt) <= timeWindow) {
          attempts.push({
            email: key.replace('login_security_', ''),
            timestamp: data.lastAttempt
          })
        }
      }
    }
    
    return attempts.sort((a, b) => b.timestamp - a.timestamp)
  }

  // Get all emails attempted recently (for detecting distributed attacks)
  const getAllRecentEmails = (timeWindow: number): string[] => {
    const now = Date.now()
    const emails: string[] = []
    
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i)
      if (key?.startsWith('login_security_')) {
        const data = JSON.parse(localStorage.getItem(key) || '{}')
        if (data.lastAttempt && (now - data.lastAttempt) <= timeWindow) {
          emails.push(key.replace('login_security_', ''))
        }
      }
    }
    
    return emails
  }

  // Log security events for monitoring
  const logSecurityEvent = (eventType: string, data: any) => {
    const securityLog = {
      type: eventType,
      timestamp: Date.now(),
      userAgent: navigator.userAgent,
      url: window.location.href,
      data
    }

    // Store locally for potential sync
    const logs = JSON.parse(localStorage.getItem('security_logs') || '[]')
    logs.push(securityLog)
    
    // Keep only last 100 logs
    if (logs.length > 100) {
      logs.splice(0, logs.length - 100)
    }
    
    localStorage.setItem('security_logs', JSON.stringify(logs))

    // In real implementation, send to backend immediately
    console.warn('SECURITY EVENT:', securityLog)
  }

  // Get time remaining in lockout
  const getTimeRemaining = (): number => {
    if (!securityState.blockUntil) return 0
    return Math.max(0, securityState.blockUntil - Date.now())
  }

  // Format time remaining for display
  const formatTimeRemaining = (): string => {
    const remaining = getTimeRemaining()
    if (remaining === 0) return ''
    
    const minutes = Math.ceil(remaining / 60000)
    if (minutes === 1) return '1 Minute'
    return `${minutes} Minuten`
  }

  // Validate if login attempt is allowed
  const canAttemptLogin = (): boolean => {
    if (securityState.isBlocked && getTimeRemaining() > 0) {
      return false
    }
    
    if (detectSuspiciousPatterns()) {
      return false
    }

    return true
  }

  return {
    securityState,
    recordFailedAttempt,
    recordSuccessfulLogin,
    canAttemptLogin,
    getTimeRemaining,
    formatTimeRemaining,
    detectSuspiciousPatterns
  }
}