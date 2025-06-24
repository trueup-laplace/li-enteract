<script setup lang="ts">
import { ref, nextTick, onMounted, onUnmounted, watch } from 'vue'
import { 
  PaperAirplaneIcon,
  XMarkIcon,
  MicrophoneIcon,
  SpeakerWaveIcon,
  ExclamationTriangleIcon
} from '@heroicons/vue/24/outline'
import { useAppStore } from '../../stores/app'
import type { ChatMessage } from '../../types'

const appStore = useAppStore()

const newMessage = ref('')
const apiKey = ref('')
const showApiKeyInput = ref(false)
const messagesContainer = ref<HTMLElement>()
const editingMessageId = ref<number | null>(null)
const editText = ref('')
const editInput = ref<HTMLInputElement>()

// Speech transcription state
const isTranscribing = ref(false)
const currentTranscript = ref('')
const transcriptionStatus = ref<'idle' | 'listening' | 'processing' | 'error'>('idle')
const transcriptionError = ref<string | null>(null)
const showWakeWordFeedback = ref(false)

onMounted(() => {
  appStore.initializeSpeechTranscription()
  setupEventListeners()
})

onUnmounted(() => {
  removeEventListeners()
})

// Setup event listeners for speech transcription
function setupEventListeners() {
  // Wake word detection events
  window.addEventListener('wake-word-detected', handleWakeWordDetected as EventListener)
  window.addEventListener('wake-word-feedback', handleWakeWordFeedback as EventListener)
  window.addEventListener('show-chat-drawer', handleShowChatDrawer as EventListener)
  
  // Speech transcription events
  window.addEventListener('transcription-started', handleTranscriptionStarted as EventListener)
  window.addEventListener('transcription-interim', handleTranscriptionInterim as EventListener)
  window.addEventListener('transcription-final', handleTranscriptionFinal as EventListener)
  window.addEventListener('transcription-error', handleTranscriptionError as EventListener)
  window.addEventListener('transcription-stopped', handleTranscriptionStopped as EventListener)
  window.addEventListener('transcription-complete', handleTranscriptionComplete as EventListener)
  window.addEventListener('transcription-auto-stopped', handleTranscriptionAutoStopped as EventListener)
  window.addEventListener('start-transcription-from-wake-word', handleStartTranscriptionFromWakeWord as EventListener)
}

function removeEventListeners() {
  window.removeEventListener('wake-word-detected', handleWakeWordDetected)
  window.removeEventListener('wake-word-feedback', handleWakeWordFeedback)
  window.removeEventListener('show-chat-drawer', handleShowChatDrawer)
  window.removeEventListener('transcription-started', handleTranscriptionStarted)
  window.removeEventListener('transcription-interim', handleTranscriptionInterim)
  window.removeEventListener('transcription-final', handleTranscriptionFinal)
  window.removeEventListener('transcription-error', handleTranscriptionError)
  window.removeEventListener('transcription-stopped', handleTranscriptionStopped)
  window.removeEventListener('transcription-complete', handleTranscriptionComplete)
  window.removeEventListener('transcription-auto-stopped', handleTranscriptionAutoStopped)
  window.removeEventListener('start-transcription-from-wake-word', handleStartTranscriptionFromWakeWord)
}

// Event handlers
function handleWakeWordDetected(event: CustomEvent) {
  console.log('ChatDrawer: Wake word detected!', event.detail)
  showWakeWordFeedback.value = true
  setTimeout(() => {
    showWakeWordFeedback.value = false
  }, 3000)
  
  // Auto-open chat drawer
  if (!appStore.chatOpen) {
    appStore.toggleChat()
  }
}

function handleWakeWordFeedback(event: CustomEvent) {
  console.log('ChatDrawer: Wake word feedback', event.detail)
  showWakeWordFeedback.value = true
  setTimeout(() => {
    showWakeWordFeedback.value = false
  }, 2000)
}

function handleShowChatDrawer(event: CustomEvent) {
  console.log('ChatDrawer: Show chat drawer requested', event.detail)
  if (!appStore.chatOpen) {
    appStore.toggleChat()
  }
}

