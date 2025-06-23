<template>
  <div class="wake-word-control">
    <div class="control-header">
      <h3 class="text-lg font-semibold text-gray-800 dark:text-gray-200">
        Wake Word Detection
      </h3>
      <div class="status-indicator">
        <div 
          :class="[
            'w-3 h-3 rounded-full transition-colors duration-300',
            isActive ? 'bg-green-500 animate-pulse' : 'bg-gray-400'
          ]"
        />
        <span class="text-sm text-gray-600 dark:text-gray-400">
          {{ isActive ? 'Active' : 'Inactive' }}
        </span>
      </div>
    </div>

    <!-- Main Controls -->
    <div class="controls-section">
      <button
        @click="toggleDetection"
        :disabled="isStarting || isStopping"
        :class="[
          'px-6 py-3 rounded-lg font-medium text-white transition-all duration-200',
          'focus:outline-none focus:ring-2 focus:ring-offset-2',
          isActive 
            ? 'bg-red-500 hover:bg-red-600 focus:ring-red-500' 
            : 'bg-blue-500 hover:bg-blue-600 focus:ring-blue-500',
          (isStarting || isStopping) && 'opacity-50 cursor-not-allowed'
        ]"
      >
        <span v-if="isStarting">Starting...</span>
        <span v-else-if="isStopping">Stopping...</span>
        <span v-else>{{ isActive ? 'Stop Detection' : 'Start Detection' }}</span>
      </button>

      <button
        v-if="totalDetections > 0"
        @click="resetStats"
        class="px-4 py-2 rounded-lg bg-gray-500 hover:bg-gray-600 text-white text-sm transition-colors duration-200"
      >
        Reset Stats
      </button>
    </div>

    <!-- Status Information -->
    <div class="status-section">
      <div class="status-grid">
        <div class="status-item">
          <div class="status-label">Status</div>
          <div class="status-value">
            {{ isListening ? 'Listening for "Aubrey"' : 'Not listening' }}
          </div>
        </div>

        <div class="status-item">
          <div class="status-label">Total Detections</div>
          <div class="status-value">{{ totalDetections }}</div>
        </div>

        <div v-if="lastDetection" class="status-item">
          <div class="status-label">Last Detection</div>
          <div class="status-value">
            <div class="text-sm">
              Confidence: {{ (lastDetection.confidence * 100).toFixed(1) }}%
            </div>
            <div class="text-xs text-gray-500">
              {{ formatTimestamp(lastDetection.timestamp) }}
            </div>
          </div>
        </div>

        <div v-if="whisperActivated" class="status-item">
          <div class="status-label">Whisper</div>
          <div class="status-value text-green-600">
            ✓ Activated
          </div>
        </div>
      </div>
    </div>

    <!-- Recent Detection Indicator -->
    <div v-if="hasRecentDetection" class="recent-detection">
      <div class="flex items-center space-x-2">
        <div class="w-4 h-4 bg-green-500 rounded-full animate-ping"></div>
        <span class="text-green-600 font-medium">
          Wake word detected recently!
        </span>
      </div>
    </div>

    <!-- Error Display -->
    <div v-if="error" class="error-section">
      <div class="bg-red-50 border border-red-200 rounded-lg p-3">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-2">
            <div class="w-4 h-4 bg-red-500 rounded-full"></div>
            <span class="text-red-800 text-sm">{{ error }}</span>
          </div>
          <button
            @click="clearError"
            class="text-red-600 hover:text-red-800 text-sm"
          >
            ✕
          </button>
        </div>
      </div>
    </div>

    <!-- Help Text -->
    <div class="help-section">
      <div class="text-xs text-gray-500 dark:text-gray-400">
        <p>Say "Aubrey" or "Hi Aubrey" to trigger the wake word detection.</p>
        <p>When detected, Whisper will activate for full speech transcription.</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { useWakeWordDetection } from '../../composables/useWakeWordDetection'

const {
  isActive,
  isListening,
  lastDetection,
  totalDetections,
  whisperActivated,
  error,
  isStarting,
  isStopping,
  hasRecentDetection,
  toggleDetection,
  resetStats,
  clearError
} = useWakeWordDetection()

function formatTimestamp(timestamp: number): string {
  const date = new Date(timestamp)
  return date.toLocaleTimeString()
}

// Listen for wake word detection events
function handleWakeWordDetection(event: CustomEvent) {
  console.log('Wake word detection event received:', event.detail)
  
  // You can add custom handling here, like:
  // - Playing a sound
  // - Showing a notification
  // - Triggering other UI changes
}

onMounted(() => {
  window.addEventListener('wakeWordDetected', handleWakeWordDetection as EventListener)
})

onUnmounted(() => {
  window.removeEventListener('wakeWordDetected', handleWakeWordDetection as EventListener)
})
</script>

<style scoped>
.wake-word-control {
  @apply bg-white dark:bg-gray-800 rounded-lg p-6 shadow-lg border border-gray-200 dark:border-gray-700 space-y-4;
}

.control-header {
  @apply flex items-center justify-between;
}

.status-indicator {
  @apply flex items-center space-x-2;
}

.controls-section {
  @apply flex items-center space-x-3;
}

.status-section {
  @apply bg-gray-50 dark:bg-gray-900 rounded-lg p-4;
}

.status-grid {
  @apply grid grid-cols-1 md:grid-cols-2 gap-4;
}

.status-item {
  @apply space-y-1;
}

.status-label {
  @apply text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wide;
}

.status-value {
  @apply text-sm text-gray-900 dark:text-gray-100;
}

.recent-detection {
  @apply bg-green-50 border border-green-200 rounded-lg p-3;
}

.error-section {
  @apply space-y-2;
}

.help-section {
  @apply bg-blue-50 dark:bg-blue-900/20 rounded-lg p-3 space-y-2;
}
</style> 