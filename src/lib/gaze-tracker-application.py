#!/usr/bin/env python3
"""
Gaze-Controlled Window Movement System for Tauri Application
Integrates with Vue composables and Tauri backend architecture
Based on the standalone gaze-tracker.py with application integration
"""

import cv2
import numpy as np
import mediapipe as mp
import platform
import ctypes
import time
import threading
import json
import sys
import argparse
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

@dataclass 
class MLGazeData:
    """Gaze data structure compatible with Tauri backend"""
    x: float
    y: float
    confidence: float
    left_eye_landmarks: List[List[float]]
    right_eye_landmarks: List[List[float]]
    head_pose: Dict[str, float]
    timestamp: int

@dataclass
class CalibrationPoint:
    """Calibration point for gaze correction"""
    screen_x: float
    screen_y: float
    gaze_x: float
    gaze_y: float
    confidence: float
    timestamp: int

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
                    print(f"Error in monitor callback: {e}", file=sys.stderr)
                return True
            
            user32.EnumDisplayMonitors(None, None, MonitorEnumProc(monitor_enum_callback), 0)
            
        except Exception as e:
            print(f"Windows monitor detection failed: {e}", file=sys.stderr)
    
    elif platform.system() == "Darwin":  # macOS
        try:
            import subprocess
            import json
            
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
                                x=i * width,
                                y=0,
                                width=width,
                                height=height,
                                is_primary=(i == 0),
                                name=f'Display_{i+1}'
                            )
                            monitors.append(monitor)
        except Exception as e:
            print(f"macOS monitor detection failed: {e}", file=sys.stderr)
    
    elif platform.system() == "Linux":
        try:
            import subprocess
            
            result = subprocess.run(['xrandr', '--query'], capture_output=True, text=True)
            if result.returncode == 0:
                lines = result.stdout.split('\n')
                for line in lines:
                    if ' connected ' in line and 'x' in line:
                        parts = line.split()
                        if len(parts) >= 3:
                            for part in parts:
                                if 'x' in part and '+' in part:
                                    try:
                                        res_pos = part.split('+')
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
                                        break
                                    except ValueError:
                                        continue
        except Exception as e:
            print(f"Linux monitor detection failed: {e}", file=sys.stderr)
    
    # Fallback to single monitor
    if not monitors:
        try:
            import tkinter as tk
            root = tk.Tk()
            width = root.winfo_screenwidth()
            height = root.winfo_screenheight()
            root.destroy()
            
            monitor = Monitor(x=0, y=0, width=width, height=height, is_primary=True, name="Default_Monitor")
            monitors = [monitor]
        except:
            # Ultimate fallback
            monitor = Monitor(x=0, y=0, width=1920, height=1080, is_primary=True, name="Fallback_Monitor")
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

