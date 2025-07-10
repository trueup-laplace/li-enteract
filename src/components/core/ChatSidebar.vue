<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  ChatBubbleLeftRightIcon,
  PlusIcon,
  PencilIcon,
  TrashIcon,
  XMarkIcon,
  ClockIcon,
  EllipsisVerticalIcon
} from '@heroicons/vue/24/outline'
import { useChatManagement } from '../../composables/useChatManagement'

interface Props {
  selectedModel: string | null
}

interface Emits {
  (e: 'close'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// Dummy scroll function for chat management
const scrollChatToBottom = () => {}

// Get chat management functions
const {
  chatSessions,
  currentChatId,
  createNewChat,
  switchChat,
  deleteChat,
  renameChat,
  clearChat
} = useChatManagement(props.selectedModel, scrollChatToBottom)

// Local state for UI interactions
const renamingChatId = ref<string | null>(null)
const newChatTitle = ref('')
const showMenuForChat = ref<string | null>(null)

// Computed properties
const sortedChats = computed(() => {
  return [...chatSessions.value].sort((a, b) => 
    new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime()
  )
})

// Chat management functions
const handleCreateNewChat = () => {
  createNewChat()
}

const handleSwitchChat = (chatId: string) => {
  switchChat(chatId)
  showMenuForChat.value = null
}

const startRenaming = (chatId: string, currentTitle: string) => {
  renamingChatId.value = chatId
  newChatTitle.value = currentTitle
  showMenuForChat.value = null
}

const finishRenaming = () => {
  if (renamingChatId.value && newChatTitle.value.trim()) {
    renameChat(renamingChatId.value, newChatTitle.value.trim())
  }
  renamingChatId.value = null
  newChatTitle.value = ''
}

const cancelRenaming = () => {
  renamingChatId.value = null
  newChatTitle.value = ''
}

const handleDeleteChat = (chatId: string) => {
  deleteChat(chatId)
  showMenuForChat.value = null
}

const handleClearChat = () => {
  clearChat()
  showMenuForChat.value = null
}

const toggleChatMenu = (chatId: string) => {
  showMenuForChat.value = showMenuForChat.value === chatId ? null : chatId
}

const formatRelativeTime = (dateString: string) => {
  const date = new Date(dateString)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / (1000 * 60))
  const diffHours = Math.floor(diffMins / 60)
  const diffDays = Math.floor(diffHours / 24)
  
  if (diffMins < 1) return 'Just now'
  if (diffMins < 60) return `${diffMins}m ago`
  if (diffHours < 24) return `${diffHours}h ago`
  if (diffDays < 7) return `${diffDays}d ago`
  return date.toLocaleDateString()
}

const closeWindow = () => {
  emit('close')
}
</script>

<template>
  <div class="chat-sidebar">
    <!-- Sidebar Header -->
    <div class="sidebar-header">
      <div class="sidebar-title">
        <ChatBubbleLeftRightIcon class="w-4 h-4 text-white/80" />
        <span class="text-sm font-medium text-white/90">Chat Sessions</span>
      </div>
      <button @click="closeWindow" class="close-btn">
        <XMarkIcon class="w-4 h-4 text-white/70 hover:text-white transition-colors" />
      </button>
    </div>

    <!-- New Chat Button -->
    <div class="new-chat-section">
      <button @click="handleCreateNewChat" class="new-chat-btn">
        <PlusIcon class="w-4 h-4" />
        <span>New Chat</span>
      </button>
    </div>

    <!-- Chat List -->
    <div class="chat-list">
      <div v-if="sortedChats.length === 0" class="empty-state">
        <ChatBubbleLeftRightIcon class="w-8 h-8 text-white/30 mb-2" />
        <p class="text-white/50 text-sm">No chat sessions yet</p>
        <p class="text-white/40 text-xs">Create your first chat to get started</p>
      </div>

      <div 
        v-for="chat in sortedChats" 
        :key="chat.id"
        class="chat-item"
        :class="{ 'active': chat.id === currentChatId }"
        @click="handleSwitchChat(chat.id)"
      >
        <div class="chat-info">
          <!-- Chat title (editable) -->
          <div v-if="renamingChatId === chat.id" class="rename-input-container">
            <input
              v-model="newChatTitle"
              @keyup.enter="finishRenaming"
              @keyup.escape="cancelRenaming"
              @blur="finishRenaming"
              class="rename-input"
              autofocus
            />
          </div>
          <div v-else class="chat-title">
            {{ chat.title }}
          </div>
          
