<script setup lang="ts">
import { ref, nextTick, onMounted, onUnmounted, watch } from 'vue'
import { 
  PaperAirplaneIcon,
  MicrophoneIcon
} from '@heroicons/vue/24/outline'
import { useAppStore } from '../../stores/app'
import type { ChatMessage } from '../../types'

const appStore = useAppStore()

const newMessage = ref('')
const messagesContainer = ref<HTMLElement>()
const editingMessageId = ref<number | null>(null)
const editText = ref('')
const editInput = ref<HTMLInputElement>()

// Speech transcription state
const isTranscribing = ref(false)
const currentTranscript = ref('')
const transcriptionStatus = ref<'idle' | 'listening' | 'processing' | 'error'>('idle')
const transcriptionError = ref<string | null>(null)

onMounted(() => {
  appStore.initializeSpeechTranscription()
  setupEventListeners()
})

onUnmounted(() => {
  removeEventListeners()
})

// Setup event listeners for speech transcription
function setupEventListeners() {
  // Speech transcription events
  window.addEventListener('transcription-started', handleTranscriptionStarted)
  window.addEventListener('transcription-interim', handleTranscriptionInterim)
  window.addEventListener('transcription-final', handleTranscriptionFinal)
  window.addEventListener('transcription-error', handleTranscriptionError)
  window.addEventListener('transcription-stopped', handleTranscriptionStopped)
  window.addEventListener('transcription-complete', handleTranscriptionComplete)
  window.addEventListener('transcription-auto-stopped', handleTranscriptionAutoStopped)
  window.addEventListener('send-transcribed-message', handleSendTranscribedMessage)
}

function removeEventListeners() {
  window.removeEventListener('transcription-started', handleTranscriptionStarted)
  window.removeEventListener('transcription-interim', handleTranscriptionInterim)
  window.removeEventListener('transcription-final', handleTranscriptionFinal)
  window.removeEventListener('transcription-error', handleTranscriptionError)
  window.removeEventListener('transcription-stopped', handleTranscriptionStopped)
  window.removeEventListener('transcription-complete', handleTranscriptionComplete)
  window.removeEventListener('transcription-auto-stopped', handleTranscriptionAutoStopped)
  window.removeEventListener('send-transcribed-message', handleSendTranscribedMessage)
}

// Event handlers
function handleTranscriptionStarted(event: Event) {
  const customEvent = event as CustomEvent
  console.log('HomeScreen: Transcription started', customEvent.detail)
  transcriptionStatus.value = 'listening'
  isTranscribing.value = true
  currentTranscript.value = ''
  transcriptionError.value = null
  scrollToBottom()
}

function handleTranscriptionInterim(event: Event) {
  const customEvent = event as CustomEvent
  console.log('HomeScreen: Interim transcription', customEvent.detail)
  transcriptionStatus.value = 'processing'
  currentTranscript.value = customEvent.detail.text || ''
  
  // Add interim message to chat if not already present
  const lastMessage = appStore.chatMessages[appStore.chatMessages.length - 1]
  if (!lastMessage || !lastMessage.isInterim) {
    appStore.addMessage(currentTranscript.value, 'transcription', { 
      source: 'web-speech',
      isInterim: true,
      confidence: customEvent.detail.confidence 
    })
  } else {
    // Update existing interim message
    appStore.updateMessage(lastMessage.id, currentTranscript.value)
  }
  
  scrollToBottom()
}

function handleTranscriptionFinal(event: Event) {
  const customEvent = event as CustomEvent
  console.log('HomeScreen: Final transcription', customEvent.detail)
  transcriptionStatus.value = 'idle'
  
  // Replace interim message with final one or add new final message
  const lastMessage = appStore.chatMessages[appStore.chatMessages.length - 1]
  if (lastMessage && lastMessage.isInterim) {
    // Update interim message to final
    appStore.updateMessage(lastMessage.id, customEvent.detail.text)
  } else {
    // Add new final message
    appStore.addMessage(customEvent.detail.text, 'transcription', { 
      source: 'web-speech',
      isInterim: false,
      confidence: customEvent.detail.confidence 
    })
  }
  
  scrollToBottom()
}

function handleTranscriptionError(event: Event) {
  const customEvent = event as CustomEvent
  console.log('HomeScreen: Transcription error', customEvent.detail)
  transcriptionStatus.value = 'error'
  transcriptionError.value = customEvent.detail.error
  isTranscribing.value = false
}

