<script setup lang="ts">
import { AdjustmentsHorizontalIcon, XMarkIcon } from '@heroicons/vue/24/outline'
import TransparencyControls from './TransparencyControls.vue'
import ChatWindow from './ChatWindow.vue'
import SettingsPanel from './SettingsPanel.vue' // Updated import

interface Props {
  showTransparencyControls: boolean
  showSettingsPanel: boolean  // Renamed from showAIModelsWindow
  showChatWindow: boolean
  selectedModel: any
}

interface Emits {
  (e: 'close-transparency'): void
  (e: 'close-settings'): void  // Renamed from close-ai-models
  (e: 'close-chat'): void
  (e: 'update:show-settings-panel', value: boolean): void  // Renamed
  (e: 'update:show-chat-window', value: boolean): void
  (e: 'toggle-chat-drawer'): void
}

defineProps<Props>()
const emit = defineEmits<Emits>()
</script>

<template>
  <!-- Transparency Controls Section -->
  <Transition name="transparency-panel">
    <div v-if="showTransparencyControls" class="transparency-controls-section">
      <div class="transparency-controls-panel">
        <div class="panel-header">
          <div class="panel-title">
            <AdjustmentsHorizontalIcon class="w-4 h-4 text-white/80" />
            <span class="text-sm font-medium text-white/90">Transparency Controls</span>
          </div>
          <button @click="emit('close-transparency')" class="panel-close-btn">
            <XMarkIcon class="w-4 h-4 text-white/70 hover:text-white transition-colors" />
          </button>
        </div>
        <div class="panel-content">
          <TransparencyControls />
        </div>
      </div>
    </div>
  </Transition>

  <!-- Settings Panel (Enhanced) -->
  <SettingsPanel 
    :show-settings-panel="showSettingsPanel"
    @close="emit('close-settings')"
    @update:show-settings-panel="emit('update:show-settings-panel', $event)"
  />

  <!-- Chat Window -->
  <ChatWindow 
    :show-chat-window="showChatWindow"
    :selected-model="selectedModel"
    @close="emit('close-chat')"
    @update:show-chat-window="emit('update:show-chat-window', $event)"
    @toggle-chat-drawer="emit('toggle-chat-drawer')"
  />
</template>

<style scoped>
.transparency-controls-section {
  @apply w-full flex justify-center;
  padding: 0 8px 8px 8px;
  background: transparent;
}

/* Transparency Controls Panel */
.transparency-controls-panel {
  @apply rounded-2xl overflow-hidden;
  width: 380px;
  pointer-events: auto;
  
  /* Same glass effect as other panels with darker background */
  background: linear-gradient(135deg, 
    rgba(17, 17, 21, 0.85) 0%,
    rgba(17, 17, 21, 0.75) 25%,
    rgba(17, 17, 21, 0.70) 50%,
    rgba(17, 17, 21, 0.75) 75%,
    rgba(17, 17, 21, 0.85) 100%
  );
  backdrop-filter: blur(60px) saturate(180%) brightness(1.1);
  border: 1px solid rgba(255, 255, 255, 0.25);
  box-shadow: 
    0 20px 60px rgba(0, 0, 0, 0.4),
    0 8px 24px rgba(0, 0, 0, 0.25),
    inset 0 1px 0 rgba(255, 255, 255, 0.3),
    inset 0 -1px 0 rgba(0, 0, 0, 0.1);
}

.panel-header {
  @apply flex items-center justify-between px-4 py-3 border-b border-white/10;
}

.panel-title {
  @apply flex items-center gap-2;
}

.panel-close-btn {
  @apply rounded-full p-1 hover:bg-white/10 transition-colors;
}

.panel-content {
  @apply p-4;
}

/* Panel Transitions */
.transparency-panel-enter-active,
.transparency-panel-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.transparency-panel-enter-from {
  opacity: 0;
  transform: translateY(-10px) scale(0.95);
}

.transparency-panel-leave-to {
  opacity: 0;
  transform: translateY(-10px) scale(0.95);
}
</style>