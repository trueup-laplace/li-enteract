<script setup lang="ts">
import { ref } from 'vue'
import { 
  PaperAirplaneIcon,
  XMarkIcon
} from '@heroicons/vue/24/outline'
import { useAppStore } from '../../stores/app'

const store = useAppStore()
const newMessage = ref('')

const sendMessage = () => {
  if (!newMessage.value.trim()) return
  
  store.addMessage(newMessage.value, 'user')
  
  // Simulate assistant response
  setTimeout(() => {
    store.addMessage("I understand. Let me help you with that.", 'assistant')
  }, 1000)
  
  newMessage.value = ''
}
</script>

<template>
  <!-- Chat Drawer -->
  <div 
    class="fixed top-0 right-0 h-full w-96 z-50 transform transition-all duration-500 ease-out"
    :class="store.chatOpen ? 'translate-x-0' : 'translate-x-full'"
  >
    <div class="h-full chat-panel flex flex-col">
      <!-- Chat Header -->
      <div class="flex items-center justify-between p-4 border-b border-white/10">
        <div class="flex items-center gap-3">
          <div class="w-2 h-2 rounded-full bg-green-400 animate-pulse"></div>
          <h3 class="text-lg font-medium text-white/90">Assistant Chat</h3>
        </div>
        <button @click="store.toggleChat" class="btn btn-sm btn-circle btn-ghost hover:bg-white/10">
          <XMarkIcon class="w-4 h-4 text-white/70" />
        </button>
      </div>
      
      <!-- Chat Messages -->
      <div class="flex-1 overflow-y-auto p-4 space-y-4 custom-scrollbar">
        <div 
          v-for="message in store.chatMessages" 
          :key="message.id"
          class="flex animate-fade-in"
          :class="message.sender === 'user' ? 'justify-end' : 'justify-start'"
        >
          <div 
            class="max-w-xs rounded-2xl px-4 py-3 text-sm shadow-lg"
            :class="message.sender === 'user' 
              ? 'bg-gradient-to-r from-blue-500 to-purple-600 text-white' 
              : 'bg-white/5 text-white/90 border border-white/10 backdrop-blur-sm'"
          >
            {{ message.text }}
          </div>
        </div>
      </div>
      
      <!-- Chat Input -->
      <div class="p-4 border-t border-white/10">
        <div class="flex gap-3">
          <input 
            v-model="newMessage"
            @keyup.enter="sendMessage"
            type="text" 
            placeholder="Type your message..."
            class="input-enhanced flex-1"
          />
          <button 
            @click="sendMessage"
            class="btn-send"
            :disabled="!newMessage.trim()"
          >
            <PaperAirplaneIcon class="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  </div>
  
  <!-- Backdrop -->
  <div 
    v-if="store.chatOpen"
    @click="store.toggleChat"
    class="fixed inset-0 bg-black/20 backdrop-blur-sm z-40 transition-all duration-500"
  ></div>
</template>

<style scoped>
.chat-panel {
  background: linear-gradient(
    135deg,
    rgba(0, 0, 0, 0.4) 0%,
    rgba(0, 0, 0, 0.3) 50%,
    rgba(0, 0, 0, 0.2) 100%
  );
  backdrop-filter: blur(40px) saturate(180%);
  border-left: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 
    -20px 0 60px rgba(0, 0, 0, 0.3),
    inset 1px 0 0 rgba(255, 255, 255, 0.1);
}

.input-enhanced {
  @apply bg-white/5 border border-white/20 rounded-xl px-4 py-3 text-white placeholder-white/50 focus:bg-white/10 focus:border-white/30 transition-all;
  backdrop-filter: blur(20px);
}

.input-enhanced:focus {
  outline: none;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.3);
}

.btn-send {
  @apply w-12 h-12 rounded-xl bg-gradient-to-r from-blue-500 to-purple-600 hover:from-blue-600 hover:to-purple-700 text-white flex items-center justify-center transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed;
  box-shadow: 0 4px 15px rgba(59, 130, 246, 0.3);
}

.btn-send:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(59, 130, 246, 0.4);
}

.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 2px;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
}

@keyframes fade-in {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.animate-fade-in {
  animation: fade-in 0.3s ease-out;
}
</style> 