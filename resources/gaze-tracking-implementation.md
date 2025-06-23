# Eye Tracking Implementation Plan: Gaze-Controlled Window Movement

## 🎯 Core Concept: Eye-Driven Interface

Transform your transparent Tauri application into a **gaze-controlled interface** that follows your eye movements in real-time. The window will intelligently reposition itself based on where you're looking on the screen, creating an intuitive, hands-free interaction paradigm.

### Vision Statement
*"Your application becomes an extension of your vision - appearing where you look, when you need it, without conscious effort."*

## 🧠 Technical Architecture Overview

### System Components Stack
```
┌─────────────────────────────────────────┐
│ Gaze-Controlled Window Movement         │ ← User Experience Layer
├─────────────────────────────────────────┤
│ Eye Tracking Engine (Vue + OpenCV.js)  │ ← Computer Vision Layer
├─────────────────────────────────────────┤
│ Camera Input Processing                 │ ← Hardware Interface Layer
├─────────────────────────────────────────┤
│ Calibration & Prediction System        │ ← Intelligence Layer
├─────────────────────────────────────────┤
│ Window Management (Tauri Commands)     │ ← OS Integration Layer
└─────────────────────────────────────────┘
```

## 📊 Data Flow Architecture

### 1. **Input Pipeline**
```
Camera Frame → Face Detection → Eye Detection → Pupil Tracking → Gaze Vector → Screen Coordinate → Window Position
```

### 2. **Processing Frequency**
- **Camera Capture**: 30-60 FPS
- **Computer Vision**: 15-30 FPS (optimized)
- **Gaze Calculation**: 15-30 FPS
- **Window Movement**: 10-15 FPS (smooth movement)
- **Calibration Updates**: 1-5 FPS (background)

## 🔧 Core Implementation Components

### 1. **Camera Access & Management**

#### **Media Stream Handling**
```typescript
interface CameraConfig {
  width: number          // 1280x720 recommended
  height: number
  frameRate: number      // 30 FPS target
  facingMode: 'user'     // Front-facing camera
  deviceId?: string      // Specific camera selection
}

interface CameraState {
  isActive: boolean
  stream: MediaStream | null
  error: string | null
  permissions: 'granted' | 'denied' | 'prompt'
}
```

#### **Camera Lifecycle Management**
```typescript
const cameraLifecycle = {
  // Initialize camera with optimal settings
  async initialize(constraints: CameraConfig)
  
  // Handle camera permissions gracefully
  async requestPermissions()
  
  // Start video stream
  async startStream()
  
  // Stop and cleanup resources
  async stopStream()
  
  // Handle device changes (camera disconnect/reconnect)
  async handleDeviceChange()
  
  // Error recovery mechanisms
  async recoverFromError()
}
```

### 2. **Computer Vision Pipeline**

#### **Face Detection Layer**
```typescript
interface FaceDetection {
  // Primary face detection using Haar Cascades or MediaPipe
  detectFaces(frame: ImageData): FaceBox[]
  
  // Face validation (size, position, confidence)
  validateFace(face: FaceBox): boolean
  
  // Track face movement between frames
  trackFaceMovement(previousFace: FaceBox, currentFace: FaceBox): FaceMovement
}

interface FaceBox {
  x: number, y: number          // Top-left corner
  width: number, height: number // Dimensions
  confidence: number            // Detection confidence (0-1)
  landmarks?: FaceLandmarks     // Optional facial landmarks
}
```

#### **Eye Detection & Tracking**
```typescript
interface EyeTracking {
  // Detect eye regions within face
  detectEyes(faceRegion: ImageData): EyePair
  
  // Extract pupil centers
  findPupils(leftEye: EyeRegion, rightEye: EyeRegion): PupilPair
  
  // Calculate gaze direction
  calculateGazeVector(pupils: PupilPair, faceBox: FaceBox): GazeVector
  
  // Smooth gaze data to reduce noise
  smoothGazeData(gazeHistory: GazeVector[]): GazeVector
}

interface EyePair {
  left: EyeRegion
  right: EyeRegion
  isValid: boolean
}

interface EyeRegion {
  boundingBox: Rectangle
  pupilCenter: Point2D
  confidence: number
  isOpen: boolean           // Blink detection
}

interface GazeVector {
  x: number                 // Horizontal gaze direction (-1 to 1)
  y: number                 // Vertical gaze direction (-1 to 1)
  confidence: number        // Tracking confidence
  timestamp: number         // Frame timestamp
}
```

