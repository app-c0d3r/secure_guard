import { useState } from 'react';
import { motion } from 'framer-motion';
import { 
  PlayIcon,
  PauseIcon,
  StopIcon,
  ArrowPathIcon,
  TrashIcon,
  ChevronDownIcon
} from '@heroicons/react/24/outline';
import { Menu, Transition } from '@headlessui/react';
import { Fragment } from 'react';
import toast from 'react-hot-toast';

interface BulkOperationsProps {
  selectedCount: number;
  selectedAssets: string[];
  onBulkOperation: (operation: string, assetIds: string[]) => Promise<void>;
  canPerformAdminActions: boolean;
}

export default function BulkOperations({ 
  selectedCount, 
  selectedAssets, 
  onBulkOperation, 
  canPerformAdminActions 
}: BulkOperationsProps) {
  const [loading, setLoading] = useState<string | null>(null);

  const handleBulkAction = async (operation: string) => {
    if (selectedAssets.length === 0) {
      toast.error('No assets selected');
      return;
    }

    const confirmationMessage = `Are you sure you want to ${operation} ${selectedCount} selected asset${selectedCount !== 1 ? 's' : ''}?`;
    
    if (['stop', 'force_stop', 'uninstall'].includes(operation)) {
      if (!confirm(confirmationMessage + '\n\nThis action cannot be undone.')) {
        return;
      }
    } else {
      if (!confirm(confirmationMessage)) {
        return;
      }
    }

    setLoading(operation);
    
    try {
      await onBulkOperation(operation, selectedAssets);
      toast.success(`Successfully ${operation}ped ${selectedCount} asset${selectedCount !== 1 ? 's' : ''}`);
    } catch (error) {
      toast.error(`Failed to ${operation} assets: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setLoading(null);
    }
  };

  const operations = [
    {
      id: 'resume',
      label: 'Resume Monitoring',
      icon: PlayIcon,
      description: 'Start monitoring on selected agents',
      variant: 'primary' as const,
      adminOnly: false
    },
    {
      id: 'pause',
      label: 'Pause Monitoring',
      icon: PauseIcon,
      description: 'Pause monitoring on selected agents',
      variant: 'warning' as const,
      adminOnly: false
    },
    {
      id: 'restart',
      label: 'Restart Agents',
      icon: ArrowPathIcon,
      description: 'Restart selected agents',
      variant: 'primary' as const,
      adminOnly: false
    },
    {
      id: 'stop',
      label: 'Stop Agents',
      icon: StopIcon,
      description: 'Gracefully stop selected agents',
      variant: 'warning' as const,
      adminOnly: false
    },
    {
      id: 'force_stop',
      label: 'Force Stop',
      icon: StopIcon,
      description: 'Force stop selected agents (Admin only)',
      variant: 'danger' as const,
      adminOnly: true
    },
    {
      id: 'uninstall',
      label: 'Uninstall Agents',
      icon: TrashIcon,
      description: 'Uninstall agents from selected systems (Admin only)',
      variant: 'danger' as const,
      adminOnly: true
    }
  ];

  const availableOperations = operations.filter(op => !op.adminOnly || canPerformAdminActions);

  const getButtonClasses = (variant: 'primary' | 'warning' | 'danger', isLoading: boolean) => {
    const baseClasses = "flex items-center space-x-2 px-3 py-2 text-sm font-medium rounded-md transition-colors duration-200 disabled:opacity-50 disabled:cursor-not-allowed";
    
    if (isLoading) {
      return `${baseClasses} bg-gray-400 text-white cursor-not-allowed`;
    }

    switch (variant) {
      case 'primary':
        return `${baseClasses} bg-blue-600 hover:bg-blue-700 text-white`;
      case 'warning':
        return `${baseClasses} bg-yellow-600 hover:bg-yellow-700 text-white`;
      case 'danger':
        return `${baseClasses} bg-red-600 hover:bg-red-700 text-white`;
      default:
        return `${baseClasses} bg-gray-600 hover:bg-gray-700 text-white`;
    }
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-blue-200 dark:border-blue-800 p-4"
    >
      <div className="flex items-center justify-between mb-4">
        <div>
          <h3 className="text-sm font-medium text-gray-900 dark:text-white">
            Bulk Operations
          </h3>
          <p className="text-xs text-gray-500 dark:text-gray-400">
            {selectedCount} asset{selectedCount !== 1 ? 's' : ''} selected
          </p>
        </div>
      </div>

      <div className="flex flex-wrap gap-2">
        {/* Quick Actions */}
        <button
          onClick={() => handleBulkAction('resume')}
          disabled={loading !== null}
          className={getButtonClasses('primary', loading === 'resume')}
        >
          {loading === 'resume' ? (
            <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
          ) : (
            <PlayIcon className="w-4 h-4" />
          )}
          <span>Resume</span>
        </button>

        <button
          onClick={() => handleBulkAction('pause')}
          disabled={loading !== null}
          className={getButtonClasses('warning', loading === 'pause')}
        >
          {loading === 'pause' ? (
            <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
          ) : (
            <PauseIcon className="w-4 h-4" />
          )}
          <span>Pause</span>
        </button>

        <button
          onClick={() => handleBulkAction('restart')}
          disabled={loading !== null}
          className={getButtonClasses('primary', loading === 'restart')}
        >
          {loading === 'restart' ? (
            <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
          ) : (
            <ArrowPathIcon className="w-4 h-4" />
          )}
          <span>Restart</span>
        </button>

        {/* More Actions Menu */}
        <Menu as="div" className="relative">
          <Menu.Button
            disabled={loading !== null}
            className="flex items-center space-x-1 px-3 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <span>More</span>
            <ChevronDownIcon className="w-4 h-4" />
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
            <Menu.Items className="absolute right-0 mt-2 w-56 origin-top-right bg-white dark:bg-gray-800 divide-y divide-gray-100 dark:divide-gray-700 rounded-md shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none z-10">
              <div className="px-1 py-1">
                {availableOperations.filter(op => !['resume', 'pause', 'restart'].includes(op.id)).map((operation) => {
                  const Icon = operation.icon;
                  const isOperationLoading = loading === operation.id;
                  
                  return (
                    <Menu.Item key={operation.id}>
                      {({ active }) => (
                        <button
                          onClick={() => handleBulkAction(operation.id)}
                          disabled={loading !== null}
                          className={`${
                            active ? 'bg-gray-100 dark:bg-gray-700' : ''
                          } group flex rounded-md items-center w-full px-2 py-2 text-sm disabled:opacity-50 disabled:cursor-not-allowed ${
                            operation.variant === 'danger' ? 'text-red-600 dark:text-red-400' :
                            operation.variant === 'warning' ? 'text-yellow-600 dark:text-yellow-400' :
                            'text-gray-900 dark:text-white'
                          }`}
                        >
                          {isOperationLoading ? (
                            <div className="w-4 h-4 mr-2 border-2 border-current border-t-transparent rounded-full animate-spin" />
                          ) : (
                            <Icon className="w-4 h-4 mr-2" />
                          )}
                          <div className="text-left">
                            <div className="font-medium">{operation.label}</div>
                            <div className="text-xs text-gray-500 dark:text-gray-400">
                              {operation.description}
                            </div>
                          </div>
                        </button>
                      )}
                    </Menu.Item>
                  );
                })}
              </div>
            </Menu.Items>
          </Transition>
        </Menu>
      </div>

      {/* Permission Warning */}
      {!canPerformAdminActions && (
        <div className="mt-3 p-2 bg-yellow-50 dark:bg-yellow-900/20 rounded-md">
          <p className="text-xs text-yellow-700 dark:text-yellow-400">
            Some actions require admin permissions and may not be available.
          </p>
        </div>
      )}
    </motion.div>
  );
}