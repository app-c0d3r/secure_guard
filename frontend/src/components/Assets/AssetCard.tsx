import { useState } from 'react';
import { motion } from 'framer-motion';
import { 
  ComputerDesktopIcon,
  ServerIcon,
  DevicePhoneMobileIcon,
  ShieldCheckIcon,
  ClockIcon,
  StopIcon,
  ExclamationTriangleIcon,
  CpuChipIcon,
  CircleStackIcon,
  EllipsisVerticalIcon
} from '@heroicons/react/24/outline';
import { Menu, Transition } from '@headlessui/react';
import { Fragment } from 'react';
import { Asset } from '@/pages/AssetManagement';
import AssetControls from './AssetControls';
import { cn } from '@/lib/utils';

interface AssetCardProps {
  asset: Asset;
  selected: boolean;
  onSelectionChange: (selected: boolean) => void;
  onAssetUpdate: (updates: Partial<Asset>) => void;
}

export default function AssetCard({ asset, selected, onSelectionChange, onAssetUpdate }: AssetCardProps) {
  const [showControls, setShowControls] = useState(false);

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'online': return 'bg-green-500';
      case 'offline': return 'bg-red-500';
      case 'paused': return 'bg-yellow-500';
      case 'stopping': return 'bg-orange-500';
      case 'error': return 'bg-red-600';
      default: return 'bg-gray-500';
    }
  };

  const getMonitoringStatusColor = (status: string) => {
    switch (status) {
      case 'monitoring': return 'text-green-600 dark:text-green-400';
      case 'paused': return 'text-yellow-600 dark:text-yellow-400';
      case 'stopped': return 'text-red-600 dark:text-red-400';
      default: return 'text-gray-600 dark:text-gray-400';
    }
  };

  const getOSIcon = (os: string) => {
    const osLower = os.toLowerCase();
    if (osLower.includes('windows')) return ComputerDesktopIcon;
    if (osLower.includes('linux') || osLower.includes('ubuntu')) return ServerIcon;
    if (osLower.includes('mac') || osLower.includes('darwin')) return DevicePhoneMobileIcon;
    return ComputerDesktopIcon;
  };

  const formatLastSeen = (date: Date) => {
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const minutes = Math.floor(diff / (1000 * 60));
    const hours = Math.floor(diff / (1000 * 60 * 60));
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));

    if (minutes < 1) return 'Just now';
    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    return `${days}d ago`;
  };

  const OSIcon = getOSIcon(asset.osInfo.os);

  return (
    <motion.div
      layout
      initial={{ opacity: 0, scale: 0.9 }}
      animate={{ opacity: 1, scale: 1 }}
      whileHover={{ y: -2 }}
      className={cn(
        "bg-white dark:bg-gray-800 rounded-lg shadow-sm border-2 transition-all duration-200",
        selected 
          ? "border-blue-500 dark:border-blue-400 shadow-md" 
          : "border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600"
      )}
    >
      {/* Header */}
      <div className="p-4 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <input
              type="checkbox"
              checked={selected}
              onChange={(e) => onSelectionChange(e.target.checked)}
              className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
            />
            <div className="flex items-center space-x-2">
              <OSIcon className="h-5 w-5 text-gray-600 dark:text-gray-400" />
              <h3 className="text-sm font-medium text-gray-900 dark:text-white">
                {asset.name}
              </h3>
            </div>
          </div>
          
          <div className="flex items-center space-x-2">
            {/* Status Indicator */}
            <div className="flex items-center space-x-1">
              <div className={cn("w-2 h-2 rounded-full", getStatusColor(asset.status))} />
              <span className="text-xs text-gray-500 dark:text-gray-400 capitalize">
                {asset.status}
              </span>
            </div>

            {/* More Actions Menu */}
            <Menu as="div" className="relative">
              <Menu.Button className="p-1 rounded-md hover:bg-gray-100 dark:hover:bg-gray-700">
                <EllipsisVerticalIcon className="h-4 w-4 text-gray-400" />
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
                <Menu.Items className="absolute right-0 mt-2 w-48 origin-top-right bg-white dark:bg-gray-800 divide-y divide-gray-100 dark:divide-gray-700 rounded-md shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none z-10">
                  <div className="px-1 py-1">
                    <Menu.Item>
                      {({ active }) => (
                        <button
                          onClick={() => setShowControls(true)}
                          className={cn(
                            active ? 'bg-gray-100 dark:bg-gray-700' : '',
                            'group flex rounded-md items-center w-full px-2 py-2 text-sm text-gray-900 dark:text-white'
                          )}
                        >
                          <ShieldCheckIcon className="w-4 h-4 mr-2" />
                          Control Agent
                        </button>
                      )}
                    </Menu.Item>
                    <Menu.Item>
                      {({ active }) => (
                        <button
                          className={cn(
                            active ? 'bg-gray-100 dark:bg-gray-700' : '',
                            'group flex rounded-md items-center w-full px-2 py-2 text-sm text-gray-900 dark:text-white'
                          )}
                        >
                          <CpuChipIcon className="w-4 h-4 mr-2" />
                          View Details
                        </button>
                      )}
                    </Menu.Item>
                    <Menu.Item>
                      {({ active }) => (
                        <button
                          className={cn(
                            active ? 'bg-gray-100 dark:bg-gray-700' : '',
                            'group flex rounded-md items-center w-full px-2 py-2 text-sm text-gray-900 dark:text-white'
                          )}
                        >
                          <CircleStackIcon className="w-4 h-4 mr-2" />
                          View Logs
                        </button>
                      )}
                    </Menu.Item>
                  </div>
                </Menu.Items>
              </Transition>
            </Menu>
          </div>
        </div>

        {/* Asset Info */}
        <div className="mt-2 space-y-1">
          <p className="text-xs text-gray-500 dark:text-gray-400">
            {asset.hostname} • {asset.ipAddress}
          </p>
          <p className="text-xs text-gray-500 dark:text-gray-400">
            {asset.osInfo.os} {asset.osInfo.version}
          </p>
        </div>
      </div>

      {/* Status and Monitoring */}
      <div className="p-4 space-y-3">
        {/* Monitoring Status */}
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-2">
            {asset.monitoringStatus === 'monitoring' && (
              <ShieldCheckIcon className="h-4 w-4 text-green-500" />
            )}
            {asset.monitoringStatus === 'paused' && (
              <ClockIcon className="h-4 w-4 text-yellow-500" />
            )}
            {asset.monitoringStatus === 'stopped' && (
              <StopIcon className="h-4 w-4 text-red-500" />
            )}
            <span className={cn("text-xs font-medium", getMonitoringStatusColor(asset.monitoringStatus))}>
              {asset.monitoringStatus.charAt(0).toUpperCase() + asset.monitoringStatus.slice(1)}
            </span>
          </div>
          
          <span className="text-xs text-gray-500 dark:text-gray-400">
            {formatLastSeen(asset.lastSeen)}
          </span>
        </div>

        {/* Metrics */}
        <div className="grid grid-cols-3 gap-2">
          <div className="text-center">
            <div className="text-xs text-gray-500 dark:text-gray-400">CPU</div>
            <div className={cn(
              "text-sm font-medium",
              asset.metrics.cpuUsage > 80 ? "text-red-600 dark:text-red-400" :
              asset.metrics.cpuUsage > 60 ? "text-yellow-600 dark:text-yellow-400" :
              "text-green-600 dark:text-green-400"
            )}>
              {asset.metrics.cpuUsage}%
            </div>
          </div>
          <div className="text-center">
            <div className="text-xs text-gray-500 dark:text-gray-400">Memory</div>
            <div className={cn(
              "text-sm font-medium",
              asset.metrics.memoryUsage > 80 ? "text-red-600 dark:text-red-400" :
              asset.metrics.memoryUsage > 60 ? "text-yellow-600 dark:text-yellow-400" :
              "text-green-600 dark:text-green-400"
            )}>
              {asset.metrics.memoryUsage}%
            </div>
          </div>
          <div className="text-center">
            <div className="text-xs text-gray-500 dark:text-gray-400">Threats</div>
            <div className={cn(
              "text-sm font-medium",
              asset.metrics.threats > 0 ? "text-red-600 dark:text-red-400" : "text-green-600 dark:text-green-400"
            )}>
              {asset.metrics.threats}
            </div>
          </div>
        </div>

        {/* Capabilities */}
        <div className="pt-2 border-t border-gray-200 dark:border-gray-700">
          <div className="flex flex-wrap gap-1">
            {asset.capabilities.slice(0, 3).map((capability) => (
              <span
                key={capability}
                className="inline-flex items-center px-2 py-1 rounded text-xs font-medium bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200"
              >
                {capability.replace('_', ' ')}
              </span>
            ))}
            {asset.capabilities.length > 3 && (
              <span className="inline-flex items-center px-2 py-1 rounded text-xs font-medium bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-200">
                +{asset.capabilities.length - 3} more
              </span>
            )}
          </div>
        </div>

        {/* Threats Alert */}
        {asset.metrics.threats > 0 && (
          <div className="flex items-center space-x-2 p-2 bg-red-50 dark:bg-red-900/20 rounded-md">
            <ExclamationTriangleIcon className="h-4 w-4 text-red-500" />
            <span className="text-xs text-red-700 dark:text-red-400">
              {asset.metrics.threats} threat{asset.metrics.threats !== 1 ? 's' : ''} detected
            </span>
          </div>
        )}
      </div>

      {/* Asset Controls Modal */}
      {showControls && (
        <AssetControls
          asset={asset}
          onClose={() => setShowControls(false)}
          onAssetUpdate={onAssetUpdate}
        />
      )}
    </motion.div>
  );
}