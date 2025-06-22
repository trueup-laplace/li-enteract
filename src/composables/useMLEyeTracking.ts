import { ref, computed, onUnmounted, readonly } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Types for ML eye tracking
export interface MLGazeData {
  x: number
  y: number
  confidence: number
  timestamp: number
  calibrated: boolean
}

export interface MLEyeTrackingConfig {
  camera_id: number
  screen_width: number
  screen_height: number
  model_path?: string
  smoothing_window: number
}

export interface MLTrackingStats {
  status: string
  model_type: string
  features: string[]
  performance: {
    expected_fps: string
    latency_ms: string
    accuracy: string
  }
}

export function useMLEyeTracking() {
  // State
  const isActive = ref(false)
  const currentGaze = ref<MLGazeData | null>(null)
  const isCalibrated = ref(false)
  const trackingStats = ref<MLTrackingStats | null>(null)
  const error = ref<string | null>(null)
  const isLoading = ref(false)

  // Tracking configuration
  const config = ref<MLEyeTrackingConfig>({
    camera_id: 0,
    screen_width: window.screen.width,
    screen_height: window.screen.height,
    smoothing_window: 5
  })

  // Performance metrics
  const fps = ref(0)
  const accuracy = ref(0)
  const lastUpdateTime = ref(0)

  // Tracking loop management
  let trackingInterval: number | null = null
  let fpsInterval: number | null = null
  let frameCount = 0

  // Computed properties
  const gazeScreenPosition = computed(() => {
    if (!currentGaze.value) return null
    return {
      x: Math.max(0, Math.min(currentGaze.value.x, config.value.screen_width)),
      y: Math.max(0, Math.min(currentGaze.value.y, config.value.screen_height))
    }
  })

  const normalizedGaze = computed(() => {
    if (!gazeScreenPosition.value) return null
    return {
      x: gazeScreenPosition.value.x / config.value.screen_width,
      y: gazeScreenPosition.value.y / config.value.screen_height
    }
  })

  const confidence = computed(() => {
    return currentGaze.value?.confidence || 0
  })

  const isHighConfidence = computed(() => {
    return confidence.value > 0.7
  })

  // Core functions
  async function startTracking(userConfig?: Partial<MLEyeTrackingConfig>) {
    if (isActive.value) {
      console.warn('ML Eye tracking already active')
      return
    }

    try {
      isLoading.value = true
      error.value = null

      // Update configuration
      if (userConfig) {
        config.value = { ...config.value, ...userConfig }
      }

      // Start the ML eye tracking process
      const result = await invoke<string>('start_ml_eye_tracking', { 
        config: config.value 
      })
      
      console.log('✅ ML Eye tracking started:', result)
      console.log('🔧 Configuration:', config.value)
      
      isActive.value = true
      
      // Start data polling loop
      startDataPolling()
      
      // Start FPS monitoring
      startFPSMonitoring()
      
      // Get initial stats
      await updateStats()
      
      console.log('🚀 ML Eye tracking fully initialized!')
      console.log('📈 Watch for real-time data below...')

    } catch (err) {
      console.error('Failed to start ML eye tracking:', err)
      error.value = err as string
    } finally {
      isLoading.value = false
    }
  }

  async function stopTracking() {
    if (!isActive.value) return

    try {
      isLoading.value = true
      
      // Stop data polling
      stopDataPolling()
      
      // Stop the ML tracking process
      const result = await invoke<string>('stop_ml_eye_tracking')
      console.log('ML Eye tracking stopped:', result)
      
      isActive.value = false
      currentGaze.value = null
      isCalibrated.value = false
      
    } catch (err) {
      console.error('Failed to stop ML eye tracking:', err)
      error.value = err as string
    } finally {
      isLoading.value = false
    }
  }

  async function calibrate() {
    if (!isActive.value) {
      error.value = 'Cannot calibrate: tracking not active'
      return false
    }

    try {
      isLoading.value = true
      
      const result = await invoke<string>('calibrate_ml_eye_tracking')
      console.log('ML Eye tracking calibration:', result)
      
      // Wait a moment for calibration to complete
      await new Promise(resolve => setTimeout(resolve, 2000))
      
      isCalibrated.value = true
      return true
      
    } catch (err) {
      console.error('Failed to calibrate ML eye tracking:', err)
      error.value = err as string
      return false
    } finally {
      isLoading.value = false
    }
  }

  // Data polling
  function startDataPolling() {
    if (trackingInterval) return

    console.log('🔄 Starting ML data polling at 30 FPS...')
    
    trackingInterval = window.setInterval(async () => {
      try {
        const gazeData = await invoke<MLGazeData | null>('get_ml_gaze_data')
        
        if (gazeData) {
          currentGaze.value = gazeData
          isCalibrated.value = gazeData.calibrated
          lastUpdateTime.value = Date.now()
          frameCount++
          
          // Update accuracy based on confidence
          accuracy.value = gazeData.confidence
          
          // Add to history for analytics
          addToHistory(gazeData)
          
          // Log every 30 frames (once per second)
          if (frameCount % 30 === 0) {
            console.log(`📊 ML Tracking: x=${gazeData.x.toFixed(1)}, y=${gazeData.y.toFixed(1)}, conf=${(gazeData.confidence * 100).toFixed(1)}%, FPS=${fps.value}`)
          }
        } else {
          console.warn('⚠️ No ML gaze data received')
        }
        
        // Clear error if we're getting data
        if (gazeData && error.value) {
          error.value = null
        }
        
      } catch (err) {
        console.error('❌ Error getting ML gaze data:', err)
        error.value = err as string
      }
    }, 33) // ~30 FPS polling
  }

  function stopDataPolling() {
    if (trackingInterval) {
      clearInterval(trackingInterval)
      trackingInterval = null
    }
    
    if (fpsInterval) {
      clearInterval(fpsInterval)
      fpsInterval = null
    }
  }

  function startFPSMonitoring() {
    frameCount = 0
    
    fpsInterval = window.setInterval(() => {
      fps.value = frameCount
      frameCount = 0
    }, 1000)
  }

  async function updateStats() {
    try {
      const stats = await invoke<MLTrackingStats>('get_ml_tracking_stats')
      trackingStats.value = stats
    } catch (err) {
      console.error('Failed to get ML tracking stats:', err)
    }
  }

  // Movement detection
  const previousGaze = ref<MLGazeData | null>(null)
  const movementThreshold = 50 // pixels

  const hasSignificantMovement = computed(() => {
    if (!currentGaze.value || !previousGaze.value) return false
    
    const dx = currentGaze.value.x - previousGaze.value.x
    const dy = currentGaze.value.y - previousGaze.value.y
    const distance = Math.sqrt(dx * dx + dy * dy)
    
    return distance > movementThreshold
  })

  // Update previous gaze for movement detection
  function updatePreviousGaze() {
    if (currentGaze.value) {
      previousGaze.value = { ...currentGaze.value }
    }
  }

  // Configuration helpers
  function updateConfig(newConfig: Partial<MLEyeTrackingConfig>) {
    config.value = { ...config.value, ...newConfig }
  }

  function resetConfig() {
    config.value = {
      camera_id: 0,
      screen_width: window.screen.width,
      screen_height: window.screen.height,
      smoothing_window: 5
    }
  }

  // Performance optimization
  const gazeHistory = ref<MLGazeData[]>([])
  const maxHistorySize = 30

  function addToHistory(gaze: MLGazeData) {
    gazeHistory.value.push(gaze)
    if (gazeHistory.value.length > maxHistorySize) {
      gazeHistory.value.shift()
    }
  }

  const averageConfidence = computed(() => {
    if (gazeHistory.value.length === 0) return 0
    const sum = gazeHistory.value.reduce((acc, gaze) => acc + gaze.confidence, 0)
    return sum / gazeHistory.value.length
  })

  // Stability detection
  const isStable = computed(() => {
    if (gazeHistory.value.length < 10) return false
    
    const recent = gazeHistory.value.slice(-10)
    const avgX = recent.reduce((acc, g) => acc + g.x, 0) / recent.length
    const avgY = recent.reduce((acc, g) => acc + g.y, 0) / recent.length
    
    const variance = recent.reduce((acc, g) => {
      const dx = g.x - avgX
      const dy = g.y - avgY
      return acc + (dx * dx + dy * dy)
    }, 0) / recent.length
    
    return variance < 1000 // Threshold for stability
  })

  // Cleanup
  onUnmounted(() => {
    stopTracking()
  })

  // Return the composable interface
  return {
    // State
    isActive: readonly(isActive),
    currentGaze: readonly(currentGaze),
    isCalibrated: readonly(isCalibrated),
    trackingStats: readonly(trackingStats),
    error: readonly(error),
    isLoading: readonly(isLoading),
    
    // Computed
    gazeScreenPosition,
    normalizedGaze,
    confidence,
    isHighConfidence,
    hasSignificantMovement,
    isStable,
    averageConfidence,
    
    // Metrics
    fps: readonly(fps),
    accuracy: readonly(accuracy),
    
    // Configuration
    config: readonly(config),
    updateConfig,
    resetConfig,
    
    // Core functions
    startTracking,
    stopTracking,
    calibrate,
    updateStats,
    updatePreviousGaze,
    
    // Utilities
    addToHistory
  }
} 