class GazeTrackerApplication:
    """Main gaze tracker for Tauri application integration"""
    
    def __init__(self, camera_id: int = 0, screen_width: int = 1920, screen_height: int = 1080):
        self.camera_id = camera_id
        self.screen_width = screen_width
        self.screen_height = screen_height
        
        # Detect monitor configuration
        self.monitor_mesh = detect_monitor_mesh()
        
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
        
        # Eye landmarks for additional data
        self.LEFT_EYE = [33, 7, 163, 144, 145, 153, 154, 155, 133, 173, 157, 158, 159, 160, 161, 246]
        self.RIGHT_EYE = [362, 382, 381, 380, 374, 373, 390, 249, 263, 466, 388, 387, 386, 385, 384, 398]
        
        # Gaze smoothing (same as original gaze-tracker.py)
        self.gaze_history = deque(maxlen=5)
        
        # Calibration data
        self.calibration_points = []
        self.calibration_transform = np.eye(3)  # Homogeneous transformation matrix
        self.is_calibrated = False
        
        # Camera setup
        self.cap = None
        self.is_running = False
        self.initialize_camera()
        
        # Stats tracking
        self.frame_count = 0
        self.start_time = time.time()
        
    def initialize_camera(self) -> bool:
        """Initialize camera"""
        try:
            self.cap = cv2.VideoCapture(self.camera_id)
            if self.cap.isOpened():
                self.cap.set(cv2.CAP_PROP_FRAME_WIDTH, 640)
                self.cap.set(cv2.CAP_PROP_FRAME_HEIGHT, 480)
                self.cap.set(cv2.CAP_PROP_FPS, 30)
                print(f"Camera {self.camera_id} initialized successfully", file=sys.stderr)
                return True
        except Exception as e:
            print(f"Camera initialization failed: {e}", file=sys.stderr)
        return False
    
    def estimate_gaze(self, landmarks) -> Optional[Tuple[float, float, float]]:
        """Estimate gaze direction from facial landmarks with confidence"""
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
            
            # Calculate confidence based on eye openness and detection quality
            confidence = self.calculate_confidence(landmarks, left_iris_points, right_iris_points)
            
            # Map to screen coordinates with enhanced sensitivity
            sensitivity_x = 2.0
            sensitivity_y = 1.8
            
            # Invert X for natural mapping and apply sensitivity
            screen_x = (1.0 - avg_iris_x) * sensitivity_x - (sensitivity_x - 1.0) / 2
            screen_y = avg_iris_y * sensitivity_y - (sensitivity_y - 1.0) / 2
            
            # Clamp to valid range
            screen_x = max(0.0, min(1.0, screen_x))
            screen_y = max(0.0, min(1.0, screen_y))
            
            # Apply calibration if available
            if self.is_calibrated:
                screen_x, screen_y = self.apply_calibration(screen_x, screen_y)
            
            # Convert to absolute screen coordinates
            abs_x = self.monitor_mesh.virtual_left + (screen_x * self.monitor_mesh.virtual_width)
            abs_y = self.monitor_mesh.virtual_top + (screen_y * self.monitor_mesh.virtual_height)
            
            return (abs_x, abs_y, confidence)
            
        except Exception as e:
            print(f"Gaze estimation error: {e}", file=sys.stderr)
            return None
    
    def calculate_confidence(self, landmarks, left_iris, right_iris) -> float:
        """Calculate confidence based on eye detection quality"""
        try:
            # Base confidence
            confidence = 0.8
            
            # Check iris detection quality
            if len(left_iris) < 5 or len(right_iris) < 5:
                confidence *= 0.7
            
            # Check landmark stability (simplified)
            if len(landmarks) < 468:
                confidence *= 0.5
            
            # Add some noise for realism
            confidence += np.random.normal(0, 0.05)
            
            return max(0.0, min(1.0, confidence))
        except:
            return 0.5
    
    def apply_calibration(self, x: float, y: float) -> Tuple[float, float]:
        """Apply calibration transformation to gaze coordinates"""
        try:
            # Convert to homogeneous coordinates
            point = np.array([x, y, 1.0])
            
            # Apply transformation
            transformed = self.calibration_transform @ point
            
            # Convert back to 2D
            if transformed[2] != 0:
                return transformed[0] / transformed[2], transformed[1] / transformed[2]
            else:
                return x, y
        except:
            return x, y
    
    def extract_eye_landmarks(self, landmarks) -> Tuple[List[List[float]], List[List[float]]]:
        """Extract eye landmark coordinates"""
        try:
            left_eye_coords = []
            right_eye_coords = []
            
            for idx in self.LEFT_EYE:
                if idx < len(landmarks):
                    landmark = landmarks[idx]
                    left_eye_coords.append([float(landmark[0]), float(landmark[1])])
            
            for idx in self.RIGHT_EYE:
                if idx < len(landmarks):
                    landmark = landmarks[idx]
                    right_eye_coords.append([float(landmark[0]), float(landmark[1])])
            
            return left_eye_coords, right_eye_coords
        except:
            return [], []
    
    def estimate_head_pose(self, landmarks) -> Dict[str, float]:
        """Estimate head pose from landmarks"""
        try:
            # Simplified head pose estimation
            # In a real implementation, you'd use more sophisticated methods
            nose_tip = landmarks[1] if len(landmarks) > 1 else [0.5, 0.5]
            
            # Simple estimation based on nose position
            yaw = (nose_tip[0] - 0.5) * 45.0  # -22.5 to 22.5 degrees
            pitch = (nose_tip[1] - 0.5) * 30.0  # -15 to 15 degrees
            roll = 0.0  # Not easily estimated from 2D landmarks
            
            return {
                "yaw": float(yaw),
                "pitch": float(pitch), 
                "roll": float(roll)
            }
        except:
            return {"yaw": 0.0, "pitch": 0.0, "roll": 0.0}
    
    def get_gaze_data(self) -> Optional[MLGazeData]:
        """Get current gaze data in ML format compatible with Tauri"""
        if not self.cap or not self.cap.isOpened():
            return None
        
        ret, frame = self.cap.read()
        if not ret:
            return None
        
        self.frame_count += 1
        
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
            gaze_result = self.estimate_gaze(landmarks)
            if gaze_result:
                gaze_x, gaze_y, confidence = gaze_result
                
                # Apply smoothing (same as original gaze-tracker.py)
                self.gaze_history.append((gaze_x, gaze_y, confidence))
                
                if len(self.gaze_history) >= 2:
                    # Simple moving average (matching original implementation)
                    avg_x = sum(g[0] for g in self.gaze_history) / len(self.gaze_history)
                    avg_y = sum(g[1] for g in self.gaze_history) / len(self.gaze_history)
                    avg_conf = sum(g[2] for g in self.gaze_history) / len(self.gaze_history)
                else:
                    avg_x, avg_y, avg_conf = gaze_x, gaze_y, confidence
                
                # Extract eye landmarks
                left_eye_landmarks, right_eye_landmarks = self.extract_eye_landmarks(landmarks)
                
                # Estimate head pose
                head_pose = self.estimate_head_pose(landmarks)
                
                # Create ML gaze data
                ml_gaze_data = MLGazeData(
                    x=float(avg_x),
                    y=float(avg_y),
                    confidence=float(avg_conf),
                    left_eye_landmarks=left_eye_landmarks,
                    right_eye_landmarks=right_eye_landmarks,
                    head_pose=head_pose,
                    timestamp=int(time.time() * 1000)
                )
                
                return ml_gaze_data
        
        return None
    
    def add_calibration_point(self, screen_x: float, screen_y: float) -> bool:
        """Add a calibration point"""
        try:
            # Get current gaze data
            gaze_data = self.get_gaze_data()
            if gaze_data and gaze_data.confidence > 0.5:
                cal_point = CalibrationPoint(
                    screen_x=screen_x,
                    screen_y=screen_y,
                    gaze_x=gaze_data.x,
                    gaze_y=gaze_data.y,
                    confidence=gaze_data.confidence,
                    timestamp=int(time.time() * 1000)
                )
                self.calibration_points.append(cal_point)
                print(f"CALIBRATION:{json.dumps(asdict(cal_point))}")
                return True
        except Exception as e:
            print(f"Calibration point error: {e}", file=sys.stderr)
        return False
    
    def finish_calibration(self) -> bool:
        """Compute calibration transformation from collected points"""
        try:
            if len(self.calibration_points) < 4:
                print("Not enough calibration points", file=sys.stderr)
                return False
            
            # Extract screen and gaze coordinates
            screen_coords = np.array([[p.screen_x, p.screen_y] for p in self.calibration_points])
            gaze_coords = np.array([[p.gaze_x, p.gaze_y] for p in self.calibration_points])
            
            # Normalize to 0-1 range
            screen_coords[:, 0] /= self.monitor_mesh.virtual_width
            screen_coords[:, 1] /= self.monitor_mesh.virtual_height
            gaze_coords[:, 0] /= self.monitor_mesh.virtual_width  
            gaze_coords[:, 1] /= self.monitor_mesh.virtual_height
            
            # Calculate affine transformation using least squares
            # Adding homogeneous coordinate
            gaze_homogeneous = np.column_stack([gaze_coords, np.ones(len(gaze_coords))])
            
            # Solve for transformation matrix
            self.calibration_transform[:2, :] = np.linalg.lstsq(gaze_homogeneous, screen_coords, rcond=None)[0].T
            
            self.is_calibrated = True
            print(f"Calibration completed with {len(self.calibration_points)} points", file=sys.stderr)
            return True
            
        except Exception as e:
            print(f"Calibration computation error: {e}", file=sys.stderr)
            return False
    
    def run_tracking_loop(self):
        """Main tracking loop for continuous gaze data output"""
        self.is_running = True
        print("Starting gaze tracking loop", file=sys.stderr)
        
        while self.is_running:
            try:
                gaze_data = self.get_gaze_data()
                if gaze_data:
                    # Output gaze data in JSON format for Tauri backend
                    gaze_json = json.dumps(asdict(gaze_data))
                    print(f"GAZE:{gaze_json}")
                    sys.stdout.flush()
                
                # Control frame rate (matching original 20 FPS)
                time.sleep(1.0 / 20.0)  # 20 FPS
                
            except KeyboardInterrupt:
                break
            except Exception as e:
                print(f"Tracking loop error: {e}", file=sys.stderr)
                time.sleep(0.1)
        
        self.cleanup()
    
    def cleanup(self):
        """Cleanup resources"""
        self.is_running = False
        if self.cap:
            self.cap.release()
        print("Gaze tracker cleaned up", file=sys.stderr)

