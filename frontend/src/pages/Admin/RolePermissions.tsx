import { useState } from 'react'
import { motion } from 'framer-motion'
import { 
  ShieldCheckIcon,
  UserGroupIcon,
  KeyIcon,
  EyeIcon,
  PencilIcon,
  CheckCircleIcon,
  XCircleIcon
} from '@heroicons/react/24/outline'
import { cn } from '@/lib/utils'

// Mock data for roles and permissions
const roles = [
  {
    id: '1',
    name: 'System Administrator',
    level: 100,
    description: 'Vollzugriff auf alle Systemfunktionen',
    userCount: 1,
    permissions: ['all'],
    color: 'bg-purple-100 text-purple-800'
  },
  {
    id: '2',
    name: 'Organization Admin',
    level: 90,
    description: 'Verwaltung der Organisation und Benutzer',
    userCount: 2,
    permissions: ['users_manage', 'agents_manage', 'settings_manage', 'security_incidents'],
    color: 'bg-red-100 text-red-800'
  },
  {
    id: '3',
    name: 'Security Manager',
    level: 80,
    description: 'Sicherheitsverwaltung und Incident-Management',
    userCount: 3,
    permissions: ['security_incidents', 'agents_view', 'users_view', 'reports_view'],
    color: 'bg-orange-100 text-orange-800'
  },
  {
    id: '4',
    name: 'Security Analyst',
    level: 70,
    description: 'Sicherheitsanalyse und Bedrohungserkennung',
    userCount: 5,
    permissions: ['security_incidents', 'agents_view', 'reports_view'],
    color: 'bg-yellow-100 text-yellow-800'
  },
  {
    id: '5',
    name: 'Security Operator',
    level: 60,
    description: 'Überwachung und erste Reaktion auf Incidents',
    userCount: 8,
    permissions: ['security_incidents', 'agents_view'],
    color: 'bg-green-100 text-green-800'
  },
  {
    id: '6',
    name: 'Team Lead',
    level: 50,
    description: 'Teamleitung mit erweiterten Berechtigungen',
    userCount: 4,
    permissions: ['dashboard_view', 'reports_view', 'agents_view'],
    color: 'bg-blue-100 text-blue-800'
  },
  {
    id: '7',
    name: 'User',
    level: 20,
    description: 'Standardbenutzer mit eingeschränkten Rechten',
    userCount: 15,
    permissions: ['dashboard_view'],
    color: 'bg-gray-100 text-gray-800'
  },
  {
    id: '8',
    name: 'Viewer',
    level: 10,
    description: 'Nur Lesezugriff auf ausgewählte Bereiche',
    userCount: 3,
    permissions: ['dashboard_view'],
    color: 'bg-slate-100 text-slate-800'
  }
]

const allPermissions = [
  {
    id: 'all',
    name: 'Alle Berechtigungen',
    description: 'Vollzugriff auf alle Systemfunktionen',
    category: 'System'
  },
  {
    id: 'users_manage',
    name: 'Benutzerverwaltung',
    description: 'Benutzer erstellen, bearbeiten und löschen',
    category: 'Administration'
  },
  {
    id: 'users_view',
    name: 'Benutzer anzeigen',
    description: 'Benutzerliste und Details einsehen',
    category: 'Administration'
  },
  {
    id: 'agents_manage',
    name: 'Agent-Verwaltung',
    description: 'Agents hinzufügen, konfigurieren und entfernen',
    category: 'Agents'
  },
  {
    id: 'agents_view',
    name: 'Agents anzeigen',
    description: 'Agent-Status und Informationen einsehen',
    category: 'Agents'
  },
  {
    id: 'security_incidents',
    name: 'Sicherheitsvorfälle',
    description: 'Vorfälle verwalten und bearbeiten',
    category: 'Security'
  },
  {
    id: 'reports_view',
    name: 'Berichte anzeigen',
    description: 'Sicherheitsberichte und Analysen einsehen',
    category: 'Reporting'
  },
  {
    id: 'settings_manage',
    name: 'Systemeinstellungen',
    description: 'Systemkonfiguration verwalten',
    category: 'System'
  },
  {
    id: 'dashboard_view',
    name: 'Dashboard anzeigen',
    description: 'Zugriff auf das Hauptdashboard',
    category: 'General'
  },
  {
    id: 'secrets_access',
    name: 'Geheimnisse verwalten',
    description: 'API-Schlüssel und sensible Daten verwalten',
    category: 'Security'
  }
]

