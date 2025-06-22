<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import * as THREE from 'three'
import { 
  MicrophoneIcon, 
  EyeIcon, 
  EyeSlashIcon, 
  ChatBubbleLeftRightIcon,
  PaperAirplaneIcon,
  XMarkIcon
} from '@heroicons/vue/24/outline'

// Reactive state
const micEnabled = ref(false)
const eyeTrackingEnabled = ref(false)
const chatOpen = ref(false)
const chatMessages = ref([
  { id: 1, text: "Welcome to your agentic assistant", sender: "assistant", timestamp: new Date() },
  { id: 2, text: "How can I help you today?", sender: "assistant", timestamp: new Date() }
])
const newMessage = ref('')

// Three.js refs
let scene: THREE.Scene
let camera: THREE.PerspectiveCamera
let renderer: THREE.WebGLRenderer
let cube: THREE.Mesh
let animationId: number

const threeContainer = ref<HTMLElement>()

const initThreeJS = () => {
  if (!threeContainer.value) return

  // Scene setup
  scene = new THREE.Scene()
  camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000)
  renderer = new THREE.WebGLRenderer({ 
    alpha: true, 
    antialias: true,
    powerPreference: "high-performance"
  })
  
  renderer.setSize(window.innerWidth, window.innerHeight)
  renderer.setClearColor(0x000000, 0)
  threeContainer.value.appendChild(renderer.domElement)

  // Create cube with glassmorphic material
  const geometry = new THREE.BoxGeometry(2, 2, 2)
  const material = new THREE.MeshBasicMaterial({ 
    color: 0x3b82f6,
    transparent: true,
    opacity: 0.6,
    wireframe: true
  })
  
  cube = new THREE.Mesh(geometry, material)
  scene.add(cube)

  // Add ambient light
  const ambientLight = new THREE.AmbientLight(0x404040, 0.6)
  scene.add(ambientLight)

  // Add directional light
  const directionalLight = new THREE.DirectionalLight(0xffffff, 0.8)
  directionalLight.position.set(1, 1, 1)
  scene.add(directionalLight)

  camera.position.z = 5

  animate()
}

const animate = () => {
  animationId = requestAnimationFrame(animate)
  
  if (cube) {
    cube.rotation.x += 0.01
    cube.rotation.y += 0.01
  }
  
  renderer.render(scene, camera)
}

const handleResize = () => {
  if (!camera || !renderer) return
  
  camera.aspect = window.innerWidth / window.innerHeight
  camera.updateProjectionMatrix()
  renderer.setSize(window.innerWidth, window.innerHeight)
}

const toggleMic = () => {
  micEnabled.value = !micEnabled.value
}

const toggleEyeTracking = () => {
  eyeTrackingEnabled.value = !eyeTrackingEnabled.value
}

const toggleChat = () => {
  chatOpen.value = !chatOpen.value
}

const sendMessage = () => {
  if (!newMessage.value.trim()) return
  
  chatMessages.value.push({
    id: Date.now(),
    text: newMessage.value,
    sender: "user",
    timestamp: new Date()
  })
  
  // Simulate assistant response
  setTimeout(() => {
    chatMessages.value.push({
      id: Date.now(),
      text: "I understand. Let me help you with that.",
      sender: "assistant", 
      timestamp: new Date()
    })
  }, 1000)
  
  newMessage.value = ''
}

onMounted(() => {
  initThreeJS()
  window.addEventListener('resize', handleResize)
})

onUnmounted(() => {
  if (animationId) {
    cancelAnimationFrame(animationId)
  }
  window.removeEventListener('resize', handleResize)
})
</script>

