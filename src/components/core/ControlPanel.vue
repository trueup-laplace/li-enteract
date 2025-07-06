<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue'
import { 
  MicrophoneIcon, 
  Cog6ToothIcon,
  CommandLineIcon,
  CpuChipIcon,
  ExclamationTriangleIcon,
  XMarkIcon,
  PaperAirplaneIcon,
  ArrowsPointingOutIcon,
  AdjustmentsHorizontalIcon,
  RocketLaunchIcon,
  TrashIcon,
  ArrowDownTrayIcon,
  CameraIcon,
  MagnifyingGlassIcon,
  ShieldCheckIcon
} from '@heroicons/vue/24/outline'
import { useAppStore } from '../../stores/app'
import { useMLEyeTracking } from '../../composables/useMLEyeTracking'
import { useWindowManager } from '../../composables/useWindowManager'
import { useWakeWordDetection } from '../../composables/useWakeWordDetection'
import { getCompatibilityReport } from '../../utils/browserCompat'
import { Window } from '@tauri-apps/api/window'
import { LogicalSize } from '@tauri-apps/api/dpi'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import TransparencyControls from './TransparencyControls.vue'

const store = useAppStore()
const mlEyeTracking = useMLEyeTracking()
const windowManager = useWindowManager()
const wakeWordDetection = useWakeWordDetection()
const currentWindow = Window.getCurrent()

// Window size constants
const CONTROL_PANEL_HEIGHT = 60
const MIN_CHAT_HEIGHT = 400
const MAX_CHAT_HEIGHT = 1200
const MIN_CHAT_WIDTH = 450
const MAX_CHAT_WIDTH = 800

// Dragging state
const isDragging = ref(false)
const dragStartTime = ref(0)

// Chat window state
const showChatWindow = ref(false)
const chatMessage = ref('')
const chatHistory = ref<Array<{type: 'user' | 'assistant' | 'transcription', message: string, timestamp: Date, isInterim?: boolean, confidence?: number}>>([])

// Transparency controls state
const showTransparencyControls = ref(false)

// AI Models window state
const showAIModelsWindow = ref(false)
const ollamaModels = ref<OllamaModel[]>([])
const ollamaStatus = ref<OllamaStatus>({ status: 'checking' })
const isLoadingModels = ref(false)
const modelsError = ref<string | null>(null)
const selectedModel = ref<string | null>(null)
const pullingModel = ref<string | null>(null)
const deletingModel = ref<string | null>(null)

// Chat window resize state
const chatWindowSize = ref({
  width: 500,
  height: 500
})
const isResizing = ref(false)
const resizeHandle = ref<string | null>(null)
const resizeStartPos = ref({ x: 0, y: 0 })
const resizeStartSize = ref({ width: 0, height: 0 })

// Error handling state
const speechError = ref<string | null>(null)
const wakeWordError = ref<string | null>(null)
const isRetrying = ref(false)

// Browser compatibility
const compatibilityReport = ref(getCompatibilityReport())

// ML Eye tracking with window movement state
const isGazeControlActive = ref(false)

// Types for Ollama
interface OllamaModel {
  name: string
  modified_at: string
  size: number
  digest: string
  details?: {
    format: string
    family: string
    parameter_size: string
    quantization_level: string
  }
}

interface OllamaStatus {
  status: string
  version?: string
}

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

// Watch for chat window state changes to resize window
watch(showChatWindow, async (newValue) => {
  await resizeWindow(newValue, showTransparencyControls.value, showAIModelsWindow.value)
})

// Watch for transparency controls state changes to resize window
watch(showTransparencyControls, async (newValue) => {
  await resizeWindow(showChatWindow.value, newValue, showAIModelsWindow.value)
})

// Watch for AI models window state changes to resize window
watch(showAIModelsWindow, async (newValue) => {
  await resizeWindow(showChatWindow.value, showTransparencyControls.value, newValue)
})

// Watch for chat window size changes to resize Tauri window
watch(chatWindowSize, async () => {
  if (showChatWindow.value) {
    await resizeWindow(showChatWindow.value, showTransparencyControls.value, showAIModelsWindow.value)
  }
}, { deep: true })

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

