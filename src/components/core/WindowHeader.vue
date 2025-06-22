<script setup lang="ts">
import { 
  MinusIcon, 
  Squares2X2Icon,
  XMarkIcon
} from '@heroicons/vue/24/outline'
import { useWindowManager } from '../../composables/useWindowManager'
import { useAppStore } from '../../stores/app'

const { startDrag, toggleCollapse, minimizeWindow, closeWindow } = useWindowManager()
const store = useAppStore()
</script>

<template>
  <div 
    class="flex items-center justify-between p-3 h-14 bg-gradient-to-r from-black/20 via-black/10 to-transparent backdrop-blur-xl border-b border-white/5"
    data-tauri-drag-region
  >
    <!-- Left: App Title -->
    <div class="flex items-center gap-3 px-2">
      <div class="w-3 h-3 rounded-full bg-gradient-to-r from-blue-400 to-purple-500 animate-pulse"></div>
      <span class="text-sm text-white/80 select-none font-medium tracking-wide">Agentic Assistant</span>
    </div>

    <!-- Center: Status -->
    <div class="flex items-center gap-2 text-xs text-white/40">
      <div class="w-1.5 h-1.5 rounded-full bg-green-400"></div>
      <span>Active</span>
    </div>

    <!-- Right: Window Controls -->
    <div class="flex items-center gap-2">
      <!-- Collapse/Expand -->
      <button 
        @click="toggleCollapse"
        class="w-7 h-7 rounded-full bg-white/5 hover:bg-white/15 flex items-center justify-center transition-all duration-200 hover:scale-110"
      >
        <Squares2X2Icon class="w-3.5 h-3.5 text-white/60" />
      </button>
      
      <!-- Minimize -->
      <button 
        @click="minimizeWindow"
        class="w-7 h-7 rounded-full bg-white/5 hover:bg-white/15 flex items-center justify-center transition-all duration-200 hover:scale-110"
      >
        <MinusIcon class="w-3.5 h-3.5 text-white/60" />
      </button>
      
      <!-- Close -->
      <button 
        @click="closeWindow"
        class="w-7 h-7 rounded-full bg-red-500/10 hover:bg-red-500/30 flex items-center justify-center transition-all duration-200 hover:scale-110"
      >
        <XMarkIcon class="w-3.5 h-3.5 text-red-400/80" />
      </button>
    </div>
  </div>
</template>

<style scoped>
/* Enhanced backdrop blur */
@supports (backdrop-filter: blur(24px)) {
  .backdrop-blur-xl {
    backdrop-filter: blur(24px);
  }
}
</style> 