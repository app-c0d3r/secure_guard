import { motion } from 'framer-motion'
import { 
  ComputerDesktopIcon, 
  ShieldCheckIcon, 
  ExclamationTriangleIcon, 
  UsersIcon,
  ArrowUpIcon,
  ArrowDownIcon
} from '@heroicons/react/24/outline'
import StatsCard from '@/components/Dashboard/StatsCard'
import AgentStatusChart from '@/components/Dashboard/AgentStatusChart'
import SecurityIncidentsChart from '@/components/Dashboard/SecurityIncidentsChart'
import RecentIncidents from '@/components/Dashboard/RecentIncidents'
import AgentsList from '@/components/Dashboard/AgentsList'
import SystemHealth from '@/components/Dashboard/SystemHealth'

export default function Dashboard() {
  return (
    <div className="space-y-6">
      {/* Page Header */}
      <motion.div
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
        className="flex justify-between items-center"
      >
        <div>
          <h1 className="text-3xl font-bold text-secondary-900">Dashboard</h1>
          <p className="text-secondary-600 mt-1">
            Überwachen Sie Ihre Sicherheitsinfrastruktur in Echtzeit
          </p>
        </div>
        <div className="flex items-center space-x-2 text-sm text-secondary-500">
          <div className="status-online" />
          <span>Letzte Aktualisierung: vor 2 Minuten</span>
        </div>
      </motion.div>

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <StatsCard
          title="Aktive Agents"
          value="127"
          change="+12"
          changeType="increase"
          icon={ComputerDesktopIcon}
          color="primary"
        />
        <StatsCard
          title="Sicherheitsereignisse"
          value="23"
          change="-5"
          changeType="decrease"
          icon={ShieldCheckIcon}
          color="success"
        />
        <StatsCard
          title="Aktive Vorfälle"
          value="3"
          change="+1"
          changeType="increase"
          icon={ExclamationTriangleIcon}
          color="warning"
        />
        <StatsCard
          title="Registrierte Benutzer"
          value="89"
          change="+7"
          changeType="increase"
          icon={UsersIcon}
          color="secondary"
        />
      </div>

      {/* Charts Row */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <AgentStatusChart />
        <SecurityIncidentsChart />
      </div>

      {/* Main Content Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Recent Incidents */}
        <div className="lg:col-span-2">
          <RecentIncidents />
        </div>

        {/* System Health */}
        <div>
          <SystemHealth />
        </div>
      </div>

      {/* Agents List */}
      <AgentsList />
    </div>
  )
}