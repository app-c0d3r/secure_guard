import React, { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { useAuthStore } from '@/stores/authStore';
import AssetGrid from '@/components/Assets/AssetGrid';
import AssetFilters from '@/components/Assets/AssetFilters';
import AssetControls from '@/components/Assets/AssetControls';
import BulkOperations from '@/components/Assets/BulkOperations';
import { 
  ComputerDesktopIcon, 
  ServerIcon,
  ShieldCheckIcon,
  ExclamationTriangleIcon,
  ClockIcon
} from '@heroicons/react/24/outline';

export interface Asset {
  id: string;
  agentId: string;
  name: string;
  hostname: string;
  ipAddress: string;
  osInfo: {
    os: string;
    version: string;
    hostname: string;
  };
  status: 'online' | 'offline' | 'paused' | 'error' | 'stopping';
  monitoringStatus: 'monitoring' | 'paused' | 'stopped';
  lastSeen: Date;
  version: string;
  capabilities: string[];
  metrics: {
    cpuUsage: number;
    memoryUsage: number;
    diskUsage: number;
    threats: number;
  };
  permissions: {
    canPause: boolean;
    canStop: boolean;
    canRestart: boolean;
    canUninstall: boolean;
    canViewLogs: boolean;
    canUpdateConfig: boolean;
  };
}

export default function AssetManagement() {
  const { user } = useAuthStore();
  const [assets, setAssets] = useState<Asset[]>([]);
  const [selectedAssets, setSelectedAssets] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);
  const [filters, setFilters] = useState({
    status: 'all',
    monitoringStatus: 'all',
    search: '',
    osType: 'all'
  });

  // Mock data - in real app, this would come from API
  useEffect(() => {
    const mockAssets: Asset[] = [
      {
        id: '1',
        agentId: 'agent-001',
        name: 'Workstation-001',
        hostname: 'WIN-DESKTOP-001',
        ipAddress: '192.168.1.100',
        osInfo: {
          os: 'Windows',
          version: '10 Pro',
          hostname: 'WIN-DESKTOP-001'
        },
        status: 'online',
        monitoringStatus: 'monitoring',
        lastSeen: new Date(Date.now() - 1000 * 60 * 2),
        version: '1.0.0',
        capabilities: ['file_monitoring', 'process_monitoring', 'network_monitoring'],
        metrics: {
          cpuUsage: 45,
          memoryUsage: 67,
          diskUsage: 78,
          threats: 0
        },
        permissions: {
          canPause: user?.canControlAgents || false,
          canStop: user?.canControlAgents || false,
          canRestart: user?.canControlAgents || false,
          canUninstall: user?.canAdminSystem || false,
          canViewLogs: true,
          canUpdateConfig: user?.canControlAgents || false
        }
      },
      {
        id: '2',
        agentId: 'agent-002',
        name: 'Server-DB-01',
        hostname: 'SRV-DATABASE-01',
        ipAddress: '192.168.1.50',
        osInfo: {
          os: 'Linux',
          version: 'Ubuntu 22.04',
          hostname: 'SRV-DATABASE-01'
        },
        status: 'online',
        monitoringStatus: 'paused',
        lastSeen: new Date(Date.now() - 1000 * 60 * 5),
        version: '1.0.0',
        capabilities: ['file_monitoring', 'process_monitoring', 'network_monitoring', 'database_monitoring'],
        metrics: {
          cpuUsage: 12,
          memoryUsage: 34,
          diskUsage: 45,
          threats: 2
        },
        permissions: {
          canPause: user?.canControlAgents || false,
          canStop: user?.canControlAgents || false,
          canRestart: user?.canControlAgents || false,
          canUninstall: user?.canAdminSystem || false,
          canViewLogs: true,
          canUpdateConfig: user?.canControlAgents || false
        }
      },
      {
        id: '3',
        agentId: 'agent-003',
        name: 'Laptop-Mobile-05',
        hostname: 'LAPTOP-MOBILE-05',
        ipAddress: '192.168.1.120',
        osInfo: {
          os: 'macOS',
          version: 'Sonoma 14.1',
          hostname: 'LAPTOP-MOBILE-05'
        },
        status: 'offline',
        monitoringStatus: 'stopped',
        lastSeen: new Date(Date.now() - 1000 * 60 * 60 * 2),
        version: '0.9.8',
        capabilities: ['file_monitoring', 'process_monitoring'],
        metrics: {
          cpuUsage: 0,
          memoryUsage: 0,
          diskUsage: 0,
          threats: 0
        },
        permissions: {
          canPause: false,
          canStop: false,
          canRestart: user?.canControlAgents || false,
          canUninstall: user?.canAdminSystem || false,
          canViewLogs: true,
          canUpdateConfig: false
        }
      }
    ];

    setTimeout(() => {
      setAssets(mockAssets);
      setLoading(false);
    }, 1000);
  }, [user]);

  const getStatusStats = () => {
    const stats = {
      total: assets.length,
      online: assets.filter(a => a.status === 'online').length,
      offline: assets.filter(a => a.status === 'offline').length,
      monitoring: assets.filter(a => a.monitoringStatus === 'monitoring').length,
      paused: assets.filter(a => a.monitoringStatus === 'paused').length,
      threats: assets.reduce((sum, a) => sum + a.metrics.threats, 0)
    };
    return stats;
  };

  const filteredAssets = assets.filter(asset => {
    if (filters.status !== 'all' && asset.status !== filters.status) return false;
    if (filters.monitoringStatus !== 'all' && asset.monitoringStatus !== filters.monitoringStatus) return false;
    if (filters.search && !asset.name.toLowerCase().includes(filters.search.toLowerCase()) &&
        !asset.hostname.toLowerCase().includes(filters.search.toLowerCase()) &&
        !asset.ipAddress.includes(filters.search)) return false;
    if (filters.osType !== 'all' && !asset.osInfo.os.toLowerCase().includes(filters.osType.toLowerCase())) return false;
    return true;
  });

  const stats = getStatusStats();

  const handleBulkOperation = async (operation: string, assetIds: string[]) => {
    console.log(`Performing ${operation} on assets:`, assetIds);
    // In real app, this would call the API
    // For now, just update local state
    setAssets(prev => prev.map(asset => {
      if (assetIds.includes(asset.id)) {
        switch (operation) {
          case 'pause':
            return { ...asset, monitoringStatus: 'paused' as const };
          case 'resume':
            return { ...asset, monitoringStatus: 'monitoring' as const };
          case 'stop':
            return { ...asset, status: 'stopping' as const, monitoringStatus: 'stopped' as const };
          default:
            return asset;
        }
      }
      return asset;
    }));
    setSelectedAssets([]);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="md:flex md:items-center md:justify-between">
        <div className="flex-1 min-w-0">
          <motion.h2 
            initial={{ opacity: 0, y: -20 }}
            animate={{ opacity: 1, y: 0 }}
            className="text-2xl font-bold leading-7 text-gray-900 dark:text-white sm:text-3xl sm:truncate"
          >
            Asset Management
          </motion.h2>
          <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
            Monitor and control all connected agents and endpoints
          </p>
        </div>
      </div>

      {/* Stats Cards */}
      <motion.div 
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.1 }}
        className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-6"
      >
        <div className="bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg border border-gray-200 dark:border-gray-700">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <ComputerDesktopIcon className="h-6 w-6 text-gray-400" />
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">
                    Total Assets
                  </dt>
                  <dd className="text-lg font-medium text-gray-900 dark:text-white">
                    {stats.total}
                  </dd>
                </dl>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg border border-gray-200 dark:border-gray-700">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <div className="w-6 h-6 bg-green-100 dark:bg-green-900 rounded-full flex items-center justify-center">
                  <div className="w-3 h-3 bg-green-500 rounded-full"></div>
                </div>
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">
                    Online
                  </dt>
                  <dd className="text-lg font-medium text-gray-900 dark:text-white">
                    {stats.online}
                  </dd>
                </dl>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg border border-gray-200 dark:border-gray-700">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <div className="w-6 h-6 bg-red-100 dark:bg-red-900 rounded-full flex items-center justify-center">
                  <div className="w-3 h-3 bg-red-500 rounded-full"></div>
                </div>
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">
                    Offline
                  </dt>
                  <dd className="text-lg font-medium text-gray-900 dark:text-white">
                    {stats.offline}
                  </dd>
                </dl>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg border border-gray-200 dark:border-gray-700">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <ShieldCheckIcon className="h-6 w-6 text-blue-500" />
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">
                    Monitoring
                  </dt>
                  <dd className="text-lg font-medium text-gray-900 dark:text-white">
                    {stats.monitoring}
                  </dd>
                </dl>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg border border-gray-200 dark:border-gray-700">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <ClockIcon className="h-6 w-6 text-yellow-500" />
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">
                    Paused
                  </dt>
                  <dd className="text-lg font-medium text-gray-900 dark:text-white">
                    {stats.paused}
                  </dd>
                </dl>
              </div>
            </div>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg border border-gray-200 dark:border-gray-700">
          <div className="p-5">
            <div className="flex items-center">
              <div className="flex-shrink-0">
                <ExclamationTriangleIcon className="h-6 w-6 text-red-500" />
              </div>
              <div className="ml-5 w-0 flex-1">
                <dl>
                  <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">
                    Threats
                  </dt>
                  <dd className="text-lg font-medium text-gray-900 dark:text-white">
                    {stats.threats}
                  </dd>
                </dl>
              </div>
            </div>
          </div>
        </div>
      </motion.div>

      {/* Filters and Controls */}
      <motion.div 
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.2 }}
        className="flex flex-col lg:flex-row gap-4"
      >
        <div className="flex-1">
          <AssetFilters 
            filters={filters}
            onFiltersChange={setFilters}
            assetCount={filteredAssets.length}
          />
        </div>
        {selectedAssets.length > 0 && (
          <BulkOperations
            selectedCount={selectedAssets.length}
            onBulkOperation={handleBulkOperation}
            selectedAssets={selectedAssets}
            canPerformAdminActions={user?.canAdminSystem || false}
          />
        )}
      </motion.div>

      {/* Asset Grid */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.3 }}
      >
        <AssetGrid
          assets={filteredAssets}
          loading={loading}
          selectedAssets={selectedAssets}
          onSelectionChange={setSelectedAssets}
          onAssetUpdate={(assetId, updates) => {
            setAssets(prev => prev.map(asset => 
              asset.id === assetId ? { ...asset, ...updates } : asset
            ));
          }}
        />
      </motion.div>
    </div>
  );
}