### 3. **Calibration System**

#### **Initial Calibration Process**
```typescript
interface CalibrationSystem {
  // Multi-point calibration sequence
  async performCalibration(): Promise<CalibrationResult>
  
  // Show calibration targets on screen
  showCalibrationPoint(screenPosition: Point2D): void
  
  // Collect gaze data for each target
  collectCalibrationData(targetPosition: Point2D): CalibrationSample[]
  
  // Calculate transformation matrix
  calculateGazeMapping(samples: CalibrationSample[]): TransformationMatrix
  
  // Validate calibration accuracy
  validateCalibration(): CalibrationAccuracy
}

interface CalibrationSequence {
  points: Point2D[]         // 9-point or 16-point grid
  duration: number          // Time per point (2-3 seconds)
  repetitions: number       // Samples per point (3-5)
  validation: boolean       // Post-calibration validation
}

interface CalibrationResult {
  accuracy: number          // Overall accuracy percentage
  transformMatrix: number[] // Gaze-to-screen mapping
  userProfile: UserProfile  // Individual user characteristics
  isValid: boolean          // Calibration success status
}
```

#### **Continuous Calibration Adjustment**
```typescript
interface AdaptiveCalibration {
  // Background calibration refinement
  async refineCalibration(gazeData: GazeVector[], actualLookPoint: Point2D): void
  
  // Detect calibration drift
  detectDrift(recentAccuracy: number[]): boolean
  
  // Auto-recalibration triggers
  shouldRecalibrate(): boolean
  
  // Personalization learning
  learnUserBehavior(interactionHistory: UserInteraction[]): void
}
```

### 4. **Gaze-to-Screen Mapping**

#### **Coordinate Transformation**
```typescript
interface GazeMapping {
  // Convert gaze vector to screen coordinates
  gazeToScreen(gaze: GazeVector, calibration: CalibrationResult): Point2D
  
  // Handle multi-monitor setups
  determineTargetMonitor(screenPosition: Point2D): MonitorInfo
  
  // Account for head movement
  compensateHeadMovement(gaze: GazeVector, headPosition: HeadPose): GazeVector
  
  // Apply smoothing filters
  applyTemporalSmoothing(rawGaze: Point2D[]): Point2D
}

interface MonitorInfo {
  id: string
  bounds: Rectangle         // Monitor boundaries
  scaleFactor: number       // DPI scaling
  isPrimary: boolean        // Primary display
}
```

#### **Prediction & Smoothing**
```typescript
interface GazePrediction {
  // Predict where user will look next
  predictNextGaze(gazeHistory: GazeVector[]): GazeVector
  
  // Smooth erratic movements
  applySpatialSmoothing(gazePoints: Point2D[]): Point2D
  
  // Filter out noise and jitter
  removeOutliers(gazeData: GazeVector[]): GazeVector[]
  
  // Compensate for processing delays
  compensateLatency(gaze: GazeVector, processingDelay: number): GazeVector
}
```

### 5. **Window Movement Controller**

#### **Movement Strategy**
```typescript
interface WindowMovement {
  // Calculate optimal window position based on gaze
  calculateTargetPosition(gazePoint: Point2D, currentWindow: WindowInfo): Point2D
  
  // Apply movement constraints and boundaries
  constrainMovement(targetPosition: Point2D, constraints: MovementConstraints): Point2D
  
  // Execute smooth window movement
  async moveWindow(targetPosition: Point2D, animation: AnimationConfig): void
  
  // Handle collision detection with screen edges/other windows
  handleCollisions(targetPosition: Point2D): Point2D
}

interface MovementConstraints {
  screenBounds: Rectangle   // Keep window on screen
  minDistance: number       // Minimum movement threshold
  maxSpeed: number          // Maximum movement speed
  deadZones: Rectangle[]    // Areas to avoid (taskbar, etc.)
}

interface AnimationConfig {
  duration: number          // Animation time (100-300ms)
  easing: EasingFunction    // Smooth movement curve
  anticipation: boolean     // Start moving before gaze settles
}
```

## 🎨 Frontend Implementation Strategy

### 1. **Vue Composables Architecture**

#### **useEyeTracking.ts**
```typescript
export function useEyeTracking() {
  const isActive = ref(false)
  const gazePosition = ref<Point2D>({ x: 0, y: 0 })
  const confidence = ref(0)
  const isCalibrated = ref(false)
  
  // Core functionality
  const startTracking = async () => { /* ... */ }
  const stopTracking = async () => { /* ... */ }
  const calibrate = async () => { /* ... */ }
  
  // Real-time data
  const currentGaze = computed(() => gazePosition.value)
  const trackingQuality = computed(() => confidence.value)
  
  return {
    isActive,
    gazePosition,
    confidence,
    isCalibrated,
    startTracking,
    stopTracking,
    calibrate,
    currentGaze,
    trackingQuality
  }
}
```

