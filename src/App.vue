<script setup lang="ts">
import { onMounted } from 'vue'
import { useWindowManager } from './composables/useWindowManager'
import ControlPanel from './components/core/ControlPanel.vue'

// Initialize window manager
const { initializeWindow } = useWindowManager()

onMounted(() => {
  initializeWindow()
})
</script>

<template>
  <div class="app-root">
    <ControlPanel />
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


