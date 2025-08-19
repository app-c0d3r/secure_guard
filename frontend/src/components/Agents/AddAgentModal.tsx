import { Fragment, useState } from 'react'
import { Dialog, Transition } from '@headlessui/react'
import { 
  XMarkIcon,
  ComputerDesktopIcon,
  DocumentDuplicateIcon,
  InformationCircleIcon
} from '@heroicons/react/24/outline'
import { motion } from 'framer-motion'

interface AddAgentModalProps {
  isOpen: boolean
  onClose: () => void
}

export default function AddAgentModal({ isOpen, onClose }: AddAgentModalProps) {
  const [step, setStep] = useState(1)
  const [agentName, setAgentName] = useState('')
  const [description, setDescription] = useState('')
  const [selectedPlatform, setSelectedPlatform] = useState('windows')
  const [apiKey] = useState('sg_12345678-1234-1234-1234-123456789abc')

  const platforms = [
    {
      id: 'windows',
      name: 'Windows',
      description: 'Windows 10/11, Windows Server 2016+',
      icon: '🪟'
    },
    {
      id: 'linux',
      name: 'Linux',
      description: 'Ubuntu, CentOS, RHEL, Debian',
      icon: '🐧',
      disabled: true
    },
    {
      id: 'macos',
      name: 'macOS',
      description: 'macOS 10.15+',
      icon: '🍎',
      disabled: true
    }
  ]

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (step === 1) {
      setStep(2)
    } else {
      // Handle agent creation
      onClose()
      setStep(1)
      setAgentName('')
      setDescription('')
    }
  }

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text)
  }

  return (
    <Transition appear show={isOpen} as={Fragment}>
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
              <Dialog.Panel className="w-full max-w-2xl transform overflow-hidden rounded-2xl bg-white p-6 text-left align-middle shadow-xl transition-all">
                <div className="flex items-center justify-between mb-6">
                  <Dialog.Title className="text-2xl font-bold text-secondary-900">
                    Neuen Agent hinzufügen
                  </Dialog.Title>
                  <button
                    onClick={onClose}
                    className="p-2 text-secondary-400 hover:text-secondary-600 transition-colors"
                  >
                    <XMarkIcon className="h-6 w-6" />
                  </button>
                </div>

                {/* Step Indicator */}
                <div className="flex items-center justify-center mb-8">
                  <div className="flex items-center space-x-4">
                    <div className={`flex items-center justify-center w-8 h-8 rounded-full ${
                      step >= 1 ? 'bg-primary-600 text-white' : 'bg-secondary-200 text-secondary-500'
                    }`}>
                      1
                    </div>
                    <div className={`w-16 h-1 ${
                      step >= 2 ? 'bg-primary-600' : 'bg-secondary-200'
                    }`} />
                    <div className={`flex items-center justify-center w-8 h-8 rounded-full ${
                      step >= 2 ? 'bg-primary-600 text-white' : 'bg-secondary-200 text-secondary-500'
                    }`}>
                      2
                    </div>
                  </div>
                </div>

                <form onSubmit={handleSubmit}>
                  {step === 1 && (
                    <motion.div
                      initial={{ opacity: 0, x: -20 }}
                      animate={{ opacity: 1, x: 0 }}
                      className="space-y-6"
                    >
                      <div>
                        <label className="block text-sm font-medium text-secondary-700 mb-2">
                          Agent Name *
                        </label>
                        <input
                          type="text"
                          value={agentName}
                          onChange={(e) => setAgentName(e.target.value)}
                          placeholder="z.B. WS-001, Server-DB-01"
                          className="input"
                          required
                        />
                      </div>

                      <div>
                        <label className="block text-sm font-medium text-secondary-700 mb-2">
                          Beschreibung (Optional)
                        </label>
                        <textarea
                          value={description}
                          onChange={(e) => setDescription(e.target.value)}
                          placeholder="Kurze Beschreibung des Systems..."
                          rows={3}
                          className="input"
                        />
                      </div>

                      <div>
                        <label className="block text-sm font-medium text-secondary-700 mb-4">
                          Betriebssystem *
                        </label>
                        <div className="grid grid-cols-1 gap-3">
                          {platforms.map((platform) => (
                            <div
                              key={platform.id}
                              className={`relative ${platform.disabled ? 'opacity-50' : ''}`}
                            >
                              <input
                                type="radio"
                                id={platform.id}
                                name="platform"
                                value={platform.id}
                                checked={selectedPlatform === platform.id}
                                onChange={(e) => setSelectedPlatform(e.target.value)}
                                disabled={platform.disabled}
                                className="sr-only"
                              />
                              <label
                                htmlFor={platform.id}
                                className={`flex items-center p-4 border-2 rounded-lg cursor-pointer transition-all ${
                                  selectedPlatform === platform.id
                                    ? 'border-primary-500 bg-primary-50'
                                    : 'border-secondary-200 hover:border-secondary-300'
                                } ${platform.disabled ? 'cursor-not-allowed' : ''}`}
                              >
                                <div className="flex items-center space-x-4">
                                  <span className="text-2xl">{platform.icon}</span>
                                  <div>
                                    <h3 className="font-semibold text-secondary-900">
                                      {platform.name}
                                      {platform.disabled && (
                                        <span className="ml-2 text-xs bg-secondary-100 text-secondary-600 px-2 py-1 rounded">
                                          Bald verfügbar
                                        </span>
                                      )}
                                    </h3>
                                    <p className="text-sm text-secondary-600">
                                      {platform.description}
                                    </p>
                                  </div>
                                </div>
                              </label>
                            </div>
                          ))}
                        </div>
                      </div>
                    </motion.div>
                  )}

                  {step === 2 && (
                    <motion.div
                      initial={{ opacity: 0, x: 20 }}
                      animate={{ opacity: 1, x: 0 }}
                      className="space-y-6"
                    >
                      <div className="text-center py-6">
                        <ComputerDesktopIcon className="mx-auto icon-3xl text-primary-600 mb-4" />
                        <h3 className="text-lg font-semibold text-secondary-900 mb-2">
                          Agent Installation
                        </h3>
                        <p className="text-secondary-600">
                          Führen Sie die folgenden Schritte auf dem Zielsystem aus:
                        </p>
                      </div>

                      <div className="bg-secondary-50 rounded-lg p-6">
                        <h4 className="font-semibold text-secondary-900 mb-4">
                          1. Agent herunterladen
                        </h4>
                        <div className="flex items-center space-x-3">
                          <button
                            type="button"
                            className="btn-primary"
                            onClick={() => {
                              // Handle download
                              const link = document.createElement('a')
                              link.href = '/api/agents/download/windows'
                              link.download = 'secureguard-agent-windows.msi'
                              link.click()
                            }}
                          >
                            SecureGuard-Agent.msi herunterladen
                          </button>
                          <span className="text-sm text-secondary-500">
                            v1.2.3 (Windows x64)
                          </span>
                        </div>
                      </div>

                      <div className="bg-secondary-50 rounded-lg p-6">
                        <h4 className="font-semibold text-secondary-900 mb-4">
                          2. Installation mit API-Schlüssel
                        </h4>
                        <div className="space-y-3">
                          <div>
                            <label className="block text-sm font-medium text-secondary-700 mb-2">
                              API-Schlüssel für {agentName}:
                            </label>
                            <div className="flex items-center space-x-2">
                              <code className="flex-1 px-3 py-2 bg-white border border-secondary-200 rounded text-sm font-mono">
                                {apiKey}
                              </code>
                              <button
                                type="button"
                                onClick={() => copyToClipboard(apiKey)}
                                className="btn-icon"
                              >
                                <DocumentDuplicateIcon className="h-4 w-4" />
                              </button>
                            </div>
                          </div>
                          
                          <div>
                            <label className="block text-sm font-medium text-secondary-700 mb-2">
                              Installationsbefehl:
                            </label>
                            <div className="flex items-center space-x-2">
                              <code className="flex-1 px-3 py-2 bg-white border border-secondary-200 rounded text-sm font-mono">
                                msiexec /i SecureGuard-Agent.msi API_KEY={apiKey} /quiet
                              </code>
                              <button
                                type="button"
                                onClick={() => copyToClipboard(`msiexec /i SecureGuard-Agent.msi API_KEY=${apiKey} /quiet`)}
                                className="btn-icon"
                              >
                                <DocumentDuplicateIcon className="h-4 w-4" />
                              </button>
                            </div>
                          </div>
                        </div>
                      </div>

                      <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
                        <div className="flex items-start space-x-3">
                          <InformationCircleIcon className="h-5 w-5 text-blue-600 mt-0.5" />
                          <div className="text-sm text-blue-800">
                            <p className="font-medium mb-1">Wichtige Hinweise:</p>
                            <ul className="list-disc list-inside space-y-1">
                              <li>Der Agent benötigt Administratorrechte für die Installation</li>
                              <li>Nach der Installation wird der Agent automatisch gestartet</li>
                              <li>Die erste Verbindung kann 1-2 Minuten dauern</li>
                              <li>Der API-Schlüssel ist eindeutig für diesen Agent</li>
                            </ul>
                          </div>
                        </div>
                      </div>
                    </motion.div>
                  )}

                  <div className="flex items-center justify-between pt-6 mt-6 border-t border-secondary-100">
                    {step === 2 && (
                      <button
                        type="button"
                        onClick={() => setStep(1)}
                        className="btn-secondary"
                      >
                        Zurück
                      </button>
                    )}
                    
                    <div className="flex items-center space-x-3 ml-auto">
                      <button
                        type="button"
                        onClick={onClose}
                        className="btn-secondary"
                      >
                        Abbrechen
                      </button>
                      <button
                        type="submit"
                        className="btn-primary"
                        disabled={!agentName.trim()}
                      >
                        {step === 1 ? 'Weiter' : 'Agent erstellen'}
                      </button>
                    </div>
                  </div>
                </form>
              </Dialog.Panel>
            </Transition.Child>
          </div>
        </div>
      </Dialog>
    </Transition>
  )
}