function handleTranscriptionStopped(event: Event) {
  const customEvent = event as CustomEvent
  console.log('HomeScreen: Transcription stopped', customEvent.detail)
  transcriptionStatus.value = 'idle'
  isTranscribing.value = false
}

function handleTranscriptionComplete(event: Event) {
  const customEvent = event as CustomEvent
  console.log('HomeScreen: Transcription complete', customEvent.detail)
  transcriptionStatus.value = 'idle'
  isTranscribing.value = false
  currentTranscript.value = ''
}

function handleTranscriptionAutoStopped(event: Event) {
  const customEvent = event as CustomEvent
  console.log('HomeScreen: Transcription auto-stopped', customEvent.detail)
  transcriptionStatus.value = 'idle'
  isTranscribing.value = false
  
  // Add system message about auto-stop
  appStore.addMessage(`Transcription stopped automatically (${customEvent.detail.reason})`, 'assistant', { 
    source: 'typed' 
  })
  scrollToBottom()
}

function handleSendTranscribedMessage(event: Event) {
  const customEvent = event as CustomEvent
  console.log('HomeScreen: Auto-sending transcribed message', customEvent.detail)
  
  const messageText = customEvent.detail.text
  if (messageText && messageText.trim()) {
    appStore.addMessage(messageText.trim(), 'user', { source: 'whisper' })
    scrollToBottom()
  }
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
  <div class="home-screen">
    <!-- Chat Interface -->
    <div class="chat-interface">
      <!-- Chat Header -->
      <div class="chat-header">
        <div class="flex items-center gap-3">
          <div class="w-2 h-2 rounded-full bg-green-400 animate-pulse"></div>
          <h3 class="text-lg font-medium text-white/90">Assistant Chat</h3>
        </div>
      </div>
      
      <!-- Chat Messages -->
      <div 
        ref="messagesContainer" 
        class="chat-messages"
      >
        <div
          v-for="message in appStore.chatMessages"
          :key="message.id"
          :class="[
            'message',
            message.sender === 'user' 
              ? 'message-user' 
              : message.sender === 'transcription'
              ? `message-transcription ${message.isInterim ? 'message-interim' : 'message-final'}`
              : 'message-system'
          ]"
        >
          <!-- Editable transcription message -->
          <div 
            v-if="message.sender === 'transcription'"
            @click="startEditing(message)"
            class="message-transcription-content group"
          >
            <div v-if="editingMessageId === message.id" class="edit-form">
              <input
                v-model="editText"
                @keyup.enter="saveEdit"
                @keyup.escape="cancelEdit"
                @blur="saveEdit"
                class="edit-input"
                :placeholder="message.text"
                ref="editInput"
              />
              <div class="edit-hint">
                Press Enter to save, Esc to cancel
              </div>
            </div>
            <div v-else>
              <!-- Thought stream animation for interim messages -->
              <div 
                v-if="message.isInterim"
                class="interim-message"
              >
                <span class="text-orange-300 mr-2">💭</span>
                <span class="italic">{{ message.text }}</span>
                <span class="ml-2 text-orange-400 animate-pulse">...</span>
              </div>
              <!-- Final transcription -->
              <div v-else class="final-message">
                <span class="text-green-300 mr-2">🎤</span>
                <span>{{ message.text }}</span>
              </div>
              
              <!-- Edit hint on hover -->
              <div class="edit-hint-hover">
                Click to edit
              </div>
              
              <!-- Confidence indicator for final messages -->
              <div v-if="!message.isInterim && message.confidence" class="confidence-indicator">
                Confidence: {{ Math.round(message.confidence * 100) }}%
              </div>
            </div>
          </div>
          
          <!-- Regular messages -->
          <div v-else class="message-content">
            {{ message.text }}
          </div>
          
          <div class="message-timestamp">
            {{ message.timestamp.toLocaleTimeString() }}
          </div>
        </div>
      </div>
      
      <!-- Chat Input -->
      <div class="chat-input-section">
        <!-- Speech Transcription Info -->
        <div v-if="appStore.speechStatus.error" class="error-message">
          Speech Error: {{ appStore.speechStatus.error }}
        </div>

        <!-- Clear Chat Button -->
        <div v-if="appStore.isTranscriptionEnabled" class="clear-chat-section">
          <button 
            @click="appStore.clearChat"
            class="clear-chat-btn"
            title="Clear All Chat Messages"
          >
            Clear Chat
          </button>
        </div>

        <!-- Text Input with Mic -->
        <div class="input-group">
          <input
            v-model="newMessage"
            @keyup.enter="sendMessage"
            type="text" 
            placeholder="Type your message..."
            class="message-input"
          />
          
          <!-- Mic Button -->
          <button
            @click="toggleSpeechTranscription"
            class="mic-button"
            :class="{
              'mic-recording': appStore.speechStatus.isRecording,
              'mic-processing': appStore.speechStatus.isProcessing,
              'mic-ready': appStore.isTranscriptionEnabled && !appStore.speechStatus.isRecording
            }"
            :disabled="appStore.speechStatus.isProcessing"
            :title="appStore.speechStatus.isRecording ? 'Stop Recording' : 'Start Speech Recording'"
          >
            <MicrophoneIcon class="w-4 h-4" />
          </button>
          
          <button
            @click="sendMessage"
            class="send-button"
            :disabled="!newMessage.trim()"
          >
            <PaperAirplaneIcon class="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.home-screen {
  @apply flex flex-col h-full;
}

