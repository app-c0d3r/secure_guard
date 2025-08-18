import { motion } from 'framer-motion'
import { 
  ComputerDesktopIcon,
  EyeIcon,
  PencilIcon,
  TrashIcon,
  ExclamationTriangleIcon,
  ClockIcon,
  CpuChipIcon,
  SignalIcon
} from '@heroicons/react/24/outline'
import { cn } from '@/lib/utils'

interface Agent {
  id: string
  name: string
  hostname: string
  status: 'online' | 'offline' | 'warning'
  lastSeen: string
  version: string
  os: string
  ip: string
  threats: number
  uptime: string
  subscription: string
}

interface AgentCardProps {
  agent: Agent
  viewMode: 'grid' | 'list'
}

const getStatusConfig = (status: string) => {
  switch (status) {
    case 'online':
      return {
        color: 'status-online',
        bgColor: 'bg-success-100',
        textColor: 'text-success-800',
        label: 'Online'
      }
    case 'offline':
      return {
        color: 'status-offline',
        bgColor: 'bg-secondary-100',
        textColor: 'text-secondary-800',
        label: 'Offline'
      }
    case 'warning':
      return {
        color: 'status-warning',
        bgColor: 'bg-warning-100',
        textColor: 'text-warning-800',
        label: 'Warnung'
      }
    default:
      return {
        color: 'status-offline',
        bgColor: 'bg-secondary-100',
        textColor: 'text-secondary-800',
        label: 'Unbekannt'
      }
  }
}

const getSubscriptionColor = (subscription: string) => {
  switch (subscription.toLowerCase()) {
    case 'free':
      return 'bg-secondary-100 text-secondary-800'
    case 'starter':
      return 'bg-primary-100 text-primary-800'
    case 'professional':
      return 'bg-success-100 text-success-800'
    case 'enterprise':
      return 'bg-purple-100 text-purple-800'
    default:
      return 'bg-secondary-100 text-secondary-800'
  }
}

export default function AgentCard({ agent, viewMode }: AgentCardProps) {
  const statusConfig = getStatusConfig(agent.status)

  if (viewMode === 'list') {
    return (
      <motion.div
        whileHover={{ scale: 1.01 }}
        className="card hover:shadow-glow transition-all duration-300"
      >
        <div className="card-body">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-4">
              <div className={cn('p-3 rounded-lg', statusConfig.bgColor)}>
                <ComputerDesktopIcon className="h-6 w-6 text-secondary-600" />
              </div>
              
              <div className="flex-1">
                <div className="flex items-center space-x-3">
                  <h3 className="text-lg font-semibold text-secondary-900">
                    {agent.name}
                  </h3>
                  <div className="flex items-center space-x-1">
                    <div className={cn('w-2 h-2 rounded-full', statusConfig.color)} />
                    <span className="text-sm text-secondary-500">
                      {statusConfig.label}
                    </span>
                  </div>
                  <span className={cn('badge', getSubscriptionColor(agent.subscription))}>
                    {agent.subscription}
                  </span>
                </div>
                
                <p className="text-secondary-600 mt-1">
                  {agent.hostname}
                </p>
                
                <div className="flex items-center space-x-6 mt-2 text-sm text-secondary-500">
                  <div className="flex items-center space-x-1">
                    <CpuChipIcon className="h-4 w-4" />
                    <span>{agent.os}</span>
                  </div>
                  <div className="flex items-center space-x-1">
                    <SignalIcon className="h-4 w-4" />
                    <span>{agent.ip}</span>
                  </div>
                  <div className="flex items-center space-x-1">
                    <ClockIcon className="h-4 w-4" />
                    <span>Uptime: {agent.uptime}</span>
                  </div>
                  {agent.threats > 0 && (
                    <div className="flex items-center space-x-1 text-warning-600">
                      <ExclamationTriangleIcon className="h-4 w-4" />
                      <span>{agent.threats} Bedrohungen</span>
                    </div>
                  )}
                </div>
              </div>
            </div>
            
            <div className="flex items-center space-x-2">
              <button className="btn-icon">
                <EyeIcon className="h-4 w-4" />
              </button>
              <button className="btn-icon">
                <PencilIcon className="h-4 w-4" />
              </button>
              <button className="btn-icon text-danger-600 hover:bg-danger-100">
                <TrashIcon className="h-4 w-4" />
              </button>
            </div>
          </div>
        </div>
      </motion.div>
    )
  }

  return (
    <motion.div
      whileHover={{ scale: 1.02, y: -5 }}
      className="card hover:shadow-glow transition-all duration-300"
    >
      <div className="card-body">
        <div className="flex items-start justify-between mb-4">
          <div className={cn('p-3 rounded-lg', statusConfig.bgColor)}>
            <ComputerDesktopIcon className="h-6 w-6 text-secondary-600" />
          </div>
          
          <div className="flex items-center space-x-1">
            <div className={cn('w-2 h-2 rounded-full', statusConfig.color)} />
            <span className="text-sm text-secondary-500">
              {statusConfig.label}
            </span>
          </div>
        </div>
        
        <div className="mb-4">
          <h3 className="text-lg font-semibold text-secondary-900 mb-1">
            {agent.name}
          </h3>
          <p className="text-sm text-secondary-600 truncate">
            {agent.hostname}
          </p>
        </div>
        
        <div className="space-y-2 mb-4">
          <div className="flex items-center justify-between text-sm">
            <span className="text-secondary-500">OS:</span>
            <span className="text-secondary-900">{agent.os}</span>
          </div>
          <div className="flex items-center justify-between text-sm">
            <span className="text-secondary-500">IP:</span>
            <span className="text-secondary-900">{agent.ip}</span>
          </div>
          <div className="flex items-center justify-between text-sm">
            <span className="text-secondary-500">Version:</span>
            <span className="text-secondary-900">{agent.version}</span>
          </div>
          <div className="flex items-center justify-between text-sm">
            <span className="text-secondary-500">Uptime:</span>
            <span className="text-secondary-900">{agent.uptime}</span>
          </div>
        </div>
        
        <div className="flex items-center justify-between mb-4">
          <span className={cn('badge', getSubscriptionColor(agent.subscription))}>
            {agent.subscription}
          </span>
          
          {agent.threats > 0 && (
            <span className="badge bg-warning-100 text-warning-800">
              {agent.threats} Bedrohungen
            </span>
          )}
        </div>
        
        <div className="text-xs text-secondary-500 mb-4">
          Letzte Aktivität: {new Date(agent.lastSeen).toLocaleString('de-DE')}
        </div>
        
        <div className="flex items-center justify-between pt-4 border-t border-secondary-100">
          <button className="btn-icon">
            <EyeIcon className="h-4 w-4" />
          </button>
          <button className="btn-icon">
            <PencilIcon className="h-4 w-4" />
          </button>
          <button className="btn-icon text-danger-600 hover:bg-danger-100">
            <TrashIcon className="h-4 w-4" />
          </button>
        </div>
      </div>
    </motion.div>
  )
}