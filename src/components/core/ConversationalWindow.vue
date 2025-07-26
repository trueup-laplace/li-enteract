<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, computed } from 'vue'
import { 
  MicrophoneIcon, 
  SpeakerWaveIcon, 
  XMarkIcon,
  PaperAirplaneIcon,
  StopIcon,
  ChatBubbleLeftRightIcon,
  CheckIcon
} from '@heroicons/vue/24/outline'
import { useSpeechTranscription } from '../../composables/useSpeechTranscription'
import { useConversationStore } from '../../stores/conversation'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

interface Props {
  showConversationalWindow: boolean
}

interface Emits {
  (e: 'close'): void
  (e: 'update:showConversationalWindow', value: boolean): void
}

defineProps<Props>()
const emit = defineEmits<Emits>()

// Store
const conversationStore = useConversationStore()

// State
const scrollContainer = ref<HTMLElement>()
const audioLoopbackDeviceId = ref<string | null>(null)
const selectedMessages = ref<Set<string>>(new Set())
const showExportControls = ref(false)

// Computed
const messages = computed(() => conversationStore.currentMessages)
const isAudioLoopbackActive = computed(() => conversationStore.isAudioLoopbackActive)
const hasSelectedMessages = computed(() => selectedMessages.value.size > 0)

// Speech transcription
const {
  initialize: initializeSpeech,
  startRecording,
  stopRecording,
  isRecording,
  isInitialized: isSpeechInitialized,
  currentTranscript,
  error: speechError,
  setAutoSendToChat,
  setContinuousMode
} = useSpeechTranscription()

// Initialize when component mounts
onMounted(async () => {
  try {
    // Create or resume a conversation session
    if (!conversationStore.currentSession) {
      conversationStore.createSession()
    }
    
    // Initialize speech transcription
    await initializeSpeech()
    
    // IMPORTANT: Disable auto-send to chat for conversational window
    setAutoSendToChat(false)
    // Enable continuous mode to keep mic open during conversations
    setContinuousMode(true)
    console.log('🎤 Speech transcription initialized for conversational window (auto-send disabled, continuous mode enabled)')
    
    // Just load settings, don't start capture
    await loadAudioLoopbackSettings()
    
    // Set up event listeners (but they won't receive data until recording starts)
    setupEventListeners()
    
  } catch (error) {
    console.error('Failed to initialize conversational window:', error)
  }
})

// Audio loopback settings
const loadAudioLoopbackSettings = async () => {
  try {
    const settings = await invoke<{
      selectedLoopbackDevice: string | null
      loopbackEnabled: boolean
    } | null>('load_audio_settings')
    
    console.log('🔊 Audio settings loaded:', settings)
    
    if (settings?.selectedLoopbackDevice) {
      audioLoopbackDeviceId.value = settings.selectedLoopbackDevice
      // Don't start capture here - wait for mic button click
    }
  } catch (error) {
    console.error('Failed to load audio loopback settings:', error)
  }
}

// Start audio loopback capture
const startAudioLoopbackCapture = async () => {
  if (!audioLoopbackDeviceId.value) return
  
  try {
    await invoke('start_audio_loopback_capture', {
      deviceId: audioLoopbackDeviceId.value
    })
    
    conversationStore.setAudioLoopbackState(true)
    console.log('🔊 Audio loopback capture started')
  } catch (error) {
    console.error('Failed to start audio loopback capture:', error)
  }
}

// Stop audio loopback capture
const stopAudioLoopbackCapture = async () => {
  try {
    await invoke('stop_audio_loopback_capture')
    conversationStore.setAudioLoopbackState(false)
    console.log('⏹️ Audio loopback capture stopped')
  } catch (error) {
    console.error('Failed to stop audio loopback capture:', error)
  }
}

// Event listeners
const setupEventListeners = async () => {
  // IMPORTANT: Create isolated event handlers for conversational UI only
  // These should NOT interfere with main chat transcription
  
  // Listen for transcription events but handle them separately
  window.addEventListener('transcription-final', handleConversationalUserSpeech)
  
  // Listen for audio loopback events from Rust backend using Tauri's event system
  console.log('🎧 Setting up Tauri audio-chunk event listener')
  await listen('audio-chunk', (event) => {
    console.log('🎧 Tauri audio-chunk event received:', event.payload)
    handleSystemAudio({ detail: event.payload } as CustomEvent)
  })
  
  // Listen for loopback transcription events
  console.log('🎙️ Setting up loopback transcription listener')
  await listen('loopback-transcription', (event) => {
    console.log('🎙️ Loopback transcription received:', event.payload)
    handleLoopbackTranscription(event.payload as any)
  })
  
  console.log('✅ Conversational audio event listeners set up')
}

