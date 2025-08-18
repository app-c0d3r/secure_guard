import { useState, useEffect, useRef } from 'react'
import { motion } from 'framer-motion'
import { ArrowPathIcon } from '@heroicons/react/24/outline'

interface CaptchaComponentProps {
  onVerify: (token: string) => void
  onError?: (error: string) => void
  difficulty?: 'easy' | 'medium' | 'hard'
}

// Simple math CAPTCHA for demo - in production use reCAPTCHA or similar
export default function CaptchaComponent({ 
  onVerify, 
  onError, 
  difficulty = 'medium' 
}: CaptchaComponentProps) {
  const [challenge, setChallenge] = useState<{
    question: string
    answer: number
    token: string
  } | null>(null)
  const [userAnswer, setUserAnswer] = useState('')
  const [isVerified, setIsVerified] = useState(false)
  const [attempts, setAttempts] = useState(0)
  const canvasRef = useRef<HTMLCanvasElement>(null)

  // Generate math challenge based on difficulty
  const generateChallenge = () => {
    let num1: number, num2: number, operation: string, answer: number, question: string

    switch (difficulty) {
      case 'easy':
        num1 = Math.floor(Math.random() * 10) + 1
        num2 = Math.floor(Math.random() * 10) + 1
        operation = Math.random() > 0.5 ? '+' : '-'
        if (operation === '-' && num1 < num2) [num1, num2] = [num2, num1]
        answer = operation === '+' ? num1 + num2 : num1 - num2
        question = `${num1} ${operation} ${num2} = ?`
        break
        
      case 'medium':
        num1 = Math.floor(Math.random() * 20) + 1
        num2 = Math.floor(Math.random() * 20) + 1
        const ops = ['+', '-', '*']
        operation = ops[Math.floor(Math.random() * ops.length)]
        
        if (operation === '-' && num1 < num2) [num1, num2] = [num2, num1]
        if (operation === '*') {
          num1 = Math.floor(Math.random() * 12) + 1
          num2 = Math.floor(Math.random() * 12) + 1
        }
        
        answer = operation === '+' ? num1 + num2 : 
                operation === '-' ? num1 - num2 : num1 * num2
        question = `${num1} ${operation} ${num2} = ?`
        break
        
      case 'hard':
        // More complex operations
        const complexOps = ['square', 'sequence', 'mixed']
        const complexOp = complexOps[Math.floor(Math.random() * complexOps.length)]
        
        if (complexOp === 'square') {
          num1 = Math.floor(Math.random() * 15) + 1
          answer = num1 * num1
          question = `${num1}² = ?`
        } else if (complexOp === 'sequence') {
          num1 = Math.floor(Math.random() * 10) + 2
          const step = Math.floor(Math.random() * 5) + 1
          const seq = [num1, num1 + step, num1 + step * 2]
          answer = num1 + step * 3
          question = `${seq.join(', ')}, ?`
        } else {
          num1 = Math.floor(Math.random() * 15) + 1
          num2 = Math.floor(Math.random() * 8) + 2
          const num3 = Math.floor(Math.random() * 10) + 1
          answer = num1 + num2 * num3
          question = `${num1} + ${num2} × ${num3} = ?`
        }
        break
        
      default:
        num1 = 5
        num2 = 3
        answer = 8
        question = '5 + 3 = ?'
    }

    const token = generateToken()
    setChallenge({ question, answer, token })
    setUserAnswer('')
    drawChallenge(question)
  }

  // Generate random token for verification
  const generateToken = (): string => {
    return Math.random().toString(36).substring(2, 15) + 
           Math.random().toString(36).substring(2, 15)
  }

  // Draw challenge on canvas with noise to prevent OCR
  const drawChallenge = (text: string) => {
    const canvas = canvasRef.current
    if (!canvas) return

    const ctx = canvas.getContext('2d')
    if (!ctx) return

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height)

    // Set background
    ctx.fillStyle = '#f8fafc'
    ctx.fillRect(0, 0, canvas.width, canvas.height)

    // Add noise lines
    ctx.strokeStyle = '#e2e8f0'
    ctx.lineWidth = 1
    for (let i = 0; i < 8; i++) {
      ctx.beginPath()
      ctx.moveTo(Math.random() * canvas.width, Math.random() * canvas.height)
      ctx.lineTo(Math.random() * canvas.width, Math.random() * canvas.height)
      ctx.stroke()
    }

    // Add noise dots
    ctx.fillStyle = '#cbd5e1'
    for (let i = 0; i < 20; i++) {
      ctx.beginPath()
      ctx.arc(
        Math.random() * canvas.width,
        Math.random() * canvas.height,
        Math.random() * 2,
        0,
        2 * Math.PI
      )
      ctx.fill()
    }

    // Draw text with slight rotation and variation
    ctx.fillStyle = '#1e293b'
    ctx.font = 'bold 24px monospace'
    ctx.textAlign = 'center'
    ctx.textBaseline = 'middle'
    
    const x = canvas.width / 2
    const y = canvas.height / 2
    
    // Add slight rotation and position variation
    ctx.save()
    ctx.translate(x, y)
    ctx.rotate((Math.random() - 0.5) * 0.2)
    ctx.fillText(text, 0, 0)
    ctx.restore()
  }

  // Verify user answer
  const verifyAnswer = () => {
    if (!challenge) return

    const answer = parseInt(userAnswer.trim())
    
    if (answer === challenge.answer) {
      setIsVerified(true)
      onVerify(challenge.token)
    } else {
      setAttempts(prev => prev + 1)
      if (attempts >= 2) {
        onError?.('Zu viele fehlgeschlagene CAPTCHA-Versuche. Neue Aufgabe wird generiert.')
        generateChallenge()
        setAttempts(0)
      } else {
        onError?.(`Falsche Antwort. ${2 - attempts} Versuche verbleibend.`)
      }
      setUserAnswer('')
    }
  }

  // Handle Enter key
  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && userAnswer.trim()) {
      verifyAnswer()
    }
  }

  useEffect(() => {
    generateChallenge()
  }, [difficulty])

  if (isVerified) {
    return (
      <motion.div
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        className="bg-success-50 border border-success-200 rounded-lg p-4"
      >
        <div className="flex items-center space-x-2 text-success-800">
          <svg className="h-5 w-5" fill="currentColor" viewBox="0 0 20 20">
            <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
          </svg>
          <span className="font-medium">CAPTCHA erfolgreich verifiziert</span>
        </div>
      </motion.div>
    )
  }

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      className="space-y-4"
    >
      <div className="bg-secondary-50 border border-secondary-200 rounded-lg p-4">
        <div className="flex items-center justify-between mb-3">
          <h3 className="text-sm font-medium text-secondary-900">
            Sicherheitsverifizierung
          </h3>
          <button
            onClick={generateChallenge}
            className="p-1 text-secondary-500 hover:text-secondary-700 transition-colors"
            title="Neue Aufgabe generieren"
          >
            <ArrowPathIcon className="h-4 w-4" />
          </button>
        </div>
        
        <div className="flex items-center space-x-4">
          <canvas
            ref={canvasRef}
            width={200}
            height={60}
            className="border border-secondary-300 rounded bg-white"
          />
          
          <div className="flex-1">
            <div className="flex space-x-2">
              <input
                type="text"
                value={userAnswer}
                onChange={(e) => setUserAnswer(e.target.value.replace(/[^0-9-]/g, ''))}
                onKeyPress={handleKeyPress}
                placeholder="Antwort eingeben"
                className="input flex-1"
                maxLength={10}
              />
              <button
                onClick={verifyAnswer}
                disabled={!userAnswer.trim()}
                className="btn-primary px-4"
              >
                Prüfen
              </button>
            </div>
            
            {attempts > 0 && (
              <p className="text-xs text-danger-600 mt-1">
                {3 - attempts} Versuche verbleibend
              </p>
            )}
          </div>
        </div>
        
        <p className="text-xs text-secondary-500 mt-2">
          Lösen Sie die Rechenaufgabe zur Sicherheitsverifizierung
        </p>
      </div>
    </motion.div>
  )
}