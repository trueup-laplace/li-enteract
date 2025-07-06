<template>
  <div class="advanced-gaze-demo">
    <h2>🎯 Advanced Gaze Tracking Demo</h2>
    <p>Multi-monitor support with MediaPipe integration</p>
    
    <div class="controls">
      <button @click="startTracking" :disabled="isActive" class="btn btn-primary">
        Start Advanced Tracking
      </button>
      <button @click="stopTracking" :disabled="!isActive" class="btn btn-secondary">
        Stop Tracking
      </button>
    </div>

    <div v-if="error" class="error">
      Error: {{ error }}
    </div>

    <div v-if="isActive" class="status">
      <p>✅ Advanced gaze tracking is active</p>
      <p v-if="currentGaze">
        Gaze Position: ({{ Math.round(currentGaze.x) }}, {{ Math.round(currentGaze.y) }})
        - Confidence: {{ Math.round(currentGaze.confidence * 100) }}%
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useAdvancedGazeTracking } from '../../composables/useAdvancedGazeTracking'

const gazeTracking = useAdvancedGazeTracking()

const isActive = gazeTracking.isActive
const currentGaze = gazeTracking.currentGaze
const error = gazeTracking.error

const startTracking = async () => {
  await gazeTracking.startTracking()
}

const stopTracking = async () => {
  await gazeTracking.stopTracking()
}
</script>

<style scoped>
.advanced-gaze-demo {
  padding: 20px;
}

.controls {
  margin: 20px 0;
}

.btn {
  padding: 10px 20px;
  margin: 0 10px;
  border: none;
  border-radius: 5px;
  cursor: pointer;
}

.btn-primary {
  background: #007bff;
  color: white;
}

.btn-secondary {
  background: #6c757d;
  color: white;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.error {
  color: red;
  margin: 10px 0;
}

.status {
  margin: 20px 0;
  padding: 15px;
  background: #f8f9fa;
  border-radius: 5px;
}
</style> 