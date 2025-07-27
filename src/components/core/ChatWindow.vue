<script setup lang="ts">
import { ref, watch, nextTick, toRef, onMounted, onUnmounted, computed } from 'vue'
import {
  CommandLineIcon,
  XMarkIcon,
  ArrowsPointingOutIcon,
  ShieldCheckIcon,
  ExclamationTriangleIcon,
  Bars3Icon,
  ChatBubbleLeftRightIcon,
  PlusIcon,
  PencilIcon,
  TrashIcon,
  EllipsisVerticalIcon,
  ClockIcon,
  MicrophoneIcon,
  StopIcon
} from '@heroicons/vue/24/outline'
import { useChatManagement } from '../../composables/useChatManagement'
import { useWindowResizing } from '../../composables/useWindowResizing'
import { useSpeechEvents } from '../../composables/useSpeechEvents'
import { useSpeechTranscription } from '../../composables/useSpeechTranscription'
import AgentActionButtons from './AgentActionButtons.vue'
import ModelSelector from './ModelSelector.vue'

interface Props {
  showChatWindow: boolean
  selectedModel: string | null
}

interface Emits {
  (e: 'close'): void
  (e: 'update:showChatWindow', value: boolean): void
  (e: 'update:selectedModel', value: string): void
  (e: 'toggle-chat-drawer'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// Refs for scrolling
const chatMessages = ref<HTMLElement>()

// Chat sidebar state
const showChatSidebar = ref(false)
const renamingChatId = ref<string | null>(null)
const newChatTitle = ref('')
const showMenuForChat = ref<string | null>(null)

// Agent and model selection state
const currentAgent = ref('enteract')
const showMentionSuggestions = ref(false)
const mentionSuggestions = ref<Array<{id: string, name: string, description: string}>>([])
const mentionStartPos = ref(-1)

// Available agents for @ mentions
const availableAgents = [
  { id: 'enteract', name: '@enteract', description: 'General purpose AI assistant' },
  { id: 'coding', name: '@coding', description: 'Programming assistance and code review' },
  { id: 'research', name: '@research', description: 'Advanced research with step-by-step thinking' },
  { id: 'vision', name: '@vision', description: 'Visual content analysis' }
]

// Handle @ mention input
const handleInput = (event: Event) => {
  const target = event.target as HTMLInputElement
  const value = target.value
  const cursorPos = target.selectionStart || 0
  
  // Check for @ mention
  const beforeCursor = value.substring(0, cursorPos)
  const lastAtIndex = beforeCursor.lastIndexOf('@')
  
  if (lastAtIndex !== -1) {
    const afterAt = beforeCursor.substring(lastAtIndex + 1)
    
    // If there's no space after @, show suggestions
    if (!afterAt.includes(' ') && afterAt.length >= 0) {
      const filtered = availableAgents.filter(agent => 
        agent.name.toLowerCase().includes(('@' + afterAt).toLowerCase())
      )
      
      if (filtered.length > 0) {
        mentionSuggestions.value = filtered
        mentionStartPos.value = lastAtIndex
        showMentionSuggestions.value = true
        return
      }
    }
  }
  
  showMentionSuggestions.value = false
}

// Select mention suggestion
const selectMention = (agent: {id: string, name: string, description: string}) => {
  const input = document.querySelector('.chat-input') as HTMLInputElement
  if (input && mentionStartPos.value !== -1) {
    const beforeMention = chatMessage.value.substring(0, mentionStartPos.value)
    const afterCursor = chatMessage.value.substring(input.selectionStart || 0)
    
    chatMessage.value = beforeMention + agent.name + ' ' + afterCursor
    currentAgent.value = agent.id
    showMentionSuggestions.value = false
    
    // Focus input and position cursor after mention
    setTimeout(() => {
      input.focus()
      const newPos = beforeMention.length + agent.name.length + 1
      input.setSelectionRange(newPos, newPos)
    }, 0)
  }
}

// Parse message for agent mentions before sending
const parseAgentFromMessage = (message: string): string => {
  const mentionMatch = message.match(/@(\w+)/)
  if (mentionMatch) {
    const mentionedAgent = mentionMatch[1]
    const agent = availableAgents.find(a => a.id === mentionedAgent)
    if (agent) {
      currentAgent.value = agent.id
      // Remove the mention from the message
      return message.replace(/@\w+\s*/, '').trim()
    }
  }
  return message
}

// Enhanced keyboard handler
const handleEnhancedKeydown = (event: KeyboardEvent) => {
  if (showMentionSuggestions.value) {
    if (event.key === 'Escape') {
      showMentionSuggestions.value = false
      return
    }
    if (event.key === 'Tab' || event.key === 'Enter') {
      event.preventDefault()
      if (mentionSuggestions.value.length > 0) {
        selectMention(mentionSuggestions.value[0])
      }
      return
    }
  }
  
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault()
    sendMessageWithAgent()
  }
}

// Enhanced send message with agent detection
const sendMessageWithAgent = async () => {
  if (!chatMessage.value.trim()) return
  
  const originalMessage = chatMessage.value.trim()
  const cleanedMessage = parseAgentFromMessage(originalMessage)
  
  chatMessage.value = ''
  showMentionSuggestions.value = false
  
  await sendMessage(currentAgent.value, cleanedMessage || originalMessage)
}

// Handle model and agent selection
const handleModelUpdate = (modelName: string) => {
  emit('update:selectedModel', modelName)
}

const handleAgentUpdate = (agentId: string) => {
  currentAgent.value = agentId
}

// Helper function to scroll chat to bottom
const scrollChatToBottom = () => {
  if (chatMessages.value) {
    chatMessages.value.scrollTop = chatMessages.value.scrollHeight
  }
}

// Chat management composable
const {
  chatMessage,
  chatHistory,
  chatSessions,
  currentChatId,
  createNewChat,
  switchChat,
  deleteChat,
  renameChat,
  clearChat,
  fileInput,
  renderMarkdown,
  takeScreenshotAndAnalyze,
  startDeepResearch,
  startConversationalAgent,
  startCodingAgent,
  startComputerUseAgent,
  sendMessage,
  handleChatKeydown,
  triggerFileUpload,
  handleFileUpload,
  estimateTokens
} = useChatManagement(props.selectedModel, scrollChatToBottom)

// Context truncation detection
const MAX_TOKENS = 4000
const isContextTruncated = computed(() => {
  if (!chatHistory.value || chatHistory.value.length === 0) return false
  
  const totalTokens = chatHistory.value.reduce((sum, message) => {
    return sum + estimateTokens(message.text)
  }, 0)
  
  return totalTokens > MAX_TOKENS
})

// Computed properties for sidebar
const sortedChats = computed(() => {
  return [...chatSessions.value].sort((a, b) => 
    new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime()
  )
})

// Set up speech events with real chat management functions
const { setupSpeechTranscriptionListeners, removeSpeechTranscriptionListeners } = useSpeechEvents(
  chatHistory,
  toRef(props, 'showChatWindow'),
  scrollChatToBottom,
  chatMessage,
  sendMessage
)

// Window resizing composable
const {
  chatWindowSize,
  isResizing,
  startResize
} = useWindowResizing()

// Speech transcription for microphone button
const {
  initialize: initializeSpeech,
  startRecording: startSpeechRecording,
  stopRecording: stopSpeechRecording,
  isRecording: isSpeechRecording,
  isInitialized: isSpeechInitialized,
  currentTranscript: speechCurrentTranscript,
  error: speechError,
  setAutoSendToChat,
  setContinuousMode
} = useSpeechTranscription()

const closeWindow = () => {
  emit('close')
  emit('update:showChatWindow', false)
}

const toggleChatDrawer = () => {
  emit('toggle-chat-drawer')
  console.log('💬 Chat drawer toggle requested')
}

// Focus input when chat window opens
watch(() => props.showChatWindow, async (newValue) => {
  if (newValue) {
    await nextTick()
    setTimeout(() => {
      const input = document.querySelector('.chat-input') as HTMLInputElement
      if (input) input.focus()
    }, 150)
  }
})

// Auto-scroll when chat history changes
watch(chatHistory, async () => {
  await nextTick()
  scrollChatToBottom()
}, { deep: true })

// Agent action handlers
const handleTakeScreenshot = () => {
  takeScreenshotAndAnalyze({ value: props.showChatWindow })
}

const handleStartDeepResearch = () => {
  startDeepResearch({ value: props.showChatWindow })
}

const handleStartConversational = () => {
  startConversationalAgent({ value: props.showChatWindow })
}

const handleStartCoding = () => {
  startCodingAgent({ value: props.showChatWindow })
}

const handleStartComputerUse = () => {
  startComputerUseAgent({ value: props.showChatWindow })
}


const handleFileUploadEvent = (event: Event) => {
  handleFileUpload(event, { value: props.showChatWindow })
}

// Microphone functionality for chat interface
const handleMicrophoneToggle = async () => {
  try {
    if (isSpeechRecording.value) {
      await stopSpeechRecording()
      console.log('🎤 Chat: Speech recording stopped')
    } else {
      if (!isSpeechInitialized.value) {
        await initializeSpeech()
      }
      await startSpeechRecording()
      console.log('🎤 Chat: Speech recording started')
    }
  } catch (error) {
    console.error('Chat microphone toggle error:', error)
  }
}

// Sidebar functions
const toggleChatSidebar = () => {
  showChatSidebar.value = !showChatSidebar.value
}

const handleCreateNewChat = () => {
  createNewChat()
  showChatSidebar.value = false
}

const handleSwitchChat = (chatId: string) => {
  switchChat(chatId)
  showMenuForChat.value = null
  showChatSidebar.value = false
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



// Setup speech events when component mounts
onMounted(async () => {
  setupSpeechTranscriptionListeners()
  console.log('🎤 ChatWindow: Speech transcription listeners set up')
  
  // Initialize speech transcription for chat interface
  try {
    await initializeSpeech()
    // Enable auto-send to chat for the chat interface (default behavior)
    setAutoSendToChat(true)
    // Disable continuous mode for chat interface (short recordings)
    setContinuousMode(false)
    console.log('🎤 ChatWindow: Speech transcription initialized for chat interface')
  } catch (error) {
    console.error('🎤 ChatWindow: Failed to initialize speech transcription:', error)
  }
  
  // Scroll to bottom on mount if there are messages
  if (chatHistory.value.length > 0) {
    await nextTick()
    scrollChatToBottom()
  }
})

// Cleanup speech events when component unmounts
onUnmounted(() => {
  removeSpeechTranscriptionListeners()
  console.log('🎤 ChatWindow: Speech transcription listeners removed')
})
</script>

<template>
  <Transition name="chat-window">
    <div v-if="showChatWindow" class="chat-window">
      <!-- Window Header -->
      <div class="window-header">
        <div class="header-title">
          <div class="flex items-center gap-2">
            <CommandLineIcon class="w-4 h-4 text-white/80" />
            <span class="text-sm font-medium text-white/90">AI Assistant</span>
            <!-- Model Selector -->
            <ModelSelector
              :selected-model="selectedModel"
              :current-agent="currentAgent"
              @update:selected-model="handleModelUpdate"
              @update:current-agent="handleAgentUpdate"
            />
            
            <!-- Context Truncation Indicator -->
            <div v-if="isContextTruncated" class="truncation-indicator" title="Chat history is being truncated to fit AI context limits">
              <ExclamationTriangleIcon class="w-3 h-3 text-yellow-400" />
              <span class="text-xs text-yellow-400">History Truncated</span>
            </div>
          </div>
          <div class="header-controls">
            <!-- Chat History Button -->
            <button 
              @click="toggleChatSidebar" 
              class="header-btn"
              :class="{ 'active': showChatSidebar }"
              title="Chat History"
            >
              <ChatBubbleLeftRightIcon class="w-4 h-4" />
            </button>
          </div>
        </div>
        <button @click="closeWindow" class="close-btn">
          <XMarkIcon class="w-4 h-4 text-white/70 hover:text-white transition-colors" />
        </button>
      </div>
      
      <!-- Window Content Container -->
      <div class="window-content">
        <!-- Chat Sidebar -->
        <div v-if="showChatSidebar" class="chat-sidebar">
          <div class="sidebar-header">
            <div class="flex items-center gap-2">
              <ChatBubbleLeftRightIcon class="w-4 h-4 text-blue-400" />
              <h3 class="text-sm font-medium text-white">Chat Sessions</h3>
            </div>
            <button @click="showChatSidebar = false" class="close-sidebar-btn">
              <XMarkIcon class="w-4 h-4" />
            </button>
          </div>
          
          <div class="sidebar-content">
            <!-- New Chat Button -->
            <button @click="handleCreateNewChat" class="new-chat-btn">
              <PlusIcon class="w-4 h-4" />
              New Chat
            </button>
            
            <!-- Chat List -->
            <div class="chat-list">
              <div v-if="sortedChats.length === 0" class="empty-state">
                <ChatBubbleLeftRightIcon class="w-8 h-8 text-white/20 mx-auto mb-2" />
                <p class="text-white/60 text-xs text-center">No chat sessions yet</p>
              </div>
              
              <div v-else class="chats-grid">
                <div 
                  v-for="chat in sortedChats" 
                  :key="chat.id"
                  class="chat-item"
                  :class="{ 'active': chat.id === currentChatId }"
                  @click="handleSwitchChat(chat.id)"
                >
                  <div class="chat-header">
                    <!-- Chat title (editable) -->
                    <div v-if="renamingChatId === chat.id" class="w-full">
                      <input
                        v-model="newChatTitle"
                        @keyup.enter="finishRenaming"
                        @keyup.escape="cancelRenaming"
                        @blur="finishRenaming"
                        class="w-full px-2 py-1 text-xs bg-white/10 border border-white/20 rounded text-white/90 focus:outline-none focus:border-blue-500/50"
                        autofocus
                      />
                    </div>
                    <span v-else class="chat-title">{{ chat.title }}</span>
                    <button 
                      @click.stop="toggleChatMenu(chat.id)"
                      class="menu-btn"
                      title="Chat options"
                    >
                      <EllipsisVerticalIcon class="w-3 h-3" />
                    </button>
                  </div>
                  
                  <div class="chat-meta">
                    <div class="meta-row">
                      <ClockIcon class="w-3 h-3 text-white/40" />
                      <span class="text-xs text-white/40">{{ formatRelativeTime(chat.updatedAt) }}</span>
                      <span class="text-xs text-white/40">•</span>
                      <span class="text-xs text-white/40">{{ chat.history.length }} messages</span>
                    </div>
                  </div>
                  
                  <!-- Dropdown menu -->
                  <div v-if="showMenuForChat === chat.id" class="chat-menu">
                    <button @click.stop="startRenaming(chat.id, chat.title)" class="menu-item">
                      <PencilIcon class="w-3 h-3" />
                      <span>Rename</span>
                    </button>
                    <button 
                      v-if="chat.id === currentChatId && chat.history.length > 0"
                      @click.stop="handleClearChat" 
                      class="menu-item"
                    >
                      <TrashIcon class="w-3 h-3" />
                      <span>Clear History</span>
                    </button>
                    <button @click.stop="handleDeleteChat(chat.id)" class="menu-item danger">
                      <TrashIcon class="w-3 h-3" />
                      <span>Delete Chat</span>
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      
        <!-- Main Content Area -->
        <div class="main-content" :class="{ 'with-sidebar': showChatSidebar }">
          <!-- Chat Messages Area -->
          <div ref="chatMessages" class="chat-area">
            <div v-if="chatHistory.length === 0" class="empty-state">
              <div class="empty-icon">
                <CommandLineIcon class="w-8 h-8 text-white/40" />
              </div>
              <p class="text-white/60 text-sm">Start a conversation</p>
              <p class="text-white/40 text-xs mt-1">
                Ask your AI assistant anything
              </p>
            </div>
            
            <div v-else class="messages-container">
              <div 
                v-for="(message, index) in chatHistory" 
                :key="index"
                class="message-wrapper"
                :class="{
                  'user-message': message.sender === 'user',
                  'assistant-message': message.sender === 'assistant',
                  'transcription-message': message.sender === 'transcription',
                  'system-message': message.sender === 'system'
                }"
              >
                <div class="message-bubble" :class="{
                  'user-bubble': message.sender === 'user',
                  'assistant-bubble': message.sender === 'assistant',
                  'transcription-bubble': message.sender === 'transcription',
                  'system-bubble': message.sender === 'system'
                }">
                  <div class="message-header">
                    <div class="message-source">
                      <span class="source-label">
                        {{ message.sender === 'user' ? 'You' : 
                           message.sender === 'assistant' ? 'Assistant' : 
                           message.sender === 'transcription' ? 'Voice' : 'System' }}
                      </span>
                    </div>
                    <span class="message-time">{{ new Date(message.timestamp).toLocaleTimeString() }}</span>
                  </div>
                  <div class="message-content">
                    <!-- Transcription messages -->
                    <div v-if="message.sender === 'transcription'" class="transcription-message">
                      <!-- Interim transcription (thought stream) -->
                      <div v-if="message.isInterim" class="interim-transcription">
                        <span class="interim-icon">💭</span>
                        <span class="interim-text">{{ message.text }}</span>
                        <span class="interim-dots">...</span>
                      </div>
                      <!-- Final transcription -->
                      <div v-else class="final-transcription">
                        <div class="transcription-content">
                          <span class="transcription-icon">🎤</span>
                          <span class="transcription-text">{{ message.text }}</span>
                        </div>
                        <div v-if="message.confidence" class="confidence-indicator">
                          {{ Math.round(message.confidence * 100) }}%
                        </div>
                      </div>
                    </div>
                    
                    <!-- Regular user/assistant messages -->
                    <div v-else>
                      <!-- Streaming text with cursor -->
                      <template v-if="message.text.endsWith('▋')">
                        <div v-html="renderMarkdown(message.text.slice(0, -1))"></div><span class="streaming-cursor">▋</span>
                      </template>
                      <!-- Regular completed text with markdown -->
                      <div v-else v-html="renderMarkdown(message.text)"></div>
                    </div>
                  </div>
                  <div v-if="message.confidence && message.sender !== 'transcription'" class="message-confidence">
                    Confidence: {{ Math.round(message.confidence * 100) }}%
                  </div>
                </div>
              </div>
            </div>
          </div>
          
          <!-- Agent Action Buttons -->
          <AgentActionButtons
            :file-input="fileInput"
            @take-screenshot="handleTakeScreenshot"
            @start-deep-research="handleStartDeepResearch"
            @start-conversational="handleStartConversational"
            @start-coding="handleStartCoding"
            @start-computer-use="handleStartComputerUse"
            @trigger-file-upload="triggerFileUpload"
            @handle-file-upload="handleFileUploadEvent"
          />
          
          <!-- Chat Input -->
          <div class="chat-input-container">
            <div class="input-wrapper">
              <input 
                v-model="chatMessage"
                @input="handleInput"
                @keydown="handleEnhancedKeydown"
                class="chat-input"
                placeholder="Ask any AI agent... (use @ to mention specific agents)"
                type="text"
              />
              
              <!-- @ Mention Suggestions -->
              <Transition name="mention-suggestions">
                <div v-if="showMentionSuggestions" class="mention-suggestions">
                  <button
                    v-for="agent in mentionSuggestions"
                    :key="agent.id"
                    @click="selectMention(agent)"
                    class="mention-suggestion"
                  >
                    <div class="mention-name">{{ agent.name }}</div>
                    <div class="mention-description">{{ agent.description }}</div>
                  </button>
                </div>
              </Transition>
            </div>
            
            <!-- Microphone Button -->
            <button 
              @click="handleMicrophoneToggle"
              :disabled="!isSpeechInitialized"
              class="chat-mic-btn"
              :class="{
                'recording': isSpeechRecording,
                'disabled': !isSpeechInitialized
              }"
              :title="isSpeechRecording ? 'Stop recording' : 'Start voice input'"
            >
              <StopIcon v-if="isSpeechRecording" class="w-4 h-4" />
              <MicrophoneIcon v-else class="w-4 h-4" />
            </button>
            
            <button @click="sendMessageWithAgent" class="chat-send-btn" :disabled="!chatMessage.trim()">
              <ShieldCheckIcon class="w-4 h-4" />
            </button>
          </div>
        </div> <!-- End main-content -->
      </div> <!-- End window-content -->
    </div>
  </Transition>
