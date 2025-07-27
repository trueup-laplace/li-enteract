import { vi } from 'vitest'

export const mockTauriInvoke = vi.fn()

export const createMockTauriStore = () => ({
  speechTranscriptionEnabled: false,
  speechModel: 'small',
  wakeWordEnabled: false,
  selectedModel: null,
  audioSettings: {
    selectedLoopbackDevice: null,
    loopbackEnabled: false,
  },
  initializeSpeechTranscription: vi.fn(),
  updateSpeechModel: vi.fn(),
  toggleSpeechTranscription: vi.fn(),
  toggleWakeWord: vi.fn(),
  setSelectedModel: vi.fn(),
})