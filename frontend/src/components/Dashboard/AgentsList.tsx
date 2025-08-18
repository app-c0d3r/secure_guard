import { motion } from 'framer-motion'
import { 
  ComputerDesktopIcon,
  EyeIcon,
  ChevronRightIcon
} from '@heroicons/react/24/outline'
import { cn } from '@/lib/utils'

const recentAgents = [
  {
    id: '1',
    name: 'WS-001',
    hostname: 'workstation-001.company.com',
    status: 'online',
    lastSeen: '2024-01-15T10:30:00Z',
    os: 'Windows 11 Pro',
    ip: '192.168.1.101'
  },
  {
    id: '2',
    name: 'SRV-012',
    hostname: 'server-012.company.com',
    status: 'offline',
    lastSeen: '2024-01-15T08:15:00Z',
    os: 'Windows Server 2022',
    ip: '192.168.1.12'
  },
  {
    id: '3',
    name: 'WS-005',
    hostname: 'workstation-005.company.com',
    status: 'warning',
    lastSeen: '2024-01-15T10:25:00Z',
    os: 'Windows 10 Pro',
    ip: '192.168.1.105'
  },
  {
    id: '4',
    name: 'WS-003',
    hostname: 'workstation-003.company.com',
    status: 'online',
    lastSeen: '2024-01-15T10:29:00Z',
    os: 'Windows 11 Pro',
    ip: '192.168.1.103'
  },
  {
    id: '5',
    name: 'LAP-007',
    hostname: 'laptop-007.company.com',
    status: 'online',
    lastSeen: '2024-01-15T10:28:00Z',
    os: 'Windows 11 Pro',
    ip: '192.168.1.107'
  }
]

const getStatusColor = (status: string) => {
  switch (status) {
    case 'online':
      return 'status-online'
    case 'offline':
      return 'status-offline'
    case 'warning':
      return 'status-warning'
    default:
      return 'status-offline'
  }
}

const getStatusLabel = (status: string) => {
  switch (status) {
    case 'online':
      return 'Online'
    case 'offline':
      return 'Offline'
    case 'warning':
      return 'Warnung'
    default:
      return 'Unbekannt'
  }
}

export default function AgentsList() {
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
              Zuletzt aktive Agents
            </h3>
            <p className="text-sm text-secondary-500">
              Neueste Aktivitäten der registrierten Agents
            </p>
          </div>
          <button className="btn-secondary">
            Alle Agents anzeigen
          </button>
        </div>
      </div>
      
      <div className="card-body p-0">
        <div className="divide-y divide-secondary-100">
          {recentAgents.map((agent, index) => (
            <motion.div
              key={agent.id}
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ delay: index * 0.1 }}
              className="p-6 hover:bg-secondary-50 transition-colors duration-200 cursor-pointer group"
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-4">
                  <div className="p-2 bg-secondary-100 rounded-lg group-hover:bg-primary-100 transition-colors">
                    <ComputerDesktopIcon className="h-6 w-6 text-secondary-600 group-hover:text-primary-600 transition-colors" />
                  </div>
                  
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center space-x-3">
                      <h4 className="text-sm font-semibold text-secondary-900">
                        {agent.name}
                      </h4>
                      <div className="flex items-center space-x-1">
                        <div className={cn('w-2 h-2 rounded-full', getStatusColor(agent.status))} />
                        <span className="text-xs text-secondary-500">
                          {getStatusLabel(agent.status)}
                        </span>
                      </div>
                    </div>
                    
                    <p className="text-sm text-secondary-600 truncate">
                      {agent.hostname}
                    </p>
                    
                    <div className="flex items-center space-x-4 mt-1 text-xs text-secondary-500">
                      <span>{agent.os}</span>
                      <span>•</span>
                      <span>{agent.ip}</span>
                      <span>•</span>
                      <span>
                        {new Date(agent.lastSeen).toLocaleString('de-DE', {
                          day: '2-digit',
                          month: '2-digit',
                          hour: '2-digit',
                          minute: '2-digit'
                        })}
                      </span>
                    </div>
                  </div>
                </div>
                
                <div className="flex items-center space-x-2">
                  <button className="p-2 text-secondary-400 hover:text-primary-600 transition-colors">
                    <EyeIcon className="h-4 w-4" />
                  </button>
                  <ChevronRightIcon className="h-4 w-4 text-secondary-400 group-hover:text-primary-600 transition-colors" />
                </div>
              </div>
            </motion.div>
          ))}
        </div>
        
        {/* Footer */}
        <div className="p-6 bg-secondary-50 rounded-b-lg border-t border-secondary-100">
          <div className="text-center">
            <button className="btn-primary">
              Alle {recentAgents.length + 122} Agents verwalten
            </button>
          </div>
        </div>
      </div>
    </motion.div>
  )
}