<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import {
  ChevronDownIcon,
  CpuChipIcon,
  EyeIcon,
  CodeBracketIcon,
  MagnifyingGlassIcon,
  ChatBubbleLeftIcon,
  CheckIcon
} from '@heroicons/vue/24/outline'
import { useAIModels } from '../../composables/useAIModels'

interface Props {
  selectedModel: string | null
  currentAgent: string
}

interface Emits {
  (e: 'update:selectedModel', value: string): void
  (e: 'update:currentAgent', value: string): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const { ollamaModels, fetchOllamaModels, getModelDisplayName } = useAIModels()

const isOpen = ref(false)
const dropdownRef = ref<HTMLElement>()

// Agent definitions with model preferences
const agents = [
  {
    id: 'enteract',
    name: 'Enteract Agent',
    description: 'General purpose AI assistant',
    icon: ChatBubbleLeftIcon,
    color: 'text-blue-400',
    bgColor: 'bg-blue-500/15',
    borderColor: 'border-blue-500/30',
    preferredModel: 'gemma3:1b-it-qat'
  },
  {
    id: 'coding',
    name: 'Coding Agent',
    description: 'Programming assistance and code review',
    icon: CodeBracketIcon,
    color: 'text-green-400',
    bgColor: 'bg-green-500/15',
    borderColor: 'border-green-500/30',
    preferredModel: 'qwen2.5-coder:1.5b'
  },
  {
    id: 'research',
    name: 'Deep Research',
    description: 'Advanced research with step-by-step thinking',
    icon: MagnifyingGlassIcon,
    color: 'text-purple-400',
    bgColor: 'bg-purple-500/15',
    borderColor: 'border-purple-500/30',
    preferredModel: 'deepseek-r1:1.5b'
  },
  {
    id: 'vision',
    name: 'Vision Agent',
    description: 'Visual content analysis',
    icon: EyeIcon,
    color: 'text-pink-400',
    bgColor: 'bg-pink-500/15',
    borderColor: 'border-pink-500/30',
    preferredModel: 'qwen2.5vl:3b'
  }
]

const currentAgentData = computed(() => {
  return agents.find(agent => agent.id === props.currentAgent) || agents[0]
})

const availableModels = computed(() => {
  return ollamaModels.value.map(model => ({
    name: model.name,
    displayName: getModelDisplayName(model),
    size: model.size
  }))
})

const selectAgent = (agentId: string) => {
  const agent = agents.find(a => a.id === agentId)
  if (agent) {
    emit('update:currentAgent', agentId)
    
    // Auto-select preferred model if available
    const hasPreferredModel = availableModels.value.some(m => 
      m.name.includes(agent.preferredModel.split(':')[0])
    )
    
    if (hasPreferredModel) {
      const preferredModel = availableModels.value.find(m => 
        m.name.includes(agent.preferredModel.split(':')[0])
      )
      if (preferredModel) {
        emit('update:selectedModel', preferredModel.name)
      }
    }
  }
  isOpen.value = false
}

const selectModel = (modelName: string) => {
  emit('update:selectedModel', modelName)
  isOpen.value = false
}

const toggleDropdown = () => {
  isOpen.value = !isOpen.value
  if (isOpen.value && ollamaModels.value.length === 0) {
    fetchOllamaModels()
  }
}

// Close dropdown when clicking outside
const handleClickOutside = (event: Event) => {
  if (dropdownRef.value && !dropdownRef.value.contains(event.target as Node)) {
    isOpen.value = false
  }
}

watch(isOpen, (newValue) => {
  if (newValue) {
    document.addEventListener('click', handleClickOutside)
  } else {
    document.removeEventListener('click', handleClickOutside)
  }
})
</script>

<template>
  <div class="model-selector" ref="dropdownRef">
    <!-- Main Selector Button -->
    <button @click="toggleDropdown" class="selector-button" :class="currentAgentData.bgColor">
      <div class="selector-content">
        <div class="selector-icon">
          <component :is="currentAgentData.icon" class="w-4 h-4" :class="currentAgentData.color" />
        </div>
        <div class="selector-text">
          <div class="agent-name">{{ currentAgentData.name }}</div>
          <div v-if="selectedModel" class="model-name">{{ selectedModel.split(':')[0] }}</div>
        </div>
      </div>
      <ChevronDownIcon class="w-4 h-4 text-white/60 transition-transform duration-200" 
                       :class="{ 'rotate-180': isOpen }" />
    </button>

