import { ref, computed, watch, onUnmounted } from 'vue'
import { useEyeTracking } from './useEyeTracking'
import { useAdvancedGazeTracking } from './useAdvancedGazeTracking'
import { useWindowManager } from './useWindowManager'

interface GazeControlConfig {
  enabled: boolean
  autoStart: boolean
  requireCalibration: boolean
  movementThreshold: number    // Minimum gaze confidence to trigger movement
  stabilityTime: number        // Time gaze must be stable before movement (ms)
  cooldownTime: number         // Minimum time between movements (ms)
}

interface GazeControlState {
  isActive: boolean
  isCalibrated: boolean
  lastGazeTime: number
  lastMoveTime: number
  stabilityTimer: number | null
  gazeHistory: Array<{ x: number, y: number, timestamp: number }>
}

export function useGazeWindowControl() {
  // Composables
  const eyeTracking = useEyeTracking()
  const advancedGazeTracking = useAdvancedGazeTracking()
  const windowManager = useWindowManager()

  // Configuration
  const config = ref<GazeControlConfig>({
    enabled: false,
    autoStart: false,
    requireCalibration: false,
    movementThreshold: 0.6,
    stabilityTime: 200,
    cooldownTime: 100
  })

  // State
  const state = ref<GazeControlState>({
    isActive: false,
    isCalibrated: false,
    lastGazeTime: 0,
    lastMoveTime: 0,
    stabilityTimer: null,
    gazeHistory: []
  })

  // Statistics
  const stats = ref({
    totalMovements: 0,
    averageConfidence: 0,
    lastMovementTime: 0,
    sessionStartTime: 0
  })

  // Check if gaze is stable enough for movement
  const isGazeStable = (currentGaze: { x: number, y: number, confidence: number }): boolean => {
    const { stabilityTime, movementThreshold } = config.value
    const now = Date.now()

    // Add current gaze to history
    state.value.gazeHistory.push({
      x: currentGaze.x,
      y: currentGaze.y,
      timestamp: now
    })

    // Keep only recent history
    state.value.gazeHistory = state.value.gazeHistory.filter(
      entry => now - entry.timestamp <= stabilityTime
    )

    // Check if we have enough stable data
    if (state.value.gazeHistory.length < 3) return false

    // Check confidence threshold
    if (currentGaze.confidence < movementThreshold) return false

    // Calculate variance in recent gaze positions
    const recentGazes = state.value.gazeHistory.slice(-5)
    const avgX = recentGazes.reduce((sum, g) => sum + g.x, 0) / recentGazes.length
    const avgY = recentGazes.reduce((sum, g) => sum + g.y, 0) / recentGazes.length

    const variance = recentGazes.reduce((sum, g) => {
      return sum + Math.pow(g.x - avgX, 2) + Math.pow(g.y - avgY, 2)
    }, 0) / recentGazes.length

    // Consider stable if variance is low
    return variance < 0.01 // Adjust threshold as needed
  }

  // Check cooldown period
  const canMove = (): boolean => {
    const { cooldownTime } = config.value
    const now = Date.now()
    return now - state.value.lastMoveTime >= cooldownTime
  }

  // Process gaze data and trigger window movement
  const processGazeForMovement = async (gazeData: any): Promise<void> => {
    if (!state.value.isActive || !config.value.enabled) return

    let gaze, confidence

    // Handle different gaze data formats
    if (advancedGazeTracking.isActive.value && advancedGazeTracking.currentGaze.value) {
      // Use advanced gaze tracking data
      const advancedGaze = advancedGazeTracking.currentGaze.value
      gaze = { x: advancedGaze.x, y: advancedGaze.y }
      confidence = advancedGaze.confidence
    } else if (gazeData && gazeData.success) {
      // Use basic eye tracking data
      gaze = gazeData.gaze
      confidence = gazeData.confidence
    } else {
      return
    }

    if (!gaze) return

    state.value.lastGazeTime = Date.now()

    // Check if gaze is stable and confident enough
    if (!isGazeStable({ x: gaze.x, y: gaze.y, confidence })) {
      return
    }

    // Check cooldown
    if (!canMove()) {
      return
    }

    try {
      // Process gaze input through window manager
      await windowManager.processGazeInput(gaze)
      
      // Update movement statistics
      state.value.lastMoveTime = Date.now()
      stats.value.totalMovements++
      stats.value.lastMovementTime = Date.now()
      
      // Update average confidence (rolling average)
      stats.value.averageConfidence = (stats.value.averageConfidence * 0.9) + (confidence * 0.1)

    } catch (error) {
      console.error('Error processing gaze for window movement:', error)
    }
  }

  // Start gaze-controlled window movement
  const startGazeControl = async (useAdvanced: boolean = true): Promise<boolean> => {
    try {
      console.log('Starting gaze-controlled window movement...')

      if (useAdvanced && advancedGazeTracking) {
        // Use the new advanced gaze tracking system
        if (!advancedGazeTracking.isActive.value) {
          const trackingStarted = await advancedGazeTracking.startTracking()
          if (!trackingStarted) {
            console.error('Failed to start advanced gaze tracking')
            // Fallback to basic eye tracking
            return startBasicGazeControl()
          }
        }
      } else {
        return startBasicGazeControl()
      }

      // Enable window movement
      await windowManager.enableGazeControl()

      // Update state
      state.value.isActive = true
      stats.value.sessionStartTime = Date.now()
      config.value.enabled = true

      console.log('Advanced gaze-controlled window movement started successfully')
      return true

    } catch (error) {
      console.error('Failed to start gaze control:', error)
      return false
    }
  }

  // Start basic gaze control as fallback
  const startBasicGazeControl = async (): Promise<boolean> => {
    try {
      console.log('Starting basic gaze-controlled window movement...')

      // Start eye tracking if not already active
      if (!eyeTracking.isActive.value) {
        const trackingStarted = await eyeTracking.startTracking()
        if (!trackingStarted) {
          console.error('Failed to start eye tracking')
          return false
        }
      }

      // Enable window movement
      await windowManager.enableGazeControl()

      // Update state
      state.value.isActive = true
      stats.value.sessionStartTime = Date.now()
      config.value.enabled = true

      console.log('Basic gaze-controlled window movement started successfully')
      return true

    } catch (error) {
      console.error('Failed to start basic gaze control:', error)
      return false
    }
  }

  // Stop gaze-controlled window movement
  const stopGazeControl = (): void => {
    console.log('Stopping gaze-controlled window movement...')

    // Disable window movement
    windowManager.disableGazeControl()

    // Clear timers
    if (state.value.stabilityTimer) {
      clearTimeout(state.value.stabilityTimer)
      state.value.stabilityTimer = null
    }

    // Update state
    state.value.isActive = false
    config.value.enabled = false
    state.value.gazeHistory = []

    console.log('Gaze-controlled window movement stopped')
  }

  // Toggle gaze control
  const toggleGazeControl = async (): Promise<boolean> => {
    if (state.value.isActive) {
      stopGazeControl()
      return false
    } else {
      return await startGazeControl()
    }
  }

  // Watch for advanced gaze data and process it
  watch(
    () => advancedGazeTracking.currentGaze.value,
    (newGazeData) => {
      if (newGazeData && state.value.isActive) {
        processGazeForMovement(newGazeData)
      }
    },
    { deep: true }
  )

  // Update configuration
  const updateConfig = (newConfig: Partial<GazeControlConfig>): void => {
    Object.assign(config.value, newConfig)
    
    // Update window manager config if needed
    if (newConfig.movementThreshold !== undefined) {
      windowManager.updateConfig({
        sensitivity: newConfig.movementThreshold
      })
    }
  }

  // Watch eye tracking data and process for movement
  watch(
    () => eyeTracking.currentGaze.value,
    (newGaze) => {
      if (newGaze && state.value.isActive) {
        // Create a tracking result-like object for processing
        const gazeData = {
          success: true,
          gaze: newGaze,
          confidence: newGaze.confidence,
          faceDetected: eyeTracking.faceDetected.value
        }
        processGazeForMovement(gazeData)
      }
    },
    { immediate: false }
  )

  // Auto-start if configured
  if (config.value.autoStart) {
    startGazeControl()
  }

  // Cleanup on unmount
  onUnmounted(() => {
    stopGazeControl()
  })

  // Computed properties
  const isReady = computed(() => 
    eyeTracking.isActive.value && windowManager.isEnabled.value
  )

  const canStart = computed(() => 
    eyeTracking.cameraState.isActive && !state.value.isActive
  )

  const movementStats = computed(() => ({
    ...stats.value,
    uptime: state.value.isActive ? Date.now() - stats.value.sessionStartTime : 0,
    movementsPerMinute: stats.value.sessionStartTime > 0 
      ? (stats.value.totalMovements / Math.max(1, (Date.now() - stats.value.sessionStartTime) / 60000))
      : 0,
    windowPosition: windowManager.state.value.position,
    isMoving: windowManager.isMoving.value
  }))

  const gazeControlStatus = computed(() => {
    if (!eyeTracking.cameraState.isActive) return 'Camera not active'
    if (!eyeTracking.isActive.value) return 'Eye tracking not active'
    if (!windowManager.isEnabled.value) return 'Window manager not enabled'
    if (!state.value.isActive) return 'Gaze control disabled'
    return 'Active'
  })

  return {
    // State
    state: computed(() => state.value),
    config: computed(() => config.value),
    stats: movementStats,
    
    // Status
    isActive: computed(() => state.value.isActive),
    isReady,
    canStart,
    status: gazeControlStatus,

    // Control methods
    startGazeControl,
    startBasicGazeControl,
    stopGazeControl,
    toggleGazeControl,
    updateConfig,

    // Data processing
    processGazeForMovement,
    isGazeStable,
    canMove,

    // Sub-composable access
    eyeTracking,
    advancedGazeTracking,
    windowManager
  }
} 