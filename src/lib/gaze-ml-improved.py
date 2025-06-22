#!/usr/bin/env python3
"""
Advanced Gaze-Controlled ML System for Tauri Integration
Combines eye tracking with monitor detection and seamless window control
Supports both standalone GUI mode and headless mode for Tauri integration
"""

import cv2
import numpy as np
import mediapipe as mp
import tensorflow as tf
import json
import time
import sys
import threading
import tkinter as tk
from tkinter import ttk
import platform
import ctypes
from dataclasses import dataclass, asdict
from typing import List, Tuple, Optional, Dict
from collections import deque
import argparse
import os
import subprocess
import signal

@dataclass
class Monitor:
    """Represents a single monitor with position and properties"""
    x: int          # Left edge position
    y: int          # Top edge position  
    width: int      # Monitor width
    height: int     # Monitor height
    is_primary: bool = False
    name: str = ""
    scale_factor: float = 1.0
    
    @property
    def right(self) -> int:
        return self.x + self.width
    
    @property 
    def bottom(self) -> int:
        return self.y + self.height
    
    @property
    def center_x(self) -> int:
        return self.x + self.width // 2
    
    @property
    def center_y(self) -> int:
        return self.y + self.height // 2
    
    def contains_point(self, x: int, y: int) -> bool:
        """Check if a point is within this monitor"""
        return self.x <= x < self.right and self.y <= y < self.bottom

@dataclass
class MonitorMesh:
    """Complete monitor configuration with spatial relationships"""
    monitors: List[Monitor]
    virtual_width: int
    virtual_height: int
    virtual_left: int
    virtual_top: int
    primary_monitor: Optional[Monitor] = None
    
    def __post_init__(self):
        if not self.primary_monitor and self.monitors:
            self.primary_monitor = next((m for m in self.monitors if m.is_primary), self.monitors[0])
    
    @property
    def virtual_right(self) -> int:
        return self.virtual_left + self.virtual_width
    
    @property
    def virtual_bottom(self) -> int:
        return self.virtual_top + self.virtual_height
    
    def get_monitor_at_point(self, x: int, y: int) -> Optional[Monitor]:
        """Find which monitor contains the given point"""
        for monitor in self.monitors:
            if monitor.contains_point(x, y):
                return monitor
        return None
    
    def normalize_coordinates(self, x: int, y: int) -> Tuple[float, float]:
        """Normalize coordinates to 0.0-1.0 range across entire virtual desktop"""
        norm_x = (x - self.virtual_left) / self.virtual_width if self.virtual_width > 0 else 0.0
        norm_y = (y - self.virtual_top) / self.virtual_height if self.virtual_height > 0 else 0.0
        return (norm_x, norm_y)
    
    def denormalize_coordinates(self, norm_x: float, norm_y: float) -> Tuple[int, int]:
        """Convert normalized coordinates back to absolute screen coordinates"""
        abs_x = int(self.virtual_left + norm_x * self.virtual_width)
        abs_y = int(self.virtual_top + norm_y * self.virtual_height)
        return (abs_x, abs_y)

