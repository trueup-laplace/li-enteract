<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { useWindowManager } from './composables/useWindowManager'
import { useAppStore } from './stores/app'
import { useTransparency } from './composables/useTransparency'

// Components
import WindowHeader from './components/core/WindowHeader.vue'
import ControlPanel from './components/core/ControlPanel.vue'
import HomeScreen from './components/core/HomeScreen.vue'
import MinimizedView from './components/core/MinimizedView.vue'
import RefractionBorder from './components/core/RefractionBorder.vue'

// Composables
const { initializeWindow } = useWindowManager()
const store = useAppStore()
const transparency = useTransparency()

// Dynamic classes based on transparency state
const appClasses = computed(() => {
  const classes = ['app-container']
  
  if (transparency.isTransparent.value) {
    classes.push('transparent-mode')
  }
  
  if (transparency.isClickThrough.value) {
    classes.push('click-through-mode')
  }
  
  return classes.join(' ')
})

// Window wrapper styles based on transparency
const windowWrapperStyle = computed(() => {
  if (!transparency.isTransparent.value) return {}
  
  const level = transparency.transparencyLevel.value
  
  return {
    '--transparency-level': level,
    '--visibility-level': Math.max(level, 0.1), // Ensure some visibility for borders
    '--border-opacity': Math.min((1 - level) * 2, 1) // More border when more transparent
  }
})

onMounted(() => {
  initializeWindow()
})
</script>

<template>
  <div :class="appClasses" data-theme="dark">
    <!-- Minimized State -->
    <template v-if="store.windowCollapsed">
      <div class="fixed inset-0 bg-transparent">
        <div class="absolute top-2 left-2 right-2">
          <MinimizedView />
        </div>
      </div>
    </template>
    
    <!-- Normal State -->
    <template v-else>
      <!-- Window with fade edges -->
      <div class="window-wrapper" :style="windowWrapperStyle">
        <!-- Window Header - Always on top -->
        <div class="header-container">
          <WindowHeader />
        </div>
        
        <!-- Content overlay with subtle gradient -->
        <div class="content-overlay">
          <!-- Main Content Area -->
          <div class="main-content">
            <!-- Home Screen with Chat Interface -->
            <div class="flex-1">
              <HomeScreen />
            </div>
            
            <!-- Control Panel -->
            <ControlPanel />
          </div>
        </div>
      </div>
      
      <!-- Refraction Border Effects -->
      <RefractionBorder />
    </template>
  </div>
</template>

