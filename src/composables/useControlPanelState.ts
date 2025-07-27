import { ref, computed } from 'vue'
import { getCompatibilityReport } from '../utils/browserCompat'

export function useControlPanelState() {
  // Dragging state
  const isDragging = ref(false)
  const dragStartTime = ref(0)

  // Window state
  const showChatWindow = ref(false)
  const showAIModelsWindow = ref(false)
  const showConversationalWindow = ref(false)

  // Error handling state
  const speechError = ref<string | null>(null)

  // Browser compatibility
  const compatibilityReport = ref(getCompatibilityReport())

  // ML Eye tracking with window movement state
  const isGazeControlActive = ref(false)

  // Computed drag indicator visibility
  const dragIndicatorVisible = computed(() => isDragging.value)

  return {
    isDragging,
    dragStartTime,
    showChatWindow,
    showAIModelsWindow,
    showConversationalWindow,
    speechError,
    compatibilityReport,
    isGazeControlActive,
    dragIndicatorVisible
  }
}