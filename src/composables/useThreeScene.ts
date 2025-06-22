import { ref, onMounted, onUnmounted } from 'vue'
import * as THREE from 'three'
import type { ThreeJSRefs } from '../types'

export function useThreeScene() {
  const threeContainer = ref<HTMLElement>()
  const threeRefs = ref<ThreeJSRefs>({})

  const initThreeJS = () => {
    if (!threeContainer.value) return

    // Scene setup
    const scene = new THREE.Scene()
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000)
    const renderer = new THREE.WebGLRenderer({ 
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
    
    const cube = new THREE.Mesh(geometry, material)
    scene.add(cube)

    // Add ambient light
    const ambientLight = new THREE.AmbientLight(0x404040, 0.6)
    scene.add(ambientLight)

    // Add directional light
    const directionalLight = new THREE.DirectionalLight(0xffffff, 0.8)
    directionalLight.position.set(1, 1, 1)
    scene.add(directionalLight)

    camera.position.z = 5

    // Store refs
    threeRefs.value = { scene, camera, renderer, cube }

    animate()
  }

  const animate = () => {
    if (!threeRefs.value.scene || !threeRefs.value.camera || !threeRefs.value.renderer) return
    
    threeRefs.value.animationId = requestAnimationFrame(animate)
    
    if (threeRefs.value.cube) {
      threeRefs.value.cube.rotation.x += 0.01
      threeRefs.value.cube.rotation.y += 0.01
    }
    
    threeRefs.value.renderer.render(threeRefs.value.scene, threeRefs.value.camera)
  }

  const handleResize = () => {
    if (!threeRefs.value.camera || !threeRefs.value.renderer) return
    
    threeRefs.value.camera.aspect = window.innerWidth / window.innerHeight
    threeRefs.value.camera.updateProjectionMatrix()
    threeRefs.value.renderer.setSize(window.innerWidth, window.innerHeight)
  }

  const cleanup = () => {
    if (threeRefs.value.animationId) {
      cancelAnimationFrame(threeRefs.value.animationId)
    }
    
    if (threeRefs.value.renderer && threeContainer.value) {
      threeContainer.value.removeChild(threeRefs.value.renderer.domElement)
      threeRefs.value.renderer.dispose()
    }
    
    if (threeRefs.value.cube) {
      threeRefs.value.cube.geometry.dispose()
      if (Array.isArray(threeRefs.value.cube.material)) {
        threeRefs.value.cube.material.forEach(material => material.dispose())
      } else {
        threeRefs.value.cube.material.dispose()
      }
    }
  }

  onMounted(() => {
    initThreeJS()
    window.addEventListener('resize', handleResize)
  })

  onUnmounted(() => {
    cleanup()
    window.removeEventListener('resize', handleResize)
  })

  return {
    threeContainer,
    threeRefs,
    initThreeJS,
    cleanup
  }
} 