#### **useGazeWindowControl.ts**
```typescript
export function useGazeWindowControl() {
  const { currentGaze, isActive } = useEyeTracking()
  const windowPosition = ref<Point2D>({ x: 0, y: 0 })
  
  // Window movement logic
  const followGaze = (gaze: Point2D) => { /* ... */ }
  const constrainPosition = (pos: Point2D) => { /* ... */ }
  const animateToPosition = (target: Point2D) => { /* ... */ }
  
  // Watch gaze changes and move window
  watch(currentGaze, (newGaze) => {
    if (isActive.value) {
      followGaze(newGaze)
    }
  }, { immediate: true })
  
  return {
    windowPosition,
    followGaze,
    enableGazeControl: () => { /* ... */ },
    disableGazeControl: () => { /* ... */ }
  }
}
```

### 2. **Computer Vision Worker**

#### **Eye Tracking Worker (Web Worker)**
```typescript
// eyeTrackingWorker.ts - Runs in separate thread
class EyeTrackingWorker {
  private opencv: any
  private faceClassifier: any
  private eyeClassifier: any
  
  async initialize() {
    // Load OpenCV.js
    this.opencv = await loadOpenCV()
    
    // Load Haar Cascade classifiers
    this.faceClassifier = await loadHaarCascade('haarcascade_frontalface_alt.xml')
    this.eyeClassifier = await loadHaarCascade('haarcascade_eye.xml')
  }
  
  processFrame(imageData: ImageData): EyeTrackingResult {
    // Convert to OpenCV Mat
    const mat = this.opencv.matFromImageData(imageData)
    
    // Detect faces
    const faces = this.detectFaces(mat)
    
    if (faces.length > 0) {
      // Process largest/most confident face
      const primaryFace = faces[0]
      
      // Extract eye regions
      const eyes = this.detectEyes(mat, primaryFace)
      
      // Calculate gaze vector
      const gaze = this.calculateGaze(eyes, primaryFace)
      
      return {
        success: true,
        gaze,
        confidence: gaze.confidence,
        faceDetected: true
      }
    }
    
    return {
      success: false,
      gaze: null,
      confidence: 0,
      faceDetected: false
    }
  }
}
```

### 3. **Calibration Interface**

#### **Calibration Component**
```vue
<template>
  <div class="calibration-overlay" v-if="isCalibrating">
    <!-- Full-screen calibration interface -->
    <div class="calibration-background">
      
      <!-- Calibration target -->
      <div 
        class="calibration-target"
        :style="targetStyle"
        @animationend="onTargetComplete"
      >
        <div class="target-center"></div>
        <div class="target-ring"></div>
      </div>
      
      <!-- Progress indicator -->
      <div class="calibration-progress">
        <div class="progress-bar">
          <div 
            class="progress-fill" 
            :style="{ width: `${calibrationProgress}%` }"
          ></div>
        </div>
        <p>Calibration Progress: {{ currentPoint }}/{{ totalPoints }}</p>
        <p>Look at the target and keep your head still</p>
      </div>
      
      <!-- Instructions -->
      <div class="calibration-instructions">
        <h3>Eye Tracking Calibration</h3>
        <ul>
          <li>Sit comfortably in front of your camera</li>
          <li>Keep your head relatively still</li>
          <li>Look directly at each target point</li>
          <li>Wait for the target to disappear before moving</li>
        </ul>
      </div>
      
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useEyeTracking } from '@/composables/useEyeTracking'

const { calibrate } = useEyeTracking()

const isCalibrating = ref(false)
const currentPoint = ref(0)
const totalPoints = ref(9)
const calibrationProgress = computed(() => (currentPoint.value / totalPoints.value) * 100)

// Calibration target positioning
const targetPosition = ref({ x: 50, y: 50 }) // Percentage positions
const targetStyle = computed(() => ({
  left: `${targetPosition.value.x}%`,
  top: `${targetPosition.value.y}%`
}))

const startCalibration = async () => {
  isCalibrating.value = true
  await runCalibrationSequence()
}

const runCalibrationSequence = async () => {
  const points = [
    { x: 10, y: 10 }, { x: 50, y: 10 }, { x: 90, y: 10 },  // Top row
    { x: 10, y: 50 }, { x: 50, y: 50 }, { x: 90, y: 50 },  // Middle row
    { x: 10, y: 90 }, { x: 50, y: 90 }, { x: 90, y: 90 }   // Bottom row
  ]
  
  for (let i = 0; i < points.length; i++) {
    currentPoint.value = i + 1
    targetPosition.value = points[i]
    
    // Wait for user to look at target
    await waitForTargetFocus(points[i])
    
    // Collect gaze data
    await collectCalibrationData(points[i])
  }
  
  // Finalize calibration
  await finalizeCalibration()
  isCalibrating.value = false
}
</script>
```

