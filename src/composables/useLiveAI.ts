import { ref, Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

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

  const startLiveAI = async (messages: any[]): Promise<void> => {
    try {
      error.value = null
      isProcessing.value = true
      
      // Create a new session
      const newSessionId = `live-ai-${Date.now()}`
      sessionId.value = newSessionId
      
      // Start live AI processing
      await invoke('start_live_ai_session', {
        sessionId: newSessionId,
        context: messages
      })
      
      isActive.value = true
      console.log('🚀 Live AI session started:', newSessionId)
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
      await invoke('stop_live_ai_session', {
        sessionId: sessionId.value
      })
      
      isActive.value = false
      console.log('⏹️ Live AI session stopped:', sessionId.value)
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to stop Live AI'
      console.error('Failed to stop Live AI:', err)
    }
  }

  const processLiveContext = async (messages: any[]): Promise<void> => {
    if (!isActive.value || !sessionId.value) return
    
    try {
      isProcessing.value = true
      
      // Process the current conversation context
      const result = await invoke<LiveAIResponse>('process_live_ai_context', {
        sessionId: sessionId.value,
        messages: messages
      })
      
      if (result && result.text) {
        response.value = result.text
      }
    } catch (err) {
      console.error('Failed to process live context:', err)
    } finally {
      isProcessing.value = false
    }
  }

  const reset = () => {
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
    processLiveContext,
    reset
  }
}