// Handle user speech from microphone - ONLY for conversational window
const handleConversationalUserSpeech = (event: Event) => {
  const customEvent = event as CustomEvent
  
  // Only process if we're recording in conversational mode
  if (!isRecording.value) return
  
  const { text, confidence, timestamp } = customEvent.detail
  
  if (text && text.trim()) {
    conversationStore.addMessage({
      type: 'user',
      source: 'microphone',
      content: text.trim(),
      confidence,
      timestamp: timestamp || Date.now()
    })
  }
}

// Handle system audio from loopback (just for monitoring audio levels)
const handleSystemAudio = (event: CustomEvent) => {
  console.log('🎧 Received audio-chunk event - level:', event.detail.level, 'dB')
  // This is now just for monitoring - transcription happens in Rust backend
}

// Handle loopback transcription results
const handleLoopbackTranscription = (payload: any) => {
  console.log('🎙️ Loopback transcription:', payload)
  const { text, confidence, timestamp, audioLevel } = payload
  
  if (text && text.trim()) {
    conversationStore.addMessage({
      type: 'system',
      source: 'loopback',
      content: text.trim(),
      confidence,
      timestamp: timestamp || Date.now()
    })
    
    console.log(`🎙️ System Audio (${audioLevel?.toFixed(1)}dB): ${text}`)
    scrollToBottom()
  }
}

// Scroll to bottom of conversation
const scrollToBottom = async () => {
  await nextTick()
  if (scrollContainer.value) {
    scrollContainer.value.scrollTop = scrollContainer.value.scrollHeight
  }
}

// Handle microphone button
const handleMicrophoneToggle = async () => {
  try {
    if (isRecording.value) {
      // Stop both microphone and audio loopback
      await stopRecording()
      conversationStore.setRecordingState(false)
      // Always try to stop audio loopback when stopping recording
      await stopAudioLoopbackCapture()
    } else {
      // Start both microphone and audio loopback
      conversationStore.setRecordingState(true)
      await startRecording()
      if (audioLoopbackDeviceId.value && !isAudioLoopbackActive.value) {
        await startAudioLoopbackCapture()
      }
    }
  } catch (error) {
    console.error('Microphone toggle error:', error)
    conversationStore.setRecordingState(false)
  }
}

// Close window
const closeWindow = () => {
  emit('close')
  emit('update:showConversationalWindow', false)
}

// Format timestamp
const formatTime = (timestamp: number) => {
  return new Date(timestamp).toLocaleTimeString('en-US', {
    hour12: true,
    hour: 'numeric',
    minute: '2-digit'
  })
}

// Message selection
const toggleMessageSelection = (messageId: string) => {
  if (selectedMessages.value.has(messageId)) {
    selectedMessages.value.delete(messageId)
  } else {
    selectedMessages.value.add(messageId)
  }
}

const selectAllMessages = () => {
  messages.value.forEach(message => {
    selectedMessages.value.add(message.id)
  })
}

const clearSelection = () => {
  selectedMessages.value.clear()
}

// Export to main chat
const exportSelectedToChat = () => {
  if (selectedMessages.value.size === 0) return
  
  const messageIds = Array.from(selectedMessages.value)
  conversationStore.exportMessagesToMainChat(messageIds)
  
  // Clear selection after export
  clearSelection()
  showExportControls.value = false
  
  console.log(`📤 Exported ${messageIds.length} messages to main chat`)
}

// Toggle export mode
const toggleExportMode = () => {
  showExportControls.value = !showExportControls.value
  if (!showExportControls.value) {
    clearSelection()
  }
}

