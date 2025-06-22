<template>
  <div class="eye-tracking-test">
    <div class="test-header">
      <h2>Eye Tracking - Phase 1 Test</h2>
      <div class="status-indicators">
        <div class="status-item" :class="{ active: eyeTracking.isReady.value }">
          <span class="indicator"></span>
          System Ready
        </div>
        <div class="status-item" :class="{ active: eyeTracking.isActive.value }">
          <span class="indicator"></span>
          Tracking Active
        </div>
        <div class="status-item" :class="{ active: eyeTracking.faceDetected.value }">
          <span class="indicator"></span>
          Face Detected
        </div>
      </div>
    </div>

    <!-- Camera View -->
    <div class="camera-section">
      <div class="camera-container">
        <video 
          ref="videoRef" 
          class="camera-video"
          autoplay 
          muted 
          playsinline
          controls="false"
          preload="metadata"
        ></video>
        <div v-if="eyeTracking.currentGaze.value" class="gaze-overlay">
          <div 
            class="gaze-indicator"
            :style="gazeIndicatorStyle"
          ></div>
        </div>
      </div>
      
      <!-- Camera Controls -->
      <div class="camera-controls">
        <button 
          @click="startTracking" 
          :disabled="eyeTracking.isActive.value"
          class="btn btn-start"
        >
          Start Tracking
        </button>
        <button 
          @click="stopTracking" 
          :disabled="!eyeTracking.isActive.value"
          class="btn btn-stop"
        >
          Stop Tracking
        </button>
      </div>
    </div>

    <!-- Information Panel -->
    <div class="info-panel">
      <div class="info-section">
        <h3>Tracking Status</h3>
        <div class="status-grid">
          <div class="status-row">
            <span>Quality:</span>
            <span class="value" :class="qualityClass">{{ eyeTracking.trackingQuality.value }}</span>
          </div>
          <div class="status-row">
            <span>Confidence:</span>
            <span class="value">{{ confidencePercent }}%</span>
          </div>
          <div class="status-row">
            <span>Frame Rate:</span>
            <span class="value">{{ eyeTracking.frameRate.value }} FPS</span>
          </div>
          <div class="status-row">
            <span>Processing:</span>
            <span class="value">{{ eyeTracking.isProcessing.value ? 'Active' : 'Idle' }}</span>
          </div>
        </div>
      </div>

      <div class="info-section">
        <h3>Gaze Data</h3>
        <div class="gaze-data" v-if="eyeTracking.currentGaze.value">
          <div class="gaze-coords">
            <div>X: {{ gazeX }}</div>
            <div>Y: {{ gazeY }}</div>
          </div>
          <div class="screen-coords" v-if="screenPosition">
            <div>Screen X: {{ Math.round(screenPosition.x) }}px</div>
            <div>Screen Y: {{ Math.round(screenPosition.y) }}px</div>
          </div>
        </div>
        <div v-else class="no-gaze">
          No gaze data available
        </div>
      </div>

      <div class="info-section">
        <h3>Settings</h3>
        <div class="settings-grid">
          <div class="setting-row">
            <label>Frame Rate:</label>
            <input 
              type="range" 
              min="5" 
              max="30" 
              :value="eyeTracking.frameRate.value"
              @input="setFrameRate"
            >
            <span>{{ eyeTracking.frameRate.value }} FPS</span>
          </div>
          <div class="setting-row">
            <label>Smoothing:</label>
            <input 
              type="range" 
              min="1" 
              max="10" 
              :value="eyeTracking.smoothingWindow.value"
              @input="setSmoothingWindow"
            >
            <span>{{ eyeTracking.smoothingWindow.value }} frames</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Error Display -->
    <div v-if="eyeTracking.hasError.value" class="error-panel">
      <h3>Error</h3>
      <p>{{ eyeTracking.state.error }}</p>
      <button @click="clearError" class="btn btn-clear">Clear Error</button>
    </div>

    <!-- Debug Info -->
    <div class="debug-panel">
      <h3>Debug Information</h3>
      <details>
        <summary>Camera State</summary>
        <pre>{{ JSON.stringify(eyeTracking.cameraState, null, 2) }}</pre>
      </details>
      <details>
        <summary>Computer Vision State</summary>
        <pre>{{ JSON.stringify(eyeTracking.cvState, null, 2) }}</pre>
      </details>
      <details>
        <summary>Eye Tracking State</summary>
        <pre>{{ JSON.stringify(eyeTracking.state, null, 2) }}</pre>
      </details>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from 'vue'
import { useEyeTracking } from '../../composables/useEyeTracking'

// Eye tracking composable
const eyeTracking = useEyeTracking()

// Template refs
const videoRef = ref<HTMLVideoElement | null>(null)

// Computed properties
const confidencePercent = computed(() => {
  return Math.round((eyeTracking.confidence.value || 0) * 100)
})

const qualityClass = computed(() => {
  const quality = eyeTracking.trackingQuality.value
  return {
    'quality-excellent': quality === 'excellent',
    'quality-good': quality === 'good',
    'quality-fair': quality === 'fair',
    'quality-poor': quality === 'poor',
    'quality-inactive': quality === 'inactive' || quality === 'no-face'
  }
})

const gazeX = computed(() => {
  return eyeTracking.currentGaze.value?.x?.toFixed(3) || 'N/A'
})

const gazeY = computed(() => {
  return eyeTracking.currentGaze.value?.y?.toFixed(3) || 'N/A'
})

const screenPosition = computed(() => {
  return eyeTracking.currentScreenPosition.value
})