</template>

<style scoped>
.chat-window {
  @apply backdrop-blur-xl border border-white/15 rounded-2xl overflow-hidden;
  background: linear-gradient(to bottom, 
    rgba(10, 10, 12, 0.9) 0%, 
    rgba(5, 5, 7, 0.95) 100%
  );
  width: 600px;
  height: 700px;
  max-width: 95vw;
  max-height: calc(100vh - 100px);
  display: flex;
  flex-direction: column;
  position: relative;
  
  /* Enhanced glass effect similar to conversational window */
  backdrop-filter: blur(80px) saturate(180%);
  box-shadow: 
    0 25px 80px rgba(0, 0, 0, 0.6),
    0 10px 30px rgba(0, 0, 0, 0.4),
    inset 0 1px 0 rgba(255, 255, 255, 0.15);
}

/* When sidebar is shown, make window wider and use row layout */
.chat-window:has(.chat-sidebar) {
  width: 980px;
  max-width: 95vw;
}

.window-content {
  @apply flex-1 flex flex-col min-h-0;
}

.chat-window:has(.chat-sidebar) .window-content {
  @apply flex flex-row;
}

.main-content {
  @apply flex-1 flex flex-col min-h-0;
}

.window-header {
  @apply flex items-center justify-between px-4 py-3 border-b border-white/10;
  flex-shrink: 0;
}

