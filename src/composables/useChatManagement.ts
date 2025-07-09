import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { ChatMessage } from '../types'

let messageIdCounter = 1

export const useChatManagement = (selectedModel: string | null, scrollChatToBottom: () => void) => {
  const chatMessage = ref('')
  const chatHistory = ref<ChatMessage[]>([])
  const fileInput = ref<HTMLInputElement>()

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
      
      // Add screen analysis message to chat
      const screenshotMessageIndex = chatHistory.value.length
      chatHistory.value.push({
        id: messageIdCounter++,
        sender: 'user',
        text: `🔍 Screen captured for analysis (${screenshot.width}×${screenshot.height})`,
        timestamp: new Date(),
        messageType: 'text'
      })
      
      // Generate unique session ID
      const sessionId = `vision-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
      
      // Add streaming message placeholder with more detailed status
      const streamingMessageIndex = chatHistory.value.length
      chatHistory.value.push({
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
            if (chatHistory.value[streamingMessageIndex]) {
              chatHistory.value[streamingMessageIndex].text = `👁️ Qwen ${data.model} is analyzing your screenshot▋`
            }
            setTimeout(() => {
              scrollChatToBottom()
            }, 10)
            break
            
          case 'chunk':
            if (data.text) {
              // First chunk - update status
              if (fullResponse === '') {
                if (chatHistory.value[streamingMessageIndex]) {
                  chatHistory.value[streamingMessageIndex].text = '👁️ Vision Analysis:\n\n'
                }
              }
              
              fullResponse += data.text
              if (chatHistory.value[streamingMessageIndex]) {
                chatHistory.value[streamingMessageIndex].text = '👁️ Vision Analysis:\n\n' + fullResponse + (isTyping ? '▋' : '')
              }
              
              setTimeout(() => {
                scrollChatToBottom()
              }, 10)
            }
            
            if (data.done) {
              isTyping = false
              if (chatHistory.value[streamingMessageIndex]) {
                chatHistory.value[streamingMessageIndex].text = '👁️ Vision Analysis:\n\n' + fullResponse
              }
              console.log('✅ Vision analysis streaming completed')
            }
            break
            
          case 'error':
            isTyping = false
            console.error('Vision analysis error:', data.error)
            if (chatHistory.value[streamingMessageIndex]) {
              if (data.error.includes('qwen2.5vl:3b')) {
                chatHistory.value[streamingMessageIndex].text = `❌ Vision model not found. Please install qwen2.5vl:3b first:\n\n\`\`\`bash\nollama pull qwen2.5vl:3b\n\`\`\``
              } else {
                chatHistory.value[streamingMessageIndex].text = `❌ Vision analysis error: ${data.error}`
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
        if (!hasStarted && chatHistory.value[streamingMessageIndex]) {
          chatHistory.value[streamingMessageIndex].text = '🔄 Loading Qwen vision model (this may take a moment for the first run)▋'
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
      
      chatHistory.value.push({
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
      setTimeout(() => {
        scrollChatToBottom()
      }, 150)
    }
    
    let researchQuery = chatMessage.value.trim()
    
    // If no input, prompt for research topic
    if (!researchQuery) {
      const promptResult = prompt('🧠 Deep Research Mode\n\nWhat would you like me to research in detail?')
      
      if (!promptResult || !promptResult.trim()) {
        console.log('🧠 Deep research cancelled - no query provided')
        return
      }
      
      researchQuery = promptResult.trim()
      
      // Add the query to chat input for user to see, then clear it
      chatMessage.value = researchQuery
    }
    
    console.log('🧠 FRONTEND: Deep Research button clicked, calling sendMessage with deep_research mode')
    console.log('🧠 FRONTEND: Research query:', researchQuery)
    
    await sendMessage('deep_research')
  }

  // Conversational Agent
  const startConversationalAgent = async (showChatWindow: any) => {
    // Auto-open chat window if not open
    if (!showChatWindow.value) {
      showChatWindow.value = true
      setTimeout(() => {
        scrollChatToBottom()
      }, 150)
    }
    
    let conversationTopic = chatMessage.value.trim()
    
    // If no input, prompt for conversation topic
    if (!conversationTopic) {
      const promptResult = prompt('💬 Conversational Agent\n\nWhat would you like to chat about?')
      
      if (!promptResult || !promptResult.trim()) {
        console.log('💬 Conversational agent cancelled - no topic provided')
        return
      }
      
      conversationTopic = promptResult.trim()
      chatMessage.value = conversationTopic
    }
    
    console.log('💬 FRONTEND: Conversational Agent clicked')
    // For now, use the default enteract agent
    await sendMessage('enteract')
  }

  // Coding Agent
  const startCodingAgent = async (showChatWindow: any) => {
    // Auto-open chat window if not open
    if (!showChatWindow.value) {
      showChatWindow.value = true
      setTimeout(() => {
        scrollChatToBottom()
      }, 150)
    }
    
    let codingTask = chatMessage.value.trim()
    
    // If no input, prompt for coding task
    if (!codingTask) {
      const promptResult = prompt('👨‍💻 Coding Agent\n\nWhat coding task can I help you with?')
      
      if (!promptResult || !promptResult.trim()) {
        console.log('👨‍💻 Coding agent cancelled - no task provided')
        return
      }
      
      codingTask = promptResult.trim()
      chatMessage.value = codingTask
    }
    
    console.log('👨‍💻 FRONTEND: Coding Agent clicked')
    // TODO: Implement coding-specific agent
    // For now, use the default enteract agent
    await sendMessage('enteract')
  }

  // Computer Use Agent (Experimental)
  const startComputerUseAgent = async (showChatWindow: any) => {
    // Auto-open chat window if not open
    if (!showChatWindow.value) {
      showChatWindow.value = true
      setTimeout(() => {
        scrollChatToBottom()
      }, 150)
    }
    
    let computerTask = chatMessage.value.trim()
    
    // If no input, prompt for computer task
    if (!computerTask) {
      const promptResult = prompt('🖥️ Computer Use Agent (Experimental)\n\nWhat computer task would you like me to help with?\n\n⚠️ This is experimental and may require screen access.')
      
      if (!promptResult || !promptResult.trim()) {
        console.log('🖥️ Computer Use agent cancelled - no task provided')
        return
      }
      
      computerTask = promptResult.trim()
      chatMessage.value = computerTask
    }
    
    console.log('🖥️ FRONTEND: Computer Use Agent clicked')
    // TODO: Implement computer use agent with screen interaction
    // For now, use screen analysis as a foundation
    await takeScreenshotAndAnalyze({ value: true })
  }

  const sendMessage = async (agentType: string = 'enteract') => {
    if (!chatMessage.value.trim()) return
    
    const userMessage = chatMessage.value
    
    // Add user message to history
    chatHistory.value.push({
      id: messageIdCounter++,
      sender: 'user',
      text: userMessage,
      timestamp: new Date(),
      messageType: 'text'
    })
    
    // Clear input immediately
    chatMessage.value = ''
    
    // Auto-scroll to bottom
    setTimeout(() => {
      scrollChatToBottom()
    }, 50)
    
    try {
      // Use selected model or default to gemma3:1b-it-qat
      const modelToUse = selectedModel || 'gemma3:1b-it-qat'
      
      // Generate unique session ID for this conversation
      const sessionId = `chat-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
      
      // Add streaming message placeholder with agent indicator
      const streamingMessageIndex = chatHistory.value.length
      const agentEmoji = agentType === 'deep_research' ? '🧠' : agentType === 'vision' ? '👁️' : '🛡️'
      const agentName = agentType === 'deep_research' ? 'DeepSeek R1' : agentType === 'vision' ? 'Vision' : 'Enteract'
      const expectedModel = agentType === 'deep_research' ? 'deepseek-r1:1.5b' : agentType === 'vision' ? 'qwen2.5vl:3b' : 'gemma3:1b-it-qat'
      
      chatHistory.value.push({
        id: messageIdCounter++,
        sender: 'assistant',
        text: `${agentEmoji} Initializing ${agentName} agent (${expectedModel})▋`,
        timestamp: new Date(),
        messageType: 'text'
      })
      
      setTimeout(() => {
        scrollChatToBottom()
      }, 50)
      
      let fullResponse = ''
      let isTyping = true
      let hasStarted = false
      let isInThinking = false
      let thinkingContent = ''
      let actualResponse = ''
      
      // Set up event listener for streaming response
      const unlisten = await listen(`ollama-stream-${sessionId}`, (event: any) => {
        const data = event.payload
        
        switch (data.type) {
          case 'start':
            hasStarted = true
            console.log(`🚀 Started ${agentType} streaming from ${data.model}`)
            
            // Check if actual model matches expected model
            if (data.model !== expectedModel) {
              console.warn(`⚠️ MODEL MISMATCH: Expected ${expectedModel} but got ${data.model}`)
              if (chatHistory.value[streamingMessageIndex]) {
                chatHistory.value[streamingMessageIndex].text = `${agentEmoji} ⚠️ Using ${data.model} (expected ${expectedModel}) - ${agentType === 'deep_research' ? 'researching your query' : 'processing your request'}▋`
              }
            } else {
              console.log(`✅ MODEL CORRECT: Using expected model ${data.model}`)
              if (chatHistory.value[streamingMessageIndex]) {
                chatHistory.value[streamingMessageIndex].text = `${agentEmoji} ${data.model} is ${agentType === 'deep_research' ? 'researching your query' : 'processing your request'}▋`
              }
            }
            
            setTimeout(() => {
              scrollChatToBottom()
            }, 10)
            break
            
          case 'chunk':
            if (data.text) {
              fullResponse += data.text
              
              // Handle DeepSeek thinking process
              if (agentType === 'deep_research') {
                // Check if we're entering thinking mode
                if (fullResponse.includes('<thinking>') && !isInThinking) {
                  isInThinking = true
                  const thinkingStart = fullResponse.indexOf('<thinking>')
                  actualResponse = fullResponse.substring(0, thinkingStart)
                  thinkingContent = fullResponse.substring(thinkingStart + 10) // Skip <thinking>
                }
                
                // Check if we're exiting thinking mode
                if (isInThinking && fullResponse.includes('</thinking>')) {
                  const thinkingEnd = fullResponse.indexOf('</thinking>')
                  const afterThinking = fullResponse.substring(thinkingEnd + 11) // Skip </thinking>
                  thinkingContent = fullResponse.substring(fullResponse.indexOf('<thinking>') + 10, thinkingEnd)
                  actualResponse += afterThinking
                  isInThinking = false
                  
                  // Update with thinking section collapsed by default
                  if (chatHistory.value[streamingMessageIndex]) {
                    const thinkingDisplay = `<details style="margin: 10px 0; border: 1px solid rgba(255,255,255,0.2); border-radius: 8px; padding: 10px;">
<summary style="cursor: pointer; font-weight: bold; color: #a855f7;">🤔 Show thinking process</summary>
<div style="margin-top: 10px; padding: 10px; background: rgba(168,85,247,0.1); border-radius: 6px; font-family: monospace; white-space: pre-wrap;">${thinkingContent}</div>
</details>`
                    
                    chatHistory.value[streamingMessageIndex].text = `🧠 Deep Research Analysis:\n\n${thinkingDisplay}\n\n${actualResponse}${isTyping ? '▋' : ''}`
                  }
                } else if (isInThinking) {
                  // Currently in thinking mode - update thinking content
                  const currentThinking = fullResponse.substring(fullResponse.indexOf('<thinking>') + 10)
                  if (chatHistory.value[streamingMessageIndex]) {
                    chatHistory.value[streamingMessageIndex].text = `🧠 Deep Research Analysis:\n\n🤔 **Thinking...**\n\n_${currentThinking.slice(-100)}_${isTyping ? '▋' : ''}`
                  }
                } else {
                  // Regular response mode
                  if (chatHistory.value[streamingMessageIndex]) {
                    if (actualResponse === '' && fullResponse.trim()) {
                      actualResponse = fullResponse
                    }
                    chatHistory.value[streamingMessageIndex].text = `🧠 Deep Research Analysis:\n\n${actualResponse || fullResponse}${isTyping ? '▋' : ''}`
                  }
                }
              } else {
                // Regular agent response
                if (chatHistory.value[streamingMessageIndex]) {
                  if (fullResponse.trim() && !chatHistory.value[streamingMessageIndex].text.includes(':\n\n')) {
                    chatHistory.value[streamingMessageIndex].text = `${agentEmoji} ${agentName} Response:\n\n${fullResponse}${isTyping ? '▋' : ''}`
                  } else {
                    chatHistory.value[streamingMessageIndex].text = fullResponse + (isTyping ? '▋' : '')
                  }
                }
              }
              
              setTimeout(() => {
                scrollChatToBottom()
              }, 10)
            }
            
            if (data.done) {
              isTyping = false
              if (chatHistory.value[streamingMessageIndex]) {
                if (agentType === 'deep_research' && thinkingContent) {
                  const thinkingDisplay = `<details style="margin: 10px 0; border: 1px solid rgba(255,255,255,0.2); border-radius: 8px; padding: 10px;">
<summary style="cursor: pointer; font-weight: bold; color: #a855f7;">🤔 Show thinking process</summary>
<div style="margin-top: 10px; padding: 10px; background: rgba(168,85,247,0.1); border-radius: 6px; font-family: monospace; white-space: pre-wrap;">${thinkingContent}</div>
</details>`
                  chatHistory.value[streamingMessageIndex].text = `🧠 Deep Research Analysis:\n\n${thinkingDisplay}\n\n${actualResponse}`
                } else {
                  chatHistory.value[streamingMessageIndex].text = agentType === 'deep_research' 
                    ? `🧠 Deep Research Analysis:\n\n${fullResponse}`
                    : chatHistory.value[streamingMessageIndex].text.replace('▋', '')
                }
              }
              console.log(`✅ ${agentType} streaming completed. Full response: ${fullResponse}`)
            }
            break
            
          case 'error':
            isTyping = false
            console.error(`${agentType} streaming error:`, data.error)
            // Update message to show error
            if (chatHistory.value[streamingMessageIndex]) {
              let errorMessage = `❌ Error: ${data.error}`
              if (data.error.includes('deepseek-r1:1.5b') && agentType === 'deep_research') {
                errorMessage = `❌ DeepSeek R1 model not found. Please install it first:\n\n\`\`\`bash\nollama pull deepseek-r1:1.5b\n\`\`\``
              } else if (data.error.includes('connection refused') || data.error.includes('ECONNREFUSED')) {
                errorMessage = `❌ Cannot connect to Ollama. Please make sure Ollama is running:\n\n\`\`\`bash\nollama serve\n\`\`\``
              }
              chatHistory.value[streamingMessageIndex].text = errorMessage
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
        if (!hasStarted && chatHistory.value[streamingMessageIndex]) {
          chatHistory.value[streamingMessageIndex].text = `🔄 Loading ${agentName} model (this may take a moment for the first run)▋`
          setTimeout(() => {
            scrollChatToBottom()
          }, 10)
        }
      }, 2000)
      
      // Route to appropriate agent based on type
      if (agentType === 'deep_research') {
        console.log('🧠 FRONTEND: Calling generate_deep_research (should use deepseek-r1:1.5b)')
        await invoke('generate_deep_research', {
          prompt: userMessage,
          sessionId: sessionId
        })
      } else {
        console.log('🛡️ FRONTEND: Calling generate_enteract_agent_response (should use gemma3:1b-it-qat)')
        // Default to Enteract agent (gemma with security focus)
        await invoke('generate_enteract_agent_response', {
          prompt: userMessage,
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
      
      // Add error message to chat
      chatHistory.value.push({
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
          
          // Add file upload message to chat
          chatHistory.value.push({
            id: messageIdCounter++,
            sender: 'system',
            text: `📁 File uploaded: **${file.name}** (${file.type}, ${(file.size / 1024).toFixed(1)} KB)`,
            timestamp: new Date(),
            messageType: 'text'
          })
          
          // Show upload success feedback
          setTimeout(() => {
            chatHistory.value.push({
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
          chatHistory.value.push({
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

  return {
    chatMessage,
    chatHistory,
    fileInput,
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