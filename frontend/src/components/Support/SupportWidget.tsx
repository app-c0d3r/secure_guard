import { useState } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { 
  ChatBubbleLeftRightIcon,
  QuestionMarkCircleIcon,
  XMarkIcon,
  PaperAirplaneIcon
} from '@heroicons/react/24/outline'
import SupportModal from './SupportModal'
import { toast } from 'react-hot-toast'

const quickActions = [
  {
    id: 'faq',
    title: 'Häufige Fragen',
    description: 'Antworten auf die häufigsten Fragen',
    action: () => {
      toast.success('FAQ wird geöffnet...')
      // In real app, navigate to FAQ page
    }
  },
  {
    id: 'docs',
    title: 'Dokumentation',
    description: 'Vollständige Produktdokumentation',
    action: () => {
      toast.success('Dokumentation wird geöffnet...')
      // In real app, open documentation
    }
  },
  {
    id: 'status',
    title: 'System Status',
    description: 'Aktuelle Systemverfügbarkeit',
    action: () => {
      toast.success('Status-Seite wird geöffnet...')
      // In real app, open status page
    }
  }
]

export default function SupportWidget() {
  const [isOpen, setIsOpen] = useState(false)
  const [showSupportModal, setShowSupportModal] = useState(false)

  return (
    <>
      {/* Support Widget Button */}
      <motion.div
        className="fixed bottom-6 right-6 z-40"
        initial={{ scale: 0 }}
        animate={{ scale: 1 }}
        transition={{ delay: 1 }}
      >
        <AnimatePresence>
          {isOpen && (
            <motion.div
              initial={{ opacity: 0, y: 20, scale: 0.95 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              exit={{ opacity: 0, y: 20, scale: 0.95 }}
              className="mb-4 bg-white rounded-2xl shadow-xl border border-secondary-200 w-80"
            >
              <div className="p-6">
                <div className="flex items-center justify-between mb-4">
                  <h3 className="text-lg font-semibold text-secondary-900">
                    Wie können wir helfen?
                  </h3>
                  <button
                    onClick={() => setIsOpen(false)}
                    className="p-1 text-secondary-400 hover:text-secondary-600 transition-colors"
                  >
                    <XMarkIcon className="h-5 w-5" />
                  </button>
                </div>

                <div className="space-y-3">
                  {quickActions.map((action) => (
                    <button
                      key={action.id}
                      onClick={action.action}
                      className="w-full text-left p-3 rounded-lg border border-secondary-200 hover:border-primary-300 hover:bg-primary-50 transition-all"
                    >
                      <div className="font-medium text-secondary-900">
                        {action.title}
                      </div>
                      <div className="text-sm text-secondary-600 mt-1">
                        {action.description}
                      </div>
                    </button>
                  ))}

                  <button
                    onClick={() => {
                      setIsOpen(false)
                      setShowSupportModal(true)
                    }}
                    className="w-full bg-primary-600 text-white p-3 rounded-lg hover:bg-primary-700 transition-colors flex items-center justify-center space-x-2"
                  >
                    <PaperAirplaneIcon className="h-4 w-4" />
                    <span>Support kontaktieren</span>
                  </button>
                </div>

                <div className="mt-4 pt-4 border-t border-secondary-100">
                  <div className="text-xs text-secondary-500 text-center">
                    💬 Durchschnittliche Antwortzeit: 2-4 Stunden
                  </div>
                </div>
              </div>
            </motion.div>
          )}
        </AnimatePresence>

        <motion.button
          onClick={() => setIsOpen(!isOpen)}
          className="w-14 h-14 bg-primary-600 text-white rounded-full shadow-lg hover:bg-primary-700 transition-all flex items-center justify-center"
          whileHover={{ scale: 1.05 }}
          whileTap={{ scale: 0.95 }}
        >
          <AnimatePresence mode="wait">
            {isOpen ? (
              <motion.div
                key="close"
                initial={{ rotate: -90, opacity: 0 }}
                animate={{ rotate: 0, opacity: 1 }}
                exit={{ rotate: 90, opacity: 0 }}
                transition={{ duration: 0.2 }}
              >
                <XMarkIcon className="h-6 w-6" />
              </motion.div>
            ) : (
              <motion.div
                key="chat"
                initial={{ rotate: 90, opacity: 0 }}
                animate={{ rotate: 0, opacity: 1 }}
                exit={{ rotate: -90, opacity: 0 }}
                transition={{ duration: 0.2 }}
              >
                <ChatBubbleLeftRightIcon className="h-6 w-6" />
              </motion.div>
            )}
          </AnimatePresence>
        </motion.button>

        {/* Pulse animation for attention */}
        <motion.div
          className="absolute inset-0 w-14 h-14 bg-primary-600 rounded-full"
          initial={{ scale: 1, opacity: 0.7 }}
          animate={{ 
            scale: [1, 1.2, 1],
            opacity: [0.7, 0, 0.7]
          }}
          transition={{
            duration: 2,
            repeat: Infinity,
            ease: "easeInOut"
          }}
        />
      </motion.div>

      {/* Support Modal */}
      <SupportModal 
        isOpen={showSupportModal}
        onClose={() => setShowSupportModal(false)}
      />
    </>
  )
}