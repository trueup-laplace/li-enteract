<script setup lang="ts">
import { 
  MinusIcon, 
  Squares2X2Icon,
  XMarkIcon,
  ChevronDownIcon,
  ChevronUpIcon
} from '@heroicons/vue/24/outline'
import { useWindowManager } from '../../composables/useWindowManager'
import { useAppStore } from '../../stores/app'

const { toggleCollapse, minimizeWindow, closeWindow, startDrag } = useWindowManager()
const store = useAppStore()

// Debug handlers with more detailed logging
const handleMinimize = (event: Event) => {
  event.stopPropagation()
  event.preventDefault()
  console.log('Minimize button clicked!')
  minimizeWindow()
}

const handleClose = (event: Event) => {
  event.stopPropagation()
  event.preventDefault()
  console.log('Close button clicked!')
  closeWindow()
}

const handleToggleCollapse = (event: Event) => {
  event.stopPropagation()
  event.preventDefault()
  console.log('Collapse button clicked!')
  toggleCollapse()
}

const handleToggleViewCollapse = (event: Event) => {
  event.stopPropagation()
  event.preventDefault()
  console.log('View collapse button clicked!')
  store.toggleViewCollapse()
}

// Mouse event debugging
const onMouseOver = () => {
  console.log('Mouse over WindowHeader detected!')
}
</script>

<template>
  <div class="window-header" @mouseover="onMouseOver">
    <!-- Left: App Title (draggable) -->
    <div class="header-section draggable-area">
      <div class="w-2 h-2 rounded-full bg-gradient-to-r from-blue-400 to-purple-500"></div>
      <span class="text-xs text-white/60 select-none font-medium">Agentic Assistant</span>
    </div>

    <!-- Center: Status (draggable) -->
    <div class="header-section draggable-area">
      <div class="w-1 h-1 rounded-full bg-green-400"></div>
      <span class="text-[10px] text-white/30">Active</span>
    </div>

    <!-- Right: Window Controls (NOT draggable) -->
    <div class="controls-container">
      <!-- View Collapse/Expand -->
      <button 
        @click="handleToggleViewCollapse"
        @mousedown.stop.prevent
        class="window-control-btn"
        style="pointer-events: auto; position: relative; z-index: 1000;"
        :title="store.viewCollapsed ? 'Expand View' : 'Collapse View'"
      >
        <ChevronDownIcon v-if="!store.viewCollapsed" class="w-2.5 h-2.5 text-white/70" />
        <ChevronUpIcon v-else class="w-2.5 h-2.5 text-white/70" />
      </button>
      
      <!-- Collapse/Expand -->
      <button 
        @click="handleToggleCollapse"
        @mousedown.stop.prevent
        class="window-control-btn"
        style="pointer-events: auto; position: relative; z-index: 1000;"
      >
        <Squares2X2Icon class="w-2.5 h-2.5 text-white/70" />
      </button>
      
      <!-- Minimize -->
      <button 
        @click="handleMinimize"
        @mousedown.stop.prevent
        class="window-control-btn"
        style="pointer-events: auto; position: relative; z-index: 1000;"
      >
        <MinusIcon class="w-2.5 h-2.5 text-white/70" />
      </button>
      
      <!-- Close -->
      <button 
        @click="handleClose"
        @mousedown.stop.prevent
        class="window-control-btn bg-red-500/10 hover:bg-red-500/30"
        style="pointer-events: auto; position: relative; z-index: 1000;"
      >
        <XMarkIcon class="w-2.5 h-2.5 text-red-400/70" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.window-header {
  @apply flex items-center justify-between px-3 py-1.5 h-8 bg-gradient-to-r from-black/10 via-black/5 to-transparent backdrop-blur-md border-b border-white/5;
  position: relative;
  cursor: grab;
  z-index: 10000;
  pointer-events: auto;
}

.window-header:active {
  cursor: grabbing;
}

.header-section {
  @apply flex items-center gap-2;
}

.draggable-area {
  -webkit-app-region: drag;
  cursor: move;
}

.controls-container {
  @apply flex items-center gap-1;
  -webkit-app-region: no-drag;
  pointer-events: auto;
  position: relative;
  z-index: 100;
}

/* Test button styles removed */

.window-control-btn {
  @apply w-5 h-5 rounded-full bg-white/5 hover:bg-white/15 flex items-center justify-center transition-all duration-200 hover:scale-110;
  @apply border border-white/20 hover:border-white/40;
  -webkit-app-region: no-drag;
  pointer-events: auto;
  cursor: pointer;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.1);
}

.window-control-btn:hover {
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4), inset 0 1px 0 rgba(255, 255, 255, 0.2);
}

.window-control-btn:active {
  transform: scale(0.95);
}

/* Enhanced backdrop blur */
@supports (backdrop-filter: blur(16px)) {
  .backdrop-blur-md {
    backdrop-filter: blur(16px);
  }
}
</style> 