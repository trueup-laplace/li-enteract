#!/usr/bin/env python3
"""
Standalone Gaze-Controlled Window Movement System
Includes comprehensive monitor calibration and dynamic window positioning
"""

import cv2
import numpy as np
import mediapipe as mp
import tkinter as tk
from tkinter import ttk, messagebox
import platform
import ctypes
import time
import threading
from dataclasses import dataclass, asdict
from typing import List, Tuple, Optional, Dict
from collections import deque
import math

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

def detect_monitor_mesh() -> MonitorMesh:
    """Detect monitor configuration using OS APIs"""
    monitors = []
    
    if platform.system() == "Windows":
        try:
            from ctypes import wintypes, Structure, POINTER, WINFUNCTYPE
            
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
            MonitorEnumProc = WINFUNCTYPE(ctypes.c_bool, wintypes.HMONITOR, wintypes.HDC, POINTER(RECT), wintypes.LPARAM)
            
            def monitor_enum_callback(hmonitor, hdc, rect, data):
                try:
                    monitor_info = MONITORINFO()
                    monitor_info.cbSize = ctypes.sizeof(MONITORINFO)
                    
                    if user32.GetMonitorInfoW(hmonitor, ctypes.byref(monitor_info)):
                        rect = monitor_info.rcMonitor
                        is_primary = bool(monitor_info.dwFlags & 1)
                        
                        monitor = Monitor(
                            x=rect.left,
                            y=rect.top,
                            width=rect.right - rect.left,
                            height=rect.bottom - rect.top,
                            is_primary=is_primary,
                            name=f'Monitor_{len(monitors) + 1}'
                        )
                        monitors.append(monitor)
                except Exception as e:
                    print(f"Error in monitor callback: {e}")
                return True
            
            user32.EnumDisplayMonitors(None, None, MonitorEnumProc(monitor_enum_callback), 0)
            
        except Exception as e:
            print(f"Windows monitor detection failed: {e}")
    
    elif platform.system() == "Darwin":  # macOS
        try:
            import subprocess
            import json
            
            # Use system_profiler to get display info
            result = subprocess.run(['system_profiler', 'SPDisplaysDataType', '-json'], 
                                  capture_output=True, text=True)
            if result.returncode == 0:
                data = json.loads(result.stdout)
                display_data = data.get('SPDisplaysDataType', [])
                
                for i, display in enumerate(display_data):
                    if 'spdisplays_ndrvs' in display:
                        for j, screen in enumerate(display['spdisplays_ndrvs']):
                            resolution = screen.get('_spdisplays_resolution', '1920 x 1080')
                            width, height = map(int, resolution.split(' x '))
                            
                            monitor = Monitor(
                                x=i * width,  # Simple horizontal arrangement
                                y=0,
                                width=width,
                                height=height,
                                is_primary=(i == 0),
                                name=f'Display_{i+1}'
                            )
                            monitors.append(monitor)
        except Exception as e:
            print(f"macOS monitor detection failed: {e}")
    
    elif platform.system() == "Linux":
        try:
            import subprocess
            
            # Try xrandr first
            result = subprocess.run(['xrandr', '--query'], capture_output=True, text=True)
            if result.returncode == 0:
                lines = result.stdout.split('\n')
                for line in lines:
                    if ' connected ' in line and 'x' in line:
                        parts = line.split()
                        if len(parts) >= 3:
                            resolution_part = None
                            position_part = None
                            
                            for part in parts:
                                if 'x' in part and '+' in part:
                                    resolution_part = part
                                    break
                            
                            if resolution_part:
                                try:
                                    res_pos = resolution_part.split('+')
                                    width_height = res_pos[0].split('x')
                                    width = int(width_height[0])
                                    height = int(width_height[1])
                                    x = int(res_pos[1]) if len(res_pos) > 1 else 0
                                    y = int(res_pos[2]) if len(res_pos) > 2 else 0
                                    
                                    monitor = Monitor(
                                        x=x, y=y, width=width, height=height,
                                        is_primary='primary' in line,
                                        name=parts[0]
                                    )
                                    monitors.append(monitor)
                                except ValueError:
                                    continue
        except Exception as e:
            print(f"Linux monitor detection failed: {e}")
    
    # Fallback to single monitor
    if not monitors:
        root = tk.Tk()
        width = root.winfo_screenwidth()
        height = root.winfo_screenheight()
        root.destroy()
        
        monitor = Monitor(x=0, y=0, width=width, height=height, is_primary=True, name="Default_Monitor")
        monitors = [monitor]
    
    # Calculate virtual desktop bounds
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
        
        return mesh
    
    return MonitorMesh(monitors=[], virtual_width=1920, virtual_height=1080, virtual_left=0, virtual_top=0)