.header-title {
  @apply flex items-center justify-between w-full mr-4;
}

.header-controls {
  @apply flex items-center gap-2;
}

.header-btn {
  @apply rounded-full p-1 hover:bg-white/10 transition-all duration-200 text-white/60 hover:text-white/90;
}

.header-btn.active {
  @apply bg-blue-500/20 text-blue-400 hover:bg-blue-500/30;
}

.model-indicator {
  @apply flex items-center px-2 py-1 bg-green-500/20 rounded-full border border-green-500/30;
}

.truncation-indicator {
  @apply flex items-center gap-1 px-2 py-1 bg-yellow-500/20 rounded-full border border-yellow-500/30;
}

.close-btn {
  @apply rounded-full p-1 hover:bg-white/10 transition-colors;
}

/* Chat Sidebar */
.chat-sidebar {
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

.new-chat-btn {
  @apply w-full flex items-center justify-center gap-2 mx-4 my-3 px-4 py-2 bg-blue-500/80 hover:bg-blue-500 text-white rounded-lg transition-all duration-200 font-medium text-sm;
}

.chat-list {
  @apply flex-1 overflow-y-auto px-4 pb-4;
}

.empty-state {
  @apply flex flex-col items-center justify-center h-full text-center p-8;
}

.empty-icon {
  @apply mb-4 p-4 rounded-full bg-white/5;
}

.chats-grid {
  @apply space-y-2;
}

.chat-item {
  @apply rounded-lg bg-white/5 hover:bg-white/10 transition-all duration-200 cursor-pointer p-3 border border-transparent relative;
}

.chat-item.active {
  @apply border-blue-500/50 bg-blue-500/10;
}

.chat-header {
  @apply flex items-center justify-between mb-2;
}

.chat-title {
  @apply text-xs font-medium text-white truncate flex-1 mr-2;
}

.menu-btn {
  @apply rounded-full p-1 hover:bg-white/10 transition-colors text-white/60 hover:text-white/90;
}

.chat-meta {
  @apply space-y-1 mb-2;
}

.meta-row {
  @apply flex items-center gap-1;
}

.chat-menu {
  @apply absolute right-3 top-12 bg-black/95 border border-white/20 rounded-lg shadow-xl z-50 py-1 min-w-32;
}

.menu-item {
  @apply w-full flex items-center gap-2 px-3 py-2 text-sm hover:bg-white/10 transition-colors text-white/80 hover:text-white/90;
}

.menu-item.danger {
  @apply text-red-400 hover:text-red-300 hover:bg-red-500/10;
}

/* Chat Area */
.chat-area {
  @apply flex-1 overflow-y-auto;
  min-height: 0;
}

.messages-container {
  @apply p-4 space-y-4;
}

.message-wrapper {
  @apply flex;
}

.message-wrapper.user-message {
  @apply justify-end;
}

.message-wrapper.assistant-message,
.message-wrapper.transcription-message,
.message-wrapper.system-message {
  @apply justify-start;
}

.message-bubble {
  @apply rounded-2xl p-3 max-w-xs;
  word-wrap: break-word;
}

.user-bubble {
  @apply bg-blue-500/80 text-white;
  border-bottom-right-radius: 6px;
}

.assistant-bubble,
.system-bubble {
  @apply text-white/90 border border-white/10;
  background: rgba(255, 255, 255, 0.05);
  border-bottom-left-radius: 6px;
}

.transcription-bubble {
  @apply bg-purple-500/20 text-white/90 border border-purple-400/30;
  border-bottom-left-radius: 6px;
}

.message-header {
  @apply flex items-center justify-between mb-1;
}

.message-source {
  @apply flex items-center gap-1 text-xs opacity-75;
}

.source-label {
  @apply font-medium;
}

.message-time {
  @apply text-xs opacity-60;
}

.message-content {
  @apply text-sm leading-relaxed;
}

.message-confidence {
  @apply text-xs opacity-60 mt-1;
}

/* Transcription specific styles */
.transcription-message {
  @apply w-full;
}

.interim-transcription {
  @apply flex items-center gap-2;
}

.interim-icon {
  @apply text-orange-300 text-sm;
}

.interim-text {
  @apply italic text-orange-200;
}

.interim-dots {
  @apply text-orange-400 animate-pulse font-bold;
}

.final-transcription {
  @apply flex flex-col gap-1;
}

.transcription-content {
  @apply flex items-center gap-2;
}

.transcription-icon {
  @apply text-green-400 text-sm;
}

.transcription-text {
  @apply text-white/90;
}

.confidence-indicator {
  @apply text-xs text-white/60 mt-1;
}

/* Chat Input */
.chat-input-container {
  @apply flex items-center gap-3 px-4 py-4 border-t border-white/10;
  flex-shrink: 0;
  background: rgba(0, 0, 0, 0.2);
}

.input-wrapper {
  @apply flex-1 relative;
}

.chat-input {
  @apply w-full border border-white/10 rounded-xl px-4 py-3 text-white placeholder-white/50 focus:outline-none focus:border-blue-500/50 focus:ring-2 focus:ring-blue-500/20 transition-all duration-200;
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(10px);
  font-size: 14px;
}

.chat-input:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.2);
}