function handleStartTranscriptionFromWakeWord(event: CustomEvent) {
  console.log('ChatDrawer: Starting transcription from wake word', event.detail)
  transcriptionStatus.value = 'listening'
  isTranscribing.value = true
  currentTranscript.value = ''
  transcriptionError.value = null
  
  // Auto-open drawer and scroll to bottom
  if (!appStore.chatOpen) {
    appStore.toggleChat()
  }
  scrollToBottom()
  
  // Start transcription via app store
  appStore.startSpeechTranscription()
}

function handleTranscriptionStarted(event: CustomEvent) {
  console.log('ChatDrawer: Transcription started', event.detail)
  transcriptionStatus.value = 'listening'
  isTranscribing.value = true
  currentTranscript.value = ''
  transcriptionError.value = null
  scrollToBottom()
}

function handleTranscriptionInterim(event: CustomEvent) {
  console.log('ChatDrawer: Interim transcription', event.detail)
  transcriptionStatus.value = 'processing'
  currentTranscript.value = event.detail.text || ''
  
  // Add interim message to chat if not already present
  const lastMessage = appStore.chatMessages[appStore.chatMessages.length - 1]
  if (!lastMessage || !lastMessage.isInterim) {
    appStore.addMessage(currentTranscript.value, 'transcription', { 
      source: 'speech',
      isInterim: true,
      confidence: event.detail.confidence 
    })
  } else {
    // Update existing interim message
    appStore.updateMessage(lastMessage.id, currentTranscript.value)
  }
  
  scrollToBottom()
}

function handleTranscriptionFinal(event: CustomEvent) {
  console.log('ChatDrawer: Final transcription', event.detail)
  transcriptionStatus.value = 'idle'
  
  // Replace interim message with final one or add new final message
  const lastMessage = appStore.chatMessages[appStore.chatMessages.length - 1]
  if (lastMessage && lastMessage.isInterim) {
    // Update interim message to final
    appStore.updateMessage(lastMessage.id, event.detail.text, { isInterim: false })
  } else {
    // Add new final message
    appStore.addMessage(event.detail.text, 'transcription', { 
      source: 'speech',
      isInterim: false,
      confidence: event.detail.confidence 
    })
  }
  
  scrollToBottom()
}

function handleTranscriptionError(event: CustomEvent) {
  console.log('ChatDrawer: Transcription error', event.detail)
  transcriptionStatus.value = 'error'
  transcriptionError.value = event.detail.error
  isTranscribing.value = false
}

function handleTranscriptionStopped(event: CustomEvent) {
  console.log('ChatDrawer: Transcription stopped', event.detail)
  transcriptionStatus.value = 'idle'
  isTranscribing.value = false
}

function handleTranscriptionComplete(event: CustomEvent) {
  console.log('ChatDrawer: Transcription complete', event.detail)
  transcriptionStatus.value = 'idle'
  isTranscribing.value = false
  currentTranscript.value = ''
}

function handleTranscriptionAutoStopped(event: CustomEvent) {
  console.log('ChatDrawer: Transcription auto-stopped', event.detail)
  transcriptionStatus.value = 'idle'
  isTranscribing.value = false
  
  // Add system message about auto-stop
  appStore.addMessage(`Transcription stopped automatically (${event.detail.reason})`, 'system', { 
    source: 'auto-stop' 
  })
  scrollToBottom()
}

const sendMessage = () => {
  if (newMessage.value.trim()) {
    appStore.addMessage(newMessage.value, 'user', { source: 'typed' })
    newMessage.value = ''
    scrollToBottom()
  }
}

const toggleSpeechTranscription = () => {
  if (appStore.speechStatus.isRecording) {
    appStore.stopSpeechTranscription()
  } else {
    appStore.startSpeechTranscription()
  }
}

const startEditing = (message: ChatMessage) => {
  if (message.sender === 'transcription' && !message.isInterim) {
    editingMessageId.value = message.id
    editText.value = message.text
    nextTick(() => {
      editInput.value?.focus()
      editInput.value?.select()
    })
  }
}

