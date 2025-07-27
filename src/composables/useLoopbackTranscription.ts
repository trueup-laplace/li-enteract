import { ref, Ref, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useConversationStore } from '../stores/conversation'

export function useLoopbackTranscription() {
  const conversationStore = useConversationStore()
  
  // State
  const loopbackBuffer = ref<string>('')
  const loopbackLastTimestamp = ref<number>(0)
  const loopbackThoughtTimer = ref<number | null>(null)
  const loopbackBufferStartTime = ref<number>(0)
  const THOUGHT_PAUSE_DURATION = 2500  // Shorter pause for natural breaks (2.5s)
  const MAX_BUFFER_DURATION = 10000    // Max 10s speaking length as requested
  const MAX_CONCATENATION_TIME = 3000  // Only concatenate within 3s of last message
  const MIN_SIZEABLE_CONTENT = 30
  
  // Deduplication state
  const lastProcessedText = ref<string>('')
  const lastProcessedTimestamp = ref<number>(0)
  const recentMessages = ref<Set<string>>(new Set())
  const sentenceBuffer = ref<string[]>([])
  const processedChunks = ref<Set<string>>(new Set())
  
  // Typing animation state
  const isLoopbackTyping = ref(false)
  const loopbackPreviewMessage = ref<string>('')
  const currentPreviewMessageId = ref<string | null>(null)
  
  // User microphone typing state
  const isMicrophoneTyping = ref(false)
  const microphonePreviewMessage = ref<string>('')
  const currentMicPreviewMessageId = ref<string | null>(null)
  
  // Event unlisteners
  const unlisteners: Array<() => void> = []
  
  // Helper functions
  const cleanTranscriptionText = (text: string): string => {
    return text
      .replace(/[\r\n]+/g, ' ')
      .replace(/\s+/g, ' ')
      .trim()
  }
  
  const isCompleteSentence = (text: string): boolean => {
    const trimmed = text.trim()
    return /[.!?]$/.test(trimmed) || /[.!?]\s*["']?$/.test(trimmed)
  }
  
  const intelligentConcatenation = (existing: string, newText: string): string => {
    if (!existing) return newText
    
    const existingTrimmed = existing.trim()
    const newTextTrimmed = newText.trim()
    
    // Check for exact duplicate
    if (existingTrimmed === newTextTrimmed) return existing
    
    // Check if new text is contained in existing
    if (existingTrimmed.includes(newTextTrimmed)) return existing
    
    // Check if existing is contained in new (replacement)
    if (newTextTrimmed.includes(existingTrimmed)) return newText
    
    // Check for overlap at boundaries
    for (let i = Math.min(existingTrimmed.length, newTextTrimmed.length); i > 5; i--) {
      const existingEnd = existingTrimmed.slice(-i).toLowerCase()
      const newStart = newTextTrimmed.slice(0, i).toLowerCase()
      
      if (existingEnd === newStart) {
        return existingTrimmed + ' ' + newTextTrimmed.slice(i)
      }
    }
    
    // No overlap, concatenate with space
    return existingTrimmed + ' ' + newTextTrimmed
  }
  
  const findLongestCommonSubstring = (str1: string, str2: string): string | null => {
    if (!str1 || !str2) return null
    
    const m = str1.length
    const n = str2.length
    let maxLength = 0
    let endingPos = 0
    const lookup = Array(m + 1).fill(null).map(() => Array(n + 1).fill(0))
    
    for (let i = 1; i <= m; i++) {
      for (let j = 1; j <= n; j++) {
        if (str1[i - 1] === str2[j - 1]) {
          lookup[i][j] = lookup[i - 1][j - 1] + 1
          if (lookup[i][j] > maxLength) {
            maxLength = lookup[i][j]
            endingPos = i
          }
        }
      }
    }
    
    return maxLength > 0 ? str1.substring(endingPos - maxLength, endingPos) : null
  }
  
  const flushLoopbackBuffer = () => {
    if (loopbackBuffer.value.trim()) {
      const finalContent = cleanTranscriptionText(loopbackBuffer.value.trim())
      
      if (finalContent.length < 5) {
        clearBufferState()
        return
      }
      
      const existingMessages = conversationStore.currentMessages || []
      const lastMessage = existingMessages[existingMessages.length - 1]
      
      // Check if we should concatenate with the last system message
      if (lastMessage && 
          lastMessage.source === 'loopback' && 
          lastMessage.type === 'system') {
        
        // Check for exact duplicate
        if (lastMessage.content === finalContent) {
          clearBufferState()
          return
        }
        
        // Check if content is already contained in last message
        if (lastMessage.content.includes(finalContent)) {
          clearBufferState()
          return
        }
        
        // Check recent timestamp - only concatenate within short window for natural flow
        const timeDiff = Date.now() - lastMessage.timestamp
        if (timeDiff < MAX_CONCATENATION_TIME) {
          // Concatenate with existing message
          const concatenatedContent = intelligentConcatenation(lastMessage.content, finalContent)
          
          // Update the existing message instead of creating a new one
          conversationStore.updateMessage(lastMessage.id, {
            content: concatenatedContent,
            timestamp: Date.now(), // Update timestamp
            confidence: Math.min(0.95, (lastMessage.confidence || 0.8) + 0.05) // Slightly increase confidence
          })
          
          console.log('🔗 Concatenated with existing message:', concatenatedContent.substring(0, 50))
          clearBufferState()
          return
        } else {
          console.log('⏰ Time gap too large for concatenation:', timeDiff, 'ms - creating new bubble')
        }
      }
      
      const messageFingerprint = finalContent.toLowerCase().replace(/[^\w\s]/g, '').slice(0, 100)
      if (recentMessages.value.has(messageFingerprint)) {
        clearBufferState()
        return
      }
      
      if (lastMessage && lastMessage.source === 'loopback') {
        const overlap = findLongestCommonSubstring(lastMessage.content, finalContent)
        if (overlap && overlap.length > Math.min(lastMessage.content.length, finalContent.length) * 0.7) {
          clearBufferState()
          return
        }
      }
      
      clearBufferState()
      
      recentMessages.value.add(messageFingerprint)
      setTimeout(() => {
        recentMessages.value.delete(messageFingerprint)
      }, 30000)
      
      // Calculate confidence based on sentence completeness and buffer chunks
      const baseConfidence = sentenceBuffer.value.length > 0 ? 
        Math.min(0.9, 0.6 + (sentenceBuffer.value.length * 0.1)) : 0.8
      const completenessBonus = isCompleteSentence(finalContent) ? 0.1 : 0
      const finalConfidence = Math.min(0.95, baseConfidence + completenessBonus)
      
      // Only add message if there's an active session
      if (conversationStore.currentSession) {
        conversationStore.addMessage({
          type: 'system',
          source: 'loopback',
          content: finalContent,
          confidence: finalConfidence,
          timestamp: Date.now()
        })
        console.log('📝 Created new loopback message:', finalContent.substring(0, 50))
      } else {
        console.log('🚫 Skipping loopback message - no active session:', finalContent.substring(0, 50))
      }
    }
    
    function clearBufferState() {
      loopbackBuffer.value = ''
      loopbackPreviewMessage.value = ''
      isLoopbackTyping.value = false
      currentPreviewMessageId.value = null
      sentenceBuffer.value = []
      if (loopbackThoughtTimer.value) {
        clearTimeout(loopbackThoughtTimer.value)
        loopbackThoughtTimer.value = null
      }
    }
  }
  
  // Event handlers
  const handleConversationalUserInterim = (event: Event) => {
    const customEvent = event as CustomEvent
    const { text } = customEvent.detail
    
    if (text && text.trim()) {
      microphonePreviewMessage.value = text.trim()
      isMicrophoneTyping.value = true
      currentMicPreviewMessageId.value = `mic-preview-${Date.now()}`
    }
  }
  
  const handleConversationalUserSpeech = (event: Event) => {
    const customEvent = event as CustomEvent
    const { text, confidence, timestamp } = customEvent.detail
    
    if (text && text.trim()) {
      isMicrophoneTyping.value = false
      microphonePreviewMessage.value = ''
      currentMicPreviewMessageId.value = null
      
      // Only add message if there's an active session
      if (conversationStore.currentSession) {
        conversationStore.addMessage({
          type: 'user',
          source: 'microphone',
          content: text.trim(),
          confidence,
          timestamp: timestamp || Date.now()
        })
      } else {
        console.log('🚫 Skipping microphone message - no active session:', text.substring(0, 50))
      }
    }
  }
  
  const handleLoopbackTranscription = (payload: any) => {
    const { text, timestamp } = payload
    
    if (text && text.trim()) {
      const currentTime = timestamp || Date.now()
      const cleanedText = cleanTranscriptionText(text)
      
      const chunkFingerprint = `${cleanedText.toLowerCase().slice(0, 50)}_${Math.floor(currentTime / 2000)}`
      if (processedChunks.value.has(chunkFingerprint)) {
        return
      }
      
      processedChunks.value.add(chunkFingerprint)
      setTimeout(() => {
        processedChunks.value.delete(chunkFingerprint)
      }, 30000)
      
      if (lastProcessedText.value === cleanedText && 
          (currentTime - lastProcessedTimestamp.value) < 1000) {
        return
      }
      
      lastProcessedText.value = cleanedText
      lastProcessedTimestamp.value = currentTime
      
      const timeSinceLastChunk = currentTime - loopbackLastTimestamp.value
      
      if (timeSinceLastChunk > THOUGHT_PAUSE_DURATION && loopbackBuffer.value.trim()) {
        flushLoopbackBuffer()
      }
      
      if (!loopbackBuffer.value.trim()) {
        loopbackBufferStartTime.value = currentTime
        currentPreviewMessageId.value = `loopback-preview-${currentTime}`
        sentenceBuffer.value = []
      }
      
      const newBufferContent = intelligentConcatenation(loopbackBuffer.value, cleanedText)
      
      if (newBufferContent !== loopbackBuffer.value) {
        loopbackBuffer.value = newBufferContent
        sentenceBuffer.value.push(cleanedText)
        
        loopbackPreviewMessage.value = loopbackBuffer.value
        isLoopbackTyping.value = true
      }
      
      loopbackLastTimestamp.value = currentTime
      
      const bufferDuration = currentTime - loopbackBufferStartTime.value
      const hasSize = loopbackBuffer.value.length >= MIN_SIZEABLE_CONTENT
      const hasTime = bufferDuration >= MAX_BUFFER_DURATION
      const isComplete = isCompleteSentence(loopbackBuffer.value)
      
      if (isComplete && hasSize && bufferDuration > 2000) {
        flushLoopbackBuffer()
        return
      }
      
      if (hasSize && hasTime) {
        flushLoopbackBuffer()
        return
      }
      
      if (loopbackThoughtTimer.value) {
        clearTimeout(loopbackThoughtTimer.value)
      }
      
      loopbackThoughtTimer.value = window.setTimeout(() => {
        flushLoopbackBuffer()
      }, THOUGHT_PAUSE_DURATION)
    }
  }
  
  const setupLoopbackListeners = async (isRecording: Ref<boolean>) => {
    // Only process events when recording
    const wrappedInterimHandler = (event: Event) => {
      if (isRecording.value) handleConversationalUserInterim(event)
    }
    
    const wrappedFinalHandler = (event: Event) => {
      if (isRecording.value) handleConversationalUserSpeech(event)
    }
    
    window.addEventListener('transcription-final', wrappedFinalHandler)
    window.addEventListener('transcription-interim', wrappedInterimHandler)
    
    unlisteners.push(() => {
      window.removeEventListener('transcription-final', wrappedFinalHandler)
      window.removeEventListener('transcription-interim', wrappedInterimHandler)
    })
    
    const unlistenLoopback = await listen('loopback-transcription', (event) => {
      handleLoopbackTranscription(event.payload)
    })
    
    unlisteners.push(unlistenLoopback)
  }
  
  const cleanupLoopback = () => {
    unlisteners.forEach(fn => fn())
    unlisteners.length = 0
    
    if (loopbackThoughtTimer.value) {
      clearTimeout(loopbackThoughtTimer.value)
    }
  }
  
  onUnmounted(() => {
    cleanupLoopback()
  })
  
  return {
    isLoopbackTyping,
    loopbackPreviewMessage,
    currentPreviewMessageId,
    isMicrophoneTyping,
    microphonePreviewMessage,
    currentMicPreviewMessageId,
    setupLoopbackListeners,
    cleanupLoopback
  }
}