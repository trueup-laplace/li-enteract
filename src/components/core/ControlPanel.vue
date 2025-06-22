<script setup lang="ts">
import { ref } from 'vue'
import { 
  MicrophoneIcon, 
  ChatBubbleLeftRightIcon,
  SparklesIcon,
  CommandLineIcon
} from '@heroicons/vue/24/outline'
import { useAppStore } from '../../stores/app'
import TransparencyControls from './TransparencyControls.vue'

const store = useAppStore()

// Transparency controls state
const showTransparencyControls = ref(false)

const toggleTransparencyControls = () => {
  showTransparencyControls.value = !showTransparencyControls.value
}

// Click outside to close transparency controls
const closeTransparencyControls = (event: Event) => {
  const target = event.target as HTMLElement
  if (!target.closest('.transparency-controls') && !target.closest('.command-btn')) {
    showTransparencyControls.value = false
  }
}

// Setup click outside listener when controls are shown
import { onMounted, onUnmounted } from 'vue'

onMounted(() => {
  document.addEventListener('click', closeTransparencyControls)
})

onUnmounted(() => {
  document.removeEventListener('click', closeTransparencyControls)
})
</script>

<template>
  <div class="p-3">
    <div class="flex justify-center">
      <div class="glass-panel-compact flex items-center justify-center gap-2 px-4 py-2">
        <!-- AI Assistant Button -->
        <button 
          class="btn btn-circle btn-sm glass-btn-compact group flex items-center justify-center"
        >
          <SparklesIcon class="w-3.5 h-3.5 text-white/80 group-hover:text-white transition-colors" />
        </button>
        
        <!-- Microphone Button -->
        <button 
          @click="store.toggleMic"
          class="btn btn-circle btn-sm glass-btn-compact group tooltip flex items-center justify-center"
          :class="{ 'btn-primary': store.micEnabled, 'glass-btn-compact': !store.micEnabled }"
          data-tip="Toggle Microphone"
        >
          <MicrophoneIcon class="w-3.5 h-3.5 transition-colors" 
            :class="store.micEnabled ? 'text-white' : 'text-white/80 group-hover:text-white'" />
        </button>
        
        <!-- Command Mode Button (Transparency Controls) -->
        <button 
          @click="toggleTransparencyControls"
          class="btn btn-circle btn-sm glass-btn-compact group tooltip flex items-center justify-center command-btn"
          :class="{ 'btn-accent': showTransparencyControls, 'glass-btn-compact': !showTransparencyControls }"
          data-tip="Transparency Controls"
        >
          <CommandLineIcon class="w-3.5 h-3.5 transition-colors"
            :class="showTransparencyControls ? 'text-white' : 'text-white/80 group-hover:text-white'" />
        </button>
        
        <!-- Chat Button -->
        <button 
          @click="store.toggleChat"
          class="btn btn-circle btn-sm glass-btn-compact group tooltip flex items-center justify-center"
          :class="{ 'btn-accent': store.chatOpen, 'glass-btn-compact': !store.chatOpen }"
          data-tip="Toggle Chat"
        >
          <ChatBubbleLeftRightIcon class="w-3.5 h-3.5 transition-colors"
            :class="store.chatOpen ? 'text-white' : 'text-white/80 group-hover:text-white'" />
        </button>
      </div>
    </div>
    
    <!-- Transparency Controls Panel -->
    <Transition name="transparency-panel">
      <div v-if="showTransparencyControls" class="transparency-panel-container">
        <TransparencyControls />
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.glass-panel-compact {
  @apply backdrop-blur-xl border border-white/15 rounded-2xl shadow-xl;
  background: linear-gradient(to right, 
    rgba(255, 255, 255, 0.05) 0%, 
    rgba(255, 255, 255, 0.08) 50%, 
    rgba(255, 255, 255, 0.05) 100%
  );
  background-image: 
    radial-gradient(circle at 30% 50%, rgba(120, 119, 198, 0.08) 0%, transparent 50%),
    radial-gradient(circle at 70% 50%, rgba(255, 119, 198, 0.08) 0%, transparent 50%);
}

.glass-btn-compact {
  @apply bg-white/5 backdrop-blur-md border border-white/15 hover:border-white/30 hover:bg-white/10 transition-all duration-200 hover:scale-105 hover:shadow-lg;
}

.btn-sm {
  @apply w-8 h-8;
  display: flex;
  align-items: center;
  justify-content: center;
}

.glass-btn-compact:hover {
  transform: translateY(-1px) scale(1.05);
  box-shadow: 0 8px 25px rgba(0, 0, 0, 0.2);
}

/* Ensure icons are perfectly centered */
.btn-circle {
  display: flex;
  align-items: center;
  justify-content: center;
}

.btn-circle svg {
  flex-shrink: 0;
}

/* Transparency panel positioning */
.transparency-panel-container {
  @apply absolute top-full left-1/2 transform -translate-x-1/2 mt-4;
  @apply z-50;
  position: relative;
}

/* Animation for transparency panel */
.transparency-panel-enter-active,
.transparency-panel-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.transparency-panel-enter-from {
  opacity: 0;
  transform: translateX(-50%) translateY(-10px) scale(0.95);
}

.transparency-panel-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(-10px) scale(0.95);
}

.transparency-panel-enter-to,
.transparency-panel-leave-from {
  opacity: 1;
  transform: translateX(-50%) translateY(0) scale(1);
}

/* Ensure the panel container has proper positioning context */
.p-3 {
  position: relative;
}

/* Command button active state */
.command-btn.btn-accent {
  @apply bg-purple-500/20 border-purple-500/30 text-purple-300;
}

.command-btn.btn-accent:hover {
  @apply bg-purple-500/30 border-purple-500/50;
}

/* Enhanced z-index for transparency controls */
.transparency-panel-container {
  z-index: 10000 !important;
}

/* Ensure transparency controls are always interactive */
.transparency-panel-container * {
  pointer-events: auto !important;
}
</style> 