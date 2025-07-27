import { ref, computed } from 'vue'
import { getCompatibilityReport } from '../utils/browserCompat'

export function useControlPanelState() {
  // Dragging state
  const isDragging = ref(false)
  const dragStartTime = ref(0)

  // Window state
  const showChatWindow = ref(false)
  const showAIModelsWindow = ref(false)
  const showConversationalWindow = ref(false)
  const showTransparencyControls = ref(false)

  // Error handling state
  const speechError = ref<string | null>(null)

  // Browser compatibility
  const compatibilityReport = ref(getCompatibilityReport())

  // ML Eye tracking with window movement state
  const isGazeControlActive = ref(false)

  // Computed drag indicator visibility
  const dragIndicatorVisible = computed(() => isDragging.value)

  // Centralized window state manager - ensures only one window is open at a time
  const closeAllWindows = () => {
    showChatWindow.value = false
    showTransparencyControls.value = false
    showAIModelsWindow.value = false
    showConversationalWindow.value = false
  }

  const openWindow = (windowType: 'chat' | 'transparency' | 'aiModels' | 'conversational') => {
    // Close all windows first
    closeAllWindows()
    
    // Open the requested window
    switch (windowType) {
      case 'chat':
        showChatWindow.value = true
        break
      case 'transparency':
        showTransparencyControls.value = true
        break
      case 'aiModels':
        showAIModelsWindow.value = true
        break
      case 'conversational':
        showConversationalWindow.value = true
        break
    }
    
    console.log(`🪟 Window Manager: Opened ${windowType} window, closed all others`)
  }

  const toggleWindow = (windowType: 'chat' | 'transparency' | 'aiModels' | 'conversational') => {
    const currentState = getWindowState(windowType)
    
    if (currentState) {
      // If this window is open, close it
      closeAllWindows()
      console.log(`🪟 Window Manager: Closed ${windowType} window`)
    } else {
      // If this window is closed, open it (and close all others)
      openWindow(windowType)
    }
  }

  const getWindowState = (windowType: 'chat' | 'transparency' | 'aiModels' | 'conversational') => {
    switch (windowType) {
      case 'chat':
        return showChatWindow.value
      case 'transparency':
        return showTransparencyControls.value
      case 'aiModels':
        return showAIModelsWindow.value
      case 'conversational':
        return showConversationalWindow.value
      default:
        return false
    }
  }

  // Check if any window is currently open
  const hasOpenWindow = computed(() => {
    return showChatWindow.value || 
           showTransparencyControls.value || 
           showAIModelsWindow.value || 
           showConversationalWindow.value
  })

  return {
    isDragging,
    dragStartTime,
    showChatWindow,
    showAIModelsWindow,
    showConversationalWindow,
    showTransparencyControls,
    speechError,
    compatibilityReport,
    isGazeControlActive,
    dragIndicatorVisible,
    // Window management functions
    closeAllWindows,
    openWindow,
    toggleWindow,
    getWindowState,
    hasOpenWindow
  }
}