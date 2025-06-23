export interface ChatMessage {
  id: number
  text: string
  sender: 'user' | 'assistant' | 'transcription'
  timestamp: Date
  isInterim?: boolean
  confidence?: number
  source?: 'web-speech' | 'whisper' | 'typed'
}

export interface AppState {
  micEnabled: boolean
  chatOpen: boolean
  windowCollapsed: boolean
  isRecording: boolean
}

export interface WindowPosition {
  x: number
  y: number
}

export interface TransparencyState {
  level: number
  isTransparent: boolean
  isClickThrough: boolean
  isVisible: boolean
}

export interface TransparencyPresets {
  invisible: () => Promise<void>
  ghostMode: () => Promise<void>
  semiTransparent: () => Promise<void>
  solid: () => Promise<void>
}

// Re-export speech transcription types
export * from './speechTranscription' 