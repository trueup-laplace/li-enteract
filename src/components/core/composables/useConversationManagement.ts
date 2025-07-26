import { ref, watch } from 'vue'
import { useConversationStore } from '../../../stores/conversation'

interface Conversation {
  id: string
  name: string
  startTime: number
  endTime?: number
  messages: any[]
  isActive?: boolean
}

export function useConversationManagement() {
  const conversationStore = useConversationStore()
  
  const allConversations = ref<Conversation[]>([])
  const isLoadingConversations = ref(false)
  
  // Watch for changes in the store's sessions and update allConversations
  watch(() => conversationStore.sessions, (newSessions) => {
    console.log(`📁 ConversationManagement: Store sessions changed:`, newSessions.length, newSessions.map(s => ({ id: s.id, name: s.name, messageCount: s.messages.length, isActive: s.isActive })))
    
    allConversations.value = newSessions.map(session => ({
      id: session.id,
      name: session.name,
      startTime: session.startTime,
      endTime: session.endTime,
      messages: session.messages,
      isActive: session.isActive
    }))
    
    console.log(`📁 ConversationManagement: Updated conversations array:`, allConversations.value.length, allConversations.value.map(c => ({ id: c.id, name: c.name, messageCount: c.messages.length })))
  }, { deep: true, immediate: true })
  
  const loadConversations = async () => {
    isLoadingConversations.value = true
    try {
      // Load sessions from the conversation store
      await conversationStore.loadSessions()
      console.log(`📁 ConversationManagement: Loaded ${conversationStore.sessions.length} conversations from store`)
      // The watcher will automatically update allConversations.value
    } catch (error) {
      console.error('Failed to load conversations:', error)
    } finally {
      isLoadingConversations.value = false
    }
  }
  
  const createNewConversation = async () => {
    try {
      // Create new conversation through session creation
      const session = conversationStore.createSession()
      console.log('🆕 ConversationManagement: Created new session:', session.id)
      await loadConversations()
    } catch (error) {
      console.error('Failed to create new conversation:', error)
    }
  }
  
  const resumeConversation = async (conversationId: string) => {
    try {
      // Switch to the selected session
      conversationStore.switchToSession(conversationId)
      console.log('🔄 ConversationManagement: Switched to session:', conversationId)
      await loadConversations()
    } catch (error) {
      console.error('Failed to resume conversation:', error)
    }
  }
  
  const deleteConversation = async (conversationId: string) => {
    try {
      // Delete the session from the store
      await conversationStore.deleteSession(conversationId)
      console.log('🗑️ ConversationManagement: Deleted session:', conversationId)
      await loadConversations()
    } catch (error) {
      console.error('Failed to delete conversation:', error)
    }
  }
  
  return {
    allConversations,
    isLoadingConversations,
    loadConversations,
    createNewConversation,
    resumeConversation,
    deleteConversation
  }
}