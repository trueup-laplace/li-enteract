# Advanced Gaze Tracking Integration

## Overview

This document outlines the integration of your advanced gaze tracking system based on `gaze-tracker.py` into the Tauri application. The new system provides enhanced functionality including multi-monitor support, MediaPipe-based tracking, and improved calibration capabilities.

## 🔧 Implementation Summary

### 1. Core Python Script: `gaze-tracker-application.py`

**Location**: `src/lib/gaze-tracker-application.py`

This is the enhanced version of your original `gaze-tracker.py` script, specifically adapted for integration with the Tauri application:

**Key Features:**
- ✅ **Multi-monitor Detection**: Automatically detects and maps monitor configurations on Windows, macOS, and Linux
- ✅ **MediaPipe Integration**: Uses MediaPipe face mesh for precise iris tracking
- ✅ **Enhanced Calibration**: Sophisticated calibration system with transformation matrices
- ✅ **JSON Output**: Structured data output compatible with Tauri backend
- ✅ **Real-time Tracking**: 30 FPS gaze data with smoothing and confidence metrics
- ✅ **Cross-platform**: Works on Windows, macOS, and Linux

**Key Components:**
```python
class GazeTrackerApplication:
    - Monitor mesh detection
    - MediaPipe face tracking
    - Gaze estimation with confidence
    - Calibration point collection
    - Real-time data streaming

class MonitorMesh:
    - Multi-monitor spatial awareness
    - Virtual desktop calculations
    - Monitor-specific gaze mapping
```

### 2. Enhanced Vue Composable: `useAdvancedGazeTracking.ts`

**Location**: `src/composables/useAdvancedGazeTracking.ts`

A new Vue composable that interfaces with the Python script:

**Features:**
- 🎯 **Advanced Tracking**: Superior gaze estimation with monitor awareness
- 📊 **Real-time Statistics**: FPS, confidence, and performance metrics
- 🔧 **Configuration Management**: Camera settings, thresholds, and smoothing
- 📈 **Quality Monitoring**: Tracking quality assessment and reporting

**API:**
```typescript
const gazeTracking = useAdvancedGazeTracking()

// Core methods
await gazeTracking.initialize()
await gazeTracking.startTracking()
await gazeTracking.stopTracking()
await gazeTracking.startCalibration()

// Reactive state
gazeTracking.isActive.value
gazeTracking.currentGaze.value
gazeTracking.trackingQuality.value
gazeTracking.stats.value
```

### 3. Enhanced Window Control: `useGazeWindowControl.ts`

**Updated Location**: `src/composables/useGazeWindowControl.ts`

Enhanced the existing window control composable to support both basic and advanced tracking:

**New Features:**
- 🔄 **Dual Tracking Support**: Can use either basic or advanced tracking
- 🎯 **Automatic Fallback**: Falls back to basic tracking if advanced fails
- 📊 **Enhanced Statistics**: Better movement tracking and analytics
- ⚙️ **Flexible Configuration**: Support for different tracking modes

**API:**
```typescript
const gazeControl = useGazeWindowControl()

// Enhanced methods
await gazeControl.startGazeControl(useAdvanced: true)
await gazeControl.startBasicGazeControl()

// Access to both tracking systems
gazeControl.eyeTracking          // Basic tracking
gazeControl.advancedGazeTracking // Advanced tracking
```

### 4. Demo Component: `AdvancedGazeDemo.vue`

**Location**: `src/components/core/AdvancedGazeDemo.vue`

A demonstration component showcasing the new advanced tracking capabilities:

**Features:**
- 🖥️ **Live Demo Interface**: Real-time gaze visualization
- 🎮 **Interactive Controls**: Start/stop tracking and calibration
- 📊 **Status Monitoring**: Live tracking quality and statistics
- ❌ **Error Handling**: Clear error reporting and recovery

### 5. Tauri Backend Integration

**Updated**: `src-tauri/src/eye_tracking.rs`

The Rust backend now launches the new Python script:

**Changes:**
- ✅ **Script Path Updated**: Now uses `gaze-tracker-application.py`
- 🔄 **Maintained Compatibility**: Same interface for existing Vue components
- 📡 **JSON Communication**: Structured data exchange with Python script

## 🚀 How to Use

### Basic Setup

1. **Install Dependencies**: Ensure Python dependencies are installed:
   ```bash
   pip install cv2 mediapipe numpy
   ```

2. **Start the Application**: Launch your Tauri app normally

