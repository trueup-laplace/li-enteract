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
import TransparencyControls from './TransparencyControls.vue'

const store = useAppStore()
const mlEyeTracking = useMLEyeTracking()
const windowManager = useWindowManager()
const wakeWordDetection = useWakeWordDetection()

// Error handling state
const speechError = ref<string | null>(null)
const wakeWordError = ref<string | null>(null)
const generalError = ref<string | null>(null)
const isRetrying = ref(false)

// Browser compatibility
const compatibilityReport = ref(getCompatibilityReport())
const showCompatibilityWarning = computed(() => 
  !compatibilityReport.value.ready && compatibilityReport.value.issues.length > 0
)
const filteredIssues = computed(() => 
  compatibilityReport.value.issues.filter((issue): issue is string => Boolean(issue))
)

// Transparency controls state
const showTransparencyControls = ref(false)

// ML Eye tracking with window movement state
const isGazeControlActive = ref(false)

// Calibration state removed - multimonitor calibration functionality removed

// Retry and error handling functions
const retrySpeechSetup = async () => {
  if (isRetrying.value) return
  
  try {
    isRetrying.value = true
    speechError.value = null
    generalError.value = null
    
    console.log('🔄 Retrying speech transcription setup...')
    
    // Reinitialize speech transcription
    await store.initializeSpeechTranscription('base')
    
    console.log('✅ Speech transcription retry successful')
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    speechError.value = message
    console.error('❌ Speech transcription retry failed:', error)
  } finally {
    isRetrying.value = false
  }
}

const retryWakeWordSetup = async () => {
  if (isRetrying.value) return
  
  try {
    isRetrying.value = true
    wakeWordError.value = null
    generalError.value = null
    
    console.log('🔄 Retrying wake word detection setup...')
    
    // Clear any existing error
    wakeWordDetection.clearError()
    
    // Restart wake word detection
    if (wakeWordDetection.isActive.value) {
      await wakeWordDetection.stopDetection()
    }
    await wakeWordDetection.startDetection()
    
    console.log('✅ Wake word detection retry successful')
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    wakeWordError.value = message
    console.error('❌ Wake word detection retry failed:', error)
  } finally {
    isRetrying.value = false
  }
}

const clearAllErrors = () => {
  speechError.value = null
  wakeWordError.value = null
  generalError.value = null
  wakeWordDetection.clearError()
}

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

const getSpeechTooltip = () => {
  if (store.speechStatus.isProcessing) return 'Processing with Whisper...'
  if (store.speechStatus.isRecording) return 'Stop Recording (Click to stop)'
  if (store.speechStatus.error) return `Error: ${store.speechStatus.error}`
  if (!store.speechStatus.isInitialized) return 'Initialize Speech Transcription'
  return 'Start Speech Recording'
}

