<script setup lang="ts">
import { computed } from 'vue'
import { 
  XMarkIcon, 
  DocumentArrowDownIcon, 
  CheckIcon 
} from '@heroicons/vue/24/outline'

interface Props {
  show: boolean
  selectedCount: number
  hasSelection: boolean
}

interface Emits {
  (e: 'close'): void
  (e: 'export'): void
  (e: 'select-all'): void
  (e: 'deselect-all'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const selectionText = computed(() => {
  return `${props.selectedCount} message${props.selectedCount !== 1 ? 's' : ''} selected`
})

const handleClose = () => emit('close')
const handleExport = () => emit('export')
const handleSelectAll = () => emit('select-all')
const handleDeselectAll = () => emit('deselect-all')
</script>

<template>
  <div v-if="show" class="export-controls">
    <div class="export-info">
      <span class="text-xs text-white/60">{{ selectionText }}</span>
    </div>
    <div class="export-actions">
      <button 
        @click="handleSelectAll" 
        class="export-action-btn"
      >
        <CheckIcon class="w-3 h-3" />
        Select All
      </button>
      <button 
        @click="handleDeselectAll" 
        class="export-action-btn"
        :class="{ 'disabled': !hasSelection }"
        :disabled="!hasSelection"
      >
        <XMarkIcon class="w-3 h-3" />
        Deselect All
      </button>
      <button 
        @click="handleExport" 
        class="export-action-btn primary"
        :disabled="!hasSelection"
      >
        <DocumentArrowDownIcon class="w-3 h-3" />
        Export Selected
      </button>
      <button 
        @click="handleClose" 
        class="export-action-btn"
      >
        Cancel
      </button>
    </div>
  </div>
</template>

<style scoped>
.export-controls {
  @apply flex items-center justify-between px-4 py-2 border-b border-white/10 bg-white/5;
  flex-shrink: 0;
}

.export-info {
  @apply flex items-center;
}

.export-actions {
  @apply flex items-center gap-2;
}

.export-action-btn {
  @apply flex items-center gap-1 px-2 py-1 rounded-lg text-white/70 hover:text-white hover:bg-white/10 transition-all duration-200 text-xs;
}

.export-action-btn.primary {
  @apply bg-blue-500/80 text-white hover:bg-blue-500 disabled:opacity-50 disabled:cursor-not-allowed;
}

.export-action-btn.disabled {
  @apply opacity-50 cursor-not-allowed hover:bg-transparent hover:text-white/70;
}

.export-action-btn:disabled {
  @apply opacity-50 cursor-not-allowed hover:bg-transparent hover:text-white/70;
}
</style>