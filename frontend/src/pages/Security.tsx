import { useState } from 'react'
import { motion } from 'framer-motion'
import { 
  ShieldCheckIcon,
  ExclamationTriangleIcon,
  EyeIcon,
  FunnelIcon,
  MagnifyingGlassIcon,
  ComputerDesktopIcon
} from '@heroicons/react/24/outline'
import { cn } from '@/lib/utils'
import { useAuthStore } from '@/stores/authStore'
import SecurityDashboard from '@/components/Security/SecurityDashboard'

// Mock security incidents data
const incidents = [
  {
    id: 1,
    title: 'Suspicious Process Detected',
    description: 'Unbekannter Prozess "malware.exe" auf Agent WS-001 erkannt',
    severity: 'critical',
    timestamp: '2024-01-15T10:30:00Z',
    agent: 'WS-001',
    status: 'investigating',
    assignedTo: 'Anna Schmidt',
    category: 'Malware Detection'
  },
  {
    id: 2,
    title: 'Agent Communication Lost',
    description: 'Verbindung zu Agent SRV-012 seit 2 Stunden unterbrochen',
    severity: 'high',
    timestamp: '2024-01-15T09:45:00Z',
    agent: 'SRV-012',
    status: 'open',
    assignedTo: 'Tom Weber',
    category: 'Connectivity'
  },
  {
    id: 3,
    title: 'File Integrity Violation',
    description: 'Kritische Systemdatei system32\\kernel.dll wurde modifiziert',
    severity: 'critical',
    timestamp: '2024-01-15T08:20:00Z',
    agent: 'WS-005',
    status: 'resolved',
    assignedTo: 'Lisa Müller',
    category: 'File Integrity'
  },
  {
    id: 4,
    title: 'Unusual Network Activity',
    description: 'Auffällige Verbindungen zu externen IPs erkannt',
    severity: 'medium',
    timestamp: '2024-01-15T07:15:00Z',
    agent: 'WS-003',
    status: 'monitoring',
    assignedTo: 'Anna Schmidt',
    category: 'Network Activity'
  },
  {
    id: 5,
    title: 'Failed Login Attempts',
    description: 'Mehrere fehlgeschlagene Anmeldeversuche erkannt',
    severity: 'low',
    timestamp: '2024-01-15T06:30:00Z',
    agent: 'SRV-001',
    status: 'closed',
    assignedTo: 'Tom Weber',
    category: 'Authentication'
  }
]

const severityConfig = {
  critical: {
    color: 'bg-danger-100 text-danger-800',
    bgColor: 'bg-danger-50',
    label: 'Kritisch'
  },
  high: {
    color: 'bg-warning-100 text-warning-800',
    bgColor: 'bg-warning-50',
    label: 'Hoch'
  },
  medium: {
    color: 'bg-primary-100 text-primary-800',
    bgColor: 'bg-primary-50',
    label: 'Mittel'
  },
  low: {
    color: 'bg-secondary-100 text-secondary-800',
    bgColor: 'bg-secondary-50',
    label: 'Niedrig'
  }
}

const statusConfig = {
  open: { color: 'bg-danger-100 text-danger-800', label: 'Offen' },
  investigating: { color: 'bg-warning-100 text-warning-800', label: 'Wird untersucht' },
  monitoring: { color: 'bg-primary-100 text-primary-800', label: 'Überwachung' },
  resolved: { color: 'bg-success-100 text-success-800', label: 'Gelöst' },
  closed: { color: 'bg-secondary-100 text-secondary-800', label: 'Geschlossen' }
}

