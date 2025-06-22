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

      // Attach to video element if available
      if (videoElement.value) {
        console.log('Attaching stream to video element')
        videoElement.value.srcObject = stream
        await videoElement.value.play()
        console.log('Video element playing')
      } else {
        console.log('No video element available yet')
      }

      console.log('Camera stream started successfully')
      return true
    } catch (error) {
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
    videoElement.value = element
    
    if (state.stream) {
      element.srcObject = state.stream
      element.play().catch(console.error)
    }
  }

  // Watch for stream changes and auto-attach to video element
  watch(() => state.stream, (newStream: MediaStream | null) => {
    if (videoElement.value && newStream) {
      videoElement.value.srcObject = newStream
      videoElement.value.play().catch(console.error)
    }
  })

  const getCurrentFrame = (): ImageData | null => {
    if (!videoElement.value || !state.isActive) {
      return null
    }

    try {
      const canvas = document.createElement('canvas')
      const ctx = canvas.getContext('2d')
      if (!ctx) return null

      const video = videoElement.value
      canvas.width = video.videoWidth
      canvas.height = video.videoHeight

      ctx.drawImage(video, 0, 0)
      return ctx.getImageData(0, 0, canvas.width, canvas.height)
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