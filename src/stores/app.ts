import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { ChatMessage, WindowPosition } from '../types'

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
  
  const chatMessages = ref<ChatMessage[]>([
    { id: 1, text: "Welcome to your agentic assistant", sender: "assistant", timestamp: new Date() },
    { id: 2, text: "How can I help you today?", sender: "assistant", timestamp: new Date() }
  ])
  
  const windowPosition = ref<WindowPosition>({ x: 0, y: 0 })

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

  const addMessage = (text: string, sender: 'user' | 'assistant') => {
    chatMessages.value.push({
      id: Date.now(),
      text,
      sender,
      timestamp: new Date()
    })
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
    // Actions
    toggleMic,
    toggleChat,
    toggleWindowCollapse,
    toggleRecording,
    addMessage,
    updateWindowPosition,
    formatRecordingTime,
    toggleTransparencyControls,
    updateTransparencyState
  }
}) 