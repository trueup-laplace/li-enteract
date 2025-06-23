import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import type { ChatMessage, WindowPosition } from '../types'
import { useSpeechTranscription } from '../composables/useSpeechTranscription'

export const useAppStore = defineStore('app', () => {
  // State
  const micEnabled = ref(false)
  const chatOpen = ref(false)
  const windowCollapsed = ref(false)
  const isRecording = ref(false)
  const recordingTime = ref(0)
  
  // Transparency state
  const transparencyEnabled = ref(false)
  const transparencyLevel = ref(1.0)
  const showTransparencyControls = ref(false)
  
  // Speech transcription state
  const speechTranscription = useSpeechTranscription()
  const currentTranscriptionId = ref<number | null>(null)
  const isTranscriptionEnabled = ref(false)
  
  const chatMessages = ref<ChatMessage[]>([
    { id: 1, text: "Welcome to your agentic assistant", sender: "assistant", timestamp: new Date() },
    { id: 2, text: "How can I help you today?", sender: "assistant", timestamp: new Date() }
  ])
  
  const windowPosition = ref<WindowPosition>({ x: 0, y: 0 })

  // Computed
  const speechStatus = computed(() => ({
    isInitialized: speechTranscription.isInitialized.value,
    isRecording: speechTranscription.isRecording.value,
    isProcessing: speechTranscription.isProcessing.value,
    hasWebSpeechSupport: speechTranscription.hasWebSpeechSupport.value,
    hasWhisperModel: speechTranscription.hasWhisperModel.value,
    error: speechTranscription.error.value
  }))

  // Actions
  const toggleMic = () => {
    micEnabled.value = !micEnabled.value
  }

  const toggleChat = () => {
    chatOpen.value = !chatOpen.value
  }

  const toggleWindowCollapse = () => {
    windowCollapsed.value = !windowCollapsed.value
  }

  const toggleRecording = () => {
    isRecording.value = !isRecording.value
    if (isRecording.value) {
      startRecordingTimer()
    } else {
      recordingTime.value = 0
    }
  }

  // Transparency actions
  const toggleTransparencyControls = () => {
    showTransparencyControls.value = !showTransparencyControls.value
  }

  const updateTransparencyState = (enabled: boolean, level: number) => {
    transparencyEnabled.value = enabled
    transparencyLevel.value = level
  }

  const startRecordingTimer = () => {
    recordingTime.value = 0
    const interval = setInterval(() => {
      if (!isRecording.value) {
        clearInterval(interval)
        return
      }
      recordingTime.value++
    }, 1000)
  }

  const addMessage = (text: string, sender: 'user' | 'assistant' | 'transcription', options?: {
    isInterim?: boolean
    confidence?: number
    source?: 'web-speech' | 'whisper' | 'typed'
  }) => {
    const message: ChatMessage = {
      id: Date.now(),
      text,
      sender,
      timestamp: new Date(),
      ...options
    }
    
    // For transcription messages, always update the current active transcription
    if (sender === 'transcription') {
      if (currentTranscriptionId.value) {
        // Update existing transcription message
        const existingIndex = chatMessages.value.findIndex(m => m.id === currentTranscriptionId.value)
        if (existingIndex !== -1) {
          // Keep the original ID and timestamp for the thought stream
          chatMessages.value[existingIndex] = {
            ...message,
            id: currentTranscriptionId.value,
            timestamp: chatMessages.value[existingIndex].timestamp
          }
          return
        }
      }
      
      // Create new transcription message
      chatMessages.value.push(message)
      currentTranscriptionId.value = message.id
      return
    }
    
    // For non-transcription messages, just add normally
    chatMessages.value.push(message)
  }

  const finalizeTranscription = () => {
    // Mark the current transcription as final and clear the ID
    if (currentTranscriptionId.value) {
      const existingIndex = chatMessages.value.findIndex(m => m.id === currentTranscriptionId.value)
      if (existingIndex !== -1) {
        chatMessages.value[existingIndex].isInterim = false
      }
      currentTranscriptionId.value = null
    }
  }

  const updateMessage = (messageId: number, newText: string) => {
    const messageIndex = chatMessages.value.findIndex(m => m.id === messageId)
    if (messageIndex !== -1) {
      chatMessages.value[messageIndex].text = newText
      chatMessages.value[messageIndex].source = 'typed' // Mark as manually edited
    }
  }

  // Speech transcription actions
  const initializeSpeechTranscription = async (modelSize: 'tiny' | 'base' | 'small' | 'medium' | 'large' = 'base') => {
    try {
      await speechTranscription.initialize({ modelSize })
      isTranscriptionEnabled.value = true
      addMessage("🎤 Speech transcription initialized", "assistant")
    } catch (error) {
      console.error('Failed to initialize speech transcription:', error)
      addMessage(`❌ Failed to initialize speech transcription: ${error}`, "assistant")
    }
  }

  const startSpeechTranscription = async () => {
    if (!speechTranscription.isInitialized.value) {
      await initializeSpeechTranscription()
    }
    
    try {
      await speechTranscription.startRecording()
      addMessage("🎙️ Started listening...", "assistant")
      currentTranscriptionId.value = null
      
      // Watch for transcription updates
      watchTranscriptionUpdates()
    } catch (error) {
      console.error('Failed to start speech transcription:', error)
      addMessage(`❌ Failed to start recording: ${error}`, "assistant")
    }
  }

  const stopSpeechTranscription = async () => {
    try {
      await speechTranscription.stopRecording()
      addMessage("⏹️ Stopped listening", "assistant")
      currentTranscriptionId.value = null
    } catch (error) {
      console.error('Failed to stop speech transcription:', error)
      addMessage(`❌ Failed to stop recording: ${error}`, "assistant")
    }
  }

  const watchTranscriptionUpdates = () => {
    // Watch for interim results from Web Speech API - show as thought stream
    watch(() => speechTranscription.interimText.value, (newText: string) => {
      if (newText && newText.trim()) {
        addMessage(newText, 'transcription', {
          isInterim: true,
          source: 'web-speech'
        })
      }
    })

    // Watch for final results from Web Speech API
    watch(() => speechTranscription.finalText.value, (newText: string) => {
      if (newText && newText.trim()) {
        addMessage(newText, 'transcription', {
          isInterim: false,
          source: 'web-speech',
          confidence: 0.9
        })
        // Don't clear currentTranscriptionId yet - wait for Whisper to potentially improve it
      }
    })

    // Watch for Whisper results - these should replace/improve the Web Speech results
    watch(() => speechTranscription.transcriptionHistory.value, (history: any[]) => {
      const latestWhisperResult = history[history.length - 1]
      if (latestWhisperResult && latestWhisperResult.source === 'whisper') {
        // Replace current transcription with improved Whisper result
        addMessage(latestWhisperResult.text, 'transcription', {
          isInterim: false,
          confidence: latestWhisperResult.confidence,
          source: 'whisper'
        })
        // Finalize this transcription
        finalizeTranscription()
      }
    })
  }

  const clearTranscription = () => {
    speechTranscription.clearTranscription()
    currentTranscriptionId.value = null
    addMessage("🧹 Transcription history cleared", "assistant")
  }

  const clearChat = () => {
    chatMessages.value = []
    currentTranscriptionId.value = null
    // Add a welcome message back
    addMessage("Welcome to your agentic assistant", "assistant")
    addMessage("How can I help you today?", "assistant")
  }

  const updateWindowPosition = (x: number, y: number) => {
    windowPosition.value = { x, y }
  }

  const formatRecordingTime = () => {
    const minutes = Math.floor(recordingTime.value / 60)
    const seconds = recordingTime.value % 60
    return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`
  }

  return {
    // State
    micEnabled,
    chatOpen,
    windowCollapsed,
    isRecording,
    recordingTime,
    chatMessages,
    windowPosition,
    transparencyEnabled,
    transparencyLevel,
    showTransparencyControls,
    isTranscriptionEnabled,
    speechStatus,
    // Actions
    toggleMic,
    toggleChat,
    toggleWindowCollapse,
    toggleRecording,
    addMessage,
    updateMessage,
    finalizeTranscription,
    updateWindowPosition,
    formatRecordingTime,
    toggleTransparencyControls,
    updateTransparencyState,
    // Speech transcription actions
    initializeSpeechTranscription,
    startSpeechTranscription,
    stopSpeechTranscription,
    clearTranscription,
    clearChat
  }
}) 