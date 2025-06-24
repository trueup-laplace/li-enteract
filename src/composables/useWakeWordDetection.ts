import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Extend the SpeechRecognition interface
declare global {
  interface Window {
    SpeechRecognition: typeof SpeechRecognition
    webkitSpeechRecognition: typeof SpeechRecognition
  }
}

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
  const hasWebSpeechSupport = ref(false)

  // Speech recognition for browser-based wake word detection
  let speechRecognition: SpeechRecognition | null = null
  let audioStream: MediaStream | null = null

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
    hasWebSpeechSupport: hasWebSpeechSupport.value,
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
      
      // Check Web Speech API support
      const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition
      hasWebSpeechSupport.value = !!SpeechRecognition

      if (hasWebSpeechSupport.value) {
        // Use Web Speech API for wake word detection
        console.log('Using Web Speech API for wake word detection')
        
        // Request microphone permission
        const hasPermission = await requestMicrophonePermission()
        if (!hasPermission) {
          throw new Error('Microphone permission required for wake word detection')
        }
        
        // Setup and start speech recognition
        speechRecognition = setupSpeechRecognition()
        if (speechRecognition) {
          speechRecognition.start()
          isActive.value = true
        } else {
          throw new Error('Failed to setup speech recognition')
        }
      } else {
        // Fallback to Rust backend
        console.log('Web Speech API not supported, using Rust backend')
        const result = await invoke<string>('start_wake_word_detection')
        console.log('Wake word detection started:', result)
        isActive.value = true
        await updateState()
        startPolling()
      }
      
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
      
      if (speechRecognition && hasWebSpeechSupport.value) {
        // Stop Web Speech API
        console.log('Stopping Web Speech API wake word detection')
        speechRecognition.stop()
        speechRecognition = null
        
        // Stop audio stream
        if (audioStream) {
          audioStream.getTracks().forEach(track => track.stop())
          audioStream = null
        }
        
        isActive.value = false
        isListening.value = false
      } else {
        // Stop Rust backend
        stopPolling()
        
        const result = await invoke<string>('stop_wake_word_detection')
        console.log('Wake word detection stopped:', result)
        
        await updateState()
      }
      
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

  // Setup Web Speech Recognition for wake word detection
  function setupSpeechRecognition(): SpeechRecognition | null {
    const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition
    
    if (!SpeechRecognition) {
      console.error('Speech Recognition not supported in this browser')
      return null
    }
    
    const recognition = new SpeechRecognition()
    recognition.continuous = true
    recognition.interimResults = true
    recognition.lang = 'en-US'
    recognition.maxAlternatives = 1
    
    recognition.onresult = (event) => {
      const transcript = Array.from(event.results)
        .map(result => result[0].transcript)
        .join('')
        .toLowerCase()
      
      // Check for wake word "aubrey"
      if (transcript.includes('aubrey')) {
        const confidence = event.results[event.resultIndex]?.[0]?.confidence || 0.8
        triggerWakeWord(confidence)
      }
    }
    
    recognition.onerror = (event) => {
      console.error('Speech recognition error:', event.error)
      error.value = `Speech recognition error: ${event.error}`
      
      // Handle specific errors
      if (event.error === 'not-allowed') {
        error.value = 'Microphone permission denied for wake word detection'
      } else if (event.error === 'network') {
        error.value = 'Network error during wake word detection'
      }
      
      // Implement retry logic
      if (isActive.value && event.error !== 'aborted') {
        setTimeout(() => {
          if (isActive.value) {
            try {
              recognition.start()
            } catch (retryError) {
              console.warn('Failed to restart wake word detection:', retryError)
            }
          }
        }, 2000) // 2 second delay before retry
      }
    }
    
    recognition.onstart = () => {
      console.log('Wake word detection started')
      isListening.value = true
      error.value = null
    }
    
    recognition.onend = () => {
      console.log('Wake word detection ended')
      isListening.value = false
      
      // Restart if still active
      if (isActive.value) {
        setTimeout(() => {
          if (isActive.value && recognition) {
            try {
              recognition.start()
            } catch (err) {
              console.warn('Failed to restart wake word detection:', err)
            }
          }
        }, 1000)
      }
    }
    
    return recognition
  }

  // Request microphone permission for wake word detection
  async function requestMicrophonePermission(): Promise<boolean> {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true })
      audioStream = stream
      return true
    } catch (err) {
      console.error('Microphone permission denied:', err)
      error.value = 'Microphone permission required for wake word detection'
      return false
    }
  }

  // Trigger wake word detection event
  function triggerWakeWord(confidence: number = 0.8) {
    const detection: WakeWordDetectionInfo = {
      confidence,
      timestamp: Date.now(),
      audio_length: 1000 // Approximate length
    }
    
    lastDetection.value = detection
    totalDetections.value += 1
    whisperActivated.value = true
    
    console.log('Wake word "Aubrey" detected!', {
      confidence,
      timestamp: new Date(detection.timestamp),
      totalDetections: totalDetections.value
    })
    
    // Emit custom event for other components to listen to
    const event = new CustomEvent('wakeWordDetected', {
      detail: detection
    })
    window.dispatchEvent(event)
    
    // Auto-reset whisper activation after 5 seconds
    setTimeout(() => {
      whisperActivated.value = false
    }, 5000)
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
    hasWebSpeechSupport,
    
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
    clearError,
    setupSpeechRecognition,
    requestMicrophonePermission,
    triggerWakeWord
  }
} 