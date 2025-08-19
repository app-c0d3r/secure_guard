import { useState } from 'react'
import { motion } from 'framer-motion'
import { 
  CogIcon,
  ShieldCheckIcon,
  BellIcon,
  UserIcon,
  GlobeAltIcon,
  CircleStackIcon as DatabaseIcon,
  KeyIcon,
  EyeIcon,
  PencilIcon,
  LockClosedIcon
} from '@heroicons/react/24/outline'
import RolePermissions from '@/pages/Admin/RolePermissions'
import PasswordChangeModal from '@/components/Security/PasswordChangeModal'

export default function Settings() {
  const [activeTab, setActiveTab] = useState('general')
  const [showPasswordModal, setShowPasswordModal] = useState(false)

  const tabs = [
    { id: 'general', name: 'Allgemein', icon: CogIcon },
    { id: 'security', name: 'Sicherheit', icon: ShieldCheckIcon },
    { id: 'notifications', name: 'Benachrichtigungen', icon: BellIcon },
    { id: 'roles', name: 'Rollen & Berechtigungen', icon: UserIcon },
    { id: 'integrations', name: 'Integrationen', icon: GlobeAltIcon },
    { id: 'database', name: 'Datenbank', icon: DatabaseIcon },
    { id: 'api', name: 'API Einstellungen', icon: KeyIcon }
  ]

  const renderTabContent = () => {
    switch (activeTab) {
      case 'roles':
        return <RolePermissions />
      
      case 'general':
        return (
          <div className="space-y-6">
            <div className="card">
              <div className="card-header">
                <h3 className="text-lg font-semibold text-secondary-900">Systemeinstellungen</h3>
                <p className="text-sm text-secondary-500">Allgemeine Konfiguration des Systems</p>
              </div>
              <div className="card-body space-y-4">
                <div>
                  <label className="block text-sm font-medium text-secondary-700 mb-2">
                    Organisationsname
                  </label>
                  <input
                    type="text"
                    defaultValue="SecureGuard Organization"
                    className="input"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-secondary-700 mb-2">
                    System-URL
                  </label>
                  <input
                    type="url"
                    defaultValue="https://secureguard.company.com"
                    className="input"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-secondary-700 mb-2">
                    Zeitzone
                  </label>
                  <select className="input">
                    <option>Europe/Berlin</option>
                    <option>Europe/London</option>
                    <option>America/New_York</option>
                  </select>
                </div>
              </div>
            </div>
          </div>
        )
      
      case 'security':
        return (
          <div className="space-y-6">
            <div className="card">
              <div className="card-header">
                <h3 className="text-lg font-semibold text-secondary-900">Sicherheitsrichtlinien</h3>
                <p className="text-sm text-secondary-500">Konfiguration der Sicherheitseinstellungen</p>
              </div>
              <div className="card-body space-y-4">
                <div className="flex items-center justify-between">
                  <div>
                    <h4 className="font-medium text-secondary-900">Zwei-Faktor-Authentifizierung</h4>
                    <p className="text-sm text-secondary-500">Erforderlich für alle Benutzer</p>
                  </div>
                  <input type="checkbox" defaultChecked className="toggle" />
                </div>
                <div className="flex items-center justify-between">
                  <div>
                    <h4 className="font-medium text-secondary-900">Session-Timeout</h4>
                    <p className="text-sm text-secondary-500">Automatische Abmeldung nach Inaktivität</p>
                  </div>
                  <select className="input w-auto">
                    <option>30 Minuten</option>
                    <option>1 Stunde</option>
                    <option>4 Stunden</option>
                    <option>8 Stunden</option>
                  </select>
                </div>
                <div className="flex items-center justify-between">
                  <div>
                    <h4 className="font-medium text-secondary-900">Passwort ändern</h4>
                    <p className="text-sm text-secondary-500">Aktuelles Passwort ändern</p>
                  </div>
                  <button 
                    onClick={() => setShowPasswordModal(true)}
                    className="btn-secondary flex items-center space-x-2"
                  >
                    <LockClosedIcon className="w-4 h-4" />
                    <span>Passwort ändern</span>
                  </button>
                </div>
                <div className="flex items-center justify-between">
                  <div>
                    <h4 className="font-medium text-secondary-900">Passwort-Komplexität</h4>
                    <p className="text-sm text-secondary-500">Mindestanforderungen für Passwörter</p>
                  </div>
                  <select className="input w-auto">
                    <option>Hoch (12+ Zeichen, Groß-/Kleinbuchstaben, Zahlen, Sonderzeichen)</option>
                    <option>Mittel (8+ Zeichen, Gemischt)</option>
                    <option>Niedrig (6+ Zeichen)</option>
                  </select>
                </div>
              </div>
            </div>
          </div>
        )
      
      case 'notifications':
        return (
          <div className="space-y-6">
            <div className="card">
              <div className="card-header">
                <h3 className="text-lg font-semibold text-secondary-900">Benachrichtigungseinstellungen</h3>
                <p className="text-sm text-secondary-500">Verwalten Sie E-Mail und System-Benachrichtigungen</p>
              </div>
              <div className="card-body space-y-4">
                <div className="flex items-center justify-between">
                  <div>
                    <h4 className="font-medium text-secondary-900">Kritische Sicherheitsvorfälle</h4>
                    <p className="text-sm text-secondary-500">Sofortige E-Mail-Benachrichtigung</p>
                  </div>
                  <input type="checkbox" defaultChecked className="toggle" />
                </div>
                <div className="flex items-center justify-between">
                  <div>
                    <h4 className="font-medium text-secondary-900">Agent-Ausfälle</h4>
                    <p className="text-sm text-secondary-500">Benachrichtigung bei Agent-Verbindungsverlusten</p>
                  </div>
                  <input type="checkbox" defaultChecked className="toggle" />
                </div>
                <div className="flex items-center justify-between">
                  <div>
                    <h4 className="font-medium text-secondary-900">Wöchentliche Berichte</h4>
                    <p className="text-sm text-secondary-500">Automatische Zusammenfassung per E-Mail</p>
                  </div>
                  <input type="checkbox" className="toggle" />
                </div>
              </div>
            </div>
          </div>
        )
      
      case 'integrations':
        return (
          <div className="space-y-6">
            <div className="card">
              <div className="card-header">
                <h3 className="text-lg font-semibold text-secondary-900">Externe Integrationen</h3>
                <p className="text-sm text-secondary-500">Verbindungen zu externen Systemen</p>
              </div>
              <div className="card-body space-y-4">
                <div className="border border-secondary-200 rounded-lg p-4">
                  <div className="flex items-center justify-between">
                    <div>
                      <h4 className="font-medium text-secondary-900">Slack Integration</h4>
                      <p className="text-sm text-secondary-500">Sende Benachrichtigungen an Slack</p>
                    </div>
                    <button className="btn-secondary">Konfigurieren</button>
                  </div>
                </div>
                <div className="border border-secondary-200 rounded-lg p-4">
                  <div className="flex items-center justify-between">
                    <div>
                      <h4 className="font-medium text-secondary-900">Microsoft Teams</h4>
                      <p className="text-sm text-secondary-500">Teams-Benachrichtigungen</p>
                    </div>
                    <button className="btn-secondary">Konfigurieren</button>
                  </div>
                </div>
                <div className="border border-secondary-200 rounded-lg p-4">
                  <div className="flex items-center justify-between">
                    <div>
                      <h4 className="font-medium text-secondary-900">SIEM Integration</h4>
                      <p className="text-sm text-secondary-500">Export zu SIEM-Systemen</p>
                    </div>
                    <button className="btn-secondary">Konfigurieren</button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )
      
      case 'database':
        return (
          <div className="space-y-6">
            <div className="card">
              <div className="card-header">
                <h3 className="text-lg font-semibold text-secondary-900">Datenbankeinstellungen</h3>
                <p className="text-sm text-secondary-500">Konfiguration und Wartung der Datenbank</p>
              </div>
              <div className="card-body space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium text-secondary-700 mb-2">
                      Backup-Intervall
                    </label>
                    <select className="input">
                      <option>Täglich</option>
                      <option>Wöchentlich</option>
                      <option>Monatlich</option>
                    </select>
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-secondary-700 mb-2">
                      Aufbewahrungszeit
                    </label>
                    <select className="input">
                      <option>30 Tage</option>
                      <option>90 Tage</option>
                      <option>1 Jahr</option>
                    </select>
                  </div>
                </div>
                <div className="flex items-center space-x-3">
                  <button className="btn-primary">Backup erstellen</button>
                  <button className="btn-secondary">Wartung ausführen</button>
                </div>
              </div>
            </div>
          </div>
        )
      
      case 'api':
        return (
          <div className="space-y-6">
            <div className="card">
              <div className="card-header">
                <h3 className="text-lg font-semibold text-secondary-900">API-Konfiguration</h3>
                <p className="text-sm text-secondary-500">Verwalten Sie API-Schlüssel und Zugriffsrechte</p>
              </div>
              <div className="card-body space-y-4">
                <div className="border border-secondary-200 rounded-lg p-4">
                  <div className="flex items-center justify-between">
                    <div>
                      <h4 className="font-medium text-secondary-900">Master API Key</h4>
                      <p className="text-sm text-secondary-500">sg_master_****-****-****-****</p>
                    </div>
                    <div className="flex items-center space-x-2">
                      <button className="btn-icon">
                        <EyeIcon className="h-4 w-4" />
                      </button>
                      <button className="btn-icon">
                        <PencilIcon className="h-4 w-4" />
                      </button>
                    </div>
                  </div>
                </div>
                <div>
                  <label className="block text-sm font-medium text-secondary-700 mb-2">
                    Rate Limiting
                  </label>
                  <select className="input">
                    <option>1000 Requests/Stunde</option>
                    <option>5000 Requests/Stunde</option>
                    <option>10000 Requests/Stunde</option>
                    <option>Unbegrenzt</option>
                  </select>
                </div>
              </div>
            </div>
          </div>
        )
      
      default:
        return <div>Tab content not implemented</div>
    }
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
          <h1 className="text-3xl font-bold text-secondary-900">Einstellungen</h1>
          <p className="text-secondary-600 mt-1">
            Systemkonfiguration und Verwaltung
          </p>
        </div>
      </motion.div>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
        {/* Sidebar */}
        <div className="lg:col-span-1">
          <div className="card">
            <div className="card-body p-0">
              <nav className="space-y-1">
                {tabs.map((tab) => {
                  const Icon = tab.icon
                  return (
                    <button
                      key={tab.id}
                      onClick={() => setActiveTab(tab.id)}
                      className={`w-full flex items-center space-x-3 px-4 py-3 text-left transition-colors ${
                        activeTab === tab.id
                          ? 'bg-primary-50 text-primary-600 border-r-2 border-primary-600'
                          : 'text-secondary-700 hover:bg-secondary-50'
                      }`}
                    >
                      <Icon className="h-5 w-5" />
                      <span className="font-medium">{tab.name}</span>
                    </button>
                  )
                })}
              </nav>
            </div>
          </div>
        </div>

        {/* Content */}
        <div className="lg:col-span-3">
          <motion.div
            key={activeTab}
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.2 }}
          >
            {renderTabContent()}
          </motion.div>
        </div>
      </div>
      
      {/* Password Change Modal */}
      <PasswordChangeModal
        isOpen={showPasswordModal}
        onClose={() => setShowPasswordModal(false)}
        isRequired={false}
        onSuccess={() => setShowPasswordModal(false)}
      />
    </div>
  )
}