import { ref, computed, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { v4 as uuidv4 } from 'uuid'
import type { ChatMessage, ChatSession, SaveChatsPayload, LoadChatsResponse } from '../types'

let messageIdCounter = 1

// Create shared singleton state that persists across all component instances
const sharedChatState = {
  chatSessions: ref<ChatSession[]>([]),
  currentChatId: ref<string | null>(null),
  isInitialized: ref(false)
}

export const useChatManagement = (selectedModel: string | null, scrollChatToBottom: () => void) => {
  const chatMessage = ref('')
  const fileInput = ref<HTMLInputElement>()
  
  // Use the shared state instead of creating new instances
  const { chatSessions, currentChatId, isInitialized } = sharedChatState
  
  // Computed property for current chat history
  const currentChatHistory = computed(() => {
    if (!currentChatId.value) return []
    const currentSession = chatSessions.value.find(session => session.id === currentChatId.value)
    const history = currentSession?.history || []
    console.log(`🔍 [SHARED STATE] Current chat history computed: ${history.length} messages for chat ${currentChatId.value}`)
    return history
  })
  
  // Get current chat session
  const currentChatSession = computed(() => {
    if (!currentChatId.value) return null
    return chatSessions.value.find(session => session.id === currentChatId.value) || null
  })

  // Persistence and session management
  const CHATS_STORAGE_KEY = 'user_chat_sessions.json'
  
  // Debounce utility
  const debounce = (func: Function, delay: number) => {
    let timeoutId: number
    return (...args: any[]) => {
      clearTimeout(timeoutId)
      timeoutId = window.setTimeout(() => func.apply(null, args), delay)
    }
  }
  
  // Save all chats to backend
  const saveAllChats = async () => {
    try {
      const payload: SaveChatsPayload = { chats: chatSessions.value }
      await invoke('save_chat_sessions', { payload })
      console.log('✅ Chat sessions saved successfully')
    } catch (error) {
      console.error('❌ Failed to save chat sessions:', error)
    }
  }
  
  // Debounced save function (1000ms delay)
  const debouncedSaveChats = debounce(saveAllChats, 1000)
  
  // Load all chats from backend
  const loadAllChats = async () => {
    try {
      const response: LoadChatsResponse = await invoke('load_chat_sessions')
      if (response.chats && response.chats.length > 0) {
        chatSessions.value = response.chats
        // Set current chat to most recently updated, or first one
        const sortedByUpdated = [...response.chats].sort((a, b) => 
          new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime()
        )
        currentChatId.value = sortedByUpdated[0].id
        console.log(`✅ Loaded ${response.chats.length} chat sessions`)
      } else {
        // No chats exist, create a new one
        createNewChat()
        console.log('📝 No existing chats found, created new chat session')
      }
    } catch (error) {
      console.error('❌ Failed to load chat sessions:', error)
      // Fallback: create new chat
      createNewChat()
    }
  }
  
  // Session management functions
  const createNewChat = (initialMessage?: ChatMessage) => {
    const newChatId = uuidv4()
    const now = new Date().toISOString()
    
    const newSession: ChatSession = {
      id: newChatId,
      title: 'New Chat',
      history: initialMessage ? [initialMessage] : [],
      createdAt: now,
      updatedAt: now,
      modelId: selectedModel || undefined
    }
    
    chatSessions.value.unshift(newSession)
    currentChatId.value = newChatId
    console.log(`📝 [SHARED STATE] Created new chat session: ${newChatId}`)
    console.log(`📝 [SHARED STATE] Current chat ID updated to: ${newChatId}`)
    console.log(`📝 [SHARED STATE] Total sessions: ${chatSessions.value.length}`)
  }
  
  const switchChat = (chatId: string) => {
    const chatExists = chatSessions.value.some(session => session.id === chatId)
    if (chatExists) {
      const oldChatId = currentChatId.value
      currentChatId.value = chatId
      console.log(`🔄 [SHARED STATE] Switched from chat: ${oldChatId} to chat: ${chatId}`)
      
      // Find the session and log its state
      const session = chatSessions.value.find(s => s.id === chatId)
      if (session) {
        console.log(`🔄 [SHARED STATE] New chat has ${session.history.length} messages`)
      }
      
      setTimeout(() => {
        scrollChatToBottom()
      }, 100)
    } else {
      console.error('❌ Chat not found:', chatId)
    }
  }
  
  const deleteChat = (chatId: string) => {
    const chatIndex = chatSessions.value.findIndex(session => session.id === chatId)
    if (chatIndex !== -1) {
      const wasCurrentChat = chatId === currentChatId.value
      chatSessions.value.splice(chatIndex, 1)
      
      // If deleted chat was current, switch to another or create new
      if (wasCurrentChat) {
        if (chatSessions.value.length > 0) {
          currentChatId.value = chatSessions.value[0].id
        } else {
          createNewChat()
        }
      }
      
      // Immediately save after deletion (not debounced)
      saveAllChats()
      console.log(`🗑️ [SHARED STATE] Deleted chat: ${chatId}`)
      if (wasCurrentChat) {
        console.log(`🔄 [SHARED STATE] Switched to ${currentChatId.value} after deletion`)
      }
    }
  }
  
  const renameChat = (chatId: string, newTitle: string) => {
    const session = chatSessions.value.find(s => s.id === chatId)
    if (session) {
      session.title = newTitle
      session.updatedAt = new Date().toISOString()
      console.log(`✏️ Renamed chat ${chatId} to: ${newTitle}`)
    }
  }
  
  const clearChat = () => {
    if (currentChatSession.value) {
      currentChatSession.value.history = []
      currentChatSession.value.updatedAt = new Date().toISOString()
      console.log(`🧹 Cleared chat: ${currentChatId.value}`)
    }
  }
  
  // Helper function to add message to current chat
  const addMessageToCurrentChat = (message: ChatMessage) => {
    // Ensure we have an active chat session
    if (!currentChatId.value || !currentChatSession.value) {
      createNewChat()
    }
    
    if (currentChatSession.value) {
      currentChatSession.value.history.push(message)
      currentChatSession.value.updatedAt = new Date().toISOString()
      
      // Auto-title: If this is the first user message in a new chat, use it as title
      if (currentChatSession.value.title === 'New Chat' && 
          message.sender === 'user' && 
          currentChatSession.value.history.length === 1) {
        const title = message.text.length > 50 
          ? message.text.substring(0, 47) + '...'
          : message.text
        currentChatSession.value.title = title
      }
    }
  }

  // Token estimation utility (~4 characters per token heuristic)
  const estimateTokens = (text: string): number => {
    if (!text) return 0
    return Math.ceil(text.length / 4)
  }

  // Context truncation logic to fit within token limits
  const getLimitedContext = (history: ChatMessage[], maxTokens: number): { role: string; content: string }[] => {
    if (!history || history.length === 0) return []
    
    const context: { role: string; content: string }[] = []
    let currentTokens = 0
    
    // Step 1: Extract and preserve all system messages at the beginning
    const systemMessages = history.filter(msg => msg.sender === 'system')
    const nonSystemMessages = history.filter(msg => msg.sender !== 'system')
    
    // Add system messages first (they should always be preserved)
    for (const msg of systemMessages) {
      const tokens = estimateTokens(msg.text)
      currentTokens += tokens
      context.push({
        role: 'system',
        content: msg.text
      })
    }
    
    // Step 2: Add a truncation indicator if we'll need to truncate
    const totalTokensNeeded = history.reduce((sum, msg) => sum + estimateTokens(msg.text), 0)
    const needsTruncation = totalTokensNeeded > maxTokens
    
    if (needsTruncation) {
      const truncationMessage = '... (earlier conversation history truncated to fit context limit) ...'
      const truncationTokens = estimateTokens(truncationMessage)
      currentTokens += truncationTokens
      context.push({
        role: 'system',
        content: truncationMessage
      })
    }
    
    // Step 3: Iterate backwards through non-system messages to keep most recent
    const messagesToInclude: ChatMessage[] = []
    
    for (let i = nonSystemMessages.length - 1; i >= 0; i--) {
      const message = nonSystemMessages[i]
      const messageTokens = estimateTokens(message.text)
      
      // Check if adding this message would exceed the limit
      if (currentTokens + messageTokens > maxTokens) {
        break // Stop here to avoid exceeding the limit
      }
      
      // Add to the beginning of our array (since we're going backwards)
      messagesToInclude.unshift(message)
      currentTokens += messageTokens
    }
    
    // Step 4: Convert messages to Ollama API format and add to context
    for (const message of messagesToInclude) {
      let role: string
      switch (message.sender) {
        case 'user':
          role = 'user'
          break
        case 'assistant':
          role = 'assistant'
          break
        case 'transcription':
          role = 'user' // Treat transcriptions as user input
          break
        default:
          role = 'user' // Default fallback
      }
      
      context.push({
        role,
        content: message.text
      })
    }
    
    console.log(`📊 Context truncation: ${history.length} messages → ${messagesToInclude.length + systemMessages.length} messages, ~${currentTokens} tokens (limit: ${maxTokens})`)
    
    return context
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

  // Screen Analysis and Vision
  const takeScreenshotAndAnalyze = async (showChatWindow: any) => {
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
      
      // Add screen analysis message to current chat
      addMessageToCurrentChat({
        id: messageIdCounter++,
        sender: 'user',
        text: `🔍 Screen captured for analysis (${screenshot.width}×${screenshot.height})`,
        timestamp: new Date(),
        messageType: 'text'
      })
      
      // Generate unique session ID
      const sessionId = `vision-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
      
      // Add streaming message placeholder with more detailed status
      const streamingMessageIndex = currentChatHistory.value.length
      addMessageToCurrentChat({
        id: messageIdCounter++,
        sender: 'assistant',
        text: '🔍 Initializing Qwen vision model for analysis▋',
        timestamp: new Date(),
        messageType: 'text'
      })
      
      setTimeout(() => {
        scrollChatToBottom()
      }, 50)
      
      let fullResponse = ''
      let isTyping = true
      let hasStarted = false
      
      // Set up event listener for vision analysis
      const unlisten = await listen(`ollama-stream-${sessionId}`, (event: any) => {
        const data = event.payload
        
        switch (data.type) {
          case 'start':
            hasStarted = true
            console.log(`👁️ Started vision analysis with ${data.model}`)
            if (currentChatHistory.value[streamingMessageIndex]) {
              currentChatHistory.value[streamingMessageIndex].text = `👁️ Qwen ${data.model} is analyzing your screenshot▋`
            }
            setTimeout(() => {
              scrollChatToBottom()
            }, 10)
            break
            
          case 'chunk':
            if (data.text) {
              // First chunk - update status
              if (fullResponse === '') {
                if (currentChatHistory.value[streamingMessageIndex]) {
                  currentChatHistory.value[streamingMessageIndex].text = '👁️ Vision Analysis:\n\n'
                }
              }
              
              fullResponse += data.text
              if (currentChatHistory.value[streamingMessageIndex]) {
                currentChatHistory.value[streamingMessageIndex].text = '👁️ Vision Analysis:\n\n' + fullResponse + (isTyping ? '▋' : '')
              }
              
              setTimeout(() => {
                scrollChatToBottom()
              }, 10)
            }
            
            if (data.done) {
              isTyping = false
              if (currentChatHistory.value[streamingMessageIndex]) {
                currentChatHistory.value[streamingMessageIndex].text = '👁️ Vision Analysis:\n\n' + fullResponse
              }
              console.log('✅ Vision analysis streaming completed')
            }
            break
            
          case 'error':
            isTyping = false
            console.error('Vision analysis error:', data.error)
            if (currentChatHistory.value[streamingMessageIndex]) {
              if (data.error.includes('qwen2.5vl:3b')) {
                currentChatHistory.value[streamingMessageIndex].text = `❌ Vision model not found. Please install qwen2.5vl:3b first:\n\n\`\`\`bash\nollama pull qwen2.5vl:3b\n\`\`\``
              } else {
                currentChatHistory.value[streamingMessageIndex].text = `❌ Vision analysis error: ${data.error}`
              }
            }
            break
            
          case 'complete':
            isTyping = false
            console.log('🎉 Vision analysis session completed')
            unlisten()
            break
        }
      })
      
      // Add a timeout to show model loading if it takes too long
      const loadingTimeout = setTimeout(() => {
        if (!hasStarted && currentChatHistory.value[streamingMessageIndex]) {
          currentChatHistory.value[streamingMessageIndex].text = '🔄 Loading Qwen vision model (this may take a moment for the first run)▋'
          setTimeout(() => {
            scrollChatToBottom()
          }, 10)
        }
      }, 2000)
      
      // Start vision analysis
      await invoke('generate_vision_analysis', {
        prompt: 'Please analyze this screenshot in detail.',
        imageBase64: screenshot.image_base64,
        sessionId: sessionId
      })
      
      // Clear the loading timeout
      clearTimeout(loadingTimeout)
      
    } catch (error) {
      console.error('Failed to analyze screen:', error)
      
      // More detailed error messages
      const errorString = error instanceof Error ? error.message : String(error)
      let errorMessage = `❌ Failed to analyze screen: ${errorString}`
      if (errorString.includes('connection refused') || errorString.includes('ECONNREFUSED')) {
        errorMessage = `❌ Cannot connect to Ollama. Please make sure Ollama is running:\n\n\`\`\`bash\nollama serve\n\`\`\``
      } else if (errorString.includes('model') && errorString.includes('not found')) {
        errorMessage = `❌ Vision model not available. Install with:\n\n\`\`\`bash\nollama pull qwen2.5vl:3b\n\`\`\``
      }
      
      addMessageToCurrentChat({
        id: messageIdCounter++,
        sender: 'assistant',
        text: errorMessage,
        timestamp: new Date(),
        messageType: 'text'
      })
    }
  }

  // Deep Research Mode
  const startDeepResearch = async (showChatWindow: any) => {
    // Auto-open chat window if not open
    if (!showChatWindow.value) {
      showChatWindow.value = true
      console.log('💬 Chat window auto-opened for deep research')
      setTimeout(() => {
        scrollChatToBottom()
      }, 150)
    }
    
    // Add deep research message to current chat
    addMessageToCurrentChat({
      id: messageIdCounter++,
      sender: 'user',
      text: '🧠 Deep Research Mode activated - I will thoroughly research your next question.',
      timestamp: new Date(),
      messageType: 'text'
    })
    
    setTimeout(() => {
      scrollChatToBottom()
    }, 50)
  }

  const startConversationalAgent = async (showChatWindow: any) => {
    // Auto-open chat window if not open
    if (!showChatWindow.value) {
      showChatWindow.value = true
      console.log('💬 Chat window auto-opened for conversational agent')
      setTimeout(() => {
        scrollChatToBottom()
      }, 150)
    }
    
    // Add conversational agent message to current chat
    addMessageToCurrentChat({
      id: messageIdCounter++,
      sender: 'user',
      text: '🤖 Conversational AI Agent activated - Ready for natural conversation.',
      timestamp: new Date(),
      messageType: 'text'
    })
    
    setTimeout(() => {
      scrollChatToBottom()
    }, 50)
  }

  const startCodingAgent = async (showChatWindow: any) => {
    // Auto-open chat window if not open
    if (!showChatWindow.value) {
      showChatWindow.value = true
      console.log('💬 Chat window auto-opened for coding agent')
      setTimeout(() => {
        scrollChatToBottom()
      }, 150)
    }
    
    // Add coding agent message to current chat
    addMessageToCurrentChat({
      id: messageIdCounter++,
      sender: 'user',
      text: '💻 Coding Agent activated - Ready to help with programming tasks.',
      timestamp: new Date(),
      messageType: 'text'
    })
    
    setTimeout(() => {
      scrollChatToBottom()
    }, 50)
  }

  const startComputerUseAgent = async (showChatWindow: any) => {
    // Auto-open chat window if not open
    if (!showChatWindow.value) {
      showChatWindow.value = true
      console.log('💬 Chat window auto-opened for computer use agent')
      setTimeout(() => {
        scrollChatToBottom()
      }, 150)
    }
    
    // Add computer use agent message to current chat
    addMessageToCurrentChat({
      id: messageIdCounter++,
      sender: 'user',
      text: '🖥️ Computer Use Agent activated - Ready to assist with computer tasks.',
      timestamp: new Date(),
      messageType: 'text'
    })
    
    setTimeout(() => {
      scrollChatToBottom()
    }, 50)
  }

  // Send message function
  const sendMessage = async (agentType: string = 'enteract') => {
    // Ensure we have an active chat session
    if (!currentChatId.value || !currentChatSession.value) {
      createNewChat()
    }
    
    if (!chatMessage.value.trim()) return
    
    const userMessage = chatMessage.value.trim()
    chatMessage.value = ''
    
    console.log(`🤖 Sending message with ${agentType} agent:`, userMessage)
    
    // Add user message to current chat
    addMessageToCurrentChat({
      id: messageIdCounter++,
      sender: 'user',
      text: userMessage,
      timestamp: new Date(),
      messageType: 'text'
    })
    
    try {
      // Generate unique session ID for streaming
      const sessionId = `chat-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
      
      // Get appropriate model and agent name
      let modelToUse = selectedModel || 'gemma3:1b-it-qat'
      let agentName = 'Enteract AI'
      
      if (agentType === 'deep_research') {
        modelToUse = 'deepseek-r1:1.5b'
        agentName = 'Deep Research AI'
      }
      
      // Add streaming response placeholder
      const streamingMessageIndex = currentChatHistory.value.length
      addMessageToCurrentChat({
        id: messageIdCounter++,
        sender: 'assistant',
        text: `🤖 ${agentName} is thinking▋`,
        timestamp: new Date(),
        messageType: 'text'
      })
      
      setTimeout(() => {
        scrollChatToBottom()
      }, 50)
      
      // Track streaming state
      let fullResponse = ''
      let actualResponse = ''
      let thinkingContent = ''
      let isTyping = true
      let hasStarted = false
      let isInThinking = false
      
      // Set up streaming listener
      const unlisten = await listen(`ollama-stream-${sessionId}`, (event: any) => {
        const data = event.payload
        
        switch (data.type) {
          case 'start':
            hasStarted = true
            console.log(`🤖 Started ${agentType} response with ${data.model}`)
            if (currentChatHistory.value[streamingMessageIndex]) {
              currentChatHistory.value[streamingMessageIndex].text = `🤖 ${agentName} (${data.model})▋`
            }
            setTimeout(() => {
              scrollChatToBottom()
            }, 10)
            break
            
          case 'chunk':
            if (data.text) {
              fullResponse += data.text
              
              // For deep research, handle thinking vs response separately
              if (agentType === 'deep_research') {
                // Check for thinking tags
                if (data.text.includes('<think>')) {
                  isInThinking = true
                } else if (data.text.includes('</think>')) {
                  isInThinking = false
                }
                
                if (isInThinking || (fullResponse.includes('<think>') && !fullResponse.includes('</think>'))) {
                  // We're in thinking mode
                  thinkingContent += data.text.replace(/<\/?think>/g, '')
                } else {
                  // We're in response mode
                  actualResponse += data.text.replace(/<\/?think>/g, '')
                }
              } else {
                // For other agents, just accumulate normally
                actualResponse += data.text
              }
              
              // Update the message in real-time
              if (currentChatHistory.value[streamingMessageIndex]) {
                const displayText = agentType === 'deep_research' 
                  ? `🧠 Deep Research AI:\n\n${actualResponse}${isTyping ? '▋' : ''}`
                  : `🤖 ${agentName}:\n\n${actualResponse}${isTyping ? '▋' : ''}`
                currentChatHistory.value[streamingMessageIndex].text = displayText
              }
              
              setTimeout(() => {
                scrollChatToBottom()
              }, 10)
            }
            
            if (data.done) {
              isTyping = false
              if (currentChatHistory.value[streamingMessageIndex]) {
                if (agentType === 'deep_research' && thinkingContent) {
                  const thinkingDisplay = `<details style="margin: 10px 0; border: 1px solid rgba(255,255,255,0.2); border-radius: 8px; padding: 10px;">
<summary style="cursor: pointer; font-weight: bold; color: #a855f7;">🤔 Show thinking process</summary>
<div style="margin-top: 10px; padding: 10px; background: rgba(168,85,247,0.1); border-radius: 6px; font-family: monospace; white-space: pre-wrap;">${thinkingContent}</div>
</details>`
                  currentChatHistory.value[streamingMessageIndex].text = `🧠 Deep Research Analysis:\n\n${thinkingDisplay}\n\n${actualResponse}`
                } else {
                  currentChatHistory.value[streamingMessageIndex].text = agentType === 'deep_research' 
                    ? `🧠 Deep Research Analysis:\n\n${actualResponse}`
                    : currentChatHistory.value[streamingMessageIndex].text.replace('▋', '')
                }
              }
              console.log(`✅ ${agentType} streaming completed. Full response: ${actualResponse}`)
            }
            break
            
          case 'error':
            isTyping = false
            console.error(`${agentType} streaming error:`, data.error)
            // Update message to show error
            if (currentChatHistory.value[streamingMessageIndex]) {
              let errorMessage = `❌ Error: ${data.error}`
              if (data.error.includes('deepseek-r1:1.5b') && agentType === 'deep_research') {
                errorMessage = `❌ DeepSeek R1 model not found. Please install it first:\n\n\`\`\`bash\nollama pull deepseek-r1:1.5b\n\`\`\``
              } else if (data.error.includes('connection refused') || data.error.includes('ECONNREFUSED')) {
                errorMessage = `❌ Cannot connect to Ollama. Please make sure Ollama is running:\n\n\`\`\`bash\nollama serve\n\`\`\``
              }
              currentChatHistory.value[streamingMessageIndex].text = errorMessage
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
      
      // Add a timeout to show model loading if it takes too long
      const loadingTimeout = setTimeout(() => {
        if (!hasStarted && currentChatHistory.value[streamingMessageIndex]) {
          currentChatHistory.value[streamingMessageIndex].text = `🔄 Loading ${agentName} model (this may take a moment for the first run)▋`
          setTimeout(() => {
            scrollChatToBottom()
          }, 10)
        }
      }, 2000)
      
      // Generate truncated context for AI (max 4000 tokens)
      const maxTokens = 4000
      const truncatedContext = getLimitedContext(currentChatHistory.value, maxTokens)
      
      console.log(`📊 Context prepared: ${truncatedContext.length} messages, estimated ~${truncatedContext.reduce((sum, msg) => sum + estimateTokens(msg.content), 0)} tokens`)
      
      // Route to appropriate agent based on type
      if (agentType === 'deep_research') {
        console.log('🧠 FRONTEND: Calling generate_deep_research (should use deepseek-r1:1.5b)')
        await invoke('generate_deep_research', {
          prompt: userMessage,
          context: truncatedContext,
          sessionId: sessionId
        })
      } else {
        console.log('🛡️ FRONTEND: Calling generate_enteract_agent_response (should use gemma3:1b-it-qat)')
        // Default to Enteract agent (gemma with security focus)
        await invoke('generate_enteract_agent_response', {
          prompt: userMessage,
          context: truncatedContext,
          sessionId: sessionId
        })
      }
      
      // Clear the loading timeout
      clearTimeout(loadingTimeout)
      
      console.log(`🤖 Started streaming AI response from ${modelToUse}`)
      
    } catch (error) {
      const errorString = error instanceof Error ? error.message : String(error)
      console.error('Failed to start AI response streaming:', error)
      
      // Enhanced error messages
      let errorMessage = `❌ Failed to get AI response: ${errorString}. Make sure Ollama is running and the model "${selectedModel || 'gemma3:1b-it-qat'}" is available.`
      if (errorString.includes('connection refused') || errorString.includes('ECONNREFUSED')) {
        errorMessage = `❌ Cannot connect to Ollama. Please make sure Ollama is running:\n\n\`\`\`bash\nollama serve\n\`\`\``
      } else if (errorString.includes('model') && errorString.includes('not found')) {
        errorMessage = `❌ Model not available. Install with:\n\n\`\`\`bash\nollama pull ${selectedModel || 'gemma3:1b-it-qat'}\n\`\`\``
      }
      
      // Add error message to current chat
      addMessageToCurrentChat({
        id: messageIdCounter++,
        sender: 'assistant',
        text: errorMessage,
        timestamp: new Date(),
        messageType: 'text'
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

  // File upload functions
  const triggerFileUpload = () => {
    fileInput.value?.click()
  }

  const handleFileUpload = async (event: Event, showChatWindow: any) => {
    const input = event.target as HTMLInputElement
    const files = input.files
    if (files) {
      // Auto-open chat window if not already open
      if (!showChatWindow.value) {
        showChatWindow.value = true
        console.log('💬 Chat window auto-opened for file upload')
      }

      for (let i = 0; i < files.length; i++) {
        const file = files[i]
        try {
          // Enhanced file upload indication
          console.log('📁 File selected:', file.name, file.type, file.size)
          
          // Add file upload message to current chat
          addMessageToCurrentChat({
            id: messageIdCounter++,
            sender: 'system',
            text: `📁 File uploaded: **${file.name}** (${file.type}, ${(file.size / 1024).toFixed(1)} KB)`,
            timestamp: new Date(),
            messageType: 'text'
          })
          
          // Show upload success feedback
          setTimeout(() => {
            addMessageToCurrentChat({
              id: messageIdCounter++,
              sender: 'system',
              text: `✅ File ready for analysis. You can now ask questions about this ${file.type.includes('image') ? 'image' : 'document'}.`,
              timestamp: new Date(),
              messageType: 'text'
            })
            
            // Auto-scroll to show the uploaded file message
            setTimeout(() => {
              scrollChatToBottom()
            }, 100)
          }, 500)
          
        } catch (error) {
          console.error('File upload error:', error)
          addMessageToCurrentChat({
            id: messageIdCounter++,
            sender: 'system',
            text: `❌ File upload failed: ${error}`,
            timestamp: new Date(),
            messageType: 'text'
          })
        }
      }
      
      // Auto-scroll to show files
      setTimeout(() => {
        scrollChatToBottom()
      }, 150)
    }
    // Reset input
    input.value = ''
  }

  // Initialize chat sessions on mount
  onMounted(async () => {
    // Only initialize shared state once
    if (!isInitialized.value) {
      await loadAllChats()
      
      // Set up watchers after initial load
      watch(chatSessions, () => {
        debouncedSaveChats()
      }, { deep: true })
      
      isInitialized.value = true
      console.log('✅ Chat management initialized')
    }
  })

  return {
    // Legacy exports for compatibility
    chatMessage,
    chatHistory: currentChatHistory, // Export as computed for backward compatibility
    fileInput,
    
    // New multi-session exports
    chatSessions,
    currentChatId,
    currentChatHistory,
    currentChatSession,
    
    // Session management functions
    createNewChat,
    switchChat,
    deleteChat,
    renameChat,
    clearChat,
    loadAllChats,
    saveAllChats,
    
    // Utility functions
    estimateTokens,
    
    // Existing functions
    renderMarkdown,
    takeScreenshotAndAnalyze,
    startDeepResearch,
    startConversationalAgent,
    startCodingAgent,
    startComputerUseAgent,
    sendMessage,
    handleChatKeydown,
    triggerFileUpload,
    handleFileUpload
  }
} 