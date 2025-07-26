<script setup lang="ts">
import { ref, watch, nextTick, toRef, onMounted, onUnmounted, computed } from 'vue'
import {
  CommandLineIcon,
  XMarkIcon,
  ArrowsPointingOutIcon,
  ShieldCheckIcon,
  ExclamationTriangleIcon,
  Bars3Icon
} from '@heroicons/vue/24/outline'
import { useChatManagement } from '../../composables/useChatManagement'
import { useWindowResizing } from '../../composables/useWindowResizing'
import { useSpeechEvents } from '../../composables/useSpeechEvents'
import AgentActionButtons from './AgentActionButtons.vue'

interface Props {
  showChatWindow: boolean
  selectedModel: string | null
}

interface Emits {
  (e: 'close'): void
  (e: 'update:showChatWindow', value: boolean): void
  (e: 'toggle-chat-drawer'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// Refs for scrolling
const chatMessages = ref<HTMLElement>()

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



// Setup speech events when component mounts
onMounted(async () => {
  setupSpeechTranscriptionListeners()
  console.log('🎤 ChatWindow: Speech transcription listeners set up')
  
})

// Cleanup speech events when component unmounts
onUnmounted(() => {
  removeSpeechTranscriptionListeners()
  console.log('🎤 ChatWindow: Speech transcription listeners removed')
})
</script>

<template>
  <Transition name="chat-window">
    <div v-if="showChatWindow" class="chat-window-section">
      <div 
        class="chat-window" 
        :class="{ 'resizing': isResizing }"
        :style="{ 
          width: chatWindowSize.width + 'px', 
          height: chatWindowSize.height + 'px' 
        }"
      >
        <!-- Chat Drawer Trigger Button -->
        <button 
          @click="toggleChatDrawer"
          class="chat-drawer-trigger"
          title="Chat History"
        >
          <Bars3Icon class="w-4 h-4 text-white/70 hover:text-white transition-colors" />
        </button>

        <!-- Chat Header with Resize Indicator -->
        <div class="chat-header">
          <div class="chat-title">
            <CommandLineIcon class="w-4 h-4 text-white/80" />
            <span class="text-sm font-medium text-white/90">AI Assistant</span>
            <div class="model-indicator" v-if="selectedModel">
              <span class="text-xs text-green-400">{{ selectedModel.split(':')[0] || selectedModel }}</span>
            </div>
            
            <!-- Context Truncation Indicator -->
            <div v-if="isContextTruncated" class="truncation-indicator" title="Chat history is being truncated to fit AI context limits">
              <ExclamationTriangleIcon class="w-3 h-3 text-yellow-400" />
              <span class="text-xs text-yellow-400">History Truncated</span>
            </div>
            
            <div class="resize-indicator">
              <ArrowsPointingOutIcon class="w-3 h-3 text-white/50" />
              <span class="text-xs text-white/50">{{ chatWindowSize.width }}×{{ chatWindowSize.height }}</span>
            </div>
          </div>
          <button @click="closeWindow" class="chat-close-btn">
            <XMarkIcon class="w-4 h-4 text-white/70 hover:text-white transition-colors" />
          </button>
        </div>
        
        <div class="chat-messages" ref="chatMessages" 
             :style="{ height: (chatWindowSize.height - 120) + 'px' }">
          <div v-if="chatHistory.length === 0" class="chat-empty">
            <CommandLineIcon class="w-6 h-6 text-white/40 mb-2" />
            <p class="text-white/60 text-sm">Start a conversation with your AI assistant</p>
          </div>
          
