<script setup lang="ts">
import {
  CameraIcon,
  MagnifyingGlassIcon,
  ChatBubbleLeftIcon,
  CodeBracketIcon,
  ComputerDesktopIcon,
  DocumentIcon,
  PhotoIcon
} from '@heroicons/vue/24/outline'

interface Props {
  fileInput: HTMLInputElement | undefined
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

defineProps<Props>()
defineEmits<Emits>()
</script>

<template>
  <!-- Agent Action Buttons - All Inline -->
  <div class="agent-actions">
    <div class="actions-container">
      <!-- All buttons in one inline row with separators -->
      <div class="inline-buttons">
        <!-- AI Agents -->
        <button @click="$emit('takeScreenshot')" class="agent-btn-icon vision-btn" title="Screen Analysis Agent - Analyze screenshots and visual content">
          <CameraIcon class="w-4 h-4" />
        </button>
        
        <button @click="$emit('startDeepResearch')" class="agent-btn-icon research-btn" title="Deep Reasoning Agent - Advanced research with step-by-step thinking">
          <MagnifyingGlassIcon class="w-4 h-4" />
        </button>
        
        <button @click="$emit('startConversational')" class="agent-btn-icon conversation-btn" title="Conversational Agent - General chat and conversation">
          <ChatBubbleLeftIcon class="w-4 h-4" />
        </button>
        
        <button @click="$emit('startCoding')" class="agent-btn-icon coding-btn" title="Coding Agent - Programming assistance and code review">
          <CodeBracketIcon class="w-4 h-4" />
        </button>
        
        <button @click="$emit('startComputerUse')" class="agent-btn-icon computer-btn" title="Computer Use Agent (Experimental) - Screen interaction and automation">
          <ComputerDesktopIcon class="w-4 h-4" />
          <div class="experimental-badge">β</div>
        </button>
        
        <!-- Separator -->
        <div class="button-separator"></div>
        
        <!-- Tools -->
        <input 
          type="file" 
          ref="fileInput" 
          @change="$emit('handleFileUpload', $event)"
          multiple
          accept="image/*,.pdf,.txt,.md,.doc,.docx"
          class="hidden"
        />
        <button @click="$emit('triggerFileUpload')" class="tool-btn-icon upload-tool" title="Upload Files - Add documents, images, and other files">
          <DocumentIcon class="w-4 h-4" />
        </button>
        
        <button @click="$emit('takeScreenshot')" class="tool-btn-icon screenshot-tool" title="Quick Screenshot - Capture screen instantly">
          <PhotoIcon class="w-4 h-4" />
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.agent-actions {
  @apply px-4 py-3 border-t border-white/10;
}

.actions-container {
  @apply w-full;
}

.inline-buttons {
  @apply flex items-center justify-center gap-2 flex-wrap;
}

.button-separator {
  @apply w-px h-6 bg-white/20 mx-1;
}

.agent-btn-icon, .tool-btn-icon {
  @apply rounded-full transition-all duration-200 flex items-center justify-center;
  @apply relative;
  width: 32px;
  height: 32px;
  border: none;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(8px);
  cursor: pointer;
  color: rgba(255, 255, 255, 0.8);
}

.agent-btn-icon:hover, .tool-btn-icon:hover {
  background: rgba(255, 255, 255, 0.2);
  color: white;
  transform: scale(1.1);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

.vision-btn {
  background: rgba(168, 85, 247, 0.15);
  border: 1px solid rgba(168, 85, 247, 0.3);
  color: rgb(196, 181, 253);
}

.vision-btn:hover {
  background: rgba(168, 85, 247, 0.25);
  border-color: rgba(168, 85, 247, 0.5);
  color: rgb(221, 214, 254);
}

.research-btn {
  background: rgba(59, 130, 246, 0.15);
  border: 1px solid rgba(59, 130, 246, 0.3);
  color: rgb(147, 197, 253);
}

.research-btn:hover {
  background: rgba(59, 130, 246, 0.25);
  border-color: rgba(59, 130, 246, 0.5);
  color: rgb(191, 219, 254);
}

.conversation-btn {
  background: rgba(168, 85, 247, 0.15);
  border: 1px solid rgba(168, 85, 247, 0.3);
  color: rgb(196, 181, 253);
}

.conversation-btn:hover {
  background: rgba(168, 85, 247, 0.25);
  border-color: rgba(168, 85, 247, 0.5);
  color: rgb(221, 214, 254);
}

.coding-btn {
  background: rgba(59, 130, 246, 0.15);
  border: 1px solid rgba(59, 130, 246, 0.3);
  color: rgb(147, 197, 253);
}

.coding-btn:hover {
  background: rgba(59, 130, 246, 0.25);
  border-color: rgba(59, 130, 246, 0.5);
  color: rgb(191, 219, 254);
}

.computer-btn {
  background: rgba(34, 197, 94, 0.15);
  border: 1px solid rgba(34, 197, 94, 0.3);
  color: rgb(134, 239, 172);
}

.computer-btn:hover {
  background: rgba(34, 197, 94, 0.25);
  border-color: rgba(34, 197, 94, 0.5);
  color: rgb(187, 247, 208);
}


.upload-tool {
  background: rgba(34, 197, 94, 0.15);
  border: 1px solid rgba(34, 197, 94, 0.3);
  color: rgb(134, 239, 172);
}

.upload-tool:hover {
  background: rgba(34, 197, 94, 0.25);
  border-color: rgba(34, 197, 94, 0.5);
  color: rgb(187, 247, 208);
}

.screenshot-tool {
  background: rgba(168, 85, 247, 0.15);
  border: 1px solid rgba(168, 85, 247, 0.3);
  color: rgb(196, 181, 253);
}

.screenshot-tool:hover {
  background: rgba(168, 85, 247, 0.25);
  border-color: rgba(168, 85, 247, 0.5);
  color: rgb(221, 214, 254);
}

.experimental-badge {
  @apply absolute -top-1 -right-1 text-xs bg-yellow-400 text-yellow-900 px-1 py-0.5 rounded-full font-bold;
  font-size: 8px;
  line-height: 1;
  min-width: 14px;
  height: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.hidden {
  display: none !important;
}
</style> 