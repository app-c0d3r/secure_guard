import { useState } from 'react'
import { motion } from 'framer-motion'
import { 
  PlusIcon,
  MagnifyingGlassIcon,
  UserIcon,
  PencilIcon,
  TrashIcon,
  ShieldCheckIcon,
  EyeIcon
} from '@heroicons/react/24/outline'
import { cn } from '@/lib/utils'

// Mock data
const users = [
  {
    id: '1',
    email: 'admin@company.com',
    name: 'Max Mustermann',
    role: 'System Administrator',
    roleLevel: 100,
    status: 'active',
    lastLogin: '2024-01-15T10:30:00Z',
    createdAt: '2023-06-15T10:30:00Z',
    permissions: ['all']
  },
  {
    id: '2',
    email: 'security@company.com',
    name: 'Anna Schmidt',
    role: 'Security Manager',
    roleLevel: 80,
    status: 'active',
    lastLogin: '2024-01-15T09:45:00Z',
    createdAt: '2023-08-20T10:30:00Z',
    permissions: ['security_incidents', 'agents_view', 'users_view']
  },
  {
    id: '3',
    email: 'operator@company.com',
    name: 'Tom Weber',
    role: 'Security Operator',
    roleLevel: 60,
    status: 'active',
    lastLogin: '2024-01-15T08:20:00Z',
    createdAt: '2023-09-10T10:30:00Z',
    permissions: ['security_incidents', 'agents_view']
  },
  {
    id: '4',
    email: 'user@company.com',
    name: 'Lisa Müller',
    role: 'User',
    roleLevel: 20,
    status: 'inactive',
    lastLogin: '2024-01-10T15:20:00Z',
    createdAt: '2023-11-05T10:30:00Z',
    permissions: ['dashboard_view']
  }
]

const roles = [
  { name: 'System Administrator', level: 100, color: 'bg-purple-100 text-purple-800' },
  { name: 'Organization Admin', level: 90, color: 'bg-red-100 text-red-800' },
  { name: 'Security Manager', level: 80, color: 'bg-orange-100 text-orange-800' },
  { name: 'Security Analyst', level: 70, color: 'bg-yellow-100 text-yellow-800' },
  { name: 'Security Operator', level: 60, color: 'bg-green-100 text-green-800' },
  { name: 'Team Lead', level: 50, color: 'bg-blue-100 text-blue-800' },
  { name: 'User', level: 20, color: 'bg-gray-100 text-gray-800' },
  { name: 'Viewer', level: 10, color: 'bg-slate-100 text-slate-800' },
  { name: 'Guest', level: 1, color: 'bg-neutral-100 text-neutral-800' }
]

const getStatusColor = (status: string) => {
  switch (status) {
    case 'active':
      return 'bg-success-100 text-success-800'
    case 'inactive':
      return 'bg-secondary-100 text-secondary-800'
    case 'suspended':
      return 'bg-danger-100 text-danger-800'
    default:
      return 'bg-secondary-100 text-secondary-800'
  }
}

const getRoleColor = (roleLevel: number) => {
  const role = roles.find(r => r.level === roleLevel)
  return role?.color || 'bg-secondary-100 text-secondary-800'
}

