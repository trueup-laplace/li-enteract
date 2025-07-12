<script setup lang="ts">
import { watch } from 'vue'
import {
  Cog6ToothIcon,
  XMarkIcon,
  ArrowsPointingOutIcon,
  TrashIcon,
  ArrowDownTrayIcon
} from '@heroicons/vue/24/outline'
import { useAIModels } from '../../composables/useAIModels'

interface Props {
  showAIModelsWindow: boolean
}

interface Emits {
  (e: 'close'): void
  (e: 'update:showAIModelsWindow', value: boolean): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const {
  ollamaModels,
  ollamaStatus,
  isLoadingModels,
  modelsError,
  selectedModel,
  pullingModel,
  deletingModel,
  fetchOllamaStatus,
  fetchOllamaModels,
  pullModel,
  deleteModel,
  formatModelSize,
  getModelDisplayName
} = useAIModels()

// Watch for AI models window state changes and fetch data when opened
watch(() => props.showAIModelsWindow, async (newValue) => {
  if (newValue) {
    await fetchOllamaStatus()
    if (ollamaStatus.value.status === 'running') {
      await fetchOllamaModels()
    }
  }
})

const closePanel = () => {
  emit('close')
  emit('update:showAIModelsWindow', false)
}

const clearError = () => {
  modelsError.value = null
}
</script>

<template>
  <Transition name="ai-models-panel">
    <div v-if="showAIModelsWindow" class="ai-models-section">
      <div class="ai-models-panel">
        <div class="panel-header">
          <div class="panel-title">
            <Cog6ToothIcon class="w-4 h-4 text-white/80" />
            <span class="text-sm font-medium text-white/90">AI Settings (Ollama)</span>
            <div class="status-indicator" :class="{
              'text-green-400': ollamaStatus.status === 'running',
              'text-red-400': ollamaStatus.status === 'not_running',
              'text-yellow-400': ollamaStatus.status === 'checking' || ollamaStatus.status === 'error'
            }">
              {{ ollamaStatus.status === 'running' ? '●' : ollamaStatus.status === 'not_running' ? '●' : '●' }}
            </div>
          </div>
          <button @click="closePanel" class="panel-close-btn">
            <XMarkIcon class="w-4 h-4 text-white/70 hover:text-white transition-colors" />
          </button>
        </div>
        
        <div class="panel-content">
          <!-- Ollama Status -->
          <div class="ollama-status">
            <div v-if="ollamaStatus.status === 'running'" class="status-good">
              <span class="text-green-400">● Ollama is running</span>
              <span v-if="ollamaStatus.version" class="text-white/60 text-xs ml-2">v{{ ollamaStatus.version }}</span>
            </div>
            <div v-else-if="ollamaStatus.status === 'not_running'" class="status-error">
              <span class="text-red-400">● Ollama is not running</span>
              <p class="text-white/60 text-xs mt-1">Please start Ollama to manage models</p>
            </div>
            <div v-else-if="ollamaStatus.status === 'checking'" class="status-loading">
              <span class="text-yellow-400">● Checking Ollama status...</span>
            </div>
            <div v-else class="status-error">
              <span class="text-red-400">● Failed to connect to Ollama</span>
            </div>
          </div>
          
          <!-- Models List -->
          <div v-if="ollamaStatus.status === 'running'" class="models-section">
            <div class="models-header">
              <h3 class="text-white/90 font-medium">Available Models</h3>
              <button 
                @click="fetchOllamaModels" 
                :disabled="isLoadingModels"
                class="refresh-btn"
                title="Refresh Models"
              >
                <ArrowsPointingOutIcon class="w-4 h-4" :class="{ 'animate-spin': isLoadingModels }" />
              </button>
            </div>
            
            <!-- Error Message -->
            <div v-if="modelsError" class="error-message">
              <span class="text-red-400 text-sm">{{ modelsError }}</span>
              <button @click="clearError" class="ml-2 text-white/60 hover:text-white">×</button>
            </div>
            
            <!-- Loading State -->
            <div v-if="isLoadingModels" class="loading-state">
              <div class="animate-pulse text-white/60">Loading models...</div>
            </div>
            
            <!-- Models List -->
            <div v-else-if="ollamaModels.length > 0" class="models-list">
              <div v-for="model in ollamaModels" :key="model.name" class="model-item">
                <div class="model-info">
                  <div class="model-name">{{ getModelDisplayName(model) }}</div>
                  <div class="model-details">
                    <span class="model-size">{{ formatModelSize(model.size) }}</span>
                    <span v-if="model.details?.parameter_size" class="model-params">
                      {{ model.details.parameter_size }}
                    </span>
                  </div>
                </div>
                
                <div class="model-actions">
                  <button
                    @click="selectedModel = model.name"
                    :class="{ 'active': selectedModel === model.name }"
                    class="select-btn"
                    title="Select Model"
                  >
                    {{ selectedModel === model.name ? '✓' : '○' }}
                  </button>
                  
                  <button
                    @click="deleteModel(model.name)"
                    :disabled="deletingModel === model.name"
                    class="delete-btn"
                    title="Delete Model"
                  >
                    <TrashIcon v-if="deletingModel !== model.name" class="w-3 h-3" />
                    <div v-else class="w-3 h-3 animate-spin">⟳</div>
                  </button>
                </div>
              </div>
            </div>
            
            <!-- No Models -->
            <div v-else class="no-models">
              <p class="text-white/60 text-sm">No models available</p>
              <p class="text-white/40 text-xs mt-1">Pull a model to get started</p>
            </div>
            
            <!-- Pull Model Section -->
            <div class="pull-model-section">
              <h4 class="text-white/80 text-sm font-medium mb-2">Pull New Model</h4>
              <div class="popular-models">
                <button 
                  v-for="modelName in ['gemma3:1b-it-qat', 'qwen2.5vl:3b', 'deepseek-r1:1.5b', 'llama3.2']" 
                  :key="modelName"
                  @click="pullModel(modelName)"
                  :disabled="pullingModel === modelName"
                  class="model-pull-btn"
                  :class="{ 
                    'recommended': modelName === 'gemma3:1b-it-qat',
                    'vision-model': modelName === 'qwen2.5vl:3b',
                    'research-model': modelName === 'deepseek-r1:1.5b'
                  }"
                >
                  <ArrowDownTrayIcon v-if="pullingModel !== modelName" class="w-3 h-3" />
                  <div v-else class="w-3 h-3 animate-spin">⟳</div>
                  <span>{{ modelName }}</span>
                  <span v-if="modelName === 'gemma3:1b-it-qat'" class="recommended-badge">Enteract Agent</span>
                  <span v-if="modelName === 'qwen2.5vl:3b'" class="vision-badge">Vision</span>
                  <span v-if="modelName === 'deepseek-r1:1.5b'" class="research-badge">Research</span>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.ai-models-section {
  @apply w-full flex justify-center;
  padding: 0 8px 8px 8px;
  background: transparent;
}

