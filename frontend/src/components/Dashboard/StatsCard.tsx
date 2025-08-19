import { motion } from 'framer-motion'
import { ArrowUpIcon, ArrowDownIcon } from '@heroicons/react/24/outline'
import { cn } from '@/lib/utils'

interface StatsCardProps {
  title: string
  value: string
  change?: string
  changeType?: 'increase' | 'decrease' | 'neutral'
  icon: React.ComponentType<React.SVGProps<SVGSVGElement>>
  color?: 'primary' | 'success' | 'warning' | 'danger' | 'secondary'
}

const colorClasses = {
  primary: {
    icon: 'bg-primary-100 text-primary-600',
    change: 'text-primary-600',
  },
  success: {
    icon: 'bg-success-100 text-success-600',
    change: 'text-success-600',
  },
  warning: {
    icon: 'bg-warning-100 text-warning-600',
    change: 'text-warning-600',
  },
  danger: {
    icon: 'bg-danger-100 text-danger-600',
    change: 'text-danger-600',
  },
  secondary: {
    icon: 'bg-secondary-100 text-secondary-600',
    change: 'text-secondary-600',
  },
}

export default function StatsCard({
  title,
  value,
  change,
  changeType = 'neutral',
  icon: Icon,
  color = 'primary'
}: StatsCardProps) {
  const colors = colorClasses[color]

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      whileHover={{ y: -2 }}
      className="card hover:shadow-glow transition-all duration-300"
    >
      <div className="card-body">
        <div className="flex items-center justify-between">
          <div className="flex-1">
            <p className="text-sm font-medium text-secondary-500 uppercase tracking-wide">
              {title}
            </p>
            <p className="text-3xl font-bold text-secondary-900 mt-2">
              {value}
            </p>
            
            {change && (
              <div className="flex items-center mt-2">
                {changeType === 'increase' && (
                  <ArrowUpIcon className="h-4 w-4 text-success-500 mr-1" />
                )}
                {changeType === 'decrease' && (
                  <ArrowDownIcon className="h-4 w-4 text-danger-500 mr-1" />
                )}
                <span className={cn(
                  'text-sm font-medium',
                  changeType === 'increase' && 'text-success-600',
                  changeType === 'decrease' && 'text-danger-600',
                  changeType === 'neutral' && 'text-secondary-600'
                )}>
                  {change} diese Woche
                </span>
              </div>
            )}
          </div>
          
          <div className={cn(
            'p-3 rounded-lg',
            colors.icon
          )}>
            <Icon className="icon-xl" />
          </div>
        </div>
      </div>
    </motion.div>
  )
}