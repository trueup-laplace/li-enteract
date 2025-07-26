// src/composables/useAudioSettings.ts
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

// Types matching the Rust implementation
export interface AudioLoopbackDevice {
  id: string
  name: string
  is_default: boolean
  sample_rate: number
  channels: number
  format: string
  device_type: 'Render' | 'Capture'
  loopback_method: 'RenderLoopback' | 'CaptureDevice' | 'StereoMix'
}

export interface AudioDeviceSettings {
  selectedLoopbackDevice: string | null
  loopbackEnabled: boolean
  bufferSize: number
  sampleRate: number
}

export interface AudioChunkData {
  device_id: string
  audio_data: string  // base64 encoded
  timestamp: number
}

export const useAudioSettings = () => {
  // State
  const audioDevices = ref<AudioLoopbackDevice[]>([])
  const isLoadingDevices = ref(false)
  const devicesError = ref<string | null>(null)
  const isCapturing = ref(false)
  const captureError = ref<string | null>(null)
  
  const audioSettings = ref<AudioDeviceSettings>({
    selectedLoopbackDevice: null,
    loopbackEnabled: false,
    bufferSize: 4096,
    sampleRate: 16000
  })

  // Audio processing state
  const audioChunkBuffer = ref<AudioChunkData[]>([])
  const processingQueue = ref<AudioChunkData[]>([])
  const transcriptionResults = ref<string[]>([])

  // Computed
  const selectedDevice = computed(() => {
    if (!audioSettings.value.selectedLoopbackDevice) return null
    return audioDevices.value.find(d => d.id === audioSettings.value.selectedLoopbackDevice) || null
  })

  const canStartCapture = computed(() => {
    return audioSettings.value.loopbackEnabled && 
           audioSettings.value.selectedLoopbackDevice && 
           !isCapturing.value
  })

  const deviceStats = computed(() => {
    const renderDevices = audioDevices.value.filter(d => d.device_type === 'Render')
    const captureDevices = audioDevices.value.filter(d => d.device_type === 'Capture')
    const stereoMixDevices = audioDevices.value.filter(d => d.loopback_method === 'StereoMix')
    
    return {
      total: audioDevices.value.length,
      render: renderDevices.length,
      capture: captureDevices.length,
      stereoMix: stereoMixDevices.length,
      hasDefault: audioDevices.value.some(d => d.is_default)
    }
  })

  // Device enumeration
  const enumerateAudioDevices = async (): Promise<void> => {
    isLoadingDevices.value = true
    devicesError.value = null
    
    try {
      console.log('🔊 Enumerating audio loopback devices...')
      const devices = await invoke<AudioLoopbackDevice[]>('enumerate_loopback_devices')
      audioDevices.value = devices
      
      console.log(`✅ Found ${devices.length} loopback devices:`)
      devices.forEach((device, index) => {
        const methodStr = device.loopback_method === 'RenderLoopback' ? 'Render Loopback' :
                         device.loopback_method === 'StereoMix' ? 'Stereo Mix' : 'Capture Device'
        const defaultStr = device.is_default ? ' (Default)' : ''
        console.log(`  ${index + 1}. ${device.name} [${methodStr}]${defaultStr}`)
      })
      
      // Auto-select best device if none selected
      if (!audioSettings.value.selectedLoopbackDevice && devices.length > 0) {
        await autoSelectBestDevice()
      }
      
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      devicesError.value = message
      console.error('❌ Failed to enumerate audio devices:', error)
    } finally {
      isLoadingDevices.value = false
    }
  }

  const autoSelectBestDevice = async (): Promise<void> => {
    try {
      const bestDevice = await invoke<AudioLoopbackDevice | null>('auto_select_best_device')
      if (bestDevice) {
        audioSettings.value.selectedLoopbackDevice = bestDevice.id
        console.log('🎯 Auto-selected audio device:', bestDevice.name)
        return
      }
      
      // Fallback: select first available device
      if (audioDevices.value.length > 0) {
        audioSettings.value.selectedLoopbackDevice = audioDevices.value[0].id
        console.log('🎯 Fallback selected:', audioDevices.value[0].name)
      }
    } catch (error) {
      console.error('Failed to auto-select device:', error)
    }
  }

  const testAudioDevice = async (deviceId: string): Promise<boolean> => {
    try {
      const result = await invoke<boolean>('test_audio_device', { deviceId })
      console.log(result ? '✅ Audio device test successful' : '❌ Audio device test failed')
      return result
    } catch (error) {
      console.error('Audio device test error:', error)
      return false
    }
  }

  // Settings management
  const loadAudioSettings = async (): Promise<void> => {
    try {
      const settings = await invoke<AudioDeviceSettings | null>('load_audio_settings')
      if (settings) {
        audioSettings.value = settings
        console.log('📂 Audio settings loaded:', settings)
      }
    } catch (error) {
      console.error('Failed to load audio settings:', error)
    }
  }

  const saveAudioSettings = async (): Promise<void> => {
    try {
      await invoke('save_audio_settings', { settings: audioSettings.value })
      console.log('💾 Audio settings saved')
    } catch (error) {
      console.error('Failed to save audio settings:', error)
    }
  }

  // Audio capture management
  const startAudioCapture = async (): Promise<void> => {
    if (!audioSettings.value.selectedLoopbackDevice) {
      throw new Error('No audio device selected')
    }

    if (!audioSettings.value.loopbackEnabled) {
      throw new Error('Audio loopback is disabled')
    }

    try {
      console.log('🎤 Starting audio loopback capture...')
      
      // Setup audio chunk listener
      await setupAudioChunkListener()
      
      // Start capture
      await invoke('start_audio_loopback_capture', {
        deviceId: audioSettings.value.selectedLoopbackDevice
      })
      
      isCapturing.value = true
      captureError.value = null
      console.log('✅ Audio capture started')
      
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      captureError.value = message
      console.error('❌ Failed to start audio capture:', error)
      throw error
    }
  }

  const stopAudioCapture = async (): Promise<void> => {
    try {
      console.log('⏹️ Stopping audio capture...')
      
      await invoke('stop_audio_loopback_capture')
      
      isCapturing.value = false
      captureError.value = null
      
      // Clear buffers
      audioChunkBuffer.value = []
      processingQueue.value = []
      
      console.log('✅ Audio capture stopped')
      
    } catch (error) {
      console.error('Failed to stop audio capture:', error)
      throw error
    }
  }

  // Audio processing
  const setupAudioChunkListener = async (): Promise<void> => {
    try {
      // Listen for audio chunks from Rust backend
      await listen<AudioChunkData>('audio-chunk', (event) => {
        const audioChunk = event.payload
        
        // Add to buffer
        audioChunkBuffer.value.push(audioChunk)
        
        // Keep buffer size manageable (last 10 seconds at ~10 chunks/second)
        if (audioChunkBuffer.value.length > 100) {
          audioChunkBuffer.value = audioChunkBuffer.value.slice(-100)
        }
        
        // Add to processing queue for transcription
        processingQueue.value.push(audioChunk)
        
        // Process transcription if queue has enough data
        processTranscriptionQueue()
      })
      
      console.log('🎧 Audio chunk listener setup complete')
    } catch (error) {
      console.error('Failed to setup audio chunk listener:', error)
    }
  }

  const processTranscriptionQueue = async (): Promise<void> => {
    // Process in batches to avoid overwhelming the transcription service
    if (processingQueue.value.length < 5) return
    
    try {
      // Take batch of audio chunks
      const batch = processingQueue.value.splice(0, 5)
      
      // Combine audio data (simple concatenation for now)
      const combinedAudioData = batch.map(chunk => chunk.audio_data).join('')
      
      // Process through transcription
      const transcription = await invoke<string>('process_audio_for_transcription', {
        audioData: Array.from(atob(combinedAudioData)).map(c => c.charCodeAt(0)),
        sampleRate: audioSettings.value.sampleRate
      })
      
      if (transcription && transcription.trim()) {
        transcriptionResults.value.push(transcription)
        console.log('📝 Transcription:', transcription)
        
        // Keep only last 50 transcriptions
        if (transcriptionResults.value.length > 50) {
          transcriptionResults.value = transcriptionResults.value.slice(-50)
        }
      }
      
    } catch (error) {
      console.error('Failed to process transcription:', error)
    }
  }

  // Device utility functions
  const getDeviceDisplayName = (device: AudioLoopbackDevice): string => {
    return device.name
  }

  const getDeviceMethodBadge = (method: string) => {
    switch (method) {
      case 'RenderLoopback':
        return { text: 'Render Loopback', class: 'bg-green-500/20 text-green-300 border-green-400/30' }
      case 'StereoMix':
        return { text: 'Stereo Mix', class: 'bg-blue-500/20 text-blue-300 border-blue-400/30' }
      case 'CaptureDevice':
        return { text: 'Capture Device', class: 'bg-purple-500/20 text-purple-300 border-purple-400/30' }
      default:
        return { text: method, class: 'bg-gray-500/20 text-gray-300 border-gray-400/30' }
    }
  }

  const formatAudioSpecs = (device: AudioLoopbackDevice): string => {
    return `${device.sample_rate} Hz, ${device.channels} ch, ${device.format}`
  }

  // Cleanup
  const cleanup = (): void => {
    if (isCapturing.value) {
      stopAudioCapture().catch(console.error)
    }
    audioChunkBuffer.value = []
    processingQueue.value = []
    transcriptionResults.value = []
  }

  // Initialize
  const initialize = async (): Promise<void> => {
    await loadAudioSettings()
    await enumerateAudioDevices()
  }

  return {
    // State
    audioDevices,
    isLoadingDevices,
    devicesError,
    isCapturing,
    captureError,
    audioSettings,
    audioChunkBuffer,
    transcriptionResults,
    
    // Computed
    selectedDevice,
    canStartCapture,
    deviceStats,
    
    // Methods
    enumerateAudioDevices,
    autoSelectBestDevice,
    testAudioDevice,
    loadAudioSettings,
    saveAudioSettings,
    startAudioCapture,
    stopAudioCapture,
    getDeviceDisplayName,
    getDeviceMethodBadge,
    formatAudioSpecs,
    cleanup,
    initialize
  }
}