import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface WakeWordDetectionInfo {
  confidence: number
  timestamp: number
  audio_length: number
}

export interface WakeWordState {
  is_active: boolean
  is_listening: boolean
  last_detection?: WakeWordDetectionInfo
  total_detections: number
  whisper_activated: boolean
}

export function useWakeWordDetection() {
  // State
  const isActive = ref(false)
  const isListening = ref(false)
  const lastDetection = ref<WakeWordDetectionInfo | null>(null)
  const totalDetections = ref(0)
  const whisperActivated = ref(false)
  const error = ref<string | null>(null)
  const isStarting = ref(false)
  const isStopping = ref(false)

  // Polling for detections
  let pollInterval: number | null = null

  // Computed
  const status = computed(() => ({
    isActive: isActive.value,
    isListening: isListening.value,
    lastDetection: lastDetection.value,
    totalDetections: totalDetections.value,
    whisperActivated: whisperActivated.value,
    isStarting: isStarting.value,
    isStopping: isStopping.value,
    hasError: !!error.value,
    error: error.value
  }))

  const hasRecentDetection = computed(() => {
    if (!lastDetection.value) return false
    const now = Date.now()
    const detectionTime = lastDetection.value.timestamp
    return (now - detectionTime) < 5000 // Within last 5 seconds
  })

  // Start wake word detection
  async function startDetection() {
    if (isActive.value || isStarting.value) return

    try {
      isStarting.value = true
      error.value = null
      
      const result = await invoke<string>('start_wake_word_detection')
      console.log('Wake word detection started:', result)
      
      await updateState()
      startPolling()
      
      isStarting.value = false
    } catch (err) {
      isStarting.value = false
      const message = err instanceof Error ? err.message : String(err)
      error.value = `Failed to start wake word detection: ${message}`
      console.error('Failed to start wake word detection:', err)
      throw err
    }
  }

  // Stop wake word detection
  async function stopDetection() {
    if (!isActive.value || isStopping.value) return

    try {
      isStopping.value = true
      
      stopPolling()
      
      const result = await invoke<string>('stop_wake_word_detection')
      console.log('Wake word detection stopped:', result)
      
      await updateState()
      
      isStopping.value = false
    } catch (err) {
      isStopping.value = false
      const message = err instanceof Error ? err.message : String(err)
      error.value = `Failed to stop wake word detection: ${message}`
      console.error('Failed to stop wake word detection:', err)
      throw err
    }
  }

  // Toggle detection
  async function toggleDetection() {
    if (isActive.value) {
      await stopDetection()
    } else {
      await startDetection()
    }
  }

  // Update state from backend
  async function updateState() {
    try {
      const state = await invoke<WakeWordState>('get_wake_word_state')
      
      isActive.value = state.is_active
      isListening.value = state.is_listening
      totalDetections.value = state.total_detections
      whisperActivated.value = state.whisper_activated
      
      if (state.last_detection) {
        lastDetection.value = state.last_detection
      }
    } catch (err) {
      console.error('Failed to get wake word state:', err)
    }
  }

  // Check for new detections
  async function checkForDetection() {
    try {
      const detection = await invoke<WakeWordDetectionInfo | null>('check_wake_word_detection')
      
      if (detection) {
        lastDetection.value = detection
        totalDetections.value += 1
        whisperActivated.value = true
        
        console.log('Wake word detected!', {
          confidence: detection.confidence,
          timestamp: new Date(detection.timestamp),
          audioLength: detection.audio_length
        })
        
        // Emit custom event for other components to listen to
        const event = new CustomEvent('wakeWordDetected', {
          detail: detection
        })
        window.dispatchEvent(event)
      }
    } catch (err) {
      console.error('Failed to check for wake word detection:', err)
    }
  }

  // Reset statistics
  async function resetStats() {
    try {
      await invoke<string>('reset_wake_word_stats')
      totalDetections.value = 0
      lastDetection.value = null
      whisperActivated.value = false
      
      console.log('Wake word statistics reset')
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err)
      error.value = `Failed to reset statistics: ${message}`
      console.error('Failed to reset wake word stats:', err)
      throw err
    }
  }

  // Start polling for detections
  function startPolling() {
    if (pollInterval) return
    
    // Poll every 200ms for new detections
    pollInterval = setInterval(async () => {
      if (isActive.value) {
        await checkForDetection()
        await updateState()
      }
    }, 200)
  }

  // Stop polling
  function stopPolling() {
    if (pollInterval) {
      clearInterval(pollInterval)
      pollInterval = null
    }
  }

  // Clear error
  function clearError() {
    error.value = null
  }

  // Initialize on mount
  onMounted(async () => {
    await updateState()
    
    // If wake word detection was already active, start polling
    if (isActive.value) {
      startPolling()
    }
  })

  // Cleanup on unmount
  onUnmounted(() => {
    stopPolling()
  })

  return {
    // State
    isActive,
    isListening,
    lastDetection,
    totalDetections,
    whisperActivated,
    error,
    isStarting,
    isStopping,
    
    // Computed
    status,
    hasRecentDetection,
    
    // Methods
    startDetection,
    stopDetection,
    toggleDetection,
    updateState,
    checkForDetection,
    resetStats,
    clearError
  }
} 