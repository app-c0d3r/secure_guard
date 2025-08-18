import { useEffect, useCallback } from 'react'
import { toast } from 'react-hot-toast'

interface SecurityEvent {
  type: string
  severity: 'low' | 'medium' | 'high' | 'critical'
  timestamp: number
  data: any
  userAgent: string
  url: string
}

interface SecurityThresholds {
  rapidClicks: number
  rapidNavigation: number
  suspiciousKeystrokes: number
  devToolsDetection: boolean
  consoleInteraction: boolean
  networkMonitoring: boolean
}

const DEFAULT_THRESHOLDS: SecurityThresholds = {
  rapidClicks: 20, // clicks per 10 seconds
  rapidNavigation: 10, // navigation events per 30 seconds
  suspiciousKeystrokes: 50, // keystrokes per 5 seconds
  devToolsDetection: true,
  consoleInteraction: true,
  networkMonitoring: true
}

export function useSecurityMonitoring(thresholds: Partial<SecurityThresholds> = {}) {
  const config = { ...DEFAULT_THRESHOLDS, ...thresholds }
  
  // Security event logging
  const logSecurityEvent = useCallback((type: string, severity: SecurityEvent['severity'], data: any) => {
    const event: SecurityEvent = {
      type,
      severity,
      timestamp: Date.now(),
      data,
      userAgent: navigator.userAgent,
      url: window.location.href
    }

    // Store locally
    const events = JSON.parse(localStorage.getItem('security_events') || '[]')
    events.push(event)
    
    // Keep only last 200 events
    if (events.length > 200) {
      events.splice(0, events.length - 200)
    }
    
    localStorage.setItem('security_events', JSON.stringify(events))

    // Send to backend in real implementation
    console.warn('SECURITY EVENT:', event)

    // Show user notification for high/critical events
    if (severity === 'high' || severity === 'critical') {
      toast.error(`Sicherheitsereignis erkannt: ${type}`)
    }
  }, [])

  useEffect(() => {
    const securityMonitors: (() => void)[] = []

    // 1. Developer Tools Detection
    if (config.devToolsDetection) {
      let devToolsOpen = false
      const detectDevTools = () => {
        const threshold = 160
        const widthThreshold = window.outerWidth - window.innerWidth > threshold
        const heightThreshold = window.outerHeight - window.innerHeight > threshold
        
        if ((widthThreshold || heightThreshold) && !devToolsOpen) {
          devToolsOpen = true
          logSecurityEvent('developer_tools_opened', 'medium', {
            outerDimensions: { width: window.outerWidth, height: window.outerHeight },
            innerDimensions: { width: window.innerWidth, height: window.innerHeight }
          })
          
          // Clear console
          console.clear()
          console.warn('🚨 Security monitoring active. Developer tools usage is logged.')
        } else if (!widthThreshold && !heightThreshold && devToolsOpen) {
          devToolsOpen = false
          logSecurityEvent('developer_tools_closed', 'low', {})
        }
      }

      const devToolsInterval = setInterval(detectDevTools, 500)
      securityMonitors.push(() => clearInterval(devToolsInterval))
    }

    // 2. Console Interaction Detection
    if (config.consoleInteraction) {
      const originalLog = console.log
      const originalError = console.error
      const originalWarn = console.warn
      
      console.log = (...args) => {
        logSecurityEvent('console_log_usage', 'low', { args: args.slice(0, 3) })
        return originalLog.apply(console, args)
      }
      
      console.error = (...args) => {
        logSecurityEvent('console_error_usage', 'medium', { args: args.slice(0, 3) })
        return originalError.apply(console, args)
      }
      
      console.warn = (...args) => {
        if (!args[0]?.toString().includes('SECURITY EVENT')) {
          logSecurityEvent('console_warn_usage', 'low', { args: args.slice(0, 3) })
        }
        return originalWarn.apply(console, args)
      }

      securityMonitors.push(() => {
        console.log = originalLog
        console.error = originalError
        console.warn = originalWarn
      })
    }

    // 3. Rapid Click Detection
    if (config.rapidClicks > 0) {
      const clickEvents: number[] = []
      const handleClick = () => {
        const now = Date.now()
        clickEvents.push(now)
        
        // Remove old events (older than 10 seconds)
        const tenSecondsAgo = now - 10000
        while (clickEvents.length > 0 && clickEvents[0] < tenSecondsAgo) {
          clickEvents.shift()
        }
        
        if (clickEvents.length > config.rapidClicks) {
          logSecurityEvent('rapid_clicking_detected', 'medium', {
            clicksInPeriod: clickEvents.length,
            threshold: config.rapidClicks
          })
          clickEvents.length = 0 // Reset
        }
      }

      document.addEventListener('click', handleClick)
      securityMonitors.push(() => document.removeEventListener('click', handleClick))
    }

    // 4. Keyboard Monitoring (for potential automated tools)
    if (config.suspiciousKeystrokes > 0) {
      const keyEvents: number[] = []
      const handleKeyDown = (e: KeyboardEvent) => {
        const now = Date.now()
        keyEvents.push(now)
        
        // Remove old events (older than 5 seconds)
        const fiveSecondsAgo = now - 5000
        while (keyEvents.length > 0 && keyEvents[0] < fiveSecondsAgo) {
          keyEvents.shift()
        }
        
        if (keyEvents.length > config.suspiciousKeystrokes) {
          logSecurityEvent('suspicious_keystroke_pattern', 'high', {
            keystrokesInPeriod: keyEvents.length,
            threshold: config.suspiciousKeystrokes,
            lastKey: e.key
          })
          keyEvents.length = 0 // Reset
        }

        // Detect common automation patterns
        if (e.ctrlKey && e.shiftKey && ['I', 'J', 'C'].includes(e.key)) {
          logSecurityEvent('developer_shortcut_usage', 'low', { key: e.key })
        }
      }

      document.addEventListener('keydown', handleKeyDown)
      securityMonitors.push(() => document.removeEventListener('keydown', handleKeyDown))
    }

    // 5. Navigation Pattern Monitoring
    if (config.rapidNavigation > 0) {
      const navigationEvents: number[] = []
      const handleNavigation = () => {
        const now = Date.now()
        navigationEvents.push(now)
        
        // Remove old events (older than 30 seconds)
        const thirtySecondsAgo = now - 30000
        while (navigationEvents.length > 0 && navigationEvents[0] < thirtySecondsAgo) {
          navigationEvents.shift()
        }
        
        if (navigationEvents.length > config.rapidNavigation) {
          logSecurityEvent('rapid_navigation_detected', 'medium', {
            navigationInPeriod: navigationEvents.length,
            threshold: config.rapidNavigation
          })
        }
      }

      window.addEventListener('popstate', handleNavigation)
      securityMonitors.push(() => window.removeEventListener('popstate', handleNavigation))
    }

    // 6. Window Focus/Blur Monitoring (potential screen recording detection)
    let lastFocusChange = Date.now()
    const handleVisibilityChange = () => {
      const now = Date.now()
      const isHidden = document.hidden
      const timeSinceLastChange = now - lastFocusChange
      
      if (timeSinceLastChange < 100) { // Very rapid focus changes
        logSecurityEvent('rapid_focus_changes', 'medium', {
          isHidden,
          timeSinceLastChange
        })
      }
      
      lastFocusChange = now
      
      if (isHidden) {
        logSecurityEvent('window_lost_focus', 'low', { timestamp: now })
      } else {
        logSecurityEvent('window_gained_focus', 'low', { timestamp: now })
      }
    }

    document.addEventListener('visibilitychange', handleVisibilityChange)
    securityMonitors.push(() => document.removeEventListener('visibilitychange', handleVisibilityChange))

    // 7. Right-click and Context Menu Detection
    const handleContextMenu = (e: MouseEvent) => {
      logSecurityEvent('context_menu_attempt', 'low', {
        x: e.clientX,
        y: e.clientY,
        target: (e.target as Element)?.tagName
      })
      e.preventDefault()
    }

    document.addEventListener('contextmenu', handleContextMenu)
    securityMonitors.push(() => document.removeEventListener('contextmenu', handleContextMenu))

    // 8. Clipboard Monitoring
    const handleCopy = () => {
      logSecurityEvent('clipboard_copy', 'low', { timestamp: Date.now() })
    }

    const handlePaste = () => {
      logSecurityEvent('clipboard_paste', 'low', { timestamp: Date.now() })
    }

    document.addEventListener('copy', handleCopy)
    document.addEventListener('paste', handlePaste)
    securityMonitors.push(() => {
      document.removeEventListener('copy', handleCopy)
      document.removeEventListener('paste', handlePaste)
    })

    // 9. Network Request Monitoring (if enabled)
    if (config.networkMonitoring) {
      const originalFetch = window.fetch
      window.fetch = async (...args) => {
        const url = args[0]?.toString() || 'unknown'
        const startTime = Date.now()
        
        try {
          const response = await originalFetch(...args)
          const endTime = Date.now()
          
          logSecurityEvent('network_request', 'low', {
            url: url.substring(0, 100), // Limit URL length
            status: response.status,
            duration: endTime - startTime,
            method: args[1]?.method || 'GET'
          })
          
          return response
        } catch (error) {
          logSecurityEvent('network_request_failed', 'medium', {
            url: url.substring(0, 100),
            error: (error as Error).message
          })
          throw error
        }
      }

      securityMonitors.push(() => {
        window.fetch = originalFetch
      })
    }

    // 10. Memory Usage Monitoring
    if ('memory' in performance) {
      const memoryMonitor = setInterval(() => {
        const memory = (performance as any).memory
        if (memory) {
          const usedMB = Math.round(memory.usedJSHeapSize / 1048576)
          const totalMB = Math.round(memory.totalJSHeapSize / 1048576)
          
          if (usedMB > 500) { // High memory usage
            logSecurityEvent('high_memory_usage', 'medium', {
              usedMB,
              totalMB,
              limitMB: Math.round(memory.jsHeapSizeLimit / 1048576)
            })
          }
        }
      }, 30000) // Check every 30 seconds

      securityMonitors.push(() => clearInterval(memoryMonitor))
    }

    // Cleanup function
    return () => {
      securityMonitors.forEach(cleanup => cleanup())
    }
  }, [config, logSecurityEvent])

  // Function to get recent security events
  const getSecurityEvents = useCallback((hours: number = 24) => {
    const events = JSON.parse(localStorage.getItem('security_events') || '[]')
    const cutoff = Date.now() - (hours * 60 * 60 * 1000)
    return events.filter((event: SecurityEvent) => event.timestamp > cutoff)
  }, [])

  // Function to clear security events
  const clearSecurityEvents = useCallback(() => {
    localStorage.removeItem('security_events')
  }, [])

  // Function to export security log
  const exportSecurityLog = useCallback(() => {
    const events = JSON.parse(localStorage.getItem('security_events') || '[]')
    const blob = new Blob([JSON.stringify(events, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `security-log-${new Date().toISOString().split('T')[0]}.json`
    a.click()
    URL.revokeObjectURL(url)
  }, [])

  return {
    logSecurityEvent,
    getSecurityEvents,
    clearSecurityEvents,
    exportSecurityLog
  }
}