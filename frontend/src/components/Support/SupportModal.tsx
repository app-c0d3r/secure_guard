import { Fragment, useState } from 'react'
import { Dialog, Transition } from '@headlessui/react'
import { 
  XMarkIcon,
  ChatBubbleLeftRightIcon,
  ExclamationTriangleIcon,
  InformationCircleIcon,
  BugAntIcon,
  LightBulbIcon,
  PaperAirplaneIcon
} from '@heroicons/react/24/outline'
import { motion } from 'framer-motion'
import { toast } from 'react-hot-toast'
import { useAuthStore } from '@/stores/authStore'

interface SupportModalProps {
  isOpen: boolean
  onClose: () => void
}

const supportCategories = [
  {
    id: 'bug',
    name: 'Fehler melden',
    description: 'Ein Problem oder Fehler in der Anwendung',
    icon: BugAntIcon,
    color: 'text-danger-600',
    bgColor: 'bg-danger-100',
    priority: 'high'
  },
  {
    id: 'security',
    name: 'Sicherheitsproblem',
    description: 'Sicherheitsbedenken oder verdächtige Aktivitäten',
    icon: ExclamationTriangleIcon,
    color: 'text-warning-600',
    bgColor: 'bg-warning-100',
    priority: 'critical'
  },
  {
    id: 'feature',
    name: 'Feature-Anfrage',
    description: 'Vorschlag für neue Funktionen oder Verbesserungen',
    icon: LightBulbIcon,
    color: 'text-primary-600',
    bgColor: 'bg-primary-100',
    priority: 'medium'
  },
  {
    id: 'question',
    name: 'Allgemeine Frage',
    description: 'Fragen zur Nutzung oder Konfiguration',
    icon: InformationCircleIcon,
    color: 'text-secondary-600',
    bgColor: 'bg-secondary-100',
    priority: 'low'
  },
  {
    id: 'feedback',
    name: 'Feedback',
    description: 'Allgemeines Feedback zur Anwendung',
    icon: ChatBubbleLeftRightIcon,
    color: 'text-success-600',
    bgColor: 'bg-success-100',
    priority: 'low'
  }
]

