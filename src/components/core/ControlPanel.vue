<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { 
  MicrophoneIcon, 
  ChatBubbleLeftRightIcon,
  SparklesIcon,
  CommandLineIcon,
  CpuChipIcon
} from '@heroicons/vue/24/outline'
import { useAppStore } from '../../stores/app'
import { useMLEyeTracking } from '../../composables/useMLEyeTracking'
import { useWindowManager } from '../../composables/useWindowManager'
import TransparencyControls from './TransparencyControls.vue'

const store = useAppStore()
const mlEyeTracking = useMLEyeTracking()
const windowManager = useWindowManager()

// Transparency controls state
const showTransparencyControls = ref(false)

// ML Eye tracking with window movement state
const isGazeControlActive = ref(false)

const toggleTransparencyControls = () => {
  showTransparencyControls.value = !showTransparencyControls.value
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
      // Convert gaze screen coordinates to normalized coordinates (0-1)
      const normalizedGaze = {
        x: gazeData.x / mlEyeTracking.config.value.screen_width,
        y: gazeData.y / mlEyeTracking.config.value.screen_height
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
        
        <!-- Microphone Button -->
        <button 
          @click="store.toggleMic"
          class="btn btn-circle btn-sm glass-btn-compact group tooltip flex items-center justify-center"
          :class="{ 'btn-primary': store.micEnabled, 'glass-btn-compact': !store.micEnabled }"
          data-tip="Toggle Microphone"
        >
          <MicrophoneIcon class="w-3.5 h-3.5 transition-colors" 
            :class="store.micEnabled ? 'text-white' : 'text-white/80 group-hover:text-white'" />
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
        
        <!-- Chat Button -->
        <button 
          @click="store.toggleChat"
          class="btn btn-circle btn-sm glass-btn-compact group tooltip flex items-center justify-center"
          :class="{ 'btn-accent': store.chatOpen, 'glass-btn-compact': !store.chatOpen }"
          data-tip="Toggle Chat"
        >
          <ChatBubbleLeftRightIcon class="w-3.5 h-3.5 transition-colors"
            :class="store.chatOpen ? 'text-white' : 'text-white/80 group-hover:text-white'" />
        </button>
      </div>
    </div>
    
    <!-- Status Indicator -->
    <div v-if="mlEyeTracking.isActive.value" class="mt-2 text-center">
      <div class="glass-panel-compact px-3 py-1 inline-block">
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