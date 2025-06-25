import cv2
import dlib
import numpy as np
import tkinter as tk
from tkinter import ttk, messagebox
import screeninfo
import threading
import time
import math
from collections import deque
import json
import os
from scipy.spatial import distance as dist

class ImprovedEyeTracker:
    def __init__(self):
        # Initialize dlib's face detector and facial landmark predictor
        self.detector = dlib.get_frontal_face_detector()
        
        try:
            predictor_path = os.path.join(os.path.dirname(__file__), "shape_predictor_68_face_landmarks.dat")
            self.predictor = dlib.shape_predictor(predictor_path)
        except:
            print("Warning: shape_predictor_68_face_landmarks.dat not found!")
            print("Download from: http://dlib.net/files/shape_predictor_68_face_landmarks.dat.bz2")
            self.predictor = None
        
        # Enhanced multi-monitor setup
        self.screens = self.get_screen_info()
        self.virtual_desktop = self.create_virtual_desktop()
        
        # Camera setup
        self.cap = None
        self.camera_running = False
        
        # Enhanced gaze tracking variables
        self.gaze_history = deque(maxlen=10)
        self.current_gaze = (0, 0)
        self.current_raw_gaze = (0, 0)
        
        # Improved calibration system
        self.calibration_points = []
        self.calibration_data = {}
        self.is_calibrated = False
        
        # Enhanced pupil detection parameters
        self.pupil_detection_method = 'improved'  # 'simple', 'improved', 'contour'
        self.debug_mode = True
        
        # GUI variables
        self.gui_window = None
        self.gui_size = 50
        self.gui_color = 'red'
        
        # Enhanced settings
        self.calibration_enabled = False
        self.smoothing_factor = 0.5
        
        # Eye tracking state
        self.left_eye_center = None
        self.right_eye_center = None
        self.face_center = None
        
        self.load_calibration()
    
    def get_screen_info(self):
        """Get information about all connected screens"""
        screens = []
        for i, monitor in enumerate(screeninfo.get_monitors()):
            screens.append({
                'id': i,
                'x': monitor.x,
                'y': monitor.y,
                'width': monitor.width,
                'height': monitor.height,
                'name': monitor.name if hasattr(monitor, 'name') else f"Monitor {i+1}",
                'right_edge': monitor.x + monitor.width,
                'bottom_edge': monitor.y + monitor.height
            })
        return sorted(screens, key=lambda s: (s['y'], s['x']))
    
    def create_virtual_desktop(self):
        """Create virtual desktop coordinate system"""
        if not self.screens:
            return {'width': 1920, 'height': 1080, 'min_x': 0, 'min_y': 0}
        
        min_x = min(screen['x'] for screen in self.screens)
        min_y = min(screen['y'] for screen in self.screens)
        max_x = max(screen['right_edge'] for screen in self.screens)
        max_y = max(screen['bottom_edge'] for screen in self.screens)
        
        return {
            'width': max_x - min_x,
            'height': max_y - min_y,
            'min_x': min_x,
            'min_y': min_y,
            'max_x': max_x,
            'max_y': max_y
        }
    
    def improved_pupil_detection(self, eye_points, frame):
        """Improved pupil detection using multiple techniques"""
        eye_region = np.array(eye_points, dtype=np.int32)
        
        # Create bounding box around eye
        x, y, w, h = cv2.boundingRect(eye_region)
        
        # Add padding to ensure we get the full eye
        padding = 5
        x = max(0, x - padding)
        y = max(0, y - padding)
        w = min(frame.shape[1] - x, w + 2*padding)
        h = min(frame.shape[0] - y, h + 2*padding)
        
        # Extract eye region
        eye_frame = frame[y:y+h, x:x+w]
        if eye_frame.size == 0:
            return None
        
        # Convert to grayscale
        eye_gray = cv2.cvtColor(eye_frame, cv2.COLOR_BGR2GRAY)
        
        # Apply Gaussian blur to reduce noise
        eye_gray = cv2.GaussianBlur(eye_gray, (7, 7), 0)
        
        # Method 1: Morphological operations to enhance pupil
        kernel = cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (3, 3))
        eye_gray = cv2.morphologyEx(eye_gray, cv2.MORPH_CLOSE, kernel)
        
        # Method 2: Threshold to isolate dark regions (pupil)
        _, threshold = cv2.threshold(eye_gray, 0, 255, cv2.THRESH_BINARY_INV + cv2.THRESH_OTSU)
        
        # Method 3: Find contours and select the most circular one
        contours, _ = cv2.findContours(threshold, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
        
        if contours:
            # Filter contours by area and circularity
            valid_contours = []
            for contour in contours:
                area = cv2.contourArea(contour)
                if area > 10:  # Minimum area threshold
                    # Calculate circularity
                    perimeter = cv2.arcLength(contour, True)
                    if perimeter > 0:
                        circularity = 4 * math.pi * area / (perimeter * perimeter)
                        if circularity > 0.3:  # Circularity threshold
                            valid_contours.append((contour, area, circularity))
            
            if valid_contours:
                # Sort by combined score (area * circularity)
                valid_contours.sort(key=lambda x: x[1] * x[2], reverse=True)
                best_contour = valid_contours[0][0]
                
                # Get centroid of best contour
                M = cv2.moments(best_contour)
                if M["m00"] != 0:
                    cx = int(M["m10"] / M["m00"]) + x
                    cy = int(M["m01"] / M["m00"]) + y
                    return (cx, cy)
        
        # Fallback: Use minimum value location with constraints
        # Create mask for eye region only
        mask = np.zeros(frame.shape[:2], dtype=np.uint8)
        cv2.fillPoly(mask, [eye_region], 255)
        
        # Apply mask to grayscale image
        gray_full = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)
        masked_eye = cv2.bitwise_and(gray_full, gray_full, mask=mask)
        
        # Find darkest point within the eye region
        min_val, max_val, min_loc, max_loc = cv2.minMaxLoc(masked_eye, mask=mask)
        
        return min_loc
    
    def calculate_gaze_vector(self, left_eye_center, right_eye_center, face_landmarks):
        """Calculate gaze direction using improved method with head pose estimation"""
        if not left_eye_center or not right_eye_center:
            return (0, 0)
        
        # Calculate average eye center
        avg_eye_x = (left_eye_center[0] + right_eye_center[0]) / 2
        avg_eye_y = (left_eye_center[1] + right_eye_center[1]) / 2
        
        # Get nose tip and nose bridge points for head pose reference
        nose_tip = (face_landmarks.part(30).x, face_landmarks.part(30).y)
        nose_bridge = (face_landmarks.part(27).x, face_landmarks.part(27).y)
        
        # Calculate head pose offset
        head_pose_x = nose_tip[0] - nose_bridge[0]
        head_pose_y = nose_tip[1] - nose_bridge[1]
        
        # Calculate inter-pupillary distance for normalization
        ipd = math.sqrt((right_eye_center[0] - left_eye_center[0])**2 + 
                       (right_eye_center[1] - left_eye_center[1])**2)
        
        if ipd == 0:
            return (0, 0)
        
        # Calculate gaze relative to head pose
        # This accounts for head rotation and position
        face_center_x = (face_landmarks.part(36).x + face_landmarks.part(45).x) / 2  # Between eye corners
        face_center_y = (face_landmarks.part(36).y + face_landmarks.part(45).y) / 2
        
        # Raw gaze vector
        raw_gaze_x = (avg_eye_x - face_center_x) / ipd  # Normalize by IPD
        raw_gaze_y = (avg_eye_y - face_center_y) / ipd
        
        # Adjust for head pose
        adjusted_gaze_x = raw_gaze_x - (head_pose_x / ipd) * 0.5
        adjusted_gaze_y = raw_gaze_y - (head_pose_y / ipd) * 0.5
        
        return (adjusted_gaze_x, adjusted_gaze_y)
    
    def map_gaze_to_virtual_desktop(self, gaze_x, gaze_y):
        """Enhanced mapping to virtual desktop coordinates"""
        if self.is_calibrated and self.calibration_enabled:
            return self.calibrated_virtual_mapping(gaze_x, gaze_y)
        else:
            return self.simple_virtual_mapping(gaze_x, gaze_y)
    
    def simple_virtual_mapping(self, gaze_x, gaze_y):
        """Improved simple mapping without calibration"""
        # Enhanced scaling based on virtual desktop size
        center_x = self.virtual_desktop['min_x'] + self.virtual_desktop['width'] // 2
        center_y = self.virtual_desktop['min_y'] + self.virtual_desktop['height'] // 2
        
        # Adaptive scaling factors based on screen resolution
        base_scale = min(self.virtual_desktop['width'], self.virtual_desktop['height'])
        scale_x = base_scale * 0.8  # Reduced from previous aggressive scaling
        scale_y = base_scale * 0.6
        
        virtual_x = center_x + (gaze_x * scale_x)
        virtual_y = center_y + (gaze_y * scale_y)
        
        # Clamp to virtual desktop bounds
        virtual_x = max(self.virtual_desktop['min_x'], 
                       min(self.virtual_desktop['max_x'] - 1, virtual_x))
        virtual_y = max(self.virtual_desktop['min_y'], 
                       min(self.virtual_desktop['max_y'] - 1, virtual_y))
        
        return int(virtual_x), int(virtual_y)
    
    def calibrated_virtual_mapping(self, gaze_x, gaze_y):
        """Use calibration data for accurate mapping"""
        if not self.calibration_data:
            return self.simple_virtual_mapping(gaze_x, gaze_y)
        
        # Apply polynomial transformation if available
        if 'polynomial_x' in self.calibration_data and 'polynomial_y' in self.calibration_data:
            poly_x = self.calibration_data['polynomial_x']
            poly_y = self.calibration_data['polynomial_y']
            
            virtual_x = np.polyval(poly_x, gaze_x)
            virtual_y = np.polyval(poly_y, gaze_y)
        else:
            # Fallback to linear mapping
            x_scale = self.calibration_data.get('x_scale', self.virtual_desktop['width'] * 0.8)
            y_scale = self.calibration_data.get('y_scale', self.virtual_desktop['height'] * 0.6)
            x_offset = self.calibration_data.get('x_offset', self.virtual_desktop['min_x'])
            y_offset = self.calibration_data.get('y_offset', self.virtual_desktop['min_y'])
            
            virtual_x = (gaze_x * x_scale) + x_offset
            virtual_y = (gaze_y * y_scale) + y_offset
        
        # Clamp to bounds
        virtual_x = max(self.virtual_desktop['min_x'], 
                       min(self.virtual_desktop['max_x'] - 1, virtual_x))
        virtual_y = max(self.virtual_desktop['min_y'], 
                       min(self.virtual_desktop['max_y'] - 1, virtual_y))
        
        return int(virtual_x), int(virtual_y)
    
    def process_frame(self):
        """Enhanced frame processing with improved pupil detection"""
        if not self.cap or not self.predictor:
            return None
        
        ret, frame = self.cap.read()
        if not ret:
            return None
        
        frame = cv2.flip(frame, 1)
        gray = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)
        
        # Enhance contrast for better detection
        gray = cv2.equalizeHist(gray)
        
        faces = self.detector(gray)
        
        for face in faces:
            landmarks = self.predictor(gray, face)
            
            # Extract eye landmark points
            left_eye_points = [(landmarks.part(i).x, landmarks.part(i).y) for i in range(36, 42)]
            right_eye_points = [(landmarks.part(i).x, landmarks.part(i).y) for i in range(42, 48)]
            
            # Improved pupil detection
            left_eye_center = self.improved_pupil_detection(left_eye_points, frame)
            right_eye_center = self.improved_pupil_detection(right_eye_points, frame)
            
            # Store for debugging
            self.left_eye_center = left_eye_center
            self.right_eye_center = right_eye_center
            
            if left_eye_center and right_eye_center:
                # Calculate improved gaze vector
                gaze_x, gaze_y = self.calculate_gaze_vector(left_eye_center, right_eye_center, landmarks)
                self.current_raw_gaze = (gaze_x, gaze_y)
                
                # Map to virtual desktop
                virtual_x, virtual_y = self.map_gaze_to_virtual_desktop(gaze_x, gaze_y)
                
                # Apply smoothing
                self.gaze_history.append((virtual_x, virtual_y))
                smooth_coords = self.smooth_gaze((virtual_x, virtual_y))
                self.current_gaze = smooth_coords
                
                # Enhanced debug visualization
                if self.debug_mode:
                    # Draw face rectangle
                    cv2.rectangle(frame, (face.left(), face.top()), (face.right(), face.bottom()), (0, 255, 0), 2)
                    
                    # Draw eye regions
                    cv2.polylines(frame, [np.array(left_eye_points)], True, (0, 255, 255), 1)
                    cv2.polylines(frame, [np.array(right_eye_points)], True, (0, 255, 255), 1)
                    
                    # Draw detected pupil centers
                    if left_eye_center:
                        cv2.circle(frame, left_eye_center, 3, (0, 0, 255), -1)
                    if right_eye_center:
                        cv2.circle(frame, right_eye_center, 3, (0, 0, 255), -1)
                    
                    # Draw gaze vector
                    avg_eye_x = (left_eye_center[0] + right_eye_center[0]) // 2
                    avg_eye_y = (left_eye_center[1] + right_eye_center[1]) // 2
                    
                    # Scale gaze vector for visualization
                    gaze_end_x = int(avg_eye_x + gaze_x * 100)
                    gaze_end_y = int(avg_eye_y + gaze_y * 100)
                    cv2.arrowedLine(frame, (avg_eye_x, avg_eye_y), (gaze_end_x, gaze_end_y), (255, 0, 0), 2)
                    
                    # Display gaze coordinates
                    cv2.putText(frame, f"Gaze: ({gaze_x:.3f}, {gaze_y:.3f})", 
                               (10, 30), cv2.FONT_HERSHEY_SIMPLEX, 0.7, (255, 255, 255), 2)
                    cv2.putText(frame, f"Virtual: ({virtual_x}, {virtual_y})", 
                               (10, 60), cv2.FONT_HERSHEY_SIMPLEX, 0.7, (255, 255, 255), 2)
            
            break  # Only process first face
        
        return frame
    
    def smooth_gaze(self, new_gaze):
        """Improved gaze smoothing"""
        if not self.gaze_history:
            return new_gaze
        
        # Weighted average with recent history
        weights = np.exp(np.linspace(-2, 0, len(self.gaze_history)))  # Exponential weighting
        weights = weights / weights.sum()
        
        avg_x = sum(point[0] * weight for point, weight in zip(self.gaze_history, weights))
        avg_y = sum(point[1] * weight for point, weight in zip(self.gaze_history, weights))
        
        # Apply smoothing
        smooth_x = int(new_gaze[0] * (1 - self.smoothing_factor) + avg_x * self.smoothing_factor)
        smooth_y = int(new_gaze[1] * (1 - self.smoothing_factor) + avg_y * self.smoothing_factor)
        
        return smooth_x, smooth_y
    
    def start_multi_monitor_calibration(self):
        """Enhanced calibration for better accuracy"""
        self.calibration_points = []
        
        # Generate more calibration points for better polynomial fitting
        all_calibration_points = []
        
        for screen in self.screens:
            # 16-point calibration per monitor for polynomial fitting
            margin = 50
            w, h = screen['width'], screen['height']
            x_base, y_base = screen['x'], screen['y']
            
            points_per_monitor = [
                # Corners
                (x_base + margin, y_base + margin),
                (x_base + w - margin, y_base + margin),
                (x_base + margin, y_base + h - margin),
                (x_base + w - margin, y_base + h - margin),
                
                # Edges
                (x_base + w//4, y_base + margin),
                (x_base + 3*w//4, y_base + margin),
                (x_base + margin, y_base + h//4),
                (x_base + w - margin, y_base + h//4),
                (x_base + margin, y_base + 3*h//4),
                (x_base + w - margin, y_base + 3*h//4),
                (x_base + w//4, y_base + h - margin),
                (x_base + 3*w//4, y_base + h - margin),
                
                # Centers
                (x_base + w//2, y_base + margin),
                (x_base + w//2, y_base + h - margin),
                (x_base + margin, y_base + h//2),
                (x_base + w - margin, y_base + h//2),
                (x_base + w//2, y_base + h//2)
            ]
            all_calibration_points.extend(points_per_monitor)
        
        messagebox.showinfo("Enhanced Calibration", 
                          f"Enhanced calibration will show {len(all_calibration_points)} points. "
                          f"Look at each point and press SPACE when focused.")
        
        for point in all_calibration_points:
            self.show_calibration_point(point)
        
        if self.calculate_enhanced_calibration():
            messagebox.showinfo("Success", "Enhanced calibration completed!")
            self.is_calibrated = True
        else:
            messagebox.showerror("Error", "Calibration failed!")
    
    def calculate_enhanced_calibration(self):
        """Calculate enhanced calibration with polynomial fitting"""
        if len(self.calibration_points) < 10:
            return False
        
        try:
            # Separate data
            virtual_points = [p['virtual'] for p in self.calibration_points]
            gaze_points = [p['gaze'] for p in self.calibration_points]
            
            virtual_x = np.array([p[0] for p in virtual_points])
            virtual_y = np.array([p[1] for p in virtual_points])
            gaze_x = np.array([p[0] for p in gaze_points])
            gaze_y = np.array([p[1] for p in gaze_points])
            
            # Fit 2nd order polynomial for better accuracy
            poly_x = np.polyfit(gaze_x, virtual_x, min(2, len(gaze_points) - 1))
            poly_y = np.polyfit(gaze_y, virtual_y, min(2, len(gaze_points) - 1))
            
            self.calibration_data = {
                'polynomial_x': poly_x.tolist(),
                'polynomial_y': poly_y.tolist(),
                'virtual_desktop': self.virtual_desktop.copy(),
                'calibration_accuracy': self.calculate_calibration_accuracy(),
                'calibration_method': 'polynomial'
            }
            
            self.save_calibration()
            return True
            
        except Exception as e:
            print(f"Enhanced calibration calculation error: {e}")
            # Fallback to linear calibration
            return self.calculate_linear_calibration()
    
    def calculate_linear_calibration(self):
        """Fallback linear calibration"""
        try:
            virtual_points = [p['virtual'] for p in self.calibration_points]
            gaze_points = [p['gaze'] for p in self.calibration_points]
            
            virtual_x = np.array([p[0] for p in virtual_points])
            virtual_y = np.array([p[1] for p in virtual_points])
            gaze_x = np.array([p[0] for p in gaze_points])
            gaze_y = np.array([p[1] for p in gaze_points])
            
            x_scale, x_offset = np.polyfit(gaze_x, virtual_x, 1)
            y_scale, y_offset = np.polyfit(gaze_y, virtual_y, 1)
            
            self.calibration_data = {
                'x_scale': x_scale,
                'y_scale': y_scale,
                'x_offset': x_offset,
                'y_offset': y_offset,
                'virtual_desktop': self.virtual_desktop.copy(),
                'calibration_accuracy': self.calculate_calibration_accuracy(),
                'calibration_method': 'linear'
            }
            
            self.save_calibration()
            return True
            
        except Exception as e:
            print(f"Linear calibration error: {e}")
            return False
    
    def calculate_calibration_accuracy(self):
        """Calculate calibration accuracy"""
        if not self.calibration_points:
            return 0
        
        total_error = 0
        for point in self.calibration_points:
            if 'polynomial_x' in self.calibration_data:
                predicted = self.calibrated_virtual_mapping(point['gaze'][0], point['gaze'][1])
            else:
                predicted = self.simple_virtual_mapping(point['gaze'][0], point['gaze'][1])
            actual = point['virtual']
            error = math.sqrt((predicted[0] - actual[0])**2 + (predicted[1] - actual[1])**2)
            total_error += error
        
        return total_error / len(self.calibration_points)
    
    def show_calibration_point(self, virtual_point):
        """Show calibration point and collect gaze data"""
        cal_window = tk.Toplevel()
        cal_window.title("Calibration Point")
        cal_window.geometry(f"20x20+{virtual_point[0]-10}+{virtual_point[1]-10}")
        cal_window.configure(bg='red')
        cal_window.overrideredirect(True)
        cal_window.attributes('-topmost', True)
        
        # Wait for stable gaze before accepting
        stable_readings = []
        
        def on_space(event):
            if hasattr(self, 'current_raw_gaze') and self.current_raw_gaze:
                stable_readings.append(self.current_raw_gaze)
                if len(stable_readings) >= 3:  # Require 3 stable readings
                    # Average the stable readings
                    avg_gaze_x = sum(r[0] for r in stable_readings) / len(stable_readings)
                    avg_gaze_y = sum(r[1] for r in stable_readings) / len(stable_readings)
                    
                    self.calibration_points.append({
                        'virtual': virtual_point,
                        'gaze': (avg_gaze_x, avg_gaze_y)
                    })
                    print(f"Calibration point: Virtual {virtual_point} -> Gaze ({avg_gaze_x:.3f}, {avg_gaze_y:.3f})")
                    cal_window.destroy()
        
        cal_window.bind('<KeyPress-space>', on_space)
        cal_window.focus_set()
        cal_window.wait_window()
    
    def initialize_camera(self):
        """Initialize camera with optimal settings"""
        self.cap = cv2.VideoCapture(0)
        if not self.cap.isOpened():
            return False
        
        # Optimize camera settings for eye tracking
        self.cap.set(cv2.CAP_PROP_FRAME_WIDTH, 640)
        self.cap.set(cv2.CAP_PROP_FRAME_HEIGHT, 480)
        self.cap.set(cv2.CAP_PROP_FPS, 30)
        self.cap.set(cv2.CAP_PROP_AUTOFOCUS, 1)  # Enable autofocus
        return True
    
    def create_gui_window(self):
        """Create GUI window"""
        self.gui_window = tk.Toplevel()
        self.gui_window.title("Eye Tracker")
        self.gui_window.geometry(f"{self.gui_size}x{self.gui_size}+100+100")
        self.gui_window.configure(bg=self.gui_color)
        self.gui_window.overrideredirect(True)
        self.gui_window.attributes('-topmost', True)
        self.gui_window.attributes('-alpha', 0.8)
        
        canvas = tk.Canvas(self.gui_window, width=self.gui_size, height=self.gui_size, 
                          highlightthickness=0, bg=self.gui_color)
        canvas.pack()
        canvas.create_oval(2, 2, self.gui_size-2, self.gui_size-2, 
                          fill=self.gui_color, outline='white', width=2)
    
    def update_gui_position(self):
        """Update GUI position"""
        if self.gui_window and self.current_gaze:
            virtual_x, virtual_y = self.current_gaze
            gui_x = virtual_x - self.gui_size // 2
            gui_y = virtual_y - self.gui_size // 2
            
            try:
                self.gui_window.geometry(f"{self.gui_size}x{self.gui_size}+{gui_x}+{gui_y}")
            except tk.TclError:
                pass
    
    def camera_loop(self):
        """Main camera loop"""
        while self.camera_running:
            frame = self.process_frame()
            if frame is not None:
                cv2.imshow('Improved Eye Tracker - Press Q to quit, D to toggle debug', frame)
                key = cv2.waitKey(1) & 0xFF
                if key == ord('q'):
                    break
                elif key == ord('d'):
                    self.debug_mode = not self.debug_mode
                    print(f"Debug mode: {'ON' if self.debug_mode else 'OFF'}")
            time.sleep(0.03)
    
    def gui_update_loop(self):
        """GUI update loop"""
        while self.camera_running:
            self.update_gui_position()
            time.sleep(0.016)
    
    def save_calibration(self):
        """Save calibration data"""
        try:
            with open('improved_eye_tracker_calibration.json', 'w') as f:
                json.dump(self.calibration_data, f, indent=2)
        except Exception as e:
            print(f"Error saving calibration: {e}")
    
    def load_calibration(self):
        """Load calibration data"""
        try:
            if os.path.exists('improved_eye_tracker_calibration.json'):
                with open('improved_eye_tracker_calibration.json', 'r') as f:
                    self.calibration_data = json.load(f)
                    saved_desktop = self.calibration_data.get('virtual_desktop', {})
                    if (saved_desktop.get('width') == self.virtual_desktop['width'] and 
                        saved_desktop.get('height') == self.virtual_desktop['height']):
                        self.is_calibrated = True
                        print("Calibration loaded successfully")
                    else:
                        print("Screen configuration changed, recalibration needed")
        except Exception as e:
            print(f"Error loading calibration: {e}")


class ImprovedEyeTrackerApp:
    def __init__(self, root):
        self.root = root
        self.root.title("Improved Eye Tracker Control Panel")
        self.root.geometry("500x800")
        
        self.tracker = ImprovedEyeTracker()
        self.camera_thread = None
        self.gui_thread = None
        
        self.create_ui()
    
    def create_ui(self):
        """Create UI with improved controls"""
        # Title
        title_label = tk.Label(self.root, text="Improved Eye Tracker", 
                              font=("Arial", 16, "bold"))
        title_label.pack(pady=10)
        
        # Virtual Desktop info
        vd_frame = tk.LabelFrame(self.root, text="Virtual Desktop", padx=5, pady=5)
        vd_frame.pack(fill="x", padx=10, pady=5)
        
        vd = self.tracker.virtual_desktop
        vd_text = f"Size: {vd['width']}x{vd['height']} | Screens: {len(self.tracker.screens)}"
        tk.Label(vd_frame, text=vd_text, font=("Arial", 9)).pack()
        
        # Screen info
        info_frame = tk.LabelFrame(self.root, text="Connected Monitors", padx=5, pady=5)
        info_frame.pack(fill="x", padx=10, pady=5)
        
        for screen in self.tracker.screens:
            screen_text = f"{screen['name']}: {screen['width']}x{screen['height']} at ({screen['x']}, {screen['y']})"
            tk.Label(info_frame, text=screen_text, font=("Arial", 9)).pack(anchor="w")
        
        # Controls
        controls_frame = tk.LabelFrame(self.root, text="Controls", padx=5, pady=5)
        controls_frame.pack(fill="x", padx=10, pady=5)
        
        button_frame = tk.Frame(controls_frame)
        button_frame.pack(fill="x", pady=5)
        
        self.start_button = tk.Button(button_frame, text="Start Tracking", 
                                     command=self.start_tracking, bg="green", fg="white")
        self.start_button.pack(side="left", padx=5)
        
        self.stop_button = tk.Button(button_frame, text="Stop Tracking", 
                                    command=self.stop_tracking, bg="red", fg="white", state="disabled")
        self.stop_button.pack(side="left", padx=5)
        
        # Debug toggle
        self.debug_var = tk.BooleanVar(value=self.tracker.debug_mode)
        debug_check = tk.Checkbutton(controls_frame, text="Debug Mode (Press 'D' in camera window)", 
                                   variable=self.debug_var, command=self.toggle_debug)
        debug_check.pack(anchor="w")
        
        # Pupil Detection Method
        detection_frame = tk.LabelFrame(self.root, text="Pupil Detection", padx=5, pady=5)
        detection_frame.pack(fill="x", padx=10, pady=5)
        
        tk.Label(detection_frame, text="Detection Method:").pack(anchor="w")
        self.method_var = tk.StringVar(value=self.tracker.pupil_detection_method)
        methods = [("Improved (Recommended)", "improved"), ("Simple", "simple")]
        for text, value in methods:
            tk.Radiobutton(detection_frame, text=text, variable=self.method_var, 
                          value=value, command=self.update_detection_method).pack(anchor="w")
        
        # Enhanced calibration controls
        cal_frame = tk.LabelFrame(self.root, text="Enhanced Calibration", padx=5, pady=5)
        cal_frame.pack(fill="x", padx=10, pady=5)
        
        self.cal_enabled_var = tk.BooleanVar(value=self.tracker.calibration_enabled)
        cal_check = tk.Checkbutton(cal_frame, text="Enable Calibration", 
                                  variable=self.cal_enabled_var, 
                                  command=self.toggle_calibration)
        cal_check.pack(anchor="w")
        
        cal_button = tk.Button(cal_frame, text="Run Enhanced Calibration", 
                              command=self.run_calibration, bg="blue", fg="white")
        cal_button.pack(pady=5)
        
        accuracy_text = "Not Calibrated"
        if self.tracker.is_calibrated and self.tracker.calibration_data:
            accuracy = self.tracker.calibration_data.get('calibration_accuracy', 0)
            method = self.tracker.calibration_data.get('calibration_method', 'unknown')
            accuracy_text = f"Calibrated ({method}): {accuracy:.1f}px error"
        
        self.cal_status = tk.Label(cal_frame, text=f"Status: {accuracy_text}")
        self.cal_status.pack(anchor="w")
        
        # Current status
        status_frame = tk.LabelFrame(self.root, text="Live Status", padx=5, pady=5)
        status_frame.pack(fill="x", padx=10, pady=5)
        
        self.gaze_coords_label = tk.Label(status_frame, text="Raw Gaze: (0.000, 0.000)")
        self.gaze_coords_label.pack(anchor="w")
        
        self.virtual_coords_label = tk.Label(status_frame, text="Virtual Coords: (0, 0)")
        self.virtual_coords_label.pack(anchor="w")
        
        self.pupil_status_label = tk.Label(status_frame, text="Pupil Detection: Not Started")
        self.pupil_status_label.pack(anchor="w")
        
        # Settings
        settings_frame = tk.LabelFrame(self.root, text="Settings", padx=5, pady=5)
        settings_frame.pack(fill="x", padx=10, pady=5)
        
        # Smoothing
        tk.Label(settings_frame, text="Smoothing Factor:").pack(anchor="w")
        self.smooth_var = tk.DoubleVar(value=self.tracker.smoothing_factor)
        smooth_scale = tk.Scale(settings_frame, from_=0.0, to=0.9, resolution=0.1, 
                               orient="horizontal", variable=self.smooth_var, 
                               command=self.update_smoothing)
        smooth_scale.pack(fill="x")
        
        # GUI size
        tk.Label(settings_frame, text="GUI Size:").pack(anchor="w")
        self.size_var = tk.IntVar(value=self.tracker.gui_size)
        size_scale = tk.Scale(settings_frame, from_=20, to=100, orient="horizontal", 
                             variable=self.size_var, command=self.update_gui_size)
        size_scale.pack(fill="x")
        
        # GUI color
        tk.Label(settings_frame, text="GUI Color:").pack(anchor="w")
        color_frame = tk.Frame(settings_frame)
        color_frame.pack(fill="x")
        
        colors = ['red', 'blue', 'green', 'yellow', 'purple', 'orange']
        self.color_var = tk.StringVar(value=self.tracker.gui_color)
        for color in colors:
            tk.Radiobutton(color_frame, text=color.title(), variable=self.color_var, 
                          value=color, command=self.update_gui_color).pack(side="left")
        
        # Main status
        self.status_label = tk.Label(self.root, text="Status: Ready", 
                                    font=("Arial", 10), fg="blue")
        self.status_label.pack(pady=10)
        
        # Enhanced instructions
        instructions = """
IMPROVED EYE TRACKING INSTRUCTIONS:

SETUP:
1. Ensure good lighting (avoid shadows on face)
2. Position yourself 50-80cm from camera
3. Keep your head relatively still during tracking
4. Make sure both eyes are clearly visible

CALIBRATION (IMPORTANT):
1. Run 'Enhanced Calibration' for best results
2. Look directly at each red dot when it appears
3. Press SPACE only when you're focused on the dot
4. Try to keep your head position consistent
5. More calibration points = better accuracy

DEBUGGING:
• Press 'D' in camera window to toggle debug view
• Debug shows: pupil detection, gaze vectors, coordinates
• Green rectangles = detected face/eyes
• Red circles = detected pupil centers
• Blue arrows = calculated gaze direction

TROUBLESHOOTING:
• If gaze is stuck in one area: recalibrate
• If pupil detection fails: improve lighting
• If tracking is jittery: increase smoothing
• If no detection: check camera/face visibility

The improved algorithm uses:
- Better pupil detection with contour analysis
- Head pose compensation
- Polynomial calibration for accuracy
- Enhanced smoothing algorithms
        """
        inst_label = tk.Label(self.root, text=instructions, justify="left", 
                             font=("Arial", 8), bg="lightyellow")
        inst_label.pack(fill="both", expand=True, padx=10, pady=5)
        
        # Start status update timer
        self.update_status_display()
    
    def update_status_display(self):
        """Update real-time status display"""
        if hasattr(self.tracker, 'current_raw_gaze') and self.tracker.current_raw_gaze:
            gaze = self.tracker.current_raw_gaze
            self.gaze_coords_label.config(text=f"Raw Gaze: ({gaze[0]:.3f}, {gaze[1]:.3f})")
        
        if hasattr(self.tracker, 'current_gaze') and self.tracker.current_gaze:
            coords = self.tracker.current_gaze
            self.virtual_coords_label.config(text=f"Virtual Coords: ({coords[0]}, {coords[1]})")
        
        # Update pupil detection status
        left_detected = self.tracker.left_eye_center is not None
        right_detected = self.tracker.right_eye_center is not None
        
        if left_detected and right_detected:
            status = "Both Eyes Detected ✓"
            color = "green"
        elif left_detected or right_detected:
            status = "One Eye Detected ⚠"
            color = "orange"
        else:
            status = "No Eyes Detected ✗"
            color = "red"
        
        self.pupil_status_label.config(text=f"Pupil Detection: {status}", fg=color)
        
        # Schedule next update
        self.root.after(100, self.update_status_display)
    
    def toggle_debug(self):
        """Toggle debug mode"""
        self.tracker.debug_mode = self.debug_var.get()
    
    def update_detection_method(self):
        """Update pupil detection method"""
        self.tracker.pupil_detection_method = self.method_var.get()
    
    def start_tracking(self):
        """Start tracking"""
        if not self.tracker.initialize_camera():
            messagebox.showerror("Error", "Could not initialize camera!")
            return
        
        if not self.tracker.predictor:
            messagebox.showerror("Error", "Facial landmark predictor not found!\n"
                               "Please download shape_predictor_68_face_landmarks.dat")
            return
        
        self.tracker.camera_running = True
        self.tracker.create_gui_window()
        
        self.camera_thread = threading.Thread(target=self.tracker.camera_loop)
        self.camera_thread.daemon = True
        self.camera_thread.start()
        
        self.gui_thread = threading.Thread(target=self.tracker.gui_update_loop)
        self.gui_thread.daemon = True
        self.gui_thread.start()
        
        self.start_button.config(state="disabled")
        self.stop_button.config(state="normal")
        self.status_label.config(text="Status: Tracking Active", fg="green")
    
    def stop_tracking(self):
        """Stop tracking"""
        self.tracker.camera_running = False
        
        if self.tracker.cap:
            self.tracker.cap.release()
        
        if self.tracker.gui_window:
            self.tracker.gui_window.destroy()
        
        cv2.destroyAllWindows()
        
        self.start_button.config(state="normal")
        self.stop_button.config(state="disabled")
        self.status_label.config(text="Status: Stopped", fg="red")
    
    def toggle_calibration(self):
        """Toggle calibration"""
        self.tracker.calibration_enabled = self.cal_enabled_var.get()
    
    def run_calibration(self):
        """Run enhanced calibration"""
        if not self.tracker.camera_running:
            messagebox.showwarning("Warning", "Please start tracking first!")
            return
        
        total_points = 17 * len(self.tracker.screens)  # 17 points per screen
        result = messagebox.askquestion("Enhanced Calibration", 
                                      f"Enhanced calibration will show {total_points} points.\n"
                                      f"This provides better accuracy than basic calibration.\n\n"
                                      f"Tips for best results:\n"
                                      f"• Keep your head still\n"
                                      f"• Look directly at each red dot\n"
                                      f"• Press SPACE only when focused\n"
                                      f"• Take your time\n\n"
                                      f"Continue?")
        
        if result == 'yes':
            self.tracker.start_multi_monitor_calibration()
            
            if self.tracker.is_calibrated:
                accuracy = self.tracker.calibration_data.get('calibration_accuracy', 0)
                method = self.tracker.calibration_data.get('calibration_method', 'unknown')
                self.cal_status.config(text=f"Status: Calibrated ({method}): {accuracy:.1f}px error")
                self.status_label.config(text="Status: Enhanced Calibration Complete", fg="green")
            else:
                self.cal_status.config(text="Status: Calibration Failed")
    
    def update_smoothing(self, value):
        """Update smoothing factor"""
        self.tracker.smoothing_factor = float(value)
    
    def update_gui_size(self, value):
        """Update GUI size"""
        self.tracker.gui_size = int(value)
        if self.tracker.gui_window:
            self.tracker.gui_window.geometry(f"{self.tracker.gui_size}x{self.tracker.gui_size}")
    
    def update_gui_color(self):
        """Update GUI color"""
        self.tracker.gui_color = self.color_var.get()
        if self.tracker.gui_window:
            self.tracker.gui_window.configure(bg=self.tracker.gui_color)


def main():
    root = tk.Tk()
    app = ImprovedEyeTrackerApp(root)
    
    def on_closing():
        app.stop_tracking()
        root.destroy()
    
    root.protocol("WM_DELETE_WINDOW", on_closing)
    root.mainloop()


if __name__ == "__main__":
    main() 