import { motion } from 'framer-motion'
import { 
  AreaChart, 
  Area, 
  XAxis, 
  YAxis, 
  CartesianGrid, 
  Tooltip, 
  ResponsiveContainer 
} from 'recharts'

const data = [
  { name: 'Mo', critical: 2, high: 5, medium: 12, low: 8 },
  { name: 'Di', critical: 1, high: 8, medium: 15, low: 12 },
  { name: 'Mi', critical: 3, high: 6, medium: 10, low: 6 },
  { name: 'Do', critical: 0, high: 4, medium: 8, low: 9 },
  { name: 'Fr', critical: 1, high: 7, medium: 14, low: 11 },
  { name: 'Sa', critical: 0, high: 2, medium: 5, low: 3 },
  { name: 'So', critical: 1, high: 3, medium: 7, low: 4 },
]

export default function SecurityIncidentsChart() {
  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.95 }}
      animate={{ opacity: 1, scale: 1 }}
      className="card"
    >
      <div className="card-header">
        <h3 className="text-lg font-semibold text-secondary-900">
          Sicherheitsvorfälle (7 Tage)
        </h3>
        <p className="text-sm text-secondary-500">
          Trend der Sicherheitsereignisse nach Schweregrad
        </p>
      </div>
      
      <div className="card-body">
        <div className="h-80">
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart data={data}>
              <defs>
                <linearGradient id="critical" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#ef4444" stopOpacity={0.8}/>
                  <stop offset="95%" stopColor="#ef4444" stopOpacity={0.1}/>
                </linearGradient>
                <linearGradient id="high" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#f59e0b" stopOpacity={0.8}/>
                  <stop offset="95%" stopColor="#f59e0b" stopOpacity={0.1}/>
                </linearGradient>
                <linearGradient id="medium" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.8}/>
                  <stop offset="95%" stopColor="#3b82f6" stopOpacity={0.1}/>
                </linearGradient>
                <linearGradient id="low" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor="#94a3b8" stopOpacity={0.8}/>
                  <stop offset="95%" stopColor="#94a3b8" stopOpacity={0.1}/>
                </linearGradient>
              </defs>
              <XAxis 
                dataKey="name" 
                axisLine={false}
                tickLine={false}
                tick={{ fontSize: 12, fill: '#64748b' }}
              />
              <YAxis 
                axisLine={false}
                tickLine={false}
                tick={{ fontSize: 12, fill: '#64748b' }}
              />
              <CartesianGrid strokeDasharray="3 3" stroke="#e2e8f0" />
              <Tooltip 
                contentStyle={{
                  backgroundColor: 'white',
                  border: '1px solid #e2e8f0',
                  borderRadius: '8px',
                  boxShadow: '0 10px 15px -3px rgba(0, 0, 0, 0.1)',
                }}
              />
              <Area
                type="monotone"
                dataKey="critical"
                stackId="1"
                stroke="#ef4444"
                fill="url(#critical)"
                strokeWidth={2}
              />
              <Area
                type="monotone"
                dataKey="high"
                stackId="1"
                stroke="#f59e0b"
                fill="url(#high)"
                strokeWidth={2}
              />
              <Area
                type="monotone"
                dataKey="medium"
                stackId="1"
                stroke="#3b82f6"
                fill="url(#medium)"
                strokeWidth={2}
              />
              <Area
                type="monotone"
                dataKey="low"
                stackId="1"
                stroke="#94a3b8"
                fill="url(#low)"
                strokeWidth={2}
              />
            </AreaChart>
          </ResponsiveContainer>
        </div>

        {/* Legend */}
        <div className="flex items-center justify-center space-x-6 mt-4 pt-4 border-t border-secondary-100">
          <div className="flex items-center space-x-2">
            <div className="w-3 h-3 rounded-full bg-danger-500" />
            <span className="text-xs text-secondary-600">Kritisch</span>
          </div>
          <div className="flex items-center space-x-2">
            <div className="w-3 h-3 rounded-full bg-warning-500" />
            <span className="text-xs text-secondary-600">Hoch</span>
          </div>
          <div className="flex items-center space-x-2">
            <div className="w-3 h-3 rounded-full bg-primary-500" />
            <span className="text-xs text-secondary-600">Mittel</span>
          </div>
          <div className="flex items-center space-x-2">
            <div className="w-3 h-3 rounded-full bg-secondary-400" />
            <span className="text-xs text-secondary-600">Niedrig</span>
          </div>
        </div>
      </div>
    </motion.div>
  )
}