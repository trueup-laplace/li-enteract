<script setup lang="ts">
import { 
  Cog6ToothIcon,
  CommandLineIcon,
  EyeIcon,
  ChatBubbleLeftRightIcon
} from '@heroicons/vue/24/outline'

interface Props {
  store: any
  mlEyeTracking: any
  showChatWindow: boolean
  showAIModelsWindow: boolean
  showConversationalWindow: boolean
  isGazeControlActive: boolean
}

interface Emits {
  (e: 'toggle-ai-models', event: Event): void
  (e: 'toggle-eye-tracking', event: Event): void
  (e: 'toggle-chat', event: Event): void
  (e: 'toggle-conversational', event: Event): void
}

defineProps<Props>()
const emit = defineEmits<Emits>()
</script>

<template>
  <div class="control-buttons-row">
    <!-- AI Settings Button -->
    <button 
      @click="emit('toggle-ai-models', $event)"
      class="control-btn group"
      :class="{ 'active': showAIModelsWindow }"
      title="AI Settings"
    >
      <Cog6ToothIcon class="w-4 h-4 transition-all" 
        :class="showAIModelsWindow ? 'text-white' : 'text-white/70 group-hover:text-white'" />
    </button>
    
    <!-- Speech Transcription Button removed - now in chat interface -->
    <!-- ML Eye Tracking + Window Movement Button -->
    <button 
      @click="emit('toggle-eye-tracking', $event)"
      class="control-btn group"
      :class="{ 
        'active': mlEyeTracking.isActive.value && mlEyeTracking.isCalibrated.value && isGazeControlActive,
        'active-warning': mlEyeTracking.isActive.value && (!mlEyeTracking.isCalibrated.value || !isGazeControlActive)
      }"
      :disabled="mlEyeTracking.isLoading.value"
      title="ML Eye Tracking + Window Movement"
    >
      <EyeIcon class="w-4 h-4 transition-all"
        :class="mlEyeTracking.isActive.value ? 'text-white' : 'text-white/70 group-hover:text-white'" />
    </button>



    <!-- Conversational Window Button -->
    <button 
      @click="emit('toggle-conversational', $event)"
      class="control-btn group"
      :class="{ 'active': showConversationalWindow }"
      title="Conversational Interface"
    >
      <ChatBubbleLeftRightIcon class="w-4 h-4 transition-all" 
        :class="showConversationalWindow ? 'text-white' : 'text-white/70 group-hover:text-white'" />
    </button>

    <!-- Chat Window Button -->
    <button 
      @click="emit('toggle-chat', $event)"
      class="control-btn group"
      :class="{ 'active': showChatWindow }"
      title="Chat Assistant"
    >
      <CommandLineIcon class="w-4 h-4 transition-all" 
        :class="showChatWindow ? 'text-white' : 'text-white/70 group-hover:text-white'" />
    </button>
  </div>
</template>

<style scoped>
.control-buttons-row {
  @apply flex items-center justify-center gap-2 px-3 py-2 relative z-10;
  height: 100%;
}

.control-btn {
  @apply rounded-full transition-all duration-200 flex items-center justify-center;
  width: 28px;
  height: 28px;
  border: none;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(8px);
  cursor: pointer;
  pointer-events: auto;
  -webkit-app-region: no-drag;
}

.control-btn:hover {
  background: rgba(255, 255, 255, 0.2);
  transform: scale(1.1);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

.control-btn.active {
  background: rgba(74, 144, 226, 0.8);
  box-shadow: 0 0 16px rgba(74, 144, 226, 0.4);
}

.control-btn.active-pulse {
  background: rgba(239, 68, 68, 0.8);
  box-shadow: 0 0 16px rgba(239, 68, 68, 0.4);
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

.control-btn.active-warning {
  background: rgba(245, 158, 11, 0.8);
  box-shadow: 0 0 16px rgba(245, 158, 11, 0.4);
}

.control-btn.active-error {
  background: rgba(239, 68, 68, 0.8);
  box-shadow: 0 0 16px rgba(239, 68, 68, 0.4);
}

.control-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.control-btn:disabled:hover {
  transform: none;
  box-shadow: none;
}

/* Pulse animation for active states */
@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.7;
  }
}
</style>