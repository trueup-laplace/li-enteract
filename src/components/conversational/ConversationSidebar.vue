<script setup lang="ts">
import { watch } from 'vue'
import { 
  QueueListIcon, 
  XMarkIcon, 
  PlusIcon, 
  TrashIcon, 
  ClockIcon 
} from '@heroicons/vue/24/outline'

interface Conversation {
  id: string
  name: string
  startTime: number
  endTime?: number
  messages: any[]
  isActive?: boolean
}

interface Props {
  show: boolean
  conversations: Conversation[]
  isLoading?: boolean
}

interface Emits {
  (e: 'close'): void
  (e: 'new-conversation'): void
  (e: 'resume-conversation', id: string): void
  (e: 'delete-conversation', id: string): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// Debug: Watch conversations prop
watch(() => props.conversations, (newConversations) => {
  console.log('📁 ConversationSidebar: Received conversations:', newConversations.length, newConversations.map(c => ({ id: c.id, name: c.name, messageCount: c.messages.length })))
}, { immediate: true })

const formatTimestamp = (timestamp: number): string => {
  const date = new Date(timestamp)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  
  if (diffMins < 1) return 'Just now'
  if (diffMins < 60) return `${diffMins}m ago`
  
  const diffHours = Math.floor(diffMins / 60)
  if (diffHours < 24) return `${diffHours}h ago`
  
  const diffDays = Math.floor(diffHours / 24)
  if (diffDays < 7) return `${diffDays}d ago`
  
  return date.toLocaleDateString()
}

const formatDuration = (startTime: number, endTime?: number): string => {
  if (!endTime) return 'Ongoing'
  
  const durationMs = endTime - startTime
  const durationSecs = Math.floor(durationMs / 1000)
  const durationMins = Math.floor(durationSecs / 60)
  
  if (durationMins < 1) return `${durationSecs}s`
  if (durationMins < 60) return `${durationMins}m`
  
  const hours = Math.floor(durationMins / 60)
  const mins = durationMins % 60
  return `${hours}h ${mins}m`
}

const getConversationSummary = (conversation: Conversation): string => {
  const messageCount = conversation.messages.length
  const userMessages = conversation.messages.filter(m => m.type === 'user').length
  const systemMessages = conversation.messages.filter(m => m.type === 'system').length
  
  return `${messageCount} messages (${userMessages} user, ${systemMessages} system)`
}

const getLastMessagePreview = (conversation: Conversation): string => {
  if (conversation.messages.length === 0) return 'No messages'
  
  const lastMessage = conversation.messages[conversation.messages.length - 1]
  const preview = lastMessage.content.substring(0, 100)
  return preview.length < lastMessage.content.length ? `${preview}...` : preview
}

const handleClose = () => emit('close')
const handleNewConversation = () => emit('new-conversation')
const handleResumeConversation = (id: string, event: Event) => {
  event.stopPropagation()
  emit('resume-conversation', id)
}
const handleDeleteConversation = (id: string, event: Event) => {
  event.stopPropagation()
  emit('delete-conversation', id)
}
</script>

<template>
  <div v-if="show" class="conversation-sidebar">
    <div class="sidebar-header">
      <div class="flex items-center gap-2">
        <QueueListIcon class="w-4 h-4 text-purple-400" />
        <h3 class="text-sm font-medium text-white">Conversations</h3>
      </div>
      <button @click="handleClose" class="close-sidebar-btn">
        <XMarkIcon class="w-4 h-4" />
      </button>
    </div>
    
    <div class="sidebar-content">
      <!-- New Conversation Button -->
      <button @click.stop="handleNewConversation" class="new-conversation-btn">
        <PlusIcon class="w-4 h-4" />
        New Conversation
      </button>
      
      <!-- Conversations List -->
      <div class="conversations-list">
        <div v-if="isLoading" class="loading-state">
          <div class="loading-spinner"></div>
          <span class="text-xs text-white/60">Loading...</span>
        </div>
        
        <div v-else-if="conversations.length === 0" class="empty-state">
          <QueueListIcon class="w-8 h-8 text-white/20 mx-auto mb-2" />
          <p class="text-white/60 text-xs text-center">No conversations yet</p>
        </div>
        
        <div v-else class="conversations-grid">
          <div 
            v-for="conversation in conversations" 
            :key="conversation.id"
            class="conversation-item"
            :class="{ 'active': conversation.isActive }"
            @click.stop="(event) => handleResumeConversation(conversation.id, event)"
          >
            <div class="conversation-header">
              <span class="conversation-title">{{ conversation.name }}</span>
              <button 
                @click.stop="(event) => handleDeleteConversation(conversation.id, event)"
                class="delete-btn"
                title="Delete conversation"
              >
                <TrashIcon class="w-3 h-3" />
              </button>
            </div>
            
            <div class="conversation-meta">
              <div class="meta-row">
                <ClockIcon class="w-3 h-3 text-white/40" />
                <span class="text-xs text-white/40">{{ formatTimestamp(conversation.startTime) }}</span>
                <span class="text-xs text-white/40">•</span>
                <span class="text-xs text-white/40">{{ formatDuration(conversation.startTime, conversation.endTime) }}</span>
              </div>
              <div class="meta-row">
                <span class="text-xs text-white/50">{{ getConversationSummary(conversation) }}</span>
              </div>
            </div>
            
            <div class="conversation-preview">
              <p class="text-xs text-white/60">{{ getLastMessagePreview(conversation) }}</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.conversation-sidebar {
  @apply w-80 border-r border-white/10 bg-white/5 backdrop-blur-sm flex flex-col;
  min-width: 320px;
  max-width: 400px;
}

.sidebar-header {
  @apply flex items-center justify-between px-4 py-3 border-b border-white/10;
}

.close-sidebar-btn {
  @apply rounded-full p-1 hover:bg-white/10 transition-colors text-white/70 hover:text-white;
}

.sidebar-content {
  @apply flex-1 overflow-hidden flex flex-col;
}

.new-conversation-btn {
  @apply flex items-center justify-center gap-2 mx-4 my-3 px-4 py-2 bg-purple-500/80 hover:bg-purple-500 text-white rounded-lg transition-all duration-200 font-medium text-sm;
  width: calc(100% - 2rem);
}

.conversations-list {
  @apply flex-1 overflow-y-auto px-4 pb-4;
}

.loading-state {
  @apply flex flex-col items-center gap-2 py-8;
}

.loading-spinner {
  @apply w-4 h-4 border-2 border-white/20 border-t-white/60 rounded-full animate-spin;
}

.empty-state {
  @apply py-8 text-center;
}

.conversations-grid {
  @apply space-y-2;
}

.conversation-item {
  @apply rounded-lg bg-white/5 hover:bg-white/10 transition-all duration-200 cursor-pointer p-3 border border-transparent;
}

.conversation-item.active {
  @apply border-purple-500/50 bg-purple-500/10;
}

.conversation-header {
  @apply flex items-center justify-between mb-2;
}

.conversation-title {
  @apply text-xs font-medium text-white truncate flex-1 mr-2;
}

.delete-btn {
  @apply rounded-full p-1 hover:bg-red-500/20 transition-colors text-white/60 hover:text-red-400;
}

.conversation-meta {
  @apply space-y-1 mb-2;
}

.meta-row {
  @apply flex items-center gap-1;
}

.conversation-preview {
  @apply line-clamp-2;
}
</style>