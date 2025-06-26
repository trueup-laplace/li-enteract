<template>
  <div v-if="visible" class="calibration-overlay">
    <!-- Calibration Info Modal -->
    <div v-if="!calibrationState.active" class="calibration-modal">
      <div class="modal-header">
        <h2>🎯 Gaze Calibration</h2>
        <p>Detected {{ monitors.length }} monitor(s)</p>
      </div>
      
      <div class="monitor-info">
        <h3>Monitor Layout</h3>
        <div v-for="(monitor, index) in monitors" :key="index" class="monitor-item">
          <span class="monitor-name">{{ monitor.name }}</span>
          <span class="monitor-details">
            {{ monitor.width }}×{{ monitor.height }} at ({{ monitor.x }}, {{ monitor.y }})
          </span>
          <span v-if="monitor.is_primary" class="primary-badge">[PRIMARY]</span>
        </div>
      </div>
      
      <div class="instructions">
        <p>Click 'Start Calibration' to begin.</p>
        <p>Look at each target that appears and press <kbd>SPACE</kbd>.</p>
        <p>This will improve gaze tracking accuracy across all monitors.</p>
      </div>
      
      <div class="button-group">
        <button @click="startCalibration" class="btn-primary">
          Start Calibration
        </button>
        <button @click="skipCalibration" class="btn-secondary">
          Skip Calibration
        </button>
      </div>
      
      <div class="status">
        {{ statusMessage }}
      </div>
    </div>
    
    <!-- Calibration Target -->
    <div v-if="calibrationState.active && currentTarget" class="calibration-target" 
         :style="targetStyle">
      <div class="target-circle">
        <div class="outer-ring"></div>
        <div class="middle-ring"></div>
        <div class="inner-dot"></div>
      </div>
      <div class="target-instructions">
        <p>Look at target {{ calibrationState.current_target + 1 }}/{{ calibrationState.total_targets }}</p>
        <p>Press <kbd>SPACE</kbd> when ready</p>
        <p class="escape-hint">Press <kbd>ESC</kbd> to cancel</p>
      </div>
    </div>
    
    <!-- Completion Modal -->
    <div v-if="calibrationState.completed" class="completion-modal">
      <div class="modal-header">
        <h2>✅ Calibration Complete!</h2>
        <p>Collected {{ calibrationState.points.length }} calibration points</p>
      </div>
      <button @click="closeCalibration" class="btn-primary">
        Close
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import type { MonitorInfo, CalibrationPoint, CalibrationState } from '../../types/monitor'

interface Props {
  visible: boolean
  monitors: MonitorInfo[]
}

