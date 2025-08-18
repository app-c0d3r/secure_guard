import { motion, AnimatePresence } from 'framer-motion';
import AssetCard from './AssetCard';
import { Asset } from '@/pages/AssetManagement';
import { ComputerDesktopIcon } from '@heroicons/react/24/outline';

interface AssetGridProps {
  assets: Asset[];
  loading: boolean;
  selectedAssets: string[];
  onSelectionChange: (selectedAssets: string[]) => void;
  onAssetUpdate: (assetId: string, updates: Partial<Asset>) => void;
}

export default function AssetGrid({ 
  assets, 
  loading, 
  selectedAssets, 
  onSelectionChange, 
  onAssetUpdate 
}: AssetGridProps) {
  const handleAssetSelection = (assetId: string, selected: boolean) => {
    if (selected) {
      onSelectionChange([...selectedAssets, assetId]);
    } else {
      onSelectionChange(selectedAssets.filter(id => id !== assetId));
    }
  };

  const handleSelectAll = () => {
    if (selectedAssets.length === assets.length) {
      onSelectionChange([]);
    } else {
      onSelectionChange(assets.map(asset => asset.id));
    }
  };

  if (loading) {
    return (
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <div className="w-4 h-4 bg-gray-200 dark:bg-gray-700 rounded animate-pulse" />
            <div className="w-24 h-4 bg-gray-200 dark:bg-gray-700 rounded animate-pulse" />
          </div>
        </div>
        
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
          {[...Array(8)].map((_, i) => (
            <div key={i} className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700 p-4 animate-pulse">
              <div className="space-y-3">
                <div className="flex items-center space-x-3">
                  <div className="w-4 h-4 bg-gray-200 dark:bg-gray-700 rounded" />
                  <div className="w-5 h-5 bg-gray-200 dark:bg-gray-700 rounded" />
                  <div className="w-24 h-4 bg-gray-200 dark:bg-gray-700 rounded" />
                </div>
                <div className="space-y-2">
                  <div className="w-full h-3 bg-gray-200 dark:bg-gray-700 rounded" />
                  <div className="w-3/4 h-3 bg-gray-200 dark:bg-gray-700 rounded" />
                </div>
                <div className="grid grid-cols-3 gap-2">
                  <div className="text-center space-y-1">
                    <div className="w-8 h-3 bg-gray-200 dark:bg-gray-700 rounded mx-auto" />
                    <div className="w-6 h-4 bg-gray-200 dark:bg-gray-700 rounded mx-auto" />
                  </div>
                  <div className="text-center space-y-1">
                    <div className="w-8 h-3 bg-gray-200 dark:bg-gray-700 rounded mx-auto" />
                    <div className="w-6 h-4 bg-gray-200 dark:bg-gray-700 rounded mx-auto" />
                  </div>
                  <div className="text-center space-y-1">
                    <div className="w-8 h-3 bg-gray-200 dark:bg-gray-700 rounded mx-auto" />
                    <div className="w-6 h-4 bg-gray-200 dark:bg-gray-700 rounded mx-auto" />
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    );
  }

  if (assets.length === 0) {
    return (
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className="text-center py-12"
      >
        <ComputerDesktopIcon className="mx-auto h-12 w-12 text-gray-400" />
        <h3 className="mt-2 text-sm font-medium text-gray-900 dark:text-white">No assets found</h3>
        <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
          No assets match your current filters. Try adjusting your search criteria.
        </p>
      </motion.div>
    );
  }

  return (
    <div className="space-y-4">
      {/* Select All Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-2">
          <input
            type="checkbox"
            checked={selectedAssets.length === assets.length && assets.length > 0}
            onChange={handleSelectAll}
            className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
          />
          <span className="text-sm text-gray-700 dark:text-gray-300">
            {selectedAssets.length > 0 
              ? `${selectedAssets.length} of ${assets.length} selected`
              : `Select all ${assets.length} asset${assets.length !== 1 ? 's' : ''}`
            }
          </span>
        </div>
        
        {selectedAssets.length > 0 && (
          <button
            onClick={() => onSelectionChange([])}
            className="text-sm text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-300"
          >
            Clear selection
          </button>
        )}
      </div>

      {/* Asset Grid */}
      <motion.div 
        layout
        className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6"
      >
        <AnimatePresence>
          {assets.map((asset) => (
            <AssetCard
              key={asset.id}
              asset={asset}
              selected={selectedAssets.includes(asset.id)}
              onSelectionChange={(selected) => handleAssetSelection(asset.id, selected)}
              onAssetUpdate={(updates) => onAssetUpdate(asset.id, updates)}
            />
          ))}
        </AnimatePresence>
      </motion.div>
    </div>
  );
}