3. **Use Advanced Tracking**: In your Vue components:
   ```vue
   <script setup>
   import { useAdvancedGazeTracking } from '@/composables/useAdvancedGazeTracking'
   
   const gazeTracking = useAdvancedGazeTracking()
   
   // Start advanced tracking
   await gazeTracking.startTracking()
   </script>
   ```

### Advanced Usage

1. **Multi-Monitor Setup**:
   - The system automatically detects all connected monitors
   - Gaze coordinates are mapped to the virtual desktop space
   - Monitor switching is detected and tracked

2. **Calibration Process**:
   ```typescript
   // Start calibration
   await gazeTracking.startCalibration()
   
   // Monitor calibration progress
   watch(gazeTracking.calibrationProgress, (progress) => {
     console.log(`Calibration: ${progress}%`)
   })
   ```

3. **Window Control Integration**:
   ```typescript
   const gazeControl = useGazeWindowControl()
   
   // Start with advanced tracking
   await gazeControl.startGazeControl(true)
   
   // Or fallback to basic tracking
   await gazeControl.startGazeControl(false)
   ```

## 📊 Data Structures

### Gaze Data Format
```typescript
interface AdvancedGazeData {
  x: number              // Screen X coordinate
  y: number              // Screen Y coordinate  
  confidence: number     // Confidence (0-1)
  left_eye_landmarks: number[][]   // Left eye landmarks
  right_eye_landmarks: number[][]  // Right eye landmarks
  head_pose: {           // Head orientation
    yaw: number
    pitch: number
    roll: number
  }
  timestamp: number      // Unix timestamp
}
```

### Monitor Information
```typescript
interface MonitorInfo {
  x: number              // Monitor X position
  y: number              // Monitor Y position
  width: number          // Monitor width
  height: number         // Monitor height
  is_primary: boolean    // Primary monitor flag
  name: string           // Monitor name
  scale_factor: number   // DPI scale factor
}
```

## 🎯 Key Improvements Over Previous Implementation

1. **Multi-Monitor Support**: 
   - Automatic detection of monitor configurations
   - Spatial awareness across multiple displays
   - Virtual desktop coordinate mapping

2. **Enhanced Accuracy**:
   - MediaPipe iris tracking for precise gaze estimation
   - Advanced calibration with transformation matrices
   - Confidence-based filtering and smoothing

3. **Better Integration**:
   - Clean Vue composable API
   - Maintains backward compatibility
   - Structured error handling and recovery

4. **Performance Optimizations**:
   - 30 FPS real-time tracking
   - Efficient data streaming
   - Optimized memory usage

5. **Cross-Platform Compatibility**:
   - Windows monitor detection via Win32 API
   - macOS support via system_profiler
   - Linux support via xrandr
   - Fallback mechanisms for all platforms

## 🔧 Configuration Options

The new system provides extensive configuration options:

```typescript
interface GazeTrackingConfig {
  camera_id: number                    // Camera device ID
  screen_width: number                 // Screen width
  screen_height: number                // Screen height
  smoothing_window: number             // Smoothing frames
  confidence_threshold: number         // Minimum confidence
  enable_monitor_mesh: boolean         // Multi-monitor support
}
```

## 🐛 Troubleshooting

### Common Issues

1. **Camera Not Found**:
   - Check camera permissions
   - Verify camera ID (try 0, 1, 2...)
   - Ensure camera is not used by other applications

2. **Poor Tracking Quality**:
   - Ensure good lighting conditions
   - Position camera at eye level
   - Run calibration process
   - Check confidence threshold settings

3. **Multi-Monitor Issues**:
   - Verify monitor detection in system settings
   - Check virtual desktop configuration
   - Ensure displays are properly arranged

### Debug Information

Enable debug output by checking the browser console and Tauri logs for:
- Monitor detection results
- Camera initialization status
- Gaze data confidence levels
- Calibration progress and results

## 🎉 Conclusion

Your original `gaze-tracker.py` script has been successfully integrated into the Tauri application with significant enhancements:

- ✅ **Seamless Integration**: Works with existing Vue components
- ✅ **Enhanced Functionality**: Multi-monitor support and improved accuracy
- ✅ **Backward Compatibility**: Existing components continue to work
- ✅ **Extensible Architecture**: Easy to add new features and improvements

The new system provides a robust foundation for advanced gaze-controlled applications while maintaining the simplicity and effectiveness of your original design. 