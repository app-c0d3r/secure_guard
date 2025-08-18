import { Fragment } from 'react'
import { Menu, Transition } from '@headlessui/react'
import { motion } from 'framer-motion'
import { 
  Bars3Icon, 
  BellIcon, 
  UserCircleIcon,
  ArrowRightOnRectangleIcon,
  CogIcon,
  ShieldCheckIcon
} from '@heroicons/react/24/outline'
import { useAuthStore } from '@/stores/authStore'
import { cn } from '@/lib/utils'
import ThemeSwitcher from '@/components/UI/ThemeSwitcher'

interface HeaderProps {
  onMenuClick: () => void
}

export default function Header({ onMenuClick }: HeaderProps) {
  const { user, logout } = useAuthStore()

  const handleLogout = () => {
    logout()
  }

  return (
    <div className="relative z-10 flex-shrink-0 flex h-16 bg-white dark:bg-gray-800 border-b border-secondary-200 dark:border-gray-700 shadow-sm">
      <button
        type="button"
        className="px-4 border-r border-secondary-200 text-secondary-500 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-primary-500 md:hidden"
        onClick={onMenuClick}
      >
        <span className="sr-only">Open sidebar</span>
        <Bars3Icon className="h-6 w-6" />
      </button>
      
      <div className="flex-1 px-4 flex justify-between items-center">
        {/* Search and breadcrumb would go here */}
        <div className="flex-1 flex">
          <motion.div
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            className="flex items-center space-x-2"
          >
            <ShieldCheckIcon className="h-6 w-6 text-primary-600" />
            <h1 className="text-lg font-semibold text-secondary-900 dark:text-white">
              SecureGuard Dashboard
            </h1>
          </motion.div>
        </div>

        <div className="ml-4 flex items-center md:ml-6 space-x-4">
          {/* Theme Switcher */}
          <ThemeSwitcher size="sm" />
          
          {/* Notifications */}
          <button className="p-1 rounded-full text-secondary-400 dark:text-gray-400 hover:text-secondary-500 dark:hover:text-gray-300 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 relative">
            <span className="sr-only">View notifications</span>
            <BellIcon className="h-6 w-6" />
            {/* Notification badge */}
            <span className="absolute -top-1 -right-1 h-4 w-4 bg-danger-500 rounded-full flex items-center justify-center">
              <span className="text-xs text-white font-medium">3</span>
            </span>
          </button>

          {/* User menu */}
          <Menu as="div" className="ml-3 relative">
            <div>
              <Menu.Button className="max-w-xs bg-white dark:bg-gray-800 flex items-center text-sm rounded-full focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 lg:p-2 lg:rounded-md lg:hover:bg-secondary-50 dark:hover:bg-gray-700">
                <div className="w-8 h-8 bg-primary-100 rounded-full flex items-center justify-center">
                  <span className="text-primary-600 font-semibold text-sm">
                    {user?.username?.charAt(0).toUpperCase()}
                  </span>
                </div>
                <span className="hidden ml-3 text-secondary-700 dark:text-gray-300 text-sm font-medium lg:block">
                  <span className="sr-only">Open user menu for </span>
                  {user?.username}
                </span>
                {/* Role indicator */}
                <div className="hidden lg:flex lg:ml-2 lg:space-x-1">
                  {user?.canAccessSecrets && (
                    <div className="w-2 h-2 bg-warning-500 rounded-full" title="Secret Access" />
                  )}
                  {user?.canAdminSystem && (
                    <div className="w-2 h-2 bg-danger-500 rounded-full" title="System Admin" />
                  )}
                </div>
              </Menu.Button>
            </div>
            
            <Transition
              as={Fragment}
              enter="transition ease-out duration-100"
              enterFrom="transform opacity-0 scale-95"
              enterTo="transform opacity-100 scale-100"
              leave="transition ease-in duration-75"
              leaveFrom="transform opacity-100 scale-100"
              leaveTo="transform opacity-0 scale-95"
            >
              <Menu.Items className="origin-top-right absolute right-0 mt-2 w-48 rounded-md shadow-lg py-1 bg-white dark:bg-gray-800 ring-1 ring-black ring-opacity-5 dark:ring-gray-600 focus:outline-none">
                <div className="px-4 py-3 border-b border-secondary-100 dark:border-gray-600">
                  <p className="text-sm text-secondary-900 dark:text-white font-medium">{user?.username}</p>
                  <p className="text-xs text-secondary-500 dark:text-gray-400">
                    {user?.role.replace('_', ' ').replace(/\b\w/g, l => l.toUpperCase())}
                  </p>
                </div>
                
                <Menu.Item>
                  {({ active }) => (
                    <a
                      href="/profile"
                      className={cn(
                        active ? 'bg-secondary-50 dark:bg-gray-700' : '',
                        'flex items-center px-4 py-2 text-sm text-secondary-700 dark:text-gray-300'
                      )}
                    >
                      <UserCircleIcon className="mr-3 h-4 w-4" />
                      Your Profile
                    </a>
                  )}
                </Menu.Item>
                
                <Menu.Item>
                  {({ active }) => (
                    <a
                      href="/settings"
                      className={cn(
                        active ? 'bg-secondary-50 dark:bg-gray-700' : '',
                        'flex items-center px-4 py-2 text-sm text-secondary-700 dark:text-gray-300'
                      )}
                    >
                      <CogIcon className="mr-3 h-4 w-4" />
                      Settings
                    </a>
                  )}
                </Menu.Item>
                
                <Menu.Item>
                  {({ active }) => (
                    <button
                      onClick={handleLogout}
                      className={cn(
                        active ? 'bg-secondary-50 dark:bg-gray-700' : '',
                        'flex items-center w-full px-4 py-2 text-sm text-secondary-700 dark:text-gray-300'
                      )}
                    >
                      <ArrowRightOnRectangleIcon className="mr-3 h-4 w-4" />
                      Sign out
                    </button>
                  )}
                </Menu.Item>
              </Menu.Items>
            </Transition>
          </Menu>
        </div>
      </div>
    </div>
  )
}