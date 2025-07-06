<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { 
  MicrophoneIcon, 
  SparklesIcon,
  CommandLineIcon,
  CpuChipIcon,
  ExclamationTriangleIcon
} from '@heroicons/vue/24/outline'
import { useAppStore } from '../../stores/app'
import { useMLEyeTracking } from '../../composables/useMLEyeTracking'
import { useWindowManager } from '../../composables/useWindowManager'
import { useWakeWordDetection } from '../../composables/useWakeWordDetection'
import { getCompatibilityReport } from '../../utils/browserCompat'

const store = useAppStore()
const mlEyeTracking = useMLEyeTracking()
const windowManager = useWindowManager()
const wakeWordDetection = useWakeWordDetection()

// Error handling state
const speechError = ref<string | null>(null)
const wakeWordError = ref<string | null>(null)
const isRetrying = ref(false)

// Browser compatibility
const compatibilityReport = ref(getCompatibilityReport())

// ML Eye tracking with window movement state
const isGazeControlActive = ref(false)

// Enhanced speech transcription with error handling
const toggleSpeechTranscription = async () => {
  try {
    speechError.value = null
    
    // Check browser compatibility first
    if (!compatibilityReport.value.ready) {
      speechError.value = 'Browser not compatible with speech features. ' + compatibilityReport.value.issues.join(', ')
      return
    }
    
    if (store.speechStatus.isRecording) {
      await store.stopSpeechTranscription()
    } else {
      if (!store.speechStatus.isInitialized) {
        await store.initializeSpeechTranscription('base')
      }
      await store.startSpeechTranscription()
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    speechError.value = message
    console.error('Speech transcription error:', error)
  }
}

const getSpeechIconClass = () => {
  if (store.speechStatus.isRecording) return 'text-red-400 animate-pulse'
  if (store.speechStatus.isProcessing) return 'text-yellow-400 animate-pulse'
  if (store.isTranscriptionEnabled) return 'text-green-400'
  return 'text-white/80 group-hover:text-white'
}

// Enhanced wake word detection with error handling
const toggleWakeWordDetection = async () => {
  try {
    wakeWordError.value = null
    
    // Check browser compatibility first
    if (!compatibilityReport.value.ready) {
      wakeWordError.value = 'Browser not compatible with speech features. ' + compatibilityReport.value.issues.join(', ')
      return
    }
    
    await wakeWordDetection.toggleDetection()
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    wakeWordError.value = message
    console.error('Wake word detection error:', error)
  }
}

const getWakeWordIconClass = () => {
  if (wakeWordDetection.hasRecentDetection.value) return 'text-green-400 animate-pulse'
  if (wakeWordDetection.isActive.value) return 'text-blue-400'
  if (wakeWordDetection.error.value) return 'text-red-400'
  return 'text-white/80 group-hover:text-white'
}

const toggleMLEyeTrackingWithMovement = async () => {
  if (mlEyeTracking.isActive.value) {
    // Stop both ML tracking and window movement
    await mlEyeTracking.stopTracking()
    windowManager.disableGazeControl()
    isGazeControlActive.value = false
    console.log('🛑 ML Eye Tracking and Window Movement stopped')
  } else {
    // Start ML tracking
    await mlEyeTracking.startTracking()
    
    // Wait a moment for initialization
    setTimeout(() => {
      if (mlEyeTracking.isActive.value) {
        // Start window movement with ML gaze data
        startMLGazeWindowMovement()
        isGazeControlActive.value = true
        console.log('🚀 ML Eye Tracking and Window Movement started')
      }
    }, 1000)
  }
}

// Function to connect ML gaze data to window movement
function startMLGazeWindowMovement() {
  // Enable the window manager for gaze control
  windowManager.enableGazeControl()
  
  // Create interval to read ML gaze data and move window
  const updateInterval = setInterval(async () => {
    if (!mlEyeTracking.isActive.value) {
      clearInterval(updateInterval)
      return
    }
    
    const gazeData = mlEyeTracking.currentGaze.value
    if (gazeData && mlEyeTracking.isHighConfidence.value) {
      // Convert gaze screen coordinates to normalized coordinates (-1 to 1)
      const virtualDesktopSize = windowManager.state.value.screenSize
      const screenCenterX = virtualDesktopSize.width / 2
      const screenCenterY = virtualDesktopSize.height / 2
      
      const normalizedGaze = {
        x: (gazeData.x - screenCenterX) / screenCenterX,
        y: (gazeData.y - screenCenterY) / screenCenterY
      }
      
      // Process gaze input through the window manager
      await windowManager.processGazeInput(normalizedGaze)
    }
  }, 33) // 30 FPS window movement
}

// Keyboard shortcuts
const handleKeydown = async (event: KeyboardEvent) => {
  // Ctrl+Shift+E = Toggle ML Eye Tracking
  if (event.ctrlKey && event.shiftKey && event.key === 'E') {
    event.preventDefault()
    await toggleMLEyeTrackingWithMovement()
    console.log('⌨️ Keyboard shortcut: ML Eye Tracking toggled')
  }
  
  // Ctrl+Shift+S = Stop all tracking (emergency stop)
  if (event.ctrlKey && event.shiftKey && event.key === 'S') {
    event.preventDefault()
    await mlEyeTracking.stopTracking()
    windowManager.disableGazeControl()
    isGazeControlActive.value = false
    console.log('🚨 Emergency stop: All tracking stopped')
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
  
  // Show keyboard shortcuts in console
  console.log('⌨️ ML Eye Tracking Keyboard Shortcuts:')
  console.log('   Ctrl+Shift+E = Start/Stop ML Eye Tracking + Window Movement')
  console.log('   Ctrl+Shift+S = Emergency Stop (stop all tracking)')
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <div class="control-panel-glass-bar" data-tauri-drag-region>
    <div class="control-buttons-row">
      <!-- AI Assistant Button -->
      <button 
        class="control-btn group"
        :class="{ 'active': false }"
        title="AI Assistant"
      >
        <SparklesIcon class="w-4 h-4 text-white/70 group-hover:text-white transition-all" />
      </button>
      
      <!-- Speech Transcription Button -->
      <button 
        @click="toggleSpeechTranscription"
        class="control-btn group"
        :class="{ 
          'active-pulse': store.speechStatus.isRecording,
          'active-warning': store.speechStatus.isProcessing,
          'active': store.isTranscriptionEnabled && !store.speechStatus.isRecording
        }"
        :disabled="store.speechStatus.isProcessing"
        title="Speech Transcription"
      >
        <MicrophoneIcon class="w-4 h-4 transition-all" 
          :class="getSpeechIconClass()" />
      </button>

      <!-- Wake Word Detection Button -->
      <button 
        @click="toggleWakeWordDetection"
        class="control-btn group"
        :class="{ 
          'active-pulse': wakeWordDetection.hasRecentDetection.value,
          'active': wakeWordDetection.isActive.value && !wakeWordDetection.hasRecentDetection.value,
          'active-error': wakeWordDetection.error.value,
          'active-warning': wakeWordDetection.isStarting.value || wakeWordDetection.isStopping.value
        }"
        :disabled="wakeWordDetection.isStarting.value || wakeWordDetection.isStopping.value"
        title="Wake Word Detection"
      >
        <ExclamationTriangleIcon class="w-4 h-4 transition-all" 
          :class="getWakeWordIconClass()" />
      </button>
      
      <!-- ML Eye Tracking + Window Movement Button -->
      <button 
        @click="toggleMLEyeTrackingWithMovement"
        class="control-btn group"
        :class="{ 
          'active': mlEyeTracking.isActive.value && mlEyeTracking.isCalibrated.value && isGazeControlActive,
          'active-warning': mlEyeTracking.isActive.value && (!mlEyeTracking.isCalibrated.value || !isGazeControlActive)
        }"
        :disabled="mlEyeTracking.isLoading.value"
        title="ML Eye Tracking + Window Movement"
      >
        <CpuChipIcon class="w-4 h-4 transition-all"
          :class="mlEyeTracking.isActive.value ? 'text-white' : 'text-white/70 group-hover:text-white'" />
      </button>

      <!-- Command Mode Button -->
      <button 
        class="control-btn group"
        title="Command Mode"
      >
        <CommandLineIcon class="w-4 h-4 text-white/70 group-hover:text-white transition-all" />
      </button>
    </div>
  </div>
