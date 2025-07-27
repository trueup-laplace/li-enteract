<script setup lang="ts">
import ChatWindow from './ChatWindow.vue'
import SettingsPanel from './SettingsPanel.vue' // Updated import
import ConversationalWindow from './ConversationalWindow.vue'

interface Props {
  showSettingsPanel: boolean  // Renamed from showAIModelsWindow
  showChatWindow: boolean
  showConversationalWindow: boolean
  selectedModel: any
}

interface Emits {
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

/* Enhanced Settings Panel Transitions */
.settings-panel-enter-active {
  transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
  transition-delay: 0.1s;
}

.settings-panel-leave-active {
  transition: all 0.25s cubic-bezier(0.55, 0.085, 0.68, 0.53);
}

.settings-panel-enter-from {
  opacity: 0;
  transform: translateY(-20px) scale(0.9) rotateX(15deg);
  filter: blur(8px);
}

.settings-panel-leave-to {
  opacity: 0;
  transform: translateY(-15px) scale(0.92);
  filter: blur(4px);
}

/* Enhanced Conversational Panel Transitions */
.conversational-panel-enter-active {
  transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
  transition-delay: 0.1s;
}

.conversational-panel-leave-active {
  transition: all 0.25s cubic-bezier(0.55, 0.085, 0.68, 0.53);
}

.conversational-panel-enter-from {
  opacity: 0;
  transform: translateY(-20px) scale(0.9) rotateX(15deg);
  filter: blur(8px);
}

.conversational-panel-leave-to {
  opacity: 0;
  transform: translateY(-15px) scale(0.92);
  filter: blur(4px);
}

/* Enhanced Chat Panel Transitions */
.chat-panel-enter-active {
  transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
  transition-delay: 0.1s;
}

.chat-panel-leave-active {
  transition: all 0.25s cubic-bezier(0.55, 0.085, 0.68, 0.53);
}

.chat-panel-enter-from {
  opacity: 0;
  transform: translateY(-20px) scale(0.9) rotateX(15deg);
  filter: blur(8px);
}

.chat-panel-leave-to {
  opacity: 0;
  transform: translateY(-15px) scale(0.92);
  filter: blur(4px);
}

/* Smooth morphing between windows */
.settings-panel-section,
.conversational-panel-section,
.chat-panel-section {
  transform-origin: center top;
  will-change: transform, opacity, filter;
  backface-visibility: hidden;
  perspective: 1000px;
}

/* Additional spring effect for entering elements */
.settings-panel-enter-active .settings-drawer,
.conversational-panel-enter-active > *,
.chat-panel-enter-active > * {
  animation: spring-in 0.6s cubic-bezier(0.175, 0.885, 0.32, 1.275) forwards;
}

@keyframes spring-in {
  0% {
    transform: scale(0.3) translateY(-50px);
    opacity: 0;
  }
  50% {
    transform: scale(1.05) translateY(-10px);
    opacity: 0.8;
  }
  70% {
    transform: scale(0.98) translateY(-2px);
    opacity: 0.95;
  }
  100% {
    transform: scale(1) translateY(0);
    opacity: 1;
  }
}
</style>