<script setup lang="ts">
import { ref } from 'vue'
import { 
  PaperAirplaneIcon,
  XMarkIcon,
  MicrophoneIcon
} from '@heroicons/vue/24/outline'
import { useAppStore } from '../../stores/app'

const store = useAppStore()
const newMessage = ref('')
const apiKey = ref('')
const showApiKeyInput = ref(false)

const sendMessage = () => {
  if (!newMessage.value.trim()) return
  
  store.addMessage(newMessage.value, 'user', { source: 'typed' })
  
  // Simulate assistant response
  setTimeout(() => {
    store.addMessage("I understand. Let me help you with that.", 'assistant')
  }, 1000)
  
  newMessage.value = ''
}

// Speech transcription - minimal controls
const toggleSpeechFromChat = async () => {
  if (store.speechStatus.isRecording) {
    await store.stopSpeechTranscription()
  } else {
    if (!store.speechStatus.isInitialized) {
      await store.initializeSpeechTranscription('base')
    }
    await store.startSpeechTranscription()
  }
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
            class="max-w-xs rounded-2xl px-4 py-3 text-sm shadow-lg relative"
            :class="{
              'bg-gradient-to-r from-blue-500 to-purple-600 text-white': message.sender === 'user',
              'bg-white/5 text-white/90 border border-white/10 backdrop-blur-sm': message.sender === 'assistant',
              'bg-gradient-to-r from-orange-500/20 to-red-500/20 text-orange-200 border border-orange-400/30 backdrop-blur-sm': message.sender === 'transcription',
              'opacity-60 italic': message.isInterim
            }"
          >
            {{ message.text }}
            
            <!-- Transcription metadata -->
            <div v-if="message.sender === 'transcription' && !message.isInterim" class="mt-1 flex items-center gap-1 text-xs opacity-70">
              <span v-if="message.source === 'whisper'" class="px-1 py-0.5 rounded bg-purple-500/30 text-purple-200">
                Whisper
              </span>
              <span v-else-if="message.source === 'web-speech'" class="px-1 py-0.5 rounded bg-blue-500/30 text-blue-200">
                WebSpeech
              </span>
              <span v-if="message.confidence" class="text-xs">
                {{ Math.round(message.confidence * 100) }}%
              </span>
            </div>
          </div>
        </div>
      </div>
      
      <!-- Chat Input -->
      <div class="p-4 border-t border-white/10">
        <!-- Speech Transcription Info -->
        <div v-if="store.speechStatus.error" class="mb-3 text-sm text-red-300 bg-red-500/10 border border-red-500/20 rounded-lg p-2">
          Speech Error: {{ store.speechStatus.error }}
        </div>

        <!-- Clear Transcription Button -->
        <div v-if="store.isTranscriptionEnabled" class="mb-3 flex justify-center">
          <button 
            @click="store.clearTranscription"
            class="btn-clear text-xs px-3 py-1"
            title="Clear All Transcriptions"
          >
            Clear Transcriptions
          </button>
        </div>

        <!-- Text Input with Minimal Mic -->
        <div class="flex gap-2">
          <input 
            v-model="newMessage"
            @keyup.enter="sendMessage"
            type="text" 
            placeholder="Type your message..."
            class="input-enhanced flex-1"
          />
          
          <!-- Minimal Mic Button -->
          <button 
            @click="toggleSpeechFromChat"
            class="btn-mic-minimal"
            :class="{
              'btn-mic-recording': store.speechStatus.isRecording,
              'btn-mic-processing': store.speechStatus.isProcessing,
              'btn-mic-ready': store.isTranscriptionEnabled && !store.speechStatus.isRecording
            }"
            :disabled="store.speechStatus.isProcessing"
            :title="store.speechStatus.isRecording ? 'Stop Recording' : 'Start Speech Recording'"
          >
            <MicrophoneIcon class="w-4 h-4" />
          </button>
          
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

.btn-clear {
  @apply bg-white/10 hover:bg-white/20 text-white/70 hover:text-white rounded-lg transition-all duration-200;
}

/* Minimal mic button */
.btn-mic-minimal {
  @apply w-12 h-12 rounded-xl bg-white/5 hover:bg-white/10 border border-white/20 hover:border-white/30 text-white/70 hover:text-white flex items-center justify-center transition-all duration-200;
}

.btn-mic-ready {
  @apply bg-green-500/20 hover:bg-green-500/30 border-green-400/40 text-green-300;
}

.btn-mic-recording {
  @apply bg-red-500/30 border-red-400/50 text-red-300 animate-pulse;
}

.btn-mic-processing {
  @apply bg-yellow-500/20 border-yellow-400/40 text-yellow-300;
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