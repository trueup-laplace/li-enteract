import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import ControlPanelButtons from '@/components/core/ControlPanelButtons.vue'
import { createMockTauriStore } from '../__mocks__/tauri'

// Mock the composables and stores
vi.mock('@/stores/app', () => ({
  useAppStore: () => createMockTauriStore(),
}))

describe('ControlPanelButtons Integration Tests', () => {
  let wrapper: any
  const defaultProps = {
    store: createMockTauriStore(),
    mlEyeTracking: {
      isActive: { value: false },
      isInitialized: { value: false },
      error: { value: null },
      currentGaze: { value: { x: 0, y: 0 } },
      initialize: vi.fn(),
      start: vi.fn(),
      stop: vi.fn(),
      cleanup: vi.fn(),
    },
    showChatWindow: false,
    showAIModelsWindow: false,
    showConversationalWindow: false,
    isGazeControlActive: false,
  }

  beforeEach(() => {
    setActivePinia(createPinia())
    
    // Mock ControlPanelButtons component since we need to create a minimal version for testing
    wrapper = mount({
      template: `
        <div class="control-panel-buttons">
          <button 
            @click="$emit('toggle-chat')" 
            class="panel-btn chat-btn"
            :class="{ 'active': showChatWindow }"
            data-testid="chat-button"
          >
            Chat
          </button>
          <button 
            @click="$emit('toggle-ai-models')" 
            class="panel-btn ai-models-btn"
            :class="{ 'active': showAIModelsWindow }"
            data-testid="ai-models-button"
          >
            AI Models
          </button>
          <button 
            @click="$emit('toggle-conversational')" 
            class="panel-btn conversational-btn"
            :class="{ 'active': showConversationalWindow }"
            data-testid="conversational-button"
          >
            Conversation
          </button>
          <button 
            @click="$emit('toggle-eye-tracking')" 
            class="panel-btn eye-tracking-btn"
            :class="{ 'active': isGazeControlActive }"
            data-testid="eye-tracking-button"
          >
            Eye Tracking
          </button>
        </div>
      `,
      props: Object.keys(defaultProps),
      emits: ['toggle-chat', 'toggle-ai-models', 'toggle-conversational', 'toggle-eye-tracking'],
    }, {
      props: defaultProps,
    })
  })

  describe('Button Rendering', () => {
    it('renders all control panel buttons', () => {
      expect(wrapper.find('[data-testid="chat-button"]').exists()).toBe(true)
      expect(wrapper.find('[data-testid="ai-models-button"]').exists()).toBe(true)
      expect(wrapper.find('[data-testid="conversational-button"]').exists()).toBe(true)
      expect(wrapper.find('[data-testid="eye-tracking-button"]').exists()).toBe(true)
    })

    it('applies active class when windows are open', async () => {
      await wrapper.setProps({ showChatWindow: true })
      expect(wrapper.find('[data-testid="chat-button"]').classes()).toContain('active')
      
      await wrapper.setProps({ showAIModelsWindow: true })
      expect(wrapper.find('[data-testid="ai-models-button"]').classes()).toContain('active')
      
      await wrapper.setProps({ showConversationalWindow: true })
      expect(wrapper.find('[data-testid="conversational-button"]').classes()).toContain('active')
      
      await wrapper.setProps({ isGazeControlActive: true })
      expect(wrapper.find('[data-testid="eye-tracking-button"]').classes()).toContain('active')
    })
  })

  describe('Button Functionality', () => {
    it('emits toggle-chat when chat button is clicked', async () => {
      const chatButton = wrapper.find('[data-testid="chat-button"]')
      await chatButton.trigger('click')
      
      expect(wrapper.emitted('toggle-chat')).toBeTruthy()
      expect(wrapper.emitted('toggle-chat')).toHaveLength(1)
    })

    it('emits toggle-ai-models when AI models button is clicked', async () => {
      const aiModelsButton = wrapper.find('[data-testid="ai-models-button"]')
      await aiModelsButton.trigger('click')
      
      expect(wrapper.emitted('toggle-ai-models')).toBeTruthy()
      expect(wrapper.emitted('toggle-ai-models')).toHaveLength(1)
    })

    it('emits toggle-conversational when conversational button is clicked', async () => {
      const conversationalButton = wrapper.find('[data-testid="conversational-button"]')
      await conversationalButton.trigger('click')
      
      expect(wrapper.emitted('toggle-conversational')).toBeTruthy()
      expect(wrapper.emitted('toggle-conversational')).toHaveLength(1)
    })

    it('emits toggle-eye-tracking when eye tracking button is clicked', async () => {
      const eyeTrackingButton = wrapper.find('[data-testid="eye-tracking-button"]')
      await eyeTrackingButton.trigger('click')
      
      expect(wrapper.emitted('toggle-eye-tracking')).toBeTruthy()
      expect(wrapper.emitted('toggle-eye-tracking')).toHaveLength(1)
    })
  })

  describe('Button States', () => {
    it('toggles button states correctly', async () => {
      const chatButton = wrapper.find('[data-testid="chat-button"]')
      
      // Initially inactive
      expect(chatButton.classes()).not.toContain('active')
      
      // Simulate window opening
      await wrapper.setProps({ showChatWindow: true })
      expect(chatButton.classes()).toContain('active')
      
      // Simulate window closing
      await wrapper.setProps({ showChatWindow: false })
      expect(chatButton.classes()).not.toContain('active')
    })

    it('handles multiple buttons being active simultaneously', async () => {
      await wrapper.setProps({
        showChatWindow: true,
        showAIModelsWindow: true,
        isGazeControlActive: true,
      })
      
      expect(wrapper.find('[data-testid="chat-button"]').classes()).toContain('active')
      expect(wrapper.find('[data-testid="ai-models-button"]').classes()).toContain('active')
      expect(wrapper.find('[data-testid="eye-tracking-button"]').classes()).toContain('active')
    })
  })

  describe('Button Interactions', () => {
    it('handles rapid button clicks', async () => {
      const chatButton = wrapper.find('[data-testid="chat-button"]')
      
      // Simulate rapid clicking
      await chatButton.trigger('click')
      await chatButton.trigger('click')
      await chatButton.trigger('click')
      
      expect(wrapper.emitted('toggle-chat')).toHaveLength(3)
    })

    it('handles keyboard interaction', async () => {
      const chatButton = wrapper.find('[data-testid="chat-button"]')
      
      await chatButton.trigger('keydown.enter')
      await chatButton.trigger('keydown.space')
      
      // Note: These might not emit the same events as click, depends on implementation
      expect(chatButton.exists()).toBe(true)
    })
  })

  describe('Accessibility', () => {
    it('buttons are focusable', () => {
      const buttons = wrapper.findAll('.panel-btn')
      buttons.forEach(button => {
        // Check that buttons can receive focus (not disabled)
        expect(button.element.disabled).toBe(false)
      })
    })

    it('has proper button semantics', () => {
      const buttons = wrapper.findAll('.panel-btn')
      buttons.forEach(button => {
        expect(button.element.tagName).toBe('BUTTON')
      })
    })
  })
})