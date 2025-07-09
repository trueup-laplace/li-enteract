<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { 
  MicrophoneIcon, 
  Cog6ToothIcon,
  CommandLineIcon,
  CpuChipIcon,
  ExclamationTriangleIcon,
  AdjustmentsHorizontalIcon,
  XMarkIcon
} from '@heroicons/vue/24/outline'
import { useAppStore } from '../../stores/app'
import { useMLEyeTracking } from '../../composables/useMLEyeTracking'
import { useWindowManager } from '../../composables/useWindowManager'
import { useWakeWordDetection } from '../../composables/useWakeWordDetection'
import { useWindowResizing } from '../../composables/useWindowResizing'
import { useAIModels } from '../../composables/useAIModels'
import { getCompatibilityReport } from '../../utils/browserCompat'
import TransparencyControls from './TransparencyControls.vue'
import ChatWindow from './ChatWindow.vue'
import AIModelsPanel from './AIModelsPanel.vue'

const store = useAppStore()
const mlEyeTracking = useMLEyeTracking()
const windowManager = useWindowManager()
const wakeWordDetection = useWakeWordDetection()

// Window resizing
const { resizeWindow } = useWindowResizing()

// AI Models management
const { selectedModel } = useAIModels()

// Dragging state
const isDragging = ref(false)
const dragStartTime = ref(0)

// Window state
const showChatWindow = ref(false)
const showTransparencyControls = ref(false)
const showAIModelsWindow = ref(false)

// Error handling state
const speechError = ref<string | null>(null)
const wakeWordError = ref<string | null>(null)

// Browser compatibility
const compatibilityReport = ref(getCompatibilityReport())

// ML Eye tracking with window movement state
const isGazeControlActive = ref(false)

// Watch for window state changes to resize window
watch(showChatWindow, async (newValue) => {
  await resizeWindow(newValue, showTransparencyControls.value, showAIModelsWindow.value)
})

watch(showTransparencyControls, async (newValue) => {
  console.log(`🔧 TRANSPARENCY WATCH: newValue=${newValue}, showChat=${showChatWindow.value}, showAI=${showAIModelsWindow.value}`)
  // Temporarily disabled to debug window disappearing issue
  await resizeWindow(showChatWindow.value, newValue, showAIModelsWindow.value)
  console.log('🔧 TRANSPARENCY WATCH: Skipping resize to debug issue')
})

watch(showAIModelsWindow, async (newValue) => {
  await resizeWindow(showChatWindow.value, showTransparencyControls.value, newValue)
})

// Drag event handlers
const handleDragStart = () => {
  isDragging.value = true
  dragStartTime.value = Date.now()
  console.log('🎯 Control panel drag started')
}

const handleDragEnd = () => {
  const dragDuration = Date.now() - dragStartTime.value
  isDragging.value = false
  console.log(`🎯 Control panel drag ended (${dragDuration}ms)`)
}

// Panel toggle functions
const toggleTransparencyControls = (event: Event) => {
  event.stopPropagation()
  
  if (showChatWindow.value) {
    showChatWindow.value = false
  }
  if (showAIModelsWindow.value) {
    showAIModelsWindow.value = false
  }
  
  showTransparencyControls.value = !showTransparencyControls.value
  console.log(`🔍 Transparency controls ${showTransparencyControls.value ? 'opened' : 'closed'}`)
}

const closeTransparencyControls = () => {
  showTransparencyControls.value = false
  console.log('🔍 Transparency controls closed')
}

const toggleAIModelsWindow = async (event: Event) => {
  event.stopPropagation()
  
  if (showChatWindow.value) {
    showChatWindow.value = false
  }
  if (showTransparencyControls.value) {
    showTransparencyControls.value = false
  }
  
  showAIModelsWindow.value = !showAIModelsWindow.value
  console.log(`🤖 AI Models window ${showAIModelsWindow.value ? 'opened' : 'closed'}`)
}

const closeAIModelsWindow = () => {
  showAIModelsWindow.value = false
  console.log('🤖 AI Models window closed')
}

