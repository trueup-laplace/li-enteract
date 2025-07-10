<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useWindowManager } from './composables/useWindowManager'
import { useAIModels } from './composables/useAIModels'
import ControlPanel from './components/core/ControlPanel.vue'
import ChatSidebar from './components/core/ChatSidebar.vue'

// Initialize window manager
const { initializeWindow } = useWindowManager()

// AI Models management for chat sidebar
const { selectedModel } = useAIModels()

// Chat drawer state
const isChatDrawerOpen = ref(false)

const toggleChatDrawer = () => {
  isChatDrawerOpen.value = !isChatDrawerOpen.value
  console.log(`💬 Chat drawer ${isChatDrawerOpen.value ? 'opened' : 'closed'}`)
}

const closeChatDrawer = () => {
  isChatDrawerOpen.value = false
  console.log('💬 Chat drawer closed')
}

onMounted(() => {
  initializeWindow()
  
  // Listen for chat drawer toggle events from ChatWindow
  window.addEventListener('toggle-chat-drawer', toggleChatDrawer)
})
</script>

<template>
  <div class="app-root">
    <ControlPanel @toggle-chat-drawer="toggleChatDrawer" />
    
    <!-- Chat Drawer -->
    <ChatSidebar 
      :is-open="isChatDrawerOpen"
      :selected-model="selectedModel"
      @close="closeChatDrawer"
    />
  </div>
</template>

<style>
/* Reset everything to ensure no extra space */
*, *::before, *::after {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

html, body {
  margin: 0;
  padding: 0;
  height: auto;
  width: auto;
  overflow: hidden;
  background: transparent;
}

#app {
  margin: 0;
  padding: 0;
  height: auto;
  width: auto;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
}

.app-root {
  margin: 0;
  padding: 0;
  height: auto;
  width: auto;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
}

/* Remove any default body styling */
body {
  background: transparent !important;
  overflow: hidden !important;
}

/* Ensure no scrollbars */
::-webkit-scrollbar {
  display: none;
}

html {
  -ms-overflow-style: none;
  scrollbar-width: none;
}
</style>


