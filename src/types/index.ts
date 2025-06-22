export interface ChatMessage {
  id: number
  text: string
  sender: 'user' | 'assistant'
  timestamp: Date
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