interface Emits {
  (e: 'close'): void
  (e: 'completed', points: CalibrationPoint[]): void
  (e: 'skipped'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// Calibration state
const calibrationState = ref<CalibrationState>({
  active: false,
  current_target: 0,
  total_targets: 0,
  points: [],
  completed: false,
  skipped: false
})

const statusMessage = ref('Ready to calibrate')
const targets = ref<CalibrationPoint[]>([])

// Generate calibration targets (5 points per monitor: 4 corners + center)
const generateTargets = () => {
  targets.value = []
  
  props.monitors.forEach((monitor, monitorIndex) => {
    const margin = 50 // Margin from edges
    
    const targetPoints = [
      // Top-left
      { x: monitor.x + margin, y: monitor.y + margin },
      // Top-right  
      { x: monitor.x + monitor.width - margin, y: monitor.y + margin },
      // Bottom-left
      { x: monitor.x + margin, y: monitor.y + monitor.height - margin },
      // Bottom-right
      { x: monitor.x + monitor.width - margin, y: monitor.y + monitor.height - margin },
      // Center
      { x: monitor.x + monitor.width / 2, y: monitor.y + monitor.height / 2 }
    ]
    
    targetPoints.forEach(point => {
      targets.value.push({
        monitor_index: monitorIndex,
        target_x: point.x,
        target_y: point.y
      })
    })
  })
  
  calibrationState.value.total_targets = targets.value.length
}

// Current target being displayed
const currentTarget = computed(() => {
  if (!calibrationState.value.active) return null
  return targets.value[calibrationState.value.current_target] || null
})

// Target positioning style
const targetStyle = computed(() => {
  if (!currentTarget.value) return {}
  
  return {
    left: `${currentTarget.value.target_x - 100}px`,
    top: `${currentTarget.value.target_y - 100}px`
  }
})

// Start calibration process
const startCalibration = () => {
  generateTargets()
  calibrationState.value.active = true
  calibrationState.value.current_target = 0
  calibrationState.value.points = []
  statusMessage.value = 'Calibration in progress...'
  
  // Focus for keyboard events
  document.addEventListener('keydown', handleKeydown)
}

// Skip calibration
const skipCalibration = () => {
  calibrationState.value.skipped = true
  statusMessage.value = 'Calibration skipped'
  emit('skipped')
  closeCalibration()
}

// Handle keyboard input
const handleKeydown = (event: KeyboardEvent) => {
  if (!calibrationState.value.active) return
  
  if (event.code === 'Space') {
    event.preventDefault()
    recordCurrentTarget()
  } else if (event.code === 'Escape') {
    event.preventDefault()
    cancelCalibration()
  }
}

// Record current target position
const recordCurrentTarget = () => {
  if (!currentTarget.value) return
  
  const point: CalibrationPoint = {
    ...currentTarget.value,
    timestamp: Date.now()
  }
  
  calibrationState.value.points.push(point)
  calibrationState.value.current_target++
  
  // Check if we're done
  if (calibrationState.value.current_target >= calibrationState.value.total_targets) {
    finishCalibration()
  }
}

// Finish calibration successfully
const finishCalibration = () => {
  calibrationState.value.active = false
  calibrationState.value.completed = true
  statusMessage.value = `Calibration complete! (${calibrationState.value.points.length} points)`
  
  document.removeEventListener('keydown', handleKeydown)
  emit('completed', calibrationState.value.points)
  
  // Auto-close after 2 seconds
  setTimeout(() => {
    closeCalibration()
  }, 2000)
}

// Cancel calibration
const cancelCalibration = () => {
  calibrationState.value.active = false
  statusMessage.value = 'Calibration cancelled'
  document.removeEventListener('keydown', handleKeydown)
}

// Close calibration modal
const closeCalibration = () => {
  document.removeEventListener('keydown', handleKeydown)
  emit('close')
}

// Initialize when visible
watch(() => props.visible, (visible) => {
  if (visible) {
    generateTargets()
    statusMessage.value = 'Ready to calibrate'
    calibrationState.value = {
      active: false,
      current_target: 0,
      total_targets: targets.value.length,
      points: [],
      completed: false,
      skipped: false
    }
  }
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.calibration-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.9);
  z-index: 10000;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: 'IBM Plex Mono', monospace;
}

.calibration-modal,
.completion-modal {
  background: #1a1a1a;
  border: 2px solid #333;
  border-radius: 12px;
  padding: 2rem;
  max-width: 600px;
  width: 90%;
  color: white;
}

.modal-header h2 {
  margin: 0 0 0.5rem 0;
  color: #fff;
  font-size: 1.5rem;
}

.modal-header p {
  margin: 0;
  color: #aaa;
}

.monitor-info {
  margin: 1.5rem 0;
  padding: 1rem;
  background: #222;
  border-radius: 8px;
}

.monitor-info h3 {
  margin: 0 0 1rem 0;
  color: #fff;
  font-size: 1.1rem;
}

.monitor-item {
  display: flex;
  gap: 1rem;
  margin-bottom: 0.5rem;
  font-family: 'Courier New', monospace;
  font-size: 0.9rem;
}

.monitor-name {
  color: #4CAF50;
  font-weight: bold;
}

.monitor-details {
  color: #ccc;
}

.primary-badge {
  color: #ff6b35;
  font-weight: bold;
}

.instructions {
  margin: 1.5rem 0;
  color: #ccc;
  line-height: 1.5;
}

.instructions kbd {
  background: #333;
  border: 1px solid #666;
  border-radius: 4px;
  padding: 2px 6px;
  font-size: 0.9em;
  color: #fff;
}

.button-group {
  display: flex;
  gap: 1rem;
  margin: 1.5rem 0;
}

.btn-primary {
  background: #4CAF50;
  color: white;
  border: none;
  padding: 0.75rem 1.5rem;
  border-radius: 6px;
  cursor: pointer;
  font-weight: bold;
  transition: background 0.2s;
}

.btn-primary:hover {
  background: #45a049;
}

.btn-secondary {
  background: #666;
  color: white;
  border: none;
  padding: 0.75rem 1.5rem;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.2s;
}

.btn-secondary:hover {
  background: #777;
}

.status {
  margin-top: 1rem;
  padding: 0.75rem;
  background: #2a2a2a;
  border-radius: 6px;
  color: #4CAF50;
  text-align: center;
  font-weight: bold;
}

.calibration-target {
  position: absolute;
  width: 200px;
  height: 200px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  z-index: 10001;
}

.target-circle {
  position: relative;
  width: 60px;
  height: 60px;
  margin-bottom: 1rem;
}

.outer-ring {
  position: absolute;
  width: 60px;
  height: 60px;
  border: 3px solid white;
  border-radius: 50%;
  background: rgba(255, 0, 0, 0.7);
}

.middle-ring {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 30px;
  height: 30px;
  border: 2px solid white;
  border-radius: 50%;
  background: white;
}

.inner-dot {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 10px;
  height: 10px;
  border: 1px solid black;
  border-radius: 50%;
  background: black;
}

.target-instructions {
  text-align: center;
  color: white;
  background: rgba(0, 0, 0, 0.8);
  padding: 1rem;
  border-radius: 8px;
  border: 1px solid #333;
}

.target-instructions p {
  margin: 0.25rem 0;
}

.escape-hint {
  color: #aaa;
  font-size: 0.8rem;
}
</style> 