<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick, computed, watch } from 'vue'
import { 
  MicrophoneIcon, 
  SpeakerWaveIcon, 
  XMarkIcon,
  PaperAirplaneIcon,
  StopIcon,
  ChatBubbleLeftRightIcon,
  CheckIcon,
  QueueListIcon,
  PlusIcon,
  PlayIcon,
  PencilIcon,
  TrashIcon,
  EllipsisVerticalIcon,
  ClockIcon
} from '@heroicons/vue/24/outline'
import { useSpeechTranscription } from '../../composables/useSpeechTranscription'
import { useConversationStore } from '../../stores/conversation'
import { useWindowResizing } from '../../composables/useWindowResizing'
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

// Store and composables
const conversationStore = useConversationStore()
const { resizeWindow } = useWindowResizing()

// State
const scrollContainer = ref<HTMLElement>()
const audioLoopbackDeviceId = ref<string | null>(null)
const selectedMessages = ref<Set<string>>(new Set())
const showExportControls = ref(false)

// Conversation sidebar state
const showConversationSidebar = ref(false)
const allConversations = ref<any[]>([])
const isLoadingConversations = ref(false)

// Loopback transcription buffering for complete thoughts
const loopbackBuffer = ref<string>('')
const loopbackLastTimestamp = ref<number>(0)
const loopbackThoughtTimer = ref<number | null>(null)
const loopbackBufferStartTime = ref<number>(0)
const THOUGHT_PAUSE_DURATION = 2000 // 2 seconds
const MAX_BUFFER_DURATION = 10000 // 10 seconds for sizeable content
const MIN_SIZEABLE_CONTENT = 50 // minimum characters for sizeable content

// Deduplication tracking
const lastProcessedText = ref<string>('')
const lastProcessedTimestamp = ref<number>(0)
const recentMessages = ref<Set<string>>(new Set())

// Typing animation state
const isLoopbackTyping = ref(false)
const loopbackPreviewMessage = ref<string>('')
const currentPreviewMessageId = ref<string | null>(null)

// User microphone typing state
const isMicrophoneTyping = ref(false)
const microphonePreviewMessage = ref<string>('')
const currentMicPreviewMessageId = ref<string | null>(null)

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

// Watch for sidebar changes to resize window appropriately
watch(showConversationSidebar, async (newValue) => {
  try {
    console.log(`🔧 CONVERSATION SIDEBAR TOGGLED: ${newValue}`)
    // Trigger window resize to accommodate sidebar changes
    // Pass the sidebar state to determine proper width (600px vs 980px)
    await resizeWindow(false, false, false, true, newValue)
    console.log('✅ Window resized for conversation sidebar')
  } catch (error) {
    console.error('❌ Failed to resize window for sidebar:', error)
  }
})