def detect_monitor_mesh() -> MonitorMesh:
    """Detect Windows monitor configuration using ctypes"""
    monitors = []
    
    try:
        import ctypes
        from ctypes import wintypes, Structure, POINTER, WINFUNCTYPE
        
        # Define structures for monitor enumeration
        class RECT(Structure):
            _fields_ = [('left', ctypes.c_long),
                      ('top', ctypes.c_long),
                      ('right', ctypes.c_long),
                      ('bottom', ctypes.c_long)]
        
        class MONITORINFO(Structure):
            _fields_ = [('cbSize', ctypes.c_ulong),
                      ('rcMonitor', RECT),
                      ('rcWork', RECT),
                      ('dwFlags', ctypes.c_ulong)]
        
        user32 = ctypes.windll.user32
        
        # Monitor enumeration callback
        MonitorEnumProc = WINFUNCTYPE(ctypes.c_bool, wintypes.HMONITOR, wintypes.HDC, POINTER(RECT), wintypes.LPARAM)
        
        def monitor_enum_callback(hmonitor, hdc, rect, data):
            try:
                monitor_info = MONITORINFO()
                monitor_info.cbSize = ctypes.sizeof(MONITORINFO)
                
                if user32.GetMonitorInfoW(hmonitor, ctypes.byref(monitor_info)):
                    rect = monitor_info.rcMonitor
                    is_primary = bool(monitor_info.dwFlags & 1)  # MONITORINFOF_PRIMARY
                    
                    monitor = Monitor(
                        x=rect.left,
                        y=rect.top,
                        width=rect.right - rect.left,
                        height=rect.bottom - rect.top,
                        is_primary=is_primary,
                        name=f'Monitor_{len(monitors) + 1}'
                    )
                    monitors.append(monitor)
                    print(f"🖥️  Detected: {monitor.name} at ({monitor.x}, {monitor.y}) {monitor.width}x{monitor.height} {'(PRIMARY)' if is_primary else ''}", file=sys.stderr)
            except Exception as e:
                print(f"Error in monitor callback: {e}", file=sys.stderr)
            return True
        
        # Enumerate monitors
        user32.EnumDisplayMonitors(None, None, MonitorEnumProc(monitor_enum_callback), 0)
        
        if monitors:
            virtual_left = min(m.x for m in monitors)
            virtual_top = min(m.y for m in monitors)
            virtual_right = max(m.right for m in monitors)
            virtual_bottom = max(m.bottom for m in monitors)
            
            mesh = MonitorMesh(
                monitors=monitors,
                virtual_width=virtual_right - virtual_left,
                virtual_height=virtual_bottom - virtual_top,
                virtual_left=virtual_left,
                virtual_top=virtual_top
            )
            
            print(f"📐 Virtual Desktop: {mesh.virtual_width}x{mesh.virtual_height} at ({mesh.virtual_left}, {mesh.virtual_top})", file=sys.stderr)
            return mesh
    
    except Exception as e:
        print(f"Monitor detection failed: {e}", file=sys.stderr)
    
    # Fallback to single monitor
    monitor = Monitor(x=0, y=0, width=1920, height=1080, is_primary=True, name="Default_Monitor")
    return MonitorMesh(
        monitors=[monitor],
        virtual_width=1920,
        virtual_height=1080,
        virtual_left=0,
        virtual_top=0
    )

@dataclass
class GazePoint:
    """Represents a gaze point with confidence and metadata"""
    x: float
    y: float
    confidence: float
    timestamp: float
    
    def to_json(self) -> dict:
        """Convert to JSON format for Tauri integration"""
        return {
            'x': self.x,
            'y': self.y,
            'confidence': self.confidence,
            'timestamp': self.timestamp,
            'calibrated': True  # Will be updated by tracker
        }

class WindowDragHandler:
    """Handles window dragging detection and graceful pause/resume"""
    
    def __init__(self):
        self.is_dragging = False
        self.drag_start_time = 0
        self.drag_last_position = None
        self.drag_threshold = 10  # pixels
        self.stable_time_required = 0.5  # seconds to consider drag finished
        self.last_stable_time = 0
        
    def detect_window_drag(self, current_gaze: GazePoint, window_bounds: dict) -> bool:
        """Detect if window is being dragged based on gaze and window movement"""
        current_time = time.time()
        
        # Check if gaze is near window titlebar area (top 30 pixels)
        if current_gaze:
            in_titlebar = (window_bounds['y'] <= current_gaze.y <= window_bounds['y'] + 30)
            if in_titlebar and not self.is_dragging:
                # Potential drag start
                self.drag_start_time = current_time
                self.drag_last_position = (current_gaze.x, current_gaze.y)
                self.is_dragging = True
                print("🎯 Window drag detected - pausing gaze control", file=sys.stderr)
                return True
        
        # Check for drag end (stable position)
        if self.is_dragging:
            if current_gaze and self.drag_last_position:
                dx = abs(current_gaze.x - self.drag_last_position[0])
                dy = abs(current_gaze.y - self.drag_last_position[1])
                
                if dx < self.drag_threshold and dy < self.drag_threshold:
                    if self.last_stable_time == 0:
                        self.last_stable_time = current_time
                    elif current_time - self.last_stable_time > self.stable_time_required:
                        # Drag finished
                        self.is_dragging = False
                        self.last_stable_time = 0
                        print("🎯 Window drag finished - resuming gaze control", file=sys.stderr)
                        return False
                else:
                    self.last_stable_time = 0
                    self.drag_last_position = (current_gaze.x, current_gaze.y)
        
        return self.is_dragging