// Cleanup
onUnmounted(async () => {
  window.removeEventListener('transcription-final', handleConversationalUserSpeech)
  
  // Stop both recording and loopback when unmounting
  if (isRecording.value) {
    await stopRecording()
    conversationStore.setRecordingState(false)
  }
  
  if (isAudioLoopbackActive.value) {
    await stopAudioLoopbackCapture()
  }
  
  // Re-enable auto-send to chat for main interface and disable continuous mode
  setAutoSendToChat(true)
  setContinuousMode(false)
  console.log('🎤 Auto-send to chat re-enabled for main interface, continuous mode disabled')
  
  // End the current session when window closes
  if (conversationStore.currentSession) {
    conversationStore.endSession()
  }
})
</script>

<template>
  <Transition name="conversational-window">
    <div v-if="showConversationalWindow" class="conversational-window">
        <!-- Window Header -->
        <div class="window-header">
          <div class="header-title">
            <div class="flex items-center gap-2">
              <MicrophoneIcon class="w-4 h-4 text-white/80" />
              <span class="text-sm font-medium text-white/90">Conversation</span>
            </div>
            <div class="header-controls">
              <div class="status-indicators">
                <div 
                  v-if="isAudioLoopbackActive" 
                  class="status-indicator text-green-400"
                  title="System audio capture active"
                >
                  <SpeakerWaveIcon class="w-3 h-3" />
                </div>
                <div 
                  v-if="isRecording" 
                  class="status-indicator text-red-400 animate-pulse"
                  title="Recording microphone"
                >
                  <MicrophoneIcon class="w-3 h-3" />
                </div>
              </div>
              <button 
                v-if="messages.length > 0"
                @click="toggleExportMode" 
                class="export-btn"
                :class="{ 'active': showExportControls }"
                title="Export messages to main chat"
              >
                <ChatBubbleLeftRightIcon class="w-4 h-4" />
              </button>
            </div>
          </div>
          <button @click="closeWindow" class="close-btn">
            <XMarkIcon class="w-4 h-4 text-white/70 hover:text-white transition-colors" />
          </button>
        </div>
        
        <!-- Export Controls -->
        <div v-if="showExportControls" class="export-controls">
          <div class="export-info">
            <span class="text-xs text-white/60">
              {{ selectedMessages.size }} message{{ selectedMessages.size !== 1 ? 's' : '' }} selected
            </span>
          </div>
          <div class="export-actions">
            <button @click="selectAllMessages" class="export-action-btn">
              <CheckIcon class="w-3 h-3" />
              <span class="text-xs">All</span>
            </button>
            <button @click="clearSelection" class="export-action-btn">
              <span class="text-xs">Clear</span>
            </button>
            <button 
              @click="exportSelectedToChat" 
              :disabled="!hasSelectedMessages"
              class="export-action-btn primary"
              :class="{ 'disabled': !hasSelectedMessages }"
            >
              <PaperAirplaneIcon class="w-3 h-3" />
              <span class="text-xs">Send to Chat</span>
            </button>
          </div>
        </div>
        
        <!-- Conversation Area -->
        <div ref="scrollContainer" class="conversation-area">
          <div v-if="messages.length === 0" class="empty-state">
            <div class="empty-icon">
              <MicrophoneIcon class="w-8 h-8 text-white/40" />
            </div>
            <p class="text-white/60 text-sm">Start a conversation</p>
            <p class="text-white/40 text-xs mt-1">
              Your voice will appear on the right, system audio on the left
            </p>
          </div>
          
          <div v-else class="messages-container">
            <div 
              v-for="message in messages" 
              :key="message.id"
              class="message-wrapper"
              :class="{
                'user-message': message.type === 'user',
                'system-message': message.type === 'system',
                'selectable': showExportControls,
                'selected': selectedMessages.has(message.id)
              }"
              @click="showExportControls ? toggleMessageSelection(message.id) : null"
            >
              <div class="message-bubble" :class="{
                'user-bubble': message.type === 'user',
                'system-bubble': message.type === 'system'
              }">
                <div class="message-header">
                  <div class="message-source">
                    <MicrophoneIcon 
                      v-if="message.source === 'microphone'" 
                      class="w-3 h-3"
                    />
                    <SpeakerWaveIcon 
                      v-else-if="message.source === 'loopback'" 
                      class="w-3 h-3"
                    />
                    <span class="source-label">
                      {{ message.source === 'microphone' ? 'You' : 'System' }}
                    </span>
                    <div v-if="showExportControls" class="selection-indicator">
                      <div class="selection-checkbox" :class="{ 'checked': selectedMessages.has(message.id) }">
                        <CheckIcon v-if="selectedMessages.has(message.id)" class="w-2 h-2" />
                      </div>
                    </div>
                  </div>
                  <span class="message-time">{{ formatTime(message.timestamp) }}</span>
                </div>
                <div class="message-content">{{ message.content }}</div>
                <div v-if="message.confidence" class="message-confidence">
                  Confidence: {{ Math.round(message.confidence * 100) }}%
                </div>
              </div>
            </div>
          </div>
        </div>
        
        <!-- Controls -->
        <div class="controls-area">
          <div class="control-buttons">
            <button
              @click="handleMicrophoneToggle"
              :disabled="!isSpeechInitialized"
              class="mic-button"
              :class="{
                'recording': isRecording,
                'disabled': !isSpeechInitialized
              }"
              :title="isRecording ? 'Stop recording' : 'Start recording'"
            >
              <StopIcon v-if="isRecording" class="w-5 h-5" />
              <MicrophoneIcon v-else class="w-5 h-5" />
            </button>
            
            <div class="recording-status">
              <div v-if="isRecording" class="recording-indicator">
                <div class="pulse-dot"></div>
                <span class="text-sm text-white/80">
                  Recording{{ isAudioLoopbackActive ? ' (Mic + System Audio)' : ' (Mic Only)' }}
                </span>
              </div>
              <div v-else-if="currentTranscript" class="transcript-preview">
                <span class="text-xs text-white/60">{{ currentTranscript }}</span>
              </div>
              <div v-else-if="speechError" class="error-indicator">
                <span class="text-xs text-red-400">{{ speechError }}</span>
              </div>
              <div v-else class="idle-indicator">
                <span class="text-xs text-white/40">
                  Click mic to start{{ audioLoopbackDeviceId ? ' (Mic + System Audio)' : ' (Mic Only)' }}
                </span>
              </div>
            </div>
          </div>
          
          <div class="audio-status">
            <div class="status-item">
              <span class="text-xs text-white/60">Mic:</span>
              <span class="text-xs" :class="{
                'text-green-400': isSpeechInitialized,
                'text-red-400': !isSpeechInitialized
              }">
                {{ isSpeechInitialized ? 'Ready' : 'Not Ready' }}
              </span>
            </div>
            <div class="status-item">
              <span class="text-xs text-white/60">System Audio:</span>
              <span class="text-xs" :class="{
                'text-green-400': isAudioLoopbackActive,
                'text-blue-400': audioLoopbackDeviceId && !isAudioLoopbackActive,
                'text-yellow-400': !audioLoopbackDeviceId
              }">
                {{ 
                  isAudioLoopbackActive ? 'Active' : 
                  audioLoopbackDeviceId ? 'Device Selected' : 
                  'Configure in Settings' 
                }}
              </span>
            </div>
          </div>
        </div>
    </div>
  </Transition>
