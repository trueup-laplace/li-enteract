export interface WindowRegistryConfig {
  closeOnClickOutside?: boolean
  isModal?: boolean
  priority?: number
  zIndex?: number
  closeHandler?: () => void
  focusHandler?: () => void
}

export interface RegisteredWindow {
  id: string
  element: HTMLElement
  config: WindowRegistryConfig
  isActive: boolean
  registeredAt: number
}

export type WindowId = string

export interface WindowRegistryState {
  windows: Map<WindowId, RegisteredWindow>
  activeWindows: Set<WindowId>
  clickHandlers: Set<(event: MouseEvent) => void>
}

export interface WindowRegistryAPI {
  // Registration
  register: (id: WindowId, element: HTMLElement, config?: WindowRegistryConfig) => void
  unregister: (id: WindowId) => void
  
  // State queries
  getWindow: (id: WindowId) => RegisteredWindow | undefined
  getAllWindows: () => RegisteredWindow[]
  getActiveWindows: () => RegisteredWindow[]
  isRegistered: (id: WindowId) => boolean
  isActive: (id: WindowId) => boolean
  
  // Focus management
  setActive: (id: WindowId) => void
  setInactive: (id: WindowId) => void
  bringToFront: (id: WindowId) => void
  
  // Click detection
  isClickOutside: (target: HTMLElement, windowId?: WindowId) => boolean
  isClickOutsideAll: (target: HTMLElement) => boolean
  
  // Cleanup
  cleanup: () => void
}