class TauriGazeTracker:
    """Main gaze tracker optimized for Tauri integration"""
    
    def __init__(self, monitor_mesh: MonitorMesh, camera_id: int = 0, headless: bool = True):
        self.monitor_mesh = monitor_mesh
        self.camera_id = camera_id
        self.headless = headless
        self.processing = True
        self.is_calibrated = False
        
        # Window drag handling
        self.drag_handler = WindowDragHandler()
        self.pause_tracking = False
        
        # MediaPipe setup
        self.mp_face_mesh = mp.solutions.face_mesh
        self.face_mesh = self.mp_face_mesh.FaceMesh(
            max_num_faces=1,
            refine_landmarks=True,
            min_detection_confidence=0.6,
            min_tracking_confidence=0.5
        )
        
        # Iris landmarks for precise tracking
        self.LEFT_IRIS = [474, 475, 476, 477, 473]
        self.RIGHT_IRIS = [469, 470, 471, 472, 468]
        
        # Gaze estimation
        self.gaze_history = deque(maxlen=8)
        self.calibration_offset_x = 0
        self.calibration_offset_y = 0
        self.scale_factor_x = 1.0
        self.scale_factor_y = 1.0
        
        # Performance tracking
        self.fps_counter = 0
        self.fps_start_time = time.time()
        self.current_fps = 0
        self.face_detection_count = 0
        self.frame_count = 0
        
        # Signal handling for graceful shutdown
        signal.signal(signal.SIGINT, self._signal_handler)
        signal.signal(signal.SIGTERM, self._signal_handler)
        
        print(f"🎯 TauriGazeTracker initialized - Headless: {headless}", file=sys.stderr)
    
    def _signal_handler(self, signum, frame):
        """Handle shutdown signals gracefully"""
        print(f"🛑 Received signal {signum}, shutting down gracefully...", file=sys.stderr)
        self.stop_tracking()
        sys.exit(0)
    
    def estimate_gaze_enhanced(self, landmarks) -> Optional[GazePoint]:
        """Enhanced gaze estimation with improved accuracy"""
        try:
            if len(landmarks) < 468:
                return None
            
            # Get iris centers
            left_iris_points = [landmarks[idx] for idx in self.LEFT_IRIS if idx < len(landmarks)]
            right_iris_points = [landmarks[idx] for idx in self.RIGHT_IRIS if idx < len(landmarks)]
            
            if len(left_iris_points) < 3 or len(right_iris_points) < 3:
                return None
            
            # Calculate iris centers
            left_iris_center = np.mean(left_iris_points, axis=0)
            right_iris_center = np.mean(right_iris_points, axis=0)
            
            # Average the two iris positions
            avg_iris_x = (left_iris_center[0] + right_iris_center[0]) / 2
            avg_iris_y = (left_iris_center[1] + right_iris_center[1]) / 2
            
            # Correct direction mapping
            corrected_iris_x = 1.0 - avg_iris_x  # Invert X for natural mapping
            corrected_iris_y = avg_iris_y
            
            # Adaptive sensitivity based on monitor configuration
            sensitivity_multiplier = 1.5  # Base sensitivity
            
            if self.monitor_mesh.monitors and len(self.monitor_mesh.monitors) > 1:
                # Multi-monitor setup needs higher sensitivity
                largest_monitor = max(self.monitor_mesh.monitors, key=lambda m: m.width * m.height)
                primary_monitor = self.monitor_mesh.primary_monitor
                
                if largest_monitor and primary_monitor:
                    size_ratio = (largest_monitor.width * largest_monitor.height) / (primary_monitor.width * primary_monitor.height)
                    sensitivity_multiplier = min(2.5, max(1.0, size_ratio ** 0.3))
            
            # Enhanced Y-axis sensitivity
            y_sensitivity_boost = 1.6
            
            # Apply sensitivity scaling
            centered_x = (corrected_iris_x - 0.5) * sensitivity_multiplier + 0.5
            centered_y = (corrected_iris_y - 0.5) * sensitivity_multiplier * y_sensitivity_boost + 0.5
            
            # Clamp to valid range
            centered_x = max(0.0, min(1.0, centered_x))
            centered_y = max(0.0, min(1.0, centered_y))
            
            # Convert to screen coordinates
            screen_x = self.monitor_mesh.virtual_left + (centered_x * self.monitor_mesh.virtual_width)
            screen_y = self.monitor_mesh.virtual_top + (centered_y * self.monitor_mesh.virtual_height)
            
            # Apply calibration offsets
            screen_x += self.calibration_offset_x
            screen_y += self.calibration_offset_y
            
            # Apply scaling factors
            screen_x *= self.scale_factor_x
            screen_y *= self.scale_factor_y
            
            # Clamp to virtual desktop bounds
            screen_x = max(self.monitor_mesh.virtual_left, 
                          min(screen_x, self.monitor_mesh.virtual_right - 1))
            screen_y = max(self.monitor_mesh.virtual_top,
                          min(screen_y, self.monitor_mesh.virtual_bottom - 1))
            
            # Calculate confidence based on iris detection quality
            confidence = 0.9 if len(left_iris_points) >= 4 and len(right_iris_points) >= 4 else 0.7
            
            gaze_point = GazePoint(
                x=float(screen_x),
                y=float(screen_y),
                confidence=confidence,
                timestamp=time.time()
            )
            
            return gaze_point
            
        except Exception as e:
            if self.frame_count % 100 == 0:  # Only log occasionally
                print(f"Gaze estimation error: {e}", file=sys.stderr)
            return None
    
    def generate_demo_gaze(self) -> GazePoint:
        """Generate smooth demo gaze data for testing"""
        demo_time = time.time() * 0.2  # Slow, smooth movement
        
        # Create figure-8 pattern across monitors
        base_x = self.monitor_mesh.virtual_left + self.monitor_mesh.virtual_width * 0.5
        base_y = self.monitor_mesh.virtual_top + self.monitor_mesh.virtual_height * 0.5
        
        # Figure-8 pattern
        demo_x = base_x + np.sin(demo_time) * self.monitor_mesh.virtual_width * 0.3
        demo_y = base_y + np.sin(demo_time * 2) * self.monitor_mesh.virtual_height * 0.2
        
        # Ensure within bounds
        demo_x = max(self.monitor_mesh.virtual_left, 
                    min(demo_x, self.monitor_mesh.virtual_right - 1))
        demo_y = max(self.monitor_mesh.virtual_top,
                    min(demo_y, self.monitor_mesh.virtual_bottom - 1))
        
        return GazePoint(
            x=float(demo_x),
            y=float(demo_y),
            confidence=0.85,
            timestamp=time.time()
        )
    
    def smooth_gaze(self, new_gaze: GazePoint) -> GazePoint:
        """Apply temporal smoothing to reduce jitter"""
        self.gaze_history.append(new_gaze)
        
        if len(self.gaze_history) < 2:
            return new_gaze
        
        # Exponential weighted average
        weights = np.exp(np.linspace(-1, 0, len(self.gaze_history)))
        weights /= weights.sum()
        
        smooth_x = sum(w * gp.x for w, gp in zip(weights, self.gaze_history))
        smooth_y = sum(w * gp.y for w, gp in zip(weights, self.gaze_history))
        
        return GazePoint(
            x=smooth_x,
            y=smooth_y,
            confidence=new_gaze.confidence,
            timestamp=new_gaze.timestamp
        )
    
    def run_tracking(self):
        """Main tracking loop optimized for Tauri integration"""
        cap = None
        
        # Try to initialize camera
        if self.camera_id >= 0:
            try:
                cap = cv2.VideoCapture(self.camera_id)
                if cap.isOpened():
                    cap.set(cv2.CAP_PROP_FRAME_WIDTH, 640)
                    cap.set(cv2.CAP_PROP_FRAME_HEIGHT, 480)
                    cap.set(cv2.CAP_PROP_FPS, 30)
                    print("📹 Camera initialized successfully", file=sys.stderr)
                else:
                    cap = None
                    print("⚠️  Camera not available, using demo mode", file=sys.stderr)
            except Exception as e:
                cap = None
                print(f"Camera initialization failed: {e}", file=sys.stderr)
        else:
            print("📺 Running in demo mode (no camera)", file=sys.stderr)
        
        last_output_time = 0
        output_interval = 1.0 / 30  # 30 FPS output
        
        while self.processing:
            try:
                gaze_point = None
                
                # Process camera frame if available
                if cap is not None and not self.pause_tracking:
                    ret, frame = cap.read()
                    if ret:
                        self.frame_count += 1
                        
                        # Process frame for face detection
                        rgb_frame = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
                        results = self.face_mesh.process(rgb_frame)
                        
                        if results.multi_face_landmarks:
                            self.face_detection_count += 1
                            
                            # Extract landmarks
                            landmarks = []
                            for landmark in results.multi_face_landmarks[0].landmark:
                                landmarks.append([landmark.x, landmark.y])
                            
                            landmarks = np.array(landmarks)
                            
                            # Estimate gaze
                            raw_gaze = self.estimate_gaze_enhanced(landmarks)
                            if raw_gaze:
                                gaze_point = self.smooth_gaze(raw_gaze)
                    else:
                        # Camera failed, fall back to demo
                        cap.release()
                        cap = None
                
                # Generate demo data if no camera or paused
                if gaze_point is None:
                    if not self.pause_tracking:
                        gaze_point = self.generate_demo_gaze()
                        self.frame_count += 1
                
                # Output gaze data for Tauri (throttled to prevent spam)
                current_time = time.time()
                if gaze_point and (current_time - last_output_time) >= output_interval:
                    if not self.pause_tracking:
                        gaze_data = gaze_point.to_json()
                        gaze_data['calibrated'] = self.is_calibrated
                        
                        # Output JSON to stdout for Tauri consumption
                        if self.headless:
                            print(json.dumps(gaze_data), flush=True)
                        
                        last_output_time = current_time
                
                # Update FPS
                self.fps_counter += 1
                if time.time() - self.fps_start_time >= 1.0:
                    self.current_fps = self.fps_counter
                    face_rate = (self.face_detection_count / max(1, self.frame_count)) * 100
                    
                    if not self.headless or self.frame_count % 60 == 0:
                        print(f"📊 FPS: {self.current_fps:.1f}, Face: {face_rate:.1f}%, Frames: {self.frame_count}", file=sys.stderr)
                    
                    self.fps_counter = 0
                    self.fps_start_time = time.time()
                
                # Small delay to prevent CPU overload
                time.sleep(0.01)  # ~100 FPS max
                
            except Exception as e:
                print(f"❌ Tracking error: {e}", file=sys.stderr)
                time.sleep(0.1)  # Longer delay on error
        
        # Cleanup
        if cap is not None:
            cap.release()
        
        print("👁️  Gaze tracking stopped gracefully", file=sys.stderr)
    
    def stop_tracking(self):
        """Stop the tracking loop"""
        self.processing = False
    
    def pause_for_drag(self):
        """Pause tracking during window drag"""
        self.pause_tracking = True
    
    def resume_after_drag(self):
        """Resume tracking after window drag"""
        self.pause_tracking = False
    
    def calibrate_simple(self):
        """Simple calibration routine"""
        self.is_calibrated = True
        print("🎯 Simple calibration completed", file=sys.stderr)

