// composables/useControlPanelEvents.ts
import { type Ref } from 'vue'

interface StateRefs {
  isDragging: Ref<boolean>
  dragStartTime: Ref<number>
  showChatWindow: Ref<boolean>
  showAIModelsWindow: Ref<boolean>
  showConversationalWindow: Ref<boolean>
  speechError: Ref<string | null>
  compatibilityReport: Ref<any>
  isGazeControlActive: Ref<boolean>
  dragIndicatorVisible: Ref<boolean>
  // Window management functions
  closeAllWindows: () => void
  openWindow: (windowType: 'chat' | 'transparency' | 'aiModels' | 'conversational') => void
  toggleWindow: (windowType: 'chat' | 'transparency' | 'aiModels' | 'conversational') => void
  getWindowState: (windowType: 'chat' | 'transparency' | 'aiModels' | 'conversational') => boolean
  hasOpenWindow: Ref<boolean>
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



  const toggleAIModelsWindow = async (event: Event) => {
    event.stopPropagation()
    
    if (showChatWindow.value) {
      showChatWindow.value = false
    }
    
    showAIModelsWindow.value = !showAIModelsWindow.value
    console.log(`🤖 AI Models window ${showAIModelsWindow.value ? 'opened' : 'closed'}`)
  }

  const closeAIModelsWindow = () => {
    stateRefs.closeAllWindows()
  }

  const toggleChatWindow = async (event: Event) => {
    event.stopPropagation()
    
    // Close other panels first to ensure only one window is open at a time
    if (showAIModelsWindow.value) {
      showAIModelsWindow.value = false
    }
    if (showConversationalWindow.value) {
      showConversationalWindow.value = false
    }
    
    showChatWindow.value = !showChatWindow.value
    console.log(`💬 Chat window ${showChatWindow.value ? 'opened' : 'closed'}`)
  }

  const closeChatWindow = async () => {
    stateRefs.closeAllWindows()
  }

  const openChatWindow = async () => {
    // Close other panels first
    if (showAIModelsWindow.value) {
      showAIModelsWindow.value = false
    }
    if (showConversationalWindow.value) {
      showConversationalWindow.value = false
    }
    
    showChatWindow.value = true
    console.log('💬 Chat window opened')
  }

  // Speech transcription functionality removed - now handled in chat interface
  // The microphone button has been moved to the chat interface for better UX

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
    

    
    if (event.ctrlKey && event.shiftKey && event.key === 'A') {
      event.preventDefault()
      await toggleAIModelsWindow(event)
      console.log('🤖 Keyboard shortcut: AI Models window toggled')
    }
    
    if (event.ctrlKey && event.shiftKey && event.key === 'V') {
      event.preventDefault()
      await toggleConversationalWindow(event)
      console.log('💬 Keyboard shortcut: Conversational window toggled')
    }
    
    if (event.key === 'Escape') {
      event.preventDefault()
      if (showChatWindow.value) {
        await closeChatWindow()
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
    stateRefs.closeAllWindows()
  }


  // Click outside to close panels
  const handleClickOutside = (event: Event) => {
    const target = event.target as HTMLElement
    const chatWindow = document.querySelector('.chat-window')
    const conversationalWindow = document.querySelector('.conversational-window')
    const aiModelsPanel = document.querySelector('.ai-models-panel')
    const controlPanel = document.querySelector('.control-panel-glass-bar')
    
    if (chatWindow && controlPanel && showChatWindow.value &&
        !chatWindow.contains(target) && 
        !controlPanel.contains(target)) {
      closeChatWindow()
    }
    
    // IMPORTANT: Disable click-outside closing for conversational window
    // The conversational window should only close via explicit user action (X button)
    // This prevents accidental closing when using controls inside the window
    // The original logic is commented out below:
    /*
    if (conversationalWindow && controlPanel && showConversationalWindow.value &&
        !conversationalWindow.contains(target) && 
        !controlPanel.contains(target)) {
      closeConversationalWindow()
    }
    */
    
    if (aiModelsPanel && controlPanel && showAIModelsWindow.value &&
        !aiModelsPanel.contains(target) && 
        !controlPanel.contains(target)) {
      closeAIModelsWindow()
    }
  }

  return {
    handleDragStart,
    handleDragEnd,
    toggleAIModelsWindow,
    closeAIModelsWindow,
    toggleChatWindow,
    closeChatWindow,
    openChatWindow,
    toggleConversationalWindow,
    closeConversationalWindow,
    toggleMLEyeTrackingWithMovement,
    handleKeydown,
    handleClickOutside
  }
}