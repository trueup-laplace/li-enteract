import { ref, computed, onMounted, onUnmounted, readonly } from 'vue'
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

  // Debouncing for wake word detection
  let lastWakeWordTime = 0
  const debounceTime = 3000 // 3 seconds
  let transcriptionPausedUntil = 0

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

  const isTranscriptionPaused = computed(() => {
    return Date.now() < transcriptionPausedUntil
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
        
        // Trigger transcription with debouncing
        await handleWakeWordDetected(detection)
        
        // Emit custom event for other components to listen to
        const event = new CustomEvent('wake-word-detected', { 
          detail: detection 
        })
        window.dispatchEvent(event)
      }
      
      await updateState()
    } catch (err) {
      console.error('Failed to check for wake word detection:', err)
    }
  }

  // Handle wake word detection with debouncing and transcription triggering
  async function handleWakeWordDetected(detection: WakeWordDetectionInfo) {
    const now = Date.now()
    
    // Check debouncing
    if (now - lastWakeWordTime < debounceTime) {
      console.log('Wake word detected but debouncing - ignoring')
      return
    }
    
    // Check if we're in transcription pause period
    if (isTranscriptionPaused.value) {
      console.log('Wake word detected but transcription is paused - ignoring')
      return
    }
    
    lastWakeWordTime = now
    
    try {
      console.log('🎤 Wake word "Aubrey" detected - starting transcription!')
      
      // Provide visual/audio feedback
      await provideFeedback()
      
      // Temporarily pause wake word detection during transcription
      pauseWakeWordDetection(15000) // 15 seconds
      
      // Trigger transcription by emitting event for the speech transcription module
      const transcriptionEvent = new CustomEvent('start-transcription-from-wake-word', {
        detail: {
          confidence: detection.confidence,
          timestamp: detection.timestamp,
          audioLength: detection.audio_length
        }
      })
      window.dispatchEvent(transcriptionEvent)
      
      // Also emit chat drawer show event
      const chatEvent = new CustomEvent('show-chat-drawer', {
        detail: { source: 'wake-word', detection }
      })
      window.dispatchEvent(chatEvent)
      
    } catch (err) {
      console.error('Failed to handle wake word detection:', err)
    }
  }

  // Provide feedback when wake word is detected
  async function provideFeedback() {
    try {
      // Visual feedback - emit event for UI components
      const feedbackEvent = new CustomEvent('wake-word-feedback', {
        detail: { type: 'visual', message: 'Wake word detected!' }
      })
      window.dispatchEvent(feedbackEvent)
      
      // Audio feedback (optional)
      if ('speechSynthesis' in window) {
        const utterance = new SpeechSynthesisUtterance('Listening')
        utterance.volume = 0.3
        utterance.rate = 1.2
        utterance.pitch = 1.1
        speechSynthesis.speak(utterance)
      }
    } catch (err) {
      console.warn('Failed to provide wake word feedback:', err)
    }
  }

  // Pause wake word detection temporarily during transcription
  function pauseWakeWordDetection(duration: number) {
    transcriptionPausedUntil = Date.now() + duration
    console.log(`⏸️ Wake word detection paused for ${duration/1000} seconds`)
    
    // Auto-resume after duration
    setTimeout(() => {
      if (Date.now() >= transcriptionPausedUntil) {
        console.log('🔄 Wake word detection resumed')
      }
    }, duration)
  }

  // Resume wake word detection immediately
  function resumeWakeWordDetection() {
    transcriptionPausedUntil = 0
    console.log('🔄 Wake word detection resumed manually')
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
    if (!hasWebSpeechSupport.value) return null
    
    try {
      const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition
      const recognition = new SpeechRecognition()
      
      recognition.continuous = true
      recognition.interimResults = false
      recognition.lang = 'en-US'
      recognition.maxAlternatives = 1
      
      recognition.onstart = () => {
        console.log('Wake word detection: Speech recognition started')
        isListening.value = true
        error.value = null
      }
      
      recognition.onresult = (event) => {
        for (let i = event.resultIndex; i < event.results.length; i++) {
          const result = event.results[i]
          if (result.isFinal) {
            const transcript = result[0].transcript.toLowerCase().trim()
            console.log('Wake word detection: Heard:', transcript)
            
            // Check for wake word "aubrey" with variations
            if (transcript.includes('aubrey') || 
                transcript.includes('aubri') || 
                transcript.includes('obrey') ||
                transcript.includes('awbrey')) {
              
              const detection: WakeWordDetectionInfo = {
                confidence: result[0].confidence || 0.8,
                timestamp: Date.now(),
                audio_length: transcript.length
              }
              
              // Handle the detection
              handleWakeWordDetected(detection)
              
              // Update state
              lastDetection.value = detection
              totalDetections.value += 1
              whisperActivated.value = true
            }
          }
        }
      }
      
      recognition.onerror = (event) => {
        console.error('Wake word detection error:', event.error)
        if (event.error !== 'no-speech' && event.error !== 'aborted') {
          error.value = `Wake word detection error: ${event.error}`
        }
      }
      
      recognition.onend = () => {
        console.log('Wake word detection: Speech recognition ended')
        isListening.value = false
        
        // Auto-restart if still active and not paused
        if (isActive.value && !isTranscriptionPaused.value) {
          setTimeout(() => {
            if (speechRecognition && isActive.value) {
              try {
                speechRecognition.start()
              } catch (err) {
                console.warn('Failed to restart wake word detection:', err)
              }
            }
          }, 1000)
        }
      }
      
      return recognition
    } catch (err) {
      console.error('Failed to setup wake word speech recognition:', err)
      return null
    }
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
    isActive: readonly(isActive),
    isListening: readonly(isListening),
    lastDetection: readonly(lastDetection),
    totalDetections: readonly(totalDetections),
    whisperActivated: readonly(whisperActivated),
    error: readonly(error),
    isStarting: readonly(isStarting),
    isStopping: readonly(isStopping),
    hasWebSpeechSupport: readonly(hasWebSpeechSupport),
    isTranscriptionPaused: readonly(isTranscriptionPaused),
    
    // Computed
    status,
    hasRecentDetection,
    
    // Methods
    startDetection,
    stopDetection,
    toggleDetection,
    updateState,
    resetStats,
    clearError,
    triggerWakeWord,
    pauseWakeWordDetection,
    resumeWakeWordDetection
  }
} 