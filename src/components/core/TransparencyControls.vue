<script setup lang="ts">
import { computed } from 'vue'
import { 
  EyeIcon, 
  EyeSlashIcon,
  ArrowUturnLeftIcon,
  ExclamationTriangleIcon,
  SparklesIcon
} from '@heroicons/vue/24/outline'
import { useTransparency } from '../../composables/useTransparency'

const transparency = useTransparency()

// Visual feedback for current state
const statusColor = computed(() => {
  if (transparency.lastError.value) return 'text-red-400'
  if (!transparency.isVisible.value) return 'text-orange-400'
  if (transparency.isClickThrough.value) return 'text-yellow-400'
  if (transparency.isTransparent.value) return 'text-blue-400'
  return 'text-green-400'
})

const statusIcon = computed(() => {
  if (!transparency.isVisible.value) return EyeSlashIcon
  return EyeIcon
})

// Refraction effect indicator
const refractionIntensity = computed(() => {
  if (!transparency.isTransparent.value) return 0
  return Math.min((1 - transparency.transparencyLevel.value) * 1.5, 1)
})

// Handle slider input
const handleSliderChange = (event: Event) => {
  const target = event.target as HTMLInputElement
  const level = parseFloat(target.value) / 100
  transparency.setLevel(level)
}
</script>

<template>
  <div class="transparency-controls">
    <!-- Header with status -->
    <div class="controls-header">
      <div class="flex items-center gap-2">
        <component 
          :is="statusIcon" 
          class="w-4 h-4"
          :class="statusColor"
        />
        <span class="text-sm font-medium text-white/90">
          Transparency
        </span>
        <span class="text-xs" :class="statusColor">
          {{ transparency.getVisibilityStatus() }}
        </span>
        
        <!-- Refraction Effect Indicator -->
        <div v-if="transparency.isTransparent.value" class="refraction-indicator">
          <SparklesIcon class="w-3 h-3 text-cyan-400" />
          <span class="text-xs text-cyan-400">
            {{ Math.round(refractionIntensity * 100) }}%
          </span>
        </div>
      </div>
      
      <!-- Emergency restore button - always visible -->
      <button
        @click="transparency.emergencyRestore"
        class="emergency-btn"
        title="Emergency Restore (Esc)"
      >
        <ArrowUturnLeftIcon class="w-3 h-3" />
      </button>
    </div>

    <!-- Error display -->
    <div v-if="transparency.lastError.value" class="error-message">
      <ExclamationTriangleIcon class="w-4 h-4" />
      <span class="text-xs">{{ transparency.lastError.value }}</span>
    </div>

    <!-- Refraction Status -->
    <div v-if="transparency.isTransparent.value" class="refraction-status">
      <div class="flex items-center justify-between">
        <span class="text-xs text-white/70">Border Refraction:</span>
        <div class="refraction-preview" :style="{ '--refraction-intensity': refractionIntensity }">
          <div class="refraction-sample"></div>
        </div>
      </div>
    </div>

    <!-- Main toggle button -->
    <div class="main-controls">
      <button
        @click="transparency.toggle"
        class="toggle-btn"
        :class="{ 'active': transparency.isTransparent.value }"
        :disabled="transparency.isLoading.value"
      >
        <component 
          :is="transparency.isTransparent.value ? EyeSlashIcon : EyeIcon" 
          class="w-4 h-4" 
        />
        <span>
          {{ transparency.isTransparent.value ? 'Make Solid' : 'See Through' }}
        </span>
        <div v-if="transparency.isLoading.value" class="loading-spinner"></div>
      </button>
    </div>

    <!-- Transparency slider -->
    <div class="slider-control">
      <label class="slider-label">
        <span class="text-xs text-white/70">Opacity:</span>
        <span class="text-xs font-mono" :class="statusColor">
          {{ transparency.getTransparencyPercentage() }}%
        </span>
      </label>
      
      <div class="slider-container">
        <input
          type="range"
          min="0"
          max="100"
          :value="transparency.getTransparencyPercentage()"
          @input="handleSliderChange"
          class="transparency-slider"
          :disabled="transparency.isLoading.value"
        />
      </div>
      
      <!-- Visual indicator bar -->
      <div class="opacity-indicator">
        <div 
          class="opacity-bar" 
          :style="{ width: transparency.getTransparencyPercentage() + '%' }"
        ></div>
      </div>
    </div>

    <!-- Preset buttons -->
    <div class="preset-controls">
      <button
        @click="transparency.presets.solid"
        class="preset-btn"
        title="Solid (100% visible)"
      >
        Solid
      </button>
      
      <button
        @click="transparency.presets.semiTransparent"
        class="preset-btn"
        title="Semi-transparent (70% visible)"
      >
        Semi
      </button>
      
      <button
        @click="transparency.presets.ghostMode"
        class="preset-btn"
        title="Ghost Mode (30% visible)"
      >
        Ghost
      </button>
      
      <button
        @click="transparency.presets.invisible"
        class="preset-btn danger"
        title="Invisible (click-through enabled)"
      >
        Hide
      </button>
    </div>

    <!-- Click-through warning -->
    <div v-if="transparency.isClickThrough.value" class="click-through-warning">
      <ExclamationTriangleIcon class="w-3 h-3" />
      <span class="text-xs">Click-through enabled - Border effects active</span>
    </div>

    <!-- Keyboard shortcuts info -->
    <div class="shortcuts-info">
      <div class="text-xs text-white/50">
        <div>Ctrl+T: Toggle</div>
        <div>Ctrl+H: Ghost Mode</div>
        <div>Esc: Restore</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.transparency-controls {
  @apply p-4 space-y-3;
  @apply backdrop-blur-xl border border-white/15 rounded-2xl;
  background: linear-gradient(to bottom, 
    rgba(0, 0, 0, 0.8) 0%, 
    rgba(0, 0, 0, 0.9) 100%
  );
  min-width: 280px;
  max-width: 320px;
}

