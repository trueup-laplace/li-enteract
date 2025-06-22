<script setup lang="ts">
import { 
  MicrophoneIcon, 
  ChatBubbleLeftRightIcon,
  SparklesIcon,
  CommandLineIcon
} from '@heroicons/vue/24/outline'
import { useAppStore } from '../../stores/app'

const store = useAppStore()

defineEmits<{
  toggleCollapse: []
}>()
</script>

<template>
  <div class="p-8">
    <div class="flex justify-center">
      <div class="glass-panel-enhanced flex items-center justify-center gap-6 px-10 py-6">
        <!-- AI Assistant Button -->
        <button 
          class="btn btn-circle btn-xl glass-btn-enhanced group flex items-center justify-center"
        >
          <SparklesIcon class="w-7 h-7 text-white/80 group-hover:text-white transition-colors" />
        </button>
        
        <!-- Microphone Button -->
        <button 
          @click="store.toggleMic"
          class="btn btn-circle btn-xl glass-btn-enhanced group tooltip flex items-center justify-center"
          :class="{ 'btn-primary': store.micEnabled, 'glass-btn-enhanced': !store.micEnabled }"
          data-tip="Toggle Microphone"
        >
          <MicrophoneIcon class="w-7 h-7 transition-colors" 
            :class="store.micEnabled ? 'text-white' : 'text-white/80 group-hover:text-white'" />
        </button>
        
        <!-- Command Mode Button -->
        <button 
          class="btn btn-circle btn-xl glass-btn-enhanced group flex items-center justify-center"
        >
          <CommandLineIcon class="w-7 h-7 text-white/80 group-hover:text-white transition-colors" />
        </button>
        
        <!-- Chat Button -->
        <button 
          @click="store.toggleChat"
          class="btn btn-circle btn-xl glass-btn-enhanced group tooltip flex items-center justify-center"
          :class="{ 'btn-accent': store.chatOpen, 'glass-btn-enhanced': !store.chatOpen }"
          data-tip="Toggle Chat"
        >
          <ChatBubbleLeftRightIcon class="w-7 h-7 transition-colors"
            :class="store.chatOpen ? 'text-white' : 'text-white/80 group-hover:text-white'" />
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.glass-panel-enhanced {
  @apply bg-gradient-to-r from-white/5 via-white/10 to-white/5 backdrop-blur-2xl border border-white/20 rounded-3xl shadow-2xl;
  background-image: 
    radial-gradient(circle at 20% 50%, rgba(120, 119, 198, 0.1) 0%, transparent 50%),
    radial-gradient(circle at 80% 50%, rgba(255, 119, 198, 0.1) 0%, transparent 50%);
}

.glass-btn-enhanced {
  @apply bg-white/5 backdrop-blur-lg border border-white/20 hover:border-white/40 hover:bg-white/10 transition-all duration-300 hover:scale-110 hover:shadow-xl;
}

.btn-xl {
  @apply w-16 h-16;
  display: flex;
  align-items: center;
  justify-content: center;
}

.glass-btn-enhanced:hover {
  transform: translateY(-2px) scale(1.05);
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
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