import { ref, watch } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { LogicalSize, LogicalPosition } from '@tauri-apps/api/dpi'
import { useAppStore } from '../stores/app'

export function useWindowManager() {
  const store = useAppStore()
  const currentWindow = getCurrentWindow()
  
  const isDragging = ref(false)
  const isResizing = ref(false)
  
  // Window dimensions for collapsed state
  const normalSize = { width: 800, height: 600 }
  const collapsedSize = { width: 300, height: 80 }

  const initializeWindow = async () => {
    try {
      // Set initial window properties
      await currentWindow.setDecorations(false)
      await currentWindow.setAlwaysOnTop(true)
      await currentWindow.setSize(new LogicalSize(normalSize.width, normalSize.height))
      
      // Center window initially
      const screenSize = await currentWindow.outerSize()
      const position = new LogicalPosition(
        Math.floor((screenSize.width - normalSize.width) / 2),
        Math.floor((screenSize.height - normalSize.height) / 2)
      )
      await currentWindow.setPosition(position)
      store.updateWindowPosition(position.x, position.y)
      
    } catch (error) {
      console.error('Failed to initialize window:', error)
    }
  }

  const toggleCollapse = async () => {
    try {
      store.toggleWindowCollapse()
      
      if (store.windowCollapsed) {
        await currentWindow.setSize(new LogicalSize(collapsedSize.width, collapsedSize.height))
      } else {
        await currentWindow.setSize(new LogicalSize(normalSize.width, normalSize.height))
      }
    } catch (error) {
      console.error('Failed to toggle window collapse:', error)
    }
  }



  const startDrag = async () => {
    try {
      isDragging.value = true
      await currentWindow.startDragging()
    } catch (error) {
      console.error('Failed to start dragging:', error)
    } finally {
      isDragging.value = false
    }
  }

  const minimizeWindow = async () => {
    try {
      console.log('Minimizing window...')
      await currentWindow.minimize()
      console.log('Window minimized successfully')
    } catch (error) {
      console.error('Failed to minimize window:', error)
      // Fallback: try to hide the window
      try {
        await currentWindow.hide()
      } catch (hideError) {
        console.error('Failed to hide window as fallback:', hideError)
      }
    }
  }

  const closeWindow = async () => {
    try {
      console.log('Closing window...')
      await currentWindow.close()
      console.log('Window closed successfully')
    } catch (error) {
      console.error('Failed to close window:', error)
      // Force close if needed
      try {
        await currentWindow.destroy()
      } catch (destroyError) {
        console.error('Failed to destroy window as fallback:', destroyError)
      }
    }
  }



  // Listen for window events
  const setupWindowListeners = () => {
    // These would be set up in a real Tauri app
    // currentWindow.listen('tauri://move', (event) => {
    //   store.updateWindowPosition(event.payload.x, event.payload.y)
    // })
  }

  return {
    isDragging,
    isResizing,
    normalSize,
    collapsedSize,
    initializeWindow,
    toggleCollapse,
    startDrag,
    minimizeWindow,
    closeWindow,
    setupWindowListeners
  }
} 