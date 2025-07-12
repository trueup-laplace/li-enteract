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
  isOpen: boolean
}

interface Emits {
  (e: 'close'): void
  (e: 'open-chat-window'): void
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
  emit('open-chat-window') // Open chat window to show the new empty chat
  emit('close') // Close drawer after creating new chat
}

const handleSwitchChat = (chatId: string) => {
  switchChat(chatId)
  showMenuForChat.value = null
  emit('close') // Close drawer after switching chat
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
  const wasCurrentChat = chatId === currentChatId.value
  deleteChat(chatId)
  showMenuForChat.value = null
  
  // If we deleted the current chat, open the chat window to show the result
  if (wasCurrentChat) {
    emit('open-chat-window')
  }
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
  <!-- Overlay Background -->
  <Transition name="drawer-overlay">
    <div v-if="isOpen" class="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm" @click="closeWindow">
      <!-- Drawer Content -->
      <Transition name="drawer-slide">
        <div v-if="isOpen" class="fixed top-0 left-0 h-full w-80 z-50 bg-black/90 backdrop-blur-xl border-r border-white/20 shadow-2xl shadow-black/50 flex flex-col" @click.stop>
          <!-- Sidebar Header -->
          <div class="flex items-center justify-between p-4 border-b border-white/10">
            <div class="flex items-center gap-2">
              <ChatBubbleLeftRightIcon class="w-4 h-4 text-white/80" />
              <span class="text-sm font-medium text-white/90">Chat Sessions</span>
            </div>
            <button @click="closeWindow" class="p-1 rounded-md hover:bg-white/10 transition-colors">
              <XMarkIcon class="w-4 h-4 text-white/70 hover:text-white transition-colors" />
            </button>
          </div>

          <!-- New Chat Button -->
          <div class="p-3 border-b border-white/10">
            <button @click="handleCreateNewChat" class="w-full flex items-center justify-center gap-2 px-3 py-2 bg-blue-600/20 hover:bg-blue-600/30 border border-blue-500/30 rounded-lg transition-colors text-sm font-medium text-white/90">
              <PlusIcon class="w-4 h-4" />
              <span>New Chat</span>
            </button>
          </div>

          <!-- Chat List -->
          <div class="flex-1 overflow-y-auto p-2" style="scrollbar-width: thin;">
            <div v-if="sortedChats.length === 0" class="flex flex-col items-center justify-center h-full text-center px-6">
              <ChatBubbleLeftRightIcon class="w-8 h-8 text-white/30 mb-2" />
              <p class="text-white/50 text-sm">No chat sessions yet</p>
              <p class="text-white/40 text-xs">Create your first chat to get started</p>
            </div>

            <div 
              v-for="chat in sortedChats" 
              :key="chat.id"
              class="flex items-center gap-3 p-3 mx-1 mb-1 rounded-lg hover:bg-white/5 cursor-pointer transition-colors border border-transparent"
              :class="{ 'bg-blue-600/20 border-blue-500/30': chat.id === currentChatId }"
              @click="handleSwitchChat(chat.id)"
            >
              <div class="flex-1 min-w-0">
                <!-- Chat title (editable) -->
                <div v-if="renamingChatId === chat.id" class="w-full">
                  <input
                    v-model="newChatTitle"
                    @keyup.enter="finishRenaming"
                    @keyup.escape="cancelRenaming"
                    @blur="finishRenaming"
                    class="w-full px-2 py-1 text-sm bg-white/10 border border-white/20 rounded text-white/90 focus:outline-none focus:border-blue-500/50"
                    autofocus
                  />
                </div>
                <div v-else class="text-sm font-medium text-white/90 truncate">
                  {{ chat.title }}
                </div>
                
                <!-- Chat metadata -->
                <div class="flex items-center gap-1 mt-1">
                  <ClockIcon class="w-3 h-3 text-white/40" />
                  <span class="text-xs text-white/40">{{ formatRelativeTime(chat.updatedAt) }}</span>
                  <span class="text-xs text-white/30">•</span>
                  <span class="text-xs text-white/40">{{ chat.history.length }} messages</span>
                </div>
              </div>

              <!-- Chat actions menu -->
              <div class="relative" @click.stop>
                <button 
                  @click="toggleChatMenu(chat.id)" 
                  class="p-1 rounded hover:bg-white/10 transition-colors text-white/60 hover:text-white/90"
                  :class="{ 'bg-white/10 text-white/90': showMenuForChat === chat.id }"
                >
                  <EllipsisVerticalIcon class="w-4 h-4" />
                </button>

                <!-- Dropdown menu -->
                <div v-if="showMenuForChat === chat.id" class="absolute right-0 top-8 bg-black/95 border border-white/20 rounded-lg shadow-xl z-50 py-1 min-w-32">
                  <button @click="startRenaming(chat.id, chat.title)" class="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-white/10 transition-colors text-white/80 hover:text-white/90">
                    <PencilIcon class="w-3 h-3" />
                    <span>Rename</span>
                  </button>
                  <button 
                    v-if="chat.id === currentChatId && chat.history.length > 0"
                    @click="handleClearChat" 
                    class="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-white/10 transition-colors text-white/80 hover:text-white/90"
                  >
                    <TrashIcon class="w-3 h-3" />
                    <span>Clear History</span>
                  </button>
                  <button @click="handleDeleteChat(chat.id)" class="w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-white/10 transition-colors text-red-400 hover:text-red-300 hover:bg-red-500/10">
                    <TrashIcon class="w-3 h-3" />
                    <span>Delete Chat</span>
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<style scoped>
/* Transitions */
.drawer-overlay-enter-active,
.drawer-overlay-leave-active {
  transition: opacity 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.drawer-overlay-enter-from,
.drawer-overlay-leave-to {
  opacity: 0;
}

.drawer-slide-enter-active,
.drawer-slide-leave-active {
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.drawer-slide-enter-from,
.drawer-slide-leave-to {
  transform: translateX(-100%);
}

/* Custom scrollbar for webkit browsers */
.flex-1.overflow-y-auto::-webkit-scrollbar {
  width: 4px;
}

.flex-1.overflow-y-auto::-webkit-scrollbar-track {
  background: transparent;
}

.flex-1.overflow-y-auto::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
}

.flex-1.overflow-y-auto::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.3);
}
</style>