import { motion } from 'framer-motion'
import { 
  CpuChipIcon,
  CircleStackIcon,
  CloudIcon,
  WifiIcon,
  CheckCircleIcon,
  ExclamationTriangleIcon
} from '@heroicons/react/24/outline'
import { cn } from '@/lib/utils'

const healthMetrics = [
  {
    name: 'API Server',
    status: 'healthy',
    value: '99.9%',
    description: 'Verfügbarkeit',
    icon: CloudIcon,
    trend: 'stable'
  },
  {
    name: 'Datenbank',
    status: 'healthy',
    value: '2.1ms',
    description: 'Antwortzeit',
    icon: CircleStackIcon,
    trend: 'stable'
  },
  {
    name: 'Agent Verbindungen',
    status: 'warning',
    value: '95.2%',
    description: 'Verbindungsrate',
    icon: WifiIcon,
    trend: 'declining'
  },
  {
    name: 'CPU Auslastung',
    status: 'healthy',
    value: '34%',
    description: 'Server CPU',
    icon: CpuChipIcon,
    trend: 'stable'
  }
]

const statusConfig = {
  healthy: {
    color: 'text-success-600',
    bgColor: 'bg-success-100',
    icon: CheckCircleIcon,
    label: 'Gesund'
  },
  warning: {
    color: 'text-warning-600',
    bgColor: 'bg-warning-100',
    icon: ExclamationTriangleIcon,
    label: 'Warnung'
  },
  error: {
    color: 'text-danger-600',
    bgColor: 'bg-danger-100',
    icon: ExclamationTriangleIcon,
    label: 'Fehler'
  }
}

export default function SystemHealth() {
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
              System Status
            </h3>
            <p className="text-sm text-secondary-500">
              Gesundheit der Systemkomponenten
            </p>
          </div>
          <div className="flex items-center space-x-2">
            <div className="status-online" />
            <span className="text-sm text-secondary-600">Alle Systeme aktiv</span>
          </div>
        </div>
      </div>
      
      <div className="card-body p-0">
        <div className="divide-y divide-secondary-100">
          {healthMetrics.map((metric, index) => {
            const status = statusConfig[metric.status as keyof typeof statusConfig]
            const IconComponent = metric.icon
            const StatusIcon = status.icon
            
            return (
              <motion.div
                key={metric.name}
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: index * 0.1 }}
                className="p-6 hover:bg-secondary-50 transition-colors duration-200"
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-4">
                    <div className="p-2 bg-secondary-100 rounded-lg">
                      <IconComponent className="h-6 w-6 text-secondary-600" />
                    </div>
                    <div>
                      <h4 className="text-sm font-semibold text-secondary-900">
                        {metric.name}
                      </h4>
                      <p className="text-xs text-secondary-500">
                        {metric.description}
                      </p>
                    </div>
                  </div>
                  
                  <div className="flex items-center space-x-3">
                    <div className="text-right">
                      <div className="text-lg font-bold text-secondary-900">
                        {metric.value}
                      </div>
                      <div className={cn(
                        'flex items-center space-x-1 text-xs',
                        status.color
                      )}>
                        <StatusIcon className="h-3 w-3" />
                        <span>{status.label}</span>
                      </div>
                    </div>
                  </div>
                </div>
                
                {/* Progress bar for percentage values */}
                {metric.value.includes('%') && (
                  <div className="mt-4">
                    <div className="w-full bg-secondary-200 rounded-full h-2">
                      <div
                        className={cn(
                          'h-2 rounded-full transition-all duration-300',
                          metric.status === 'healthy' && 'bg-success-500',
                          metric.status === 'warning' && 'bg-warning-500',
                          metric.status === 'error' && 'bg-danger-500'
                        )}
                        style={{
                          width: metric.value
                        }}
                      />
                    </div>
                  </div>
                )}
              </motion.div>
            )
          })}
        </div>
        
        {/* Footer */}
        <div className="p-6 bg-secondary-50 rounded-b-lg border-t border-secondary-100">
          <div className="flex items-center justify-between text-sm">
            <span className="text-secondary-600">
              Letzte Prüfung: vor 30 Sekunden
            </span>
            <button className="btn-secondary text-xs">
              Status aktualisieren
            </button>
          </div>
        </div>
      </div>
    </motion.div>
  )
}