// Initialize when component mounts
onMounted(async () => {
  try {
    // Don't create a session here - sessions are created when recording starts
    // This ensures one conversation per recording session
    
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
  window.addEventListener('transcription-interim', handleConversationalUserInterim)
  
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

// Handle user speech interim results for typing animation
const handleConversationalUserInterim = (event: Event) => {
  const customEvent = event as CustomEvent
  
  // Only process if we're recording in conversational mode
  if (!isRecording.value) return
  
  const { text } = customEvent.detail
  
  if (text && text.trim()) {
    microphonePreviewMessage.value = text.trim()
    isMicrophoneTyping.value = true
    currentMicPreviewMessageId.value = `mic-preview-${Date.now()}`
    scrollToBottom()
  }
}

// Handle user speech from microphone - ONLY for conversational window
const handleConversationalUserSpeech = (event: Event) => {
  const customEvent = event as CustomEvent
  
  // Only process if we're recording in conversational mode
  if (!isRecording.value) return
  
  const { text, confidence, timestamp } = customEvent.detail
  
  if (text && text.trim()) {
    const finalContent = text.trim()
    
    // Clear typing state FIRST to prevent duplication
    isMicrophoneTyping.value = false
    microphonePreviewMessage.value = ''
    currentMicPreviewMessageId.value = null
    
    // Then add the final message
    conversationStore.addMessage({
      type: 'user',
      source: 'microphone',
      content: finalContent,
      confidence,
      timestamp: timestamp || Date.now()
    })
    
    scrollToBottom()
  }
}

// Handle system audio from loopback (just for monitoring audio levels)
const handleSystemAudio = (event: CustomEvent) => {
  console.log('🎧 Received audio-chunk event - level:', event.detail.level, 'dB')
  // This is now just for monitoring - transcription happens in Rust backend
}

// Handle loopback transcription results with thought grouping and typing animation
const handleLoopbackTranscription = (payload: any) => {
  console.log('🎙️ Loopback transcription chunk:', payload)
  const { text, confidence, timestamp, audioLevel } = payload
  
  if (text && text.trim()) {
    const currentTime = timestamp || Date.now()
    const trimmedText = text.trim()
    
    // Deduplication check - ignore if we just processed the same text recently
    const textFingerprint = `${trimmedText}_${Math.floor(currentTime / 1000)}` // Group by second
    if (lastProcessedText.value === trimmedText && 
        (currentTime - lastProcessedTimestamp.value) < 500) { // 500ms window
      console.log('🎙️ Skipping duplicate transcription:', trimmedText)
      return
    }
    
    // Update last processed tracking
    lastProcessedText.value = trimmedText
    lastProcessedTimestamp.value = currentTime
    
    const timeSinceLastChunk = currentTime - loopbackLastTimestamp.value
    
    // If more than 2 seconds have passed, start a new thought
    if (timeSinceLastChunk > THOUGHT_PAUSE_DURATION && loopbackBuffer.value.trim()) {
      flushLoopbackBuffer()
    }
    
    // Initialize buffer start time if this is the first chunk
    if (!loopbackBuffer.value.trim()) {
      loopbackBufferStartTime.value = currentTime
      currentPreviewMessageId.value = `loopback-preview-${currentTime}`
    }
    
    // Check if this text is already in the buffer to avoid concatenating duplicates
    if (!loopbackBuffer.value.includes(trimmedText)) {
      // Add current text to buffer
      if (loopbackBuffer.value.trim()) {
        loopbackBuffer.value += ' ' + trimmedText
      } else {
        loopbackBuffer.value = trimmedText
      }
    } else {
      console.log('🎙️ Text already in buffer, updating timestamp only')
    }
    
    // Update real-time preview
    loopbackPreviewMessage.value = loopbackBuffer.value
    isLoopbackTyping.value = true
    
    loopbackLastTimestamp.value = currentTime
    
    // Check if we should flush due to sizeable content + time
    const bufferDuration = currentTime - loopbackBufferStartTime.value
    const hasSize = loopbackBuffer.value.length >= MIN_SIZEABLE_CONTENT
    const hasTime = bufferDuration >= MAX_BUFFER_DURATION
    
    if (hasSize && hasTime) {
      console.log('🎙️ Flushing buffer due to sizeable content + time:', loopbackBuffer.value.length, 'chars,', bufferDuration, 'ms')
      flushLoopbackBuffer()
      return
    }
    
    // Clear existing timer and start new one
    if (loopbackThoughtTimer.value) {
      clearTimeout(loopbackThoughtTimer.value)
    }
    
    // Set timer to flush buffer after pause duration
    loopbackThoughtTimer.value = window.setTimeout(() => {
      flushLoopbackBuffer()
    }, THOUGHT_PAUSE_DURATION)
    
    // Auto-scroll to show typing animation
    scrollToBottom()
    
    console.log(`🎙️ System Audio chunk (${audioLevel?.toFixed(1)}dB): ${trimmedText} [buffered: "${loopbackBuffer.value}" (${loopbackBuffer.value.length} chars, ${bufferDuration}ms)]`)
  }
}

// Flush the loopback buffer as a complete thought
const flushLoopbackBuffer = () => {
  if (loopbackBuffer.value.trim()) {
    const finalContent = loopbackBuffer.value.trim()
    
    // Check for duplicate messages - don't add if the last message has the same content
    const existingMessages = conversationStore.currentMessages || []
    const lastMessage = existingMessages[existingMessages.length - 1]
    
    if (lastMessage && 
        lastMessage.source === 'loopback' && 
        lastMessage.content === finalContent) {
      console.log('🎙️ Skipping duplicate final message:', finalContent)
      // Still clear the buffer state
      loopbackBuffer.value = ''
      loopbackPreviewMessage.value = ''
      isLoopbackTyping.value = false
      currentPreviewMessageId.value = null
      loopbackBufferStartTime.value = 0
      return
    }
    
    // Add to recent messages tracking to prevent immediate duplicates
    const messageFingerprint = finalContent.toLowerCase().replace(/[^\w\s]/g, '')
    if (recentMessages.value.has(messageFingerprint)) {
      console.log('🎙️ Skipping recently added message:', finalContent)
      // Still clear the buffer state
      loopbackBuffer.value = ''
      loopbackPreviewMessage.value = ''
      isLoopbackTyping.value = false
      currentPreviewMessageId.value = null
      loopbackBufferStartTime.value = 0
      return
    }
    
    // Clear buffer and typing state FIRST to prevent duplication
    loopbackBuffer.value = ''
    loopbackPreviewMessage.value = ''
    isLoopbackTyping.value = false
    currentPreviewMessageId.value = null
    loopbackBufferStartTime.value = 0
    
    // Add to recent messages tracking
    recentMessages.value.add(messageFingerprint)
    // Clean up old entries after 30 seconds
    setTimeout(() => {
      recentMessages.value.delete(messageFingerprint)
    }, 30000)
    
    // Then add the final message
    conversationStore.addMessage({
      type: 'system',
      source: 'loopback',
      content: finalContent,
      confidence: 0.8, // Average confidence for grouped chunks
      timestamp: Date.now()
    })
    
    console.log(`🎙️ Complete thought: "${finalContent}"`)
    
    scrollToBottom()
  }
  
  if (loopbackThoughtTimer.value) {
    clearTimeout(loopbackThoughtTimer.value)
    loopbackThoughtTimer.value = null
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
      // Flush any pending loopback buffer before stopping
      flushLoopbackBuffer()
      
      // Stop both microphone and audio loopback
      await stopRecording()
      conversationStore.setRecordingState(false)
      // Always try to stop audio loopback when stopping recording
      await stopAudioLoopbackCapture()
      
      // Save the current session when recording stops, but keep it active
      if (conversationStore.currentSession && conversationStore.currentSession.messages.length > 0) {
        // Mark the session as ended but don't set currentSession to null
        // This keeps the window open while preserving the conversation
        conversationStore.currentSession.isActive = false
        conversationStore.currentSession.endTime = Date.now()
        console.log('💾 Recording stopped - conversation saved but session remains accessible')
      } else {
        console.log('🎤 Recording stopped - no messages to save')
      }
    } else {
      // Create a new session when starting recording, or reactivate the current one
      if (!conversationStore.currentSession || conversationStore.currentSession.messages.length === 0) {
        conversationStore.createSession()
      } else {
        // Reactivate existing session for continued recording
        conversationStore.currentSession.isActive = true
        conversationStore.currentSession.endTime = undefined
      }
      
      // Start both microphone and audio loopback
      conversationStore.setRecordingState(true)
      await startRecording()
      if (audioLoopbackDeviceId.value && !isAudioLoopbackActive.value) {
        await startAudioLoopbackCapture()
      }
      
      console.log('🎤 Recording started - conversation session ready')
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

// Conversation sidebar functions
const toggleConversationSidebar = () => {
  showConversationSidebar.value = !showConversationSidebar.value
  if (showConversationSidebar.value) {
    loadConversations()
  }
}

const loadConversations = async () => {
  try {
    isLoadingConversations.value = true
    const response = await invoke<{conversations: any[]}>('load_conversations')
    allConversations.value = response.conversations.sort((a, b) => b.startTime - a.startTime)
    console.log(`📁 Loaded ${allConversations.value.length} conversations`)
  } catch (error) {
    console.error('Failed to load conversations:', error)
  } finally {
    isLoadingConversations.value = false
  }
}

const handleNewConversation = async (event?: Event) => {
  console.log('🆕 Starting new conversation - ENTRY')
  if (event) {
    event.preventDefault()
    event.stopPropagation()
  }
  
  try {
    // Prevent any recording-related state conflicts
    if (isRecording.value) {
      console.log('⚠️ Recording active - stopping before creating new conversation')
      await handleMicrophoneToggle() // This will save current session
    }
    
    // Use nextTick to ensure clean state transition
    await nextTick()
    console.log('🆕 After nextTick')
    
    // Always create a fresh session for new conversation
    console.log('🆕 Creating new session...')
    conversationStore.createSession()
    console.log('🆕 Session created successfully')
    
    showConversationSidebar.value = false
    console.log('✅ New conversation started successfully - window should remain open')
  } catch (error) {
    console.error('❌ Failed to create new conversation:', error)
    console.error('❌ Error details:', error)
    // Ensure sidebar closes even on error
    showConversationSidebar.value = false
  }
}

const handleResumeConversation = async (conversationId: string, event?: Event) => {
  console.log('🔄 Resume conversation - ENTRY, ID:', conversationId)
  if (event) {
    event.preventDefault()
    event.stopPropagation()
  }
  
  try {
    // Stop any active recording before switching
    if (isRecording.value) {
      console.log('⚠️ Recording active - stopping before resuming conversation')
      await handleMicrophoneToggle() // This will save current session
    }
    
    console.log('🔄 Before nextTick')
    // Use nextTick to ensure state updates are processed
    await nextTick()
    console.log('🔄 After nextTick')
    
    console.log('🔄 Switching to session:', conversationId)
    // Switch to the selected conversation
    conversationStore.switchToSession(conversationId)
    console.log('🔄 Session switched successfully')
    
    showConversationSidebar.value = false
    console.log('🔄 Sidebar closed')
    
    // Scroll to bottom to show latest messages
    await scrollToBottom()
    
    console.log('✅ Conversation resumed successfully - window should remain open:', conversationId)
  } catch (error) {
    console.error('❌ Failed to resume conversation:', error)
    console.error('❌ Error details:', error)
    // Don't let errors propagate and close the window
    showConversationSidebar.value = false // At least close sidebar on error
  }
}

const handleDeleteConversation = async (conversationId: string, event?: Event) => {
  console.log('🗑️ Delete conversation - ENTRY, ID:', conversationId)
  if (event) {
    event.preventDefault()
    event.stopPropagation()
  }
  
  try {
    // Stop any active recording if we're deleting the current session
    if (conversationStore.currentSession?.id === conversationId && isRecording.value) {
      console.log('⚠️ Stopping recording before deleting current session')
      await handleMicrophoneToggle()
    }
    
    console.log('🗑️ Before delete operation')
    // If deleting current session, clear it first
    const wasCurrentSession = conversationStore.currentSession?.id === conversationId
    if (wasCurrentSession) {
      console.log('⚠️ Deleting current active session - clearing session')
      conversationStore.currentSession = null
    }
    
    // Delete the conversation
    console.log('🗑️ Calling deleteSession...')
    await conversationStore.deleteSession(conversationId)
    console.log('🗑️ DeleteSession completed')
    
    // If we deleted the current session, user can create new one when recording
    if (wasCurrentSession) {
      console.log('📝 Deleted current session - ready for new conversation')
    }
    
    // Reload conversations list
    console.log('🗑️ Reloading conversations...')
    await loadConversations()
    console.log('✅ Conversation deleted successfully - window should remain open')
  } catch (error) {
    console.error('❌ Failed to delete conversation:', error)
    console.error('❌ Error details:', error)
    // Don't let deletion errors close the window
    // Just reload conversations to ensure UI state is consistent
    try {
      console.log('🗑️ Attempting to reload conversations after error...')
      await loadConversations()
    } catch (reloadError) {
      console.error('❌ Failed to reload conversations after delete error:', reloadError)
    }
  }
}

// Format conversation display helpers
const formatTimestamp = (timestamp: number) => {
  const date = new Date(timestamp)
  const now = new Date()
  const diffDays = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60 * 24))
  
  if (diffDays === 0) {
    return date.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' })
  } else if (diffDays === 1) {
    return 'Yesterday'
  } else if (diffDays < 7) {
    return date.toLocaleDateString('en-US', { weekday: 'short' })
  }
  return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' })
}

const formatDuration = (startTime: number, endTime?: number) => {
  const duration = (endTime || Date.now()) - startTime
  const minutes = Math.floor(duration / 60000)
  const seconds = Math.floor((duration % 60000) / 1000)
  
  if (minutes > 0) {
    return `${minutes}m ${seconds}s`
  }
  return `${seconds}s`
}

const getConversationSummary = (conversation: any) => {
  const userMessages = conversation.messages.filter((m: any) => m.type === 'user').length
  const systemMessages = conversation.messages.filter((m: any) => m.type === 'system').length
  return `${userMessages} you, ${systemMessages} system`
}

const getLastMessagePreview = (conversation: any) => {
  if (conversation.messages.length === 0) return 'No messages'
  const lastMessage = conversation.messages[conversation.messages.length - 1]
  return lastMessage.content.substring(0, 50) + (lastMessage.content.length > 50 ? '...' : '')
}

// Cleanup
onUnmounted(async () => {
  window.removeEventListener('transcription-final', handleConversationalUserSpeech)
  window.removeEventListener('transcription-interim', handleConversationalUserInterim)
  
  // Flush any remaining loopback buffer
  flushLoopbackBuffer()
  
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
  
  // End any active session when window closes (conversations are now saved per recording session)
  if (conversationStore.currentSession && conversationStore.currentSession.messages.length > 0) {
    conversationStore.endSession()
    console.log('💾 Final conversation saved when window closed')
  } else if (conversationStore.currentSession) {
    // If there are no messages, just clear the session without saving
    conversationStore.currentSession = null
    console.log('🗑️ Empty conversation session cleared on window close')
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
              
              <!-- Conversation History Button -->
              <button 
                @click="toggleConversationSidebar" 
                class="export-btn"
                :class="{ 'active': showConversationSidebar }"
                title="Conversation History"
              >
                <QueueListIcon class="w-4 h-4" />
              </button>
              
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
        
        <!-- Window Content Container -->
        <div class="window-content">
          <!-- Conversation Sidebar -->
          <div v-if="showConversationSidebar" class="conversation-sidebar">
            <div class="sidebar-header">
              <div class="flex items-center gap-2">
                <QueueListIcon class="w-4 h-4 text-purple-400" />
                <h3 class="text-sm font-medium text-white">Conversations</h3>
              </div>
              <button @click="showConversationSidebar = false" class="close-sidebar-btn">
                <XMarkIcon class="w-4 h-4" />
              </button>
            </div>
            
            <div class="sidebar-content">
              <!-- New Conversation Button -->
              <button @click.stop="handleNewConversation" class="new-conversation-btn">
                <PlusIcon class="w-4 h-4" />
                New Conversation
              </button>
              
              <!-- Conversations List -->
              <div class="conversations-list">
                <div v-if="isLoadingConversations" class="loading-state">
                  <div class="loading-spinner"></div>
                  <span class="text-xs text-white/60">Loading...</span>
                </div>
                
                <div v-else-if="allConversations.length === 0" class="empty-state">
                  <QueueListIcon class="w-8 h-8 text-white/20 mx-auto mb-2" />
                  <p class="text-white/60 text-xs text-center">No conversations yet</p>
                </div>
                
                <div v-else class="conversations-grid">
                  <div 
                    v-for="conversation in allConversations" 
                    :key="conversation.id"
                    class="conversation-item"
                    :class="{ 'active': conversation.isActive }"
                    @click.stop="(event) => handleResumeConversation(conversation.id, event)"
                  >
                    <div class="conversation-header">
                      <span class="conversation-title">{{ conversation.name }}</span>
                      <button 
                        @click.stop="(event) => handleDeleteConversation(conversation.id, event)"
                        class="delete-btn"
                        title="Delete conversation"
                      >
                        <TrashIcon class="w-3 h-3" />
                      </button>
                    </div>
                    
                    <div class="conversation-meta">
                      <div class="meta-row">
                        <ClockIcon class="w-3 h-3 text-white/40" />
                        <span class="text-xs text-white/40">{{ formatTimestamp(conversation.startTime) }}</span>
                        <span class="text-xs text-white/40">•</span>
                        <span class="text-xs text-white/40">{{ formatDuration(conversation.startTime, conversation.endTime) }}</span>
                      </div>
                      <div class="meta-row">
                        <span class="text-xs text-white/50">{{ getConversationSummary(conversation) }}</span>
                      </div>
                    </div>
                    
                    <div class="conversation-preview">
                      <p class="text-xs text-white/60">{{ getLastMessagePreview(conversation) }}</p>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        
          <!-- Main Content Area -->
          <div class="main-content" :class="{ 'with-sidebar': showConversationSidebar }">
        
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
        <div ref="scrollContainer" class="conversation-area" :class="{ 'with-sidebar': showConversationSidebar }">
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
                'selected': selectedMessages.has(message.id),
                'typing-preview': message.isPreview && message.isTyping
              }"
              @click="showExportControls ? toggleMessageSelection(message.id) : null"
            >
              <div class="message-bubble" :class="{
                'user-bubble': message.type === 'user',
                'system-bubble': message.type === 'system',
                'typing-bubble': message.isPreview && message.isTyping
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
                <div class="message-content">
                  {{ message.content }}
                  <span v-if="message.isPreview && message.isTyping" class="typing-indicator">
                    <span class="typing-dot"></span>
                    <span class="typing-dot"></span>
                    <span class="typing-dot"></span>
                  </span>
                </div>
                <div v-if="message.confidence && !message.isPreview" class="message-confidence">
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
          </div> <!-- End main-content -->
        </div> <!-- End window-content -->
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

/* When sidebar is shown, make window wider and use row layout */
.conversational-window:has(.conversation-sidebar) {
  width: 980px;
  max-width: 95vw;
}

.window-content {
  @apply flex-1 flex flex-col min-h-0;
}

.conversational-window:has(.conversation-sidebar) .window-content {
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

/* Typing animation */
.typing-preview {
  animation: fadeIn 0.3s ease-in-out;
  opacity: 0.6; /* Lower opacity for typing previews */
}

.typing-bubble {
  opacity: 0.5; /* Even lower opacity for typing bubbles */
  border: 1px dashed rgba(255, 255, 255, 0.3) !important;
  animation: typingPulse 2s infinite ease-in-out; /* Slow pulsing animation */
  background: rgba(255, 255, 255, 0.02) !important; /* Subtle background to distinguish from final messages */
  position: relative;
}

.typing-bubble::before {
  content: '';
  position: absolute;
  top: -2px;
  left: -2px;
  right: -2px;
  bottom: -2px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: inherit;
  animation: typingGlow 2s infinite ease-in-out;
  pointer-events: none;
}

@keyframes typingGlow {
  0%, 100% {
    opacity: 0.2;
    transform: scale(1);
  }
  50% {
    opacity: 0.4;
    transform: scale(1.02);
  }
}

.typing-indicator {
  @apply inline-flex items-center gap-1 ml-2;
}

.typing-dot {
  @apply w-1 h-1 bg-current rounded-full opacity-60;
  animation: typingDotPulse 1.4s infinite ease-in-out;
}

.typing-dot:nth-child(1) {
  animation-delay: -0.32s;
}

.typing-dot:nth-child(2) {
  animation-delay: -0.16s;
}

.typing-dot:nth-child(3) {
  animation-delay: 0s;
}

@keyframes typingPulse {
  0%, 100% {
    opacity: 0.4;
    transform: scale(0.98);
  }
  50% {
    opacity: 0.7;
    transform: scale(1.02);
  }
}

/* Separate animation for typing dots */
@keyframes typingDotPulse {
  0%, 80%, 100% {
    opacity: 0.3;
    transform: scale(0.8);
  }
  40% {
    opacity: 1;
    transform: scale(1);
  }
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
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

/* Conversation Sidebar */
.conversation-sidebar {
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

.new-conversation-btn {
  @apply w-full flex items-center justify-center gap-2 mx-4 my-3 px-4 py-2 bg-purple-500/80 hover:bg-purple-500 text-white rounded-lg transition-all duration-200 font-medium text-sm;
}

.conversations-list {
  @apply flex-1 overflow-y-auto px-4 pb-4;
}

.loading-state {
  @apply flex flex-col items-center gap-2 py-8;
}

.loading-spinner {
  @apply w-4 h-4 border-2 border-white/20 border-t-white/60 rounded-full animate-spin;
}

.empty-state {
  @apply py-8 text-center;
}

.conversations-grid {
  @apply space-y-2;
}

.conversation-item {
  @apply rounded-lg bg-white/5 hover:bg-white/10 transition-all duration-200 cursor-pointer p-3 border border-transparent;
}

.conversation-item.active {
  @apply border-purple-500/50 bg-purple-500/10;
}

.conversation-header {
  @apply flex items-center justify-between mb-2;
}

.conversation-title {
  @apply text-xs font-medium text-white truncate flex-1 mr-2;
}

.delete-btn {
  @apply rounded-full p-1 hover:bg-red-500/20 transition-colors text-white/60 hover:text-red-400;
}

.conversation-meta {
  @apply space-y-1 mb-2;
}

.meta-row {
  @apply flex items-center gap-1;
}

.conversation-preview {
  @apply line-clamp-2;
}


/* Scrollbar */
.conversation-area::-webkit-scrollbar,
.conversations-list::-webkit-scrollbar {
  width: 4px;
}

.conversation-area::-webkit-scrollbar-track,
.conversations-list::-webkit-scrollbar-track {
  background: transparent;
}

.conversation-area::-webkit-scrollbar-thumb,
.conversations-list::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
}
</style>