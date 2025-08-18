import React from 'react';
import { SunIcon, MoonIcon } from '@heroicons/react/24/outline';
import { motion } from 'framer-motion';
import { useTheme } from '../../contexts/ThemeContext';

interface ThemeSwitcherProps {
  className?: string;
  size?: 'sm' | 'md' | 'lg';
}

export const ThemeSwitcher: React.FC<ThemeSwitcherProps> = ({ 
  className = '', 
  size = 'md' 
}) => {
  const { theme, toggleTheme } = useTheme();

  const sizeClasses = {
    sm: 'w-12 h-6',
    md: 'w-14 h-7',
    lg: 'w-16 h-8'
  };

  const iconSizeClasses = {
    sm: 'w-4 h-4',
    md: 'w-5 h-5',
    lg: 'w-6 h-6'
  };

  return (
    <motion.button
      onClick={toggleTheme}
      className={`
        relative inline-flex items-center justify-center
        ${sizeClasses[size]}
        bg-gray-200 dark:bg-gray-700
        rounded-full
        transition-colors duration-200 ease-in-out
        focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2
        dark:focus:ring-offset-gray-800
        hover:bg-gray-300 dark:hover:bg-gray-600
        ${className}
      `}
      whileTap={{ scale: 0.95 }}
      initial={false}
      title={`Switch to ${theme === 'light' ? 'dark' : 'light'} mode`}
    >
      <motion.div
        className={`
          absolute inset-1 
          bg-white dark:bg-gray-900 
          rounded-full 
          shadow-md
          flex items-center justify-center
          ${theme === 'light' ? 'left-1' : 'right-1'}
        `}
        layout
        transition={{
          type: "spring",
          stiffness: 700,
          damping: 30
        }}
      >
        <motion.div
          key={theme}
          initial={{ rotate: -180, opacity: 0 }}
          animate={{ rotate: 0, opacity: 1 }}
          exit={{ rotate: 180, opacity: 0 }}
          transition={{ duration: 0.2 }}
          className="flex items-center justify-center"
        >
          {theme === 'light' ? (
            <SunIcon className={`${iconSizeClasses[size]} text-yellow-500`} />
          ) : (
            <MoonIcon className={`${iconSizeClasses[size]} text-blue-400`} />
          )}
        </motion.div>
      </motion.div>
      
      {/* Background icons for visual context */}
      <div className="absolute inset-0 flex items-center justify-between px-2">
        <SunIcon 
          className={`
            ${iconSizeClasses[size]} 
            ${theme === 'light' ? 'text-transparent' : 'text-gray-400 dark:text-gray-500'}
            transition-colors duration-200
          `} 
        />
        <MoonIcon 
          className={`
            ${iconSizeClasses[size]} 
            ${theme === 'dark' ? 'text-transparent' : 'text-gray-400 dark:text-gray-500'}
            transition-colors duration-200
          `} 
        />
      </div>
    </motion.button>
  );
};

export default ThemeSwitcher;