import { useState } from 'react';
import { Dialog, Transition } from '@headlessui/react';
import { Fragment } from 'react';
import { 
  XMarkIcon,
  PlayIcon,
  PauseIcon,
  StopIcon,
  ArrowPathIcon,
  TrashIcon,
  ExclamationTriangleIcon,
  ShieldCheckIcon,
  CogIcon,
  DocumentTextIcon
} from '@heroicons/react/24/outline';
import { Asset } from '@/pages/AssetManagement';
import { useAuthStore } from '@/stores/authStore';
import toast from 'react-hot-toast';

interface AssetControlsProps {
  asset: Asset;
  onClose: () => void;
  onAssetUpdate: (updates: Partial<Asset>) => void;
}

export default function AssetControls({ asset, onClose, onAssetUpdate }: AssetControlsProps) {
  const { user } = useAuthStore();
  const [loading, setLoading] = useState(false);
  const [confirmAction, setConfirmAction] = useState<string | null>(null);

  const canPerformAction = (action: string) => {
    switch (action) {
      case 'pause':
      case 'resume':
      case 'restart':
        return asset.permissions.canPause && (user?.canControlAgents || false);
      case 'stop':
        return asset.permissions.canStop && (user?.canControlAgents || false);
      case 'uninstall':
      case 'force_stop':
        return asset.permissions.canUninstall && (user?.canAdminSystem || false);
      case 'update_config':
        return asset.permissions.canUpdateConfig && (user?.canControlAgents || false);
      case 'view_logs':
        return asset.permissions.canViewLogs;
      default:
        return false;
    }
  };

  const executeAction = async (action: string) => {
    if (!canPerformAction(action)) {
      toast.error('You do not have permission to perform this action');
      return;
    }

    setLoading(true);
    
    try {
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1500));
      
      let updates: Partial<Asset> = {};
      let successMessage = '';

      switch (action) {
        case 'pause':
          updates = { 
            monitoringStatus: 'paused',
            status: asset.status === 'online' ? 'online' : asset.status
          };
          successMessage = 'Agent monitoring paused successfully';
          break;
        case 'resume':
          updates = { 
            monitoringStatus: 'monitoring',
            status: 'online'
          };
          successMessage = 'Agent monitoring resumed successfully';
          break;
        case 'restart':
          updates = { 
            status: 'online',
            monitoringStatus: 'monitoring',
            lastSeen: new Date()
          };
          successMessage = 'Agent restarted successfully';
          break;
        case 'stop':
          updates = { 
            status: 'stopping',
            monitoringStatus: 'stopped'
          };
          successMessage = 'Agent stop command sent';
          setTimeout(() => {
            onAssetUpdate({ status: 'offline', monitoringStatus: 'stopped' });
          }, 3000);
          break;
        case 'force_stop':
          updates = { 
            status: 'offline',
            monitoringStatus: 'stopped'
          };
          successMessage = 'Agent force stopped';
          break;
        case 'uninstall':
          updates = { 
            status: 'offline',
            monitoringStatus: 'stopped'
          };
          successMessage = 'Agent uninstall initiated';
          break;
        default:
          throw new Error('Unknown action');
      }

      onAssetUpdate(updates);
      toast.success(successMessage);
      
      if (action === 'uninstall' || action === 'force_stop') {
        setTimeout(onClose, 1000);
      }
      
    } catch (error) {
      toast.error(`Failed to ${action} agent: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setLoading(false);
      setConfirmAction(null);
    }
  };

  const getActionButton = (action: string, icon: React.ElementType, label: string, description: string, variant: 'primary' | 'warning' | 'danger' = 'primary') => {
    const canPerform = canPerformAction(action);
    const Icon = icon;
    
    const baseClasses = "w-full flex items-center justify-between p-4 rounded-lg border-2 transition-all duration-200 text-left";
    const variantClasses = {
      primary: canPerform 
        ? "border-blue-200 dark:border-blue-800 hover:border-blue-300 dark:hover:border-blue-700 hover:bg-blue-50 dark:hover:bg-blue-900/20" 
        : "border-gray-200 dark:border-gray-700 opacity-50 cursor-not-allowed",
      warning: canPerform 
        ? "border-yellow-200 dark:border-yellow-800 hover:border-yellow-300 dark:hover:border-yellow-700 hover:bg-yellow-50 dark:hover:bg-yellow-900/20" 
        : "border-gray-200 dark:border-gray-700 opacity-50 cursor-not-allowed",
      danger: canPerform 
        ? "border-red-200 dark:border-red-800 hover:border-red-300 dark:hover:border-red-700 hover:bg-red-50 dark:hover:bg-red-900/20" 
        : "border-gray-200 dark:border-gray-700 opacity-50 cursor-not-allowed"
    };

    return (
      <button
        key={action}
        onClick={() => canPerform && setConfirmAction(action)}
        disabled={!canPerform || loading}
        className={`${baseClasses} ${variantClasses[variant]}`}
      >
        <div className="flex items-center space-x-3">
          <Icon className={`h-5 w-5 ${
            variant === 'danger' ? 'text-red-500' :
            variant === 'warning' ? 'text-yellow-500' :
            'text-blue-500'
          }`} />
          <div>
            <div className="font-medium text-gray-900 dark:text-white">{label}</div>
            <div className="text-sm text-gray-500 dark:text-gray-400">{description}</div>
          </div>
        </div>
        {!canPerform && (
          <span className="text-xs text-gray-400 bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded">
            No permission
          </span>
        )}
      </button>
    );
  };

  const getAvailableActions = () => {
    const actions = [];

    // Basic monitoring controls
    if (asset.monitoringStatus === 'monitoring') {
      actions.push(getActionButton(
        'pause',
        PauseIcon,
        'Pause Monitoring',
        'Pause security monitoring while keeping agent online',
        'warning'
      ));
    } else if (asset.monitoringStatus === 'paused') {
      actions.push(getActionButton(
        'resume',
        PlayIcon,
        'Resume Monitoring',
        'Resume security monitoring on this agent',
        'primary'
      ));
    }

    // Restart option (always available if permissions allow)
    actions.push(getActionButton(
      'restart',
      ArrowPathIcon,
      'Restart Agent',
      'Restart the agent service and resume monitoring',
      'primary'
    ));

    // Stop option
    if (asset.status !== 'offline' && asset.status !== 'stopping') {
      actions.push(getActionButton(
        'stop',
        StopIcon,
        'Stop Agent',
        'Gracefully stop the agent service',
        'warning'
      ));
    }

    // Admin-only options
    if (user?.canAdminSystem) {
      if (asset.status !== 'offline') {
        actions.push(getActionButton(
          'force_stop',
          StopIcon,
          'Force Stop Agent',
          'Immediately terminate agent process (Admin only)',
          'danger'
        ));
      }

      actions.push(getActionButton(
        'uninstall',
        TrashIcon,
        'Uninstall Agent',
        'Remove agent from system and disable autostart (Admin only)',
        'danger'
      ));
    }

    return actions;
  };

  return (
    <>
      <Transition appear show={true} as={Fragment}>
        <Dialog as="div" className="relative z-50" onClose={onClose}>
          <Transition.Child
            as={Fragment}
            enter="ease-out duration-300"
            enterFrom="opacity-0"
            enterTo="opacity-100"
            leave="ease-in duration-200"
            leaveFrom="opacity-100"
            leaveTo="opacity-0"
          >
            <div className="fixed inset-0 bg-black bg-opacity-25" />
          </Transition.Child>

          <div className="fixed inset-0 overflow-y-auto">
            <div className="flex min-h-full items-center justify-center p-4 text-center">
              <Transition.Child
                as={Fragment}
                enter="ease-out duration-300"
                enterFrom="opacity-0 scale-95"
                enterTo="opacity-100 scale-100"
                leave="ease-in duration-200"
                leaveFrom="opacity-100 scale-100"
                leaveTo="opacity-0 scale-95"
              >
                <Dialog.Panel className="w-full max-w-2xl transform overflow-hidden rounded-2xl bg-white dark:bg-gray-800 p-6 text-left align-middle shadow-xl transition-all">
                  <div className="flex items-center justify-between mb-6">
                    <div className="flex items-center space-x-3">
                      <ShieldCheckIcon className="h-6 w-6 text-blue-500" />
                      <Dialog.Title as="h3" className="text-lg font-medium leading-6 text-gray-900 dark:text-white">
                        Agent Controls - {asset.name}
                      </Dialog.Title>
                    </div>
                    <button
                      onClick={onClose}
                      className="rounded-md p-2 hover:bg-gray-100 dark:hover:bg-gray-700"
                    >
                      <XMarkIcon className="h-5 w-5 text-gray-400" />
                    </button>
                  </div>

                  {/* Asset Status */}
                  <div className="mb-6 p-4 bg-gray-50 dark:bg-gray-700 rounded-lg">
                    <div className="grid grid-cols-2 gap-4 text-sm">
                      <div>
                        <span className="text-gray-500 dark:text-gray-400">Status:</span>
                        <span className={`ml-2 font-medium ${
                          asset.status === 'online' ? 'text-green-600 dark:text-green-400' :
                          asset.status === 'offline' ? 'text-red-600 dark:text-red-400' :
                          'text-yellow-600 dark:text-yellow-400'
                        }`}>
                          {asset.status.charAt(0).toUpperCase() + asset.status.slice(1)}
                        </span>
                      </div>
                      <div>
                        <span className="text-gray-500 dark:text-gray-400">Monitoring:</span>
                        <span className={`ml-2 font-medium ${
                          asset.monitoringStatus === 'monitoring' ? 'text-green-600 dark:text-green-400' :
                          asset.monitoringStatus === 'paused' ? 'text-yellow-600 dark:text-yellow-400' :
                          'text-red-600 dark:text-red-400'
                        }`}>
                          {asset.monitoringStatus.charAt(0).toUpperCase() + asset.monitoringStatus.slice(1)}
                        </span>
                      </div>
                    </div>
                  </div>

                  {/* Available Actions */}
                  <div className="space-y-3">
                    <h4 className="text-sm font-medium text-gray-900 dark:text-white">Available Actions</h4>
                    {getAvailableActions()}
                  </div>

                  {/* Additional Options */}
                  <div className="mt-6 pt-6 border-t border-gray-200 dark:border-gray-600">
                    <h4 className="text-sm font-medium text-gray-900 dark:text-white mb-3">Additional Options</h4>
                    <div className="grid grid-cols-2 gap-3">
                      <button
                        disabled={!canPerformAction('update_config')}
                        className="flex items-center justify-center space-x-2 p-3 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed"
                      >
                        <CogIcon className="h-4 w-4" />
                        <span className="text-sm">Update Config</span>
                      </button>
                      <button
                        disabled={!canPerformAction('view_logs')}
                        className="flex items-center justify-center space-x-2 p-3 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed"
                      >
                        <DocumentTextIcon className="h-4 w-4" />
                        <span className="text-sm">View Logs</span>
                      </button>
                    </div>
                  </div>
                </Dialog.Panel>
              </Transition.Child>
            </div>
          </div>
        </Dialog>
      </Transition>

      {/* Confirmation Dialog */}
      {confirmAction && (
        <Transition appear show={!!confirmAction} as={Fragment}>
          <Dialog as="div" className="relative z-50" onClose={() => setConfirmAction(null)}>
            <Transition.Child
              as={Fragment}
              enter="ease-out duration-300"
              enterFrom="opacity-0"
              enterTo="opacity-100"
              leave="ease-in duration-200"
              leaveFrom="opacity-100"
              leaveTo="opacity-0"
            >
              <div className="fixed inset-0 bg-black bg-opacity-25" />
            </Transition.Child>

            <div className="fixed inset-0 overflow-y-auto">
              <div className="flex min-h-full items-center justify-center p-4 text-center">
                <Transition.Child
                  as={Fragment}
                  enter="ease-out duration-300"
                  enterFrom="opacity-0 scale-95"
                  enterTo="opacity-100 scale-100"
                  leave="ease-in duration-200"
                  leaveFrom="opacity-100 scale-100"
                  leaveTo="opacity-0 scale-95"
                >
                  <Dialog.Panel className="w-full max-w-md transform overflow-hidden rounded-2xl bg-white dark:bg-gray-800 p-6 text-left align-middle shadow-xl transition-all">
                    <div className="flex items-center space-x-3 mb-4">
                      <ExclamationTriangleIcon className="h-6 w-6 text-yellow-500" />
                      <Dialog.Title as="h3" className="text-lg font-medium leading-6 text-gray-900 dark:text-white">
                        Confirm Action
                      </Dialog.Title>
                    </div>

                    <div className="mb-6">
                      <p className="text-sm text-gray-500 dark:text-gray-400">
                        Are you sure you want to <strong>{confirmAction}</strong> the agent on <strong>{asset.name}</strong>?
                      </p>
                      {['uninstall', 'force_stop'].includes(confirmAction) && (
                        <p className="mt-2 text-sm text-red-600 dark:text-red-400">
                          This action cannot be undone and may require manual intervention to restore the agent.
                        </p>
                      )}
                    </div>

                    <div className="flex space-x-3">
                      <button
                        onClick={() => setConfirmAction(null)}
                        disabled={loading}
                        className="flex-1 px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-600 disabled:opacity-50"
                      >
                        Cancel
                      </button>
                      <button
                        onClick={() => executeAction(confirmAction)}
                        disabled={loading}
                        className={`flex-1 px-4 py-2 text-sm font-medium text-white rounded-md disabled:opacity-50 ${
                          ['uninstall', 'force_stop'].includes(confirmAction)
                            ? 'bg-red-600 hover:bg-red-700'
                            : confirmAction === 'stop'
                            ? 'bg-yellow-600 hover:bg-yellow-700'
                            : 'bg-blue-600 hover:bg-blue-700'
                        }`}
                      >
                        {loading ? (
                          <div className="flex items-center justify-center space-x-2">
                            <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                            <span>Processing...</span>
                          </div>
                        ) : (
                          `Confirm ${confirmAction.charAt(0).toUpperCase() + confirmAction.slice(1)}`
                        )}
                      </button>
                    </div>
                  </Dialog.Panel>
                </Transition.Child>
              </div>
            </div>
          </Dialog>
        </Transition>
      )}
    </>
  );
}