<template>
  <div class="min-h-screen bg-gradient-to-br from-gray-900 via-blue-900 to-purple-900 relative overflow-hidden" data-theme="dark">
    <!-- Three.js Container -->
    <div ref="threeContainer" class="absolute inset-0 z-0"></div>
    
    <!-- Background overlay for better contrast -->
    <div class="absolute inset-0 bg-black/20 z-10"></div>
    
    <!-- Main Content -->
    <div class="relative z-20 min-h-screen flex flex-col">
      <!-- Top section with breathing room -->
      <div class="flex-1"></div>
      
      <!-- Bottom Control Panel -->
      <div class="p-6">
        <div class="flex justify-center">
          <div class="glass-panel flex items-center gap-4 px-8 py-4">
            <!-- Microphone Button -->
            <button 
              @click="toggleMic"
              class="btn btn-circle btn-lg glass-btn"
              :class="{ 'btn-primary': micEnabled, 'btn-ghost': !micEnabled }"
            >
              <MicrophoneIcon class="w-6 h-6" />
            </button>
            
            <!-- Eye Tracking Button -->
            <button 
              @click="toggleEyeTracking"
              class="btn btn-circle btn-lg glass-btn"
              :class="{ 'btn-secondary': eyeTrackingEnabled, 'btn-ghost': !eyeTrackingEnabled }"
            >
              <EyeIcon v-if="eyeTrackingEnabled" class="w-6 h-6" />
              <EyeSlashIcon v-else class="w-6 h-6" />
            </button>
            
            <!-- Chat Button -->
            <button 
              @click="toggleChat"
              class="btn btn-circle btn-lg glass-btn"
              :class="{ 'btn-accent': chatOpen, 'btn-ghost': !chatOpen }"
            >
              <ChatBubbleLeftRightIcon class="w-6 h-6" />
            </button>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Chat Drawer -->
    <div 
      class="fixed top-0 right-0 h-full w-96 z-50 transform transition-transform duration-300 ease-in-out"
      :class="chatOpen ? 'translate-x-0' : 'translate-x-full'"
    >
      <div class="h-full glass-panel-vertical flex flex-col">
        <!-- Chat Header -->
        <div class="flex items-center justify-between p-4 border-b border-glass">
          <h3 class="text-lg font-semibold text-white">Chat History</h3>
          <button @click="toggleChat" class="btn btn-sm btn-circle btn-ghost">
            <XMarkIcon class="w-5 h-5" />
          </button>
        </div>
        
        <!-- Chat Messages -->
        <div class="flex-1 overflow-y-auto p-4 space-y-4">
          <div 
            v-for="message in chatMessages" 
            :key="message.id"
            class="flex"
            :class="message.sender === 'user' ? 'justify-end' : 'justify-start'"
          >
            <div 
              class="max-w-xs rounded-lg px-4 py-2 text-sm"
              :class="message.sender === 'user' 
                ? 'bg-primary text-primary-content' 
                : 'bg-glass-dark text-white border border-glass'"
            >
              {{ message.text }}
            </div>
          </div>
        </div>
        
        <!-- Chat Input -->
        <div class="p-4 border-t border-glass">
          <div class="flex gap-2">
            <input 
              v-model="newMessage"
              @keyup.enter="sendMessage"
              type="text" 
              placeholder="Type a message..."
              class="input input-bordered flex-1 bg-glass-dark border-glass text-white placeholder-gray-400"
            />
            <button 
              @click="sendMessage"
              class="btn btn-primary btn-circle"
            >
              <PaperAirplaneIcon class="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Backdrop for chat -->
    <div 
      v-if="chatOpen"
      @click="toggleChat"
      class="fixed inset-0 bg-black/30 backdrop-blur-sm z-40"
    ></div>
  </div>
</template>

<style scoped>
.glass-panel {
  @apply bg-glass-dark/30 backdrop-blur-lg border border-glass rounded-2xl shadow-2xl;
}

.glass-panel-vertical {
  @apply bg-glass-dark/40 backdrop-blur-xl border-l border-glass shadow-2xl;
}

.glass-btn {
  @apply backdrop-blur-sm border border-glass/50 hover:border-glass transition-all duration-200;
}

.glass-btn:hover {
  @apply scale-105 shadow-lg;
}
</style>
