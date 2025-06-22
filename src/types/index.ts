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