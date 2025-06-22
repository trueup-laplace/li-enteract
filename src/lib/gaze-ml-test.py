#!/usr/bin/env python3
"""
Gaze-Controlled GUI Demo with Multi-Monitor Support
Combines eye tracking with monitor detection to create a window that follows your gaze
across multiple monitors in real-time.
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
from dataclasses import dataclass
from typing import List, Tuple, Optional, Dict
from collections import deque
import argparse
import os
import subprocess

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
                    print(f"🖥️  Detected: {monitor.name} at ({monitor.x}, {monitor.y}) {monitor.width}x{monitor.height} {'(PRIMARY)' if is_primary else ''}")
            except Exception as e:
                print(f"Error in monitor callback: {e}")
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
            
            print(f"📐 Virtual Desktop: {mesh.virtual_width}x{mesh.virtual_height} at ({mesh.virtual_left}, {mesh.virtual_top})")
            return mesh
    
    except Exception as e:
        print(f"Monitor detection failed: {e}")
    
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

class GazeControlledWindow:
    """A Tkinter window that follows your gaze across multiple monitors"""
    
    def __init__(self, monitor_mesh: MonitorMesh):
        self.monitor_mesh = monitor_mesh
        self.window_size = (300, 200)
        self.current_gaze = None
        self.gaze_history = deque(maxlen=10)
        self.smoothing_factor = 0.7  # For smooth movement
        self.last_position = (100, 100)
        
        # Create the main window
        self.root = tk.Tk()
        self.root.title("🎯 Gaze-Controlled Window")
        self.root.geometry(f"{self.window_size[0]}x{self.window_size[1]}")
        self.root.configure(bg='#2c3e50')
        
        # Make window always on top and remove decorations for demo effect
        self.root.attributes('-topmost', True)
        self.root.overrideredirect(False)  # Keep title bar for now
        
        # Create content
        self.setup_ui()
        
        # Position window initially at center of LAPTOP SCREEN (where camera is)
        if self.monitor_mesh.primary_monitor:
            initial_x = self.monitor_mesh.primary_monitor.center_x - self.window_size[0] // 2
            initial_y = self.monitor_mesh.primary_monitor.center_y - self.window_size[1] // 2
            print(f"🎯 Starting window on LAPTOP SCREEN ({self.monitor_mesh.primary_monitor.name}) at ({initial_x}, {initial_y})")
            self.root.geometry(f"+{initial_x}+{initial_y}")
            self.last_position = (initial_x, initial_y)
        else:
            # Fallback to virtual desktop center
            initial_x = self.monitor_mesh.virtual_left + self.monitor_mesh.virtual_width // 2 - self.window_size[0] // 2
            initial_y = self.monitor_mesh.virtual_top + self.monitor_mesh.virtual_height // 2 - self.window_size[1] // 2
            print(f"🎯 Starting window at virtual center ({initial_x}, {initial_y})")
            self.root.geometry(f"+{initial_x}+{initial_y}")
            self.last_position = (initial_x, initial_y)
    
    def setup_ui(self):
        """Setup the UI elements"""
        # Main frame
        main_frame = tk.Frame(self.root, bg='#2c3e50', padx=20, pady=20)
        main_frame.pack(fill='both', expand=True)
        
        # Title
        title_label = tk.Label(
            main_frame,
            text="👁️ Gaze Tracker",
            font=('Arial', 16, 'bold'),
            fg='#ecf0f1',
            bg='#2c3e50'
        )
        title_label.pack(pady=(0, 10))
        
        # Current gaze info
        self.gaze_label = tk.Label(
            main_frame,
            text="Gaze: (0, 0)",
            font=('Courier', 12),
            fg='#3498db',
            bg='#2c3e50'
        )
        self.gaze_label.pack(pady=5)
        
        # Monitor info
        self.monitor_label = tk.Label(
            main_frame,
            text="Monitor: None",
            font=('Courier', 10),
            fg='#e74c3c',
            bg='#2c3e50'
        )
        self.monitor_label.pack(pady=5)
        
        # Confidence info
        self.confidence_label = tk.Label(
            main_frame,
            text="Confidence: 0%",
            font=('Courier', 10),
            fg='#f39c12',
            bg='#2c3e50'
        )
        self.confidence_label.pack(pady=5)
        
        # Progress bar for confidence
        self.confidence_bar = ttk.Progressbar(
            main_frame,
            length=200,
            mode='determinate'
        )
        self.confidence_bar.pack(pady=5)
        
        # Instructions
        instructions = tk.Label(
            main_frame,
            text="This window follows your gaze!\nLook around to move it across monitors.",
            font=('Arial', 9),
            fg='#95a5a6',
            bg='#2c3e50',
            justify='center'
        )
        instructions.pack(pady=(10, 0))
    
    def update_gaze(self, gaze_point: GazePoint):
        """Update the window position based on gaze data"""
        if gaze_point is None:
            return
        
        self.current_gaze = gaze_point
        self.gaze_history.append(gaze_point)
        
        # Smooth the gaze data
        if len(self.gaze_history) > 1:
            # Weight recent points more heavily
            weights = np.exp(np.linspace(-1, 0, len(self.gaze_history)))
            weights /= weights.sum()
            
            smooth_x = sum(w * gp.x for w, gp in zip(weights, self.gaze_history))
            smooth_y = sum(w * gp.y for w, gp in zip(weights, self.gaze_history))
        else:
            smooth_x, smooth_y = gaze_point.x, gaze_point.y
        
        # Calculate new window position (offset slightly so window doesn't block view)
        offset_x, offset_y = 50, 50  # Offset from gaze point
        target_x = int(smooth_x + offset_x)
        target_y = int(smooth_y + offset_y)
        
        # Ensure window stays within virtual desktop bounds
        target_x = max(self.monitor_mesh.virtual_left, 
                      min(target_x, self.monitor_mesh.virtual_right - self.window_size[0]))
        target_y = max(self.monitor_mesh.virtual_top,
                      min(target_y, self.monitor_mesh.virtual_bottom - self.window_size[1]))
        
        # Apply smoothing to window movement
        current_x, current_y = self.last_position
        new_x = int(current_x * self.smoothing_factor + target_x * (1 - self.smoothing_factor))
        new_y = int(current_y * self.smoothing_factor + target_y * (1 - self.smoothing_factor))
        
        # Move the window
        try:
            self.root.geometry(f"+{new_x}+{new_y}")
            self.last_position = (new_x, new_y)
        except Exception as e:
            print(f"Failed to move window: {e}")
        
        # Update UI labels
        self.update_labels(gaze_point, smooth_x, smooth_y)
    
    def update_labels(self, gaze_point: GazePoint, smooth_x: float, smooth_y: float):
        """Update the information labels in the window"""
        try:
            # Update gaze coordinates
            self.gaze_label.config(text=f"Gaze: ({int(smooth_x)}, {int(smooth_y)})")
            
            # Update monitor information
            monitor = self.monitor_mesh.get_monitor_at_point(int(smooth_x), int(smooth_y))
            if monitor:
                rel_x = (smooth_x - monitor.x) / monitor.width
                rel_y = (smooth_y - monitor.y) / monitor.height
                monitor_text = f"Monitor: {monitor.name} ({rel_x:.2f}, {rel_y:.2f})"
                self.monitor_label.config(text=monitor_text, fg='#27ae60')
            else:
                self.monitor_label.config(text="Monitor: Outside bounds", fg='#e74c3c')
            
            # Update confidence
            confidence_pct = int(gaze_point.confidence * 100)
            self.confidence_label.config(text=f"Confidence: {confidence_pct}%")
            self.confidence_bar['value'] = confidence_pct
            
            # Color code confidence
            if confidence_pct > 70:
                self.confidence_label.config(fg='#27ae60')  # Green
            elif confidence_pct > 40:
                self.confidence_label.config(fg='#f39c12')  # Orange
            else:
                self.confidence_label.config(fg='#e74c3c')   # Red
                
        except Exception as e:
            print(f"Failed to update labels: {e}")
    
    def start(self):
        """Start the GUI event loop"""
        self.root.mainloop()
    
    def destroy(self):
        """Clean up the window"""
        if self.root:
            self.root.destroy()

class SimpleEyeTracker:
    """Simplified eye tracker focused on demo functionality"""
    
    def __init__(self, monitor_mesh: MonitorMesh, camera_id: int = 0):
        self.monitor_mesh = monitor_mesh
        self.camera_id = camera_id
        self.processing = True
        
        # MediaPipe setup
        self.mp_face_mesh = mp.solutions.face_mesh
        self.face_mesh = self.mp_face_mesh.FaceMesh(
            max_num_faces=1,
            refine_landmarks=True,
            min_detection_confidence=0.5,
            min_tracking_confidence=0.5
        )
        
        # Iris landmarks for precise tracking
        self.LEFT_IRIS = [474, 475, 476, 477, 473]
        self.RIGHT_IRIS = [469, 470, 471, 472, 468]
        
        # Simple gaze estimation (no ML model for demo)
        self.calibration_offset_x = 0
        self.calibration_offset_y = 0
        self.scale_factor_x = 1.0
        self.scale_factor_y = 1.0
        
        # Performance tracking
        self.fps_counter = 0
        self.fps_start_time = time.time()
        self.current_fps = 0
    
    def estimate_gaze_simple(self, landmarks) -> Optional[GazePoint]:
        """Simple gaze estimation with improved sensitivity for larger displays"""
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
            
            # CORRECTED DIRECTION MAPPING
            # Invert X direction to match natural expectation
            corrected_iris_x = 1.0 - avg_iris_x
            corrected_iris_y = avg_iris_y
            
            # ADAPTIVE SENSITIVITY based on monitor sizes
            laptop_screen = self.monitor_mesh.primary_monitor  # This is now the actual laptop screen
            
            # Calculate sensitivity multipliers
            # Larger external monitors need higher sensitivity for comfortable use
            if laptop_screen:
                base_resolution = laptop_screen.width * laptop_screen.height
                
                # Find the largest external monitor for sensitivity scaling
                external_monitors = [m for m in self.monitor_mesh.monitors if m != laptop_screen]
                if external_monitors:
                    largest_external = max(external_monitors, key=lambda m: m.width * m.height)
                    external_resolution = largest_external.width * largest_external.height
                    
                    # Calculate sensitivity multiplier (larger screens = higher sensitivity)
                    sensitivity_multiplier = max(1.0, (external_resolution / base_resolution) ** 0.3)
                    print(f"🎯 Sensitivity multiplier: {sensitivity_multiplier:.2f} for external {largest_external.width}x{largest_external.height}")
                else:
                    sensitivity_multiplier = 1.0
            else:
                sensitivity_multiplier = 1.0
            
            # Apply sensitivity scaling to iris movement
            # INCREASED Y-AXIS SENSITIVITY for better vertical tracking
            y_sensitivity_boost = 1.4  # 40% more sensitive in Y direction
            
            # Center the normalized coordinates around 0.5, apply sensitivity, then normalize back
            centered_x = (corrected_iris_x - 0.5) * sensitivity_multiplier + 0.5
            centered_y = (corrected_iris_y - 0.5) * sensitivity_multiplier * y_sensitivity_boost + 0.5
            
            # Clamp to valid range
            centered_x = max(0.0, min(1.0, centered_x))
            centered_y = max(0.0, min(1.0, centered_y))
            
            # Convert to screen coordinates across the virtual desktop
            base_screen_x = self.monitor_mesh.virtual_left + (centered_x * self.monitor_mesh.virtual_width)
            base_screen_y = self.monitor_mesh.virtual_top + (centered_y * self.monitor_mesh.virtual_height)
            
            # Add gentle demo movement for testing (reduced influence)
            demo_time = time.time() * 0.15  # Even slower movement
            demo_offset_x = np.sin(demo_time) * 100  # Smaller amplitude
            demo_offset_y = np.cos(demo_time * 0.4) * 60
            
            # Start from laptop screen center as base (where camera is)
            laptop_center_x = laptop_screen.center_x if laptop_screen else 0
            laptop_center_y = laptop_screen.center_y if laptop_screen else 0
            
            # Favor real tracking more heavily now
            final_x = laptop_center_x + demo_offset_x + (base_screen_x - laptop_center_x) * 0.8
            final_y = laptop_center_y + demo_offset_y + (base_screen_y - laptop_center_y) * 0.8
            
            # Clamp to virtual desktop bounds
            final_x = max(self.monitor_mesh.virtual_left, 
                         min(final_x, self.monitor_mesh.virtual_right - 1))
            final_y = max(self.monitor_mesh.virtual_top,
                         min(final_y, self.monitor_mesh.virtual_bottom - 1))
            
            # Calculate confidence based on iris detection quality
            confidence = 0.8 if len(left_iris_points) >= 4 and len(right_iris_points) >= 4 else 0.6
            
            return GazePoint(
                x=float(final_x),
                y=float(final_y),
                confidence=confidence,
                timestamp=time.time()
            )
            
        except Exception as e:
            print(f"Gaze estimation error: {e}")
            return None
    
    def run_tracking(self, gaze_window: GazeControlledWindow):
        """Main tracking loop"""
        cap = None
        
        # Try to open camera
        try:
            cap = cv2.VideoCapture(self.camera_id)
            if not cap.isOpened():
                print("⚠️  Camera not available, running in demo mode")
                cap = None
            else:
                print("📹 Camera opened successfully")
                cap.set(cv2.CAP_PROP_FRAME_WIDTH, 640)
                cap.set(cv2.CAP_PROP_FRAME_HEIGHT, 480)
        except Exception as e:
            print(f"Camera initialization failed: {e}")
            cap = None
        
        frame_count = 0
        face_detection_count = 0
        
        while self.processing:
            gaze_point = None
            
            # Try to get camera frame
            if cap is not None:
                ret, frame = cap.read()
                if ret:
                    frame_count += 1
                    
                    # Process frame for face detection
                    rgb_frame = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
                    results = self.face_mesh.process(rgb_frame)
                    
                    if results.multi_face_landmarks:
                        face_detection_count += 1
                        
                        # Extract landmarks
                        landmarks = []
                        for landmark in results.multi_face_landmarks[0].landmark:
                            landmarks.append([landmark.x, landmark.y])
                        
                        landmarks = np.array(landmarks)
                        
                        # Estimate gaze
                        gaze_point = self.estimate_gaze_simple(landmarks)
                else:
                    # Camera failed, fall back to demo
                    cap.release()
                    cap = None
            
            # Generate demo gaze data if no camera or no face detected
            if gaze_point is None:
                frame_count += 1
                
                # Create smooth demo movement starting from PRIMARY monitor
                demo_time = time.time() * 0.3
                
                # Base movement around laptop screen center (where camera is)
                laptop_center_x = self.monitor_mesh.primary_monitor.center_x if self.monitor_mesh.primary_monitor else 0
                laptop_center_y = self.monitor_mesh.primary_monitor.center_y if self.monitor_mesh.primary_monitor else 0
                
                # Create a more natural movement pattern within and between monitors
                demo_range_x = self.monitor_mesh.virtual_width * 0.6  # Use 60% of total width
                demo_range_y = self.monitor_mesh.virtual_height * 0.4  # Use 40% of total height
                
                demo_x = laptop_center_x + np.sin(demo_time) * demo_range_x * 0.5
                demo_y = laptop_center_y + np.sin(demo_time * 0.7) * demo_range_y * 0.5
                
                # Ensure demo coordinates stay within bounds
                demo_x = max(self.monitor_mesh.virtual_left, 
                           min(demo_x, self.monitor_mesh.virtual_right - 1))
                demo_y = max(self.monitor_mesh.virtual_top,
                           min(demo_y, self.monitor_mesh.virtual_bottom - 1))
                
                gaze_point = GazePoint(
                    x=float(demo_x),
                    y=float(demo_y),
                    confidence=0.75,
                    timestamp=time.time()
                )
            
            # Update the gaze window
            try:
                gaze_window.update_gaze(gaze_point)
            except Exception as e:
                print(f"Window update failed: {e}")
                break
            
            # Update FPS
            self.fps_counter += 1
            if time.time() - self.fps_start_time >= 1.0:
                self.current_fps = self.fps_counter
                face_rate = (face_detection_count / frame_count * 100) if frame_count > 0 else 0
                print(f"📊 FPS: {self.current_fps:.1f}, Face detection: {face_rate:.1f}%")
                self.fps_counter = 0
                self.fps_start_time = time.time()
            
            # Small delay
            time.sleep(0.03)  # ~30 FPS
        
        # Cleanup
        if cap is not None:
            cap.release()
        
        print("👁️  Eye tracking stopped")
    
    def stop(self):
        """Stop the tracking loop"""
        self.processing = False

def main():
    """Main function"""
    parser = argparse.ArgumentParser(description='Gaze-Controlled GUI Demo')
    parser.add_argument('--camera', type=int, default=0, help='Camera device ID')
    parser.add_argument('--demo-only', action='store_true', help='Run in demo mode without camera')
    parser.add_argument('--force-laptop-size', type=str, help='Force laptop screen by resolution (e.g., "1280x720")')
    
    args = parser.parse_args()
    
    print("🎯 Starting Gaze-Controlled GUI Demo")
    print("=" * 50)
    
    # Detect monitor configuration
    print("🔍 Detecting monitor configuration...")
    monitor_mesh = detect_monitor_mesh()
    
    # Manual override for laptop screen if detection fails
    if args.force_laptop_size:
        try:
            forced_width, forced_height = map(int, args.force_laptop_size.split('x'))
            print(f"🔧 MANUAL OVERRIDE: Looking for laptop screen with resolution {forced_width}x{forced_height}")
            
            # Find monitor with exact resolution
            forced_laptop = None
            for monitor in monitor_mesh.monitors:
                if monitor.width == forced_width and monitor.height == forced_height:
                    forced_laptop = monitor
                    break
            
            if forced_laptop:
                # Reset all monitors
                for monitor in monitor_mesh.monitors:
                    monitor.is_primary = False
                
                # Force this monitor as laptop screen
                forced_laptop.is_primary = True
                forced_laptop.name = "Laptop_Screen_FORCED"
                monitor_mesh.primary_monitor = forced_laptop
                
                print(f"✅ FORCED LAPTOP SCREEN: {forced_laptop.name} at ({forced_laptop.x}, {forced_laptop.y})")
            else:
                print(f"❌ Could not find monitor with resolution {forced_width}x{forced_height}")
                
        except ValueError:
            print(f"❌ Invalid resolution format. Use format like: --force-laptop-size 1280x720")
    
    print(f"\n📐 Virtual Desktop Setup:")
    print(f"   Size: {monitor_mesh.virtual_width} x {monitor_mesh.virtual_height}")
    print(f"   Bounds: ({monitor_mesh.virtual_left}, {monitor_mesh.virtual_top}) to ({monitor_mesh.virtual_right}, {monitor_mesh.virtual_bottom})")
    print(f"   Monitors: {len(monitor_mesh.monitors)}")
    
    for i, monitor in enumerate(monitor_mesh.monitors, 1):
        camera_indicator = "📹" if monitor.is_primary else "🖥️ "
        print(f"   {i}. {monitor.name}: {monitor.width}x{monitor.height} at ({monitor.x}, {monitor.y}) {camera_indicator}")
    
    print("\n🎮 Controls:")
    print("   • Look around to move the window across monitors")
    print("   • Close the window or press Ctrl+C to exit")
    print("   • Window will show gaze coordinates and confidence")
    print("   • Use --force-laptop-size WIDTHxHEIGHT to manually set laptop screen")
    
    print("\n🚀 Starting demo...")
    
    try:
        # Create the gaze-controlled window
        gaze_window = GazeControlledWindow(monitor_mesh)
        
        # Create eye tracker
        camera_id = -1 if args.demo_only else args.camera
        eye_tracker = SimpleEyeTracker(monitor_mesh, camera_id)
        
        # Start eye tracking in a separate thread
        tracking_thread = threading.Thread(
            target=eye_tracker.run_tracking,
            args=(gaze_window,),
            daemon=True
        )
        tracking_thread.start()
        
        # Start the GUI (this blocks until window is closed)
        gaze_window.start()
        
    except KeyboardInterrupt:
        print("\n🛑 Demo interrupted")
    except Exception as e:
        print(f"❌ Error: {e}")
        import traceback
        traceback.print_exc()
    finally:
        # Cleanup
        try:
            eye_tracker.stop()
            gaze_window.destroy()
        except:
            pass
        
        print("👋 Demo ended")

if __name__ == "__main__":
    main()

# Full demo with camera
# python gaze-ml-test.py --camera 0

# Demo mode without camera (smooth figure-8 movement)
# python gaze-ml-test.py --demo-only