## 🚨 Edge Cases & Error Handling

### 1. **Hardware & Environmental Issues**

#### **Camera Problems**
```typescript
interface CameraErrorHandling {
  // No camera available
  handleNoCameraFound(): void {
    showFallbackUI("No camera detected")
    disableEyeTracking()
  }
  
  // Camera access denied
  handlePermissionDenied(): void {
    showPermissionDialog()
    provideFallbackControls()
  }
  
  // Camera disconnected during use
  handleCameraDisconnect(): void {
    pauseEyeTracking()
    attemptReconnection()
    showReconnectionStatus()
  }
  
  // Poor camera quality
  handleLowQuality(): void {
    suggestCameraSettings()
    adjustProcessingParameters()
  }
}
```

#### **Lighting Conditions**
```typescript
interface LightingAdaptation {
  // Too dark
  handleLowLight(): void {
    increaseCameraExposure()
    adjustDetectionSensitivity()
    showLightingTips()
  }
  
  // Too bright/glare
  handleBrightLight(): void {
    decreaseCameraExposure()
    adjustContrastSettings()
    suggestPositioning()
  }
  
  // Inconsistent lighting
  handleVariableLighting(): void {
    enableAutoAdjustment()
    increaseCalibrationFrequency()
  }
}
```

### 2. **User Behavior Edge Cases**

#### **Head Movement**
```typescript
interface HeadMovementHandling {
  // Excessive head movement
  handleHeadMovement(movement: HeadMovement): void {
    if (movement.magnitude > MOVEMENT_THRESHOLD) {
      pauseTracking()
      showStabilityMessage()
      waitForStability()
    }
  }
  
  // User leaves camera view
  handleUserNotVisible(): void {
    pauseWindowMovement()
    showReturnToViewMessage()
    maintainLastKnownPosition()
  }
  
  // Multiple faces detected
  handleMultipleFaces(faces: FaceBox[]): void {
    selectPrimaryFace(faces)
    notifyMultipleFacesDetected()
  }
}
```

#### **Eye Conditions**
```typescript
interface EyeConditionHandling {
  // User wearing glasses
  handleGlasses(): void {
    adjustDetectionParameters()
    useGlassesSpecificModel()
    increaseCalibrationSamples()
  }
  
  // One eye not visible
  handleMonocularTracking(): void {
    switchToSingleEyeMode()
    adjustAccuracyExpectations()
    informUserOfLimitations()
  }
  
  // Frequent blinking
  handleExcessiveBlinking(): void {
    filterBlinkFrames()
    extendPredictionWindow()
    adjustSensitivity()
  }
}
```

### 3. **Performance & System Issues**

#### **Performance Degradation**
```typescript
interface PerformanceManagement {
  // High CPU usage
  handleHighCPUUsage(): void {
    reduceProcessingFrameRate()
    simplifyDetectionModel()
    showPerformanceWarning()
  }
  
  // Memory issues
  handleMemoryPressure(): void {
    clearOldFrameData()
    reduceHistoryBuffer()
    forceGarbageCollection()
  }
  
  // Latency issues
  handleHighLatency(): void {
    increasePredictionAggression()
    reduceProcessingComplexity()
    skipFramesIfNecessary()
  }
}
```

#### **Multi-Monitor Handling**
```typescript
interface MultiMonitorSupport {
  // Monitor configuration changes
  handleMonitorChange(): void {
    recalibrateForNewSetup()
    updateScreenBounds()
    remapGazeCoordinates()
  }
  
  // Different DPI scales
  handleDPIVariations(): void {
    normalizeCoordinates()
    adjustMovementSensitivity()
    scaleCalibrationPoints()
  }
  
  // Gaze crosses monitor boundaries
  handleMonitorTransition(fromMonitor: MonitorInfo, toMonitor: MonitorInfo): void {
    adjustCoordinateSpace()
    maintainContinuousTracking()
  }
}
```