const saveEdit = () => {
  if (editingMessageId.value && editText.value.trim()) {
    appStore.updateMessage(editingMessageId.value, editText.value.trim())
  }
  cancelEdit()
}

const cancelEdit = () => {
  editingMessageId.value = null
  editText.value = ''
}

const scrollToBottom = () => {
  nextTick(() => {
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
    }
  })
}

// Auto-scroll when new messages arrive
watch(() => appStore.chatMessages.length, scrollToBottom)
</script>

<template>
  <!-- Chat Drawer -->
  <div 
    class="fixed top-0 right-0 h-full w-96 z-50 transform transition-all duration-500 ease-out"
    :class="appStore.chatOpen ? 'translate-x-0' : 'translate-x-full'"
  >
    <div class="h-full chat-panel flex flex-col">
      <!-- Chat Header -->
      <div class="flex items-center justify-between p-4 border-b border-white/10">
        <div class="flex items-center gap-3">
          <div class="w-2 h-2 rounded-full bg-green-400 animate-pulse"></div>
          <h3 class="text-lg font-medium text-white/90">Assistant Chat</h3>
        </div>
        <button @click="appStore.toggleChat" class="btn btn-sm btn-circle btn-ghost hover:bg-white/10">
          <XMarkIcon class="w-4 h-4 text-white/70" />
        </button>
      </div>
      
      <!-- Chat Messages -->
      <div 
        ref="messagesContainer" 
        class="flex-1 p-4 space-y-3 overflow-y-auto max-h-[300px] scrollbar-thin scrollbar-thumb-white/20 scrollbar-track-transparent"
      >
        <div
          v-for="message in appStore.chatMessages"
          :key="message.id"
          :class="[
            'p-3 rounded-lg max-w-[80%] break-words',
            message.sender === 'user' 
              ? 'bg-blue-600 text-white ml-auto' 
              : message.sender === 'transcription'
              ? `${message.isInterim ? 'bg-orange-500/20 text-orange-200 border border-orange-500/30' : 'bg-green-600/20 text-green-200 border border-green-600/30'} mr-auto`
              : 'bg-gray-700 text-white mr-auto'
          ]"
        >
          <!-- Editable transcription message -->
          <div 
            v-if="message.sender === 'transcription'"
            @click="startEditing(message)"
            class="cursor-pointer group relative"
          >
            <div v-if="editingMessageId === message.id" class="space-y-2">
              <input
                v-model="editText"
                @keyup.enter="saveEdit"
                @keyup.escape="cancelEdit"
                @blur="saveEdit"
                class="w-full bg-transparent border-b border-white/30 text-white outline-none"
                :placeholder="message.text"
                ref="editInput"
              />
              <div class="text-xs text-white/60">
                Press Enter to save, Esc to cancel
              </div>
            </div>
            <div v-else>
              <!-- Thought stream animation for interim messages -->
              <div 
                v-if="message.isInterim"
                :class="['flex items-center', message.isInterim ? 'animate-pulse' : '']"
              >
                <span class="text-orange-300 mr-2">💭</span>
                <span :class="message.isInterim ? 'italic' : ''">{{ message.text }}</span>
                <span v-if="message.isInterim" class="ml-2 text-orange-400 animate-pulse">...</span>
              </div>
              <!-- Final transcription -->
              <div v-else class="flex items-center">
                <span class="text-green-300 mr-2">🎤</span>
                <span>{{ message.text }}</span>
              </div>
              
              <!-- Edit hint on hover -->
              <div class="absolute -top-6 right-0 text-xs text-white/60 opacity-0 group-hover:opacity-100 transition-opacity">
                Click to edit
              </div>
              
              <!-- Confidence indicator for final messages -->
              <div v-if="!message.isInterim && message.confidence" class="text-xs text-white/60 mt-1">
                Confidence: {{ Math.round(message.confidence * 100) }}%
              </div>
            </div>
          </div>
          
          <!-- Regular messages -->
          <div v-else>
            {{ message.text }}
          </div>
          
          <div class="text-xs opacity-60 mt-1">
            {{ message.timestamp.toLocaleTimeString() }}
          </div>
        </div>
      </div>
      
      <!-- Chat Input -->
      <div class="p-4 border-t border-white/10">
        <!-- Speech Transcription Info -->
        <div v-if="appStore.speechStatus.error" class="mb-3 text-sm text-red-300 bg-red-500/10 border border-red-500/20 rounded-lg p-2">
          Speech Error: {{ appStore.speechStatus.error }}
        </div>

        <!-- Clear Chat Button -->
        <div v-if="appStore.isTranscriptionEnabled" class="mb-3 flex justify-center">
          <button 
            @click="appStore.clearChat"
            class="btn-clear text-xs px-3 py-1"
            title="Clear All Chat Messages"
          >
            Clear Chat
          </button>
        </div>

        <!-- Text Input with Minimal Mic -->
        <div class="flex gap-2">
          <input
            v-model="newMessage"
            @keyup.enter="sendMessage"
            type="text" 
            placeholder="Type your message..."
            class="input-enhanced flex-1"
          />
          
          <!-- Minimal Mic Button -->
          <button
            @click="toggleSpeechTranscription"
            class="btn-mic-minimal"
            :class="{
              'btn-mic-recording': appStore.speechStatus.isRecording,
              'btn-mic-processing': appStore.speechStatus.isProcessing,
              'btn-mic-ready': appStore.isTranscriptionEnabled && !appStore.speechStatus.isRecording
            }"
            :disabled="appStore.speechStatus.isProcessing"
            :title="appStore.speechStatus.isRecording ? 'Stop Recording' : 'Start Speech Recording'"
          >
            <MicrophoneIcon class="w-4 h-4" />
          </button>
          
          <button
            @click="sendMessage"
            class="btn-send"
            :disabled="!newMessage.trim()"
          >
            <PaperAirplaneIcon class="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  </div>
  
  <!-- Backdrop -->
  <div 
    v-if="appStore.chatOpen"
    @click="appStore.toggleChat"
    class="fixed inset-0 bg-black/20 backdrop-blur-sm z-40 transition-all duration-500"
  ></div>
