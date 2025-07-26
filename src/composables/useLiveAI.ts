import { ref, Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export interface LiveAISession {
  id: string
  startTime: number
  endTime?: number
  isActive: boolean
}

export interface LiveAIResponse {
  text: string
  timestamp: number
  confidence: number
  sessionId: string
}

export function useLiveAI() {
  const isActive = ref(false)
  const sessionId = ref<string | null>(null)
  const response = ref('')
  const isProcessing = ref(false)
  const error = ref<string | null>(null)
  let streamListener: any = null

  const startLiveAI = async (messages: any[]): Promise<void> => {
    try {
      error.value = null
      isProcessing.value = true
      
      // Create a new session
      const newSessionId = `live-ai-${Date.now()}`
      sessionId.value = newSessionId
      
      // Set up streaming listener for live AI responses
      streamListener = await listen(`ollama-stream-${newSessionId}`, (event: any) => {
        const data = event.payload
        
        if (data.type === 'start') {
          console.log('🚀 Live AI streaming started')
          isProcessing.value = true
          response.value = ''
        } else if (data.type === 'chunk') {
          response.value += data.text
        } else if (data.type === 'complete') {
          console.log('✅ Live AI streaming completed')
          isProcessing.value = false
        } else if (data.type === 'error') {
          console.error('❌ Live AI streaming error:', data.error)
          error.value = data.error
          isProcessing.value = false
        }
      })
      
      isActive.value = true
      console.log('🚀 Live AI session started:', newSessionId)
      
      // Initial analysis of current conversation context
      if (messages.length > 0) {
        await analyzeConversationContext(messages)
      } else {
        response.value = "Live AI Response Assistant is now active. The AI will provide real-time response suggestions when you're in a conversation."
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to start Live AI'
      console.error('Failed to start Live AI:', err)
    } finally {
      isProcessing.value = false
    }
  }

  const stopLiveAI = async (): Promise<void> => {
    if (!sessionId.value) return
    
    try {
      console.log('⏹️ Live AI session stopped:', sessionId.value)
      
      // Clean up stream listener
      if (streamListener) {
        streamListener()
        streamListener = null
      }
      
      isActive.value = false
      response.value = ''
      sessionId.value = null
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to stop Live AI'
      console.error('Failed to stop Live AI:', err)
    }
  }

  const analyzeConversationContext = async (messages: any[]): Promise<void> => {
    if (!isActive.value || !sessionId.value) return
    
    try {
      isProcessing.value = true
      
      // Format conversation context for the AI
      const conversationContext = messages
        .filter(msg => !msg.isPreview) // Exclude preview messages
        .slice(-10) // Only take the last 10 messages for context
        .map(msg => `${msg.type === 'user' ? 'User' : 'System'}: ${msg.content}`)
        .join('\n')
      
      if (conversationContext.trim()) {
        console.log('💬 Analyzing conversation context for response suggestions')
        
        // Call the conversational AI backend function
        await invoke('generate_conversational_ai', {
          conversationContext,
          sessionId: sessionId.value
        })
      }
    } catch (err) {
      console.error('Failed to analyze conversation context:', err)
      error.value = err instanceof Error ? err.message : 'Failed to analyze conversation'
    }
  }

  // Function to trigger response assistance when system is speaking
  const onSystemSpeaking = async (messages: any[]): Promise<void> => {
    if (!isActive.value) return
    
    // Only trigger if the last message is from system/loopback
    const lastMessage = messages[messages.length - 1]
    if (lastMessage && lastMessage.source === 'loopback' && !lastMessage.isPreview) {
      console.log('🎤 System is speaking, generating response suggestions...')
      await analyzeConversationContext(messages)
    }
  }

  // Function to continuously analyze conversation changes
  const onConversationChange = async (messages: any[]): Promise<void> => {
    if (!isActive.value || isProcessing.value) return
    
    // Filter out preview messages and get recent context
    const realMessages = messages.filter(msg => !msg.isPreview)
    if (realMessages.length === 0) return
    
    // Only analyze if there are actual messages and we're not already processing
    console.log('💭 Conversation updated, analyzing for response suggestions...')
    await analyzeConversationContext(realMessages)
  }

  const reset = () => {
    if (streamListener) {
      streamListener()
      streamListener = null
    }
    isActive.value = false
    sessionId.value = null
    response.value = ''
    isProcessing.value = false
    error.value = null
  }

  return {
    isActive,
    sessionId,
    response,
    isProcessing,
    error,
    startLiveAI,
    stopLiveAI,
    analyzeConversationContext,
    onSystemSpeaking,
    onConversationChange,
    reset
  }
}