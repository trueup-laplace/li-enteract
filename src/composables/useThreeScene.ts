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

    // Create glass-like cube geometry
    const geometry = new THREE.BoxGeometry(1.5, 1.5, 1.5)
    
    // Create multiple materials for glass effect
    const glassMaterial = new THREE.MeshPhysicalMaterial({
      color: 0x4f46e5,
      transparent: true,
      opacity: 0.15,
      roughness: 0.1,
      metalness: 0.1,
      clearcoat: 1.0,
      clearcoatRoughness: 0.1,
      transmission: 0.8,
      ior: 1.5,
    })

    // Create wireframe material
    const wireframeMaterial = new THREE.MeshBasicMaterial({
      color: 0x8b5cf6,
      wireframe: true,
      transparent: true,
      opacity: 0.3,
    })

    // Create the main glass cube
    const glassCube = new THREE.Mesh(geometry, glassMaterial)
    
    // Create wireframe overlay
    const wireframeCube = new THREE.Mesh(geometry, wireframeMaterial)
    
    // Group them together
    const cubeGroup = new THREE.Group()
    cubeGroup.add(glassCube)
    cubeGroup.add(wireframeCube)
    
    scene.add(cubeGroup)

    // Add ambient light for glass effect
    const ambientLight = new THREE.AmbientLight(0x4f46e5, 0.4)
    scene.add(ambientLight)

    // Add directional light for glass reflections
    const directionalLight = new THREE.DirectionalLight(0x8b5cf6, 1.0)
    directionalLight.position.set(2, 2, 2)
    scene.add(directionalLight)

    // Add point light for internal glow
    const pointLight = new THREE.PointLight(0x06b6d4, 0.8, 10)
    pointLight.position.set(0, 0, 2)
    scene.add(pointLight)

    // Add rim light
    const rimLight = new THREE.DirectionalLight(0xffffff, 0.5)
    rimLight.position.set(-2, -2, -2)
    scene.add(rimLight)

    camera.position.z = 4

    // Store refs
    threeRefs.value = { scene, camera, renderer, cube: cubeGroup }

    animate()
  }

  const animate = () => {
    if (!threeRefs.value.scene || !threeRefs.value.camera || !threeRefs.value.renderer) return
    
    threeRefs.value.animationId = requestAnimationFrame(animate)
    
    if (threeRefs.value.cube) {
      // Smooth, organic rotation
      threeRefs.value.cube.rotation.x += 0.005
      threeRefs.value.cube.rotation.y += 0.008
      threeRefs.value.cube.rotation.z += 0.003
      
      // Subtle floating motion
      threeRefs.value.cube.position.y = Math.sin(Date.now() * 0.001) * 0.1
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
      // Clean up geometry and materials
      threeRefs.value.cube.traverse((child) => {
        if (child instanceof THREE.Mesh) {
          child.geometry.dispose()
          if (Array.isArray(child.material)) {
            child.material.forEach(material => material.dispose())
          } else {
            child.material.dispose()
          }
        }
      })
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