### 4. **Accessibility & Usability**

#### **Accessibility Support**
```typescript
interface AccessibilityFeatures {
  // Visual impairments
  supportLowVision(): void {
    provideHighContrastCalibration()
    enlargeCalibrationTargets()
    addAudioFeedback()
  }
  
  // Motor impairments
  supportMotorLimitations(): void {
    relaxMovementConstraints()
    increaseToleranceZones()
    provideDwellClickOptions()
  }
  
  // Cognitive support
  provideCognitiveAids(): void {
    simplifyCalibrationProcess()
    addProgressiveGuidance()
    enablePauseAndResume()
  }
}
```

#### **User Experience Fallbacks**
```typescript
interface FallbackMechanisms {
  // Eye tracking fails
  provideAlternativeControl(): void {
    enableMouseControl()
    addKeyboardShortcuts()
    provideTouchSupport()
  }
  
  // Calibration difficulties
  handleCalibrationFailure(): void {
    offerSimplifiedCalibration()
    provideManualConfiguration()
    enableAssistedMode()
  }
  
  // System incompatibility
  handleUnsupportedSystem(): void {
    detectSystemCapabilities()
    showCompatibilityInfo()
    disableGracefully()
  }
}
```

## 📊 Performance Optimization Strategy

### 1. **Processing Pipeline Optimization**

#### **Frame Rate Management**
```typescript
interface FrameRateOptimization {
  // Adaptive frame rate based on performance
  adaptiveFrameRate: {
    target: 30,        // Ideal FPS
    minimum: 15,       // Acceptable minimum
    current: number,   // Current achieved FPS
    adjustment: 'auto' // Automatic adjustment
  }
  
  // Frame skipping strategy
  frameSkipping: {
    skipPattern: [1, 0, 1, 0],  // Skip every other frame under load
    skipCount: number,           // Current skip count
    enabled: boolean             // Skip frames when needed
  }
}
```

#### **Memory Management**
```typescript
interface MemoryOptimization {
  // Buffer management
  frameBuffer: {
    maxSize: 10,              // Keep last 10 frames
    currentSize: number,      // Current buffer size
    cleanupThreshold: 8       // Cleanup when reaching 8 frames
  }
  
  // Data retention
  gazeHistory: {
    retentionTime: 5000,      // Keep 5 seconds of history
    maxPoints: 150,           // Maximum points to retain
    compressionEnabled: true   // Compress old data
  }
}
```

### 2. **Algorithm Optimization**

#### **Detection Hierarchy**
```typescript
interface DetectionOptimization {
  // Multi-level detection strategy
  detectionLevels: {
    coarse: {
      frequency: 10,          // 10 FPS for face detection
      accuracy: 'medium'      // Medium accuracy, fast processing
    },
    fine: {
      frequency: 30,          // 30 FPS for eye tracking
      accuracy: 'high',       // High accuracy when face found
      region: 'face_area'     // Only process face region
    }
  }
}
```

## 🎯 Implementation Roadmap

### **Phase 1: Foundation (Week 1-2)**
1. **Camera Integration**
   - MediaDevices API implementation
   - Permission handling
   - Stream management
   - Error recovery

2. **Basic Computer Vision**
   - OpenCV.js integration
   - Face detection pipeline
   - Eye region detection
   - Basic gaze estimation

### **Phase 2: Core Tracking (Week 3-4)**
1. **Gaze Calculation**
   - Pupil center detection
   - Gaze vector computation
   - Coordinate transformation
   - Real-time processing

2. **Window Movement**
   - Tauri window commands
   - Position calculation
   - Smooth animation
   - Boundary constraints

### **Phase 3: Calibration System (Week 5-6)**
1. **Calibration Interface**
   - Multi-point calibration
   - User guidance system
   - Progress tracking
   - Validation process

2. **Adaptive Learning**
   - Continuous calibration
   - Drift detection
   - User personalization
   - Accuracy improvement

### **Phase 4: Polish & Optimization (Week 7-8)**
1. **Performance Optimization**
   - Frame rate optimization
   - Memory management
   - Algorithm refinement
   - Battery efficiency

2. **Edge Case Handling**
   - Error recovery systems
   - Fallback mechanisms
   - Accessibility features
   - User experience polish

This comprehensive plan provides a robust foundation for implementing sophisticated eye tracking that makes your application truly follow where you look, creating an intuitive and magical user experience.