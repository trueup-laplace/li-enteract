<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue'
import { 
  MicrophoneIcon, 
  SpeakerWaveIcon, 
  XMarkIcon,
  QueueListIcon,
  PencilIcon,
  SparklesIcon,
  RocketLaunchIcon
} from '@heroicons/vue/24/outline'
import { useSpeechTranscription } from '../../composables/useSpeechTranscription'
import { useConversationStore } from '../../stores/conversation'
import { useWindowResizing } from '../../composables/useWindowResizing'
import { useLiveAI } from '../../composables/useLiveAI'
import { invoke } from '@tauri-apps/api/core'

// Components
import MessageList from '../conversational/MessageList.vue'
import ConversationSidebar from '../conversational/ConversationSidebar.vue'
import AIAssistant from '../conversational/AIAssistant.vue'
import LiveAI from '../conversational/LiveAI.vue'
import ExportControls from '../conversational/ExportControls.vue'

// Composables
import { useLoopbackTranscription } from './composables/useLoopbackTranscription'
import { useConversationManagement } from './composables/useConversationManagement'
import { useAIAssistant } from './composables/useAIAssistant'

interface Props {
  showConversationalWindow: boolean
}

interface Emits {
  (e: 'close'): void
  (e: 'update:showConversationalWindow', value: boolean): void
}

defineProps<Props>()
const emit = defineEmits<Emits>()

// Store and composables
const conversationStore = useConversationStore()
const { resizeWindow } = useWindowResizing()

// Components refs
const messageListRef = ref<InstanceType<typeof MessageList>>()

// State
const audioLoopbackDeviceId = ref<string | null>(null)
const selectedMessages = ref<Set<string>>(new Set())
const showExportControls = ref(false)

// Sidebar and panel states
const showConversationSidebar = ref(false)
const showAIAssistant = ref(false)
const showLiveAI = ref(false)

// Speech transcription
const {
  initialize: initializeSpeech,
  startRecording,
  stopRecording,
  isRecording,
  isInitialized: isSpeechInitialized,
  error: speechError,
  setAutoSendToChat,
  setContinuousMode
} = useSpeechTranscription()

// Loopback transcription composable
const {
  isLoopbackTyping,
  loopbackPreviewMessage,
  currentPreviewMessageId,
  isMicrophoneTyping,
  microphonePreviewMessage,
  currentMicPreviewMessageId,
  setupLoopbackListeners,
  cleanupLoopback
} = useLoopbackTranscription()

// Conversation management
const {
  allConversations,
  isLoadingConversations,
  loadConversations,
  createNewConversation,
  resumeConversation,
  deleteConversation
} = useConversationManagement()

// AI Assistant
const {
  aiResponse,
  aiIsProcessing,
  aiError,
  queryAI
} = useAIAssistant()

// Live AI
const {
  isActive: isLiveAIActive,
  response: liveAIResponse,
  isProcessing: liveAIIsProcessing,
  error: liveAIError,
  startLiveAI,
  stopLiveAI
} = useLiveAI()

// Computed
const messages = computed(() => {
  try {
    const baseMessages = conversationStore.currentMessages || []
    const messagesWithPreviews = [...baseMessages]
    
    // Add loopback typing preview if active
    if (isLoopbackTyping.value && loopbackPreviewMessage.value.trim()) {
      messagesWithPreviews.push({
        id: currentPreviewMessageId.value || 'loopback-preview',
        type: 'system',
        source: 'loopback',
        content: loopbackPreviewMessage.value,
        confidence: 0.5,
        timestamp: Date.now(),
        isPreview: true,
        isTyping: true
      })
    }
    
    // Add microphone typing preview if active
    if (isMicrophoneTyping.value && microphonePreviewMessage.value.trim()) {
      messagesWithPreviews.push({
        id: currentMicPreviewMessageId.value || 'mic-preview',
        type: 'user',
        source: 'microphone',
        content: microphonePreviewMessage.value,
        confidence: 0.5,
        timestamp: Date.now(),
        isPreview: true,
        isTyping: true
      })
    }
    
    return messagesWithPreviews
  } catch (error) {
    console.error('❌ Error computing messages:', error)
    return []
  }
})

const isAudioLoopbackActive = computed(() => conversationStore.isAudioLoopbackActive)
const hasSelectedMessages = computed(() => selectedMessages.value.size > 0)

