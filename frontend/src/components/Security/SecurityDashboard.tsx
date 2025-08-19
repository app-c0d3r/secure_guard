import { useState, useEffect } from 'react'
import { motion } from 'framer-motion'
import { 
  ShieldExclamationIcon,
  ComputerDesktopIcon,
  ExclamationTriangleIcon,
  InformationCircleIcon,
  ArrowDownTrayIcon,
  MagnifyingGlassIcon,
  ClockIcon
} from '@heroicons/react/24/outline'
import { useSecurityMonitoring } from '@/hooks/useSecurityMonitoring'
import { cn } from '@/lib/utils'

interface SecurityEvent {
  type: string
  severity: 'low' | 'medium' | 'high' | 'critical'
  timestamp: number
  data: any
  userAgent: string
  url: string
}

const severityConfig = {
  critical: {
    color: 'text-danger-600',
    bgColor: 'bg-danger-50',
    badge: 'bg-danger-100 text-danger-800',
    icon: ShieldExclamationIcon
  },
  high: {
    color: 'text-warning-600',
    bgColor: 'bg-warning-50',
    badge: 'bg-warning-100 text-warning-800',
    icon: ExclamationTriangleIcon
  },
  medium: {
    color: 'text-primary-600',
    bgColor: 'bg-primary-50',
    badge: 'bg-primary-100 text-primary-800',
    icon: ExclamationTriangleIcon
  },
  low: {
    color: 'text-secondary-600',
    bgColor: 'bg-secondary-50',
    badge: 'bg-secondary-100 text-secondary-800',
    icon: InformationCircleIcon
  }
}

const eventTypeLabels: Record<string, string> = {
  developer_tools_opened: 'Entwicklertools geöffnet',
  developer_tools_closed: 'Entwicklertools geschlossen',
  console_log_usage: 'Konsole-Log-Nutzung',
  console_error_usage: 'Konsole-Error-Nutzung',
  console_warn_usage: 'Konsole-Warn-Nutzung',
  rapid_clicking_detected: 'Schnelle Klicks erkannt',
  suspicious_keystroke_pattern: 'Verdächtige Tastenanschläge',
  developer_shortcut_usage: 'Entwickler-Shortcuts',
  rapid_navigation_detected: 'Schnelle Navigation',
  rapid_focus_changes: 'Schnelle Fokusänderungen',
  window_lost_focus: 'Fenster-Fokus verloren',
  window_gained_focus: 'Fenster-Fokus erhalten',
  context_menu_attempt: 'Kontextmenü-Versuch',
  clipboard_copy: 'Zwischenablage-Kopieren',
  clipboard_paste: 'Zwischenablage-Einfügen',
  network_request: 'Netzwerk-Anfrage',
  network_request_failed: 'Netzwerk-Anfrage fehlgeschlagen',
  high_memory_usage: 'Hoher Speicherverbrauch',
  failed_login_attempt: 'Fehlgeschlagener Login',
  successful_login: 'Erfolgreicher Login'
}

