import { ref, reactive, onUnmounted, readonly, computed, watch } from 'vue'

// Types based on the implementation plan
interface CameraConfig {
  width: number          // 1280x720 recommended
  height: number
  frameRate: number      // 30 FPS target
  facingMode: 'user'     // Front-facing camera
  deviceId?: string      // Specific camera selection
}

interface CameraState {
  isActive: boolean
  stream: MediaStream | null
  error: string | null
  permissions: 'granted' | 'denied' | 'prompt'
}

interface CameraDevice {
  deviceId: string
  label: string
  kind: string
}

export function useCameraManager() {
  // Reactive state
  const state = reactive<CameraState>({
    isActive: false,
    stream: null,
    error: null,
    permissions: 'prompt'
  })

  const videoElement = ref<HTMLVideoElement | null>(null)
  const availableDevices = ref<CameraDevice[]>([])
  const currentConfig = ref<CameraConfig>({
    width: 1280,
    height: 720,
    frameRate: 30,
    facingMode: 'user'
  })

  // Camera lifecycle management functions
  const requestPermissions = async (): Promise<boolean> => {
    try {
      // Check if permissions API is available
      if (navigator.permissions) {
        const permission = await navigator.permissions.query({ name: 'camera' as PermissionName })
        state.permissions = permission.state
        
        // Listen for permission changes
        permission.onchange = () => {
          state.permissions = permission.state
        }
      }

      // Request access to test permissions
      const stream = await navigator.mediaDevices.getUserMedia({ video: true })
      stream.getTracks().forEach(track => track.stop()) // Stop test stream
      
      state.permissions = 'granted'
      state.error = null
      return true
    } catch (error) {
      state.permissions = 'denied'
      state.error = `Camera permission denied: ${(error as Error).message}`
      return false
    }
  }

  const getAvailableDevices = async (): Promise<CameraDevice[]> => {
    try {
      const devices = await navigator.mediaDevices.enumerateDevices()
      const videoDevices = devices
        .filter(device => device.kind === 'videoinput')
        .map(device => ({
          deviceId: device.deviceId,
          label: device.label || `Camera ${device.deviceId}`,
          kind: device.kind
        }))
      
      availableDevices.value = videoDevices
      return videoDevices
    } catch (error) {
      state.error = `Failed to enumerate devices: ${(error as Error).message}`
      return []
    }
  }

  const buildConstraints = (config: CameraConfig): MediaStreamConstraints => {
    return {
      video: {
        width: { ideal: config.width },
        height: { ideal: config.height },
        frameRate: { ideal: config.frameRate },
        facingMode: config.facingMode,
        ...(config.deviceId && { deviceId: config.deviceId })
      },
      audio: false
    }
  }

  const startStream = async (config?: Partial<CameraConfig>): Promise<boolean> => {
    try {
      console.log('Starting camera stream...')
      
      // Stop existing stream first
      if (state.stream) {
        console.log('Stopping existing stream')
        await stopStream()
      }

      // Update configuration
      if (config) {
        Object.assign(currentConfig.value, config)
      }
      console.log('Camera config:', currentConfig.value)

      // Check permissions first
      if (state.permissions !== 'granted') {
        console.log('Requesting camera permissions...')
        const hasPermission = await requestPermissions()
        if (!hasPermission) {
          console.error('Camera permission denied')
          return false
        }
      }

      // Build constraints and start stream
      const constraints = buildConstraints(currentConfig.value)
      console.log('Camera constraints:', constraints)
      
      const stream = await navigator.mediaDevices.getUserMedia(constraints)
      console.log('Got media stream:', stream)
      
      state.stream = stream
      state.isActive = true
      state.error = null

      // Video element attachment will be handled by the watch function
      console.log('Stream ready, video attachment will be handled automatically')

      console.log('Camera stream started successfully')
      return true
    } catch (error) {
      // Check if this is just a video play error but stream was created
      if (state.stream && error instanceof DOMException && error.name === 'AbortError') {
        console.log('Video play interrupted, but camera stream is active')
        state.isActive = true
        state.error = null
        return true // Consider this a success since we have the stream
      }
      
      const errorMsg = `Failed to start camera: ${(error as Error).message}`
      state.error = errorMsg
      state.isActive = false
      console.error('Camera start error:', error)
      return false
    }
  }

  const stopStream = async (): Promise<void> => {
    try {
      if (state.stream) {
        state.stream.getTracks().forEach(track => {
          track.stop()
        })
      }

      if (videoElement.value) {
        videoElement.value.srcObject = null
      }

      state.stream = null
      state.isActive = false
      state.error = null

      console.log('Camera stream stopped')
    } catch (error) {
      state.error = `Error stopping camera: ${(error as Error).message}`
    }
  }

  const handleDeviceChange = async (): Promise<void> => {
    try {
      // Re-enumerate devices
      await getAvailableDevices()
      
      // If current stream is using a device that's no longer available, restart
      if (state.isActive && state.stream) {
        const currentDeviceId = state.stream.getVideoTracks()[0]?.getSettings().deviceId
        const isDeviceAvailable = availableDevices.value.some(
          device => device.deviceId === currentDeviceId
        )
        
        if (!isDeviceAvailable) {
          console.log('Current camera device disconnected, attempting to reconnect...')
          await recoverFromError()
        }
      }
    } catch (error) {
      state.error = `Device change handling failed: ${(error as Error).message}`
    }
  }

  const recoverFromError = async (): Promise<boolean> => {
    try {
      console.log('Attempting camera recovery...')
      
      // Stop current stream
      await stopStream()
      
      // Wait a moment
      await new Promise(resolve => setTimeout(resolve, 1000))
      
      // Try to restart with default settings
      const recovered = await startStream()
      
      if (recovered) {
        console.log('Camera recovery successful')
      } else {
        console.log('Camera recovery failed')
      }
      
      return recovered
    } catch (error) {
      state.error = `Recovery failed: ${(error as Error).message}`
      return false
    }
  }

  const attachVideoElement = (element: HTMLVideoElement): void => {
    console.log('Attaching video element to camera manager')
    videoElement.value = element
    
    // The watch function will handle stream attachment automatically
    // when the stream becomes available
  }

  // Watch for stream changes and auto-attach to video element
  watch(() => state.stream, async (newStream: MediaStream | null) => {
    if (videoElement.value && newStream) {
      try {
        console.log('Auto-attaching stream to video element')
        
        // Clear any existing source first
        videoElement.value.srcObject = null
        
        // Small delay to ensure clean state
        await new Promise(resolve => setTimeout(resolve, 100))
        
        // Set new stream
        videoElement.value.srcObject = newStream
        
        // Wait for metadata to load before playing
        await new Promise((resolve, reject) => {
          const video = videoElement.value!
          
          const onLoadedMetadata = () => {
            video.removeEventListener('loadedmetadata', onLoadedMetadata)
            video.removeEventListener('error', onError)
            resolve(void 0)
          }
          
          const onError = (e: Event) => {
            video.removeEventListener('loadedmetadata', onLoadedMetadata)
            video.removeEventListener('error', onError)
            reject(e)
          }
          
          video.addEventListener('loadedmetadata', onLoadedMetadata)
          video.addEventListener('error', onError)
        })
        
        // Now play the video
        await videoElement.value.play()
        console.log('Video playing successfully')
        
      } catch (error) {
        console.error('Video attachment error:', error)
        // Don't fail the whole process for video issues
      }
    } else if (videoElement.value && !newStream) {
      // Clear video when stream is removed
      videoElement.value.srcObject = null
    }
  })

  const getCurrentFrame = (): ImageData | null => {
    if (!videoElement.value || !state.isActive) {
      console.log('getCurrentFrame: No video element or not active')
      return null
    }

    try {
      const video = videoElement.value
      
      // Check if video is ready
      if (video.readyState < 2) { // HAVE_CURRENT_DATA
        console.log('getCurrentFrame: Video not ready, readyState:', video.readyState)
        return null
      }
      
      if (video.videoWidth === 0 || video.videoHeight === 0) {
        console.log('getCurrentFrame: Video dimensions not available')
        return null
      }

      const canvas = document.createElement('canvas')
      const ctx = canvas.getContext('2d')
      if (!ctx) {
        console.log('getCurrentFrame: Could not get canvas context')
        return null
      }

      canvas.width = video.videoWidth
      canvas.height = video.videoHeight

      ctx.drawImage(video, 0, 0)
      const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height)
      
      console.log('getCurrentFrame: Successfully captured frame', canvas.width, 'x', canvas.height)
      return imageData
    } catch (error) {
      console.error('Failed to get current frame:', error)
      return null
    }
  }

  // Setup device change listener
  navigator.mediaDevices?.addEventListener?.('devicechange', handleDeviceChange)

  // Cleanup on unmount
  onUnmounted(async () => {
    await stopStream()
    navigator.mediaDevices?.removeEventListener?.('devicechange', handleDeviceChange)
  })

  // Initialize available devices
  getAvailableDevices()

  return {
    // State
    state: readonly(state),
    availableDevices: readonly(availableDevices),
    currentConfig: readonly(currentConfig),
    videoElement: readonly(videoElement),

    // Methods
    requestPermissions,
    getAvailableDevices,
    startStream,
    stopStream,
    handleDeviceChange,
    recoverFromError,
    attachVideoElement,
    getCurrentFrame,

    // Computed
    isActive: computed(() => state.isActive),
    hasError: computed(() => !!state.error),
    hasPermission: computed(() => state.permissions === 'granted'),
    currentStream: computed(() => state.stream)
  }
} 