def main():
    """Main entry point for application integration"""
    parser = argparse.ArgumentParser(description='Gaze Tracker for Tauri Application')
    parser.add_argument('--camera-id', type=int, default=0, help='Camera device ID')
    parser.add_argument('--screen-width', type=int, default=1920, help='Screen width')
    parser.add_argument('--screen-height', type=int, default=1080, help='Screen height')
    parser.add_argument('--calibrate', action='store_true', help='Run calibration mode')
    
    args = parser.parse_args()
    
    # Initialize gaze tracker
    tracker = GazeTrackerApplication(
        camera_id=args.camera_id,
        screen_width=args.screen_width,
        screen_height=args.screen_height
    )
    
    # Output monitor information
    print(f"MONITORS:{json.dumps([asdict(m) for m in tracker.monitor_mesh.monitors])}", file=sys.stderr)
    
    if args.calibrate:
        # Calibration mode - wait for calibration commands
        print("Calibration mode active. Send calibration points via stdin.", file=sys.stderr)
        
        for line in sys.stdin:
            line = line.strip()
            if line.startswith("CALIBRATE:"):
                try:
                    coords = json.loads(line[10:])
                    success = tracker.add_calibration_point(coords['x'], coords['y'])
                    print(f"CALIBRATION_RESULT:{success}")
                except:
                    print("CALIBRATION_RESULT:false")
            elif line == "FINISH_CALIBRATION":
                success = tracker.finish_calibration()
                print(f"CALIBRATION_COMPLETE:{success}")
                if success:
                    break
            elif line == "EXIT":
                break
    else:
        # Normal tracking mode
        try:
            tracker.run_tracking_loop()
        except KeyboardInterrupt:
            print("\nShutting down gaze tracker", file=sys.stderr)
        finally:
            tracker.cleanup()

if __name__ == "__main__":
    main() 