export default function SupportModal({ isOpen, onClose }: SupportModalProps) {
  const { user } = useAuthStore()
  const [step, setStep] = useState(1)
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null)
  const [subject, setSubject] = useState('')
  const [message, setMessage] = useState('')
  const [urgency, setUrgency] = useState<'low' | 'medium' | 'high' | 'critical'>('medium')
  const [attachments, setAttachments] = useState<File[]>([])
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [includeSystemInfo, setIncludeSystemInfo] = useState(true)
  const [allowFollowUp, setAllowFollowUp] = useState(true)

  const handleClose = () => {
    setStep(1)
    setSelectedCategory(null)
    setSubject('')
    setMessage('')
    setUrgency('medium')
    setAttachments([])
    setIncludeSystemInfo(true)
    setAllowFollowUp(true)
    onClose()
  }

  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(e.target.files || [])
    const validFiles = files.filter(file => {
      // Max 10MB per file
      if (file.size > 10 * 1024 * 1024) {
        toast.error(`Datei ${file.name} ist zu groß (max. 10MB)`)
        return false
      }
      
      // Allowed file types
      const allowedTypes = [
        'image/jpeg', 'image/png', 'image/gif',
        'application/pdf', 'text/plain',
        'application/zip', 'application/x-zip-compressed'
      ]
      
      if (!allowedTypes.includes(file.type)) {
        toast.error(`Dateityp von ${file.name} ist nicht erlaubt`)
        return false
      }
      
      return true
    })
    
    setAttachments(prev => [...prev, ...validFiles].slice(0, 5)) // Max 5 files
  }

  const removeAttachment = (index: number) => {
    setAttachments(prev => prev.filter((_, i) => i !== index))
  }

  const getSystemInfo = () => {
    return {
      userAgent: navigator.userAgent,
      language: navigator.language,
      platform: navigator.platform,
      cookieEnabled: navigator.cookieEnabled,
      onLine: navigator.onLine,
      timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
      screen: `${screen.width}x${screen.height}`,
      window: `${window.innerWidth}x${window.innerHeight}`,
      timestamp: new Date().toISOString(),
      url: window.location.href
    }
  }

  const handleSubmit = async () => {
    if (!selectedCategory || !subject.trim() || !message.trim()) {
      toast.error('Bitte füllen Sie alle erforderlichen Felder aus.')
      return
    }

    if (message.trim().length < 20) {
      toast.error('Die Nachricht sollte mindestens 20 Zeichen lang sein.')
      return
    }

    setIsSubmitting(true)

    try {
      const category = supportCategories.find(c => c.id === selectedCategory)!
      
      const ticketData = {
        id: `TICKET-${Date.now()}`,
        user: {
          id: user?.id,
          email: user?.email,
          name: user?.name,
          role: user?.role
        },
        category: {
          id: selectedCategory,
          name: category.name,
          priority: category.priority
        },
        subject: subject.trim(),
        message: message.trim(),
        urgency,
        systemInfo: includeSystemInfo ? getSystemInfo() : null,
        allowFollowUp,
        attachments: attachments.map(file => ({
          name: file.name,
          size: file.size,
          type: file.type
        })),
        timestamp: new Date().toISOString(),
        status: 'open'
      }

      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 2000))

      // In real implementation, this would send to backend API
      console.log('Support ticket submitted:', ticketData)
      
      // Simulate email notification to support team
      const emailData = {
        to: 'support@secureguard.company.com',
        cc: ['admin@secureguard.company.com'],
        subject: `[${urgency.toUpperCase()}] ${category.name}: ${subject}`,
        body: generateEmailBody(ticketData),
        priority: urgency === 'critical' ? 'high' : urgency === 'high' ? 'high' : 'normal'
      }

      console.log('Email notification:', emailData)

      // Show success message
      toast.success(
        `Support-Anfrage erfolgreich eingereicht! ` +
        `Ticket-ID: ${ticketData.id}. ` +
        `Sie erhalten eine Bestätigung per E-Mail.`
      )

      // Store ticket locally for user reference
      const existingTickets = JSON.parse(localStorage.getItem('support_tickets') || '[]')
      existingTickets.push(ticketData)
      localStorage.setItem('support_tickets', JSON.stringify(existingTickets))

      handleClose()
    } catch (error) {
      toast.error('Fehler beim Senden der Support-Anfrage. Bitte versuchen Sie es erneut.')
    } finally {
      setIsSubmitting(false)
    }
  }

  const generateEmailBody = (ticketData: any) => {
    const category = supportCategories.find(c => c.id === selectedCategory)!
    
    return `
Neue Support-Anfrage eingegangen

Ticket-ID: ${ticketData.id}
Priorität: ${urgency.toUpperCase()}
Kategorie: ${category.name}

Benutzer:
- Name: ${user?.name}
- E-Mail: ${user?.email}
- Rolle: ${user?.role}

Betreff: ${subject}

Nachricht:
${message}

${includeSystemInfo ? `
Systeminformationen:
- Browser: ${navigator.userAgent}
- Sprache: ${navigator.language}
- Zeitzone: ${Intl.DateTimeFormat().resolvedOptions().timeZone}
- Bildschirmauflösung: ${screen.width}x${screen.height}
- URL: ${window.location.href}
` : ''}

${attachments.length > 0 ? `
Anhänge: ${attachments.map(f => f.name).join(', ')}
` : ''}

Follow-up erlaubt: ${allowFollowUp ? 'Ja' : 'Nein'}

Zeitstempel: ${new Date().toLocaleString('de-DE')}
    `.trim()
  }

  return (
    <Transition appear show={isOpen} as={Fragment}>
      <Dialog as="div" className="relative z-50" onClose={handleClose}>
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
          <div className="flex min-h-full items-center justify-center p-4">
            <Transition.Child
              as={Fragment}
              enter="ease-out duration-300"
              enterFrom="opacity-0 scale-95"
              enterTo="opacity-100 scale-100"
              leave="ease-in duration-200"
              leaveFrom="opacity-100 scale-100"
              leaveTo="opacity-0 scale-95"
            >
              <Dialog.Panel className="w-full max-w-2xl transform overflow-hidden rounded-2xl bg-white shadow-xl transition-all">
                <div className="flex items-center justify-between p-6 border-b border-secondary-100">
                  <Dialog.Title className="text-xl font-bold text-secondary-900">
                    Support kontaktieren
                  </Dialog.Title>
                  <button
                    onClick={handleClose}
                    className="p-2 text-secondary-400 hover:text-secondary-600 transition-colors"
                  >
                    <XMarkIcon className="h-6 w-6" />
                  </button>
                </div>

                <div className="p-6">
                  {step === 1 && (
                    <motion.div
                      initial={{ opacity: 0, x: -20 }}
                      animate={{ opacity: 1, x: 0 }}
                      className="space-y-6"
                    >
                      <div>
                        <h3 className="text-lg font-semibold text-secondary-900 mb-4">
                          Wählen Sie eine Kategorie
                        </h3>
                        <div className="grid grid-cols-1 gap-3">
                          {supportCategories.map((category) => {
                            const Icon = category.icon
                            return (
                              <div
                                key={category.id}
                                onClick={() => setSelectedCategory(category.id)}
                                className={`relative cursor-pointer rounded-lg border-2 p-4 transition-all ${
                                  selectedCategory === category.id
                                    ? 'border-primary-500 bg-primary-50'
                                    : 'border-secondary-200 hover:border-secondary-300'
                                }`}
                              >
                                <div className="flex items-start space-x-4">
                                  <div className={`p-2 rounded-lg ${category.bgColor}`}>
                                    <Icon className={`h-6 w-6 ${category.color}`} />
                                  </div>
                                  <div className="flex-1">
                                    <h4 className="font-semibold text-secondary-900">
                                      {category.name}
                                    </h4>
                                    <p className="text-sm text-secondary-600 mt-1">
                                      {category.description}
                                    </p>
                                    <span className={`inline-block mt-2 px-2 py-1 text-xs rounded-full ${
                                      category.priority === 'critical' ? 'bg-danger-100 text-danger-800' :
                                      category.priority === 'high' ? 'bg-warning-100 text-warning-800' :
                                      category.priority === 'medium' ? 'bg-primary-100 text-primary-800' :
                                      'bg-secondary-100 text-secondary-800'
                                    }`}>
                                      Priorität: {category.priority}
                                    </span>
                                  </div>
                                </div>
                              </div>
                            )
                          })}
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
                      <div>
                        <label className="block text-sm font-medium text-secondary-700 mb-2">
                          Betreff *
                        </label>
                        <input
                          type="text"
                          value={subject}
                          onChange={(e) => setSubject(e.target.value)}
                          placeholder="Kurze Beschreibung Ihres Anliegens"
                          className="input"
                          maxLength={200}
                        />
                        <p className="text-xs text-secondary-500 mt-1">
                          {subject.length}/200 Zeichen
                        </p>
                      </div>

                      <div>
                        <label className="block text-sm font-medium text-secondary-700 mb-2">
                          Dringlichkeit
                        </label>
                        <select
                          value={urgency}
                          onChange={(e) => setUrgency(e.target.value as any)}
                          className="input"
                        >
                          <option value="low">Niedrig - Kann warten</option>
                          <option value="medium">Mittel - Normal</option>
                          <option value="high">Hoch - Dringend</option>
                          <option value="critical">Kritisch - Sofort</option>
                        </select>
                      </div>

                      <div>
                        <label className="block text-sm font-medium text-secondary-700 mb-2">
                          Nachricht *
                        </label>
                        <textarea
                          value={message}
                          onChange={(e) => setMessage(e.target.value)}
                          placeholder="Beschreiben Sie Ihr Anliegen so detailliert wie möglich..."
                          rows={6}
                          className="input"
                          maxLength={2000}
                        />
                        <p className="text-xs text-secondary-500 mt-1">
                          {message.length}/2000 Zeichen (mindestens 20)
                        </p>
                      </div>

                      <div>
                        <label className="block text-sm font-medium text-secondary-700 mb-2">
                          Anhänge (optional)
                        </label>
                        <input
                          type="file"
                          multiple
                          onChange={handleFileUpload}
                          accept=".jpg,.jpeg,.png,.gif,.pdf,.txt,.zip"
                          className="input"
                        />
                        <p className="text-xs text-secondary-500 mt-1">
                          Max. 5 Dateien, je 10MB. Erlaubt: JPG, PNG, GIF, PDF, TXT, ZIP
                        </p>
                        
                        {attachments.length > 0 && (
                          <div className="mt-2 space-y-1">
                            {attachments.map((file, index) => (
                              <div key={index} className="flex items-center justify-between p-2 bg-secondary-50 rounded">
                                <span className="text-sm text-secondary-700">
                                  {file.name} ({(file.size / 1024 / 1024).toFixed(1)} MB)
                                </span>
                                <button
                                  onClick={() => removeAttachment(index)}
                                  className="text-danger-600 hover:text-danger-700"
                                >
                                  <XMarkIcon className="h-4 w-4" />
                                </button>
                              </div>
                            ))}
                          </div>
                        )}
                      </div>

                      <div className="space-y-3">
                        <label className="flex items-center">
                          <input
                            type="checkbox"
                            checked={includeSystemInfo}
                            onChange={(e) => setIncludeSystemInfo(e.target.checked)}
                            className="mr-2"
                          />
                          <span className="text-sm text-secondary-700">
                            Systeminformationen zur Fehlerbehebung mitschicken
                          </span>
                        </label>
                        
                        <label className="flex items-center">
                          <input
                            type="checkbox"
                            checked={allowFollowUp}
                            onChange={(e) => setAllowFollowUp(e.target.checked)}
                            className="mr-2"
                          />
                          <span className="text-sm text-secondary-700">
                            Follow-up E-Mails für diese Anfrage erlauben
                          </span>
                        </label>
                      </div>
                    </motion.div>
                  )}
                </div>

                <div className="flex items-center justify-between p-6 border-t border-secondary-100">
                  {step === 2 && (
                    <button
                      onClick={() => setStep(1)}
                      className="btn-secondary"
                    >
                      Zurück
                    </button>
                  )}
                  
                  <div className="flex items-center space-x-3 ml-auto">
                    <button
                      onClick={handleClose}
                      className="btn-secondary"
                    >
                      Abbrechen
                    </button>
                    
                    {step === 1 ? (
                      <button
                        onClick={() => setStep(2)}
                        disabled={!selectedCategory}
                        className="btn-primary"
                      >
                        Weiter
                      </button>
                    ) : (
                      <button
                        onClick={handleSubmit}
                        disabled={isSubmitting || !subject.trim() || !message.trim() || message.length < 20}
                        className="btn-primary flex items-center space-x-2"
                      >
                        <PaperAirplaneIcon className="h-4 w-4" />
                        <span>{isSubmitting ? 'Wird gesendet...' : 'Senden'}</span>
                      </button>
                    )}
                  </div>
                </div>
              </Dialog.Panel>
            </Transition.Child>
          </div>
        </div>
      </Dialog>
    </Transition>
  )
}