const gazeIndicatorStyle = computed(() => {
  if (!eyeTracking.currentGaze.value) return {}
  
  const gaze = eyeTracking.currentGaze.value
  // Convert normalized gaze (-1 to 1) to percentage (0 to 100)
  const x = ((gaze.x + 1) / 2) * 100
  const y = ((gaze.y + 1) / 2) * 100
  
  return {
    left: `${x}%`,
    top: `${y}%`,
    opacity: gaze.confidence
  }
})

// Methods
const startTracking = async () => {
  console.log('Starting eye tracking...')
  
  // First attach the video element to the camera manager
  if (videoRef.value) {
    eyeTracking.attachVideoElement(videoRef.value)
    console.log('Video element attached to camera manager')
  }
  
  const success = await eyeTracking.startTracking()
  console.log('Tracking start result:', success)
  
  if (!success) {
    console.error('Failed to start eye tracking')
  }
}

const stopTracking = async () => {
  await eyeTracking.stopTracking()
}

const setFrameRate = (event: Event) => {
  const target = event.target as HTMLInputElement
  eyeTracking.setFrameRate(parseInt(target.value))
}

const setSmoothingWindow = (event: Event) => {
  const target = event.target as HTMLInputElement
  eyeTracking.setSmoothingWindow(parseInt(target.value))
}

const clearError = () => {
  // Note: In a production app, you'd want a proper method to clear errors
  // For now, we'll restart tracking to clear the error state
  if (eyeTracking.isActive.value) {
    eyeTracking.stopTracking()
  }
}

// Initialize on mount
onMounted(async () => {
  console.log('Eye Tracking Test Component mounted')
  console.log('Phase 1: Foundation - Camera Integration & Basic Computer Vision')
  
  // Attach video element right away
  if (videoRef.value) {
    await nextTick()
    eyeTracking.attachVideoElement(videoRef.value)
    console.log('Video element attached on mount')
  }
  
  // Log initial states
  console.log('Initial camera state:', eyeTracking.cameraState)
  console.log('Initial CV state:', eyeTracking.cvState)
  console.log('Is eye tracking ready:', eyeTracking.isReady.value)
})
</script>

<style scoped>
.eye-tracking-test {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
  font-family: 'IBM Plex Mono', monospace;
}

.test-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 30px;
  padding-bottom: 20px;
  border-bottom: 2px solid #333;
}

.test-header h2 {
  color: #fff;
  margin: 0;
}

.status-indicators {
  display: flex;
  gap: 20px;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #666;
  transition: color 0.3s;
}

.status-item.active {
  color: #00ff88;
}

.indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #333;
  transition: background 0.3s;
}

.status-item.active .indicator {
  background: #00ff88;
  box-shadow: 0 0 10px #00ff88;
}

.camera-section {
  margin-bottom: 30px;
}

.camera-container {
  position: relative;
  width: 640px;
  height: 480px;
  margin: 0 auto 20px;
  border: 2px solid #333;
  border-radius: 8px;
  overflow: hidden;
  background: #000;
}

.camera-video {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.gaze-overlay {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
}

.gaze-indicator {
  position: absolute;
  width: 20px;
  height: 20px;
  background: #00ff88;
  border: 2px solid #fff;
  border-radius: 50%;
  transform: translate(-50%, -50%);
  box-shadow: 0 0 15px #00ff88;
  transition: all 0.1s ease-out;
}

.camera-controls {
  display: flex;
  justify-content: center;
  gap: 15px;
}

.btn {
  padding: 12px 24px;
  border: 2px solid #333;
  background: transparent;
  color: #fff;
  cursor: pointer;
  border-radius: 6px;
  font-family: inherit;
  font-weight: 500;
  transition: all 0.3s;
}

.btn:hover:not(:disabled) {
  background: #333;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-start {
  border-color: #00ff88;
  color: #00ff88;
}

.btn-start:hover:not(:disabled) {
  background: #00ff88;
  color: #000;
}

.btn-stop {
  border-color: #ff4444;
  color: #ff4444;
}

.btn-stop:hover:not(:disabled) {
  background: #ff4444;
  color: #fff;
}

.info-panel {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 20px;
  margin-bottom: 30px;
}

.info-section {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid #333;
  border-radius: 8px;
  padding: 20px;
}

.info-section h3 {
  margin: 0 0 15px 0;
  color: #fff;
  font-size: 16px;
}

.status-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.status-row {
  display: flex;
  justify-content: space-between;
}

.value {
  font-weight: bold;
}

.quality-excellent { color: #00ff88; }
.quality-good { color: #88ff00; }
.quality-fair { color: #ffaa00; }
.quality-poor { color: #ff4444; }
.quality-inactive { color: #666; }

.gaze-data {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.gaze-coords, .screen-coords {
  display: flex;
  gap: 20px;
}

.no-gaze {
  color: #666;
  font-style: italic;
}

.settings-grid {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.setting-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.setting-row label {
  min-width: 80px;
}

.setting-row input[type="range"] {
  flex: 1;
}

.error-panel {
  background: rgba(255, 68, 68, 0.1);
  border: 2px solid #ff4444;
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 30px;
}

.error-panel h3 {
  color: #ff4444;
  margin: 0 0 10px 0;
}

.error-panel p {
  margin: 0 0 15px 0;
  color: #fff;
}

.btn-clear {
  border-color: #ff4444;
  color: #ff4444;
}

.debug-panel {
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid #222;
  border-radius: 8px;
  padding: 20px;
}

.debug-panel h3 {
  margin: 0 0 15px 0;
  color: #fff;
}

.debug-panel details {
  margin-bottom: 15px;
}

.debug-panel summary {
  cursor: pointer;
  color: #00ff88;
  font-weight: bold;
  margin-bottom: 10px;
}

.debug-panel pre {
  background: #000;
  padding: 15px;
  border-radius: 4px;
  color: #fff;
  font-size: 12px;
  overflow-x: auto;
}
</style> 