class CalibrationWindow:
    """Interactive calibration window for screen layout and gaze mapping"""
    
    def __init__(self, monitor_mesh: MonitorMesh):
        self.monitor_mesh = monitor_mesh
        self.calibration_points = []
        self.current_target = 0
        self.gaze_tracker = None
        self.calibration_complete = False
        
        # Create calibration targets (corners and center of each monitor)
        self.targets = []
        for monitor in monitor_mesh.monitors:
            # Add 5 points per monitor: 4 corners + center
            points = [
                (monitor.x + 50, monitor.y + 50),  # Top-left
                (monitor.right - 50, monitor.y + 50),  # Top-right
                (monitor.x + 50, monitor.bottom - 50),  # Bottom-left
                (monitor.right - 50, monitor.bottom - 50),  # Bottom-right
                (monitor.center_x, monitor.center_y),  # Center
            ]
            self.targets.extend(points)
        
        self.setup_ui()
    
    def setup_ui(self):
        """Setup calibration UI"""
        self.root = tk.Tk()
        self.root.title("Gaze Calibration")
        self.root.geometry("400x300")
        self.root.resizable(False, False)
        
        # Center the window
        self.root.update_idletasks()
        x = (self.root.winfo_screenwidth() // 2) - (400 // 2)
        y = (self.root.winfo_screenheight() // 2) - (300 // 2)
        self.root.geometry(f"400x300+{x}+{y}")
        
        main_frame = ttk.Frame(self.root, padding="20")
        main_frame.grid(row=0, column=0, sticky=(tk.W, tk.E, tk.N, tk.S))
        
        # Title
        title_label = ttk.Label(main_frame, text="Gaze Calibration", font=("Arial", 16, "bold"))
        title_label.grid(row=0, column=0, columnspan=2, pady=(0, 20))
        
        # Monitor info
        monitor_info = ttk.Label(main_frame, text=f"Detected {len(self.monitor_mesh.monitors)} monitor(s)")
        monitor_info.grid(row=1, column=0, columnspan=2, pady=(0, 10))
        
        # Monitor details
        details_frame = ttk.LabelFrame(main_frame, text="Monitor Layout", padding="10")
        details_frame.grid(row=2, column=0, columnspan=2, sticky=(tk.W, tk.E), pady=(0, 20))
        
        for i, monitor in enumerate(self.monitor_mesh.monitors):
            detail_text = f"{monitor.name}: {monitor.width}x{monitor.height} at ({monitor.x}, {monitor.y})"
            if monitor.is_primary:
                detail_text += " [PRIMARY]"
            
            detail_label = ttk.Label(details_frame, text=detail_text, font=("Courier", 9))
            detail_label.grid(row=i, column=0, sticky=tk.W)
        
        # Instructions
        instructions = ttk.Label(main_frame, 
            text="Click 'Start Calibration' to begin.\nLook at each target that appears and press SPACE.",
            justify=tk.CENTER)
        instructions.grid(row=3, column=0, columnspan=2, pady=(0, 20))
        
        # Buttons
        button_frame = ttk.Frame(main_frame)
        button_frame.grid(row=4, column=0, columnspan=2)
        
        self.start_button = ttk.Button(button_frame, text="Start Calibration", command=self.start_calibration)
        self.start_button.grid(row=0, column=0, padx=(0, 10))
        
        self.skip_button = ttk.Button(button_frame, text="Skip Calibration", command=self.skip_calibration)
        self.skip_button.grid(row=0, column=1)
        
        # Status
        self.status_label = ttk.Label(main_frame, text="Ready to calibrate", foreground="blue")
        self.status_label.grid(row=5, column=0, columnspan=2, pady=(20, 0))
    
    def start_calibration(self):
        """Start the calibration process"""
        self.start_button.config(state="disabled")
        self.skip_button.config(state="disabled")
        self.status_label.config(text="Starting calibration...", foreground="orange")
        self.root.update()
        
        # Hide main window and show calibration targets
        self.root.withdraw()
        self.show_calibration_target()
    
    def show_calibration_target(self):
        """Show calibration target window"""
        if self.current_target >= len(self.targets):
            self.finish_calibration()
            return
        
        target_x, target_y = self.targets[self.current_target]
        
        # Create target window
        target_window = tk.Toplevel()
        target_window.title("Calibration Target")
        target_window.geometry(f"200x200+{target_x-100}+{target_y-100}")
        target_window.configure(bg='black')
        target_window.attributes('-topmost', True)
        target_window.focus_force()
        
        # Create target circle
        canvas = tk.Canvas(target_window, width=200, height=200, bg='black', highlightthickness=0)
        canvas.pack()
        
        # Draw concentric circles for target
        canvas.create_oval(70, 70, 130, 130, outline='white', width=3, fill='red')
        canvas.create_oval(85, 85, 115, 115, outline='white', width=2, fill='white')
        canvas.create_oval(95, 95, 105, 105, outline='black', width=1, fill='black')
        
        # Instructions
        instruction_text = f"Look at target {self.current_target + 1}/{len(self.targets)}\nPress SPACE when ready"
        canvas.create_text(100, 30, text=instruction_text, fill='white', font=("Arial", 10), justify=tk.CENTER)
        
        def on_space(event):
            self.calibration_points.append((target_x, target_y))
            target_window.destroy()
            self.current_target += 1
            self.show_calibration_target()
        
        def on_escape(event):
            target_window.destroy()
            self.finish_calibration()
        
        target_window.bind('<KeyPress-space>', on_space)
        target_window.bind('<KeyPress-Escape>', on_escape)
        target_window.focus_set()
    
    def skip_calibration(self):
        """Skip calibration and use default settings"""
        self.calibration_complete = True
        self.root.destroy()
    
    def finish_calibration(self):
        """Finish calibration process"""
        self.calibration_complete = True
        self.root.deiconify()
        self.status_label.config(text=f"Calibration complete! ({len(self.calibration_points)} points)", 
                               foreground="green")
        
        # Add close button
        close_button = ttk.Button(self.root, text="Close", command=self.root.destroy)
        close_button.pack(pady=10)
        
        self.root.after(2000, self.root.destroy)  # Auto-close after 2 seconds
    
    def run(self):
        """Run calibration window"""
        self.root.mainloop()
        return self.calibration_complete, self.calibration_points

class GazeTracker:
    """Gaze tracking using MediaPipe"""
    
    def __init__(self, monitor_mesh: MonitorMesh, camera_id: int = 0):
        self.monitor_mesh = monitor_mesh
        self.camera_id = camera_id
        
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
        
        # Gaze smoothing
        self.gaze_history = deque(maxlen=5)
        
        # Camera setup
        self.cap = None
        self.initialize_camera()
    
    def initialize_camera(self):
        """Initialize camera"""
        try:
            self.cap = cv2.VideoCapture(self.camera_id)
            if self.cap.isOpened():
                self.cap.set(cv2.CAP_PROP_FRAME_WIDTH, 640)
                self.cap.set(cv2.CAP_PROP_FRAME_HEIGHT, 480)
                self.cap.set(cv2.CAP_PROP_FPS, 30)
                return True
        except Exception as e:
            print(f"Camera initialization failed: {e}")
        return False
    
    def estimate_gaze(self, landmarks) -> Optional[Tuple[float, float]]:
        """Estimate gaze direction from facial landmarks"""
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
            
            # Map to screen coordinates with enhanced sensitivity
            sensitivity_x = 2.0
            sensitivity_y = 1.8
            
            # Invert X for natural mapping and apply sensitivity
            screen_x = (1.0 - avg_iris_x) * sensitivity_x - (sensitivity_x - 1.0) / 2
            screen_y = avg_iris_y * sensitivity_y - (sensitivity_y - 1.0) / 2
            
            # Clamp to valid range
            screen_x = max(0.0, min(1.0, screen_x))
            screen_y = max(0.0, min(1.0, screen_y))
            
            # Convert to absolute screen coordinates
            abs_x = self.monitor_mesh.virtual_left + (screen_x * self.monitor_mesh.virtual_width)
            abs_y = self.monitor_mesh.virtual_top + (screen_y * self.monitor_mesh.virtual_height)
            
            return (abs_x, abs_y)
            
        except Exception as e:
            return None
    
    def get_gaze_point(self) -> Optional[Tuple[float, float]]:
        """Get current gaze point"""
        if not self.cap or not self.cap.isOpened():
            return None
        
        ret, frame = self.cap.read()
        if not ret:
            return None
        
        # Process frame
        rgb_frame = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
        results = self.face_mesh.process(rgb_frame)
        
        if results.multi_face_landmarks:
            # Extract landmarks
            landmarks = []
            for landmark in results.multi_face_landmarks[0].landmark:
                landmarks.append([landmark.x, landmark.y])
            
            landmarks = np.array(landmarks)
            
            # Estimate gaze
            gaze_point = self.estimate_gaze(landmarks)
            if gaze_point:
                # Apply smoothing
                self.gaze_history.append(gaze_point)
                if len(self.gaze_history) >= 2:
                    # Simple moving average
                    avg_x = sum(gp[0] for gp in self.gaze_history) / len(self.gaze_history)
                    avg_y = sum(gp[1] for gp in self.gaze_history) / len(self.gaze_history)
                    return (avg_x, avg_y)
                else:
                    return gaze_point
        
        return None
    
    def cleanup(self):
        """Cleanup camera resources"""
        if self.cap:
            self.cap.release()

class GazeControlledWindow:
    """Main application window that follows gaze"""
    
    def __init__(self, monitor_mesh: MonitorMesh, gaze_tracker: GazeTracker):
        self.monitor_mesh = monitor_mesh
        self.gaze_tracker = gaze_tracker
        self.window_size = (300, 200)
        self.tracking_enabled = False
        
        self.setup_ui()
        self.update_position()
    
    def setup_ui(self):
        """Setup main application UI"""
        self.root = tk.Tk()
        self.root.title("Gaze-Controlled Window")
        self.root.geometry(f"{self.window_size[0]}x{self.window_size[1]}")
        self.root.configure(bg='lightblue')
        self.root.attributes('-topmost', True)
        
        # Main frame
        main_frame = ttk.Frame(self.root, padding="20")
        main_frame.pack(fill=tk.BOTH, expand=True)
        
        # Title
        title_label = ttk.Label(main_frame, text="👁️ Gaze Tracker", font=("Arial", 14, "bold"))
        title_label.pack(pady=(0, 10))
        
        # Status display
        self.status_label = ttk.Label(main_frame, text="Status: Ready", font=("Arial", 10))
        self.status_label.pack(pady=5)
        
        self.position_label = ttk.Label(main_frame, text="Position: (0, 0)", font=("Arial", 9))
        self.position_label.pack(pady=2)
        
        self.monitor_label = ttk.Label(main_frame, text="Monitor: Unknown", font=("Arial", 9))
        self.monitor_label.pack(pady=2)
        
        # Controls
        control_frame = ttk.Frame(main_frame)
        control_frame.pack(pady=10)
        
        self.toggle_button = ttk.Button(control_frame, text="Start Tracking", command=self.toggle_tracking)
        self.toggle_button.pack(side=tk.LEFT, padx=5)
        
        self.calibrate_button = ttk.Button(control_frame, text="Recalibrate", command=self.recalibrate)
        self.calibrate_button.pack(side=tk.LEFT, padx=5)
        
        # Window close binding
        self.root.protocol("WM_DELETE_WINDOW", self.on_closing)
    
    def toggle_tracking(self):
        """Toggle gaze tracking on/off"""
        self.tracking_enabled = not self.tracking_enabled
        if self.tracking_enabled:
            self.toggle_button.config(text="Stop Tracking")
            self.status_label.config(text="Status: Tracking Active")
        else:
            self.toggle_button.config(text="Start Tracking")
            self.status_label.config(text="Status: Tracking Paused")
    
    def recalibrate(self):
        """Restart calibration process"""
        self.tracking_enabled = False
        self.toggle_button.config(text="Start Tracking")
        
        calibration = CalibrationWindow(self.monitor_mesh)
        calibration_complete, points = calibration.run()
        
        if calibration_complete:
            messagebox.showinfo("Calibration", f"Calibration completed with {len(points)} points!")
        else:
            messagebox.showwarning("Calibration", "Calibration was skipped or cancelled.")
    
    def update_position(self):
        """Update window position based on gaze"""
        if self.tracking_enabled:
            gaze_point = self.gaze_tracker.get_gaze_point()
            if gaze_point:
                gaze_x, gaze_y = gaze_point
                
                # Position window relative to gaze point
                window_x = int(gaze_x - self.window_size[0] // 2)
                window_y = int(gaze_y - self.window_size[1] // 2)
                
                # Keep window within screen bounds
                window_x = max(self.monitor_mesh.virtual_left, 
                             min(window_x, self.monitor_mesh.virtual_right - self.window_size[0]))
                window_y = max(self.monitor_mesh.virtual_top,
                             min(window_y, self.monitor_mesh.virtual_bottom - self.window_size[1]))
                
                # Update window position
                self.root.geometry(f"{self.window_size[0]}x{self.window_size[1]}+{window_x}+{window_y}")
                
                # Update status labels
                self.position_label.config(text=f"Gaze: ({int(gaze_x)}, {int(gaze_y)})")
                
                # Determine which monitor
                current_monitor = self.monitor_mesh.get_monitor_at_point(gaze_x, gaze_y)
                if current_monitor:
                    self.monitor_label.config(text=f"Monitor: {current_monitor.name}")
                else:
                    self.monitor_label.config(text="Monitor: Outside bounds")
        
        # Schedule next update
        self.root.after(50, self.update_position)  # 20 FPS
    
    def on_closing(self):
        """Handle window closing"""
        self.gaze_tracker.cleanup()
        self.root.destroy()
    
    def run(self):
        """Run the main application"""
        self.root.mainloop()

def main():
    """Main application entry point"""
    print("🚀 Starting Standalone Gaze-Controlled Window System")
    
    # Detect monitor configuration
    print("🖥️  Detecting monitor configuration...")
    monitor_mesh = detect_monitor_mesh()
    
    print(f"📊 Detected {len(monitor_mesh.monitors)} monitor(s):")
    for monitor in monitor_mesh.monitors:
        print(f"   {monitor.name}: {monitor.width}x{monitor.height} at ({monitor.x}, {monitor.y}) {'[PRIMARY]' if monitor.is_primary else ''}")
    
    print(f"🌐 Virtual Desktop: {monitor_mesh.virtual_width}x{monitor_mesh.virtual_height} at ({monitor_mesh.virtual_left}, {monitor_mesh.virtual_top})")
    
    # Initialize gaze tracker
    print("👁️  Initializing gaze tracker...")
    gaze_tracker = GazeTracker(monitor_mesh)
    
    if not gaze_tracker.cap or not gaze_tracker.cap.isOpened():
        print("❌ Failed to initialize camera. Using demo mode.")
        # Could implement demo mode here
    else:
        print("📹 Camera initialized successfully")
    
    # Run calibration
    print("🎯 Starting calibration process...")
    calibration = CalibrationWindow(monitor_mesh)
    calibration_complete, calibration_points = calibration.run()
    
    if calibration_complete:
        print(f"✅ Calibration completed with {len(calibration_points)} points")
    else:
        print("⚠️  Calibration skipped")
    
    # Start main application
    print("🏃 Starting gaze-controlled window...")
    app = GazeControlledWindow(monitor_mesh, gaze_tracker)
    
    try:
        app.run()
    except KeyboardInterrupt:
        print("\n🛑 Shutting down...")
    finally:
        gaze_tracker.cleanup()
        print("👋 Application ended")

if __name__ == "__main__":
    main()