</template>

<style scoped>
.conversational-window {
  @apply backdrop-blur-xl border border-white/15 rounded-2xl overflow-hidden;
  background: linear-gradient(to bottom, 
    rgba(10, 10, 12, 0.9) 0%, 
    rgba(5, 5, 7, 0.95) 100%
  );
  width: 600px;
  height: 700px;
  max-width: 95vw;
  max-height: calc(100vh - 100px); /* Account for control panel */
  display: flex;
  flex-direction: column;
  position: relative;
  
  /* Enhanced glass effect similar to transparency controls */
  backdrop-filter: blur(80px) saturate(180%);
  box-shadow: 
    0 25px 80px rgba(0, 0, 0, 0.6),
    0 10px 30px rgba(0, 0, 0, 0.4),
    inset 0 1px 0 rgba(255, 255, 255, 0.15);
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

.status-indicators {
  @apply flex items-center gap-2;
}

.export-btn {
  @apply rounded-full p-1 hover:bg-white/10 transition-all duration-200 text-white/60 hover:text-white/90;
}

.export-btn.active {
  @apply bg-blue-500/20 text-blue-400 hover:bg-blue-500/30;
}

.status-indicator {
  @apply flex items-center justify-center w-6 h-6 rounded-full bg-white/10;
}

.close-btn {
  @apply rounded-full p-1 hover:bg-white/10 transition-colors;
}

.export-controls {
  @apply flex items-center justify-between px-4 py-2 border-b border-white/10 bg-white/5;
  flex-shrink: 0;
}

.export-info {
  @apply flex items-center;
}

.export-actions {
  @apply flex items-center gap-2;
}

.export-action-btn {
  @apply flex items-center gap-1 px-2 py-1 rounded-lg text-white/70 hover:text-white hover:bg-white/10 transition-all duration-200;
}

.export-action-btn.primary {
  @apply bg-blue-500/80 text-white hover:bg-blue-500 disabled:opacity-50 disabled:cursor-not-allowed;
}

.export-action-btn.disabled {
  @apply opacity-50 cursor-not-allowed hover:bg-transparent hover:text-white/70;
}

.conversation-area {
  @apply flex-1 overflow-y-auto;
  min-height: 0;
}

.empty-state {
  @apply flex flex-col items-center justify-center h-full text-center p-8;
}

.empty-icon {
  @apply mb-4 p-4 rounded-full bg-white/5;
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

.message-wrapper.system-message {
  @apply justify-start;
}

.message-wrapper.selectable {
  @apply cursor-pointer hover:bg-white/5 rounded-lg p-1 transition-all duration-200;
}

.message-wrapper.selected {
  @apply bg-blue-500/10 border border-blue-500/30 rounded-lg;
}

.message-bubble {
  @apply rounded-2xl p-3 max-w-xs;
  word-wrap: break-word;
}

.user-bubble {
  @apply bg-blue-500/80 text-white;
  border-bottom-right-radius: 6px;
}

.system-bubble {
  @apply text-white/90 border border-white/10;
  background: rgba(255, 255, 255, 0.05);
  border-bottom-left-radius: 6px;
}

.message-header {
  @apply flex items-center justify-between mb-1;
}

.message-source {
  @apply flex items-center gap-1 text-xs opacity-75;
}

.selection-indicator {
  @apply ml-auto;
}

.selection-checkbox {
  @apply w-4 h-4 rounded border border-white/30 flex items-center justify-center bg-transparent transition-all duration-200;
}

.selection-checkbox.checked {
  @apply bg-blue-500 border-blue-500;
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

.controls-area {
  @apply border-t border-white/10 p-4;
  flex-shrink: 0;
}

.control-buttons {
  @apply flex items-center gap-4 mb-3;
}

.mic-button {
  @apply w-12 h-12 rounded-full flex items-center justify-center transition-all duration-200;
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.8), rgba(37, 99, 235, 0.8));
  border: 2px solid rgba(59, 130, 246, 0.4);
}

.mic-button:hover:not(:disabled) {
  @apply scale-105;
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.9), rgba(37, 99, 235, 0.9));
  border-color: rgba(59, 130, 246, 0.6);
}