const toggleChatWindow = async (event: Event) => {
  event.stopPropagation()
  
  if (showTransparencyControls.value) {
    showTransparencyControls.value = false
  }
  if (showAIModelsWindow.value) {
    showAIModelsWindow.value = false
  }
  
  showChatWindow.value = !showChatWindow.value
  console.log(`💬 Chat window ${showChatWindow.value ? 'opened' : 'closed'}`)
}

const closeChatWindow = async () => {
  showChatWindow.value = false
  console.log('💬 Chat window closed')
}

// Enhanced speech transcription with error handling
const toggleSpeechTranscription = async (event: Event) => {
  event.stopPropagation()
  
  try {
    speechError.value = null
    
    if (!compatibilityReport.value.ready) {
      speechError.value = 'Browser not compatible with speech features. ' + compatibilityReport.value.issues.join(', ')
      return
    }
    
    if (store.speechStatus.isRecording) {
      await store.stopSpeechTranscription()
    } else {
      if (!store.speechStatus.isInitialized) {
        await store.initializeSpeechTranscription('small')
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
const toggleWakeWordDetection = async (event: Event) => {
  event.stopPropagation()
  
  try {
    wakeWordError.value = null
    
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

const toggleMLEyeTrackingWithMovement = async (event: Event) => {
  event.stopPropagation()
  
  if (mlEyeTracking.isActive.value) {
    await mlEyeTracking.stopTracking()
    windowManager.disableGazeControl()
    isGazeControlActive.value = false
    console.log('🛑 ML Eye Tracking and Window Movement stopped')
  } else {
    await mlEyeTracking.startTracking()
    
    setTimeout(() => {
      if (mlEyeTracking.isActive.value) {
        startMLGazeWindowMovement()
        isGazeControlActive.value = true
        console.log('🚀 ML Eye Tracking and Window Movement started')
      }
    }, 1000)
  }
}

// Function to connect ML gaze data to window movement
function startMLGazeWindowMovement() {
  windowManager.enableGazeControl()
  
  const updateInterval = setInterval(async () => {
    if (!mlEyeTracking.isActive.value) {
      clearInterval(updateInterval)
      return
    }
    
    const gazeData = mlEyeTracking.currentGaze.value
    if (gazeData && mlEyeTracking.isHighConfidence.value) {
      const virtualDesktopSize = windowManager.state.value.screenSize
      const screenCenterX = virtualDesktopSize.width / 2
      const screenCenterY = virtualDesktopSize.height / 2
      
      const normalizedGaze = {
        x: (gazeData.x - screenCenterX) / screenCenterX,
        y: (gazeData.y - screenCenterY) / screenCenterY
      }
      
      await windowManager.processGazeInput(normalizedGaze)
    }
  }, 33)
}

// Keyboard shortcuts
const handleKeydown = async (event: KeyboardEvent) => {
  if (event.ctrlKey && event.shiftKey && event.key === 'E') {
    event.preventDefault()
    await toggleMLEyeTrackingWithMovement(event)
    console.log('⌨️ Keyboard shortcut: ML Eye Tracking toggled')
  }
  
  if (event.ctrlKey && event.shiftKey && event.key === 'S') {
    event.preventDefault()
    await mlEyeTracking.stopTracking()
    windowManager.disableGazeControl()
    isGazeControlActive.value = false
    console.log('🚨 Emergency stop: All tracking stopped')
  }
  
  if (event.ctrlKey && event.shiftKey && event.key === 'C') {
    event.preventDefault()
    await toggleChatWindow(event)
    console.log('💬 Keyboard shortcut: Chat window toggled')
  }
  
  if (event.ctrlKey && event.shiftKey && event.key === 'T') {
    event.preventDefault()
    toggleTransparencyControls(event)
    console.log('🔍 Keyboard shortcut: Transparency controls toggled')
  }
  
  if (event.ctrlKey && event.shiftKey && event.key === 'A') {
    event.preventDefault()
    await toggleAIModelsWindow(event)
    console.log('🤖 Keyboard shortcut: AI Models window toggled')
  }
  
  if (event.key === 'Escape') {
    event.preventDefault()
    if (showChatWindow.value) {
      await closeChatWindow()
    }
    if (showTransparencyControls.value) {
      closeTransparencyControls()
    }
    if (showAIModelsWindow.value) {
      closeAIModelsWindow()
    }
  }
}

// Click outside to close panels
const handleClickOutside = (event: Event) => {
  const target = event.target as HTMLElement
  const chatWindow = document.querySelector('.chat-window')
  const transparencyPanel = document.querySelector('.transparency-controls-panel')
  const aiModelsPanel = document.querySelector('.ai-models-panel')
  const controlPanel = document.querySelector('.control-panel-glass-bar')
  
  if (chatWindow && controlPanel && showChatWindow.value &&
      !chatWindow.contains(target) && 
      !controlPanel.contains(target)) {
    closeChatWindow()
  }
  
  if (transparencyPanel && controlPanel && showTransparencyControls.value &&
      !transparencyPanel.contains(target) && 
      !controlPanel.contains(target)) {
    closeTransparencyControls()
  }
  
  if (aiModelsPanel && controlPanel && showAIModelsWindow.value &&
      !aiModelsPanel.contains(target) && 
      !controlPanel.contains(target)) {
    closeAIModelsWindow()
  }
}

// Event handlers for wake word events
const handleWakeWordOpenChat = async (event: Event) => {
  console.log('🔔 Wake word detected: Opening Chat Window')
  const mockEvent = event || new Event('wake-word')
  await toggleChatWindow(mockEvent)
}

const handleWakeWordTriggerMic = async (event: Event) => {
  console.log('🔔 Wake word detected: Triggering Mic')
  const mockEvent = event || new Event('wake-word')
  await toggleSpeechTranscription(mockEvent)
}

const handleSpeechOpenChat = async (event: Event) => {
  console.log('🔔 Speech detected: Opening Chat Window')
  const mockEvent = event || new Event('speech')
  await toggleChatWindow(mockEvent)
}

onMounted(async () => {
  document.addEventListener('keydown', handleKeydown)
  document.addEventListener('click', handleClickOutside)
  
  const controlPanel = document.querySelector('.control-panel-glass-bar') as HTMLElement
  if (controlPanel) {
    controlPanel.addEventListener('mousedown', handleDragStart)
    document.addEventListener('mouseup', handleDragEnd)
  }
  
  // Add wake word event listeners
  window.addEventListener('wake-word-open-chat-window', handleWakeWordOpenChat)
  window.addEventListener('wake-word-trigger-mic', handleWakeWordTriggerMic)
  window.addEventListener('speech-open-chat-window', handleSpeechOpenChat)
  
  await store.initializeSpeechTranscription('small')
  
  // Auto-start wake word detection for "Aubrey"
  try {
    await wakeWordDetection.startDetection()
    console.log('🎤 Wake word detection started - listening for "Aubrey"')
  } catch (error) {
    console.warn('Wake word detection failed to auto-start:', error)
  }
  
  await resizeWindow(false, false, false)
  
  console.log('⌨️ Keyboard Shortcuts:')
  console.log('   Ctrl+Shift+E = Start/Stop ML Eye Tracking + Window Movement')
  console.log('   Ctrl+Shift+S = Emergency Stop (stop all tracking)')
  console.log('   Ctrl+Shift+C = Toggle Chat Window')
  console.log('   Ctrl+Shift+T = Toggle Transparency Controls')
  console.log('   Ctrl+Shift+A = Toggle AI Models Window')
  console.log('   Escape = Close any open panels')
  console.log('🎯 Control Panel is draggable - click and drag to move!')
  console.log('📐 Chat Window is resizable - drag the resize handles!')
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
  document.removeEventListener('click', handleClickOutside)
  document.removeEventListener('mouseup', handleDragEnd)
  
  // Clean up wake word event listeners
  window.removeEventListener('wake-word-open-chat-window', handleWakeWordOpenChat)
  window.removeEventListener('wake-word-trigger-mic', handleWakeWordTriggerMic)
  window.removeEventListener('speech-open-chat-window', handleSpeechOpenChat)
})
</script>

<template>
  <div class="app-layout">
    <!-- Control Panel Section -->
    <div class="control-panel-section">
      <div 
        class="control-panel-glass-bar" 
        :class="{ 'dragging': isDragging }"
        data-tauri-drag-region
      >
        <div class="control-buttons-row">
          <!-- AI Settings Button -->
          <button 
            @click="toggleAIModelsWindow"
            class="control-btn group"
            :class="{ 'active': showAIModelsWindow }"
            title="AI Settings (Ollama)"
          >
            <Cog6ToothIcon class="w-4 h-4 transition-all" 
              :class="showAIModelsWindow ? 'text-white' : 'text-white/70 group-hover:text-white'" />
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
            title="Wake Word Detection - Say 'Aubrey' to activate speech"
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

          <!-- Transparency Controls Button -->
          <button 
            @click="toggleTransparencyControls"
            class="control-btn group"
            :class="{ 'active': showTransparencyControls }"
            title="Transparency Controls"
          >
            <AdjustmentsHorizontalIcon class="w-4 h-4 transition-all" 
              :class="showTransparencyControls ? 'text-white' : 'text-white/70 group-hover:text-white'" />
          </button>

          <!-- Chat Window Button -->
          <button 
            @click="toggleChatWindow"
            class="control-btn group"
            :class="{ 'active': showChatWindow }"
            title="Chat Assistant"
          >
            <CommandLineIcon class="w-4 h-4 transition-all" 
              :class="showChatWindow ? 'text-white' : 'text-white/70 group-hover:text-white'" />
          </button>
        </div>
        
        <!-- Drag indicator -->
        <div class="drag-indicator" :class="{ 'visible': isDragging }">
          <div class="drag-dots">
            <span></span>
            <span></span>
            <span></span>
          </div>
        </div>
      </div>
    </div>

    <!-- Transparency Controls Section -->
    <Transition name="transparency-panel">
      <div v-if="showTransparencyControls" class="transparency-controls-section">
        <div class="transparency-controls-panel">
          <div class="panel-header">
            <div class="panel-title">
              <AdjustmentsHorizontalIcon class="w-4 h-4 text-white/80" />
              <span class="text-sm font-medium text-white/90">Transparency Controls</span>
            </div>
            <button @click="closeTransparencyControls" class="panel-close-btn">
              <XMarkIcon class="w-4 h-4 text-white/70 hover:text-white transition-colors" />
            </button>
          </div>
          <div class="panel-content">
            <TransparencyControls />
          </div>
        </div>
      </div>
    </Transition>

    <!-- AI Models Panel -->
    <AIModelsPanel 
      :show-a-i-models-window="showAIModelsWindow"
      @close="closeAIModelsWindow"
      @update:show-a-i-models-window="showAIModelsWindow = $event"
    />

    <!-- Chat Window -->
    <ChatWindow 
      :show-chat-window="showChatWindow"
      :selected-model="selectedModel"
      @close="closeChatWindow"
      @update:show-chat-window="showChatWindow = $event"
    />
  </div>
</template>

<style scoped>
.app-layout {
  @apply w-full h-full bg-transparent;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.control-panel-section {
  @apply w-full flex justify-center;
  height: 60px;
  padding: 8px;
  background: transparent;
}

.transparency-controls-section {
  @apply w-full flex justify-center;
  padding: 0 8px 8px 8px;
  background: transparent;
}

/* Curved Glass Control Panel Bar */
.control-panel-glass-bar {
  @apply rounded-full overflow-hidden relative;
  height: 44px;
  width: 320px;
  cursor: grab;
  user-select: none;
  
  /* Premium curved glass effect with darker background */
  background: linear-gradient(135deg, 
    rgba(17, 17, 21, 0.85) 0%,
    rgba(17, 17, 21, 0.75) 25%,
    rgba(17, 17, 21, 0.70) 50%,
    rgba(17, 17, 21, 0.75) 75%,
    rgba(17, 17, 21, 0.85) 100%
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
    rgba(17, 17, 21, 0.90) 0%,
    rgba(17, 17, 21, 0.80) 25%,
    rgba(17, 17, 21, 0.75) 50%,
    rgba(17, 17, 21, 0.80) 75%,
    rgba(17, 17, 21, 0.90) 100%
  );
  border-color: rgba(255, 255, 255, 0.4);
  box-shadow: 
    0 12px 40px rgba(0, 0, 0, 0.3),
    0 4px 12px rgba(0, 0, 0, 0.2),
    inset 0 1px 0 rgba(255, 255, 255, 0.5),
    inset 0 -1px 0 rgba(0, 0, 0, 0.1);
  transform: translateY(-1px);
}

/* Dragging state */
.control-panel-glass-bar.dragging {
  cursor: grabbing;
  transform: translateY(-2px) scale(1.02);
  box-shadow: 
    0 16px 50px rgba(0, 0, 0, 0.4),
    0 8px 20px rgba(0, 0, 0, 0.3),
    inset 0 1px 0 rgba(255, 255, 255, 0.6),
    inset 0 -1px 0 rgba(0, 0, 0, 0.1);
  border-color: rgba(255, 255, 255, 0.5);
  background: linear-gradient(135deg, 
    rgba(17, 17, 21, 0.95) 0%,
    rgba(17, 17, 21, 0.85) 25%,
    rgba(17, 17, 21, 0.80) 50%,
    rgba(17, 17, 21, 0.85) 75%,
    rgba(17, 17, 21, 0.95) 100%
  );
}

.control-buttons-row {
  @apply flex items-center justify-center gap-2 px-3 py-2 relative z-10;
  height: 100%;
}

.control-btn {
  @apply rounded-full transition-all duration-200 flex items-center justify-center;
  width: 28px;
  height: 28px;
  border: none;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(8px);
  cursor: pointer;
  pointer-events: auto;
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

/* Drag indicator */
.drag-indicator {
  @apply absolute top-1/2 left-3 transform -translate-y-1/2;
  opacity: 0;
  transition: opacity 0.3s ease;
  pointer-events: none;
}

.drag-indicator.visible {
  opacity: 1;
}

.drag-dots {
  @apply flex flex-col gap-1;
}

.drag-dots span {
  @apply w-1 h-1 rounded-full bg-white/60;
  display: block;
}

/* Transparency Controls Panel */
.transparency-controls-panel {
  @apply rounded-2xl overflow-hidden;
  width: 380px;
  pointer-events: auto;
  
  /* Same glass effect as other panels with darker background */
  background: linear-gradient(135deg, 
    rgba(17, 17, 21, 0.85) 0%,
    rgba(17, 17, 21, 0.75) 25%,
    rgba(17, 17, 21, 0.70) 50%,
    rgba(17, 17, 21, 0.75) 75%,
    rgba(17, 17, 21, 0.85) 100%
  );
  backdrop-filter: blur(60px) saturate(180%) brightness(1.1);
  border: 1px solid rgba(255, 255, 255, 0.25);
  box-shadow: 
    0 20px 60px rgba(0, 0, 0, 0.4),
    0 8px 24px rgba(0, 0, 0, 0.25),
    inset 0 1px 0 rgba(255, 255, 255, 0.3),
    inset 0 -1px 0 rgba(0, 0, 0, 0.1);
}

.panel-header {
  @apply flex items-center justify-between px-4 py-3 border-b border-white/10;
}

.panel-title {
  @apply flex items-center gap-2;
}

.panel-close-btn {
  @apply rounded-full p-1 hover:bg-white/10 transition-colors;
}

.panel-content {
  @apply p-4;
}

/* Transparency Panel Transitions */
.transparency-panel-enter-active,
.transparency-panel-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.transparency-panel-enter-from {
  opacity: 0;
  transform: translateY(-10px) scale(0.95);
}

.transparency-panel-leave-to {
  opacity: 0;
  transform: translateY(-10px) scale(0.95);
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

/* Floating animation for the entire bar (disabled when dragging) */
.control-panel-glass-bar:not(.dragging) {
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

.control-panel-glass-bar:hover:not(.dragging) {
  animation: none;
}

/* Ensure buttons don't interfere with dragging */
.control-btn {
  position: relative;
  z-index: 10;
}

/* Drag region styling */
.control-panel-glass-bar[data-tauri-drag-region] {
  -webkit-app-region: drag;
}

.control-btn {
  -webkit-app-region: no-drag;
}
</style> 