// Watch for sidebar changes to resize window
watch(showConversationSidebar, async (newValue) => {
  try {
    await resizeWindow(false, false, false, true, newValue)
  } catch (error) {
    console.error('❌ Failed to resize window for sidebar:', error)
  }
})

// Initialize when component mounts
onMounted(async () => {
  try {
    // Initialize speech transcription
    await initializeSpeech()
    setAutoSendToChat(false)
    setContinuousMode(true)
    
    // Load audio loopback settings
    await loadAudioLoopbackSettings()
    
    // Set up event listeners
    setupLoopbackListeners(isRecording)
    
  } catch (error) {
    console.error('Failed to initialize conversational window:', error)
  }
})

// Cleanup on unmount
onUnmounted(() => {
  cleanupLoopback()
})

// Audio loopback settings
const loadAudioLoopbackSettings = async () => {
  try {
    const settings = await invoke<{
      selectedLoopbackDevice: string | null
      loopbackEnabled: boolean
    } | null>('load_audio_settings')
    
    if (settings?.selectedLoopbackDevice) {
      audioLoopbackDeviceId.value = settings.selectedLoopbackDevice
    }
  } catch (error) {
    console.error('Failed to load audio loopback settings:', error)
  }
}

// Start/stop audio loopback capture
const startAudioLoopbackCapture = async () => {
  if (!audioLoopbackDeviceId.value) return
  
  try {
    await invoke('start_audio_loopback_capture', {
      deviceId: audioLoopbackDeviceId.value
    })
    conversationStore.setAudioLoopbackState(true)
  } catch (error) {
    console.error('Failed to start audio loopback capture:', error)
  }
}

const stopAudioLoopbackCapture = async () => {
  try {
    await invoke('stop_audio_loopback_capture')
    conversationStore.setAudioLoopbackState(false)
  } catch (error) {
    console.error('Failed to stop audio loopback capture:', error)
  }
}

// Microphone toggle
const toggleMicrophone = async () => {
  if (isRecording.value) {
    await stopRecording()
    await stopAudioLoopbackCapture()
    
    if (conversationStore.currentSession) {
      // End the current session
      conversationStore.endSession()
    }
  } else {
    if (!conversationStore.currentSession) {
      // Create a new session if none exists
      conversationStore.createSession()
    }
    
    await startRecording()
    if (audioLoopbackDeviceId.value) {
      await startAudioLoopbackCapture()
    }
  }
}

// Export controls
const toggleExportControls = () => {
  showExportControls.value = !showExportControls.value
  if (!showExportControls.value) {
    selectedMessages.value.clear()
  }
}

const toggleMessageSelection = (messageId: string) => {
  if (selectedMessages.value.has(messageId)) {
    selectedMessages.value.delete(messageId)
  } else {
    selectedMessages.value.add(messageId)
  }
}

const selectAllMessages = () => {
  const allMessages = conversationStore.currentMessages || []
  allMessages.forEach(msg => selectedMessages.value.add(msg.id))
}

const deselectAllMessages = () => {
  selectedMessages.value.clear()
}

const exportSelectedMessages = async () => {
  const selectedMessagesList = messages.value.filter(msg => 
    selectedMessages.value.has(msg.id) && !msg.isPreview
  )
  
  if (selectedMessagesList.length === 0) return
  
  try {
    // For now, just log the messages that would be exported
    console.log('Exporting messages:', selectedMessagesList)
    showExportControls.value = false
    selectedMessages.value.clear()
  } catch (error) {
    console.error('Failed to export messages:', error)
  }
}

// Sidebar actions
const toggleConversationSidebar = () => {
  showConversationSidebar.value = !showConversationSidebar.value
  if (showConversationSidebar.value) {
    loadConversations()
  }
}

const handleNewConversation = async () => {
  await createNewConversation()
  showConversationSidebar.value = false
}

const handleResumeConversation = async (id: string) => {
  await resumeConversation(id)
  showConversationSidebar.value = false
}

const handleDeleteConversation = async (id: string) => {
  await deleteConversation(id)
}

// AI Assistant actions
const toggleAIAssistant = () => {
  showAIAssistant.value = !showAIAssistant.value
}

const handleAIQuery = async (query: string) => {
  const currentMessages = conversationStore.currentMessages || []
  await queryAI(query, currentMessages)
}

// Live AI actions
const toggleLiveAI = () => {
  showLiveAI.value = !showLiveAI.value
}

