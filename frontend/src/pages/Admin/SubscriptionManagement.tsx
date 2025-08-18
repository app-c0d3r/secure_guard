import { useState } from 'react'
import { motion } from 'framer-motion'
import { 
  PlusIcon,
  PencilIcon,
  TrashIcon,
  CheckIcon,
  XMarkIcon,
  CurrencyEuroIcon,
  UsersIcon,
  ComputerDesktopIcon,
  KeyIcon,
  ClockIcon
} from '@heroicons/react/24/outline'
import { cn } from '@/lib/utils'

// Mock data for subscription plans
const subscriptionPlans = [
  {
    id: '1',
    name: 'Free',
    description: 'Für Einzelpersonen und kleine Teams',
    price: 0,
    currency: 'EUR',
    billing: 'month',
    features: {
      agents: 1,
      apiKeys: 1,
      storage: '1 GB',
      support: 'Community',
      sla: null,
      advancedFeatures: false,
      customBranding: false,
      apiAccess: 'Basic'
    },
    limits: {
      maxAgents: 1,
      maxApiKeys: 1,
      maxIncidents: 10,
      maxUsers: 1
    },
    status: 'active',
    userCount: 1247,
    color: 'bg-gray-100 text-gray-800'
  },
  {
    id: '2',
    name: 'Starter',
    description: 'Für kleine Unternehmen',
    price: 29,
    currency: 'EUR',
    billing: 'month',
    features: {
      agents: 5,
      apiKeys: 3,
      storage: '10 GB',
      support: 'Email Support',
      sla: '99.5%',
      advancedFeatures: false,
      customBranding: false,
      apiAccess: 'Standard'
    },
    limits: {
      maxAgents: 5,
      maxApiKeys: 3,
      maxIncidents: 100,
      maxUsers: 3
    },
    status: 'active',
    userCount: 543,
    color: 'bg-blue-100 text-blue-800'
  },
  {
    id: '3',
    name: 'Professional',
    description: 'Für mittelständische Unternehmen',
    price: 99,
    currency: 'EUR',
    billing: 'month',
    features: {
      agents: 25,
      apiKeys: 10,
      storage: '100 GB',
      support: 'Priority Support',
      sla: '99.9%',
      advancedFeatures: true,
      customBranding: true,
      apiAccess: 'Advanced'
    },
    limits: {
      maxAgents: 25,
      maxApiKeys: 10,
      maxIncidents: 1000,
      maxUsers: 10
    },
    status: 'active',
    userCount: 289,
    color: 'bg-green-100 text-green-800'
  },
  {
    id: '4',
    name: 'Enterprise',
    description: 'Für große Organisationen',
    price: 499,
    currency: 'EUR',
    billing: 'month',
    features: {
      agents: 'Unlimited',
      apiKeys: 'Unlimited',
      storage: '1 TB',
      support: '24/7 Dedicated Support',
      sla: '99.99%',
      advancedFeatures: true,
      customBranding: true,
      apiAccess: 'Full API'
    },
    limits: {
      maxAgents: -1, // -1 = unlimited
      maxApiKeys: -1,
      maxIncidents: -1,
      maxUsers: -1
    },
    status: 'active',
    userCount: 67,
    color: 'bg-purple-100 text-purple-800'
  }
]

const featureIcons = {
  agents: ComputerDesktopIcon,
  apiKeys: KeyIcon,
  storage: ClockIcon,
  support: UsersIcon,
  sla: CheckIcon
}

