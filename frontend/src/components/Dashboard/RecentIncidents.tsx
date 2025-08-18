import { motion } from 'framer-motion'
import { 
  ExclamationTriangleIcon, 
  ShieldExclamationIcon,
  ComputerDesktopIcon,
  ClockIcon
} from '@heroicons/react/24/outline'
import { cn } from '@/lib/utils'

const incidents = [
  {
    id: 1,
    title: 'Suspicious Process Detected',
    description: 'Unbekannter Prozess auf Agent WS-001 erkannt',
    severity: 'high',
    timestamp: '2024-01-15T10:30:00Z',
    agent: 'WS-001',
    status: 'investigating'
  },
  {
    id: 2,
    title: 'Agent Communication Lost',
    description: 'Verbindung zu Agent SRV-012 unterbrochen',
    severity: 'medium',
    timestamp: '2024-01-15T09:45:00Z',
    agent: 'SRV-012',
    status: 'resolved'
  },
  {
    id: 3,
    title: 'File Integrity Violation',
    description: 'Kritische Systemdatei wurde modifiziert',
    severity: 'critical',
    timestamp: '2024-01-15T08:20:00Z',
    agent: 'WS-005',
    status: 'open'
  },
  {
    id: 4,
    title: 'Unusual Network Activity',
    description: 'Auffällige Netzwerkverbindungen erkannt',
    severity: 'low',
    timestamp: '2024-01-15T07:15:00Z',
    agent: 'WS-003',
    status: 'monitoring'
  },
]

const severityConfig = {
  critical: {
    color: 'bg-danger-100 text-danger-800',
    icon: ShieldExclamationIcon,
    iconColor: 'text-danger-600'
  },
  high: {
    color: 'bg-warning-100 text-warning-800',
    icon: ExclamationTriangleIcon,
    iconColor: 'text-warning-600'
  },
  medium: {
    color: 'bg-primary-100 text-primary-800',
    icon: ExclamationTriangleIcon,
    iconColor: 'text-primary-600'
  },
  low: {
    color: 'bg-secondary-100 text-secondary-800',
    icon: ExclamationTriangleIcon,
    iconColor: 'text-secondary-600'
  }
}

const statusConfig = {
  open: { color: 'bg-danger-100 text-danger-800', label: 'Offen' },
  investigating: { color: 'bg-warning-100 text-warning-800', label: 'Untersuchen' },
  monitoring: { color: 'bg-primary-100 text-primary-800', label: 'Überwachen' },
  resolved: { color: 'bg-success-100 text-success-800', label: 'Gelöst' }
}

export default function RecentIncidents() {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      className="card"
    >
      <div className="card-header">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-lg font-semibold text-secondary-900">
              Aktuelle Sicherheitsvorfälle
            </h3>
            <p className="text-sm text-secondary-500">
              Neueste Ereignisse und Bedrohungen
            </p>
          </div>
          <button className="btn-secondary">
            Alle anzeigen
          </button>
        </div>
      </div>
      
      <div className="card-body p-0">
        <div className="divide-y divide-secondary-100">
          {incidents.map((incident, index) => {
            const severity = severityConfig[incident.severity as keyof typeof severityConfig]
            const status = statusConfig[incident.status as keyof typeof statusConfig]
            
            return (
              <motion.div
                key={incident.id}
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: index * 0.1 }}
                className="p-6 hover:bg-secondary-50 transition-colors duration-200 cursor-pointer"
              >
                <div className="flex items-start space-x-4">
                  <div className={cn(
                    'p-2 rounded-lg',
                    severity.color.replace('text-', 'bg-').replace('-800', '-100')
                  )}>
                    <severity.icon className={cn('h-5 w-5', severity.iconColor)} />
                  </div>
                  
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center justify-between">
                      <h4 className="text-sm font-semibold text-secondary-900 truncate">
                        {incident.title}
                      </h4>
                      <div className="flex items-center space-x-2 ml-4">
                        <span className={cn('badge', severity.color)}>
                          {incident.severity.toUpperCase()}
                        </span>
                        <span className={cn('badge', status.color)}>
                          {status.label}
                        </span>
                      </div>
                    </div>
                    
                    <p className="text-sm text-secondary-600 mt-1">
                      {incident.description}
                    </p>
                    
                    <div className="flex items-center space-x-4 mt-3 text-xs text-secondary-500">
                      <div className="flex items-center space-x-1">
                        <ComputerDesktopIcon className="h-4 w-4" />
                        <span>Agent: {incident.agent}</span>
                      </div>
                      <div className="flex items-center space-x-1">
                        <ClockIcon className="h-4 w-4" />
                        <span>
                          {new Date(incident.timestamp).toLocaleString('de-DE')}
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
              </motion.div>
            )
          })}
        </div>
      </div>
    </motion.div>
  )
}