export default function UserManagement() {
  const [searchTerm, setSearchTerm] = useState('')
  const [selectedRole, setSelectedRole] = useState('all')
  const [selectedStatus, setSelectedStatus] = useState('all')

  const filteredUsers = users.filter(user => {
    const matchesSearch = user.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         user.email.toLowerCase().includes(searchTerm.toLowerCase())
    
    const matchesRole = selectedRole === 'all' || user.role === selectedRole
    const matchesStatus = selectedStatus === 'all' || user.status === selectedStatus
    
    return matchesSearch && matchesRole && matchesStatus
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
          <h1 className="text-3xl font-bold text-secondary-900">Benutzerverwaltung</h1>
          <p className="text-secondary-600 mt-1">
            Verwalten Sie Benutzer, Rollen und Berechtigungen
          </p>
        </div>
        <button className="btn-primary">
          <PlusIcon className="h-5 w-5 mr-2" />
          Benutzer hinzufügen
        </button>
      </motion.div>

      {/* Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="card">
          <div className="card-body flex items-center space-x-3">
            <div className="p-2 bg-primary-100 rounded-lg">
              <UserIcon className="h-6 w-6 text-primary-600" />
            </div>
            <div>
              <p className="text-sm text-secondary-500">Gesamt</p>
              <p className="text-xl font-bold text-secondary-900">{users.length}</p>
            </div>
          </div>
        </div>
        
        <div className="card">
          <div className="card-body flex items-center space-x-3">
            <div className="p-2 bg-success-100 rounded-lg">
              <div className="status-online w-3 h-3" />
            </div>
            <div>
              <p className="text-sm text-secondary-500">Aktiv</p>
              <p className="text-xl font-bold text-secondary-900">
                {users.filter(u => u.status === 'active').length}
              </p>
            </div>
          </div>
        </div>
        
        <div className="card">
          <div className="card-body flex items-center space-x-3">
            <div className="p-2 bg-purple-100 rounded-lg">
              <ShieldCheckIcon className="h-6 w-6 text-purple-600" />
            </div>
            <div>
              <p className="text-sm text-secondary-500">Admins</p>
              <p className="text-xl font-bold text-secondary-900">
                {users.filter(u => u.roleLevel >= 80).length}
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
              <p className="text-sm text-secondary-500">Inaktiv</p>
              <p className="text-xl font-bold text-secondary-900">
                {users.filter(u => u.status === 'inactive').length}
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
            placeholder="Benutzer suchen..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="input pl-10"
          />
        </div>
        
        <select
          value={selectedRole}
          onChange={(e) => setSelectedRole(e.target.value)}
          className="input w-auto"
        >
          <option value="all">Alle Rollen</option>
          {roles.map(role => (
            <option key={role.name} value={role.name}>{role.name}</option>
          ))}
        </select>
        
        <select
          value={selectedStatus}
          onChange={(e) => setSelectedStatus(e.target.value)}
          className="input w-auto"
        >
          <option value="all">Alle Status</option>
          <option value="active">Aktiv</option>
          <option value="inactive">Inaktiv</option>
          <option value="suspended">Gesperrt</option>
        </select>
      </div>

      {/* Users Table */}
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        className="card"
      >
        <div className="card-body p-0">
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead className="bg-secondary-50 border-b border-secondary-100">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-secondary-500 uppercase tracking-wider">
                    Benutzer
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-secondary-500 uppercase tracking-wider">
                    Rolle
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-secondary-500 uppercase tracking-wider">
                    Status
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-secondary-500 uppercase tracking-wider">
                    Letzte Anmeldung
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-secondary-500 uppercase tracking-wider">
                    Erstellt
                  </th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-secondary-500 uppercase tracking-wider">
                    Aktionen
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-secondary-100">
                {filteredUsers.map((user, index) => (
                  <motion.tr
                    key={user.id}
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ delay: index * 0.1 }}
                    className="hover:bg-secondary-50 transition-colors"
                  >
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center">
                        <div className="p-2 bg-secondary-100 rounded-lg mr-3">
                          <UserIcon className="h-5 w-5 text-secondary-600" />
                        </div>
                        <div>
                          <div className="text-sm font-medium text-secondary-900">
                            {user.name}
                          </div>
                          <div className="text-sm text-secondary-500">
                            {user.email}
                          </div>
                        </div>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className={cn('badge', getRoleColor(user.roleLevel))}>
                        {user.role}
                      </span>
                      <div className="text-xs text-secondary-500 mt-1">
                        Level {user.roleLevel}
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className={cn('badge', getStatusColor(user.status))}>
                        {user.status === 'active' ? 'Aktiv' : 
                         user.status === 'inactive' ? 'Inaktiv' : 'Gesperrt'}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-secondary-500">
                      {new Date(user.lastLogin).toLocaleString('de-DE')}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-secondary-500">
                      {new Date(user.createdAt).toLocaleDateString('de-DE')}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                      <div className="flex items-center justify-end space-x-2">
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
                    </td>
                  </motion.tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </motion.div>

      {filteredUsers.length === 0 && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          className="text-center py-12"
        >
          <UserIcon className="mx-auto h-12 w-12 text-secondary-400" />
          <h3 className="mt-2 text-sm font-medium text-secondary-900">Keine Benutzer gefunden</h3>
          <p className="mt-1 text-sm text-secondary-500">
            Versuchen Sie, Ihre Suchkriterien zu ändern.
          </p>
        </motion.div>
      )}
    </div>
  )
}