          <div v-for="(message, index) in chatHistory" :key="index" class="chat-message"
               :class="{ 
                 'user': message.sender === 'user', 
                 'assistant': message.sender === 'assistant',
                 'transcription': message.sender === 'transcription',
                 'system': message.sender === 'system'
               }">
            <div class="message-bubble">
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
              <div v-else class="message-text">
                <!-- Streaming text with cursor -->
                <template v-if="message.text.endsWith('▋')">
                  <div v-html="renderMarkdown(message.text.slice(0, -1))"></div><span class="streaming-cursor">▋</span>
                </template>
                <!-- Regular completed text with markdown -->
                <div v-else v-html="renderMarkdown(message.text)"></div>
              </div>
              
              <span class="message-time">{{ new Date(message.timestamp).toLocaleTimeString() }}</span>
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
        
        
        <div class="chat-input-container">
          <input 
            v-model="chatMessage"
            @keydown="handleChatKeydown"
            class="chat-input"
            placeholder="Ask any AI agent..."
            type="text"
          />
          <button @click="() => sendMessage()" class="chat-send-btn" :disabled="!chatMessage.trim()">
            <ShieldCheckIcon class="w-4 h-4" />
          </button>
        </div>

        <!-- Resize Handles -->
        <div class="resize-handles">
          <!-- Corner handles -->
          <div class="resize-handle corner top-left" @mousedown="startResize($event, 'top-left')"></div>
          <div class="resize-handle corner top-right" @mousedown="startResize($event, 'top-right')"></div>
          <div class="resize-handle corner bottom-left" @mousedown="startResize($event, 'bottom-left')"></div>
          <div class="resize-handle corner bottom-right" @mousedown="startResize($event, 'bottom-right')"></div>
          
          <!-- Edge handles -->
          <div class="resize-handle edge top" @mousedown="startResize($event, 'top')"></div>
          <div class="resize-handle edge bottom" @mousedown="startResize($event, 'bottom')"></div>
          <div class="resize-handle edge left" @mousedown="startResize($event, 'left')"></div>
          <div class="resize-handle edge right" @mousedown="startResize($event, 'right')"></div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.chat-window-section {
  @apply w-full flex justify-center;
  padding: 8px 8px 8px 8px; /* Ensure top padding for menu button visibility */
  background: transparent;
  /* Ensure the section doesn't get cut off */
  min-height: 100%;
  box-sizing: border-box;
}

/* Chat Window Styles */
.chat-window {
  @apply rounded-2xl overflow-hidden relative;
  pointer-events: auto;
  min-width: 450px;
  min-height: 400px;
  max-width: 800px;
  max-height: calc(100vh - 80px); /* Ensure it fits within viewport with margin */
  
  /* Same glass effect as control panel with darker background */
  background: linear-gradient(135deg, 
    rgba(17, 17, 21, 0.85) 0%,
    rgba(17, 17, 21, 0.75) 25%,
    rgba(17, 17, 21, 0.70) 50%,
    rgba(17, 17, 21, 0.75) 75%,
    rgba(17, 17, 21, 0.85) 100%
  );
  backdrop-filter: blur(60px) saturate(180%) brightness(1.1);
  border: 1px solid rgba(255, 255, 255, 0.25);
  box-shadow: 
    0 20px 60px rgba(0, 0, 0, 0.4),
    0 8px 24px rgba(0, 0, 0, 0.25),
    inset 0 1px 0 rgba(255, 255, 255, 0.3),
    inset 0 -1px 0 rgba(0, 0, 0, 0.1);

  transition: all 0.2s ease;
}

.chat-window.resizing {
  box-shadow: 
    0 25px 70px rgba(0, 0, 0, 0.5),
    0 12px 30px rgba(0, 0, 0, 0.3),
    inset 0 1px 0 rgba(255, 255, 255, 0.4),
    inset 0 -1px 0 rgba(0, 0, 0, 0.1);
  border-color: rgba(255, 255, 255, 0.35);
}

.chat-header {
  @apply flex items-center justify-between px-4 py-3 border-b border-white/10;
}

.chat-title {
  @apply flex items-center gap-2;
}

.model-indicator {
  @apply flex items-center px-2 py-1 bg-green-500/20 rounded-full border border-green-500/30;
}

.truncation-indicator {
  @apply flex items-center gap-1 px-2 py-1 bg-yellow-500/20 rounded-full border border-yellow-500/30;
}

.resize-indicator {
  @apply flex items-center gap-1;
}

.chat-close-btn {
  @apply rounded-full p-1 hover:bg-white/10 transition-colors;
}

.chat-drawer-trigger {
  @apply absolute z-10 rounded-full p-2 bg-white/10 hover:bg-white/20 transition-colors;
  backdrop-filter: blur(8px);
  border: 1px solid rgba(255, 255, 255, 0.2);
  /* Position it safely within the chat window bounds */
  top: 8px;
  left: 8px;
  /* Ensure it's always visible */
  min-width: 36px;
  min-height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.chat-messages {
  @apply flex-1 overflow-y-auto px-4 py-3;
  max-height: calc(100% - 120px); /* Ensure it doesn't overflow the window */
  scroll-behavior: smooth; /* Smooth scrolling animation */
  scrollbar-width: thin;
  scrollbar-color: rgba(255, 255, 255, 0.4) rgba(255, 255, 255, 0.1);
}

.chat-messages::-webkit-scrollbar {
  width: 8px; /* Made wider for better visibility */
}

.chat-messages::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 4px;
}

