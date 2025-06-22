<script setup lang="ts">
import { 
  MinusIcon, 
  Squares2X2Icon,
  XMarkIcon
} from '@heroicons/vue/24/outline'
import { useWindowManager } from '../../composables/useWindowManager'
import { useAppStore } from '../../stores/app'

const { toggleCollapse, minimizeWindow, closeWindow, startDrag } = useWindowManager()
const store = useAppStore()

// Fallback drag handler
const handleDrag = (event: MouseEvent) => {
  event.preventDefault()
  startDrag()
}
</script>

<template>
  <div 
    class="window-header"
    data-tauri-drag-region
    @mousedown="handleDrag"
  >
    <!-- Left: App Title -->
    <div class="header-section cursor-move" data-tauri-drag-region>
      <div class="w-2 h-2 rounded-full bg-gradient-to-r from-blue-400 to-purple-500"></div>
      <span class="text-xs text-white/60 select-none font-medium">Agentic Assistant</span>
    </div>

    <!-- Center: Status -->
    <div class="header-section cursor-move" data-tauri-drag-region>
      <div class="w-1 h-1 rounded-full bg-green-400"></div>
      <span class="text-[10px] text-white/30">Active</span>
    </div>

    <!-- Right: Window Controls -->
    <div class="flex items-center gap-1">
      <!-- Collapse/Expand -->
      <button 
        @click="toggleCollapse"
        @mousedown.stop
        class="window-control-btn"
      >
        <Squares2X2Icon class="w-2.5 h-2.5 text-white/50" />
      </button>
      
      <!-- Minimize -->
      <button 
        @click="minimizeWindow"
        @mousedown.stop
        class="window-control-btn"
      >
        <MinusIcon class="w-2.5 h-2.5 text-white/50" />
      </button>
      
      <!-- Close -->
      <button 
        @click="closeWindow"
        @mousedown.stop
        class="window-control-btn bg-red-500/10 hover:bg-red-500/30"
      >
        <XMarkIcon class="w-2.5 h-2.5 text-red-400/70" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.window-header {
  @apply flex items-center justify-between px-3 py-1.5 h-8 bg-gradient-to-r from-black/10 via-black/5 to-transparent backdrop-blur-md border-b border-white/5;
  -webkit-app-region: drag;
  cursor: grab;
}

.window-header:active {
  cursor: grabbing;
}

.header-section {
  @apply flex items-center gap-2;
  -webkit-app-region: drag;
}

.window-control-btn {
  @apply w-5 h-5 rounded-full bg-white/5 hover:bg-white/15 flex items-center justify-center transition-all duration-200 hover:scale-110;
  -webkit-app-region: no-drag;
}

/* Ensure buttons don't inherit drag behavior */
button {
  -webkit-app-region: no-drag;
}

/* Enhanced backdrop blur */
@supports (backdrop-filter: blur(16px)) {
  .backdrop-blur-md {
    backdrop-filter: blur(16px);
  }
}
</style> 