const getSpeechIconClass = () => {
  if (store.speechStatus.isRecording) return 'text-white'
  if (store.speechStatus.isProcessing) return 'text-white'
  if (store.isTranscriptionEnabled) return 'text-white'
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

const getWakeWordTooltip = () => {
  if (wakeWordDetection.isStarting.value) return 'Starting wake word detection...'
  if (wakeWordDetection.isStopping.value) return 'Stopping wake word detection...'
  if (wakeWordDetection.error.value) return `Error: ${wakeWordDetection.error.value}`
  if (wakeWordDetection.isActive.value) return 'Stop Wake Word Detection (Say "Aubrey")'
  return 'Start Wake Word Detection (Say "Aubrey")'
}

const getWakeWordIconClass = () => {
  if (wakeWordDetection.hasRecentDetection.value) return 'text-green-400 animate-pulse'
  if (wakeWordDetection.isActive.value) return 'text-white'
  if (wakeWordDetection.error.value) return 'text-red-400'
  return 'text-white/80 group-hover:text-white'
}

const toggleTransparencyControls = () => {
  showTransparencyControls.value = !showTransparencyControls.value
}

// Calibration functions removed - multimonitor calibration functionality removed

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
      // Python backend sends absolute screen coordinates using virtual desktop, normalize to (-1, 1)
      
      // Get virtual desktop size from window manager (matches Python backend)
      const virtualDesktopSize = windowManager.state.value.screenSize
      const screenCenterX = virtualDesktopSize.width / 2
      const screenCenterY = virtualDesktopSize.height / 2
      
      const normalizedGaze = {
        x: (gazeData.x - screenCenterX) / screenCenterX,  // Convert to -1 to 1 range
        y: (gazeData.y - screenCenterY) / screenCenterY   // Convert to -1 to 1 range
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
  
  // Ctrl+Shift+C = Calibrate
  if (event.ctrlKey && event.shiftKey && event.key === 'C') {
    event.preventDefault()
    if (mlEyeTracking.isActive.value) {
      await mlEyeTracking.calibrate()
      console.log('🎯 Calibration triggered via keyboard')
    }
  }
}

// Click outside to close controls
const closeControls = (event: Event) => {
  const target = event.target as HTMLElement
  if (!target.closest('.transparency-controls') && 
      !target.closest('.command-btn')) {
    showTransparencyControls.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', closeControls)
  document.addEventListener('keydown', handleKeydown)
  
  // Show keyboard shortcuts in console
  console.log('⌨️ ML Eye Tracking Keyboard Shortcuts:')
  console.log('   Ctrl+Shift+E = Start/Stop ML Eye Tracking + Window Movement')
  console.log('   Ctrl+Shift+S = Emergency Stop (stop all tracking)')
  console.log('   Ctrl+Shift+C = Calibrate ML tracking')
})

onUnmounted(() => {
  document.removeEventListener('click', closeControls)
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <div class="p-3">
    <div class="flex justify-center">
      <div class="glass-panel-compact flex items-center justify-center gap-2 px-4 py-2">
        <!-- AI Assistant Button -->
        <button 
          class="btn btn-circle btn-sm glass-btn-compact group flex items-center justify-center"
        >
          <SparklesIcon class="w-3.5 h-3.5 text-white/80 group-hover:text-white transition-colors" />
        </button>
        
        <!-- Speech Transcription Button -->
        <button 
          @click="toggleSpeechTranscription"
          class="btn btn-circle btn-sm glass-btn-compact group tooltip flex items-center justify-center"
          :class="{ 
            'btn-error animate-pulse': store.speechStatus.isRecording,
            'btn-warning': store.speechStatus.isProcessing,
            'btn-success': store.isTranscriptionEnabled && !store.speechStatus.isRecording,
            'glass-btn-compact': !store.isTranscriptionEnabled 
          }"
          :data-tip="getSpeechTooltip()"
          :disabled="store.speechStatus.isProcessing"
        >
          <MicrophoneIcon class="w-3.5 h-3.5 transition-colors" 
            :class="getSpeechIconClass()" />
        </button>

        <!-- Wake Word Detection Button -->
        <button 
          @click="toggleWakeWordDetection"
          class="btn btn-circle btn-sm glass-btn-compact group tooltip flex items-center justify-center"
          :class="{ 
            'btn-success animate-pulse': wakeWordDetection.hasRecentDetection.value,
            'btn-success': wakeWordDetection.isActive.value && !wakeWordDetection.hasRecentDetection.value,
            'btn-error': wakeWordDetection.error.value,
            'btn-warning': wakeWordDetection.isStarting.value || wakeWordDetection.isStopping.value,
            'glass-btn-compact': !wakeWordDetection.isActive.value && !wakeWordDetection.error.value
          }"
          :data-tip="getWakeWordTooltip()"
          :disabled="wakeWordDetection.isStarting.value || wakeWordDetection.isStopping.value"
        >
          <ExclamationTriangleIcon class="w-3.5 h-3.5 transition-colors" 
            :class="getWakeWordIconClass()" />
        </button>
        
        <!-- ML Eye Tracking + Window Movement Button -->
        <button 
          @click="toggleMLEyeTrackingWithMovement"
          class="btn btn-circle btn-sm glass-btn-compact group tooltip flex items-center justify-center"
          :class="{ 
            'btn-success': mlEyeTracking.isActive.value && mlEyeTracking.isCalibrated.value && isGazeControlActive,
            'btn-warning': mlEyeTracking.isActive.value && (!mlEyeTracking.isCalibrated.value || !isGazeControlActive),
            'glass-btn-compact': !mlEyeTracking.isActive.value 
          }"
          :data-tip="mlEyeTracking.isActive.value ? 
            'Stop ML Gaze Control (Ctrl+Shift+E)' : 
            'Start ML Gaze Control (Ctrl+Shift+E)'"
          :disabled="mlEyeTracking.isLoading.value"
        >
          <CpuChipIcon class="w-3.5 h-3.5 transition-colors"
            :class="mlEyeTracking.isActive.value ? 'text-white' : 'text-white/80 group-hover:text-white'" />
        </button>

        <!-- Calibration Button removed - multimonitor calibration functionality removed -->

        <!-- Command Mode Button (Transparency Controls) -->
        <button 
          @click="toggleTransparencyControls"
          class="btn btn-circle btn-sm glass-btn-compact group tooltip flex items-center justify-center command-btn"
          :class="{ 'btn-accent': showTransparencyControls, 'glass-btn-compact': !showTransparencyControls }"
          data-tip="Transparency Controls"
        >
          <CommandLineIcon class="w-3.5 h-3.5 transition-colors"
            :class="showTransparencyControls ? 'text-white' : 'text-white/80 group-hover:text-white'" />
        </button>
        
        <!-- Chat Button removed - chat is now part of home screen -->
      </div>
    </div>
    
    <!-- Status Indicators -->
    <div class="mt-2 text-center space-y-1">
      <!-- Browser Compatibility Warning -->
      <div v-if="showCompatibilityWarning" class="glass-panel-compact px-3 py-2 bg-red-500/20 border border-red-500/30">
        <div class="flex items-center justify-between text-xs">
          <div class="flex items-center gap-2">
            <ExclamationTriangleIcon class="w-4 h-4 text-red-400" />
            <span class="text-red-300">Browser Compatibility Issues</span>
          </div>
          <button @click="compatibilityReport = getCompatibilityReport()" 
                  class="text-red-300 hover:text-red-200 underline">
            Recheck
          </button>
        </div>
        <div class="mt-1">
                                <div v-for="issue in filteredIssues" :key="issue" class="text-xs text-red-200">
             • {{ issue }}
           </div>
          <div class="text-xs text-red-200 mt-1">
            Browser: {{ compatibilityReport.browser.name }} {{ compatibilityReport.browser.version }}
          </div>
        </div>
      </div>

      <!-- Speech Error Display -->
      <div v-if="speechError || store.speechStatus.error" class="glass-panel-compact px-3 py-2 bg-red-500/20 border border-red-500/30">
        <div class="flex items-center justify-between text-xs">
          <div class="flex items-center gap-2">
            <ExclamationTriangleIcon class="w-4 h-4 text-red-400" />
            <span class="text-red-300">Speech Error</span>
          </div>
          <div class="flex gap-2">
            <button @click="retrySpeechSetup" 
                    :disabled="isRetrying"
                    class="text-blue-300 hover:text-blue-200 underline disabled:opacity-50">
              {{ isRetrying ? 'Retrying...' : 'Retry' }}
            </button>
            <button @click="speechError = null" class="text-gray-300 hover:text-gray-200">✕</button>
          </div>
        </div>
        <div class="text-xs text-red-200 mt-1">
          {{ speechError || store.speechStatus.error }}
        </div>
      </div>

      <!-- Wake Word Error Display -->
      <div v-if="wakeWordError || wakeWordDetection.error.value" class="glass-panel-compact px-3 py-2 bg-red-500/20 border border-red-500/30">
        <div class="flex items-center justify-between text-xs">
          <div class="flex items-center gap-2">
            <ExclamationTriangleIcon class="w-4 h-4 text-red-400" />
            <span class="text-red-300">Wake Word Error</span>
          </div>
          <div class="flex gap-2">
            <button @click="retryWakeWordSetup" 
                    :disabled="isRetrying"
                    class="text-blue-300 hover:text-blue-200 underline disabled:opacity-50">
              {{ isRetrying ? 'Retrying...' : 'Retry' }}
            </button>
            <button @click="wakeWordError = null; wakeWordDetection.clearError()" class="text-gray-300 hover:text-gray-200">✕</button>
          </div>
        </div>
        <div class="text-xs text-red-200 mt-1">
          {{ wakeWordError || wakeWordDetection.error.value }}
        </div>
      </div>

      <!-- General Error Display -->
      <div v-if="generalError" class="glass-panel-compact px-3 py-2 bg-red-500/20 border border-red-500/30">
        <div class="flex items-center justify-between text-xs">
          <div class="flex items-center gap-2">
            <ExclamationTriangleIcon class="w-4 h-4 text-red-400" />
            <span class="text-red-300">System Error</span>
          </div>
          <button @click="clearAllErrors" class="text-gray-300 hover:text-gray-200">✕</button>
        </div>
        <div class="text-xs text-red-200 mt-1">
          {{ generalError }}
        </div>
      </div>

      <!-- Speech Status Indicator -->
      <div v-if="store.speechStatus.isInitialized || wakeWordDetection.isActive.value" class="glass-panel-compact px-3 py-1 inline-block">
        <span class="text-xs text-white/80">
          Speech Status: 
          <span :class="{
            'text-green-400': store.speechStatus.isRecording || wakeWordDetection.isListening.value,
            'text-blue-400': store.speechStatus.isProcessing,
            'text-yellow-400': store.speechStatus.isInitialized && !store.speechStatus.isRecording
          }">
            {{ store.speechStatus.isRecording ? 'Recording' :
               store.speechStatus.isProcessing ? 'Processing' :
               wakeWordDetection.isListening.value ? 'Listening for "Aubrey"' :
               store.speechStatus.isInitialized ? 'Ready' : 'Inactive' }}
          </span>
          <span v-if="wakeWordDetection.totalDetections.value > 0" class="text-green-400">
            • Detections: {{ wakeWordDetection.totalDetections.value }}
          </span>
        </span>
      </div>

      <!-- ML Eye Tracking Status -->
      <div v-if="mlEyeTracking.isActive.value" class="glass-panel-compact px-3 py-1 inline-block">
        <span class="text-xs text-white/80">
          ML Gaze Control: 
          <span :class="{
            'text-green-400': mlEyeTracking.isCalibrated.value && isGazeControlActive,
            'text-yellow-400': !mlEyeTracking.isCalibrated.value || !isGazeControlActive,
            'text-red-400': !mlEyeTracking.isActive.value
          }">
            {{ mlEyeTracking.isCalibrated.value && isGazeControlActive ? 'Active' : 
               mlEyeTracking.isActive.value ? 'Tracking' : 'Inactive' }}
          </span>
          • FPS: {{ mlEyeTracking.fps.value }}
          • Conf: {{ Math.round(mlEyeTracking.confidence.value * 100) }}%
        </span>
      </div>
    </div>
    
    <!-- Transparency Controls Panel -->
    <Transition name="transparency-panel">
      <div v-if="showTransparencyControls" class="transparency-panel-container">
        <TransparencyControls />
      </div>
    </Transition>
    
    <!-- Calibration Modal removed - multimonitor calibration functionality removed -->
  </div>
