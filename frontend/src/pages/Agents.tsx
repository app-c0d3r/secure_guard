import { useState } from 'react'
import { motion } from 'framer-motion'
import { 
  PlusIcon, 
  FunnelIcon, 
  MagnifyingGlassIcon,
  ComputerDesktopIcon
} from '@heroicons/react/24/outline'
import AgentCard from '@/components/Agents/AgentCard'
import AgentFilters from '@/components/Agents/AgentFilters'
import AddAgentModal from '@/components/Agents/AddAgentModal'
import { cn } from '@/lib/utils'

// Mock data
const agents = [
  {
    id: '1',
    name: 'WS-001',
    hostname: 'workstation-001.company.com',
    status: 'online',
    lastSeen: '2024-01-15T10:30:00Z',
    version: '1.2.3',
    os: 'Windows 11 Pro',
    ip: '192.168.1.101',
    threats: 0,
    uptime: '7 Tage, 3 Stunden',
    subscription: 'Professional'
  },
  {
    id: '2',
    name: 'SRV-012',
    hostname: 'server-012.company.com',
    status: 'offline',
    lastSeen: '2024-01-15T08:15:00Z',
    version: '1.2.3',
    os: 'Windows Server 2022',
    ip: '192.168.1.12',
    threats: 2,
    uptime: '0',
    subscription: 'Enterprise'
  },
  {
    id: '3',
    name: 'WS-005',
    hostname: 'workstation-005.company.com',
    status: 'warning',
    lastSeen: '2024-01-15T10:25:00Z',
    version: '1.2.2',
    os: 'Windows 10 Pro',
    ip: '192.168.1.105',
    threats: 1,
    uptime: '2 Tage, 14 Stunden',
    subscription: 'Starter'
  },
  {
    id: '4',
    name: 'WS-003',
    hostname: 'workstation-003.company.com',
    status: 'online',
    lastSeen: '2024-01-15T10:29:00Z',
    version: '1.2.3',
    os: 'Windows 11 Pro',
    ip: '192.168.1.103',
    threats: 0,
    uptime: '15 Tage, 8 Stunden',
    subscription: 'Free'
  }
]

export default function Agents() {
  const [searchTerm, setSearchTerm] = useState('')
  const [selectedFilter, setSelectedFilter] = useState('all')
  const [showAddModal, setShowAddModal] = useState(false)
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid')

  const filteredAgents = agents.filter(agent => {
    const matchesSearch = agent.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         agent.hostname.toLowerCase().includes(searchTerm.toLowerCase())
    
    const matchesFilter = selectedFilter === 'all' || agent.status === selectedFilter
    
    return matchesSearch && matchesFilter
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
          <h1 className="text-3xl font-bold text-secondary-900">Agents</h1>
          <p className="text-secondary-600 mt-1">
            Verwalten und überwachen Sie alle SecureGuard Agents
          </p>
        </div>
        <button 
          onClick={() => setShowAddModal(true)}
          className="btn-primary"
        >
          <PlusIcon className="h-5 w-5 mr-2" />
          Agent hinzufügen
        </button>
      </motion.div>

      {/* Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="card">
          <div className="card-body flex items-center space-x-3">
            <div className="p-2 bg-success-100 rounded-lg">
              <div className="status-online w-3 h-3" />
            </div>
            <div>
              <p className="text-sm text-secondary-500">Online</p>
              <p className="text-xl font-bold text-secondary-900">
                {agents.filter(a => a.status === 'online').length}
              </p>
            </div>
          </div>
        </div>
        
        <div className="card">
          <div className="card-body flex items-center space-x-3">
            <div className="p-2 bg-secondary-100 rounded-lg">
              <div className="status-offline w-3 h-3" />
            </div>
            <div>
              <p className="text-sm text-secondary-500">Offline</p>
              <p className="text-xl font-bold text-secondary-900">
                {agents.filter(a => a.status === 'offline').length}
              </p>
            </div>
          </div>
        </div>
        
        <div className="card">
          <div className="card-body flex items-center space-x-3">
            <div className="p-2 bg-warning-100 rounded-lg">
              <div className="status-warning w-3 h-3" />
            </div>
            <div>
              <p className="text-sm text-secondary-500">Warnings</p>
              <p className="text-xl font-bold text-secondary-900">
                {agents.filter(a => a.status === 'warning').length}
              </p>
            </div>
          </div>
        </div>
        
        <div className="card">
          <div className="card-body flex items-center space-x-3">
            <div className="p-2 bg-primary-100 rounded-lg">
              <ComputerDesktopIcon className="h-5 w-5 text-primary-600" />
            </div>
            <div>
              <p className="text-sm text-secondary-500">Gesamt</p>
              <p className="text-xl font-bold text-secondary-900">{agents.length}</p>
            </div>
          </div>
        </div>
      </div>

      {/* Search and Filters */}
      <div className="flex flex-col sm:flex-row gap-4">
        <div className="flex-1 relative">
          <MagnifyingGlassIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 h-5 w-5 text-secondary-400" />
          <input
            type="text"
            placeholder="Agents suchen..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="input pl-10"
          />
        </div>
        
        <AgentFilters 
          selectedFilter={selectedFilter}
          onFilterChange={setSelectedFilter}
        />
        
        <div className="flex items-center space-x-2">
          <button
            onClick={() => setViewMode('grid')}
            className={cn(
              'p-2 rounded-lg transition-colors',
              viewMode === 'grid' 
                ? 'bg-primary-100 text-primary-600' 
                : 'text-secondary-500 hover:text-secondary-700'
            )}
          >
            <svg className="h-5 w-5" fill="currentColor" viewBox="0 0 20 20">
              <path d="M5 3a2 2 0 00-2 2v2a2 2 0 002 2h2a2 2 0 002-2V5a2 2 0 00-2-2H5zM5 11a2 2 0 00-2 2v2a2 2 0 002 2h2a2 2 0 002-2v-2a2 2 0 00-2-2H5zM11 5a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V5zM11 13a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
            </svg>
          </button>
          <button
            onClick={() => setViewMode('list')}
            className={cn(
              'p-2 rounded-lg transition-colors',
              viewMode === 'list' 
                ? 'bg-primary-100 text-primary-600' 
                : 'text-secondary-500 hover:text-secondary-700'
            )}
          >
            <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 10h16M4 14h16M4 18h16" />
            </svg>
          </button>
        </div>
      </div>

      {/* Agents Grid/List */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        className={cn(
          viewMode === 'grid' 
            ? 'grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6'
            : 'space-y-4'
        )}
      >
        {filteredAgents.map((agent, index) => (
          <motion.div
            key={agent.id}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: index * 0.1 }}
          >
            <AgentCard agent={agent} viewMode={viewMode} />
          </motion.div>
        ))}
      </motion.div>

      {filteredAgents.length === 0 && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          className="text-center py-12"
        >
          <ComputerDesktopIcon className="mx-auto h-12 w-12 text-secondary-400" />
          <h3 className="mt-2 text-sm font-medium text-secondary-900">Keine Agents gefunden</h3>
          <p className="mt-1 text-sm text-secondary-500">
            Versuchen Sie, Ihre Suchkriterien zu ändern oder fügen Sie einen neuen Agent hinzu.
          </p>
        </motion.div>
      )}

      {/* Add Agent Modal */}
      <AddAgentModal 
        isOpen={showAddModal}
        onClose={() => setShowAddModal(false)}
      />
    </div>
  )
}