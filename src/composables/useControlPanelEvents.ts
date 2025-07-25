// composables/useControlPanelEvents.ts
import { type Ref } from 'vue'

interface StateRefs {
  isDragging: Ref<boolean>
  dragStartTime: Ref<number>
  showChatWindow: Ref<boolean>
  showTransparencyControls: Ref<boolean>
  showAIModelsWindow: Ref<boolean>
  showConversationalWindow: Ref<boolean>
  speechError: Ref<string | null>
  compatibilityReport: Ref<any>
  isGazeControlActive: Ref<boolean>
  dragIndicatorVisible: Ref<boolean>
}

export function useControlPanelEvents(
  store: any,
  mlEyeTracking: any,
  windowManager: any,
  wakeWordDetection: any,
  stateRefs: StateRefs
) {
  const {
    isDragging,
    dragStartTime,
    showChatWindow,
    showTransparencyControls,
    showAIModelsWindow,
    showConversationalWindow,
    speechError,
    compatibilityReport,
    isGazeControlActive
  } = stateRefs

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

  const openChatWindow = async () => {
    // Close other panels first
    if (showTransparencyControls.value) {
      showTransparencyControls.value = false
    }
    if (showAIModelsWindow.value) {
      showAIModelsWindow.value = false
    }
    
    showChatWindow.value = true
    console.log('💬 Chat window opened')
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
          await store.initializeSpeechTranscription('tiny')
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

  // Conversational window handlers
  const toggleConversationalWindow = async (event: Event) => {
    event.stopPropagation()
    
    // Close other panels first
    if (showTransparencyControls.value) {
      showTransparencyControls.value = false
    }
    if (showAIModelsWindow.value) {
      showAIModelsWindow.value = false
    }
    if (showChatWindow.value) {
      showChatWindow.value = false
    }
    
    showConversationalWindow.value = !showConversationalWindow.value
    console.log(`💬 Conversational window ${showConversationalWindow.value ? 'opened' : 'closed'}`)
  }

  const closeConversationalWindow = async () => {
    showConversationalWindow.value = false
    console.log('💬 Conversational window closed')
  }

  // Click outside to close panels
  const handleClickOutside = (event: Event) => {
    const target = event.target as HTMLElement
    const chatWindow = document.querySelector('.chat-window')
    const conversationalWindow = document.querySelector('.conversational-window')
    const transparencyPanel = document.querySelector('.transparency-controls-panel')
    const aiModelsPanel = document.querySelector('.ai-models-panel')
    const controlPanel = document.querySelector('.control-panel-glass-bar')
    
    if (chatWindow && controlPanel && showChatWindow.value &&
        !chatWindow.contains(target) && 
        !controlPanel.contains(target)) {
      closeChatWindow()
    }
    
    if (conversationalWindow && controlPanel && showConversationalWindow.value &&
        !conversationalWindow.contains(target) && 
        !controlPanel.contains(target)) {
      closeConversationalWindow()
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

  return {
    handleDragStart,
    handleDragEnd,
    toggleTransparencyControls,
    closeTransparencyControls,
    toggleAIModelsWindow,
    closeAIModelsWindow,
    toggleChatWindow,
    closeChatWindow,
    openChatWindow,
    toggleConversationalWindow,
    closeConversationalWindow,
    toggleSpeechTranscription,
    getSpeechIconClass,
    toggleMLEyeTrackingWithMovement,
    handleKeydown,
    handleClickOutside
  }
}