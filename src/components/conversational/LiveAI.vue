<script setup lang="ts">
import { computed } from 'vue'
import { 
  RocketLaunchIcon, 
  XMarkIcon, 
  PlayIcon, 
  StopIcon 
} from '@heroicons/vue/24/outline'

interface Props {
  show: boolean
  isActive?: boolean
  processing?: boolean
  response?: string
  error?: string | null
  sessionId?: string | null
}

interface Emits {
  (e: 'close'): void
  (e: 'toggle-live'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const handleClose = () => emit('close')
const handleToggleLive = () => emit('toggle-live')

const statusMessage = computed(() => {
  if (props.isActive && props.processing) {
    return 'Processing conversation in real-time...'
  }
  if (props.isActive) {
    return 'Listening to conversation...'
  }
  return 'Click Start to enable live AI responses'
})
</script>

<template>
  <div v-if="show" class="live-ai-drawer">
    <div class="live-ai-header">
      <div class="flex items-center gap-2">
        <RocketLaunchIcon class="w-4 h-4 text-orange-400" />
        <h3 class="text-sm font-medium text-white">Live Response Assistant</h3>
        <div v-if="isActive" class="live-indicator">
          <div class="live-dot"></div>
          <span class="text-xs text-orange-400">LIVE</span>
        </div>
      </div>
      <button @click="handleClose" class="close-drawer-btn">
        <XMarkIcon class="w-4 h-4" />
      </button>
    </div>
    
    <div class="live-ai-content">
      <!-- Control Section -->
      <div class="live-control-section">
        <button 
          @click="handleToggleLive"
          class="live-toggle-btn"
          :class="{ 'active': isActive }"
        >
          <StopIcon v-if="isActive" class="w-5 h-5" />
          <PlayIcon v-else class="w-5 h-5" />
          {{ isActive ? 'Stop Live AI' : 'Start Live AI' }}
        </button>
        <p class="text-xs text-white/60 mt-2 text-center">{{ statusMessage }}</p>
      </div>
      
      <!-- Response Area -->
      <div class="live-response-section">
        <div v-if="error" class="live-error">
          <div class="error-header">
            <XMarkIcon class="w-4 h-4 text-red-400" />
            <span class="text-sm text-red-400">Error</span>
          </div>
          <p class="text-xs text-red-300 mt-1">{{ error }}</p>
        </div>
        
        <div v-else-if="processing" class="live-processing">
          <div class="processing-indicator">
            <div class="processing-dots">
              <div class="dot"></div>
              <div class="dot"></div>
              <div class="dot"></div>
            </div>
            <span class="text-sm text-orange-400">AI is analyzing conversation...</span>
          </div>
        </div>
        
        <div v-else-if="response" class="live-response">
          <div class="response-header">
            <RocketLaunchIcon class="w-4 h-4 text-orange-400" />
            <span class="text-sm text-orange-400">Live AI Response</span>
          </div>
          <div class="response-content">
            <p class="text-sm text-white/90 leading-relaxed whitespace-pre-wrap">{{ response }}</p>
          </div>
        </div>
        
        <div v-else class="live-empty-state">
          <RocketLaunchIcon class="w-8 h-8 text-white/20 mx-auto mb-2" />
          <p class="text-white/60 text-xs text-center">Live AI Response System</p>
          <p class="text-white/40 text-xs text-center mt-1">
            The AI will provide real-time responses and insights during your conversation
          </p>
        </div>
      </div>
      
      <!-- Info Section -->
      <div class="live-info-section">
        <div class="info-card">
          <h4 class="text-xs font-medium text-white/80 mb-1">How it works:</h4>
          <ul class="text-xs text-white/60 space-y-1">
            <li>• AI listens to the conversation in real-time</li>
            <li>• Provides contextual responses automatically</li>
            <li>• Suggests relevant information as you speak</li>
            <li>• Helps maintain conversation flow</li>
          </ul>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.live-ai-drawer {
  @apply w-80 border-l border-white/10 bg-white/5 backdrop-blur-sm flex flex-col;
  min-width: 320px;
  max-width: 400px;
  background: linear-gradient(135deg, 
    rgba(20, 15, 15, 0.95) 0%,
    rgba(25, 15, 10, 0.95) 50%,
    rgba(20, 15, 15, 0.95) 100%
  );
}

.live-ai-header {
  @apply flex items-center justify-between px-4 py-3 border-b border-white/10;
  background: rgba(251, 146, 60, 0.1);
}

.live-indicator {
  @apply flex items-center gap-1 px-2 py-0.5 rounded-full bg-orange-500/20;
}

.live-dot {
  @apply w-1.5 h-1.5 bg-orange-400 rounded-full;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.6;
    transform: scale(0.8);
  }
}

.close-drawer-btn {
  @apply rounded-full p-1 hover:bg-white/10 transition-colors text-white/70 hover:text-white;
}

.live-ai-content {
  @apply flex-1 overflow-hidden flex flex-col p-4 gap-4;
}

.live-control-section {
  @apply flex flex-col items-center;
}

.live-toggle-btn {
  @apply flex items-center gap-2 px-6 py-3 rounded-lg font-medium text-sm transition-all duration-200;
  background: linear-gradient(135deg, rgba(251, 146, 60, 0.8), rgba(245, 124, 0, 0.8));
  border: 1px solid rgba(251, 146, 60, 0.4);
  color: white;
}

.live-toggle-btn:hover {
  background: linear-gradient(135deg, rgba(251, 146, 60, 0.9), rgba(245, 124, 0, 0.9));
  border-color: rgba(251, 146, 60, 0.6);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(251, 146, 60, 0.3);
}

.live-toggle-btn.active {
  background: linear-gradient(135deg, rgba(239, 68, 68, 0.8), rgba(220, 38, 38, 0.8));
  border-color: rgba(239, 68, 68, 0.4);
}

.live-toggle-btn.active:hover {
  background: linear-gradient(135deg, rgba(239, 68, 68, 0.9), rgba(220, 38, 38, 0.9));
  border-color: rgba(239, 68, 68, 0.6);
  box-shadow: 0 4px 12px rgba(239, 68, 68, 0.3);
}

.live-response-section {
  @apply flex-1 overflow-y-auto;
  min-height: 200px;
}

.live-error {
  @apply p-3 rounded-lg bg-red-500/10 border border-red-500/30;
}

.error-header {
  @apply flex items-center gap-2;
}

.live-processing {
  @apply flex items-center justify-center p-8;
}

.processing-indicator {
  @apply flex flex-col items-center gap-3;
}

.processing-dots {
  @apply flex gap-1;
}

.processing-dots .dot {
  @apply w-2 h-2 bg-orange-400 rounded-full;
  animation: bounce 1.4s infinite ease-in-out;
}

.processing-dots .dot:nth-child(2) {
  animation-delay: 0.16s;
}

.processing-dots .dot:nth-child(3) {
  animation-delay: 0.32s;
}

@keyframes bounce {
  0%, 80%, 100% {
    transform: scale(0);
    opacity: 0.5;
  }
  40% {
    transform: scale(1);
    opacity: 1;
  }
}

.live-response {
  @apply p-3 rounded-lg bg-orange-500/10 border border-orange-500/30;
}

.response-header {
  @apply flex items-center gap-2 mb-2;
}

.response-content {
  @apply pl-6;
}

.live-empty-state {
  @apply flex flex-col items-center justify-center p-8 text-center;
}

.live-info-section {
  @apply border-t border-white/10 pt-3;
}

.info-card {
  @apply p-3 rounded-lg bg-white/5;
}
</style>