.chat-input:focus {
  background: rgba(255, 255, 255, 0.1);
}

/* @ Mention Suggestions */
.mention-suggestions {
  @apply absolute bottom-full left-0 right-0 mb-2 bg-black/95 border border-white/20 rounded-xl shadow-xl backdrop-blur-sm z-50 max-h-32 overflow-y-auto;
}

.mention-suggestion {
  @apply w-full p-3 text-left hover:bg-white/10 transition-colors duration-200 border-b border-white/10 last:border-b-0;
}

.mention-suggestion:first-child {
  @apply rounded-t-xl;
}

.mention-suggestion:last-child {
  @apply rounded-b-xl;
}

.mention-name {
  @apply text-sm font-medium text-blue-400;
}

.mention-description {
  @apply text-xs text-white/60 mt-0.5;
}

/* Mention Suggestions Transition */
.mention-suggestions-enter-active,
.mention-suggestions-leave-active {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.mention-suggestions-enter-from {
  opacity: 0;
  transform: translateY(10px) scale(0.95);
}

.mention-suggestions-leave-to {
  opacity: 0;
  transform: translateY(10px) scale(0.95);
}

/* Chat Microphone Button */
.chat-mic-btn {
  @apply rounded-xl p-3 transition-all duration-200 flex items-center justify-center;
  background: linear-gradient(135deg, rgba(34, 197, 94, 0.8), rgba(22, 163, 74, 0.8));
  border: 1px solid rgba(34, 197, 94, 0.4);
  color: white;
  min-width: 44px;
  min-height: 44px;
}

.chat-mic-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, rgba(34, 197, 94, 0.9), rgba(22, 163, 74, 0.9));
  border-color: rgba(34, 197, 94, 0.6);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(34, 197, 94, 0.3);
}