<style>
.app-container {
  @apply min-h-screen overflow-hidden;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

/* Enhanced styles when transparent */
.app-container.transparent-mode {
  position: relative;
}

.app-container.click-through-mode {
  /* Visual indicator that click-through is enabled */
  filter: hue-rotate(30deg) saturate(1.2);
}

.window-wrapper {
  @apply relative min-h-screen;
  background: linear-gradient(135deg, 
    rgba(17, 24, 39, 0.95) 0%,
    rgba(31, 41, 55, 0.90) 25%,
    rgba(55, 65, 81, 0.85) 50%,
    rgba(75, 85, 99, 0.80) 75%,
    rgba(107, 114, 128, 0.70) 100%
  );
  
  /* Fade edges effect */
  mask: 
    radial-gradient(ellipse 80% 100% at center, black 60%, transparent 100%),
    linear-gradient(to bottom, transparent 0%, black 5%, black 95%, transparent 100%),
    linear-gradient(to right, transparent 0%, black 5%, black 95%, transparent 100%);
  mask-composite: intersect;
  -webkit-mask: 
    radial-gradient(ellipse 80% 100% at center, black 60%, transparent 100%),
    linear-gradient(to bottom, transparent 0%, black 5%, black 95%, transparent 100%),
    linear-gradient(to right, transparent 0%, black 5%, black 95%, transparent 100%);
  -webkit-mask-composite: source-in;
  
  transition: all 0.3s ease;
}

/* Transparency mode adaptations */
.transparent-mode .window-wrapper {
  /* Enhance edges when transparent */
  background: linear-gradient(135deg, 
    rgba(17, 24, 39, calc(0.95 * var(--visibility-level, 1))) 0%,
    rgba(31, 41, 55, calc(0.90 * var(--visibility-level, 1))) 25%,
    rgba(55, 65, 81, calc(0.85 * var(--visibility-level, 1))) 50%,
    rgba(75, 85, 99, calc(0.80 * var(--visibility-level, 1))) 75%,
    rgba(107, 114, 128, calc(0.70 * var(--visibility-level, 1))) 100%
  );
  
  /* Add subtle inner border */
  box-shadow: 
    inset 0 0 0 1px rgba(255, 255, 255, calc(var(--border-opacity, 0) * 0.2)),
    inset 0 0 20px rgba(255, 255, 255, calc(var(--border-opacity, 0) * 0.05));
}

.click-through-mode .window-wrapper {
  /* Warning visual for click-through */
  box-shadow: 
    inset 0 0 0 2px rgba(255, 165, 0, 0.4),
    inset 0 0 30px rgba(255, 165, 0, 0.1);
}

.header-container {
  @apply relative;
  z-index: 9999;
  position: relative;
  pointer-events: auto;
}

.content-overlay {
  @apply absolute inset-0;
  top: 32px; /* Start below the 8 * 4px = 32px header */
  background: linear-gradient(
    180deg,
    rgba(0, 0, 0, 0.1) 0%,
    rgba(0, 0, 0, 0.05) 50%,
    rgba(0, 0, 0, 0.1) 100%
  );
  z-index: 10;
  pointer-events: none;
}

/* Adapt content overlay for transparency */
.transparent-mode .content-overlay {
  background: linear-gradient(
    180deg,
    rgba(0, 0, 0, calc(0.1 * var(--visibility-level, 1))) 0%,
    rgba(0, 0, 0, calc(0.05 * var(--visibility-level, 1))) 50%,
    rgba(0, 0, 0, calc(0.1 * var(--visibility-level, 1))) 100%
  );
}

.main-content {
  @apply relative min-h-full flex flex-col;
  z-index: 20;
  pointer-events: auto;
}

/* Enhanced glass effects */
.glass-enhanced {
  backdrop-filter: blur(40px) saturate(180%);
  background: linear-gradient(
    135deg,
    rgba(255, 255, 255, 0.1) 0%,
    rgba(255, 255, 255, 0.05) 50%,
    rgba(255, 255, 255, 0.1) 100%
  );
  border: 1px solid rgba(255, 255, 255, 0.2);
  box-shadow: 
    0 8px 32px rgba(0, 0, 0, 0.3),
    inset 0 1px 0 rgba(255, 255, 255, 0.2);
}

/* Enhanced glass for transparency mode */
.transparent-mode .glass-enhanced {
  backdrop-filter: blur(60px) saturate(200%) brightness(1.1);
  border: 1px solid rgba(255, 255, 255, calc(0.3 * var(--border-opacity, 1)));
}

/* Smooth animations */
* {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

/* Custom scrollbars */
::-webkit-scrollbar {
  width: 6px;
}

::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 3px;
}

::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 3px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.3);
}

/* Special cursor for click-through mode */
.click-through-mode {
  cursor: none;
}

/* Ensure certain elements remain interactive in transparency mode */
.transparent-mode .header-container,
.transparent-mode .main-content {
  position: relative;
  z-index: 100;
}

/* Add subtle animation when entering/leaving transparency */
.app-container:not(.transparent-mode) .window-wrapper {
  animation: solidify 0.3s ease-out;
}

.app-container.transparent-mode .window-wrapper {
  animation: transparentify 0.3s ease-out;
}

@keyframes solidify {
  from {
    filter: blur(1px) brightness(1.1);
  }
  to {
    filter: none;
  }
}

@keyframes transparentify {
  from {
    filter: none;
  }
  to {
    filter: blur(0.5px) brightness(1.05);
  }
}
</style>