export default function SecurityDashboard() {
  const [events, setEvents] = useState<SecurityEvent[]>([])
  const [filteredEvents, setFilteredEvents] = useState<SecurityEvent[]>([])
  const [searchTerm, setSearchTerm] = useState('')
  const [severityFilter, setSeverityFilter] = useState<string>('all')
  const [timeRange, setTimeRange] = useState<number>(24) // hours
  const { getSecurityEvents, clearSecurityEvents, exportSecurityLog } = useSecurityMonitoring()

  useEffect(() => {
    const loadEvents = () => {
      const securityEvents = getSecurityEvents(timeRange)
      setEvents(securityEvents)
    }

    loadEvents()
    const interval = setInterval(loadEvents, 5000) // Refresh every 5 seconds

    return () => clearInterval(interval)
  }, [getSecurityEvents, timeRange])

  useEffect(() => {
    let filtered = events

    // Apply search filter
    if (searchTerm) {
      filtered = filtered.filter(event =>
        eventTypeLabels[event.type]?.toLowerCase().includes(searchTerm.toLowerCase()) ||
        event.type.toLowerCase().includes(searchTerm.toLowerCase()) ||
        JSON.stringify(event.data).toLowerCase().includes(searchTerm.toLowerCase())
      )
    }

    // Apply severity filter
    if (severityFilter !== 'all') {
      filtered = filtered.filter(event => event.severity === severityFilter)
    }

    // Sort by timestamp (newest first)
    filtered.sort((a, b) => b.timestamp - a.timestamp)

    setFilteredEvents(filtered)
  }, [events, searchTerm, severityFilter])

  const getEventCounts = () => {
    const counts = {
      total: events.length,
      critical: events.filter(e => e.severity === 'critical').length,
      high: events.filter(e => e.severity === 'high').length,
      medium: events.filter(e => e.severity === 'medium').length,
      low: events.filter(e => e.severity === 'low').length
    }
    return counts
  }

  const formatTimestamp = (timestamp: number) => {
    return new Date(timestamp).toLocaleString('de-DE')
  }

  const formatEventData = (data: any) => {
    if (!data || Object.keys(data).length === 0) return null
    
    return Object.entries(data)
      .slice(0, 3) // Show only first 3 properties
      .map(([key, value]) => (
        <span key={key} className="inline-block mr-3 text-xs">
          <span className="font-medium">{key}:</span> {String(value).substring(0, 50)}
        </span>
      ))
  }

  const counts = getEventCounts()

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-bold text-secondary-900">Sicherheitsüberwachung</h2>
          <p className="text-secondary-600 mt-1">
            Echtzeitüberwachung von Sicherheitsereignissen und verdächtigen Aktivitäten
          </p>
        </div>
        <div className="flex items-center space-x-3">
          <button
            onClick={exportSecurityLog}
            className="btn-secondary flex items-center space-x-2"
          >
            <ArrowDownTrayIcon className="h-4 w-4" />
            <span>Exportieren</span>
          </button>
          <button
            onClick={() => {
              if (confirm('Möchten Sie wirklich alle Sicherheitsereignisse löschen?')) {
                clearSecurityEvents()
                setEvents([])
              }
            }}
            className="btn-danger"
          >
            Ereignisse löschen
          </button>
        </div>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-5 gap-4">
        <div className="card">
          <div className="card-body flex items-center space-x-3">
            <div className="p-2 bg-secondary-100 rounded-lg">
              <ComputerDesktopIcon className="h-6 w-6 text-secondary-600" />
            </div>
            <div>
              <p className="text-sm text-secondary-500">Gesamt</p>
              <p className="text-xl font-bold text-secondary-900">{counts.total}</p>
            </div>
          </div>
        </div>
        
        <div className="card">
          <div className="card-body flex items-center space-x-3">
            <div className="p-2 bg-danger-100 rounded-lg">
              <ShieldExclamationIcon className="h-6 w-6 text-danger-600" />
            </div>
            <div>
              <p className="text-sm text-secondary-500">Kritisch</p>
              <p className="text-xl font-bold text-secondary-900">{counts.critical}</p>
            </div>
          </div>
        </div>
        
        <div className="card">
          <div className="card-body flex items-center space-x-3">
            <div className="p-2 bg-warning-100 rounded-lg">
              <ExclamationTriangleIcon className="h-6 w-6 text-warning-600" />
            </div>
            <div>
              <p className="text-sm text-secondary-500">Hoch</p>
              <p className="text-xl font-bold text-secondary-900">{counts.high}</p>
            </div>
          </div>
        </div>
        
        <div className="card">
          <div className="card-body flex items-center space-x-3">
            <div className="p-2 bg-primary-100 rounded-lg">
              <ExclamationTriangleIcon className="h-6 w-6 text-primary-600" />
            </div>
            <div>
              <p className="text-sm text-secondary-500">Mittel</p>
              <p className="text-xl font-bold text-secondary-900">{counts.medium}</p>
            </div>
          </div>
        </div>
        
        <div className="card">
          <div className="card-body flex items-center space-x-3">
            <div className="p-2 bg-secondary-100 rounded-lg">
              <InformationCircleIcon className="h-6 w-6 text-secondary-600" />
            </div>
            <div>
              <p className="text-sm text-secondary-500">Niedrig</p>
              <p className="text-xl font-bold text-secondary-900">{counts.low}</p>
            </div>
          </div>
        </div>
      </div>

      {/* Filters */}
      <div className="flex flex-col sm:flex-row gap-4">
        <div className="flex-1 relative">
          <MagnifyingGlassIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-secondary-400" />
          <input
            type="text"
            placeholder="Ereignisse suchen..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="input pl-10"
          />
        </div>
        
        <select
          value={severityFilter}
          onChange={(e) => setSeverityFilter(e.target.value)}
          className="input w-auto"
        >
          <option value="all">Alle Schweregrade</option>
          <option value="critical">Kritisch</option>
          <option value="high">Hoch</option>
          <option value="medium">Mittel</option>
          <option value="low">Niedrig</option>
        </select>
        
        <select
          value={timeRange}
          onChange={(e) => setTimeRange(Number(e.target.value))}
          className="input w-auto"
        >
          <option value={1}>Letzte Stunde</option>
          <option value={6}>Letzten 6 Stunden</option>
          <option value={24}>Letzten 24 Stunden</option>
          <option value={168}>Letzte Woche</option>
          <option value={720}>Letzter Monat</option>
        </select>
      </div>

      {/* Events List */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        className="card"
      >
        <div className="card-header">
          <h3 className="text-lg font-semibold text-secondary-900">
            Sicherheitsereignisse ({filteredEvents.length})
          </h3>
          <p className="text-sm text-secondary-500">
            Sortiert nach Zeit (neueste zuerst)
          </p>
        </div>
        
        <div className="card-body p-0">
          {filteredEvents.length === 0 ? (
            <div className="text-center py-12">
              <ShieldExclamationIcon className="mx-auto icon-3xl text-secondary-400" />
              <h3 className="mt-2 text-sm font-medium text-secondary-900">
                Keine Ereignisse gefunden
              </h3>
              <p className="mt-1 text-sm text-secondary-500">
                Versuchen Sie, Ihre Filter zu ändern oder erweitern Sie den Zeitraum.
              </p>
            </div>
          ) : (
            <div className="divide-y divide-secondary-100 max-h-96 overflow-y-auto">
              {filteredEvents.map((event, index) => {
                const config = severityConfig[event.severity]
                const Icon = config.icon
                
                return (
                  <motion.div
                    key={`${event.timestamp}-${index}`}
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ delay: index * 0.05 }}
                    className={cn(
                      'p-4 hover:bg-secondary-50 transition-colors',
                      config.bgColor
                    )}
                  >
                    <div className="flex items-start space-x-4">
                      <div className="p-2 rounded-lg bg-white shadow-sm">
                        <Icon className={cn('h-5 w-5', config.color)} />
                      </div>
                      
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center justify-between">
                          <h4 className="font-medium text-secondary-900">
                            {eventTypeLabels[event.type] || event.type}
                          </h4>
                          <div className="flex items-center space-x-2">
                            <span className={cn('badge', config.badge)}>
                              {event.severity.toUpperCase()}
                            </span>
                            <div className="flex items-center space-x-1 text-xs text-secondary-500">
                              <ClockIcon className="h-3 w-3" />
                              <span>{formatTimestamp(event.timestamp)}</span>
                            </div>
                          </div>
                        </div>
                        
                        {formatEventData(event.data) && (
                          <div className="mt-2 text-secondary-600">
                            {formatEventData(event.data)}
                          </div>
                        )}
                        
                        <div className="mt-2 text-xs text-secondary-500">
                          <span className="font-medium">URL:</span> {event.url.substring(0, 60)}
                          {event.url.length > 60 && '...'}
                        </div>
                      </div>
                    </div>
                  </motion.div>
                )
              })}
            </div>
          )}
        </div>
      </motion.div>
    </div>
  )
}