// Transparency controls functions
const toggleTransparencyControls = (event: Event) => {
  event.stopPropagation()
  
  // Close other panels if open to avoid conflicts
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

// AI models window functions
const toggleAIModelsWindow = async (event: Event) => {
  event.stopPropagation()
  
  // Close other panels if open to avoid conflicts
  if (showChatWindow.value) {
    showChatWindow.value = false
  }
  if (showTransparencyControls.value) {
    showTransparencyControls.value = false
  }
  
  showAIModelsWindow.value = !showAIModelsWindow.value
  console.log(`🤖 AI Models window ${showAIModelsWindow.value ? 'opened' : 'closed'}`)
  
  // Fetch Ollama status and models when opening
  if (showAIModelsWindow.value) {
    await fetchOllamaStatus()
    if (ollamaStatus.value.status === 'running') {
      await fetchOllamaModels()
    }
  }
}

const closeAIModelsWindow = () => {
  showAIModelsWindow.value = false
  console.log('🤖 AI Models window closed')
}

// Chat window functions
const toggleChatWindow = async (event: Event) => {
  event.stopPropagation()
  
  // Close other panels if open to avoid conflicts
  if (showTransparencyControls.value) {
    showTransparencyControls.value = false
  }
  if (showAIModelsWindow.value) {
    showAIModelsWindow.value = false
  }
  
  showChatWindow.value = !showChatWindow.value
  console.log(`💬 Chat window ${showChatWindow.value ? 'opened' : 'closed'}`)
  
  // Focus on input when opening
  if (showChatWindow.value) {
    setTimeout(() => {
      const input = document.querySelector('.chat-input') as HTMLInputElement
      if (input) input.focus()
    }, 150)
  }
}

// Screen Analysis and Vision
const takeScreenshotAndAnalyze = async () => {
  try {
    console.log('🔍 Analyzing screen for vision analysis...')
    
    // Take screenshot
    const screenshot = await invoke<{image_base64: string, width: number, height: number}>('capture_screenshot')
    
    // Auto-open chat window if not open
    if (!showChatWindow.value) {
      showChatWindow.value = true
      setTimeout(() => {
        scrollChatToBottom()
      }, 150)
    }
    
    // Add screen analysis message to chat
    const screenshotMessageIndex = chatHistory.value.length
    chatHistory.value.push({
      type: 'user',
      message: `🔍 Screen captured for analysis (${screenshot.width}×${screenshot.height})`,
      timestamp: new Date()
    })
    
    // Generate unique session ID
    const sessionId = `vision-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
    
    // Add streaming message placeholder
    const streamingMessageIndex = chatHistory.value.length
    chatHistory.value.push({
      type: 'assistant',
      message: '🔍 Analyzing screenshot▋',
      timestamp: new Date()
    })
    
    setTimeout(() => {
      scrollChatToBottom()
    }, 50)
    
    let fullResponse = ''
    let isTyping = true
    
    // Set up event listener for vision analysis
    const unlisten = await listen(`ollama-stream-${sessionId}`, (event: any) => {
      const data = event.payload
      
      switch (data.type) {
        case 'start':
          console.log(`👁️ Started vision analysis with ${data.model}`)
          break
          
        case 'chunk':
          if (data.text) {
            fullResponse += data.text
            if (chatHistory.value[streamingMessageIndex]) {
              chatHistory.value[streamingMessageIndex].message = fullResponse + (isTyping ? '▋' : '')
            }
            
            setTimeout(() => {
              scrollChatToBottom()
            }, 10)
          }
          
          if (data.done) {
            isTyping = false
            if (chatHistory.value[streamingMessageIndex]) {
              chatHistory.value[streamingMessageIndex].message = fullResponse
            }
          }
          break
          
        case 'error':
          isTyping = false
          console.error('Vision analysis error:', data.error)
          if (chatHistory.value[streamingMessageIndex]) {
            chatHistory.value[streamingMessageIndex].message = `❌ Vision analysis error: ${data.error}`
          }
          break
          
        case 'complete':
          isTyping = false
          unlisten()
          break
      }
    })
    
    // Start vision analysis
    await invoke('generate_vision_analysis', {
      prompt: 'Please analyze this screenshot in detail.',
      imageBase64: screenshot.image_base64,
      sessionId: sessionId
    })
    
  } catch (error) {
    console.error('Failed to analyze screen:', error)
    chatHistory.value.push({
      type: 'assistant',
      message: `❌ Failed to analyze screen: ${error}`,
      timestamp: new Date()
    })
  }
}

// Deep Research Mode
const startDeepResearch = async () => {
  if (!chatMessage.value.trim()) {
    chatMessage.value = 'Conduct a deep research analysis on: '
    const input = document.querySelector('.chat-input') as HTMLInputElement
    if (input) {
      input.focus()
      input.setSelectionRange(input.value.length, input.value.length)
    }
    return
  }
  
  await sendMessage('deep_research')
}

const closeChatWindow = async () => {
  showChatWindow.value = false
  console.log('💬 Chat window closed')
}

const sendMessage = async (agentType: string = 'enteract') => {
  if (!chatMessage.value.trim()) return
  
  const userMessage = chatMessage.value
  
  // Add user message to history
  chatHistory.value.push({
    type: 'user',
    message: userMessage,
    timestamp: new Date()
  })
  
  // Clear input immediately
  chatMessage.value = ''
  
  // Auto-scroll to bottom
  setTimeout(() => {
    scrollChatToBottom()
  }, 50)
  
  try {
    // Use selected model or default to gemma3:1b-it-qat
    const modelToUse = selectedModel.value || 'gemma3:1b-it-qat'
    
    // Generate unique session ID for this conversation
    const sessionId = `chat-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
    
    // Add streaming message placeholder with agent indicator
    const streamingMessageIndex = chatHistory.value.length
    const agentEmoji = agentType === 'deep_research' ? '🧠' : agentType === 'vision' ? '👁️' : '🛡️'
    chatHistory.value.push({
      type: 'assistant',
      message: `${agentEmoji}▋`,
      timestamp: new Date()
    })
    
    setTimeout(() => {
      scrollChatToBottom()
    }, 50)
    
    let fullResponse = ''
    let isTyping = true
    
    // Set up event listener for streaming response
    const unlisten = await listen(`ollama-stream-${sessionId}`, (event: any) => {
      const data = event.payload
      
      switch (data.type) {
        case 'start':
          console.log(`🚀 Started ${agentType} streaming from ${data.model}`)
          break
          
        case 'chunk':
          if (data.text) {
            fullResponse += data.text
            // Update the streaming message with accumulated text + cursor
            if (chatHistory.value[streamingMessageIndex]) {
              chatHistory.value[streamingMessageIndex].message = fullResponse + (isTyping ? '▋' : '')
            }
            
            // Auto-scroll to bottom as text streams in
            setTimeout(() => {
              scrollChatToBottom()
            }, 10)
          }
          
          if (data.done) {
            isTyping = false
            // Remove cursor when done
            if (chatHistory.value[streamingMessageIndex]) {
              chatHistory.value[streamingMessageIndex].message = fullResponse
            }
            console.log(`✅ ${agentType} streaming completed. Full response: ${fullResponse}`)
          }
          break
          
        case 'error':
          isTyping = false
          console.error(`${agentType} streaming error:`, data.error)
          // Update message to show error
          if (chatHistory.value[streamingMessageIndex]) {
            chatHistory.value[streamingMessageIndex].message = `❌ Error: ${data.error}`
          }
          break
          
        case 'complete':
          isTyping = false
          console.log(`🎉 ${agentType} streaming session completed`)
          // Clean up listener
          unlisten()
          break
      }
    })
    
    // Route to appropriate agent based on type
    if (agentType === 'deep_research') {
      await invoke('generate_deep_research', {
        prompt: userMessage,
        sessionId: sessionId
      })
    } else {
      // Default to Enteract agent (gemma with security focus)
      await invoke('generate_enteract_agent_response', {
        prompt: userMessage,
        sessionId: sessionId
      })
    }
    
    console.log(`🤖 Started streaming AI response from ${modelToUse}`)
    
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    console.error('Failed to start AI response streaming:', error)
    
    // Add error message to chat
    chatHistory.value.push({
      type: 'assistant',
      message: `❌ Failed to get AI response: ${errorMessage}. Make sure Ollama is running and the model "${selectedModel.value || 'gemma3:1b-it-qat'}" is available.`,
      timestamp: new Date()
    })
  }
  
  // Auto-scroll to bottom
  setTimeout(() => {
    scrollChatToBottom()
  }, 50)
}

const handleChatKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault()
    sendMessage()
  }
}