.ai-models-panel {
  @apply rounded-2xl overflow-hidden;
  width: 420px;
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

.status-indicator {
  @apply ml-2 text-xs;
}

.panel-close-btn {
  @apply rounded-full p-1 hover:bg-white/10 transition-colors;
}

.panel-content {
  @apply p-4;
}

.ollama-status {
  @apply mb-4;
}

.models-section {
  @apply mt-4;
}

.models-header {
  @apply flex items-center justify-between mb-3;
}

.refresh-btn {
  @apply p-1 rounded-lg bg-white/10 hover:bg-white/20 transition-colors text-white/70 hover:text-white;
}

.error-message {
  @apply flex items-center justify-between bg-red-500/20 border border-red-400/30 rounded-lg p-2 mb-3;
}

.loading-state {
  @apply p-4 text-center;
}

.models-list {
  @apply space-y-2 mb-4 max-h-48 overflow-y-auto;
  scrollbar-width: thin;
  scrollbar-color: rgba(255, 255, 255, 0.2) transparent;
}

.models-list::-webkit-scrollbar {
  width: 4px;
}

.models-list::-webkit-scrollbar-track {
  background: transparent;
}

.models-list::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
}

.model-item {
  @apply flex items-center justify-between p-3 bg-white/5 rounded-lg border border-white/10 hover:bg-white/10 transition-colors;
}

.model-info {
  @apply flex-1;
}

.model-name {
  @apply text-white/90 font-medium text-sm;
}

.model-details {
  @apply flex items-center gap-2 mt-1;
}

.model-size {
  @apply text-white/60 text-xs;
}

.model-params {
  @apply text-white/60 text-xs px-1.5 py-0.5 bg-white/10 rounded-md;
}

.model-actions {
  @apply flex items-center gap-2;
}

.select-btn {
  @apply w-6 h-6 rounded-full border border-white/30 text-xs flex items-center justify-center hover:bg-white/10 transition-colors;
}

.select-btn.active {
  @apply bg-green-500/80 border-green-400 text-white;
}

.delete-btn {
  @apply p-1.5 rounded-lg bg-red-500/20 hover:bg-red-500/40 transition-colors text-red-400 hover:text-red-300;
}

.delete-btn:disabled {
  @apply opacity-50 cursor-not-allowed;
}

.no-models {
  @apply text-center p-4 bg-white/5 rounded-lg border border-white/10;
}

.pull-model-section {
  @apply mt-4 pt-4 border-t border-white/10;
}

.popular-models {
  @apply grid grid-cols-2 gap-2;
}

.model-pull-btn {
  @apply flex items-center gap-2 p-2 bg-blue-500/20 hover:bg-blue-500/40 rounded-lg border border-blue-400/30 text-blue-300 hover:text-blue-200 transition-colors text-sm;
}

.model-pull-btn:disabled {
  @apply opacity-50 cursor-not-allowed;
}

.model-pull-btn.recommended {
  @apply bg-green-500/30 border-green-400/50 text-green-200;
}

.recommended-badge {
  @apply text-xs bg-green-400/80 text-green-900 px-1 py-0.5 rounded-md font-medium;
}

.vision-badge {
  @apply text-xs bg-purple-400/80 text-purple-900 px-1 py-0.5 rounded-md font-medium;
}

.research-badge {
  @apply text-xs bg-blue-400/80 text-blue-900 px-1 py-0.5 rounded-md font-medium;
}

.model-pull-btn.vision-model {
  @apply bg-purple-500/30 border-purple-400/50 text-purple-200;
}

.model-pull-btn.research-model {
  @apply bg-blue-500/30 border-blue-400/50 text-blue-200;
}

/* AI Models Panel Transitions */
.ai-models-panel-enter-active,
.ai-models-panel-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.ai-models-panel-enter-from {
  opacity: 0;
  transform: translateY(-10px) scale(0.95);
}

.ai-models-panel-leave-to {
  opacity: 0;
  transform: translateY(-10px) scale(0.95);
}
</style> 