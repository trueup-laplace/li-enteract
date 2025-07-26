import { ref } from 'vue'
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
  
  const loadConversations = async () => {
    isLoadingConversations.value = true
    try {
      // Placeholder for now - would load from store when methods are available
      allConversations.value = []
    } catch (error) {
      console.error('Failed to load conversations:', error)
    } finally {
      isLoadingConversations.value = false
    }
  }
  
  const createNewConversation = async () => {
    try {
      // Create new conversation through session creation
      conversationStore.createSession()
      await loadConversations()
    } catch (error) {
      console.error('Failed to create new conversation:', error)
    }
  }
  
  const resumeConversation = async (conversationId: string) => {
    try {
      // Update active state
      allConversations.value = allConversations.value.map(conv => ({
        ...conv,
        isActive: conv.id === conversationId
      }))
    } catch (error) {
      console.error('Failed to resume conversation:', error)
    }
  }
  
  const deleteConversation = async (conversationId: string) => {
    try {
      // Remove from local list for now
      allConversations.value = allConversations.value.filter(conv => conv.id !== conversationId)
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