.chat-interface {
  @apply flex flex-col h-full max-w-4xl mx-auto w-full;
}

.chat-header {
  @apply flex items-center justify-between p-4 border-b border-white/10;
}

.chat-messages {
  @apply flex-1 p-4 space-y-3 overflow-y-auto;
  max-height: 400px;
}

.message {
  @apply p-3 rounded-lg max-w-[80%] break-words;
}

.message-user {
  @apply bg-blue-600 text-white ml-auto;
}

.message-transcription {
  @apply mr-auto cursor-pointer relative;
}

.message-interim {
  @apply bg-orange-500/20 text-orange-200 border border-orange-500/30;
}

.message-final {
  @apply bg-green-600/20 text-green-200 border border-green-600/30;
}

.message-system {
  @apply bg-gray-700 text-white mr-auto;
}

.message-transcription-content {
  @apply w-full;
}

.edit-form {
  @apply space-y-2;
}

.edit-input {
  @apply w-full bg-transparent border-b border-white/30 text-white outline-none;
}

.edit-hint {
  @apply text-xs text-white/60;
}

.edit-hint-hover {
  @apply absolute -top-6 right-0 text-xs text-white/60 opacity-0 group-hover:opacity-100 transition-opacity;
}

.interim-message {
  @apply flex items-center animate-pulse;
}

.final-message {
  @apply flex items-center;
}

.confidence-indicator {
  @apply text-xs text-white/60 mt-1;
}

.message-content {
  @apply w-full;
}

.message-timestamp {
  @apply text-xs opacity-60 mt-1;
}

.chat-input-section {
  @apply p-4 border-t border-white/10;
}

.error-message {
  @apply mb-3 text-sm text-red-300 bg-red-500/10 border border-red-500/20 rounded-lg p-2;
}

.clear-chat-section {
  @apply mb-3 flex justify-center;
}

.clear-chat-btn {
  @apply bg-white/10 hover:bg-white/20 text-white/70 hover:text-white rounded-lg px-3 py-1 text-xs transition-all duration-200;
}

.input-group {
  @apply flex gap-2;
}

.message-input {
  @apply flex-1 bg-white/5 border border-white/20 rounded-xl px-4 py-3 text-white placeholder-white/50 focus:bg-white/10 focus:border-white/30 transition-all;
  backdrop-filter: blur(20px);
}

.message-input:focus {
  outline: none;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.3);
}

.mic-button {
  @apply w-12 h-12 rounded-xl bg-white/5 hover:bg-white/10 border border-white/20 hover:border-white/30 text-white/70 hover:text-white flex items-center justify-center transition-all duration-200;
}

.mic-ready {
  @apply bg-green-500/20 hover:bg-green-500/30 border-green-400/40 text-green-300;
}

.mic-recording {
  @apply bg-red-500/30 border-red-400/50 text-red-300 animate-pulse;
}

.mic-processing {
  @apply bg-yellow-500/20 border-yellow-400/40 text-yellow-300;
}

.send-button {
  @apply w-12 h-12 rounded-xl bg-gradient-to-r from-blue-500 to-purple-600 hover:from-blue-600 hover:to-purple-700 text-white flex items-center justify-center transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed;
  box-shadow: 0 4px 15px rgba(59, 130, 246, 0.3);
}

.send-button:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(59, 130, 246, 0.4);
}

/* Custom scrollbar */
.chat-messages::-webkit-scrollbar {
  width: 4px;
}

.chat-messages::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 2px;
}

.chat-messages::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
}

.chat-messages::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.3);
}
</style> 