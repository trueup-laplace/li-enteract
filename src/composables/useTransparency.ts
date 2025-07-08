import { ref, computed, watch, onMounted, onUnmounted, readonly } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Window } from '@tauri-apps/api/window'

export interface TransparencyState {
  level: number
  isTransparent: boolean
  isClickThrough: boolean
  isVisible: boolean
}

export function useTransparency() {
  // Reactive state
  const transparencyLevel = ref<number>(1.0)  // 0.0 = invisible, 1.0 = solid
  const isEnabled = ref<boolean>(false)
  const isLoading = ref<boolean>(false)
  const lastError = ref<string | null>(null)
  
  // Computed properties
  const isTransparent = computed(() => transparencyLevel.value < 0.95)
  const isClickThrough = computed(() => transparencyLevel.value < 0.1)
  const isVisible = computed(() => transparencyLevel.value > 0.05)
  
  const transparencyState = computed<TransparencyState>(() => ({
    level: transparencyLevel.value,
    isTransparent: isTransparent.value,
    isClickThrough: isClickThrough.value,
    isVisible: isVisible.value
  }))

  // Tauri window reference
  const currentWindow = Window.getCurrent()

  // Apply transparency to OS window
  const applyTransparency = async (level: number): Promise<void> => {
    if (isLoading.value) return
    
    try {
      isLoading.value = true
      lastError.value = null
      
      const clampedLevel = Math.max(0.1, Math.min(1, level)) // Minimum 10% opacity to prevent full invisibility
      console.log(`🔧 TRANSPARENCY: Applying level ${clampedLevel} (original: ${level})`)
      
      await invoke('set_window_transparency', { alpha: clampedLevel })
      
      transparencyLevel.value = clampedLevel
      
      // Save to localStorage for persistence
      localStorage.setItem('transparency_level', clampedLevel.toString())
      localStorage.setItem('transparency_enabled', 'true')
      
      console.log(`✅ TRANSPARENCY: Successfully applied ${clampedLevel}`)
      
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : 'Failed to set transparency'
      console.error('🚨 TRANSPARENCY ERROR:', error)
      
      // Emergency restore on error
      try {
        console.log('🔄 TRANSPARENCY: Emergency restore due to error')
        await invoke('emergency_restore_window')
        transparencyLevel.value = 1.0
      } catch (restoreError) {
        console.error('🚨 EMERGENCY RESTORE FAILED:', restoreError)
      }
    } finally {
      isLoading.value = false
    }
  }

  // Toggle transparency on/off
  const toggle = async (): Promise<void> => {
    try {
      const result = await invoke<number>('toggle_transparency', { 
        currentAlpha: transparencyLevel.value 
      })
      transparencyLevel.value = result
      isEnabled.value = result < 1.0
      
      // Save state
      localStorage.setItem('transparency_level', result.toString())
      localStorage.setItem('transparency_enabled', isEnabled.value.toString())
      
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : 'Failed to toggle transparency'
      console.error('Toggle transparency error:', error)
    }
  }

  // Set specific transparency level
  const setLevel = async (level: number): Promise<void> => {
    await applyTransparency(level)
    isEnabled.value = level < 1.0
  }

  // Preset transparency levels (with safety minimums)
  const presets = {
    invisible: () => setLevel(0.1), // Changed from 0.0 to prevent full invisibility
    ghostMode: () => setLevel(0.3),
    semiTransparent: () => setLevel(0.7),
    solid: () => setLevel(1.0)
  }

  // Emergency restore - always works
  const emergencyRestore = async (): Promise<void> => {
    try {
      await invoke('emergency_restore_window')
      transparencyLevel.value = 1.0
      isEnabled.value = false
      
      // Clear error state
      lastError.value = null
      
      // Update localStorage
      localStorage.setItem('transparency_level', '1.0')
      localStorage.setItem('transparency_enabled', 'false')
      
    } catch (error) {
      console.error('Emergency restore failed:', error)
      // Even if the invoke fails, update local state
      transparencyLevel.value = 1.0
      isEnabled.value = false
    }
  }

  // Keyboard shortcuts handler
  const handleKeyDown = (event: KeyboardEvent) => {
    // Ctrl+T: Toggle transparency
    if (event.ctrlKey && event.key === 't') {
      event.preventDefault()
      toggle()
      return
    }
    
    // Ctrl+H: Ghost mode (30% opacity)
    if (event.ctrlKey && event.key === 'h') {
      event.preventDefault()
      presets.ghostMode()
      return
    }
    
    // Escape: Emergency restore
    if (event.key === 'Escape') {
      event.preventDefault()
      emergencyRestore()
      return
    }
    
    // Ctrl+Shift+T: Invisible mode
    if (event.ctrlKey && event.shiftKey && event.key === 'T') {
      event.preventDefault()
      presets.invisible()
      return
    }
  }

  // Load saved preferences
  const loadPreferences = () => {
    try {
      const savedLevel = localStorage.getItem('transparency_level')
      const savedEnabled = localStorage.getItem('transparency_enabled')
      
      if (savedLevel !== null) {
        const level = parseFloat(savedLevel)
        if (!isNaN(level)) {
          transparencyLevel.value = Math.max(0, Math.min(1, level))
        }
      }
      
      if (savedEnabled !== null) {
        isEnabled.value = savedEnabled === 'true'
      }
      
      // Apply saved transparency if it was enabled
      if (isEnabled.value && transparencyLevel.value < 1.0) {
        // Don't await this to avoid blocking initialization
        applyTransparency(transparencyLevel.value).catch(console.error)
      }
      
    } catch (error) {
      console.error('Failed to load transparency preferences:', error)
    }
  }

  // Save preferences when state changes
  watch([transparencyLevel, isEnabled], () => {
    try {
      localStorage.setItem('transparency_level', transparencyLevel.value.toString())
      localStorage.setItem('transparency_enabled', isEnabled.value.toString())
    } catch (error) {
      console.error('Failed to save transparency preferences:', error)
    }
  })

  // Setup and cleanup
  onMounted(() => {
    // Temporarily disable loading preferences to debug window disappearing issue
    console.log('🔧 TRANSPARENCY DEBUG: Skipping preference loading to debug window disappearing')
    // loadPreferences()
    
    // Setup keyboard shortcuts
    document.addEventListener('keydown', handleKeyDown)
    
    // Setup emergency safety timer (auto-restore after 30 seconds of full invisibility)
    let invisibilityTimer: number | null = null
    
    watch(transparencyLevel, (newLevel) => {
      if (newLevel <= 0.05) {
        // Start timer for emergency restore
        invisibilityTimer = setTimeout(() => {
          console.warn('Auto-restoring visibility after 30 seconds of invisibility')
          emergencyRestore()
        }, 30000)
      } else {
        // Clear timer if window becomes visible
        if (invisibilityTimer) {
          clearTimeout(invisibilityTimer)
          invisibilityTimer = null
        }
      }
    })
  })

  onUnmounted(() => {
    // Remove keyboard listeners
    document.removeEventListener('keydown', handleKeyDown)
  })

  // Utility functions
  const getTransparencyPercentage = () => Math.round(transparencyLevel.value * 100)
  const getVisibilityStatus = () => {
    if (transparencyLevel.value >= 0.95) return 'Solid'
    if (transparencyLevel.value >= 0.7) return 'Semi-transparent'
    if (transparencyLevel.value >= 0.3) return 'Ghost mode'
    if (transparencyLevel.value > 0.05) return 'Nearly invisible'
    return 'Invisible'
  }

  return {
    // State
    transparencyLevel: readonly(transparencyLevel),
    isEnabled: readonly(isEnabled),
    isLoading: readonly(isLoading),
    lastError: readonly(lastError),
    transparencyState,
    
    // Computed
    isTransparent,
    isClickThrough,
    isVisible,
    
    // Actions
    toggle,
    setLevel,
    applyTransparency,
    emergencyRestore,
    
    // Presets
    presets,
    
    // Utilities
    getTransparencyPercentage,
    getVisibilityStatus,
    
    // Manual control
    loadPreferences
  }
} 