<script setup lang="ts">
import { ref, computed } from 'vue'
import { 
  SparklesIcon, 
  XMarkIcon, 
  PaperAirplaneIcon, 
  RocketLaunchIcon 
} from '@heroicons/vue/24/outline'

interface Props {
  show: boolean
  processing?: boolean
  response?: string
  error?: string | null
  messageCount?: number
}

interface Emits {
  (e: 'close'): void
  (e: 'query', query: string): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const aiQuery = ref('')

const handleClose = () => emit('close')

const queryAIAssistant = () => {
  if (aiQuery.value.trim() && !props.processing) {
    emit('query', aiQuery.value)
  }
}

const handleAIQueryKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault()
    queryAIAssistant()
  }
}

const setSuggestion = (suggestion: string) => {
  aiQuery.value = suggestion
  queryAIAssistant()
}

const showSuggestions = computed(() => {
  return !props.processing && !props.response && props.messageCount && props.messageCount > 0
})
</script>

<template>
  <div v-if="show" class="ai-assistant-drawer">
    <div class="ai-drawer-header">
      <div class="flex items-center gap-2">
        <SparklesIcon class="w-4 h-4 text-blue-400" />
        <h3 class="text-sm font-medium text-white">AI Conversation Assistant</h3>
      </div>
      <button @click="handleClose" class="close-drawer-btn">
        <XMarkIcon class="w-4 h-4" />
      </button>
    </div>
    
    <div class="ai-drawer-content">
      <!-- AI Query Input -->
      <div class="ai-query-section">
        <textarea
          v-model="aiQuery"
          @keydown="handleAIQueryKeydown"
          class="ai-query-input"
          placeholder="Ask the AI assistant about this conversation..."
          rows="3"
          :disabled="processing"
        />
        <button 
          @click="queryAIAssistant"
          :disabled="!aiQuery.trim() || processing"
          class="ai-query-btn"
        >
          <RocketLaunchIcon v-if="processing" class="w-4 h-4 animate-pulse" />
          <PaperAirplaneIcon v-else class="w-4 h-4" />
        </button>
      </div>
      
      <!-- AI Response Area -->
      <div class="ai-response-section">
        <div v-if="error" class="ai-error">
          <div class="error-header">
            <XMarkIcon class="w-4 h-4 text-red-400" />
            <span class="text-sm text-red-400">Error</span>
          </div>
          <p class="text-xs text-red-300 mt-1">{{ error }}</p>
        </div>
        
        <div v-else-if="processing" class="ai-processing">
          <div class="processing-indicator">
            <RocketLaunchIcon class="w-5 h-5 text-blue-400 animate-pulse" />
            <span class="text-sm text-blue-400">AI is thinking...</span>
          </div>
        </div>
        
        <div v-else-if="response" class="ai-response">
          <div class="response-header">
            <SparklesIcon class="w-4 h-4 text-green-400" />
            <span class="text-sm text-green-400">AI Assistant</span>
          </div>
          <div class="response-content">
            <p class="text-sm text-white/90 leading-relaxed whitespace-pre-wrap">{{ response }}</p>
          </div>
        </div>
        
        <div v-else class="ai-empty-state">
          <SparklesIcon class="w-8 h-8 text-white/20 mx-auto mb-2" />
          <p class="text-white/60 text-xs text-center">Ask the AI assistant about your conversation</p>
          <p class="text-white/40 text-xs text-center mt-1">The AI will analyze your conversation context to provide helpful insights</p>
        </div>
      </div>
      
      <!-- Quick Suggestions -->
      <div v-if="showSuggestions" class="ai-suggestions">
        <p class="text-xs text-white/60 mb-2">Quick suggestions:</p>
        <div class="suggestion-buttons">
          <button @click="setSuggestion('Summarize this conversation')" class="suggestion-btn">
            📝 Summarize
          </button>
          <button @click="setSuggestion('What are the key points discussed?')" class="suggestion-btn">
            🎯 Key Points
          </button>
          <button @click="setSuggestion('Suggest follow-up questions')" class="suggestion-btn">
            ❓ Follow-ups
          </button>
          <button @click="setSuggestion('What should I do next?')" class="suggestion-btn">
            🚀 Next Steps
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.ai-assistant-drawer {
  @apply w-80 border-l border-white/10 bg-white/5 backdrop-blur-sm flex flex-col;
  min-width: 320px;
  max-width: 400px;
  background: linear-gradient(135deg, 
    rgba(15, 15, 20, 0.95) 0%,
    rgba(10, 15, 25, 0.95) 50%,
    rgba(15, 15, 20, 0.95) 100%
  );
}

.ai-drawer-header {
  @apply flex items-center justify-between px-4 py-3 border-b border-white/10;
  background: rgba(59, 130, 246, 0.1);
}

.close-drawer-btn {
  @apply rounded-full p-1 hover:bg-white/10 transition-colors text-white/70 hover:text-white;
}

.ai-drawer-content {
  @apply flex-1 overflow-hidden flex flex-col p-4 gap-4;
}

.ai-query-section {
  @apply flex flex-col gap-2;
}

.ai-query-input {
  @apply w-full border border-white/20 rounded-lg px-3 py-2 text-white placeholder-white/50 focus:outline-none focus:border-blue-500/50 focus:ring-2 focus:ring-blue-500/20 transition-all duration-200 resize-none;
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(10px);
  font-size: 13px;
}

.ai-query-input:focus {
  background: rgba(255, 255, 255, 0.08);
}

.ai-query-btn {
  @apply self-end rounded-lg p-2 transition-all duration-200 flex items-center justify-center;
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.8), rgba(37, 99, 235, 0.8));
  border: 1px solid rgba(59, 130, 246, 0.4);
  color: white;
  min-width: 36px;
  min-height: 36px;
}

.ai-query-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.9), rgba(37, 99, 235, 0.9));
  border-color: rgba(59, 130, 246, 0.6);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
}

.ai-query-btn:disabled {
  background: rgba(255, 255, 255, 0.05);
  border-color: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.3);
  cursor: not-allowed;
}

.ai-response-section {
  @apply flex-1 overflow-y-auto;
  min-height: 200px;
}

.ai-error {
  @apply p-3 rounded-lg bg-red-500/10 border border-red-500/30;
}

.error-header {
  @apply flex items-center gap-2;
}

.ai-processing {
  @apply flex items-center justify-center p-8;
}

.processing-indicator {
  @apply flex flex-col items-center gap-2;
}

.ai-response {
  @apply p-3 rounded-lg bg-green-500/10 border border-green-500/30;
}

.response-header {
  @apply flex items-center gap-2 mb-2;
}

.response-content {
  @apply pl-6;
}

.ai-empty-state {
  @apply flex flex-col items-center justify-center p-8 text-center;
}

.ai-suggestions {
  @apply border-t border-white/10 pt-3;
}

.suggestion-buttons {
  @apply grid grid-cols-2 gap-2;
}

.suggestion-btn {
  @apply px-3 py-2 text-xs rounded-lg bg-white/5 hover:bg-white/10 text-white/70 hover:text-white transition-all duration-200 text-left;
}
</style>