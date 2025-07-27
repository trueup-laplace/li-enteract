<script setup lang="ts">
import { AdjustmentsHorizontalIcon, XMarkIcon } from '@heroicons/vue/24/outline'
import TransparencyControls from './TransparencyControls.vue'
import ChatWindow from './ChatWindow.vue'
import SettingsPanel from './SettingsPanel.vue' // Updated import
import ConversationalWindow from './ConversationalWindow.vue'

interface Props {
  showTransparencyControls: boolean
  showSettingsPanel: boolean  // Renamed from showAIModelsWindow
  showChatWindow: boolean
  showConversationalWindow: boolean
  selectedModel: any
}

interface Emits {
  (e: 'close-transparency'): void
  (e: 'close-settings'): void  // Renamed from close-ai-models
  (e: 'close-chat'): void
  (e: 'close-conversational'): void
  (e: 'update:show-settings-panel', value: boolean): void  // Renamed
  (e: 'update:show-chat-window', value: boolean): void
  (e: 'update:show-conversational-window', value: boolean): void
  (e: 'update:selected-model', value: string): void
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

  <!-- Settings Panel Section -->
  <Transition name="settings-panel">
    <div v-if="showSettingsPanel" class="settings-panel-section">
      <SettingsPanel 
        :show-settings-panel="showSettingsPanel"
        @close="emit('close-settings')"
        @update:show-settings-panel="emit('update:show-settings-panel', $event)"
      />
    </div>
  </Transition>

  <!-- Conversational Window Section -->
  <Transition name="conversational-panel">
    <div v-if="showConversationalWindow" class="conversational-panel-section">
      <ConversationalWindow 
        :show-conversational-window="showConversationalWindow"
        @close="emit('close-conversational')"
        @update:show-conversational-window="emit('update:show-conversational-window', $event)"
      />
    </div>
  </Transition>

  <!-- Chat Window Section -->
  <Transition name="chat-panel">
    <div v-if="showChatWindow" class="chat-panel-section">
      <ChatWindow 
        :show-chat-window="showChatWindow"
        :selected-model="selectedModel"
        @close="emit('close-chat')"
        @update:show-chat-window="emit('update:show-chat-window', $event)"
        @update:selected-model="emit('update:selected-model', $event)"
        @toggle-chat-drawer="emit('toggle-chat-drawer')"
      />
    </div>
  </Transition>
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
    rgba(10, 10, 12, 0.90) 0%,
    rgba(10, 10, 12, 0.80) 25%,
    rgba(10, 10, 12, 0.75) 50%,
    rgba(10, 10, 12, 0.80) 75%,
    rgba(10, 10, 12, 0.90) 100%
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

/* Settings Panel Section */
.settings-panel-section {
  @apply w-full flex justify-center;
  padding: 0 8px 8px 8px;
  background: transparent;
}

/* Conversational Panel Section */
.conversational-panel-section {
  @apply w-full flex justify-center;
  padding: 8px;
  background: transparent;
  position: relative;
}

/* Chat Panel Section */
.chat-panel-section {
  @apply w-full flex justify-center;
  padding: 8px;
  background: transparent;
  position: relative;
}

/* Settings Panel Transitions */
.settings-panel-enter-active,
.settings-panel-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.settings-panel-enter-from {
  opacity: 0;
  transform: translateY(-10px) scale(0.95);
}

.settings-panel-leave-to {
  opacity: 0;
  transform: translateY(-10px) scale(0.95);
}

/* Conversational Panel Transitions */
.conversational-panel-enter-active,
.conversational-panel-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.conversational-panel-enter-from {
  opacity: 0;
  transform: translateY(-10px) scale(0.95);
}

.conversational-panel-leave-to {
  opacity: 0;
  transform: translateY(-10px) scale(0.95);
}

/* Chat Panel Transitions */
.chat-panel-enter-active,
.chat-panel-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.chat-panel-enter-from {
  opacity: 0;
  transform: translateY(-10px) scale(0.95);
}

.chat-panel-leave-to {
  opacity: 0;
  transform: translateY(-10px) scale(0.95);
}
</style>