const toggleLiveAIActive = async () => {
  if (isLiveAIActive.value) {
    await stopLiveAI()
  } else {
    const currentMessages = conversationStore.currentMessages || []
    await startLiveAI(currentMessages)
  }
}

// Close window
const closeWindow = () => {
  if (isRecording.value) {
    stopRecording()
    stopAudioLoopbackCapture()
  }
  emit('close')
  emit('update:showConversationalWindow', false)
}

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
              <button 
                @click="toggleExportControls" 
                class="export-btn"
                :class="{ 'active': showExportControls }"
                title="Export conversation"
              >
                <PencilIcon class="w-3 h-3" />
              </button>
              <button 
                @click="toggleConversationSidebar" 
                class="export-btn"
                :class="{ 'active': showConversationSidebar }"
                title="Show conversations"
              >
                <QueueListIcon class="w-3 h-3" />
              </button>
              <button 
                @click="toggleAIAssistant" 
                class="ai-assistant-btn"
                :class="{ 'active': showAIAssistant }"
                title="AI Assistant"
              >
                <SparklesIcon class="w-3 h-3" />
              </button>
              <button 
                @click="toggleLiveAI" 
                class="live-ai-btn"
                :class="{ 'active': showLiveAI }"
                title="Live AI Response"
              >
                <RocketLaunchIcon class="w-3 h-3" />
              </button>
              <div v-if="isRecording" class="status-indicator" title="Recording">
                <div class="recording-dot"></div>
              </div>
              <div v-if="isAudioLoopbackActive" class="status-indicator" title="Audio loopback active">
                <SpeakerWaveIcon class="w-3 h-3 text-purple-400 animate-pulse" />
              </div>
            </div>
            <button @click="closeWindow" class="close-btn">
              <XMarkIcon class="w-5 h-5 text-white/70 hover:text-white" />
            </button>
          </div>
        </div>
      </div>
      
      <!-- Window Content Container -->
      <div class="window-content">
        <!-- Conversation Sidebar -->
        <ConversationSidebar
          :show="showConversationSidebar"
          :conversations="allConversations"
          :is-loading="isLoadingConversations"
          @close="showConversationSidebar = false"
          @new-conversation="handleNewConversation"
          @resume-conversation="handleResumeConversation"
          @delete-conversation="handleDeleteConversation"
        />
        
        <!-- Main Content Area -->
        <div class="main-content" :class="{ 'with-sidebar': showConversationSidebar }">
          <!-- Export Controls -->
          <ExportControls
            :show="showExportControls"
            :selected-count="selectedMessages.size"
            :has-selection="hasSelectedMessages"
            @close="toggleExportControls"
            @export="exportSelectedMessages"
            @select-all="selectAllMessages"
            @deselect-all="deselectAllMessages"
          />
          
          <!-- Conversation Area -->
          <div class="conversation-area">
            <MessageList
              ref="messageListRef"
              :messages="messages"
              :show-export-controls="showExportControls"
              :selected-messages="selectedMessages"
              @toggle-message-selection="toggleMessageSelection"
            />
          </div>
          
          <!-- Bottom Action Bar -->
          <div class="action-bar">
            <button 
              @click="toggleMicrophone" 
              class="mic-btn"
              :class="{ 'active': isRecording }"
              :disabled="!isSpeechInitialized"
            >
              <MicrophoneIcon class="w-6 h-6" />
              <span class="mic-btn-text">{{ isRecording ? 'Stop' : 'Start' }}</span>
            </button>
            <div v-if="speechError" class="error-message">
              {{ speechError }}
            </div>
          </div>
        </div>
        
        <!-- AI Assistant Drawer -->
        <AIAssistant
          :show="showAIAssistant"
          :processing="aiIsProcessing"
          :response="aiResponse"
          :error="aiError"
          :message-count="messages.length"
          @close="showAIAssistant = false"
          @query="handleAIQuery"
        />
        
        <!-- Live AI Drawer -->
        <LiveAI
          :show="showLiveAI"
          :is-active="isLiveAIActive"
          :processing="liveAIIsProcessing"
          :response="liveAIResponse"
          :error="liveAIError"
          @close="showLiveAI = false"
          @toggle-live="toggleLiveAIActive"
        />
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
    0 0 0 1px rgba(255, 255, 255, 0.1),
    0 10px 30px rgba(0, 0, 0, 0.4),
    inset 0 1px 0 rgba(255, 255, 255, 0.15);
}

