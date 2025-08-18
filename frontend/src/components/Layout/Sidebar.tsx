import { NavLink, useLocation } from 'react-router-dom'
import { motion } from 'framer-motion'
import { 
  HomeIcon, 
  ComputerDesktopIcon, 
  ShieldCheckIcon, 
  UsersIcon, 
  CreditCardIcon, 
  CogIcon,
  ChartBarIcon,
  KeyIcon,
  ExclamationTriangleIcon,
  ServerIcon
} from '@heroicons/react/24/outline'
import { useAuthStore } from '@/stores/authStore'
import { cn } from '@/lib/utils'

const navigation = [
  { name: 'Dashboard', href: '/dashboard', icon: HomeIcon, roles: ['all'] },
  { name: 'Agents', href: '/agents', icon: ComputerDesktopIcon, roles: ['all'] },
  { 
    name: 'Asset Management', 
    href: '/assets', 
    icon: ServerIcon, 
    roles: ['system_admin', 'admin', 'manager'],
    permission: 'assets.view'
  },
  { name: 'Security', href: '/security', icon: ShieldCheckIcon, roles: ['all'] },
  { name: 'Analytics', href: '/analytics', icon: ChartBarIcon, roles: ['all'] },
  { 
    name: 'Users', 
    href: '/users', 
    icon: UsersIcon, 
    roles: ['system_admin', 'admin', 'manager'],
    permission: 'users.read'
  },
  { 
    name: 'Secrets', 
    href: '/secrets', 
    icon: KeyIcon, 
    roles: ['system_admin', 'security_analyst'],
    permission: 'secrets.read'
  },
  { 
    name: 'Subscriptions', 
    href: '/subscriptions', 
    icon: CreditCardIcon, 
    roles: ['system_admin', 'admin'],
    permission: 'subscriptions.read'
  },
  { 
    name: 'Incidents', 
    href: '/incidents', 
    icon: ExclamationTriangleIcon, 
    roles: ['system_admin', 'security_analyst', 'admin'],
    permission: 'security.incidents'
  },
  { name: 'Settings', href: '/settings', icon: CogIcon, roles: ['all'] },
]

export default function Sidebar() {
  const location = useLocation()
  const { user, hasPermission, hasAnyRole } = useAuthStore()

  const filteredNavigation = navigation.filter(item => {
    // Show item for all roles
    if (item.roles.includes('all')) return true
    
    // Check if user has required role
    if (!hasAnyRole(item.roles)) return false
    
    // Check if user has required permission
    if (item.permission && !hasPermission(item.permission)) return false
    
    return true
  })

  return (
    <div className="flex flex-col w-64 bg-white dark:bg-gray-800 border-r border-secondary-200 dark:border-gray-700">
      {/* Logo */}
      <div className="flex items-center justify-center h-16 px-4 bg-primary-600">
        <motion.div
          initial={{ scale: 0.8, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          className="flex items-center space-x-2"
        >
          <ShieldCheckIcon className="h-8 w-8 text-white" />
          <span className="text-white text-xl font-bold">SecureGuard</span>
        </motion.div>
      </div>

      {/* User info */}
      <div className="p-4 border-b border-secondary-200 dark:border-gray-700">
        <div className="flex items-center space-x-3">
          <div className="w-10 h-10 bg-primary-100 rounded-full flex items-center justify-center">
            <span className="text-primary-600 font-semibold text-sm">
              {user?.username?.charAt(0).toUpperCase()}
            </span>
          </div>
          <div className="flex-1 min-w-0">
            <p className="text-sm font-medium text-secondary-900 dark:text-white truncate">
              {user?.username}
            </p>
            <p className="text-xs text-secondary-500 dark:text-gray-400 truncate">
              {user?.role.replace('_', ' ').replace(/\b\w/g, l => l.toUpperCase())}
            </p>
          </div>
          {(user?.canAccessSecrets || user?.canAdminSystem) && (
            <div className="flex space-x-1">
              {user.canAccessSecrets && (
                <div className="w-2 h-2 bg-warning-500 rounded-full" title="Secret Access" />
              )}
              {user.canAdminSystem && (
                <div className="w-2 h-2 bg-danger-500 rounded-full" title="System Admin" />
              )}
            </div>
          )}
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-2 py-4 space-y-1 overflow-y-auto">
        {filteredNavigation.map((item) => {
          const isActive = location.pathname === item.href
          
          return (
            <NavLink
              key={item.name}
              to={item.href}
              className={cn(
                'group flex items-center px-3 py-2 text-sm font-medium rounded-lg transition-all duration-200',
                isActive
                  ? 'bg-primary-50 text-primary-700 border-l-4 border-primary-600'
                  : 'text-secondary-600 hover:bg-secondary-50 hover:text-secondary-900'
              )}
            >
              <item.icon
                className={cn(
                  'mr-3 h-5 w-5 transition-colors',
                  isActive ? 'text-primary-600' : 'text-secondary-400 group-hover:text-secondary-500'
                )}
              />
              {item.name}
              
              {/* Permission indicator */}
              {item.permission && (
                <div className="ml-auto">
                  <div className="w-1.5 h-1.5 bg-warning-400 rounded-full" title="Requires permission" />
                </div>
              )}
            </NavLink>
          )
        })}
      </nav>

      {/* System status */}
      <div className="p-4 border-t border-secondary-200">
        <div className="flex items-center justify-between text-xs text-secondary-500">
          <span>System Status</span>
          <div className="flex items-center space-x-1">
            <div className="status-online" />
            <span>Online</span>
          </div>
        </div>
      </div>
    </div>
  )
}