export default function Security() {
  const { user } = useAuthStore()
  const [activeTab, setActiveTab] = useState('incidents')
  const [searchTerm, setSearchTerm] = useState('')
  const [selectedSeverity, setSelectedSeverity] = useState('all')
  const [selectedStatus, setSelectedStatus] = useState('all')
  
  const isAdmin = user?.role === 'admin' || user?.role === 'System Administrator'

  const filteredIncidents = incidents.filter(incident => {
    const matchesSearch = incident.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         incident.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         incident.agent.toLowerCase().includes(searchTerm.toLowerCase())
    
    const matchesSeverity = selectedSeverity === 'all' || incident.severity === selectedSeverity
    const matchesStatus = selectedStatus === 'all' || incident.status === selectedStatus
    
    return matchesSearch && matchesSeverity && matchesStatus
  })

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <motion.div
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
        className="flex justify-between items-center"
      >
        <div>
          <h1 className="text-3xl font-bold text-secondary-900">Sicherheit</h1>
          <p className="text-secondary-600 mt-1">
            Überwachen und verwalten Sie Sicherheitsereignisse und Bedrohungen
          </p>
        </div>
        
        {isAdmin && (
          <div className="flex items-center space-x-1 bg-white rounded-lg p-1 border border-secondary-200">
            <button
              onClick={() => setActiveTab('incidents')}
              className={cn(
                'px-4 py-2 rounded-md text-sm font-medium transition-colors',
                activeTab === 'incidents'
                  ? 'bg-primary-100 text-primary-700'
                  : 'text-secondary-600 hover:text-secondary-900'
              )}
            >
              Vorfälle
            </button>
            <button
              onClick={() => setActiveTab('monitoring')}
              className={cn(
                'px-4 py-2 rounded-md text-sm font-medium transition-colors',
                activeTab === 'monitoring'
                  ? 'bg-primary-100 text-primary-700'
                  : 'text-secondary-600 hover:text-secondary-900'
              )}
            >
              Überwachung
            </button>
          </div>
        )}
      </motion.div>

      {/* Stats */}
      <div className="grid grid-cols-1 md:grid-cols-5 gap-4">
        <div className="card">
          <div className="card-body flex items-center space-x-3">
            <div className="p-2 bg-danger-100 rounded-lg">
              <ExclamationTriangleIcon className="h-6 w-6 text-danger-600" />
            </div>
            <div>
              <p className="text-sm text-secondary-500">Kritisch</p>
              <p className="text-xl font-bold text-secondary-900">
                {incidents.filter(i => i.severity === 'critical').length}
              </p>
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
              <p className="text-xl font-bold text-secondary-900">
                {incidents.filter(i => i.severity === 'high').length}
              </p>
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
              <p className="text-xl font-bold text-secondary-900">
                {incidents.filter(i => i.severity === 'medium').length}
              </p>
            </div>
          </div>
        </div>
        
        <div className="card">
          <div className="card-body flex items-center space-x-3">
            <div className="p-2 bg-secondary-100 rounded-lg">
              <ExclamationTriangleIcon className="h-6 w-6 text-secondary-600" />
            </div>
            <div>
              <p className="text-sm text-secondary-500">Niedrig</p>
              <p className="text-xl font-bold text-secondary-900">
                {incidents.filter(i => i.severity === 'low').length}
              </p>
            </div>
          </div>
        </div>
        
        <div className="card">
          <div className="card-body flex items-center space-x-3">
            <div className="p-2 bg-success-100 rounded-lg">
              <ShieldCheckIcon className="h-6 w-6 text-success-600" />
            </div>
            <div>
              <p className="text-sm text-secondary-500">Gelöst</p>
              <p className="text-xl font-bold text-secondary-900">
                {incidents.filter(i => i.status === 'resolved').length}
              </p>
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
            placeholder="Vorfälle suchen..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="input pl-10"
          />
        </div>
        
        <select
          value={selectedSeverity}
          onChange={(e) => setSelectedSeverity(e.target.value)}
          className="input w-auto"
        >
          <option value="all">Alle Schweregrade</option>
          <option value="critical">Kritisch</option>
          <option value="high">Hoch</option>
          <option value="medium">Mittel</option>
          <option value="low">Niedrig</option>
        </select>
        
        <select
          value={selectedStatus}
          onChange={(e) => setSelectedStatus(e.target.value)}
          className="input w-auto"
        >
          <option value="all">Alle Status</option>
          <option value="open">Offen</option>
          <option value="investigating">Wird untersucht</option>
          <option value="monitoring">Überwachung</option>
          <option value="resolved">Gelöst</option>
          <option value="closed">Geschlossen</option>
        </select>
      </div>

      {/* Incidents List */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        className="card"
      >
        <div className="card-body p-0">
          <div className="divide-y divide-secondary-100">
            {filteredIncidents.map((incident, index) => {
              const severity = severityConfig[incident.severity as keyof typeof severityConfig]
              const status = statusConfig[incident.status as keyof typeof statusConfig]
              
              return (
                <motion.div
                  key={incident.id}
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: index * 0.1 }}
                  className="p-6 hover:bg-secondary-50 transition-colors duration-200 cursor-pointer"
                >
                  <div className="flex items-start space-x-4">
                    <div className={cn('p-3 rounded-lg', severity.bgColor)}>
                      <ExclamationTriangleIcon className="h-6 w-6 text-secondary-600" />
                    </div>
                    
                    <div className="flex-1 min-w-0">
                      <div className="flex items-start justify-between">
                        <div className="flex-1">
                          <div className="flex items-center space-x-3 mb-2">
                            <h3 className="text-lg font-semibold text-secondary-900">
                              {incident.title}
                            </h3>
                            <span className={cn('badge', severity.color)}>
                              {severity.label}
                            </span>
                            <span className={cn('badge', status.color)}>
                              {status.label}
                            </span>
                          </div>
                          
                          <p className="text-secondary-600 mb-3">
                            {incident.description}
                          </p>
                          
                          <div className="flex items-center space-x-6 text-sm text-secondary-500">
                            <div>
                              <span className="font-medium">Agent:</span> {incident.agent}
                            </div>
                            <div>
                              <span className="font-medium">Kategorie:</span> {incident.category}
                            </div>
                            <div>
                              <span className="font-medium">Zugewiesen an:</span> {incident.assignedTo}
                            </div>
                            <div>
                              <span className="font-medium">Zeit:</span> {new Date(incident.timestamp).toLocaleString('de-DE')}
                            </div>
                          </div>
                        </div>
                        
                        <div className="flex items-center space-x-2 ml-4">
                          <button className="btn-icon">
                            <EyeIcon className="h-4 w-4" />
                          </button>
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

      {/* Security Monitoring Dashboard for Admins */}
      {isAdmin && activeTab === 'monitoring' && (
        <SecurityDashboard />
      )}

      {/* Show incidents tab content only when incidents tab is active or user is not admin */}
      {(!isAdmin || activeTab === 'incidents') && filteredIncidents.length === 0 && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          className="text-center py-12"
        >
          <ShieldCheckIcon className="mx-auto h-12 w-12 text-secondary-400" />
          <h3 className="mt-2 text-sm font-medium text-secondary-900">Keine Vorfälle gefunden</h3>
          <p className="mt-1 text-sm text-secondary-500">
            Versuchen Sie, Ihre Suchkriterien zu ändern.
          </p>
        </motion.div>
      )}
    </div>
  )
}