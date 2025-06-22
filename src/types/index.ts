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

import * as THREE from 'three'

export interface ThreeJSRefs {
  scene?: THREE.Scene
  camera?: THREE.PerspectiveCamera
  renderer?: THREE.WebGLRenderer
  cube?: THREE.Mesh | THREE.Group
  animationId?: number
} 