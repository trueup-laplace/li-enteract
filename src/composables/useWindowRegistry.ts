import { ref, computed, onUnmounted, nextTick } from 'vue'

// Types for window registry system
export interface RegisteredWindow {
  id: string
  element: HTMLElement
  closeOnClickOutside: boolean
  isModal: boolean
  zIndex?: number
  closeHandler?: () => void
  priority: number // Higher priority windows close first
}

export interface WindowRegistryConfig {
  debugMode?: boolean
  defaultCloseOnClickOutside?: boolean
  defaultIsModal?: boolean
  defaultPriority?: number
}

export interface ClickOutsideOptions {
  excludeSelectors?: string[]
  includeDescendants?: boolean
  stopPropagation?: boolean
}

// Global window registry state
const windows = ref(new Map<string, RegisteredWindow>())
const clickListenerAttached = ref(false)
const debugMode = ref(false)

// Debug logging utility
const debugLog = (message: string, data?: any) => {
  if (debugMode.value) {
    console.log(`[WindowRegistry] ${message}`, data || '')
  }
}

// Global click handler
const handleGlobalClick = (event: Event) => {
  const target = event.target as HTMLElement
  if (!target) return

  debugLog('Global click detected', { target: target.tagName, className: target.className })

  // Get all registered windows sorted by priority (higher priority first)
  const sortedWindows = Array.from(windows.value.values())
    .filter(window => window.closeOnClickOutside)
    .sort((a, b) => b.priority - a.priority)

  debugLog('Windows to check for click-outside', sortedWindows.map(w => ({ id: w.id, priority: w.priority })))

  // Check each window for click-outside
  for (const window of sortedWindows) {
    if (isClickOutsideWindow(target, window)) {
      debugLog(`Click outside detected for window: ${window.id}`)
      
      // Close the window
      if (window.closeHandler) {
        window.closeHandler()
      }
      
      // If this is a modal window, stop checking other windows
      if (window.isModal) {
        debugLog(`Modal window ${window.id} closed, stopping propagation`)
        event.stopPropagation()
        break
      }
    }
  }
}

// Check if click is outside a specific window
const isClickOutsideWindow = (target: HTMLElement, window: RegisteredWindow): boolean => {
  const { element } = window
  
  if (!element || !element.parentNode) {
    debugLog(`Window ${window.id} element not found or not in DOM`)
    return false
  }

  // Check if target is within the window element
  if (element.contains(target)) {
    debugLog(`Click is inside window ${window.id}`)
    return false
  }

  // Check if target is within any excluded elements (like control panels)
  const controlPanel = document.querySelector('.control-panel-glass-bar')
  if (controlPanel && controlPanel.contains(target)) {
    debugLog(`Click is within control panel, ignoring for window ${window.id}`)
    return false
  }

  debugLog(`Click is outside window ${window.id}`)
  return true
}

// Attach global click listener
const attachGlobalClickListener = () => {
  if (clickListenerAttached.value) return

  document.addEventListener('click', handleGlobalClick, true)
  clickListenerAttached.value = true
  debugLog('Global click listener attached')
}

// Detach global click listener
const detachGlobalClickListener = () => {
  if (!clickListenerAttached.value) return

  document.removeEventListener('click', handleGlobalClick, true)
  clickListenerAttached.value = false
  debugLog('Global click listener detached')
}