const permissionCategories = [...new Set(allPermissions.map(p => p.category))]

export default function RolePermissions() {
  const [selectedRole, setSelectedRole] = useState(roles[0])
  const [showPermissionDetails, setShowPermissionDetails] = useState(false)

  const hasPermission = (permission: string) => {
    return selectedRole.permissions.includes('all') || selectedRole.permissions.includes(permission)
  }

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <motion.div
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
        className="flex justify-between items-center"
      >
        <div>
          <h1 className="text-3xl font-bold text-secondary-900">Rollen & Berechtigungen</h1>
          <p className="text-secondary-600 mt-1">
            Verwalten Sie Rollen und deren Berechtigungen im System
          </p>
        </div>
        <button 
          onClick={() => setShowPermissionDetails(!showPermissionDetails)}
          className="btn-secondary"
        >
          <KeyIcon className="h-5 w-5 mr-2" />
          {showPermissionDetails ? 'Berechtigungen ausblenden' : 'Alle Berechtigungen anzeigen'}
        </button>
      </motion.div>

      {/* Permission Details Panel */}
      {showPermissionDetails && (
        <motion.div
          initial={{ opacity: 0, height: 0 }}
          animate={{ opacity: 1, height: 'auto' }}
          exit={{ opacity: 0, height: 0 }}
          className="card"
        >
          <div className="card-header">
            <h3 className="text-lg font-semibold text-secondary-900">
              Verfügbare Berechtigungen
            </h3>
            <p className="text-sm text-secondary-500">
              Übersicht aller Systemberechtigungen nach Kategorien
            </p>
          </div>
          <div className="card-body">
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              {permissionCategories.map(category => (
                <div key={category}>
                  <h4 className="font-semibold text-secondary-900 mb-3">{category}</h4>
                  <div className="space-y-2">
                    {allPermissions
                      .filter(p => p.category === category)
                      .map(permission => (
                        <div key={permission.id} className="border border-secondary-200 rounded-lg p-3">
                          <div className="font-medium text-sm text-secondary-900">
                            {permission.name}
                          </div>
                          <div className="text-xs text-secondary-500 mt-1">
                            {permission.description}
                          </div>
                        </div>
                      ))}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </motion.div>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Roles List */}
        <motion.div
          initial={{ opacity: 0, x: -20 }}
          animate={{ opacity: 1, x: 0 }}
          className="card"
        >
          <div className="card-header">
            <h3 className="text-lg font-semibold text-secondary-900">Rollen</h3>
            <p className="text-sm text-secondary-500">
              Klicken Sie auf eine Rolle, um Details anzuzeigen
            </p>
          </div>
          <div className="card-body p-0">
            <div className="divide-y divide-secondary-100">
              {roles.map((role, index) => (
                <motion.div
                  key={role.id}
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: index * 0.1 }}
                  onClick={() => setSelectedRole(role)}
                  className={cn(
                    'p-6 cursor-pointer transition-colors hover:bg-secondary-50',
                    selectedRole.id === role.id && 'bg-primary-50 border-r-4 border-primary-500'
                  )}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-4">
                      <div className="p-2 bg-secondary-100 rounded-lg">
                        <ShieldCheckIcon className="h-6 w-6 text-secondary-600" />
                      </div>
                      <div>
                        <div className="flex items-center space-x-2">
                          <h4 className="font-semibold text-secondary-900">{role.name}</h4>
                          <span className={cn('badge', role.color)}>
                            Level {role.level}
                          </span>
                        </div>
                        <p className="text-sm text-secondary-600 mt-1">
                          {role.description}
                        </p>
                        <div className="flex items-center space-x-4 mt-2 text-xs text-secondary-500">
                          <div className="flex items-center space-x-1">
                            <UserGroupIcon className="h-4 w-4" />
                            <span>{role.userCount} Benutzer</span>
                          </div>
                          <div className="flex items-center space-x-1">
                            <KeyIcon className="h-4 w-4" />
                            <span>
                              {role.permissions.includes('all') 
                                ? 'Alle Berechtigungen' 
                                : `${role.permissions.length} Berechtigungen`}
                            </span>
                          </div>
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
                    </div>
                  </div>
                </motion.div>
              ))}
            </div>
          </div>
        </motion.div>

        {/* Role Details */}
        <motion.div
          initial={{ opacity: 0, x: 20 }}
          animate={{ opacity: 1, x: 0 }}
          className="card"
        >
          <div className="card-header">
            <div className="flex items-center space-x-3">
              <div className="p-2 bg-secondary-100 rounded-lg">
                <ShieldCheckIcon className="h-6 w-6 text-secondary-600" />
              </div>
              <div>
                <h3 className="text-lg font-semibold text-secondary-900">
                  {selectedRole.name}
                </h3>
                <p className="text-sm text-secondary-500">
                  Berechtigungen und Details
                </p>
              </div>
            </div>
          </div>
          <div className="card-body">
            <div className="space-y-6">
              {/* Role Info */}
              <div>
                <div className="flex items-center space-x-2 mb-2">
                  <span className={cn('badge', selectedRole.color)}>
                    Level {selectedRole.level}
                  </span>
                  <span className="badge bg-secondary-100 text-secondary-800">
                    {selectedRole.userCount} Benutzer
                  </span>
                </div>
                <p className="text-secondary-600">{selectedRole.description}</p>
              </div>

              {/* Permissions */}
              <div>
                <h4 className="font-semibold text-secondary-900 mb-4">Berechtigungen</h4>
                
                {selectedRole.permissions.includes('all') ? (
                  <div className="bg-purple-50 border border-purple-200 rounded-lg p-4">
                    <div className="flex items-center space-x-2">
                      <CheckCircleIcon className="h-5 w-5 text-purple-600" />
                      <span className="font-medium text-purple-800">
                        Alle Systemberechtigungen
                      </span>
                    </div>
                    <p className="text-sm text-purple-600 mt-1">
                      Diese Rolle hat Vollzugriff auf alle Funktionen des Systems.
                    </p>
                  </div>
                ) : (
                  <div className="space-y-4">
                    {permissionCategories.map(category => {
                      const categoryPermissions = allPermissions.filter(
                        p => p.category === category
                      )
                      
                      return (
                        <div key={category}>
                          <h5 className="font-medium text-secondary-800 mb-2">{category}</h5>
                          <div className="space-y-2">
                            {categoryPermissions.map(permission => (
                              <div 
                                key={permission.id}
                                className="flex items-center justify-between p-3 bg-secondary-50 rounded-lg"
                              >
                                <div>
                                  <div className="font-medium text-sm text-secondary-900">
                                    {permission.name}
                                  </div>
                                  <div className="text-xs text-secondary-500">
                                    {permission.description}
                                  </div>
                                </div>
                                <div>
                                  {hasPermission(permission.id) ? (
                                    <CheckCircleIcon className="h-5 w-5 text-success-600" />
                                  ) : (
                                    <XCircleIcon className="h-5 w-5 text-secondary-400" />
                                  )}
                                </div>
                              </div>
                            ))}
                          </div>
                        </div>
                      )
                    })}
                  </div>
                )}
              </div>

              {/* Actions */}
              <div className="flex items-center space-x-3 pt-4 border-t border-secondary-100">
                <button className="btn-primary">
                  Rolle bearbeiten
                </button>
                <button className="btn-secondary">
                  Benutzer zuweisen
                </button>
              </div>
            </div>
          </div>
        </motion.div>
      </div>
    </div>
  )
}