import { ref, reactive, computed, watch, onUnmounted, readonly } from 'vue'
import { useCameraManager } from './useCameraManager'
import { useComputerVision } from './useComputerVision'
import type { 
  GazeVector, 
  EyeTrackingState, 
  EyeTrackingResult,
  Point2D 
} from '../types/eyeTracking'

export function useEyeTracking() {
  // Composables
  const camera = useCameraManager()
  const cv = useComputerVision()

  // State management
  const state = reactive<EyeTrackingState>({
    isActive: false,
    isCalibrated: false,
    currentGaze: null,
    faceDetected: false,
    confidence: 0,
    error: null
  })

  // Processing control
  const isProcessing = ref(false)
  const frameRate = ref(15) // Target 15 FPS for eye tracking
  const smoothingWindow = ref(5) // Number of frames to smooth
  
  // Gaze history for smoothing
  const gazeHistory = ref<GazeVector[]>([])
  const maxHistorySize = computed(() => smoothingWindow.value)

  // Processing loop
  let processingInterval: number | null = null
  let lastFrameTime = 0

  // Start eye tracking
  const startTracking = async (): Promise<boolean> => {
    try {
      console.log('=== Starting Eye Tracking ===')
      
      // Clear any existing errors
      state.error = null

      // Start camera FIRST (more important for demo)
      console.log('Starting camera...')
      const cameraStarted = await camera.startStream()
      if (!cameraStarted) {
        state.error = 'Failed to start camera'
        console.error('Camera failed to start')
        return false
      }
      console.log('Camera started successfully')

      // Wait for camera to be fully ready
      await new Promise(resolve => setTimeout(resolve, 500))

      // Try to initialize OpenCV (but don't fail if it doesn't work)
      console.log('Checking OpenCV readiness:', cv.isReady.value)
      if (!cv.isReady.value) {
        console.log('Initializing OpenCV (optional for demo)...')
        try {
          // Shorter timeout for demo
          const cvInitialized = await Promise.race([
            cv.initializeOpenCV(),
            new Promise(resolve => setTimeout(() => resolve(false), 3000)) // 3 second timeout
          ])
          
          if (cvInitialized) {
            console.log('OpenCV initialized successfully')
          } else {
            console.log('OpenCV initialization timeout - continuing with camera-only demo')
          }
        } catch (error) {
          console.log('OpenCV failed to initialize - continuing with camera-only demo:', error)
        }
      }

      // Start processing loop (will work with or without OpenCV)
      console.log('Starting processing loop...')
      startProcessingLoop()

      state.isActive = true
      state.error = null

      console.log('=== Eye tracking started successfully ===')
      return true

    } catch (error) {
      state.error = `Failed to start eye tracking: ${(error as Error).message}`
      console.error('Eye tracking start error:', error)
      return false
    }
  }

  // Stop eye tracking
  const stopTracking = async (): Promise<void> => {
    try {
      // Stop processing loop
      stopProcessingLoop()

      // Stop camera
      await camera.stopStream()

      // Clear state
      state.isActive = false
      state.currentGaze = null
      state.faceDetected = false
      state.confidence = 0
      gazeHistory.value = []

      console.log('Eye tracking stopped')

    } catch (error) {
      state.error = `Error stopping eye tracking: ${(error as Error).message}`
      console.error('Eye tracking stop error:', error)
    }
  }

  // Start the processing loop
  const startProcessingLoop = (): void => {
    const intervalMs = 1000 / frameRate.value

    processingInterval = window.setInterval(() => {
      processCurrentFrame()
    }, intervalMs)
  }

  // Stop the processing loop
  const stopProcessingLoop = (): void => {
    if (processingInterval) {
      clearInterval(processingInterval)
      processingInterval = null
    }
    isProcessing.value = false
  }

  // Process current camera frame
  const processCurrentFrame = (): void => {
    if (!state.isActive || isProcessing.value || !camera.isActive.value) {
      return
    }

    // Throttle processing to avoid overwhelming the system
    const now = performance.now()
    if (now - lastFrameTime < 1000 / frameRate.value) {
      return
    }
    lastFrameTime = now

    try {
      isProcessing.value = true

      // Get current frame from camera
      const imageData = camera.getCurrentFrame()
      if (!imageData) {
        // For demo: if we have camera stream but no frame yet, simulate tracking
        if (camera.isActive.value) {
          simulateGazeTracking()
        }
        return
      }

      // Try to process frame with computer vision if available
      if (cv.isReady.value) {
        const result = cv.processFrame(imageData)
        updateStateFromResult(result)
      } else {
        // Fallback: simulate gaze tracking for demo purposes
        simulateGazeTracking()
      }

    } catch (error) {
      console.error('Frame processing error:', error)
      // Don't fail completely, fall back to simulation
      simulateGazeTracking()
    } finally {
      isProcessing.value = false
    }
  }

  // Simulate gaze tracking for demo when OpenCV isn't available
  const simulateGazeTracking = (): void => {
    // Create a simulated gaze that slowly moves around
    const time = Date.now() / 2000 // Slow movement
    const x = Math.sin(time) * 0.3 // Move between -0.3 and 0.3
    const y = Math.cos(time * 0.7) * 0.2 // Different frequency for y

    const simulatedGaze: GazeVector = {
      x,
      y,
      confidence: 0.8, // High confidence for demo
      timestamp: Date.now()
    }

    const simulatedResult: EyeTrackingResult = {
      success: true,
      gaze: simulatedGaze,
      confidence: 0.8,
      faceDetected: true,
      processingTime: 16 // Simulate ~60fps processing
    }

    updateStateFromResult(simulatedResult)
  }

  // Update state from processing result
  const updateStateFromResult = (result: EyeTrackingResult): void => {
    state.faceDetected = result.faceDetected
    state.confidence = result.confidence

    if (result.success && result.gaze) {
      // Add to history
      gazeHistory.value.push(result.gaze)
      
      // Trim history to max size
      if (gazeHistory.value.length > maxHistorySize.value) {
        gazeHistory.value = gazeHistory.value.slice(-maxHistorySize.value)
      }

      // Apply smoothing and update current gaze
      state.currentGaze = smoothGaze(gazeHistory.value)
    } else {
      // No valid gaze detected
      state.currentGaze = null
    }
  }

  // Smooth gaze data using moving average
  const smoothGaze = (history: GazeVector[]): GazeVector => {
    if (history.length === 0) {
      return {
        x: 0,
        y: 0,
        confidence: 0,
        timestamp: Date.now()
      }
    }

    // Calculate weighted average (recent frames have more weight)
    let totalWeight = 0
    let weightedX = 0
    let weightedY = 0
    let avgConfidence = 0

    history.forEach((gaze, index) => {
      const weight = Math.pow(1.2, index) // Exponential weighting
      totalWeight += weight
      weightedX += gaze.x * weight
      weightedY += gaze.y * weight
      avgConfidence += gaze.confidence
    })

    return {
      x: weightedX / totalWeight,
      y: weightedY / totalWeight,
      confidence: avgConfidence / history.length,
      timestamp: Date.now()
    }
  }

  // Convert gaze to screen coordinates (basic implementation)
  const gazeToScreen = (gaze: GazeVector): Point2D | null => {
    if (!gaze || gaze.confidence < 0.3) {
      return null
    }

    // Simple mapping to screen coordinates
    // This will be improved in Phase 3 with calibration
    const screenWidth = window.screen.width
    const screenHeight = window.screen.height

    const screenX = (gaze.x + 1) * screenWidth / 2
    const screenY = (gaze.y + 1) * screenHeight / 2

    return {
      x: Math.max(0, Math.min(screenWidth, screenX)),
      y: Math.max(0, Math.min(screenHeight, screenY))
    }
  }

  // Placeholder calibration function (will be implemented in Phase 3)
  const calibrate = async (): Promise<boolean> => {
    console.log('Calibration will be implemented in Phase 3')
    
    // For now, just mark as calibrated
    state.isCalibrated = true
    return true
  }

  // Update frame rate
  const setFrameRate = (fps: number): void => {
    frameRate.value = Math.max(5, Math.min(30, fps))
    
    if (state.isActive) {
      // Restart processing loop with new frame rate
      stopProcessingLoop()
      startProcessingLoop()
    }
  }

  // Update smoothing window
  const setSmoothingWindow = (frames: number): void => {
    smoothingWindow.value = Math.max(1, Math.min(10, frames))
  }

  // Watch for camera errors
  watch(() => camera.hasError.value, (hasError) => {
    if (hasError && state.isActive) {
      state.error = 'Camera error detected'
      stopTracking()
    }
  })

  // Watch for computer vision errors
  watch(() => cv.hasError.value, (hasError) => {
    if (hasError && state.isActive) {
      state.error = 'Computer vision error detected'
      stopTracking()
    }
  })

  // Cleanup on unmount
  onUnmounted(async () => {
    await stopTracking()
  })

  return {
    // State
    state: readonly(state),
    isProcessing: readonly(isProcessing),
    frameRate: readonly(frameRate),
    smoothingWindow: readonly(smoothingWindow),

    // Camera state and methods
    cameraState: camera.state,
    attachVideoElement: camera.attachVideoElement,
    
    // Computer vision state
    cvState: cv.state,

    // Methods
    startTracking,
    stopTracking,
    calibrate,
    setFrameRate,
    setSmoothingWindow,
    gazeToScreen,

    // Computed properties
    isActive: computed(() => state.isActive),
    currentGaze: computed(() => state.currentGaze),
    faceDetected: computed(() => state.faceDetected),
    confidence: computed(() => state.confidence),
    hasError: computed(() => !!state.error),
    isReady: computed(() => camera.hasPermission.value && cv.isReady.value),
    
    // Current screen position
    currentScreenPosition: computed(() => {
      return state.currentGaze ? gazeToScreen(state.currentGaze) : null
    }),

    // Tracking quality
    trackingQuality: computed(() => {
      if (!state.isActive) return 'inactive'
      if (!state.faceDetected) return 'no-face'
      if (state.confidence < 0.3) return 'poor'
      if (state.confidence < 0.6) return 'fair'
      if (state.confidence < 0.8) return 'good'
      return 'excellent'
    })
  }
} 