// Enhanced speech transcription with error handling
const toggleSpeechTranscription = async (event: Event) => {
  // Prevent drag if clicking button
  event.stopPropagation()
  
  try {
    speechError.value = null
    
    // Check browser compatibility first
    if (!compatibilityReport.value.ready) {
      speechError.value = 'Browser not compatible with speech features. ' + compatibilityReport.value.issues.join(', ')
      return
    }
    
    if (store.speechStatus.isRecording) {
      await store.stopSpeechTranscription()
    } else {
      if (!store.speechStatus.isInitialized) {
        await store.initializeSpeechTranscription('base')
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
  // Prevent drag if clicking button
  event.stopPropagation()
  
  try {
    wakeWordError.value = null
    
    // Check browser compatibility first
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
  // Prevent drag if clicking button
  event.stopPropagation()
  
  if (mlEyeTracking.isActive.value) {
    // Stop both ML tracking and window movement
    await mlEyeTracking.stopTracking()
    windowManager.disableGazeControl()
    isGazeControlActive.value = false
    console.log('🛑 ML Eye Tracking and Window Movement stopped')
  } else {
    // Start ML tracking
    await mlEyeTracking.startTracking()
    
    // Wait a moment for initialization
    setTimeout(() => {
      if (mlEyeTracking.isActive.value) {
        // Start window movement with ML gaze data
        startMLGazeWindowMovement()
        isGazeControlActive.value = true
        console.log('🚀 ML Eye Tracking and Window Movement started')
      }
    }, 1000)
  }
}

// Function to connect ML gaze data to window movement
function startMLGazeWindowMovement() {
  // Enable the window manager for gaze control
  windowManager.enableGazeControl()
  
  // Create interval to read ML gaze data and move window
  const updateInterval = setInterval(async () => {
    if (!mlEyeTracking.isActive.value) {
      clearInterval(updateInterval)
      return
    }
    
    const gazeData = mlEyeTracking.currentGaze.value
    if (gazeData && mlEyeTracking.isHighConfidence.value) {
      // Convert gaze screen coordinates to normalized coordinates (-1 to 1)
      const virtualDesktopSize = windowManager.state.value.screenSize
      const screenCenterX = virtualDesktopSize.width / 2
      const screenCenterY = virtualDesktopSize.height / 2
      
      const normalizedGaze = {
        x: (gazeData.x - screenCenterX) / screenCenterX,
        y: (gazeData.y - screenCenterY) / screenCenterY
      }
      
      // Process gaze input through the window manager
      await windowManager.processGazeInput(normalizedGaze)
    }
  }, 33) // 30 FPS window movement
}

// Keyboard shortcuts
const handleKeydown = async (event: KeyboardEvent) => {
  // Ctrl+Shift+E = Toggle ML Eye Tracking
  if (event.ctrlKey && event.shiftKey && event.key === 'E') {
    event.preventDefault()
    await toggleMLEyeTrackingWithMovement(event)
    console.log('⌨️ Keyboard shortcut: ML Eye Tracking toggled')
  }
  
  // Ctrl+Shift+S = Stop all tracking (emergency stop)
  if (event.ctrlKey && event.shiftKey && event.key === 'S') {
    event.preventDefault()
    await mlEyeTracking.stopTracking()
    windowManager.disableGazeControl()
    isGazeControlActive.value = false
    console.log('🚨 Emergency stop: All tracking stopped')
  }
  
  // Ctrl+Shift+C = Toggle Chat Window
  if (event.ctrlKey && event.shiftKey && event.key === 'C') {
    event.preventDefault()
    await toggleChatWindow(event)
    console.log('💬 Keyboard shortcut: Chat window toggled')
  }
  
  // Ctrl+Shift+T = Toggle Transparency Controls
  if (event.ctrlKey && event.shiftKey && event.key === 'T') {
    event.preventDefault()
    toggleTransparencyControls(event)
    console.log('🔍 Keyboard shortcut: Transparency controls toggled')
  }
  
  // Ctrl+Shift+A = Toggle AI Models Window
  if (event.ctrlKey && event.shiftKey && event.key === 'A') {
    event.preventDefault()
    await toggleAIModelsWindow(event)
    console.log('🤖 Keyboard shortcut: AI Models window toggled')
  }
  
  // Escape = Close any open panels
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
  if (isResizing.value) return
  
  const target = event.target as HTMLElement
  const chatWindow = document.querySelector('.chat-window')
  const transparencyPanel = document.querySelector('.transparency-controls-panel')
  const aiModelsPanel = document.querySelector('.ai-models-panel')
  const controlPanel = document.querySelector('.control-panel-glass-bar')
  
  // Close chat window if clicking outside
  if (chatWindow && controlPanel && showChatWindow.value &&
      !chatWindow.contains(target) && 
      !controlPanel.contains(target)) {
    closeChatWindow()
  }
  
  // Close transparency panel if clicking outside
  if (transparencyPanel && controlPanel && showTransparencyControls.value &&
      !transparencyPanel.contains(target) && 
      !controlPanel.contains(target)) {
    closeTransparencyControls()
  }
  
  // Close AI models panel if clicking outside
  if (aiModelsPanel && controlPanel && showAIModelsWindow.value &&
      !aiModelsPanel.contains(target) && 
      !controlPanel.contains(target)) {
    closeAIModelsWindow()
  }
}

onMounted(async () => {
  document.addEventListener('keydown', handleKeydown)
  document.addEventListener('click', handleClickOutside)
  
  // Add drag event listeners
  const controlPanel = document.querySelector('.control-panel-glass-bar') as HTMLElement
  if (controlPanel) {
    controlPanel.addEventListener('mousedown', handleDragStart)
    document.addEventListener('mouseup', handleDragEnd)
  }
  
  // Setup speech transcription event listeners
  setupSpeechTranscriptionListeners()
  
  // Initialize speech transcription system
  await store.initializeSpeechTranscription('base')
  
  // Initialize window size
  await resizeWindow(false, false, false)
  
  // Show keyboard shortcuts in console
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
  document.removeEventListener('mousemove', handleResize)
  document.removeEventListener('mouseup', stopResize)
  
  // Remove speech transcription event listeners
  removeSpeechTranscriptionListeners()
})

// Setup speech transcription event listeners
const setupSpeechTranscriptionListeners = () => {
  window.addEventListener('transcription-started', handleTranscriptionStarted)
  window.addEventListener('transcription-interim', handleTranscriptionInterim)
  window.addEventListener('transcription-final', handleTranscriptionFinal)
  window.addEventListener('transcription-error', handleTranscriptionError)
  window.addEventListener('transcription-stopped', handleTranscriptionStopped)
  window.addEventListener('transcription-complete', handleTranscriptionComplete)
  window.addEventListener('transcription-auto-stopped', handleTranscriptionAutoStopped)
}

const removeSpeechTranscriptionListeners = () => {
  window.removeEventListener('transcription-started', handleTranscriptionStarted)
  window.removeEventListener('transcription-interim', handleTranscriptionInterim)
  window.removeEventListener('transcription-final', handleTranscriptionFinal)
  window.removeEventListener('transcription-error', handleTranscriptionError)
  window.removeEventListener('transcription-stopped', handleTranscriptionStopped)
  window.removeEventListener('transcription-complete', handleTranscriptionComplete)
  window.removeEventListener('transcription-auto-stopped', handleTranscriptionAutoStopped)
}

// Speech transcription event handlers
const handleTranscriptionStarted = (event: Event) => {
  const customEvent = event as CustomEvent
  console.log('🎤 Transcription started', customEvent.detail)
  
  // Auto-open chat window when transcription starts
  if (!showChatWindow.value) {
    showChatWindow.value = true
    setTimeout(() => {
      scrollChatToBottom()
    }, 150)
  }
}

const handleTranscriptionInterim = (event: Event) => {
  const customEvent = event as CustomEvent
  console.log('🎤 Interim transcription', customEvent.detail)
  
  const interimText = customEvent.detail.text || ''
  if (interimText.trim()) {
    // Add or update interim message
    const lastMessage = chatHistory.value[chatHistory.value.length - 1]
    if (lastMessage && lastMessage.type === 'transcription' && lastMessage.isInterim) {
      // Update existing interim message
      lastMessage.message = interimText
    } else {
      // Add new interim message
      chatHistory.value.push({
        type: 'transcription',
        message: interimText,
        timestamp: new Date(),
        isInterim: true,
        confidence: customEvent.detail.confidence || 0.5
      })
    }
    
    setTimeout(() => {
      scrollChatToBottom()
    }, 50)
  }
}

const handleTranscriptionFinal = async (event: Event) => {
  const customEvent = event as CustomEvent
  console.log('🎤 Final transcription', customEvent.detail)
  
  const finalText = customEvent.detail.text || ''
  if (finalText.trim()) {
    // Replace interim message with final one or add new final message
    const lastMessage = chatHistory.value[chatHistory.value.length - 1]
    if (lastMessage && lastMessage.type === 'transcription' && lastMessage.isInterim) {
      // Update interim message to final
      lastMessage.message = finalText
      lastMessage.isInterim = false
      lastMessage.confidence = customEvent.detail.confidence || 0.9
    } else {
      // Add new final message
      chatHistory.value.push({
        type: 'transcription',
        message: finalText,
        timestamp: new Date(),
        isInterim: false,
        confidence: customEvent.detail.confidence || 0.9
      })
    }
    
    setTimeout(() => {
      scrollChatToBottom()
    }, 50)
    
    // Auto-send transcribed text to AI if confidence is high enough
    if ((customEvent.detail.confidence || 0.9) > 0.7) {
      console.log('🎤 Auto-sending transcription to Enteract Agent:', finalText)
      
      // Set the transcribed text as the chat message and send it
      setTimeout(async () => {
        // Temporarily set the message to trigger sendMessage
        chatMessage.value = finalText
        await sendMessage('enteract') // Use Enteract agent for transcriptions
        // Clear it again since sendMessage already clears it
      }, 1000) // Small delay to show the transcription first
    }
  }
}

const handleTranscriptionError = (event: Event) => {
  const customEvent = event as CustomEvent
  console.log('❌ Transcription error', customEvent.detail)
  
  // Add error message to chat
  chatHistory.value.push({
    type: 'assistant',
    message: `❌ Transcription error: ${customEvent.detail.error}`,
    timestamp: new Date()
  })
  
  setTimeout(() => {
    scrollChatToBottom()
  }, 50)
}

const handleTranscriptionStopped = (event: Event) => {
  const customEvent = event as CustomEvent
  console.log('⏹️ Transcription stopped', customEvent.detail)
}

const handleTranscriptionComplete = (event: Event) => {
  const customEvent = event as CustomEvent
  console.log('✅ Transcription complete', customEvent.detail)
}

const handleTranscriptionAutoStopped = (event: Event) => {
  const customEvent = event as CustomEvent
  console.log('🔄 Transcription auto-stopped', customEvent.detail)
  
  // Add system message about auto-stop
  chatHistory.value.push({
    type: 'assistant',
    message: `🔄 Transcription stopped automatically (${customEvent.detail.reason})`,
    timestamp: new Date()
  })
  
  setTimeout(() => {
    scrollChatToBottom()
  }, 50)
}

// Helper function to scroll chat to bottom
const scrollChatToBottom = () => {
  const chatMessages = document.querySelector('.chat-messages')
  if (chatMessages) {
    chatMessages.scrollTop = chatMessages.scrollHeight
  }
}

// Ollama functions
const fetchOllamaStatus = async () => {
  try {
    const status = await invoke<OllamaStatus>('get_ollama_status')
    ollamaStatus.value = status
    return status
  } catch (error) {
    console.error('Failed to get Ollama status:', error)
    ollamaStatus.value = { status: 'error' }
    return { status: 'error' }
  }
}

const fetchOllamaModels = async () => {
  isLoadingModels.value = true
  modelsError.value = null
  
  try {
    const models = await invoke<OllamaModel[]>('get_ollama_models')
    ollamaModels.value = models
    console.log('📋 Fetched Ollama models:', models)
    
    // Auto-select gemma3:1b-it-qat if available and no model is selected
    if (!selectedModel.value) {
      const gemmaModel = models.find(model => 
        model.name.includes('gemma3:1b-it-qat') || 
        model.name.includes('gemma3') ||
        model.name.includes('gemma')
      )
      
      if (gemmaModel) {
        selectedModel.value = gemmaModel.name
        console.log('🎯 Auto-selected Gemma model:', gemmaModel.name)
      } else if (models.length > 0) {
        // Fallback to first available model
        selectedModel.value = models[0].name
        console.log('🎯 Auto-selected first available model:', models[0].name)
      }
    }
    
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    modelsError.value = message
    console.error('Failed to fetch Ollama models:', error)
  } finally {
    isLoadingModels.value = false
  }
}

const pullModel = async (modelName: string) => {
  pullingModel.value = modelName
  
  try {
    const result = await invoke<string>('pull_ollama_model', { modelName })
    console.log('📥 Pull result:', result)
    // Refresh models list after pulling
    setTimeout(() => {
      fetchOllamaModels()
    }, 2000)
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    console.error('Failed to pull model:', error)
    modelsError.value = `Failed to pull ${modelName}: ${message}`
  } finally {
    pullingModel.value = null
  }
}

const deleteModel = async (modelName: string) => {
  if (!confirm(`Are you sure you want to delete the model "${modelName}"?`)) {
    return
  }
  
  deletingModel.value = modelName
  
  try {
    const result = await invoke<string>('delete_ollama_model', { modelName })
    console.log('🗑️ Delete result:', result)
    // Refresh models list after deletion
    await fetchOllamaModels()
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    console.error('Failed to delete model:', error)
    modelsError.value = `Failed to delete ${modelName}: ${message}`
  } finally {
    deletingModel.value = null
  }
}

const formatModelSize = (size: number): string => {
  const gb = size / (1024 * 1024 * 1024)
  return `${gb.toFixed(1)} GB`
}

const getModelDisplayName = (model: OllamaModel): string => {
  return model.name.split(':')[0] || model.name
}

// Simple markdown renderer for basic formatting
const renderMarkdown = (text: string): string => {
  if (!text) return ''
  
  return text
    // Headers
    .replace(/^### (.*$)/gim, '<h3 class="text-lg font-semibold text-white/90 mt-4 mb-2">$1</h3>')
    .replace(/^## (.*$)/gim, '<h2 class="text-xl font-semibold text-white/95 mt-4 mb-2">$1</h2>')
    .replace(/^# (.*$)/gim, '<h1 class="text-2xl font-bold text-white mt-4 mb-3">$1</h1>')
    
    // Bold and italic
    .replace(/\*\*(.*?)\*\*/g, '<strong class="font-semibold text-white">$1</strong>')
    .replace(/\*(.*?)\*/g, '<em class="italic text-white/90">$1</em>')
    
    // Code blocks
    .replace(/```([\s\S]*?)```/g, '<div class="bg-black/30 border border-white/20 rounded-lg p-3 my-2 font-mono text-sm text-green-300 overflow-x-auto">$1</div>')
    .replace(/`(.*?)`/g, '<code class="bg-black/40 px-1.5 py-0.5 rounded text-sm font-mono text-cyan-300">$1</code>')
    
    // Lists
    .replace(/^\* (.*$)/gim, '<li class="ml-4 text-white/85">• $1</li>')
    .replace(/^- (.*$)/gim, '<li class="ml-4 text-white/85">• $1</li>')
    .replace(/^\+ (.*$)/gim, '<li class="ml-4 text-white/85">• $1</li>')
    .replace(/^\d+\. (.*$)/gim, '<li class="ml-4 text-white/85">$1</li>')
    
    // Links
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" class="text-blue-400 hover:text-blue-300 underline" target="_blank" rel="noopener noreferrer">$1</a>')
    
    // Line breaks
    .replace(/\n\n/g, '<br/><br/>')
    .replace(/\n/g, '<br/>')
}
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
            title="Wake Word Detection"
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

    <!-- AI Models Window Section -->
    <Transition name="ai-models-panel">
      <div v-if="showAIModelsWindow" class="ai-models-section">
        <div class="ai-models-panel">
          <div class="panel-header">
            <div class="panel-title">
              <Cog6ToothIcon class="w-4 h-4 text-white/80" />
              <span class="text-sm font-medium text-white/90">AI Settings (Ollama)</span>
              <div class="status-indicator" :class="{
                'text-green-400': ollamaStatus.status === 'running',
                'text-red-400': ollamaStatus.status === 'not_running',
                'text-yellow-400': ollamaStatus.status === 'checking' || ollamaStatus.status === 'error'
              }">
                {{ ollamaStatus.status === 'running' ? '●' : ollamaStatus.status === 'not_running' ? '●' : '●' }}
              </div>
            </div>
            <button @click="closeAIModelsWindow" class="panel-close-btn">
              <XMarkIcon class="w-4 h-4 text-white/70 hover:text-white transition-colors" />
            </button>
          </div>
          
          <div class="panel-content">
            <!-- Ollama Status -->
            <div class="ollama-status">
              <div v-if="ollamaStatus.status === 'running'" class="status-good">
                <span class="text-green-400">● Ollama is running</span>
                <span v-if="ollamaStatus.version" class="text-white/60 text-xs ml-2">v{{ ollamaStatus.version }}</span>
              </div>
              <div v-else-if="ollamaStatus.status === 'not_running'" class="status-error">
                <span class="text-red-400">● Ollama is not running</span>
                <p class="text-white/60 text-xs mt-1">Please start Ollama to manage models</p>
              </div>
              <div v-else-if="ollamaStatus.status === 'checking'" class="status-loading">
                <span class="text-yellow-400">● Checking Ollama status...</span>
              </div>
              <div v-else class="status-error">
                <span class="text-red-400">● Failed to connect to Ollama</span>
              </div>
            </div>
            
            <!-- Models List -->
            <div v-if="ollamaStatus.status === 'running'" class="models-section">
              <div class="models-header">
                <h3 class="text-white/90 font-medium">Available Models</h3>
                <button 
                  @click="fetchOllamaModels" 
                  :disabled="isLoadingModels"
                  class="refresh-btn"
                  title="Refresh Models"
                >
                  <ArrowsPointingOutIcon class="w-4 h-4" :class="{ 'animate-spin': isLoadingModels }" />
                </button>
              </div>
              
              <!-- Error Message -->
              <div v-if="modelsError" class="error-message">
                <span class="text-red-400 text-sm">{{ modelsError }}</span>
                <button @click="modelsError = null" class="ml-2 text-white/60 hover:text-white">×</button>
              </div>
              
              <!-- Loading State -->
              <div v-if="isLoadingModels" class="loading-state">
                <div class="animate-pulse text-white/60">Loading models...</div>
              </div>
              
              <!-- Models List -->
              <div v-else-if="ollamaModels.length > 0" class="models-list">
                <div v-for="model in ollamaModels" :key="model.name" class="model-item">
                  <div class="model-info">
                    <div class="model-name">{{ getModelDisplayName(model) }}</div>
                    <div class="model-details">
                      <span class="model-size">{{ formatModelSize(model.size) }}</span>
                      <span v-if="model.details?.parameter_size" class="model-params">
                        {{ model.details.parameter_size }}
                      </span>
                    </div>
                  </div>
                  
                  <div class="model-actions">
                    <button
                      @click="selectedModel = model.name"
                      :class="{ 'active': selectedModel === model.name }"
                      class="select-btn"
                      title="Select Model"
                    >
                      {{ selectedModel === model.name ? '✓' : '○' }}
                    </button>
                    
                    <button
                      @click="deleteModel(model.name)"
                      :disabled="deletingModel === model.name"
                      class="delete-btn"
                      title="Delete Model"
                    >
                      <TrashIcon v-if="deletingModel !== model.name" class="w-3 h-3" />
                      <div v-else class="w-3 h-3 animate-spin">⟳</div>
                    </button>
                  </div>
                </div>
              </div>
              
              <!-- No Models -->
              <div v-else class="no-models">
                <p class="text-white/60 text-sm">No models available</p>
                <p class="text-white/40 text-xs mt-1">Pull a model to get started</p>
              </div>
              
              <!-- Pull Model Section -->
              <div class="pull-model-section">
                <h4 class="text-white/80 text-sm font-medium mb-2">Pull New Model</h4>
                <div class="popular-models">
                  <button 
                    v-for="modelName in ['gemma3:1b-it-qat', 'qwen2.5vl:3b', 'deepseek-r1:1.5b', 'llama3.2']" 
                    :key="modelName"
                    @click="pullModel(modelName)"
                    :disabled="pullingModel === modelName"
                    class="model-pull-btn"
                    :class="{ 
                      'recommended': modelName === 'gemma3:1b-it-qat',
                      'vision-model': modelName === 'qwen2.5vl:3b',
                      'research-model': modelName === 'deepseek-r1:1.5b'
                    }"
                  >
                    <ArrowDownTrayIcon v-if="pullingModel !== modelName" class="w-3 h-3" />
                    <div v-else class="w-3 h-3 animate-spin">⟳</div>
                    <span>{{ modelName }}</span>
                    <span v-if="modelName === 'gemma3:1b-it-qat'" class="recommended-badge">Enteract Agent</span>
                    <span v-if="modelName === 'qwen2.5vl:3b'" class="vision-badge">Vision</span>
                    <span v-if="modelName === 'deepseek-r1:1.5b'" class="research-badge">Research</span>
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>

    <!-- Chat Window Section -->
    <Transition name="chat-window">
      <div v-if="showChatWindow" class="chat-window-section">
        <div 
          class="chat-window" 
          :class="{ 'resizing': isResizing }"
          :style="{ 
            width: chatWindowSize.width + 'px', 
            height: chatWindowSize.height + 'px' 
          }"
        >
          <!-- Chat Header with Resize Indicator -->
          <div class="chat-header">
            <div class="chat-title">
              <CommandLineIcon class="w-4 h-4 text-white/80" />
              <span class="text-sm font-medium text-white/90">AI Assistant</span>
              <div class="model-indicator" v-if="selectedModel">
                <span class="text-xs text-green-400">{{ selectedModel.split(':')[0] || selectedModel }}</span>
              </div>
              <div class="resize-indicator">
                <ArrowsPointingOutIcon class="w-3 h-3 text-white/50" />
                <span class="text-xs text-white/50">{{ chatWindowSize.width }}×{{ chatWindowSize.height }}</span>
              </div>
            </div>
            <button @click="closeChatWindow" class="chat-close-btn">
              <XMarkIcon class="w-4 h-4 text-white/70 hover:text-white transition-colors" />
            </button>
          </div>
          
          <div class="chat-messages" ref="chatMessages" 
               :style="{ height: (chatWindowSize.height - 120) + 'px' }">
            <div v-if="chatHistory.length === 0" class="chat-empty">
              <CommandLineIcon class="w-6 h-6 text-white/40 mb-2" />
              <p class="text-white/60 text-sm">Start a conversation with your AI assistant</p>
            </div>
            
            <div v-for="(message, index) in chatHistory" :key="index" class="chat-message"
                 :class="{ 
                   'user': message.type === 'user', 
                   'assistant': message.type === 'assistant',
                   'transcription': message.type === 'transcription'
                 }">
              <div class="message-bubble">
                <!-- Transcription messages -->
                <div v-if="message.type === 'transcription'" class="transcription-message">
                  <!-- Interim transcription (thought stream) -->
                  <div v-if="message.isInterim" class="interim-transcription">
                    <span class="interim-icon">💭</span>
                    <span class="interim-text">{{ message.message }}</span>
                    <span class="interim-dots">...</span>
                  </div>
                  <!-- Final transcription -->
                  <div v-else class="final-transcription">
                    <div class="transcription-content">
                      <span class="transcription-icon">🎤</span>
                      <span class="transcription-text">{{ message.message }}</span>
                    </div>
                    <div v-if="message.confidence" class="confidence-indicator">
                      {{ Math.round(message.confidence * 100) }}%
                    </div>
                  </div>
                </div>
                
                <!-- Regular user/assistant messages -->
                <div v-else class="message-text">
                  <!-- Streaming text with cursor -->
                  <template v-if="message.message.endsWith('▋')">
                    <div v-html="renderMarkdown(message.message.slice(0, -1))"></div><span class="streaming-cursor">▋</span>
                  </template>
                  <!-- Regular completed text with markdown -->
                  <div v-else v-html="renderMarkdown(message.message)"></div>
                </div>
                
                <span class="message-time">{{ message.timestamp.toLocaleTimeString() }}</span>
              </div>
            </div>
          </div>
          
          <!-- Agent Action Buttons -->
          <div class="agent-actions">
            <button @click="takeScreenshotAndAnalyze" class="agent-btn vision-btn" title="Analyze Screen">
              <CameraIcon class="w-4 h-4" />
              <span>Analyze Screen</span>
            </button>
            
            <button @click="startDeepResearch" class="agent-btn research-btn" title="Deep Research Mode">
              <MagnifyingGlassIcon class="w-4 h-4" />
              <span>Research</span>
            </button>
          </div>
          
          <div class="chat-input-container">
            <input 
              v-model="chatMessage"
              @keydown="handleChatKeydown"
              class="chat-input"
              placeholder="Ask the Enteract Agent..."
              type="text"
            />
            <button @click="() => sendMessage()" class="chat-send-btn" :disabled="!chatMessage.trim()">
              <ShieldCheckIcon class="w-4 h-4" />
            </button>
          </div>

          <!-- Resize Handles -->
          <div class="resize-handles">
            <!-- Corner handles -->
            <div class="resize-handle corner top-left" @mousedown="startResize($event, 'top-left')"></div>
            <div class="resize-handle corner top-right" @mousedown="startResize($event, 'top-right')"></div>
            <div class="resize-handle corner bottom-left" @mousedown="startResize($event, 'bottom-left')"></div>
            <div class="resize-handle corner bottom-right" @mousedown="startResize($event, 'bottom-right')"></div>
            
            <!-- Edge handles -->
            <div class="resize-handle edge top" @mousedown="startResize($event, 'top')"></div>
            <div class="resize-handle edge bottom" @mousedown="startResize($event, 'bottom')"></div>
            <div class="resize-handle edge left" @mousedown="startResize($event, 'left')"></div>
            <div class="resize-handle edge right" @mousedown="startResize($event, 'right')"></div>
          </div>
        </div>
      </div>
    </Transition>
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

.chat-window-section {
  @apply w-full flex justify-center;
  padding: 0 8px 8px 8px;
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

/* Chat Window Styles */
.chat-window {
  @apply rounded-2xl overflow-hidden relative;
  pointer-events: auto;
  min-width: 450px;
  min-height: 400px;
  max-width: 800px;
  max-height: 1200px;
  
  /* Same glass effect as control panel with darker background */
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

  transition: all 0.2s ease;
}

.chat-window.resizing {
  box-shadow: 
    0 25px 70px rgba(0, 0, 0, 0.5),
    0 12px 30px rgba(0, 0, 0, 0.3),
    inset 0 1px 0 rgba(255, 255, 255, 0.4),
    inset 0 -1px 0 rgba(0, 0, 0, 0.1);
  border-color: rgba(255, 255, 255, 0.35);
}

.chat-header {
  @apply flex items-center justify-between px-4 py-3 border-b border-white/10;
}

.chat-title {
  @apply flex items-center gap-2;
}

.model-indicator {
  @apply flex items-center gap-1 ml-2 px-2 py-1 rounded-md bg-green-500/20 border border-green-400/30;
}

.resize-indicator {
  @apply flex items-center gap-1 ml-2 px-2 py-1 rounded-md bg-white/5;
}

.chat-close-btn {
  @apply rounded-full p-1 hover:bg-white/10 transition-colors;
}

.chat-messages {
  @apply flex-1 overflow-y-auto px-4 py-3;
  scrollbar-width: thin;
  scrollbar-color: rgba(255, 255, 255, 0.2) transparent;
}

.chat-messages::-webkit-scrollbar {
  width: 4px;
}

.chat-messages::-webkit-scrollbar-track {
  background: transparent;
}

.chat-messages::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
}

.chat-empty {
  @apply flex flex-col items-center justify-center h-full text-center;
}

.chat-message {
  @apply mb-4;
}

.chat-message.user {
  @apply flex justify-end;
}

.chat-message.assistant {
  @apply flex justify-start;
}

.chat-message.transcription {
  @apply flex justify-start;
}

.message-bubble {
  @apply max-w-xs px-3 py-2 rounded-2xl;
}

.chat-message.user .message-bubble {
  @apply bg-blue-500/80 text-white;
}

.chat-message.assistant .message-bubble {
  @apply bg-white/15 text-white/90;
}

.chat-message.transcription .message-bubble {
  @apply bg-purple-500/20 text-white/90 border border-purple-400/30;
}

.transcription-message {
  @apply w-full;
}

.interim-transcription {
  @apply flex items-center gap-2;
}

.interim-icon {
  @apply text-orange-300 text-sm;
}

.interim-text {
  @apply italic text-orange-200;
}

.interim-dots {
  @apply text-orange-400 animate-pulse font-bold;
}

.final-transcription {
  @apply flex flex-col gap-1;
}

.transcription-content {
  @apply flex items-center gap-2;
}

.transcription-icon {
  @apply text-green-400 text-sm;
}

.transcription-text {
  @apply text-white/90;
}

.confidence-indicator {
  @apply text-xs text-white/60 mt-1;
}

.message-text {
  @apply text-sm leading-relaxed;
}

.message-time {
  @apply text-xs opacity-70 mt-1 block;
}

.agent-actions {
  @apply flex gap-2 px-4 py-2 border-t border-white/10;
}

.agent-btn {
  @apply flex items-center gap-2 px-3 py-2 rounded-lg text-xs font-medium transition-all duration-200;
  @apply bg-white/5 border border-white/15 text-white/80;
  @apply hover:bg-white/10 hover:border-white/30 hover:text-white;
  @apply hover:scale-105 hover:shadow-lg;
}

.vision-btn {
  @apply bg-purple-500/10 border-purple-400/20 text-purple-300;
  @apply hover:bg-purple-500/20 hover:border-purple-400/40 hover:text-purple-200;
}

.research-btn {
  @apply bg-blue-500/10 border-blue-400/20 text-blue-300;
  @apply hover:bg-blue-500/20 hover:border-blue-400/40 hover:text-blue-200;
}

.chat-input-container {
  @apply flex items-center gap-2 px-4 py-3 border-t border-white/10;
}

.chat-input {
  @apply flex-1 bg-white/10 border border-white/20 rounded-full px-4 py-2 text-white placeholder-white/50 focus:outline-none focus:border-white/40 transition-colors;
}

.chat-send-btn {
  @apply bg-blue-500/80 hover:bg-blue-500 disabled:bg-white/10 disabled:opacity-50 rounded-full p-2 transition-colors;
  color: white;
}

.chat-send-btn:disabled {
  cursor: not-allowed;
}

/* Resize Handles */
.resize-handles {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  pointer-events: none;
}

.resize-handle {
  position: absolute;
  pointer-events: auto;
  transition: background-color 0.2s ease;
}

/* Corner handles */
.resize-handle.corner {
  width: 16px;
  height: 16px;
  background: rgba(255, 255, 255, 0.15);
  border-radius: 50%;
  border: 1px solid rgba(255, 255, 255, 0.3);
}

.resize-handle.corner:hover {
  background: rgba(255, 255, 255, 0.3);
  border-color: rgba(255, 255, 255, 0.4);
}

.resize-handle.top-left {
  top: -8px;
  left: -8px;
  cursor: nw-resize;
}

.resize-handle.top-right {
  top: -8px;
  right: -8px;
  cursor: ne-resize;
}

.resize-handle.bottom-left {
  bottom: -8px;
  left: -8px;
  cursor: sw-resize;
}

.resize-handle.bottom-right {
  bottom: -8px;
  right: -8px;
  cursor: se-resize;
}

/* Edge handles */
.resize-handle.edge {
  background: transparent;
}

.resize-handle.edge:hover {
  background: rgba(255, 255, 255, 0.1);
}

.resize-handle.top {
  top: -4px;
  left: 16px;
  right: 16px;
  height: 8px;
  cursor: n-resize;
}

.resize-handle.bottom {
  bottom: -4px;
  left: 16px;
  right: 16px;
  height: 8px;
  cursor: s-resize;
}

.resize-handle.left {
  left: -4px;
  top: 16px;
  bottom: 16px;
  width: 8px;
  cursor: w-resize;
}

.resize-handle.right {
  right: -4px;
  top: 16px;
  bottom: 16px;
  width: 8px;
  cursor: e-resize;
}

/* Chat Window Transitions */
.chat-window-enter-active,
.chat-window-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.chat-window-enter-from {
  opacity: 0;
  transform: translateY(-10px) scale(0.95);
}

.chat-window-leave-to {
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

/* Blinking cursor animation for streaming responses */
@keyframes blink {
  0%, 50% {
    opacity: 1;
  }
  51%, 100% {
    opacity: 0;
  }
}

.message-text {
  @apply text-sm leading-relaxed;
}

/* Style for streaming cursor */
.streaming-cursor {
  animation: blink 1s infinite;
  color: #60a5fa;
  font-weight: bold;
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

/* Prevent text selection during resize */
.chat-window.resizing {
  user-select: none;
}

.chat-window.resizing * {
  user-select: none;
}

/* AI Models Window */
.ai-models-section {
  @apply w-full flex justify-center;
  padding: 0 8px 8px 8px;
  background: transparent;
}

.ai-models-panel {
  @apply rounded-2xl overflow-hidden;
  width: 420px;
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

.status-indicator {
  @apply ml-2 text-xs;
}

.ollama-status {
  @apply mb-4;
}

.models-section {
  @apply mt-4;
}

.models-header {
  @apply flex items-center justify-between mb-3;
}

.refresh-btn {
  @apply p-1 rounded-lg bg-white/10 hover:bg-white/20 transition-colors text-white/70 hover:text-white;
}

.error-message {
  @apply flex items-center justify-between bg-red-500/20 border border-red-400/30 rounded-lg p-2 mb-3;
}

.loading-state {
  @apply p-4 text-center;
}

.models-list {
  @apply space-y-2 mb-4 max-h-48 overflow-y-auto;
  scrollbar-width: thin;
  scrollbar-color: rgba(255, 255, 255, 0.2) transparent;
}

.models-list::-webkit-scrollbar {
  width: 4px;
}

.models-list::-webkit-scrollbar-track {
  background: transparent;
}

.models-list::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
}

.model-item {
  @apply flex items-center justify-between p-3 bg-white/5 rounded-lg border border-white/10 hover:bg-white/10 transition-colors;
}

.model-info {
  @apply flex-1;
}

.model-name {
  @apply text-white/90 font-medium text-sm;
}

.model-details {
  @apply flex items-center gap-2 mt-1;
}

.model-size {
  @apply text-white/60 text-xs;
}

.model-params {
  @apply text-white/60 text-xs px-1.5 py-0.5 bg-white/10 rounded-md;
}

.model-actions {
  @apply flex items-center gap-2;
}

.select-btn {
  @apply w-6 h-6 rounded-full border border-white/30 text-xs flex items-center justify-center hover:bg-white/10 transition-colors;
}

.select-btn.active {
  @apply bg-green-500/80 border-green-400 text-white;
}

.delete-btn {
  @apply p-1.5 rounded-lg bg-red-500/20 hover:bg-red-500/40 transition-colors text-red-400 hover:text-red-300;
}

.delete-btn:disabled {
  @apply opacity-50 cursor-not-allowed;
}

.no-models {
  @apply text-center p-4 bg-white/5 rounded-lg border border-white/10;
}

.pull-model-section {
  @apply mt-4 pt-4 border-t border-white/10;
}

.popular-models {
  @apply grid grid-cols-2 gap-2;
}

.model-pull-btn {
  @apply flex items-center gap-2 p-2 bg-blue-500/20 hover:bg-blue-500/40 rounded-lg border border-blue-400/30 text-blue-300 hover:text-blue-200 transition-colors text-sm;
}

.model-pull-btn:disabled {
  @apply opacity-50 cursor-not-allowed;
}

.model-pull-btn.recommended {
  @apply bg-green-500/30 border-green-400/50 text-green-200;
}

.recommended-badge {
  @apply text-xs bg-green-400/80 text-green-900 px-1 py-0.5 rounded-md font-medium;
}

.vision-badge {
  @apply text-xs bg-purple-400/80 text-purple-900 px-1 py-0.5 rounded-md font-medium;
}

.research-badge {
  @apply text-xs bg-blue-400/80 text-blue-900 px-1 py-0.5 rounded-md font-medium;
}

.model-pull-btn.vision-model {
  @apply bg-purple-500/30 border-purple-400/50 text-purple-200;
}

.model-pull-btn.research-model {
  @apply bg-blue-500/30 border-blue-400/50 text-blue-200;
}

/* AI Models Panel Transitions */
.ai-models-panel-enter-active,
.ai-models-panel-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.ai-models-panel-enter-from {
  opacity: 0;
  transform: translateY(-10px) scale(0.95);
}

.ai-models-panel-leave-to {
  opacity: 0;
  transform: translateY(-10px) scale(0.95);
}
</style> 