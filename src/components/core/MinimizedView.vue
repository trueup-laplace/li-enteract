<script setup lang="ts">
import { 
  MicrophoneIcon,
  Squares2X2Icon,
  EyeIcon,
  EyeSlashIcon
} from '@heroicons/vue/24/outline'
import { useAppStore } from '../../stores/app'

const store = useAppStore()
</script>

<template>
  <div class="h-20 flex items-center justify-between px-4 py-2 bg-gradient-to-r from-black/40 via-black/20 to-transparent backdrop-blur-xl border border-white/10 rounded-2xl shadow-2xl">
    <!-- Left: Recording Status -->
    <div class="flex items-center gap-3">
      <!-- Record Button -->
      <button 
        @click="store.toggleRecording"
        class="relative w-8 h-8 rounded-full transition-all duration-300 hover:scale-110"
        :class="store.isRecording ? 'bg-red-500 animate-pulse' : 'bg-white/20 hover:bg-white/30'"
      >
        <div 
          class="absolute inset-0 rounded-full"
          :class="store.isRecording ? 'bg-red-600 scale-75' : 'bg-white/40 scale-50'"
        ></div>
      </button>
      
      <!-- Timer -->
      <div class="text-white/90 font-mono text-sm tracking-wider">
        {{ store.formatRecordingTime() }}
      </div>
    </div>

    <!-- Center: Status Indicator -->
    <div class="flex items-center gap-2">
      <div class="w-2 h-2 rounded-full bg-green-400 animate-pulse"></div>
      <span class="text-white/70 text-xs font-medium">Ready</span>
    </div>

    <!-- Right: Quick Actions -->
    <div class="flex items-center gap-2">
      <!-- Ask AI -->
      <button class="px-3 py-1 text-xs text-white/80 bg-white/10 hover:bg-white/20 rounded-lg border border-white/10 transition-all">
        Ask AI
      </button>
      
      <!-- Chat Toggle removed - chat is now part of home screen -->
      
      <!-- Expand -->
      <button 
        @click="store.toggleWindowCollapse"
        class="w-6 h-6 flex items-center justify-center bg-white/10 hover:bg-white/20 rounded-lg transition-all"
      >
        <Squares2X2Icon class="w-4 h-4 text-white/60" />
      </button>
    </div>
  </div>
</template>

<style scoped>
/* Custom blur and fade effects */
@supports (backdrop-filter: blur(20px)) {
  .backdrop-blur-xl {
    backdrop-filter: blur(20px);
  }
}
</style> 