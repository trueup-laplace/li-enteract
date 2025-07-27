import { useControlPanelState } from './useControlPanelState'
import { useWindowRegistry } from './useWindowRegistry'
import { useWindowResizing } from './useWindowResizing'

/**
 * Refactored Control Panel Events using centralized window registry
 * This replaces the fragile CSS selector-based approach with a robust registry pattern
 */
export function useControlPanelEvents(
  store: any,
  mlEyeTracking: any,
  windowManager: any,
  wakeWordDetection: any,
  providedStateRefs: any
) {
  // Get state management (use provided refs if available, otherwise create new)
  const stateRefs = providedStateRefs || useControlPanelState()
  const windowRegistry = useWindowRegistry({ debugMode: false })
  const { resizeWindow } = useWindowResizing()

  // Extract reactive refs and methods for easier access
  const {
    showAIModelsWindow,
    showChatWindow,
    showConversationalWindow,
    closeAllWindows: stateCloseAllWindows,
    openWindow: stateOpenWindow,
    toggleWindow: stateToggleWindow
  } = stateRefs

  // Window control handlers
  const closeAllWindows = async () => {
    // Use state method for consistency
    stateCloseAllWindows()
    
    // Also close via registry for any remaining cleanup
    windowRegistry.closeAllWindows()
    
    // Reset window size
    try {
      await resizeWindow(false, false, false, false, false)
    } catch (error) {
      console.error('❌ Failed to resize window after closing all:', error)
    }
  }

  const closeSpecificWindows = (windowIds: string[]) => {
    windowRegistry.closeWindows(windowIds)
  }

  // AI Models window handlers
  const toggleAIModelsWindow = async (event: Event) => {
    event.stopPropagation()
    
    // If this window is already open, close it
    if (showAIModelsWindow.value) {
      stateCloseAllWindows() // Use state method for consistency
      windowRegistry.closeAllWindows() // Also close via registry
      console.log('⚙️ AI Models window closed')
      try {
        await resizeWindow(false, false, false, false, false)
      } catch (error) {
        console.error('❌ Failed to resize window after closing AI models:', error)
      }
      return
    }
    
    // Use state method to open window (handles closing others)
    stateOpenWindow('aiModels')
    
    // Close via registry for cleanup
    closeSpecificWindows(['chat-window', 'conversational-window'])
    
    // Small delay for smooth transition (matching CSS transition duration)
    await new Promise(resolve => setTimeout(resolve, 100))
    
    console.log('⚙️ AI Models window opened')

    try {
      // Resize window for AI models panel
      await resizeWindow(true, false, false, false, false)
    } catch (error) {
      console.error('❌ Failed to resize window for AI models:', error)
    }
  }

  const closeAIModelsWindow = () => {
    if (showAIModelsWindow.value) {
      showAIModelsWindow.value = false
      console.log('⚙️ AI Models window closed')
      
      // Resize window back to normal
      resizeWindow(false, false, false, false, false).catch(error => {
        console.error('❌ Failed to resize window after closing AI models:', error)
      })
    }
  }

  // Chat window handlers
  const toggleChatWindow = async (event: Event) => {
    event.stopPropagation()
    
    // If this window is already open, close it
    if (showChatWindow.value) {
      stateCloseAllWindows() // Use state method for consistency
      windowRegistry.closeAllWindows() // Also close via registry
      console.log('💬 Chat window closed')
      try {
        await resizeWindow(false, false, false, false, false)
      } catch (error) {
        console.error('❌ Failed to resize window after closing chat:', error)
      }
      return
    }
    
    // Use state method to open window (handles closing others)
    stateOpenWindow('chat')
    
    // Close via registry for cleanup
    closeSpecificWindows(['ai-models-window', 'conversational-window'])
    
    // Small delay for smooth transition (matching CSS transition duration)
    await new Promise(resolve => setTimeout(resolve, 100))
    
    console.log('💬 Chat window opened')

    try {
      // Resize window for chat
      await resizeWindow(false, true, false, false, false)
    } catch (error) {
      console.error('❌ Failed to resize window for chat:', error)
    }
  }

  const closeChatWindow = () => {
    if (showChatWindow.value) {
      showChatWindow.value = false
      console.log('💬 Chat window closed')
      
      // Resize window back to normal
      resizeWindow(false, false, false, false, false).catch(error => {
        console.error('❌ Failed to resize window after closing chat:', error)
      })
    }
  }

  // Conversational window handlers
  const toggleConversationalWindow = async (event: Event) => {
    event.stopPropagation()
    
    // If this window is already open, close it
    if (showConversationalWindow.value) {
      stateCloseAllWindows() // Use state method for consistency
      windowRegistry.closeAllWindows() // Also close via registry
      console.log('🎤 Conversational window closed')
      try {
        await resizeWindow(false, false, false, false, false)
      } catch (error) {
        console.error('❌ Failed to resize window after closing conversational:', error)
      }
      return
    }
    
    // Use state method to open window (handles closing others)
    stateOpenWindow('conversational')
    
    // Close via registry for cleanup
    closeSpecificWindows(['ai-models-window', 'chat-window'])
    
    // Small delay for smooth transition (matching CSS transition duration)
    await new Promise(resolve => setTimeout(resolve, 100))
    
    console.log('🎤 Conversational window opened')

    try {
      // Resize window for conversational interface
      await resizeWindow(false, false, true, false, false)
    } catch (error) {
      console.error('❌ Failed to resize window for conversational:', error)
    }
  }

  const closeConversationalWindow = () => {
    if (showConversationalWindow.value) {
      showConversationalWindow.value = false
      console.log('🎤 Conversational window closed')
      
      // Resize window back to normal
      resizeWindow(false, false, false, false, false).catch(error => {
        console.error('❌ Failed to resize window after closing conversational:', error)
      })
    }
  }

  // Drag handling (remains the same as it's window-level, not panel-level)
  const handleDragStart = async (event: Event) => {
    event.stopPropagation()
    
    try {
      console.log('🖱️ Starting window drag')
      // The actual drag implementation would go here
      // This is placeholder for the existing drag functionality
    } catch (error) {
      console.error('❌ Failed to start window drag:', error)
    }
  }

  const handleDragEnd = async () => {
    try {
      console.log('🖱️ Ending window drag')
      // The actual drag end implementation would go here
    } catch (error) {
      console.error('❌ Failed to end window drag:', error)
    }
  }

  // Window management helpers
  const registerWindow = (
    id: string,
    element: HTMLElement,
    closeHandler: () => void,
    options: {
      closeOnClickOutside?: boolean
      isModal?: boolean
      priority?: number
    } = {}
  ) => {
    return windowRegistry.registerWindow(id, element, {
      closeHandler,
      closeOnClickOutside: options.closeOnClickOutside ?? true,
      isModal: options.isModal ?? false,
      priority: options.priority ?? 100
    })
  }

  const unregisterWindow = (id: string) => {
    return windowRegistry.unregisterWindow(id)
  }

  // Focus management
  const bringWindowToFront = (windowId: string) => {
    return windowRegistry.bringToFront(windowId)
  }

  // State helpers
  const isAnyWindowOpen = () => {
    return showAIModelsWindow.value || 
           showChatWindow.value || 
           showConversationalWindow.value
  }

  const getOpenWindows = () => {
    const openWindows: string[] = []
    if (showAIModelsWindow.value) openWindows.push('ai-models-window')
    if (showChatWindow.value) openWindows.push('chat-window')
    if (showConversationalWindow.value) openWindows.push('conversational-window')
    return openWindows
  }

  // Registry debugging helpers
  const getRegistryInfo = () => {
    return {
      windowCount: windowRegistry.registeredWindowCount.value,
      windowIds: windowRegistry.windowIds.value,
      hasModals: windowRegistry.hasModalWindows.value
    }
  }

  const enableRegistryDebugMode = () => {
    windowRegistry.setDebugMode(true)
  }

  const disableRegistryDebugMode = () => {
    windowRegistry.setDebugMode(false)
  }

  // Additional methods for backward compatibility
  const openChatWindow = async () => {
    showChatWindow.value = true
    try {
      await resizeWindow(false, true, false, false, false)
    } catch (error) {
      console.error('❌ Failed to resize window for chat:', error)
    }
  }

  const toggleMLEyeTrackingWithMovement = () => {
    if (mlEyeTracking) {
      mlEyeTracking.toggleMLEyeTrackingWithMovement()
    }
  }

  const handleKeydown = async (event: KeyboardEvent) => {
    // Handle keyboard shortcuts
    if (event.key === 'Escape') {
      await closeAllWindows()
    }
  }

  const handleClickOutside = async (event: Event) => {
    // Use window registry for click outside detection
    const target = event.target as HTMLElement
    if (windowRegistry.isClickOutsideAll(target)) {
      await closeAllWindows()
    }
  }

  return {
    // Window toggle handlers (for buttons)
    toggleAIModelsWindow,
    toggleChatWindow,
    toggleConversationalWindow,
    
    // Window close handlers (for close buttons and registry)
    closeAIModelsWindow,
    closeChatWindow,
    closeConversationalWindow,
    closeAllWindows,
    closeSpecificWindows,
    
    // Additional window methods for backward compatibility
    openChatWindow,
    
    // Drag handlers
    handleDragStart,
    handleDragEnd,
    
    // ML Eye tracking and event handlers
    toggleMLEyeTrackingWithMovement,
    handleKeydown,
    handleClickOutside,
    
    // Window registry integration
    registerWindow,
    unregisterWindow,
    bringWindowToFront,
    
    // State helpers
    isAnyWindowOpen,
    getOpenWindows,
    
    // Registry debugging
    getRegistryInfo,
    enableRegistryDebugMode,
    disableRegistryDebugMode,
    
    // Direct access to registry for advanced use cases
    windowRegistry,
    
    // Legacy state refs (for backward compatibility during migration)
    showAIModelsWindow,
    showChatWindow,
    showConversationalWindow
  }
}