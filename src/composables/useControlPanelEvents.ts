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
    stateRefs.toggleWindow('transparency')
  }

  const closeTransparencyControls = () => {
    stateRefs.closeAllWindows()
  }

  const toggleAIModelsWindow = async (event: Event) => {
    event.stopPropagation()
    stateRefs.toggleWindow('aiModels')
  }

  const closeAIModelsWindow = () => {
    stateRefs.closeAllWindows()
  }

  const toggleChatWindow = async (event: Event) => {
    event.stopPropagation()
    stateRefs.toggleWindow('chat')
  }

  const closeChatWindow = async () => {
    stateRefs.closeAllWindows()
  }

  const openChatWindow = async () => {
    stateRefs.openWindow('chat')
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
    
    if (event.ctrlKey && event.shiftKey && event.key === 'V') {
      event.preventDefault()
      await toggleConversationalWindow(event)
      console.log('💬 Keyboard shortcut: Conversational window toggled')
    }
    
    if (event.key === 'Escape') {
      event.preventDefault()
      // Close all windows when escape is pressed
      stateRefs.closeAllWindows()
      console.log('🪟 Window Manager: Escape key pressed - closed all windows')
    }
  }

  // Conversational window handlers
  const toggleConversationalWindow = async (event: Event) => {
    event.stopPropagation()
    stateRefs.toggleWindow('conversational')
  }

  const closeConversationalWindow = async () => {
    stateRefs.closeAllWindows()
  }


  // Click outside to close panels
  const handleClickOutside = (event: Event) => {
    const target = event.target as HTMLElement
    const chatWindow = document.querySelector('.chat-window')
    const conversationalWindow = document.querySelector('.conversational-window')
    const transparencyPanel = document.querySelector('.transparency-controls-panel')
    const settingsPanel = document.querySelector('.settings-panel-section')
    const aiModelsPanel = document.querySelector('.ai-models-panel')
    const controlPanel = document.querySelector('.control-panel-glass-bar')
    
    // Check if click is outside any window and control panel
    const isOutsideAllWindows = (
      (!chatWindow || !chatWindow.contains(target)) &&
      (!conversationalWindow || !conversationalWindow.contains(target)) &&
      (!transparencyPanel || !transparencyPanel.contains(target)) &&
      (!settingsPanel || !settingsPanel.contains(target)) &&
      (!aiModelsPanel || !aiModelsPanel.contains(target)) &&
      (!controlPanel || !controlPanel.contains(target))
    )
    
    // Only close windows if user clicked outside all windows AND control panel
    // AND there's actually a window open
    if (isOutsideAllWindows && stateRefs.hasOpenWindow.value) {
      // Exception: Don't close conversational window on outside clicks
      // It should only close via explicit user action (X button)
      if (!showConversationalWindow.value) {
        stateRefs.closeAllWindows()
        console.log('🪟 Window Manager: Clicked outside - closed all windows')
      }
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
    toggleMLEyeTrackingWithMovement,
    handleKeydown,
    handleClickOutside
  }
}