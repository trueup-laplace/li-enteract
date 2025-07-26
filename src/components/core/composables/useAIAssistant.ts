import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface Message {
  id: string
  type: 'user' | 'system'
  source: 'microphone' | 'loopback'
  content: string
  confidence?: number
  timestamp: number
  isPreview?: boolean
}

export function useAIAssistant() {
  const aiResponse = ref('')
  const aiIsProcessing = ref(false)
  const aiError = ref<string | null>(null)
  
  const queryAI = async (query: string, messages: Message[]) => {
    if (!query.trim()) return
    
    aiIsProcessing.value = true
    aiError.value = null
    aiResponse.value = ''
    
    try {
      // Build context from messages
      const context = messages
        .filter(msg => !msg.isPreview)
        .map(msg => `${msg.type === 'user' ? 'User' : 'System'}: ${msg.content}`)
        .join('\n')
      
      // Call AI service through Tauri
      const response = await invoke<string>('query_ai_assistant', {
        query,
        context,
        sessionId: `conversation-${Date.now()}`
      })
      
      aiResponse.value = response
    } catch (error) {
      aiError.value = error instanceof Error ? error.message : 'Failed to get AI response'
      console.error('AI Assistant error:', error)
    } finally {
      aiIsProcessing.value = false
    }
  }
  
  const reset = () => {
    aiResponse.value = ''
    aiIsProcessing.value = false
    aiError.value = null
  }
  
  return {
    aiResponse,
    aiIsProcessing,
    aiError,
    queryAI,
    reset
  }
}