<script setup lang="ts">
import { onMounted } from 'vue'
import { useWindowManager } from './composables/useWindowManager'
import { useAppStore } from './stores/app'

// Components
import WindowHeader from './components/core/WindowHeader.vue'
import ThreeScene from './components/three/ThreeScene.vue'
import ControlPanel from './components/core/ControlPanel.vue'
import ChatDrawer from './components/core/ChatDrawer.vue'
import MinimizedView from './components/core/MinimizedView.vue'

// Composables
const { initializeWindow } = useWindowManager()
const store = useAppStore()

onMounted(() => {
  initializeWindow()
})
</script>

<template>
  <div class="app-container" data-theme="dark">
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
      <div class="window-wrapper">
        <!-- Window Header -->
        <WindowHeader />
        
        <!-- Three.js Background -->
        <ThreeScene />
        
        <!-- Content overlay with subtle gradient -->
        <div class="content-overlay">
          <!-- Main Content Area -->
          <div class="main-content">
            <!-- Breathing room -->
            <div class="flex-1"></div>
            
            <!-- Control Panel -->
            <ControlPanel />
          </div>
        </div>
        
        <!-- Chat Drawer -->
        <ChatDrawer />
      </div>
    </template>
  </div>
</template>

<style>
.app-container {
  @apply min-h-screen overflow-hidden;
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
}

.content-overlay {
  @apply absolute inset-0 pt-8;
  background: linear-gradient(
    180deg,
    rgba(0, 0, 0, 0.1) 0%,
    rgba(0, 0, 0, 0.05) 50%,
    rgba(0, 0, 0, 0.1) 100%
  );
}

.main-content {
  @apply relative z-20 min-h-full flex flex-col;
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
</style>