</template>

<style scoped>
.glass-panel-compact {
  @apply backdrop-blur-xl border border-white/15 rounded-2xl shadow-xl;
  background: linear-gradient(to right, 
    rgba(255, 255, 255, 0.05) 0%, 
    rgba(255, 255, 255, 0.08) 50%, 
    rgba(255, 255, 255, 0.05) 100%
  );
  background-image: 
    radial-gradient(circle at 30% 50%, rgba(120, 119, 198, 0.08) 0%, transparent 50%),
    radial-gradient(circle at 70% 50%, rgba(255, 119, 198, 0.08) 0%, transparent 50%);
}

.glass-btn-compact {
  @apply bg-white/5 backdrop-blur-md border border-white/15 hover:border-white/30 hover:bg-white/10 transition-all duration-200 hover:scale-105 hover:shadow-lg;
}

.btn-sm {
  @apply w-8 h-8;
  display: flex;
  align-items: center;
  justify-content: center;
}

.glass-btn-compact:hover {
  transform: translateY(-1px) scale(1.05);
  box-shadow: 0 8px 25px rgba(0, 0, 0, 0.2);
}

/* Ensure icons are perfectly centered */
.btn-circle {
  display: flex;
  align-items: center;
  justify-content: center;
}

.btn-circle svg {
  flex-shrink: 0;
}

/* Calibration modal styles removed - multimonitor calibration functionality removed */

/* Transparency panel positioning */
.transparency-panel-container {
  @apply absolute top-full left-1/2 transform -translate-x-1/2 mt-4;
  @apply z-50;
  position: relative;
}

/* Panel transitions */
.transparency-panel-enter-active,
.transparency-panel-leave-active {
  transition: all 0.3s ease;
}

.transparency-panel-enter-from,
.transparency-panel-leave-to {
  opacity: 0;
  transform: translateY(-10px) translateX(-50%);
}

/* Status indicator animation */
.glass-panel-compact {
  transition: all 0.3s ease;
}
</style> 