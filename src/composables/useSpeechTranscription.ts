import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type {
  TranscriptionResult,
  WhisperConfig,
  TranscriptionSession,
  SpeechRecognitionConfig,
  AudioStreamConfig,
  TranscriptionStatus
} from '../types/speechTranscription'

// Extend the SpeechRecognition interface
declare global {
  interface Window {
    SpeechRecognition: typeof SpeechRecognition
    webkitSpeechRecognition: typeof SpeechRecognition
  }
}

export function useSpeechTranscription() {
  // State
  const isInitialized = ref(false)
  const isRecording = ref(false)
  const isProcessing = ref(false)
  const hasWebSpeechSupport = ref(false)
  const hasWhisperModel = ref(false)
  const currentText = ref('')
  const interimText = ref('')
  const finalText = ref('')
  const transcriptionHistory = ref<TranscriptionResult[]>([])
  const currentSession = ref<TranscriptionSession | null>(null)
  const error = ref<string | null>(null)

  // Audio recording
  let mediaRecorder: MediaRecorder | null = null
  let audioChunks: Blob[] = []
  let recognition: SpeechRecognition | null = null
  let audioStream: MediaStream | null = null

  // Configuration
  const defaultWhisperConfig: WhisperConfig = {
    modelSize: 'base',
    language: 'en',
    enableVAD: true,
    silenceThreshold: 0.01,
    maxSegmentLength: 30
  }

  const defaultSpeechConfig: SpeechRecognitionConfig = {
    continuous: true,
    interimResults: true,
    language: 'en-US',
    maxAlternatives: 1
  }

  const defaultAudioConfig: AudioStreamConfig = {
    sampleRate: 16000,
    channels: 1,
    bufferSize: 4096,
    mimeType: 'audio/webm;codecs=opus'
  }

  // Computed
  const status = computed<TranscriptionStatus>(() => ({
    isRecording: isRecording.value,
    isProcessing: isProcessing.value,
    hasWebSpeechSupport: hasWebSpeechSupport.value,
    hasWhisperModel: hasWhisperModel.value,
    currentSession: currentSession.value || undefined
  }))

  const combinedText = computed(() => {
    if (interimText.value) {
      return finalText.value + ' ' + interimText.value
    }
    return finalText.value
  })

  // Initialize the transcription system
  async function initialize(whisperConfig: Partial<WhisperConfig> = {}) {
    try {
      error.value = null

      // Check Web Speech API support
      const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition
      hasWebSpeechSupport.value = !!SpeechRecognition

      if (hasWebSpeechSupport.value) {
        recognition = new SpeechRecognition()
        setupSpeechRecognition()
      }

      // Initialize Whisper model
      const config = { ...defaultWhisperConfig, ...whisperConfig }
      
      // Check if model exists
      const modelExists = await invoke<boolean>('check_whisper_model_availability', {
        modelSize: config.modelSize
      })

      if (!modelExists) {
        console.log(`Downloading Whisper model: ${config.modelSize}`)
        await invoke<string>('download_whisper_model', {
          modelSize: config.modelSize
        })
      }

      // Initialize the model
      await invoke<string>('initialize_whisper_model', {
        config: {
          model_size: config.modelSize,
          language: config.language,
          enable_vad: config.enableVAD,
          silence_threshold: config.silenceThreshold,
          max_segment_length: config.maxSegmentLength
        }
      })

      hasWhisperModel.value = true
      isInitialized.value = true
      
      console.log('Speech transcription system initialized successfully')
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err)
      error.value = `Failed to initialize: ${message}`
      console.error('Initialization error:', err)
    }
  }

  // Setup Web Speech Recognition
  function setupSpeechRecognition() {
    if (!recognition) return

    recognition.continuous = defaultSpeechConfig.continuous
    recognition.interimResults = defaultSpeechConfig.interimResults
    recognition.lang = defaultSpeechConfig.language
    recognition.maxAlternatives = defaultSpeechConfig.maxAlternatives

    recognition.onstart = () => {
      console.log('Speech recognition started')
    }

    recognition.onresult = (event) => {
      let interim = ''
      let final = ''

      for (let i = event.resultIndex; i < event.results.length; i++) {
        const result = event.results[i]
        const transcript = result[0].transcript

        if (result.isFinal) {
          final += transcript
        } else {
          interim += transcript
        }
      }

      if (final) {
        finalText.value += (finalText.value ? ' ' : '') + final
        interimText.value = ''
        
        // Add to history
        const transcriptionResult: TranscriptionResult = {
          text: final,
          isFinal: true,
          confidence: event.results[event.resultIndex]?.[0]?.confidence || 0.9,
          timestamp: Date.now(),
          source: 'web-speech'
        }
        
        transcriptionHistory.value.push(transcriptionResult)
      } else {
        interimText.value = interim
      }
    }

    recognition.onerror = (event) => {
      console.error('Speech recognition error:', event.error)
      error.value = `Speech recognition error: ${event.error}`
    }

    recognition.onend = () => {
      console.log('Speech recognition ended')
      if (isRecording.value) {
        // Restart if we're still supposed to be recording
        try {
          recognition?.start()
        } catch (err) {
          console.warn('Failed to restart speech recognition:', err)
        }
      }
    }
  }

  // Start recording
  async function startRecording() {
    if (!isInitialized.value) {
      throw new Error('Transcription system not initialized')
    }

    try {
      error.value = null
      isRecording.value = true

      // Clear previous results
      interimText.value = ''
      finalText.value = ''
      audioChunks = []

      // Create session
      currentSession.value = {
        id: `session_${Date.now()}`,
        isActive: true,
        startTime: Date.now(),
        language: defaultWhisperConfig.language || 'en',
        config: defaultWhisperConfig
      }

      // Start Web Speech API for interim results
      if (recognition && hasWebSpeechSupport.value) {
        recognition.start()
      }

      // Start audio recording for Whisper processing
      audioStream = await navigator.mediaDevices.getUserMedia({
        audio: {
          sampleRate: defaultAudioConfig.sampleRate,
          channelCount: defaultAudioConfig.channels,
          echoCancellation: true,
          noiseSuppression: true
        }
      })

      mediaRecorder = new MediaRecorder(audioStream, {
        mimeType: defaultAudioConfig.mimeType
      })

      mediaRecorder.ondataavailable = (event) => {
        if (event.data.size > 0) {
          audioChunks.push(event.data)
        }
      }

      mediaRecorder.onstop = async () => {
        if (audioChunks.length > 0) {
          await processAudioWithWhisper()
        }
      }

      // Record in chunks for real-time processing
      mediaRecorder.start(1000) // 1 second chunks

      console.log('Recording started')
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err)
      error.value = `Failed to start recording: ${message}`
      isRecording.value = false
      throw err
    }
  }

  // Stop recording
  const stopRecording = async () => {
    if (!isRecording.value) return

    try {
      isRecording.value = false
      
      // Stop Web Speech API
      if (recognition) {
        recognition.stop()
      }

      // Stop MediaRecorder and process with Whisper
      if (mediaRecorder && mediaRecorder.state === 'recording') {
        mediaRecorder.stop()
        isProcessing.value = true
        
        // Wait for the recorded data to be processed
        // The Whisper processing will happen in the dataavailable event
      }

      // Clear interim text when stopping
      interimText.value = ''
      
    } catch (err) {
      console.error('Error stopping recording:', err)
      error.value = err instanceof Error ? err.message : 'Failed to stop recording'
    }
  }

  // Process audio with Whisper
  async function processAudioWithWhisper() {
    if (audioChunks.length === 0) return

    try {
      isProcessing.value = true

      // Combine audio chunks
      const audioBlob = new Blob(audioChunks, { type: defaultAudioConfig.mimeType })
      
      // Convert to base64
      const audioBase64 = await blobToBase64(audioBlob)

      // Send to Whisper for transcription
      const result = await invoke<{
        text: string
        confidence: number
        start_time: number
        end_time: number
        language?: string
      }>('transcribe_audio_base64', {
        audioData: audioBase64,
        config: {
          model_size: defaultWhisperConfig.modelSize,
          language: defaultWhisperConfig.language,
          enable_vad: defaultWhisperConfig.enableVAD,
          silence_threshold: defaultWhisperConfig.silenceThreshold,
          max_segment_length: defaultWhisperConfig.maxSegmentLength
        }
      })

      if (result.text.trim()) {
        // Replace interim text with Whisper result
        finalText.value = result.text
        interimText.value = ''

        // Add to history
        const transcriptionResult: TranscriptionResult = {
          text: result.text,
          isFinal: true,
          confidence: result.confidence,
          timestamp: Date.now(),
          source: 'whisper'
        }

        transcriptionHistory.value.push(transcriptionResult)

        console.log('Whisper transcription:', result.text)
      }

      // Clear processed chunks
      audioChunks = []
    } catch (err) {
      console.error('Whisper processing error:', err)
      error.value = `Whisper processing failed: ${err}`
    } finally {
      isProcessing.value = false
    }
  }

  // Clear transcription
  function clearTranscription() {
    finalText.value = ''
    interimText.value = ''
    transcriptionHistory.value = []
    error.value = null
  }

  // Get available models
  async function getAvailableModels() {
    try {
      return await invoke<string[]>('list_available_models')
    } catch (err) {
      console.error('Failed to get available models:', err)
      return []
    }
  }

  // Helper function to convert blob to base64
  function blobToBase64(blob: Blob): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader()
      reader.onload = () => {
        const result = reader.result as string
        // Remove data URL prefix to get just the base64 data
        const base64 = result.split(',')[1]
        resolve(base64)
      }
      reader.onerror = reject
      reader.readAsDataURL(blob)
    })
  }

  // Cleanup
  onUnmounted(() => {
    if (isRecording.value) {
      stopRecording().catch(console.error)
    }
  })

  return {
    // State
    isInitialized,
    isRecording,
    isProcessing,
    hasWebSpeechSupport,
    hasWhisperModel,
    currentText: combinedText,
    interimText,
    finalText,
    transcriptionHistory,
    currentSession,
    error,
    status,

    // Methods
    initialize,
    startRecording,
    stopRecording,
    clearTranscription,
    getAvailableModels
  }
} 