    <!-- Dropdown Menu -->
    <Transition name="dropdown">
      <div v-if="isOpen" class="dropdown-menu">
        <!-- Agents Section -->
        <div class="dropdown-section">
          <div class="section-header">
            <CpuChipIcon class="w-4 h-4 text-blue-400" />
            <span class="section-title">AI Agents</span>
          </div>
          <div class="agents-grid">
            <button
              v-for="agent in agents"
              :key="agent.id"
              @click="selectAgent(agent.id)"
              class="agent-option"
              :class="[
                agent.bgColor,
                agent.borderColor,
                { 'selected': currentAgent === agent.id }
              ]"
            >
              <div class="agent-option-content">
                <component :is="agent.icon" class="w-4 h-4" :class="agent.color" />
                <div class="agent-details">
                  <div class="agent-option-name">{{ agent.name }}</div>
                  <div class="agent-description">{{ agent.description }}</div>
                </div>
                <CheckIcon v-if="currentAgent === agent.id" class="w-4 h-4 text-green-400" />
              </div>
            </button>
          </div>
        </div>

        <!-- Models Section -->
        <div class="dropdown-section">
          <div class="section-header">
            <CpuChipIcon class="w-4 h-4 text-green-400" />
            <span class="section-title">Available Models</span>
          </div>
          <div class="models-list">
            <button
              v-for="model in availableModels"
              :key="model.name"
              @click="selectModel(model.name)"
              class="model-option"
              :class="{ 'selected': selectedModel === model.name }"
            >
              <div class="model-option-content">
                <div class="model-details">
                  <div class="model-option-name">{{ model.displayName }}</div>
                  <div class="model-size">{{ Math.round(model.size / (1024 * 1024 * 1024) * 10) / 10 }}GB</div>
                </div>
                <CheckIcon v-if="selectedModel === model.name" class="w-4 h-4 text-green-400" />
              </div>
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.model-selector {
  @apply relative;
}

.selector-button {
  @apply flex items-center justify-between w-full px-3 py-2 rounded-xl border transition-all duration-200;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  min-width: 200px;
}

.selector-button:hover {
  background: rgba(255, 255, 255, 0.1);
  border-color: rgba(255, 255, 255, 0.2);
}

.selector-content {
  @apply flex items-center gap-3;
}

.selector-icon {
  @apply flex-shrink-0;
}

.selector-text {
  @apply flex flex-col items-start;
}

.agent-name {
  @apply text-sm font-medium text-white;
}

.model-name {
  @apply text-xs text-white/60;
}

.dropdown-menu {
  @apply absolute top-full left-0 right-0 mt-2 bg-black/95 border border-white/20 rounded-xl shadow-xl backdrop-blur-sm z-50 max-h-96 overflow-y-auto;
}

.dropdown-section {
  @apply p-3;
}

.dropdown-section:not(:last-child) {
  @apply border-b border-white/10;
}

.section-header {
  @apply flex items-center gap-2 mb-3;
}

.section-title {
  @apply text-xs font-medium text-white/80 uppercase tracking-wide;
}

.agents-grid {
  @apply space-y-2;
}

.agent-option {
  @apply w-full p-3 rounded-lg border transition-all duration-200 text-left;
  background: rgba(255, 255, 255, 0.03);
}

.agent-option:hover {
  background: rgba(255, 255, 255, 0.08);
  transform: translateY(-1px);
}

.agent-option.selected {
  @apply ring-2 ring-blue-500/50;
  background: rgba(59, 130, 246, 0.1);
}

.agent-option-content {
  @apply flex items-center gap-3;
}

.agent-details {
  @apply flex-1;
}

.agent-option-name {
  @apply text-sm font-medium text-white;
}

.agent-description {
  @apply text-xs text-white/60 mt-0.5;
}

.models-list {
  @apply space-y-1;
}

.model-option {
  @apply w-full p-2 rounded-lg hover:bg-white/10 transition-all duration-200 text-left;
}

.model-option.selected {
  @apply bg-green-500/20 ring-1 ring-green-500/50;
}

.model-option-content {
  @apply flex items-center justify-between;
}

.model-details {
  @apply flex-1;
}

.model-option-name {
  @apply text-sm text-white;
}

.model-size {
  @apply text-xs text-white/60;
}

/* Dropdown Transition */
.dropdown-enter-active,
.dropdown-leave-active {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.dropdown-enter-from {
  opacity: 0;
  transform: translateY(-10px) scale(0.95);
}

.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-10px) scale(0.95);
}

/* Scrollbar */
.dropdown-menu::-webkit-scrollbar {
  width: 4px;
}

.dropdown-menu::-webkit-scrollbar-track {
  background: transparent;
}

.dropdown-menu::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
}
</style>