.chat-mic-btn.recording {
  @apply animate-pulse;
  background: linear-gradient(135deg, rgba(239, 68, 68, 0.8), rgba(220, 38, 38, 0.8));
  border-color: rgba(239, 68, 68, 0.6);
}

.chat-mic-btn.recording:hover {
  background: linear-gradient(135deg, rgba(239, 68, 68, 0.9), rgba(220, 38, 38, 0.9));
  border-color: rgba(239, 68, 68, 0.7);
}

.chat-mic-btn.disabled {
  background: rgba(75, 85, 99, 0.8);
  border-color: rgba(75, 85, 99, 0.4);
  color: rgba(255, 255, 255, 0.3);
  cursor: not-allowed;
}

.chat-send-btn {
  @apply rounded-xl p-3 transition-all duration-200 flex items-center justify-center;
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.8), rgba(37, 99, 235, 0.8));
  border: 1px solid rgba(59, 130, 246, 0.4);
  color: white;
  min-width: 44px;
  min-height: 44px;
}

.chat-send-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.9), rgba(37, 99, 235, 0.9));
  border-color: rgba(59, 130, 246, 0.6);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
}

.chat-send-btn:disabled {
  background: rgba(255, 255, 255, 0.05);
  border-color: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.3);
  cursor: not-allowed;
}

/* Streaming cursor */
.streaming-cursor {
  animation: blink 1s infinite;
  color: #60a5fa;
  font-weight: bold;
}

@keyframes blink {
  0%, 50% {
    opacity: 1;
  }
  51%, 100% {
    opacity: 0;
  }
}

/* Transitions */
.chat-window-enter-active,
.chat-window-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.chat-window-enter-from {
  opacity: 0;
  transform: translateY(-10px) scale(0.98);
}

.chat-window-leave-to {
  opacity: 0;
  transform: translateY(-10px) scale(0.98);
}

/* Scrollbar */
.chat-area::-webkit-scrollbar,
.chat-list::-webkit-scrollbar {
  width: 4px;
}

.chat-area::-webkit-scrollbar-track,
.chat-list::-webkit-scrollbar-track {
  background: transparent;
}

.chat-area::-webkit-scrollbar-thumb,
.chat-list::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
}
</style> 