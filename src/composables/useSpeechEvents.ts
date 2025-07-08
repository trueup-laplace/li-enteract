import { ref } from 'vue'
import type { ChatMessage } from '../types'

let messageIdCounter = 1

export const useSpeechEvents = (
  chatHistory: any,
  showChatWindow: any,
  scrollChatToBottom: () => void,
  chatMessage: any,
  sendMessage: (agentType?: string) => Promise<void>
) => {
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
      if (lastMessage && lastMessage.sender === 'transcription' && lastMessage.isInterim) {
        // Update existing interim message
        lastMessage.text = interimText
      } else {
        // Add new interim message
        chatHistory.value.push({
          id: messageIdCounter++,
          sender: 'transcription',
          text: interimText,
          timestamp: new Date(),
          messageType: 'text',
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
      if (lastMessage && lastMessage.sender === 'transcription' && lastMessage.isInterim) {
        // Update interim message to final
        lastMessage.text = finalText
        lastMessage.isInterim = false
        lastMessage.confidence = customEvent.detail.confidence || 0.9
      } else {
        // Add new final message
        chatHistory.value.push({
          id: messageIdCounter++,
          sender: 'transcription',
          text: finalText,
          timestamp: new Date(),
          messageType: 'text',
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
      id: messageIdCounter++,
      sender: 'assistant',
      text: `❌ Transcription error: ${customEvent.detail.error}`,
      timestamp: new Date(),
      messageType: 'text'
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
      id: messageIdCounter++,
      sender: 'assistant',
      text: `🔄 Transcription stopped automatically (${customEvent.detail.reason})`,
      timestamp: new Date(),
      messageType: 'text'
    })
    
    setTimeout(() => {
      scrollChatToBottom()
    }, 50)
  }

  return {
    setupSpeechTranscriptionListeners,
    removeSpeechTranscriptionListeners
  }
} 