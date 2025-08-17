<script setup lang="ts">
import {
  CloudArrowUpIcon,
  PhotoIcon
} from '@heroicons/vue/24/outline'

interface Props {
  fileInput: HTMLInputElement | undefined
  isUploading?: boolean
}

interface Emits {
  (e: 'takeScreenshot'): void
  (e: 'startDeepResearch'): void
  (e: 'startConversational'): void
  (e: 'startCoding'): void
  (e: 'startComputerUse'): void
  (e: 'triggerFileUpload'): void
  (e: 'handleFileUpload', event: Event): void
}

const props = withDefaults(defineProps<Props>(), {
  isUploading: false
})
defineEmits<Emits>()
</script>

<template>
  <!-- Quick Action Tools -->
  <div class="quick-actions">
    <div class="actions-container">
      <div class="inline-buttons">
        <!-- Upload Documents Button (replaces Upload Files) -->
        <button 
          @click="$emit('triggerFileUpload')" 
          :disabled="isUploading"
          class="tool-btn upload-docs" 
          :title="isUploading ? 'Uploading documents...' : 'Upload Documents - Add documents for RAG context'"
        >
          <CloudArrowUpIcon class="w-4 h-4" />
          <span class="tool-label">{{ isUploading ? 'Uploading...' : 'Upload Docs' }}</span>
        </button>
        
        <button @click="$emit('takeScreenshot')" class="tool-btn take-screenshot" title="Take Screenshot - Capture and analyze screen content">
          <PhotoIcon class="w-4 h-4" />
          <span class="tool-label">Screenshot</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.quick-actions {
  @apply px-4 py-3 border-t border-white/10;
  background: rgba(0, 0, 0, 0.1);
}

.actions-container {
  @apply w-full;
}

.inline-buttons {
  @apply flex items-center justify-end gap-3 flex-wrap;
}

.tool-btn {
  @apply flex items-center gap-2 px-4 py-2 rounded-xl transition-all duration-200 border;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.8);
  font-size: 13px;
  font-weight: 500;
}

.tool-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.1);
  border-color: rgba(255, 255, 255, 0.2);
  color: white;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

.tool-btn.upload-docs {
  background: rgba(34, 197, 94, 0.1);
  border-color: rgba(34, 197, 94, 0.2);
  color: rgb(134, 239, 172);
}

.tool-btn.upload-docs:hover:not(:disabled) {
  background: rgba(34, 197, 94, 0.2);
  border-color: rgba(34, 197, 94, 0.4);
  color: rgb(187, 247, 208);
}

.tool-btn.upload-docs:disabled {
  background: rgba(75, 85, 99, 0.1);
  border-color: rgba(75, 85, 99, 0.2);
  color: rgba(255, 255, 255, 0.3);
  cursor: not-allowed;
}

.tool-btn.take-screenshot {
  background: rgba(168, 85, 247, 0.1);
  border-color: rgba(168, 85, 247, 0.2);
  color: rgb(196, 181, 253);
}

.tool-btn.take-screenshot:hover {
  background: rgba(168, 85, 247, 0.2);
  border-color: rgba(168, 85, 247, 0.4);
  color: rgb(221, 214, 254);
}

.tool-label {
  @apply text-xs;
}

.hidden {
  display: none !important;
}
</style>