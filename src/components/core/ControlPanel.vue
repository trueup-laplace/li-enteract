<script setup lang="ts">
import { 
  MicrophoneIcon, 
  ChatBubbleLeftRightIcon,
  SparklesIcon,
  CommandLineIcon
} from '@heroicons/vue/24/outline'
import { useAppStore } from '../../stores/app'

const store = useAppStore()
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
        
        <!-- Command Mode Button -->
        <button 
          class="btn btn-circle btn-sm glass-btn-compact group flex items-center justify-center"
        >
          <CommandLineIcon class="w-3.5 h-3.5 text-white/80 group-hover:text-white transition-colors" />
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
</style> 