.controls-header {
  @apply flex items-center justify-between;
}

.refraction-indicator {
  @apply flex items-center gap-1 px-2 py-1 rounded-lg;
  @apply bg-cyan-500/10 border border-cyan-500/20;
  animation: refractionPulse 2s ease-in-out infinite;
}

@keyframes refractionPulse {
  0%, 100% { opacity: 0.8; transform: scale(1); }
  50% { opacity: 1; transform: scale(1.02); }
}

.refraction-status {
  @apply p-2 rounded-lg bg-white/5 border border-white/10;
}

.refraction-preview {
  @apply w-12 h-4 rounded overflow-hidden border border-white/20;
  background: linear-gradient(45deg, 
    rgba(255, 255, 255, calc(var(--refraction-intensity) * 0.3)),
    rgba(0, 255, 255, calc(var(--refraction-intensity) * 0.2)),
    rgba(255, 0, 255, calc(var(--refraction-intensity) * 0.15))
  );
}

.refraction-sample {
  @apply w-full h-full;
  background: linear-gradient(
    90deg,
    transparent 30%, 
    rgba(255, 255, 255, calc(var(--refraction-intensity) * 0.6)) 50%, 
    transparent 70%
  );
  background-size: 200% 100%;
  animation: miniShimmer 1.5s ease-in-out infinite;
}

@keyframes miniShimmer {
  0% { background-position: -200% 0; }
  100% { background-position: 200% 0; }
}

.emergency-btn {
  @apply p-1 rounded-lg bg-red-500/20 border border-red-500/30;
  @apply hover:bg-red-500/30 hover:border-red-500/50;
  @apply transition-all duration-200;
  @apply text-red-400 hover:text-red-300;
}

.error-message {
  @apply flex items-center gap-2 p-2 rounded-lg;
  @apply bg-red-500/10 border border-red-500/20;
  @apply text-red-400 text-xs;
}

.main-controls {
  @apply flex;
}

.toggle-btn {
  @apply flex-1 flex items-center justify-center gap-2 p-3 rounded-lg;
  @apply bg-white/5 border border-white/15;
  @apply hover:bg-white/10 hover:border-white/30;
  @apply transition-all duration-200;
  @apply text-white/90 hover:text-white;
  @apply font-medium text-sm;
  position: relative;
}

.toggle-btn.active {
  @apply bg-blue-500/20 border-blue-500/30 text-blue-300;
}

.toggle-btn:disabled {
  @apply opacity-50 cursor-not-allowed;
}

.loading-spinner {
  @apply absolute right-2 w-4 h-4 border-2 border-white/20 border-t-white/60;
  @apply rounded-full animate-spin;
}

.slider-control {
  @apply space-y-2;
}

.slider-label {
  @apply flex items-center justify-between;
}

.slider-container {
  @apply relative;
}

.transparency-slider {
  @apply w-full h-2 rounded-lg appearance-none cursor-pointer;
  @apply bg-white/10;
  background-image: linear-gradient(
    to right,
    rgba(255, 0, 0, 0.3) 0%,
    rgba(255, 255, 0, 0.3) 30%,
    rgba(0, 255, 0, 0.3) 100%
  );
}

.transparency-slider::-webkit-slider-thumb {
  @apply appearance-none w-4 h-4 rounded-full cursor-pointer;
  @apply bg-white border-2 border-white/50;
  @apply shadow-lg;
}

.transparency-slider::-moz-range-thumb {
  @apply w-4 h-4 rounded-full cursor-pointer;
  @apply bg-white border-2 border-white/50;
  @apply shadow-lg;
}

.opacity-indicator {
  @apply h-1 bg-white/10 rounded-full overflow-hidden;
}

.opacity-bar {
  @apply h-full bg-gradient-to-r from-red-400 via-yellow-400 to-green-400;
  @apply transition-all duration-300;
}

.preset-controls {
  @apply grid grid-cols-4 gap-2;
}

.preset-btn {
  @apply px-2 py-1.5 text-xs rounded-lg;
  @apply bg-white/5 border border-white/15;
  @apply hover:bg-white/10 hover:border-white/30;
  @apply transition-all duration-200;
  @apply text-white/80 hover:text-white;
}

.preset-btn.danger {
  @apply bg-red-500/10 border-red-500/20;
  @apply hover:bg-red-500/20 hover:border-red-500/40;
  @apply text-red-400 hover:text-red-300;
}

.click-through-warning {
  @apply flex items-center gap-2 p-2 rounded-lg;
  @apply bg-orange-500/10 border border-orange-500/20;
  @apply text-orange-400 text-xs;
  @apply animate-pulse;
}

.shortcuts-info {
  @apply pt-2 border-t border-white/10;
}

/* Ensure controls remain visible and interactive */
.transparency-controls {
  pointer-events: auto !important;
  z-index: 9999;
}

/* Smooth transitions for all interactive elements */
.transparency-controls button,
.transparency-controls input {
  @apply transition-all duration-200 ease-out;
}

/* Hover effects */
.transparency-controls button:hover {
  transform: translateY(-1px);
  @apply shadow-lg;
}

/* Focus styles for accessibility */
.transparency-controls button:focus,
.transparency-controls input:focus {
  @apply outline-none ring-2 ring-blue-500/50 ring-offset-2 ring-offset-black/50;
}
</style> 