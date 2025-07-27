<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  QueueListIcon,
  XMarkIcon,
  PlusIcon,
  TrashIcon,
  ClockIcon,
  PencilIcon,
  EllipsisVerticalIcon
} from '@heroicons/vue/24/outline'
import { useChatManagement } from '../../composables/useChatManagement'

interface Props {
  show: boolean
  selectedModel: string | null
}

interface Emits {
  (e: 'close'): void
  (e: 'new-chat'): void
  (e: 'switch-chat', chatId: string): void
  (e: 'delete-chat', chatId: string): void
  (e: 'clear-chat'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// Dummy scroll function for chat management
const scrollChatToBottom = () => {}

// Get chat management functions
const {
  chatSessions,
  currentChatId,
  renameChat
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

// Format timestamp for display
const formatTimestamp = (timestamp: Date | string) => {
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

// Rename chat functions
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

// Menu functions
const toggleMenu = (chatId: string) => {
  showMenuForChat.value = showMenuForChat.value === chatId ? null : chatId
}

// Event handlers
const handleClose = () => emit('close')
const handleNewChat = () => emit('new-chat')
const handleSwitchChat = (chatId: string) => emit('switch-chat', chatId)
const handleDeleteChat = (chatId: string) => {
  emit('delete-chat', chatId)
  showMenuForChat.value = null
}
const handleClearChat = () => emit('clear-chat')
</script>

<template>
  <Transition name="sidebar">
    <div v-if="show" class="chat-window-sidebar">
      <!-- Sidebar Header -->
      <div class="sidebar-header">
        <div class="flex items-center gap-2">
          <QueueListIcon class="w-4 h-4 text-white/80" />
          <span class="text-sm font-medium text-white/90">Chat History</span>
        </div>
        <button @click="handleClose" class="close-btn">
          <XMarkIcon class="w-4 h-4 text-white/70 hover:text-white transition-colors" />
        </button>
      </div>
      
      <!-- Chat List -->
      <div class="chat-list">
        <div v-if="sortedChats.length === 0" class="empty-state">
          <p class="text-white/40 text-xs">No chat history</p>
        </div>
        
        <div v-else class="space-y-1">
          <div 
            v-for="chat in sortedChats" 
            :key="chat.id"
            @click="handleSwitchChat(chat.id)"
            class="chat-item"
            :class="{ 'active': chat.id === currentChatId }"
          >
            <div class="chat-item-content">
              <div v-if="renamingChatId === chat.id" class="rename-input-wrapper">
                <input
                  v-model="newChatTitle"
                  @keyup.enter="finishRenaming"
                  @keyup.escape="cancelRenaming"
                  @blur="finishRenaming"
                  class="rename-input"
                  :placeholder="chat.title"
                  autofocus
                />
              </div>
              <div v-else class="chat-info">
                <span class="chat-title">{{ chat.title }}</span>
                <span class="chat-time">{{ formatTimestamp(chat.updatedAt) }}</span>
              </div>
            </div>
            
            <div class="chat-actions">
              <button 
                @click.stop="toggleMenu(chat.id)"
                class="menu-btn"
              >
                <EllipsisVerticalIcon class="w-4 h-4" />
              </button>
              
              <!-- Dropdown Menu -->
              <Transition name="menu">
                <div 
                  v-if="showMenuForChat === chat.id" 
                  class="dropdown-menu"
                >
                  <button 
                    @click.stop="startRenaming(chat.id, chat.title)"
                    class="menu-item"
                  >
                    <PencilIcon class="w-3 h-3" />
                    <span>Rename</span>
                  </button>
                  <button 
                    @click.stop="handleDeleteChat(chat.id)"
                    class="menu-item text-red-400 hover:bg-red-500/20"
                  >
                    <TrashIcon class="w-3 h-3" />
                    <span>Delete</span>
                  </button>
                </div>
              </Transition>
            </div>
          </div>
        </div>
      </div>
      
      <!-- Sidebar Footer -->
      <div class="sidebar-footer">
        <button @click="handleNewChat" class="new-chat-btn">
          <PlusIcon class="w-4 h-4" />
          <span>New Chat</span>
        </button>
        <button @click="handleClearChat" class="clear-chat-btn">
          <TrashIcon class="w-4 h-4" />
          <span>Clear All</span>
        </button>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.chat-window-sidebar {
  @apply w-80 flex flex-col border-r border-white/10;
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(20px);
}

.sidebar-header {
  @apply flex items-center justify-between px-4 py-3 border-b border-white/10;
  flex-shrink: 0;
}

.close-btn {
  @apply rounded-full p-1 hover:bg-white/10 transition-colors;
}

.chat-list {
  @apply flex-1 overflow-y-auto p-3;
  min-height: 0;
}

.empty-state {
  @apply flex items-center justify-center h-32;
}

.chat-item {
  @apply relative flex items-center justify-between p-3 rounded-lg cursor-pointer transition-all duration-200;
  @apply hover:bg-white/5;
}

.chat-item.active {
  @apply bg-blue-500/20 hover:bg-blue-500/25;
}

.chat-item-content {
  @apply flex-1 min-w-0 mr-2;
}

.chat-info {
  @apply flex flex-col;
}

.chat-title {
  @apply text-sm text-white/90 truncate;
}

.chat-time {
  @apply text-xs text-white/50 mt-0.5;
}

.rename-input-wrapper {
  @apply w-full;
}

.rename-input {
  @apply w-full px-2 py-1 text-sm bg-white/10 border border-white/20 rounded focus:outline-none focus:border-blue-500/50;
  @apply text-white placeholder-white/50;
}

.chat-actions {
  @apply relative flex items-center;
}

.menu-btn {
  @apply p-1 rounded hover:bg-white/10 transition-colors text-white/60 hover:text-white/90;
}

.dropdown-menu {
  @apply absolute right-0 top-full mt-1 bg-black/95 border border-white/20 rounded-lg shadow-xl z-10;
  @apply min-w-[120px] py-1;
  backdrop-filter: blur(10px);
}

.menu-item {
  @apply flex items-center gap-2 w-full px-3 py-2 text-xs text-white/80 hover:bg-white/10 transition-colors;
}

.menu-item:first-child {
  @apply rounded-t-lg;
}

.menu-item:last-child {
  @apply rounded-b-lg;
}

.sidebar-footer {
  @apply flex gap-2 p-3 border-t border-white/10;
  flex-shrink: 0;
}

.new-chat-btn,
.clear-chat-btn {
  @apply flex-1 flex items-center justify-center gap-2 px-3 py-2 rounded-lg transition-all duration-200;
  @apply text-xs font-medium;
}

.new-chat-btn {
  @apply bg-blue-500/20 text-blue-400 hover:bg-blue-500/30 border border-blue-500/30;
}

.clear-chat-btn {
  @apply bg-white/5 text-white/60 hover:bg-white/10 border border-white/10;
}

/* Transitions */
.sidebar-enter-active,
.sidebar-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.sidebar-enter-from {
  transform: translateX(-100%);
  opacity: 0;
}

.sidebar-leave-to {
  transform: translateX(-100%);
  opacity: 0;
}

.menu-enter-active,
.menu-leave-active {
  transition: all 0.15s ease-out;
}

.menu-enter-from {
  opacity: 0;
  transform: translateY(-4px) scale(0.95);
}

.menu-leave-to {
  opacity: 0;
  transform: translateY(-4px) scale(0.95);
}

/* Scrollbar */
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
</style>