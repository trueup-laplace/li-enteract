// Core geometric types
export interface Point2D {
  x: number
  y: number
}

export interface Rectangle {
  x: number
  y: number
  width: number
  height: number
}

// Face detection types
export interface FaceBox {
  x: number
  y: number          // Top-left corner
  width: number
  height: number     // Dimensions
  confidence: number // Detection confidence (0-1)
  landmarks?: FaceLandmarks // Optional facial landmarks
}

export interface FaceLandmarks {
  leftEye: Point2D
  rightEye: Point2D
  nose: Point2D
  leftMouth: Point2D
  rightMouth: Point2D
}

export interface FaceMovement {
  deltaX: number
  deltaY: number
  confidence: number
}

// Eye tracking types
export interface EyeRegion {
  boundingBox: Rectangle
  pupilCenter: Point2D
  confidence: number
  isOpen: boolean // Blink detection
}

export interface EyePair {
  left: EyeRegion
  right: EyeRegion
  isValid: boolean
}

export interface PupilPair {
  left: Point2D
  right: Point2D
  confidence: number
}

export interface GazeVector {
  x: number         // Horizontal gaze direction (-1 to 1)
  y: number         // Vertical gaze direction (-1 to 1)
  confidence: number // Tracking confidence
  timestamp: number  // Frame timestamp
}

// Calibration types
export interface CalibrationSample {
  targetPosition: Point2D
  gazeData: GazeVector[]
  timestamp: number
}

export interface CalibrationResult {
  accuracy: number          // Overall accuracy percentage
  transformMatrix: number[] // Gaze-to-screen mapping
  userProfile: UserProfile  // Individual user characteristics
  isValid: boolean          // Calibration success status
}

export interface CalibrationSequence {
  points: Point2D[]     // 9-point or 16-point grid
  duration: number      // Time per point (2-3 seconds)
  repetitions: number   // Samples per point (3-5)
  validation: boolean   // Post-calibration validation
}

export interface CalibrationAccuracy {
  overall: number
  perPoint: number[]
  worstPoint: number
  bestPoint: number
}

export interface UserProfile {
  id: string
  eyeDistance: number
  headSize: number
  preferences: UserPreferences
}

export interface UserPreferences {
  sensitivity: number
  smoothing: number
  dwellTime: number
}

// Monitor and window types
export interface MonitorInfo {
  id: string
  bounds: Rectangle     // Monitor boundaries
  scaleFactor: number   // DPI scaling
  isPrimary: boolean    // Primary display
}

export interface WindowInfo {
  position: Point2D
  size: { width: number; height: number }
  bounds: Rectangle
}

// Movement and animation types
export interface MovementConstraints {
  screenBounds: Rectangle   // Keep window on screen
  minDistance: number       // Minimum movement threshold
  maxSpeed: number          // Maximum movement speed
  deadZones: Rectangle[]    // Areas to avoid (taskbar, etc.)
}

export interface AnimationConfig {
  duration: number          // Animation time (100-300ms)
  easing: EasingFunction    // Smooth movement curve
  anticipation: boolean     // Start moving before gaze settles
}

export type EasingFunction = 'linear' | 'ease-in' | 'ease-out' | 'ease-in-out' | 'cubic-bezier'

// Processing and performance types
export interface EyeTrackingResult {
  success: boolean
  gaze: GazeVector | null
  confidence: number
  faceDetected: boolean
  processingTime?: number
}

export interface PerformanceMetrics {
  fps: number
  processingTime: number
  memoryUsage: number
  accuracy: number
}

export interface HeadPose {
  rotation: {
    pitch: number  // Up/down
    yaw: number    // Left/right
    roll: number   // Tilt
  }
  position: Point2D
  confidence: number
}

// State management types
export interface EyeTrackingState {
  isActive: boolean
  isCalibrated: boolean
  currentGaze: GazeVector | null
  faceDetected: boolean
  confidence: number
  error: string | null
}

export interface CalibrationState {
  isCalibrating: boolean
  currentPoint: number
  totalPoints: number
  accuracy: number
  isComplete: boolean
}

// Error types
export interface EyeTrackingError {
  code: string
  message: string
  severity: 'low' | 'medium' | 'high'
  timestamp: number
}

// Configuration types
export interface EyeTrackingConfig {
  camera: {
    frameRate: number
    resolution: { width: number; height: number }
  }
  processing: {
    faceDetectionRate: number
    eyeTrackingRate: number
    smoothingWindow: number
  }
  movement: {
    sensitivity: number
    smoothing: number
    boundaries: Rectangle
  }
}

// User interaction types
export interface UserInteraction {
  type: 'gaze' | 'click' | 'dwell'
  position: Point2D
  duration: number
  timestamp: number
}

// Worker communication types
export interface WorkerMessage {
  type: 'process-frame' | 'configure' | 'calibrate'
  data: any
  timestamp: number
}

export interface WorkerResponse {
  type: 'result' | 'error' | 'status'
  data: any
  timestamp: number
} 