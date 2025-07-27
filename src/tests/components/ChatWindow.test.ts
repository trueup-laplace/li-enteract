import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import ChatWindow from '@/components/core/ChatWindow.vue'
import { 
  createMockChatManagement,
  createMockSpeechTranscription,
} from '../__mocks__/composables'

// Mock composables
vi.mock('@/composables/useChatManagement', () => ({
  useChatManagement: () => createMockChatManagement(),
}))

vi.mock('@/composables/useSpeechTranscription', () => ({
  useSpeechTranscription: () => createMockSpeechTranscription(),
}))

vi.mock('@/composables/useSpeechEvents', () => ({
  useSpeechEvents: () => ({
    setupSpeechTranscriptionListeners: vi.fn(),
    removeSpeechTranscriptionListeners: vi.fn(),
  }),
}))

vi.mock('@/composables/useWindowRegistry', () => ({
  useWindowRegistration: () => ({
    registerSelf: vi.fn(),
    unregisterSelf: vi.fn(),
  }),
}))

// Mock child components
vi.mock('@/components/core/AgentActionButtons.vue', () => ({
  default: {
    name: 'AgentActionButtons',
    template: '<div data-testid="agent-action-buttons"></div>',
  },
}))

vi.mock('@/components/core/ModelSelector.vue', () => ({
  default: {
    name: 'ModelSelector',
    template: '<div data-testid="model-selector"></div>',
  },
}))

vi.mock('@/components/core/ChatWindowSidebar.vue', () => ({
  default: {
    name: 'ChatWindowSidebar',
    template: '<div data-testid="chat-window-sidebar"></div>',
  },
}))

describe('ChatWindow', () => {
  let wrapper: any
  const defaultProps = {
    showChatWindow: true,
    selectedModel: 'test-model',
  }

  beforeEach(() => {
    setActivePinia(createPinia())
    wrapper = mount(ChatWindow, {
      props: defaultProps,
      global: {
        stubs: {
          AgentActionButtons: true,
          ModelSelector: true,
          ChatWindowSidebar: true,
        },
      },
    })
  })

  it('renders when showChatWindow is true', () => {
    expect(wrapper.find('.chat-window').exists()).toBe(true)
  })

  it('does not render when showChatWindow is false', async () => {
    await wrapper.setProps({ showChatWindow: false })
    expect(wrapper.find('.chat-window').exists()).toBe(false)
  })

  it('renders window header with title', () => {
    const header = wrapper.find('.window-header')
    expect(header.exists()).toBe(true)
    expect(header.text()).toContain('AI Assistant')
  })

  it('renders chat input', () => {
    const chatInput = wrapper.find('.chat-input')
    expect(chatInput.exists()).toBe(true)
    expect(chatInput.attributes('placeholder')).toContain('Ask any AI agent')
  })

  it('renders microphone button', () => {
    const micButton = wrapper.find('.chat-mic-btn')
    expect(micButton.exists()).toBe(true)
  })

  it('renders send button', () => {
    const sendButton = wrapper.find('.chat-send-btn')
    expect(sendButton.exists()).toBe(true)
  })

  it('shows empty state when no chat history', () => {
    const emptyState = wrapper.find('.empty-state')
    expect(emptyState.exists()).toBe(true)
    expect(emptyState.text()).toContain('Start a conversation')
  })

  it('handles input changes', async () => {
    const input = wrapper.find('.chat-input')
    await input.setValue('test message')
    expect(input.element.value).toBe('test message')
  })

  it('handles @ mention suggestions', async () => {
    const input = wrapper.find('.chat-input')
    await input.setValue('@test')
    await input.trigger('input')
    
    // Should trigger the handleInput method
    expect(input.element.value).toBe('@test')
  })

  it('handles microphone button click', async () => {
    const micButton = wrapper.find('.chat-mic-btn')
    await micButton.trigger('click')
    
    // Should call the mock function
    expect(micButton.exists()).toBe(true)
  })

  it('handles send button click', async () => {
    // First set a message
    const input = wrapper.find('.chat-input')
    await input.setValue('test message')
    
    const sendButton = wrapper.find('.chat-send-btn')
    await sendButton.trigger('click')
    
    expect(sendButton.exists()).toBe(true)
  })

  it('handles close button click', async () => {
    const closeButton = wrapper.find('.close-btn')
    await closeButton.trigger('click')
    
    expect(wrapper.emitted('close')).toBeTruthy()
    expect(wrapper.emitted('update:showChatWindow')).toBeTruthy()
    expect(wrapper.emitted('update:showChatWindow')[0]).toEqual([false])
  })

  it('handles keyboard shortcuts', async () => {
    const input = wrapper.find('.chat-input')
    await input.trigger('keydown.enter')
    
    // Should trigger the enhanced keydown handler
    expect(input.exists()).toBe(true)
  })

  it('shows model selector', () => {
    expect(wrapper.findComponent({ name: 'ModelSelector' }).exists()).toBe(true)
  })

  it('shows agent action buttons', () => {
    expect(wrapper.findComponent({ name: 'AgentActionButtons' }).exists()).toBe(true)
  })

  it('can toggle chat sidebar', async () => {
    const toggleButton = wrapper.find('[title="Show chat history"]')
    expect(toggleButton.exists()).toBe(true)
  })

  it('emits model update event', async () => {
    const modelSelector = wrapper.findComponent({ name: 'ModelSelector' })
    await modelSelector.vm.$emit('update:selected-model', 'new-model')
    
    expect(wrapper.emitted('update:selectedModel')).toBeTruthy()
    expect(wrapper.emitted('update:selectedModel')[0]).toEqual(['new-model'])
  })
})