import { ref } from 'vue'
import { Window } from '@tauri-apps/api/window'
import { LogicalSize } from '@tauri-apps/api/dpi'

const CONTROL_PANEL_HEIGHT = 60
const MIN_CHAT_HEIGHT = 400
const MAX_CHAT_HEIGHT = 1200
const MIN_CHAT_WIDTH = 650
const MAX_CHAT_WIDTH = 1200

export const useWindowResizing = () => {
  const currentWindow = Window.getCurrent()
  
  // Chat window resize state
  const chatWindowSize = ref({
    width: 750,
    height: 500
  })
  const isResizing = ref(false)
  const resizeHandle = ref<string | null>(null)
  const resizeStartPos = ref({ x: 0, y: 0 })
  const resizeStartSize = ref({ width: 0, height: 0 })

  // Dynamic window resizing
  const resizeWindow = async (showChat: boolean, showTransparency: boolean = false, showAIModels: boolean = false) => {
    try {
      let height = CONTROL_PANEL_HEIGHT
      
      // Add transparency panel height if shown
      if (showTransparency) {
        height += 380 // Transparency panel height (increased for all components)
      }
      
      // Add AI models window height if shown
      if (showAIModels) {
        height += 550 // AI models window height (increased to show all content)
      }
      
      // Add chat window height if shown
      if (showChat) {
        height += chatWindowSize.value.height + 20
      }
      
      const width = Math.max(320, chatWindowSize.value.width + 40)
      await currentWindow.setSize(new LogicalSize(width, height))
      console.log(`🪟 Window resized to: ${width}x${height}px`)
    } catch (error) {
      console.error('Failed to resize window:', error)
    }
  }

  // Chat window resize functionality
  const startResize = (event: MouseEvent, handle: string) => {
    event.preventDefault()
    event.stopPropagation()
    
    isResizing.value = true
    resizeHandle.value = handle
    resizeStartPos.value = { x: event.clientX, y: event.clientY }
    resizeStartSize.value = { ...chatWindowSize.value }
    
    document.addEventListener('mousemove', handleResize)
    document.addEventListener('mouseup', stopResize)
    
    console.log(`🔄 Started resizing chat window from ${handle}`)
  }

  const handleResize = (event: MouseEvent) => {
    if (!isResizing.value || !resizeHandle.value) return
    
    const deltaX = event.clientX - resizeStartPos.value.x
    const deltaY = event.clientY - resizeStartPos.value.y
    
    let newWidth = resizeStartSize.value.width
    let newHeight = resizeStartSize.value.height
    
    // Handle different resize directions
    if (resizeHandle.value.includes('right')) {
      newWidth = resizeStartSize.value.width + deltaX
    }
    if (resizeHandle.value.includes('left')) {
      newWidth = resizeStartSize.value.width - deltaX
    }
    if (resizeHandle.value.includes('bottom')) {
      newHeight = resizeStartSize.value.height + deltaY
    }
    if (resizeHandle.value.includes('top')) {
      newHeight = resizeStartSize.value.height - deltaY
    }
    
    // Apply constraints
    newWidth = Math.max(MIN_CHAT_WIDTH, Math.min(MAX_CHAT_WIDTH, newWidth))
    newHeight = Math.max(MIN_CHAT_HEIGHT, Math.min(MAX_CHAT_HEIGHT, newHeight))
    
    chatWindowSize.value = { width: newWidth, height: newHeight }
  }

  const stopResize = () => {
    isResizing.value = false
    resizeHandle.value = null
    
    document.removeEventListener('mousemove', handleResize)
    document.removeEventListener('mouseup', stopResize)
    
    console.log(`✅ Finished resizing chat window to: ${chatWindowSize.value.width}x${chatWindowSize.value.height}px`)
  }

  const cleanup = () => {
    document.removeEventListener('mousemove', handleResize)
    document.removeEventListener('mouseup', stopResize)
  }

  return {
    chatWindowSize,
    isResizing,
    resizeWindow,
    startResize,
    cleanup
  }
} 