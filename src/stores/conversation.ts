import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'

export interface ConversationMessage {
  id: string
  type: 'user' | 'system'
  source: 'microphone' | 'loopback'
  content: string
  timestamp: number
  confidence?: number
}

export interface ConversationSession {
  id: string
  name: string
  startTime: number
  endTime?: number
  messages: ConversationMessage[]
  isActive: boolean
}

export const useConversationStore = defineStore('conversation', () => {
  // State
  const currentSession = ref<ConversationSession | null>(null)
  const sessions = ref<ConversationSession[]>([])
  const isRecording = ref(false)
  const isAudioLoopbackActive = ref(false)

  // Persistence key
  const STORAGE_KEY = 'conversation-sessions'

  // Load sessions from localStorage on store creation
  const loadSessions = () => {
    try {
      const stored = localStorage.getItem(STORAGE_KEY)
      if (stored) {
        const parsed = JSON.parse(stored)
        sessions.value = parsed
        console.log(`📁 Loaded ${parsed.length} conversation sessions from storage`)
      }
    } catch (error) {
      console.error('Failed to load conversation sessions:', error)
    }
  }

  // Save sessions to localStorage
  const saveSessions = () => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(sessions.value))
      console.log(`💾 Saved ${sessions.value.length} conversation sessions to storage`)
    } catch (error) {
      console.error('Failed to save conversation sessions:', error)
    }
  }

  // Watch for changes and auto-save
  watch(sessions, saveSessions, { deep: true })

  // Initialize on store creation
  loadSessions()

  // Computed
  const currentMessages = computed(() => {
    return currentSession.value?.messages || []
  })

  const activeSessions = computed(() => {
    return sessions.value.filter(session => session.isActive)
  })

  const recentSessions = computed(() => {
    return [...sessions.value]
      .sort((a, b) => b.startTime - a.startTime)
      .slice(0, 10)
  })

  // Actions
  const createSession = (name?: string): ConversationSession => {
    const session: ConversationSession = {
      id: `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      name: name || `Conversation ${new Date().toLocaleTimeString()}`,
      startTime: Date.now(),
      messages: [],
      isActive: true
    }

    sessions.value.push(session)
    currentSession.value = session
    return session
  }

  const endSession = (sessionId?: string) => {
    const targetSession = sessionId 
      ? sessions.value.find(s => s.id === sessionId)
      : currentSession.value

    if (targetSession) {
      targetSession.isActive = false
      targetSession.endTime = Date.now()
      
      if (currentSession.value?.id === targetSession.id) {
        currentSession.value = null
      }
    }
  }

  const switchToSession = (sessionId: string) => {
    const session = sessions.value.find(s => s.id === sessionId)
    if (session) {
      // End current session if any
      if (currentSession.value) {
        endSession(currentSession.value.id)
      }
      
      session.isActive = true
      currentSession.value = session
    }
  }

  const addMessage = (messageData: Omit<ConversationMessage, 'id'>) => {
    if (!currentSession.value) {
      createSession()
    }

    const message: ConversationMessage = {
      id: `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      ...messageData
    }

    currentSession.value!.messages.push(message)
    return message
  }

  const updateMessage = (messageId: string, updates: Partial<ConversationMessage>) => {
    if (!currentSession.value) return null

    const messageIndex = currentSession.value.messages.findIndex(m => m.id === messageId)
    if (messageIndex !== -1) {
      currentSession.value.messages[messageIndex] = {
        ...currentSession.value.messages[messageIndex],
        ...updates
      }
      return currentSession.value.messages[messageIndex]
    }
    return null
  }

  const deleteMessage = (messageId: string) => {
    if (!currentSession.value) return false

    const messageIndex = currentSession.value.messages.findIndex(m => m.id === messageId)
    if (messageIndex !== -1) {
      currentSession.value.messages.splice(messageIndex, 1)
      return true
    }
    return false
  }

  const clearCurrentSession = () => {
    if (currentSession.value) {
      currentSession.value.messages = []
    }
  }

  const deleteSession = (sessionId: string) => {
    const sessionIndex = sessions.value.findIndex(s => s.id === sessionId)
    if (sessionIndex !== -1) {
      const deletedSession = sessions.value.splice(sessionIndex, 1)[0]
      
      // If we deleted the current session, clear it
      if (currentSession.value?.id === sessionId) {
        currentSession.value = null
      }
      
      return deletedSession
    }
    return null
  }

  const setRecordingState = (recording: boolean) => {
    isRecording.value = recording
  }

  const setAudioLoopbackState = (active: boolean) => {
    isAudioLoopbackActive.value = active
  }

  // Export messages to main chat (will be used for sending selected messages to main chat)
  const exportMessagesToMainChat = (messageIds: string[]) => {
    if (!currentSession.value) return []

    const messagesToExport = currentSession.value.messages.filter(m => 
      messageIds.includes(m.id)
    )

    // Emit custom event that can be caught by main chat system
    const exportEvent = new CustomEvent('conversation-export-to-chat', {
      detail: {
        messages: messagesToExport,
        sessionId: currentSession.value.id,
        sessionName: currentSession.value.name
      }
    })
    window.dispatchEvent(exportEvent)

    return messagesToExport
  }

  // Get session statistics
  const getSessionStats = (sessionId?: string) => {
    const session = sessionId 
      ? sessions.value.find(s => s.id === sessionId)
      : currentSession.value

    if (!session) return null

    const microphoneMessages = session.messages.filter(m => m.source === 'microphone')
    const loopbackMessages = session.messages.filter(m => m.source === 'loopback')
    const duration = session.endTime 
      ? session.endTime - session.startTime 
      : Date.now() - session.startTime

    return {
      totalMessages: session.messages.length,
      microphoneMessages: microphoneMessages.length,
      loopbackMessages: loopbackMessages.length,
      duration: Math.round(duration / 1000), // in seconds
      averageConfidence: session.messages
        .filter(m => m.confidence !== undefined)
        .reduce((sum, m) => sum + (m.confidence || 0), 0) / 
        session.messages.filter(m => m.confidence !== undefined).length
    }
  }

  // Export session data for backup/sharing
  const exportSessionData = (sessionId?: string) => {
    const sessionToExport = sessionId 
      ? sessions.value.find(s => s.id === sessionId)
      : currentSession.value

    if (!sessionToExport) return null

    return {
      session: sessionToExport,
      exportedAt: Date.now(),
      version: '1.0'
    }
  }

  // Import session data from backup/sharing
  const importSessionData = (sessionData: any) => {
    try {
      if (!sessionData.session) throw new Error('Invalid session data')
      
      const session: ConversationSession = {
        ...sessionData.session,
        id: `imported_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`, // New ID to avoid conflicts
        isActive: false
      }

      sessions.value.push(session)
      return session
    } catch (error) {
      console.error('Failed to import session data:', error)
      return null
    }
  }

  // Clear all sessions (with confirmation)
  const clearAllSessions = () => {
    sessions.value = []
    currentSession.value = null
  }

  // Get storage usage info
  const getStorageInfo = () => {
    const data = localStorage.getItem(STORAGE_KEY)
    const sizeBytes = data ? new Blob([data]).size : 0
    const sizeKB = Math.round(sizeBytes / 1024 * 100) / 100

    return {
      sessionCount: sessions.value.length,
      totalMessages: sessions.value.reduce((sum, s) => sum + s.messages.length, 0),
      storageSize: `${sizeKB} KB`,
      lastSaved: data ? 'Auto-saved' : 'Never'
    }
  }

  return {
    // State
    currentSession,
    sessions,
    isRecording,
    isAudioLoopbackActive,

    // Computed
    currentMessages,
    activeSessions,
    recentSessions,

    // Actions
    createSession,
    endSession,
    switchToSession,
    addMessage,
    updateMessage,
    deleteMessage,
    clearCurrentSession,
    deleteSession,
    setRecordingState,
    setAudioLoopbackState,
    exportMessagesToMainChat,
    getSessionStats,
    
    // Persistence actions
    loadSessions,
    saveSessions,
    exportSessionData,
    importSessionData,
    clearAllSessions,
    getStorageInfo
  }
})