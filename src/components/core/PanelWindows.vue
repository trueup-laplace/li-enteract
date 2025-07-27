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