.mic-button.recording {
  @apply animate-pulse;
  background: linear-gradient(135deg, rgba(239, 68, 68, 0.8), rgba(220, 38, 38, 0.8));
  border-color: rgba(239, 68, 68, 0.6);
}

.mic-button.disabled {
  @apply opacity-50 cursor-not-allowed;
  background: rgba(75, 85, 99, 0.8);
  border-color: rgba(75, 85, 99, 0.4);
}

.recording-status {
  @apply flex-1;
}

.recording-indicator {
  @apply flex items-center gap-2;
}

.pulse-dot {
  @apply w-2 h-2 bg-red-400 rounded-full animate-pulse;
}

.transcript-preview {
  @apply truncate;
}

.error-indicator {
  @apply truncate;
}

.idle-indicator {
  /* Empty for now */
}

.audio-status {
  @apply flex items-center justify-between text-xs;
}

.status-item {
  @apply flex items-center gap-1;
}

/* Transitions */
.conversational-window-enter-active,
.conversational-window-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.conversational-window-enter-from {
  opacity: 0;
  transform: translateY(-10px) scale(0.98);
}

.conversational-window-leave-to {
  opacity: 0;
  transform: translateY(-10px) scale(0.98);
}

/* Scrollbar */
.conversation-area::-webkit-scrollbar {
  width: 4px;
}

.conversation-area::-webkit-scrollbar-track {
  background: transparent;
}

.conversation-area::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
}
</style>