export default function SubscriptionManagement() {
  const [selectedPlan, setSelectedPlan] = useState(subscriptionPlans[0])
  const [showCreateModal, setShowCreateModal] = useState(false)

  const formatFeatureValue = (key: string, value: any) => {
    if (value === -1 || value === 'Unlimited') return 'Unbegrenzt'
    if (typeof value === 'number' && key.includes('max')) return value.toLocaleString()
    return value
  }

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <motion.div
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
        className="flex justify-between items-center"
      >
        <div>
          <h1 className="text-3xl font-bold text-secondary-900">Abo-Verwaltung</h1>
          <p className="text-secondary-600 mt-1">
            Verwalten Sie Abonnement-Pläne, Features und Limits
          </p>
        </div>
        <button 
          onClick={() => setShowCreateModal(true)}
          className="btn-primary"
        >
          <PlusIcon className="h-5 w-5 mr-2" />
          Neuen Plan erstellen
        </button>
      </motion.div>

      {/* Stats Overview */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="card">
          <div className="card-body flex items-center space-x-3">
            <div className="p-2 bg-primary-100 rounded-lg">
              <UsersIcon className="h-6 w-6 text-primary-600" />
            </div>
            <div>
              <p className="text-sm text-secondary-500">Aktive Abos</p>
              <p className="text-xl font-bold text-secondary-900">
                {subscriptionPlans.reduce((sum, plan) => sum + plan.userCount, 0)}
              </p>
            </div>
          </div>
        </div>
        
        <div className="card">
          <div className="card-body flex items-center space-x-3">
            <div className="p-2 bg-success-100 rounded-lg">
              <CurrencyEuroIcon className="h-6 w-6 text-success-600" />
            </div>
            <div>
              <p className="text-sm text-secondary-500">Monatlicher Umsatz</p>
              <p className="text-xl font-bold text-secondary-900">
                €{subscriptionPlans.reduce((sum, plan) => 
                  sum + (plan.price * plan.userCount), 0
                ).toLocaleString()}
              </p>
            </div>
          </div>
        </div>
        
        <div className="card">
          <div className="card-body flex items-center space-x-3">
            <div className="p-2 bg-warning-100 rounded-lg">
              <div className="w-6 h-6 bg-warning-600 rounded" />
            </div>
            <div>
              <p className="text-sm text-secondary-500">Pläne</p>
              <p className="text-xl font-bold text-secondary-900">
                {subscriptionPlans.length}
              </p>
            </div>
          </div>
        </div>
        
        <div className="card">
          <div className="card-body flex items-center space-x-3">
            <div className="p-2 bg-purple-100 rounded-lg">
              <ComputerDesktopIcon className="h-6 w-6 text-purple-600" />
            </div>
            <div>
              <p className="text-sm text-secondary-500">Durchschn. Agents</p>
              <p className="text-xl font-bold text-secondary-900">
                {Math.round(
                  subscriptionPlans.reduce((sum, plan) => 
                    sum + (typeof plan.features.agents === 'number' ? plan.features.agents : 50) * plan.userCount, 0
                  ) / subscriptionPlans.reduce((sum, plan) => sum + plan.userCount, 0)
                )}
              </p>
            </div>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Plans List */}
        <motion.div
          initial={{ opacity: 0, x: -20 }}
          animate={{ opacity: 1, x: 0 }}
          className="space-y-4"
        >
          <h3 className="text-lg font-semibold text-secondary-900">Verfügbare Pläne</h3>
          
          {subscriptionPlans.map((plan, index) => (
            <motion.div
              key={plan.id}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: index * 0.1 }}
              onClick={() => setSelectedPlan(plan)}
              className={cn(
                'card cursor-pointer transition-all hover:shadow-glow',
                selectedPlan.id === plan.id && 'ring-2 ring-primary-500'
              )}
            >
              <div className="card-body">
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center space-x-3 mb-2">
                      <h4 className="text-lg font-semibold text-secondary-900">
                        {plan.name}
                      </h4>
                      <span className={cn('badge', plan.color)}>
                        {plan.userCount} Benutzer
                      </span>
                    </div>
                    
                    <p className="text-secondary-600 mb-3">{plan.description}</p>
                    
                    <div className="flex items-baseline space-x-2">
                      <span className="text-3xl font-bold text-secondary-900">
                        €{plan.price}
                      </span>
                      <span className="text-secondary-500">
                        /{plan.billing === 'month' ? 'Monat' : 'Jahr'}
                      </span>
                    </div>
                    
                    <div className="grid grid-cols-2 gap-4 mt-4">
                      <div>
                        <div className="text-sm text-secondary-500">Agents</div>
                        <div className="font-semibold text-secondary-900">
                          {formatFeatureValue('agents', plan.features.agents)}
                        </div>
                      </div>
                      <div>
                        <div className="text-sm text-secondary-500">API Keys</div>
                        <div className="font-semibold text-secondary-900">
                          {formatFeatureValue('apiKeys', plan.features.apiKeys)}
                        </div>
                      </div>
                    </div>
                  </div>
                  
                  <div className="flex items-center space-x-2 ml-4">
                    <button className="btn-icon">
                      <PencilIcon className="h-4 w-4" />
                    </button>
                    {plan.name !== 'Free' && (
                      <button className="btn-icon text-danger-600 hover:bg-danger-100">
                        <TrashIcon className="h-4 w-4" />
                      </button>
                    )}
                  </div>
                </div>
              </div>
            </motion.div>
          ))}
        </motion.div>

        {/* Plan Details */}
        <motion.div
          initial={{ opacity: 0, x: 20 }}
          animate={{ opacity: 1, x: 0 }}
          className="space-y-6"
        >
          <div className="card">
            <div className="card-header">
              <div className="flex items-center justify-between">
                <div>
                  <h3 className="text-lg font-semibold text-secondary-900">
                    {selectedPlan.name} - Details
                  </h3>
                  <p className="text-sm text-secondary-500">
                    Features und Limitierungen
                  </p>
                </div>
                <span className={cn('badge', selectedPlan.color)}>
                  {selectedPlan.status === 'active' ? 'Aktiv' : 'Inaktiv'}
                </span>
              </div>
            </div>
            
            <div className="card-body space-y-6">
              {/* Pricing */}
              <div>
                <h4 className="font-semibold text-secondary-900 mb-3">Preise</h4>
                <div className="bg-secondary-50 rounded-lg p-4">
                  <div className="flex items-baseline space-x-2">
                    <span className="text-3xl font-bold text-secondary-900">
                      €{selectedPlan.price}
                    </span>
                    <span className="text-secondary-500">
                      pro {selectedPlan.billing === 'month' ? 'Monat' : 'Jahr'}
                    </span>
                  </div>
                  <div className="text-sm text-secondary-600 mt-1">
                    {selectedPlan.userCount} aktive Abonnements
                  </div>
                </div>
              </div>

              {/* Features */}
              <div>
                <h4 className="font-semibold text-secondary-900 mb-3">Features</h4>
                <div className="space-y-3">
                  <div className="flex items-center justify-between p-3 bg-secondary-50 rounded-lg">
                    <div className="flex items-center space-x-2">
                      <ComputerDesktopIcon className="h-5 w-5 text-secondary-600" />
                      <span className="font-medium text-secondary-900">Agents</span>
                    </div>
                    <span className="text-secondary-700">
                      {formatFeatureValue('agents', selectedPlan.features.agents)}
                    </span>
                  </div>
                  
                  <div className="flex items-center justify-between p-3 bg-secondary-50 rounded-lg">
                    <div className="flex items-center space-x-2">
                      <KeyIcon className="h-5 w-5 text-secondary-600" />
                      <span className="font-medium text-secondary-900">API Schlüssel</span>
                    </div>
                    <span className="text-secondary-700">
                      {formatFeatureValue('apiKeys', selectedPlan.features.apiKeys)}
                    </span>
                  </div>
                  
                  <div className="flex items-center justify-between p-3 bg-secondary-50 rounded-lg">
                    <div className="flex items-center space-x-2">
                      <ClockIcon className="h-5 w-5 text-secondary-600" />
                      <span className="font-medium text-secondary-900">Speicher</span>
                    </div>
                    <span className="text-secondary-700">{selectedPlan.features.storage}</span>
                  </div>
                  
                  <div className="flex items-center justify-between p-3 bg-secondary-50 rounded-lg">
                    <div className="flex items-center space-x-2">
                      <UsersIcon className="h-5 w-5 text-secondary-600" />
                      <span className="font-medium text-secondary-900">Support</span>
                    </div>
                    <span className="text-secondary-700">{selectedPlan.features.support}</span>
                  </div>
                  
                  {selectedPlan.features.sla && (
                    <div className="flex items-center justify-between p-3 bg-secondary-50 rounded-lg">
                      <div className="flex items-center space-x-2">
                        <CheckIcon className="h-5 w-5 text-secondary-600" />
                        <span className="font-medium text-secondary-900">SLA</span>
                      </div>
                      <span className="text-secondary-700">{selectedPlan.features.sla}</span>
                    </div>
                  )}
                </div>
              </div>

              {/* Advanced Features */}
              <div>
                <h4 className="font-semibold text-secondary-900 mb-3">Erweiterte Features</h4>
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="text-secondary-700">Erweiterte Features</span>
                    {selectedPlan.features.advancedFeatures ? (
                      <CheckIcon className="h-5 w-5 text-success-600" />
                    ) : (
                      <XMarkIcon className="h-5 w-5 text-secondary-400" />
                    )}
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-secondary-700">Custom Branding</span>
                    {selectedPlan.features.customBranding ? (
                      <CheckIcon className="h-5 w-5 text-success-600" />
                    ) : (
                      <XMarkIcon className="h-5 w-5 text-secondary-400" />
                    )}
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-secondary-700">API Zugang</span>
                    <span className="text-secondary-700">{selectedPlan.features.apiAccess}</span>
                  </div>
                </div>
              </div>

              {/* Limits */}
              <div>
                <h4 className="font-semibold text-secondary-900 mb-3">Limits</h4>
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="text-secondary-700">Max. Agents</span>
                    <span className="text-secondary-700">
                      {formatFeatureValue('maxAgents', selectedPlan.limits.maxAgents)}
                    </span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-secondary-700">Max. API Keys</span>
                    <span className="text-secondary-700">
                      {formatFeatureValue('maxApiKeys', selectedPlan.limits.maxApiKeys)}
                    </span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-secondary-700">Max. Vorfälle</span>
                    <span className="text-secondary-700">
                      {formatFeatureValue('maxIncidents', selectedPlan.limits.maxIncidents)}
                    </span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-secondary-700">Max. Benutzer</span>
                    <span className="text-secondary-700">
                      {formatFeatureValue('maxUsers', selectedPlan.limits.maxUsers)}
                    </span>
                  </div>
                </div>
              </div>

              {/* Actions */}
              <div className="flex items-center space-x-3 pt-4 border-t border-secondary-100">
                <button className="btn-primary">
                  Plan bearbeiten
                </button>
                <button className="btn-secondary">
                  Plan duplizieren
                </button>
              </div>
            </div>
          </div>
        </motion.div>
      </div>
    </div>
  )
}