.chat-messages::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.3); /* Made more visible */
  border-radius: 4px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.chat-messages::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.5); /* Even more visible on hover */
}

.chat-empty {
  @apply flex flex-col items-center justify-center h-full text-center;
}

.chat-message {
  @apply mb-4;
}

.chat-message:last-child {
  @apply mb-6; /* Extra margin for the last message to ensure good scroll space */
}

.chat-message.user {
  @apply flex justify-end;
}

.chat-message.assistant {
  @apply flex justify-start;
}

.chat-message.transcription {
  @apply flex justify-start;
}

.message-bubble {
  @apply max-w-sm px-4 py-3 rounded-2xl; /* Increased max width and padding for better readability */
}

.chat-message.user .message-bubble {
  @apply bg-blue-500/80 text-white;
}

.chat-message.assistant .message-bubble {
  @apply bg-white/15 text-white/90;
}

.chat-message.transcription .message-bubble {
  @apply bg-purple-500/20 text-white/90 border border-purple-400/30;
}

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

.message-text {
  @apply text-sm leading-relaxed;
}

.message-time {
  @apply text-xs opacity-70 mt-1 block;
}

.chat-input-container {
  @apply flex items-center gap-2 px-4 py-3 border-t border-white/10;
}

.chat-input {
  @apply flex-1 bg-white/10 border border-white/20 rounded-full px-4 py-2 text-white placeholder-white/50 focus:outline-none focus:border-white/40 transition-colors;
}

.chat-send-btn {
  @apply bg-blue-500/80 hover:bg-blue-500 disabled:bg-white/10 disabled:opacity-50 rounded-full p-2 transition-colors;
  color: white;
}

.chat-send-btn:disabled {
  cursor: not-allowed;
}

/* Resize Handles */
.resize-handles {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  pointer-events: none;
}

.resize-handle {
  position: absolute;
  pointer-events: auto;
  transition: background-color 0.2s ease;
}

/* Corner handles */
.resize-handle.corner {
  width: 16px;
  height: 16px;
  background: rgba(255, 255, 255, 0.15);
  border-radius: 50%;
  border: 1px solid rgba(255, 255, 255, 0.3);
}

.resize-handle.corner:hover {
  background: rgba(255, 255, 255, 0.3);
  border-color: rgba(255, 255, 255, 0.4);
}

.resize-handle.top-left {
  top: -8px;
  left: -8px;
  cursor: nw-resize;
}

.resize-handle.top-right {
  top: -8px;
  right: -8px;
  cursor: ne-resize;
}

.resize-handle.bottom-left {
  bottom: -8px;
  left: -8px;
  cursor: sw-resize;
}

.resize-handle.bottom-right {
  bottom: -8px;
  right: -8px;
  cursor: se-resize;
}

/* Edge handles */
.resize-handle.edge {
  background: transparent;
}

.resize-handle.edge:hover {
  background: rgba(255, 255, 255, 0.1);
}

.resize-handle.top {
  top: -4px;
  left: 16px;
  right: 16px;
  height: 8px;
  cursor: n-resize;
}

.resize-handle.bottom {
  bottom: -4px;
  left: 16px;
  right: 16px;
  height: 8px;
  cursor: s-resize;
}

.resize-handle.left {
  left: -4px;
  top: 16px;
  bottom: 16px;
  width: 8px;
  cursor: w-resize;
}

.resize-handle.right {
  right: -4px;
  top: 16px;
  bottom: 16px;
  width: 8px;
  cursor: e-resize;
}

/* Chat Window Transitions */
.chat-window-enter-active,
.chat-window-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.chat-window-enter-from {
  opacity: 0;
  transform: translateY(-10px) scale(0.95);
}

.chat-window-leave-to {
  opacity: 0;
  transform: translateY(-10px) scale(0.95);
}

/* Style for streaming cursor */
.streaming-cursor {
  animation: blink 1s infinite;
  color: #60a5fa;
  font-weight: bold;
}

/* Blinking cursor animation for streaming responses */
@keyframes blink {
  0%, 50% {
    opacity: 1;
  }
  51%, 100% {
    opacity: 0;
  }
}

/* Prevent text selection during resize */
.chat-window.resizing {
  user-select: none;
}

.chat-window.resizing * {
  user-select: none;
}

</style> 