          <!-- Chat metadata -->
          <div class="chat-meta">
            <ClockIcon class="w-3 h-3 text-white/40" />
            <span class="text-xs text-white/40">{{ formatRelativeTime(chat.updatedAt) }}</span>
            <span class="text-xs text-white/30">•</span>
            <span class="text-xs text-white/40">{{ chat.history.length }} messages</span>
          </div>
        </div>

        <!-- Chat actions menu -->
        <div class="chat-actions" @click.stop>
          <button 
            @click="toggleChatMenu(chat.id)" 
            class="menu-trigger"
            :class="{ 'active': showMenuForChat === chat.id }"
          >
            <EllipsisVerticalIcon class="w-4 h-4" />
          </button>

          <!-- Dropdown menu -->
          <div v-if="showMenuForChat === chat.id" class="chat-menu">
            <button @click="startRenaming(chat.id, chat.title)" class="menu-item">
              <PencilIcon class="w-3 h-3" />
              <span>Rename</span>
            </button>
            <button 
              v-if="chat.id === currentChatId && chat.history.length > 0"
              @click="handleClearChat" 
              class="menu-item"
            >
              <TrashIcon class="w-3 h-3" />
              <span>Clear History</span>
            </button>
            <button @click="handleDeleteChat(chat.id)" class="menu-item danger">
              <TrashIcon class="w-3 h-3" />
              <span>Delete Chat</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.chat-sidebar {
  @apply bg-black/90 backdrop-blur-xl border border-white/20 rounded-xl;
  @apply w-80 h-96 flex flex-col;
  @apply shadow-2xl shadow-black/50;
}

.sidebar-header {
  @apply flex items-center justify-between p-4 border-b border-white/10;
}

.sidebar-title {
  @apply flex items-center gap-2;
}

.close-btn {
  @apply p-1 rounded-md hover:bg-white/10 transition-colors;
}

.new-chat-section {
  @apply p-3 border-b border-white/10;
}

.new-chat-btn {
  @apply w-full flex items-center justify-center gap-2 px-3 py-2;
  @apply bg-blue-600/20 hover:bg-blue-600/30 border border-blue-500/30;
  @apply rounded-lg transition-colors text-sm font-medium text-white/90;
}

.chat-list {
  @apply flex-1 overflow-y-auto p-2;
}

.empty-state {
  @apply flex flex-col items-center justify-center h-full text-center px-6;
}

.chat-item {
  @apply flex items-center gap-3 p-3 mx-1 mb-1 rounded-lg;
  @apply hover:bg-white/5 cursor-pointer transition-colors;
  @apply border border-transparent;
}

.chat-item.active {
  @apply bg-blue-600/20 border-blue-500/30;
}

.chat-info {
  @apply flex-1 min-w-0;
}

.chat-title {
  @apply text-sm font-medium text-white/90 truncate;
}

.rename-input-container {
  @apply w-full;
}

.rename-input {
  @apply w-full px-2 py-1 text-sm bg-white/10 border border-white/20;
  @apply rounded text-white/90 focus:outline-none focus:border-blue-500/50;
}

.chat-meta {
  @apply flex items-center gap-1 mt-1;
}

.chat-actions {
  @apply relative;
}

.menu-trigger {
  @apply p-1 rounded hover:bg-white/10 transition-colors text-white/60;
  @apply hover:text-white/90;
}

.menu-trigger.active {
  @apply bg-white/10 text-white/90;
}

.chat-menu {
  @apply absolute right-0 top-8 bg-black/95 border border-white/20;
  @apply rounded-lg shadow-xl z-50 py-1 min-w-32;
}

.menu-item {
  @apply w-full flex items-center gap-2 px-3 py-2 text-sm;
  @apply hover:bg-white/10 transition-colors text-white/80;
  @apply hover:text-white/90;
}

.menu-item.danger {
  @apply text-red-400 hover:text-red-300 hover:bg-red-500/10;
}

/* Custom scrollbar */
.chat-list::-webkit-scrollbar {
  width: 4px;
}

.chat-list::-webkit-scrollbar-track {
  background: transparent;
}

.chat-list::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
}

.chat-list::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.3);
}
</style> 