// Main composable function
export function useWindowRegistry(config: WindowRegistryConfig = {}) {
  const {
    debugMode: configDebugMode = false,
    defaultCloseOnClickOutside = true,
    defaultIsModal = false,
    defaultPriority = 100
  } = config

  // Set debug mode
  debugMode.value = configDebugMode

  /**
   * Register a window element in the global registry
   */
  const registerWindow = async (
    id: string,
    element: HTMLElement | null,
    options: {
      closeOnClickOutside?: boolean
      isModal?: boolean
      zIndex?: number
      closeHandler?: () => void
      priority?: number
    } = {}
  ) => {
    if (!element) {
      console.warn(`[WindowRegistry] Cannot register window ${id}: element is null`)
      return false
    }

    await nextTick() // Ensure DOM is ready

    const windowConfig: RegisteredWindow = {
      id,
      element,
      closeOnClickOutside: options.closeOnClickOutside ?? defaultCloseOnClickOutside,
      isModal: options.isModal ?? defaultIsModal,
      zIndex: options.zIndex,
      closeHandler: options.closeHandler,
      priority: options.priority ?? defaultPriority
    }

    windows.value.set(id, windowConfig)
    
    // Apply z-index if provided
    if (windowConfig.zIndex) {
      element.style.zIndex = windowConfig.zIndex.toString()
    }

    // Attach global click listener if we have windows that need click-outside detection
    if (windowConfig.closeOnClickOutside) {
      attachGlobalClickListener()
    }

    debugLog(`Window registered: ${id}`, windowConfig)
    return true
  }

  /**
   * Unregister a window from the global registry
   */
  const unregisterWindow = (id: string) => {
    const wasRegistered = windows.value.has(id)
    windows.value.delete(id)

    // If no more windows need click-outside detection, remove global listener
    const hasClickOutsideWindows = Array.from(windows.value.values())
      .some(window => window.closeOnClickOutside)

    if (!hasClickOutsideWindows) {
      detachGlobalClickListener()
    }

    debugLog(`Window unregistered: ${id}`, { wasRegistered })
    return wasRegistered
  }

  /**
   * Update window configuration
   */
  const updateWindow = (id: string, updates: Partial<RegisteredWindow>) => {
    const existingWindow = windows.value.get(id)
    if (!existingWindow) {
      console.warn(`[WindowRegistry] Cannot update window ${id}: not found`)
      return false
    }

    const updatedWindow = { ...existingWindow, ...updates }
    windows.value.set(id, updatedWindow)

    // Update z-index if changed
    if (updates.zIndex && existingWindow.element) {
      existingWindow.element.style.zIndex = updates.zIndex.toString()
    }

    debugLog(`Window updated: ${id}`, updates)
    return true
  }

  /**
   * Check if a click target is outside all registered windows
   */
  const isClickOutsideAll = (target: HTMLElement, options: ClickOutsideOptions = {}): boolean => {
    const {
      excludeSelectors = ['.control-panel-glass-bar']
    } = options

    // Check if target is within any excluded elements
    for (const selector of excludeSelectors) {
      const excludedElement = document.querySelector(selector)
      if (excludedElement && excludedElement.contains(target)) {
        debugLog('Click is within excluded element', selector)
        return false
      }
    }

    // Check if target is within any registered window
    for (const window of windows.value.values()) {
      if (window.element && window.element.contains(target)) {
        debugLog('Click is within registered window', window.id)
        return false
      }
    }

    debugLog('Click is outside all registered windows')
    return true
  }

  /**
   * Check if a specific window exists in the registry
   */
  const hasWindow = (id: string): boolean => {
    return windows.value.has(id)
  }

  /**
   * Get window configuration by ID
   */
  const getWindow = (id: string): RegisteredWindow | undefined => {
    return windows.value.get(id)
  }

  /**
   * Get all registered windows
   */
  const getAllWindows = (): RegisteredWindow[] => {
    return Array.from(windows.value.values())
  }

  /**
   * Close all registered windows with smooth transitions
   */
  const closeAllWindows = async () => {
    const sortedWindows = getAllWindows().sort((a, b) => b.priority - a.priority)
    
    // Close windows sequentially for better visual effect
    for (const window of sortedWindows) {
      if (window.closeHandler) {
        debugLog(`Closing window: ${window.id}`)
        window.closeHandler()
        // Small delay between closures for smooth transitions
        await new Promise(resolve => setTimeout(resolve, 50))
      }
    }
  }

  /**
   * Close specific windows by ID
   */
  const closeWindows = (ids: string[]) => {
    const windowsToClose = ids
      .map(id => windows.value.get(id))
      .filter(Boolean) as RegisteredWindow[]
      
    const sortedWindows = windowsToClose.sort((a, b) => b.priority - a.priority)
    
    for (const window of sortedWindows) {
      if (window.closeHandler) {
        debugLog(`Closing specific window: ${window.id}`)
        window.closeHandler()
      }
    }
  }

  /**
   * Get the highest z-index among registered windows
   */
  const getHighestZIndex = (): number => {
    let highest = 1000 // Base z-index
    
    for (const window of windows.value.values()) {
      if (window.zIndex && window.zIndex > highest) {
        highest = window.zIndex
      }
    }
    
    return highest
  }

  /**
   * Bring window to front by setting highest z-index
   */
  const bringToFront = (id: string): boolean => {
    const window = windows.value.get(id)
    if (!window || !window.element) {
      return false
    }

    const newZIndex = getHighestZIndex() + 1
    window.element.style.zIndex = newZIndex.toString()
    window.zIndex = newZIndex

    debugLog(`Brought window to front: ${id}`, { zIndex: newZIndex })
    return true
  }

  /**
   * Enable or disable debug mode
   */
  const setDebugMode = (enabled: boolean) => {
    debugMode.value = enabled
    debugLog(`Debug mode ${enabled ? 'enabled' : 'disabled'}`)
  }

  // Computed values
  const registeredWindowCount = computed(() => windows.value.size)
  const hasModalWindows = computed(() => 
    Array.from(windows.value.values()).some(window => window.isModal)
  )
  const windowIds = computed(() => Array.from(windows.value.keys()))

  // Cleanup on unmount
  onUnmounted(() => {
    debugLog('Cleaning up window registry')
    
    // Clear all windows for this instance
    windows.value.clear()
    
    // Detach global listener if no windows remain
    if (windows.value.size === 0) {
      detachGlobalClickListener()
    }
  })

  return {
    // Registration methods
    registerWindow,
    unregisterWindow,
    updateWindow,
    
    // Query methods
    hasWindow,
    getWindow,
    getAllWindows,
    isClickOutsideAll,
    
    // Control methods
    closeAllWindows,
    closeWindows,
    bringToFront,
    getHighestZIndex,
    
    // Configuration
    setDebugMode,
    
    // Computed state
    registeredWindowCount,
    hasModalWindows,
    windowIds,
    
    // Direct access to registry (for debugging)
    windows: computed(() => windows.value)
  }
}

// Utility function for components to easily register themselves
export function useWindowRegistration(
  windowId: string,
  options: {
    closeOnClickOutside?: boolean
    isModal?: boolean
    priority?: number
    closeHandler?: () => void
  } = {}
) {
  const registry = useWindowRegistry()
  
  const registerSelf = (element: HTMLElement | null) => {
    if (element) {
      return registry.registerWindow(windowId, element, options)
    }
    return Promise.resolve(false)
  }
  
  const unregisterSelf = () => {
    return registry.unregisterWindow(windowId)
  }
  
  // Auto-unregister on unmount
  onUnmounted(() => {
    unregisterSelf()
  })
  
  return {
    registerSelf,
    unregisterSelf,
    ...registry
  }
}