</template>

<style scoped>
/* Curved Glass Control Panel Bar */
.control-panel-glass-bar {
  @apply rounded-full overflow-hidden;
  height: 44px;
  min-width: 240px;
  cursor: move;
  user-select: none;
  
  /* Premium curved glass effect */
  background: linear-gradient(135deg, 
    rgba(255, 255, 255, 0.25) 0%,
    rgba(255, 255, 255, 0.15) 25%,
    rgba(255, 255, 255, 0.10) 50%,
    rgba(255, 255, 255, 0.15) 75%,
    rgba(255, 255, 255, 0.25) 100%
  );
  backdrop-filter: blur(40px) saturate(180%) brightness(1.1);
  border: 1px solid rgba(255, 255, 255, 0.3);
  box-shadow: 
    0 8px 32px rgba(0, 0, 0, 0.25),
    0 2px 8px rgba(0, 0, 0, 0.15),
    inset 0 1px 0 rgba(255, 255, 255, 0.4),
    inset 0 -1px 0 rgba(0, 0, 0, 0.1);
  
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.control-panel-glass-bar:hover {
  background: linear-gradient(135deg, 
    rgba(255, 255, 255, 0.35) 0%,
    rgba(255, 255, 255, 0.25) 25%,
    rgba(255, 255, 255, 0.20) 50%,
    rgba(255, 255, 255, 0.25) 75%,
    rgba(255, 255, 255, 0.35) 100%
  );
  border-color: rgba(255, 255, 255, 0.4);
  box-shadow: 
    0 12px 40px rgba(0, 0, 0, 0.3),
    0 4px 12px rgba(0, 0, 0, 0.2),
    inset 0 1px 0 rgba(255, 255, 255, 0.5),
    inset 0 -1px 0 rgba(0, 0, 0, 0.1);
  transform: translateY(-1px);
}

.control-buttons-row {
  @apply flex items-center justify-center gap-2 px-4 py-2;
  height: 100%;
}

.control-btn {
  @apply rounded-full transition-all duration-200 flex items-center justify-center;
  width: 32px;
  height: 32px;
  border: none;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(8px);
  cursor: pointer;
}

.control-btn:hover {
  background: rgba(255, 255, 255, 0.2);
  transform: scale(1.1);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

.control-btn.active {
  background: rgba(74, 144, 226, 0.8);
  box-shadow: 0 0 16px rgba(74, 144, 226, 0.4);
}

.control-btn.active-pulse {
  background: rgba(239, 68, 68, 0.8);
  box-shadow: 0 0 16px rgba(239, 68, 68, 0.4);
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

.control-btn.active-warning {
  background: rgba(245, 158, 11, 0.8);
  box-shadow: 0 0 16px rgba(245, 158, 11, 0.4);
}

.control-btn.active-error {
  background: rgba(239, 68, 68, 0.8);
  box-shadow: 0 0 16px rgba(239, 68, 68, 0.4);
}

.control-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.control-btn:disabled:hover {
  transform: none;
  box-shadow: none;
}

/* Pulse animation for active states */
@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.7;
  }
}

/* Floating animation for the entire bar */
.control-panel-glass-bar {
  animation: float 6s ease-in-out infinite;
}

@keyframes float {
  0%, 100% {
    transform: translateY(0px);
  }
  50% {
    transform: translateY(-2px);
  }
}

.control-panel-glass-bar:hover {
  animation: none;
}
</style> 