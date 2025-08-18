import { Fragment } from 'react'
import { Menu, Transition } from '@headlessui/react'
import { FunnelIcon, ChevronDownIcon } from '@heroicons/react/24/outline'
import { cn } from '@/lib/utils'

interface AgentFiltersProps {
  selectedFilter: string
  onFilterChange: (filter: string) => void
}

const filterOptions = [
  { value: 'all', label: 'Alle Status', count: null },
  { value: 'online', label: 'Online', count: 127, color: 'text-success-600' },
  { value: 'offline', label: 'Offline', count: 8, color: 'text-secondary-600' },
  { value: 'warning', label: 'Warnung', count: 5, color: 'text-warning-600' }
]

export default function AgentFilters({ selectedFilter, onFilterChange }: AgentFiltersProps) {
  const currentFilter = filterOptions.find(option => option.value === selectedFilter)

  return (
    <Menu as="div" className="relative">
      <Menu.Button className="flex items-center space-x-2 px-4 py-2 bg-white border border-secondary-200 rounded-lg hover:bg-secondary-50 transition-colors">
        <FunnelIcon className="h-5 w-5 text-secondary-400" />
        <span className="text-sm font-medium text-secondary-700">
          {currentFilter?.label || 'Filter'}
        </span>
        <ChevronDownIcon className="h-4 w-4 text-secondary-400" />
      </Menu.Button>

      <Transition
        as={Fragment}
        enter="transition ease-out duration-100"
        enterFrom="transform opacity-0 scale-95"
        enterTo="transform opacity-100 scale-100"
        leave="transition ease-in duration-75"
        leaveFrom="transform opacity-100 scale-100"
        leaveTo="transform opacity-0 scale-95"
      >
        <Menu.Items className="absolute right-0 z-50 mt-2 w-56 origin-top-right bg-white rounded-lg shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none">
          <div className="py-1">
            {filterOptions.map((option) => (
              <Menu.Item key={option.value}>
                {({ active }) => (
                  <button
                    onClick={() => onFilterChange(option.value)}
                    className={cn(
                      'flex items-center justify-between w-full px-4 py-2 text-sm transition-colors',
                      active ? 'bg-secondary-50' : '',
                      selectedFilter === option.value
                        ? 'bg-primary-50 text-primary-600 font-medium'
                        : 'text-secondary-700'
                    )}
                  >
                    <div className="flex items-center space-x-3">
                      {option.value !== 'all' && (
                        <div className={cn(
                          'w-2 h-2 rounded-full',
                          option.value === 'online' && 'bg-success-500',
                          option.value === 'offline' && 'bg-secondary-400',
                          option.value === 'warning' && 'bg-warning-500'
                        )} />
                      )}
                      <span>{option.label}</span>
                    </div>
                    
                    {option.count !== null && (
                      <span className={cn(
                        'px-2 py-1 text-xs rounded-full',
                        selectedFilter === option.value
                          ? 'bg-primary-100 text-primary-600'
                          : 'bg-secondary-100 text-secondary-600'
                      )}>
                        {option.count}
                      </span>
                    )}
                  </button>
                )}
              </Menu.Item>
            ))}
          </div>
          
          <div className="border-t border-secondary-100 py-1">
            <div className="px-4 py-2">
              <h4 className="text-xs font-semibold text-secondary-500 uppercase tracking-wide mb-2">
                Erweiterte Filter
              </h4>
              <div className="space-y-1">
                <label className="flex items-center">
                  <input type="checkbox" className="mr-2 rounded" />
                  <span className="text-sm text-secondary-700">Mit Bedrohungen</span>
                </label>
                <label className="flex items-center">
                  <input type="checkbox" className="mr-2 rounded" />
                  <span className="text-sm text-secondary-700">Veraltete Version</span>
                </label>
                <label className="flex items-center">
                  <input type="checkbox" className="mr-2 rounded" />
                  <span className="text-sm text-secondary-700">Hohe CPU-Last</span>
                </label>
              </div>
            </div>
          </div>
        </Menu.Items>
      </Transition>
    </Menu>
  )
}