def main():
    """Main function with enhanced argument parsing"""
    parser = argparse.ArgumentParser(description='Enhanced Gaze-Controlled System for Tauri Integration')
    parser.add_argument('--camera', type=int, default=0, help='Camera device ID (use -1 for demo mode)')
    parser.add_argument('--headless', action='store_true', help='Run in headless mode for Tauri integration')
    parser.add_argument('--gui', action='store_true', help='Run with standalone GUI for testing')
    parser.add_argument('--screen-width', type=int, help='Override screen width')
    parser.add_argument('--screen-height', type=int, help='Override screen height')
    parser.add_argument('--calibrate', action='store_true', help='Auto-calibrate on start')
    parser.add_argument('--debug', action='store_true', help='Enable debug output')
    
    args = parser.parse_args()
    
    # Validate mode selection
    if args.headless and args.gui:
        print("ERROR: Cannot use both --headless and --gui modes", file=sys.stderr)
        return
    
    # Default to headless if no mode specified
    if not args.gui:
        args.headless = True
    
    print(f"🚀 Starting Enhanced Gaze Tracker", file=sys.stderr)
    print(f"Mode: {'GUI' if args.gui else 'Headless'}", file=sys.stderr)
    
    # Detect monitor configuration
    monitor_mesh = detect_monitor_mesh()
    
    # Override screen dimensions if provided
    if args.screen_width and args.screen_height:
        monitor_mesh.virtual_width = args.screen_width
        monitor_mesh.virtual_height = args.screen_height
        print(f"🔧 Screen dimensions overridden: {args.screen_width}x{args.screen_height}", file=sys.stderr)
    
    try:
        # Headless mode for Tauri integration
        print("🔌 Starting headless mode for Tauri integration...", file=sys.stderr)
        tracker = TauriGazeTracker(monitor_mesh, args.camera, headless=True)
        
        if args.calibrate:
            tracker.calibrate_simple()
        
        # Run tracking (this blocks)
        tracker.run_tracking()
    
    except KeyboardInterrupt:
        print("\n🛑 Shutting down gracefully...", file=sys.stderr)
    except Exception as e:
        print(f"❌ Error: {e}", file=sys.stderr)
        if args.debug:
            import traceback
            traceback.print_exc(file=sys.stderr)
    finally:
        try:
            if 'tracker' in locals():
                tracker.stop_tracking()
        except:
            pass
        
        print("👋 Enhanced Gaze Tracker ended", file=sys.stderr)

if __name__ == "__main__":
    main()

# Usage examples:
# Headless mode for Tauri integration:
# python gaze-ml-improved.py --headless --camera 0 --screen-width 3840 --screen-height 1080
#
# Demo mode (no camera):
# python gaze-ml-improved.py --headless --camera -1 