/* Dynamic width based on visible panels */
.conversational-window:has(.conversation-sidebar) {
  width: 980px;
  max-width: 95vw;
}

.conversational-window:has(.ai-assistant-drawer) {
  width: 980px;
  max-width: 95vw;
}

.conversational-window:has(.live-ai-drawer) {
  width: 980px;
  max-width: 95vw;
}

.conversational-window:has(.conversation-sidebar):has(.ai-assistant-drawer) {
  width: 1280px;
  max-width: 95vw;
}

.conversational-window:has(.conversation-sidebar):has(.live-ai-drawer) {
  width: 1280px;
  max-width: 95vw;
}

.conversational-window:has(.ai-assistant-drawer):has(.live-ai-drawer) {
  width: 1280px;
  max-width: 95vw;
}

.conversational-window:has(.conversation-sidebar):has(.ai-assistant-drawer):has(.live-ai-drawer) {
  width: 1600px;
  max-width: 95vw;
}

.window-content {
  @apply flex-1 flex flex-col min-h-0;
}

.conversational-window:has(.conversation-sidebar) .window-content,
.conversational-window:has(.ai-assistant-drawer) .window-content,
.conversational-window:has(.live-ai-drawer) .window-content {
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

.status-indicators {
  @apply flex items-center gap-2;
}

.export-btn {
  @apply rounded-full p-1 hover:bg-white/10 transition-all duration-200 text-white/60 hover:text-white/90;
}

.export-btn.active {
  @apply bg-blue-500/20 text-blue-400 hover:bg-blue-500/30;
}

.ai-assistant-btn {
  @apply rounded-full p-1 hover:bg-white/10 transition-all duration-200 text-white/60 hover:text-white/90;
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.1), rgba(147, 51, 234, 0.1));
  border: 1px solid rgba(59, 130, 246, 0.2);
}

.ai-assistant-btn.active {
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.4), rgba(147, 51, 234, 0.4));
  border-color: rgba(59, 130, 246, 0.6);
}

.live-ai-btn {
  @apply rounded-full p-1 hover:bg-white/10 transition-all duration-200 text-white/60 hover:text-white/90;
  background: linear-gradient(135deg, rgba(251, 146, 60, 0.1), rgba(234, 88, 12, 0.1));
  border: 1px solid rgba(251, 146, 60, 0.2);
}

.live-ai-btn.active {
  background: linear-gradient(135deg, rgba(251, 146, 60, 0.4), rgba(234, 88, 12, 0.4));
  border-color: rgba(251, 146, 60, 0.6);
}

.status-indicator {
  @apply flex items-center justify-center w-6 h-6 rounded-full bg-white/10;
}

.recording-dot {
  @apply w-2 h-2 bg-red-500 rounded-full animate-pulse;
}

.close-btn {
  @apply rounded-full p-1 hover:bg-white/10 transition-colors;
}

.conversation-area {
  @apply flex-1 overflow-y-auto;
  min-height: 0;
}

/* Scrollbar styles */
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

.conversation-area::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.3);
}

.action-bar {
  @apply flex items-center justify-center gap-4 px-4 py-4 border-t border-white/10;
  background: linear-gradient(to top, rgba(0, 0, 0, 0.4), transparent);
}

.mic-btn {
  @apply flex items-center gap-2 px-6 py-3 rounded-full transition-all duration-200;
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.2), rgba(147, 51, 234, 0.2));
  border: 1px solid rgba(59, 130, 246, 0.3);
  color: rgba(255, 255, 255, 0.8);
}

.mic-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.3), rgba(147, 51, 234, 0.3));
  border-color: rgba(59, 130, 246, 0.5);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.2);
}

.mic-btn.active {
  background: linear-gradient(135deg, rgba(239, 68, 68, 0.8), rgba(220, 38, 38, 0.8));
  border-color: rgba(239, 68, 68, 0.5);
  color: white;
}

.mic-btn.active:hover {
  background: linear-gradient(135deg, rgba(239, 68, 68, 0.9), rgba(220, 38, 38, 0.9));
  border-color: rgba(239, 68, 68, 0.7);
  box-shadow: 0 4px 12px rgba(239, 68, 68, 0.3);
}

.mic-btn:disabled {
  @apply opacity-50 cursor-not-allowed;
}

.mic-btn-text {
  @apply text-sm font-medium;
}

.error-message {
  @apply text-xs text-red-400;
}

/* Transition styles */
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
</style>