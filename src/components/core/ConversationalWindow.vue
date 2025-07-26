<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { 
  MicrophoneIcon, 
  SpeakerWaveIcon, 
  XMarkIcon,
  PaperAirplaneIcon,
  StopIcon
} from '@heroicons/vue/24/outline'
import { useSpeechTranscription } from '../../composables/useSpeechTranscription'
import { invoke } from '@tauri-apps/api/core'

interface Props {
  showConversationalWindow: boolean
}

interface Emits {
  (e: 'close'): void
  (e: 'update:showConversationalWindow', value: boolean): void
}

interface ConversationMessage {
  id: string
  type: 'user' | 'system'
  source: 'microphone' | 'loopback'
  content: string
  timestamp: number
  confidence?: number
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// State
const messages = ref<ConversationMessage[]>([])
const scrollContainer = ref<HTMLElement>()
const isAudioLoopbackActive = ref(false)
const audioLoopbackDeviceId = ref<string | null>(null)

// Speech transcription
const {
  initialize: initializeSpeech,
  startRecording,
  stopRecording,
  isRecording,
  isInitialized: isSpeechInitialized,
  currentTranscript,
  finalText,
  error: speechError
} = useSpeechTranscription()

// Initialize when component mounts
onMounted(async () => {
  try {
    // Initialize speech transcription
    await initializeSpeech()
    console.log('🎤 Speech transcription initialized for conversational window')
    
    // Load audio loopback settings
    await loadAudioLoopbackSettings()
    
    // Set up event listeners
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
      
      // Always try to start if a device is selected (ignore loopbackEnabled for now)
      // The user explicitly selected a device, so they want to use it
      await startAudioLoopbackCapture()
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
    
    isAudioLoopbackActive.value = true
    console.log('🔊 Audio loopback capture started')
  } catch (error) {
    console.error('Failed to start audio loopback capture:', error)
  }
}

// Stop audio loopback capture
const stopAudioLoopbackCapture = async () => {
  try {
    await invoke('stop_audio_loopback_capture')
    isAudioLoopbackActive.value = false
    console.log('⏹️ Audio loopback capture stopped')
  } catch (error) {
    console.error('Failed to stop audio loopback capture:', error)
  }
}

// Event listeners
const setupEventListeners = () => {
  // Listen for transcription events
  window.addEventListener('transcription-final', handleUserSpeech)
  window.addEventListener('send-transcribed-message', handleTranscribedMessage)
  
  // Listen for audio loopback events from Rust backend
  window.addEventListener('audio-chunk', handleSystemAudio)
}

// Handle user speech from microphone
const handleUserSpeech = (event: CustomEvent) => {
  const { text, confidence, timestamp } = event.detail
  
  if (text && text.trim()) {
    addMessage({
      type: 'user',
      source: 'microphone',
      content: text.trim(),
      confidence,
      timestamp: timestamp || Date.now()
    })
  }
}

// Handle transcribed message
const handleTranscribedMessage = (event: CustomEvent) => {
  const { text, timestamp } = event.detail
  
  if (text && text.trim()) {
    addMessage({
      type: 'user',
      source: 'microphone',
      content: text.trim(),
      timestamp: timestamp || Date.now()
    })
  }
}

// Handle system audio from loopback
const handleSystemAudio = async (event: CustomEvent) => {
  const { audio_data, device_id, timestamp } = event.detail
  
  if (!audio_data) return
  
  try {
    // Process audio through transcription
    const transcription = await invoke<string>('process_audio_for_transcription', {
      audioData: audio_data,
      sampleRate: 16000
    })
    
    if (transcription && transcription.trim() && transcription !== 'Transcribed text would go here') {
      addMessage({
        type: 'system',
        source: 'loopback',
        content: transcription.trim(),
        timestamp: timestamp || Date.now()
      })
    }
  } catch (error) {
    console.error('Failed to process system audio:', error)
  }
}

// Add message to conversation
const addMessage = (messageData: Omit<ConversationMessage, 'id'>) => {
  const message: ConversationMessage = {
    id: `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
    ...messageData
  }
  
  messages.value.push(message)
  scrollToBottom()
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
      await stopRecording()
    } else {
      await startRecording()
    }
  } catch (error) {
    console.error('Microphone toggle error:', error)
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

// Cleanup
onUnmounted(async () => {
  window.removeEventListener('transcription-final', handleUserSpeech)
  window.removeEventListener('send-transcribed-message', handleTranscribedMessage)
  window.removeEventListener('audio-chunk', handleSystemAudio)
  
  if (isRecording.value) {
    await stopRecording()
  }
  
  if (isAudioLoopbackActive.value) {
    await stopAudioLoopbackCapture()
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
          </div>
          <button @click="closeWindow" class="close-btn">
            <XMarkIcon class="w-4 h-4 text-white/70 hover:text-white transition-colors" />
          </button>
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
                'system-message': message.type === 'system'
              }"
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
                <span class="text-sm text-white/80">Recording...</span>
              </div>
              <div v-else-if="currentTranscript" class="transcript-preview">
                <span class="text-xs text-white/60">{{ currentTranscript }}</span>
              </div>
              <div v-else-if="speechError" class="error-indicator">
                <span class="text-xs text-red-400">{{ speechError }}</span>
              </div>
              <div v-else class="idle-indicator">
                <span class="text-xs text-white/40">Ready to record</span>
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
    rgba(0, 0, 0, 0.8) 0%, 
    rgba(0, 0, 0, 0.9) 100%
  );
  width: 600px;
  height: 700px;
  max-width: 95vw;
  max-height: 95vh;
  display: flex;
  flex-direction: column;
  
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

.status-indicators {
  @apply flex items-center gap-2;
}

.status-indicator {
  @apply flex items-center justify-center w-6 h-6 rounded-full bg-white/10;
}

.close-btn {
  @apply rounded-full p-1 hover:bg-white/10 transition-colors;
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

.message-bubble {
  @apply rounded-2xl p-3 max-w-xs;
  word-wrap: break-word;
}

.user-bubble {
  @apply bg-blue-500/80 text-white;
  border-bottom-right-radius: 6px;
}

.system-bubble {
  @apply bg-white/10 text-white/90 border border-white/20;
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
  transform: translateY(-10px);
}

.conversational-window-leave-to {
  opacity: 0;
  transform: translateY(-10px);
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