</template>

<style scoped>
.chat-panel {
  background: linear-gradient(
    135deg,
    rgba(0, 0, 0, 0.4) 0%,
    rgba(0, 0, 0, 0.3) 50%,
    rgba(0, 0, 0, 0.2) 100%
  );
  backdrop-filter: blur(40px) saturate(180%);
  border-left: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 
    -20px 0 60px rgba(0, 0, 0, 0.3),
    inset 1px 0 0 rgba(255, 255, 255, 0.1);
}

.input-enhanced {
  @apply bg-white/5 border border-white/20 rounded-xl px-4 py-3 text-white placeholder-white/50 focus:bg-white/10 focus:border-white/30 transition-all;
  backdrop-filter: blur(20px);
}

.input-enhanced:focus {
  outline: none;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.3);
}

.btn-send {
  @apply w-12 h-12 rounded-xl bg-gradient-to-r from-blue-500 to-purple-600 hover:from-blue-600 hover:to-purple-700 text-white flex items-center justify-center transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed;
  box-shadow: 0 4px 15px rgba(59, 130, 246, 0.3);
}

.btn-send:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(59, 130, 246, 0.4);
}

.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 2px;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
}

.btn-clear {
  @apply bg-white/10 hover:bg-white/20 text-white/70 hover:text-white rounded-lg transition-all duration-200;
}

/* Minimal mic button */
.btn-mic-minimal {
  @apply w-12 h-12 rounded-xl bg-white/5 hover:bg-white/10 border border-white/20 hover:border-white/30 text-white/70 hover:text-white flex items-center justify-center transition-all duration-200;
}

.btn-mic-ready {
  @apply bg-green-500/20 hover:bg-green-500/30 border-green-400/40 text-green-300;
}

.btn-mic-recording {
  @apply bg-red-500/30 border-red-400/50 text-red-300 animate-pulse;
}

.btn-mic-processing {
  @apply bg-yellow-500/20 border-yellow-400/40 text-yellow-300;
}

@keyframes fade-in {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.animate-fade-in {
  animation: fade-in 0.3s ease-out;
}
</style> 