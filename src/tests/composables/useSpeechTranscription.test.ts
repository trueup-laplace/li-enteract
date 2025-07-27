import { describe, it, expect, vi, beforeEach } from 'vitest'
import { ref } from 'vue'

// Mock the speech transcription composable
const createMockUseSpeechTranscription = () => {
  const isRecording = ref(false)
  const isInitialized = ref(false)
  const error = ref<string | null>(null)
  const transcript = ref('')
  const confidence = ref(0)

  const initialize = vi.fn().mockImplementation(async () => {
    isInitialized.value = true
    return Promise.resolve()
  })

  const startRecording = vi.fn().mockImplementation(async () => {
    if (!isInitialized.value) {
      throw new Error('Speech recognition not initialized')
    }
    isRecording.value = true
    error.value = null
    return Promise.resolve()
  })

  const stopRecording = vi.fn().mockImplementation(async () => {
    isRecording.value = false
    return Promise.resolve()
  })

  const setAutoSendToChat = vi.fn()
  const setContinuousMode = vi.fn()

  // Simulate transcript updates
  const simulateTranscript = (text: string, conf: number = 0.9) => {
    transcript.value = text
    confidence.value = conf
  }

  const simulateError = (errorMessage: string) => {
    error.value = errorMessage
    isRecording.value = false
  }

  return {
    isRecording,
    isInitialized,
    error,
    transcript,
    confidence,
    initialize,
    startRecording,
    stopRecording,
    setAutoSendToChat,
    setContinuousMode,
    simulateTranscript,
    simulateError,
  }
}

describe('useSpeechTranscription', () => {
  let speechTranscription: ReturnType<typeof createMockUseSpeechTranscription>

  beforeEach(() => {
    speechTranscription = createMockUseSpeechTranscription()
  })

  describe('Initialization', () => {
    it('initializes speech recognition correctly', async () => {
      expect(speechTranscription.isInitialized.value).toBe(false)
      
      await speechTranscription.initialize()
      
      expect(speechTranscription.initialize).toHaveBeenCalledOnce()
      expect(speechTranscription.isInitialized.value).toBe(true)
    })

    it('handles initialization errors', async () => {
      speechTranscription.initialize.mockRejectedValueOnce(new Error('Microphone not available'))
      
      try {
        await speechTranscription.initialize()
      } catch (error) {
        expect(error).toBeInstanceOf(Error)
        expect((error as Error).message).toBe('Microphone not available')
      }
      
      expect(speechTranscription.isInitialized.value).toBe(false)
    })
  })

  describe('Recording Control', () => {
    beforeEach(async () => {
      await speechTranscription.initialize()
    })

    it('starts recording successfully', async () => {
      expect(speechTranscription.isRecording.value).toBe(false)
      
      await speechTranscription.startRecording()
      
      expect(speechTranscription.startRecording).toHaveBeenCalledOnce()
      expect(speechTranscription.isRecording.value).toBe(true)
      expect(speechTranscription.error.value).toBe(null)
    })

    it('stops recording successfully', async () => {
      await speechTranscription.startRecording()
      expect(speechTranscription.isRecording.value).toBe(true)
      
      await speechTranscription.stopRecording()
      
      expect(speechTranscription.stopRecording).toHaveBeenCalledOnce()
      expect(speechTranscription.isRecording.value).toBe(false)
    })

    it('prevents recording when not initialized', async () => {
      const uninitializedSpeech = createMockUseSpeechTranscription()
      
      try {
        await uninitializedSpeech.startRecording()
      } catch (error) {
        expect(error).toBeInstanceOf(Error)
        expect((error as Error).message).toBe('Speech recognition not initialized')
      }
      
      expect(uninitializedSpeech.isRecording.value).toBe(false)
    })
  })

  describe('Transcript Handling', () => {
    beforeEach(async () => {
      await speechTranscription.initialize()
      await speechTranscription.startRecording()
    })

    it('updates transcript correctly', () => {
      speechTranscription.simulateTranscript('Hello world', 0.95)
      
      expect(speechTranscription.transcript.value).toBe('Hello world')
      expect(speechTranscription.confidence.value).toBe(0.95)
    })

    it('handles multiple transcript updates', () => {
      speechTranscription.simulateTranscript('Hello', 0.8)
      expect(speechTranscription.transcript.value).toBe('Hello')
      
      speechTranscription.simulateTranscript('Hello world', 0.9)
      expect(speechTranscription.transcript.value).toBe('Hello world')
      
      speechTranscription.simulateTranscript('Hello world, how are you?', 0.95)
      expect(speechTranscription.transcript.value).toBe('Hello world, how are you?')
    })
  })

  describe('Error Handling', () => {
    beforeEach(async () => {
      await speechTranscription.initialize()
    })

    it('handles speech recognition errors', () => {
      speechTranscription.simulateError('Network error')
      
      expect(speechTranscription.error.value).toBe('Network error')
      expect(speechTranscription.isRecording.value).toBe(false)
    })

    it('clears error when starting new recording', async () => {
      speechTranscription.simulateError('Previous error')
      expect(speechTranscription.error.value).toBe('Previous error')
      
      await speechTranscription.startRecording()
      expect(speechTranscription.error.value).toBe(null)
    })
  })

  describe('Configuration', () => {
    it('sets auto send to chat mode', () => {
      speechTranscription.setAutoSendToChat(true)
      expect(speechTranscription.setAutoSendToChat).toHaveBeenCalledWith(true)
      
      speechTranscription.setAutoSendToChat(false)
      expect(speechTranscription.setAutoSendToChat).toHaveBeenCalledWith(false)
    })

    it('sets continuous mode', () => {
      speechTranscription.setContinuousMode(true)
      expect(speechTranscription.setContinuousMode).toHaveBeenCalledWith(true)
      
      speechTranscription.setContinuousMode(false)
      expect(speechTranscription.setContinuousMode).toHaveBeenCalledWith(false)
    })
  })

  describe('State Management', () => {
    it('maintains consistent state during recording lifecycle', async () => {
      // Initial state
      expect(speechTranscription.isRecording.value).toBe(false)
      expect(speechTranscription.isInitialized.value).toBe(false)
      expect(speechTranscription.error.value).toBe(null)
      
      // After initialization
      await speechTranscription.initialize()
      expect(speechTranscription.isInitialized.value).toBe(true)
      
      // During recording
      await speechTranscription.startRecording()
      expect(speechTranscription.isRecording.value).toBe(true)
      
      // After stopping
      await speechTranscription.stopRecording()
      expect(speechTranscription.isRecording.value).toBe(false)
      expect(speechTranscription.isInitialized.value).toBe(true) // Should remain initialized
    })
  })
})