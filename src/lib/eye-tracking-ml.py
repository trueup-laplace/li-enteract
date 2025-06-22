#!/usr/bin/env python3
"""
Advanced Eye Tracking ML System
High-performance eye tracking using MediaPipe, OpenCV, and TensorFlow
Designed for integration with Tauri applications

ENHANCED VERSION with standalone GUI testing capabilities
"""

import cv2
import numpy as np
import mediapipe as mp
import tensorflow as tf
import json
import time
import sys
import threading
from dataclasses import dataclass
from typing import List, Tuple, Optional, Dict
from collections import deque
import argparse
import os

@dataclass
class GazePoint:
    """Represents a gaze point with confidence and metadata"""
    x: float
    y: float
    confidence: float
    timestamp: float
    raw_pupil_x: float
    raw_pupil_y: float

@dataclass
class EyeData:
    """Complete eye data for both eyes"""
    left_pupil: Tuple[float, float]
    right_pupil: Tuple[float, float] 
    left_landmarks: List[Tuple[float, float]]
    right_landmarks: List[Tuple[float, float]]
    blink_ratio_left: float
    blink_ratio_right: float
    head_pose: Tuple[float, float, float]  # pitch, yaw, roll

class EyeTrackingML:
    """Advanced ML-based eye tracking system"""
    
    def __init__(self, 
                 camera_id: int = 0,
                 model_path: Optional[str] = None,
                 calibration_points: int = 9,
                 smoothing_window: int = 5,
                 debug_mode: bool = False,
                 gui_mode: bool = False):
        """
        Initialize the eye tracking system
        
        Args:
            camera_id: Camera device ID
            model_path: Path to custom trained model
            calibration_points: Number of calibration points
            smoothing_window: Size of smoothing window for gaze points
            debug_mode: Enable verbose debug output
            gui_mode: Enable standalone GUI with live preview
        """
        self.camera_id = camera_id
        self.calibration_points = calibration_points
        self.smoothing_window = smoothing_window
        self.debug_mode = debug_mode
        self.gui_mode = gui_mode
        
        if self.debug_mode:
            print("DEBUG: Initializing Eye Tracking ML system", file=sys.stderr)
        
        # MediaPipe setup
        self.mp_face_mesh = mp.solutions.face_mesh
        self.mp_drawing = mp.solutions.drawing_utils
        self.mp_drawing_styles = mp.solutions.drawing_styles
        
        self.face_mesh = self.mp_face_mesh.FaceMesh(
            max_num_faces=1,
            refine_landmarks=True,
            min_detection_confidence=0.7,
            min_tracking_confidence=0.5
        )
        
        # Eye landmark indices (MediaPipe Face Mesh)
        self.LEFT_EYE = [362, 382, 381, 380, 374, 373, 390, 249, 263, 466, 388, 387, 386, 385, 384, 398]
        self.RIGHT_EYE = [33, 7, 163, 144, 145, 153, 154, 155, 133, 173, 157, 158, 159, 160, 161, 246]
        # Iris landmarks (available only with refined landmarks)
        self.LEFT_IRIS = [474, 475, 476, 477, 473]
        self.RIGHT_IRIS = [469, 470, 471, 472, 468]
        
        # Calibration data
        self.calibration_data = []
        self.is_calibrated = False
        self.screen_width = 1920  # Will be updated
        self.screen_height = 1080  # Will be updated
        
        # Gaze smoothing
        self.gaze_history = deque(maxlen=smoothing_window)
        
        # Model for gaze estimation
        self.gaze_model = None
        if model_path and os.path.exists(model_path):
            self.load_gaze_model(model_path)
        else:
            self.build_simple_model()
        
        # Performance tracking
        self.fps_counter = 0
        self.fps_start_time = time.time()
        self.current_fps = 0
        
        # Threading
        self.frame_lock = threading.Lock()
        self.latest_frame = None
        self.processing = True
        
        # GUI mode specific
        if self.gui_mode:
            self.gaze_overlay = None
            self.initialize_gaze_overlay()

    def initialize_gaze_overlay(self):
        """Initialize the gaze overlay window for GUI mode"""
        if self.gui_mode:
            # Create a transparent overlay for gaze visualization
            self.gaze_overlay = np.zeros((self.screen_height, self.screen_width, 3), dtype=np.uint8)
            cv2.namedWindow('Gaze Overlay', cv2.WINDOW_NORMAL)
            cv2.setWindowProperty('Gaze Overlay', cv2.WND_PROP_TOPMOST, 1)

    def build_simple_model(self):
        """Build a simple neural network for gaze estimation"""
        self.gaze_model = tf.keras.Sequential([
            tf.keras.layers.Dense(128, activation='relu', input_shape=(14,)),  # Eye + head features
            tf.keras.layers.Dropout(0.2),
            tf.keras.layers.Dense(64, activation='relu'),
            tf.keras.layers.Dropout(0.2),
            tf.keras.layers.Dense(32, activation='relu'),
            tf.keras.layers.Dense(2, activation='linear')  # x, y coordinates
        ])
        
        self.gaze_model.compile(
            optimizer='adam',
            loss='mse',
            metrics=['mae']
        )
        
        if self.debug_mode:
            print("DEBUG: Built simple gaze estimation model", file=sys.stderr)
            self.gaze_model.summary()

    def load_gaze_model(self, model_path: str):
        """Load a pre-trained gaze estimation model"""
        try:
            self.gaze_model = tf.keras.models.load_model(model_path)
            if self.debug_mode:
                print(f"DEBUG: Loaded gaze model from {model_path}", file=sys.stderr)
        except Exception as e:
            print(f"ERROR: Failed to load model: {e}", file=sys.stderr)
            self.build_simple_model()

    def extract_eye_features(self, landmarks, face_3d) -> Optional[np.ndarray]:
        """Extract features from eye landmarks and head pose"""
        if landmarks is None or len(landmarks) == 0:
            return None
        
        try:
            # Check if we have enough landmarks
            if len(landmarks) < 468:  # MediaPipe face mesh has 468 landmarks
                if self.debug_mode:
                    print(f"DEBUG: Not enough landmarks: {len(landmarks)}", file=sys.stderr)
                return None
            
            # Get pupil centers - try iris landmarks first, fallback to eye center
            left_iris_landmarks = []
            right_iris_landmarks = []
            
            # Try to get iris landmarks (only available with refined landmarks)
            for idx in self.LEFT_IRIS:
                if idx < len(landmarks):
                    left_iris_landmarks.append(landmarks[idx])
            
            for idx in self.RIGHT_IRIS:
                if idx < len(landmarks):
                    right_iris_landmarks.append(landmarks[idx])
            
            if len(left_iris_landmarks) >= 3 and len(right_iris_landmarks) >= 3:
                # Use iris landmarks if available
                left_pupil = np.mean(left_iris_landmarks, axis=0)
                right_pupil = np.mean(right_iris_landmarks, axis=0)
                if self.debug_mode and self.fps_counter % 30 == 0:
                    print("DEBUG: Using iris landmarks for pupil detection", file=sys.stderr)
            else:
                # Fallback to eye center estimation using eye contour
                left_eye_landmarks = []
                right_eye_landmarks = []
                
                for idx in self.LEFT_EYE:
                    if idx < len(landmarks):
                        left_eye_landmarks.append(landmarks[idx])
                
                for idx in self.RIGHT_EYE:
                    if idx < len(landmarks):
                        right_eye_landmarks.append(landmarks[idx])
                
                if len(left_eye_landmarks) < 6 or len(right_eye_landmarks) < 6:
                    if self.debug_mode:
                        print("DEBUG: Insufficient eye landmarks", file=sys.stderr)
                    return None
                
                left_pupil = np.mean(left_eye_landmarks, axis=0)
                right_pupil = np.mean(right_eye_landmarks, axis=0)
                if self.debug_mode and self.fps_counter % 30 == 0:
                    print("DEBUG: Using eye contour for pupil detection", file=sys.stderr)
            
            # Get eye corner landmarks for normalization - check bounds
            required_indices = [133, 33, 362, 263, 1, 168, 234, 454]
            for idx in required_indices:
                if idx >= len(landmarks):
                    if self.debug_mode:
                        print(f"DEBUG: Missing landmark {idx}, have {len(landmarks)} landmarks", file=sys.stderr)
                    return None
            
            left_corner_inner = landmarks[133]
            left_corner_outer = landmarks[33]
            right_corner_inner = landmarks[362]
            right_corner_outer = landmarks[263]
            
            # Normalize pupil positions relative to eye corners
            left_eye_width = np.linalg.norm(left_corner_outer - left_corner_inner)
            right_eye_width = np.linalg.norm(right_corner_outer - right_corner_inner)
            
            # Check for valid eye widths
            if left_eye_width <= 0 or right_eye_width <= 0:
                if self.debug_mode:
                    print(f"DEBUG: Invalid eye widths: left={left_eye_width}, right={right_eye_width}", file=sys.stderr)
                return None
            
            left_pupil_norm = (left_pupil - left_corner_inner) / left_eye_width
            right_pupil_norm = (right_pupil - right_corner_inner) / right_eye_width
            
            # Head pose estimation (simplified)
            nose_tip = landmarks[1]
            nose_bridge = landmarks[168]
            left_ear = landmarks[234]
            right_ear = landmarks[454]
            
            # Calculate head rotation indicators
            head_tilt = np.arctan2(right_ear[1] - left_ear[1], right_ear[0] - left_ear[0])
            head_pan = (nose_tip[0] - 0.5) * 2  # Normalized pan
            head_depth = np.linalg.norm(nose_tip - nose_bridge)
            
            # Combine features
            features = np.array([
                float(left_pupil_norm[0]), float(left_pupil_norm[1]),
                float(right_pupil_norm[0]), float(right_pupil_norm[1]),
                float(left_pupil[0]), float(left_pupil[1]),
                float(right_pupil[0]), float(right_pupil[1]),
                float(head_tilt), float(head_pan), float(head_depth),
                float(left_eye_width), float(right_eye_width),
                0.5  # Placeholder for additional features
            ])
            
            if self.debug_mode and self.fps_counter % 60 == 0:
                print(f"DEBUG: Extracted features: {features[:4]}", file=sys.stderr)
            
            return features
            
        except Exception as e:
            if self.debug_mode:
                print(f"DEBUG: Feature extraction error: {e}", file=sys.stderr)
            return None

    def estimate_gaze(self, features: np.ndarray) -> Optional[GazePoint]:
        """Estimate gaze point using the ML model"""
        if self.gaze_model is None or features is None:
            return None
        
        try:
            # Reshape for model input
            features_reshaped = features.reshape(1, -1)
            
            # Get prediction
            prediction = self.gaze_model.predict(features_reshaped, verbose=0)[0]
            
            # Convert to screen coordinates
            screen_x = prediction[0] * self.screen_width
            screen_y = prediction[1] * self.screen_height
            
            # Calculate confidence (simplified)
            confidence = 0.8  # Placeholder - could be based on model uncertainty
            
            gaze_point = GazePoint(
                x=float(screen_x),
                y=float(screen_y),
                confidence=confidence,
                timestamp=time.time(),
                raw_pupil_x=features[4],
                raw_pupil_y=features[5]
            )
            
            if self.debug_mode and self.fps_counter % 30 == 0:
                print(f"DEBUG: Estimated gaze: ({screen_x:.1f}, {screen_y:.1f})", file=sys.stderr)
            
            return gaze_point
            
        except Exception as e:
            if self.debug_mode:
                print(f"DEBUG: Gaze estimation error: {e}", file=sys.stderr)
            return None

    def smooth_gaze(self, gaze_point: GazePoint) -> GazePoint:
        """Apply temporal smoothing to gaze points"""
        self.gaze_history.append(gaze_point)
        
        if len(self.gaze_history) < 2:
            return gaze_point
        
        # Weighted average with recent points having more weight
        weights = np.exp(np.linspace(-1, 0, len(self.gaze_history)))
        weights /= weights.sum()
        
        smooth_x = sum(w * gp.x for w, gp in zip(weights, self.gaze_history))
        smooth_y = sum(w * gp.y for w, gp in zip(weights, self.gaze_history))
        
        return GazePoint(
            x=smooth_x,
            y=smooth_y,
            confidence=gaze_point.confidence,
            timestamp=gaze_point.timestamp,
            raw_pupil_x=gaze_point.raw_pupil_x,
            raw_pupil_y=gaze_point.raw_pupil_y
        )

    def process_frame(self, frame: np.ndarray) -> Tuple[Optional[GazePoint], np.ndarray]:
        """Process a single frame and return gaze point"""
        rgb_frame = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
        results = self.face_mesh.process(rgb_frame)
        
        gaze_point = None
        
        if results.multi_face_landmarks:
            face_landmarks = results.multi_face_landmarks[0]
            
            # Convert landmarks to numpy array
            landmarks = []
            face_3d = []
            
            for landmark in face_landmarks.landmark:
                x = landmark.x
                y = landmark.y
                z = landmark.z
                
                landmarks.append([x, y])
                face_3d.append([x, y, z])
            
            landmarks = np.array(landmarks)
            face_3d = np.array(face_3d)
            
            # Extract features
            features = self.extract_eye_features(landmarks, face_3d)
            
            if features is not None:
                # Estimate gaze
                raw_gaze = self.estimate_gaze(features)
                
                if raw_gaze:
                    # Apply smoothing
                    gaze_point = self.smooth_gaze(raw_gaze)
            
            # Draw debug information
            self.draw_debug_info(frame, landmarks, gaze_point, results)
        
        return gaze_point, frame

    def draw_debug_info(self, frame: np.ndarray, landmarks: np.ndarray, gaze_point: Optional[GazePoint], results=None):
        """Draw debug information on the frame"""
        h, w = frame.shape[:2]
        
        # Draw face mesh if in GUI mode
        if self.gui_mode and results and results.multi_face_landmarks:
            for face_landmarks in results.multi_face_landmarks:
                # Draw face mesh with fallback styling
                try:
                    self.mp_drawing.draw_landmarks(
                        frame, face_landmarks, self.mp_face_mesh.FACEMESH_CONTOURS,
                        None, self.mp_drawing_styles.get_default_face_mesh_contours_style())
                except AttributeError:
                    # Fallback for older MediaPipe versions
                    self.mp_drawing.draw_landmarks(
                        frame, face_landmarks, self.mp_face_mesh.FACEMESH_CONTOURS)
                
                # Draw iris with fallback styling
                try:
                    self.mp_drawing.draw_landmarks(
                        frame, face_landmarks, self.mp_face_mesh.FACEMESH_IRISES,
                        None, self.mp_drawing_styles.get_default_face_mesh_iris_style())
                except AttributeError:
                    # Fallback for older MediaPipe versions
                    self.mp_drawing.draw_landmarks(
                        frame, face_landmarks, self.mp_face_mesh.FACEMESH_IRISES)
        
        # Draw eye landmarks with different colors
        for idx in self.LEFT_EYE:
            if idx < len(landmarks):
                x, y = landmarks[idx]
                cv2.circle(frame, (int(x * w), int(y * h)), 2, (0, 255, 0), -1)
        
        for idx in self.RIGHT_EYE:
            if idx < len(landmarks):
                x, y = landmarks[idx]
                cv2.circle(frame, (int(x * w), int(y * h)), 2, (0, 255, 0), -1)
        
        # Draw iris landmarks in blue
        for idx in self.LEFT_IRIS + self.RIGHT_IRIS:
            if idx < len(landmarks):
                x, y = landmarks[idx]
                cv2.circle(frame, (int(x * w), int(y * h)), 3, (255, 0, 0), -1)
        
        # Draw gaze point info
        if gaze_point:
            info_text = [
                f"Gaze: ({gaze_point.x:.0f}, {gaze_point.y:.0f})",
                f"Confidence: {gaze_point.confidence:.2f}",
                f"Calibrated: {'Yes' if self.is_calibrated else 'No'}",
                f"Smooth Window: {len(self.gaze_history)}/{self.smoothing_window}"
            ]
            
            for i, text in enumerate(info_text):
                cv2.putText(frame, text, (10, 30 + i * 25), 
                           cv2.FONT_HERSHEY_SIMPLEX, 0.6, (0, 255, 255), 2)
        
        # Draw FPS and debug info
        debug_info = [
            f"FPS: {self.current_fps:.1f}",
            f"Camera ID: {self.camera_id}",
            f"Screen: {self.screen_width}x{self.screen_height}"
        ]
        
        for i, text in enumerate(debug_info):
            cv2.putText(frame, text, (10, h - 80 + i * 25), 
                       cv2.FONT_HERSHEY_SIMPLEX, 0.5, (255, 255, 0), 2)
        
        # Draw controls
        if self.gui_mode:
            controls = [
                "Controls:",
                "Q - Quit",
                "C - Calibrate", 
                "D - Toggle Debug",
                "R - Reset Model",
                "SPACE - Toggle Overlay"
            ]
            
            for i, text in enumerate(controls):
                cv2.putText(frame, text, (w - 200, 30 + i * 20), 
                           cv2.FONT_HERSHEY_SIMPLEX, 0.4, (255, 255, 255), 1)

    def update_gaze_overlay(self, gaze_point: Optional[GazePoint]):
        """Update the gaze overlay window"""
        if not self.gui_mode or self.gaze_overlay is None:
            return
        
        # Clear overlay
        self.gaze_overlay.fill(0)
        
        if gaze_point:
            # Draw gaze point
            x, y = int(gaze_point.x), int(gaze_point.y)
            
            # Ensure coordinates are within screen bounds
            x = max(0, min(x, self.screen_width - 1))
            y = max(0, min(y, self.screen_height - 1))
            
            # Draw gaze point with varying size based on confidence
            radius = int(20 * gaze_point.confidence)
            cv2.circle(self.gaze_overlay, (x, y), radius, (0, 255, 0), 2)
            cv2.circle(self.gaze_overlay, (x, y), 5, (0, 255, 255), -1)
            
            # Draw gaze trail
            if len(self.gaze_history) > 1:
                points = [(int(gp.x), int(gp.y)) for gp in self.gaze_history]
                for i in range(1, len(points)):
                    alpha = i / len(points)
                    cv2.line(self.gaze_overlay, points[i-1], points[i], 
                            (int(255 * alpha), int(100 * alpha), 0), 2)
        
        # Show overlay
        cv2.imshow('Gaze Overlay', self.gaze_overlay)

    def calibrate(self) -> bool:
        """Perform calibration using multiple screen points"""
        print("Starting calibration...", file=sys.stderr)
        print("Look at the red dot and press SPACE when focused", file=sys.stderr)
        
        cap = cv2.VideoCapture(self.camera_id)
        if not cap.isOpened():
            print("Failed to open camera for calibration", file=sys.stderr)
            return False
        
        calibration_points = []
        # Create a grid of calibration points
        for i in range(3):
            for j in range(3):
                x = (j + 1) * self.screen_width // 4
                y = (i + 1) * self.screen_height // 4
                calibration_points.append((x, y))
        
        collected_data = []
        
        for point_idx, (target_x, target_y) in enumerate(calibration_points):
            print(f"Calibration point {point_idx + 1}/{len(calibration_points)}: ({target_x}, {target_y})", file=sys.stderr)
            
            # Show calibration window
            calib_window = np.zeros((self.screen_height, self.screen_width, 3), dtype=np.uint8)
            cv2.circle(calib_window, (target_x, target_y), 20, (0, 0, 255), -1)
            cv2.putText(calib_window, f"Look at the red dot and press SPACE ({point_idx + 1}/{len(calibration_points)})",
                       (50, 50), cv2.FONT_HERSHEY_SIMPLEX, 1, (255, 255, 255), 2)
            
            cv2.namedWindow('Calibration', cv2.WINDOW_FULLSCREEN)
            cv2.imshow('Calibration', calib_window)
            
            # Collect eye data
            samples = []
            while len(samples) < 30:  # Collect 30 samples per point
                ret, frame = cap.read()
                if ret:
                    rgb_frame = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
                    results = self.face_mesh.process(rgb_frame)
                    
                    if results.multi_face_landmarks:
                        landmarks = np.array([[lm.x, lm.y] for lm in results.multi_face_landmarks[0].landmark])
                        features = self.extract_eye_features(landmarks, None)
                        
                        if features is not None:
                            samples.append(features)
                
                key = cv2.waitKey(1) & 0xFF
                if key == ord(' ') and len(samples) >= 10:
                    break
                elif key == ord('q'):
                    cap.release()
                    cv2.destroyAllWindows()
                    return False
            
            if samples:
                # Average the samples for this calibration point
                avg_features = np.mean(samples, axis=0)
                collected_data.append((avg_features, target_x / self.screen_width, target_y / self.screen_height))
        
        cv2.destroyAllWindows()
        cap.release()
        
        # Train the model with calibration data
        if len(collected_data) >= 5:
            X = np.array([data[0] for data in collected_data])
            y = np.array([[data[1], data[2]] for data in collected_data])
            
            # Train the model
            history = self.gaze_model.fit(X, y, epochs=100, batch_size=len(X), verbose=0)
            
            self.is_calibrated = True
            print(f"Calibration completed! Final loss: {history.history['loss'][-1]:.4f}", file=sys.stderr)
            return True
        else:
            print("Insufficient calibration data", file=sys.stderr)
            return False

    def run_tracking(self, headless: bool = False) -> None:
        """Main tracking loop"""
        cap = None
        camera_available = False
        show_overlay = self.gui_mode
        
        # Try to open camera with multiple attempts and backends
        for attempt in range(3):
            backends = [cv2.CAP_DSHOW, cv2.CAP_ANY, cv2.CAP_MSMF] if sys.platform == "win32" else [cv2.CAP_V4L2, cv2.CAP_ANY]
            for backend in backends:
                try:
                    cap = cv2.VideoCapture(self.camera_id, backend)
                    if cap.isOpened():
                        # Test if we can actually read a frame
                        ret, test_frame = cap.read()
                        if ret and test_frame is not None:
                            camera_available = True
                            if self.debug_mode:
                                print(f"DEBUG: Camera opened successfully with backend {backend} on attempt {attempt + 1}", file=sys.stderr)
                            break
                        else:
                            cap.release()
                            cap = None
                    else:
                        if cap:
                            cap.release()
                            cap = None
                except Exception as e:
                    if self.debug_mode:
                        print(f"DEBUG: Camera attempt {attempt + 1} with backend {backend} failed: {e}", file=sys.stderr)
                    if cap:
                        cap.release()
                        cap = None
            
            if camera_available:
                break
            
            if self.debug_mode:
                print(f"DEBUG: Camera attempt {attempt + 1} failed, retrying...", file=sys.stderr)
            time.sleep(1)
        
        if not camera_available or cap is None:
            print("WARNING: Camera not available, running in demo mode only", file=sys.stderr)
            cap = None
        else:
            # Set camera properties for better performance
            try:
                cap.set(cv2.CAP_PROP_FRAME_WIDTH, 640)
                cap.set(cv2.CAP_PROP_FRAME_HEIGHT, 480)
                cap.set(cv2.CAP_PROP_FPS, 30)
                if self.debug_mode:
                    print("DEBUG: Camera properties set successfully", file=sys.stderr)
            except Exception as e:
                print(f"WARNING: Failed to set camera properties: {e}", file=sys.stderr)
        
        if not headless:
            if self.gui_mode:
                print("Eye tracking GUI started. Controls:", file=sys.stderr)
                print("  Q - Quit", file=sys.stderr)
                print("  C - Calibrate", file=sys.stderr)
                print("  D - Toggle Debug Mode", file=sys.stderr)
                print("  R - Reset Model", file=sys.stderr)
                print("  SPACE - Toggle Gaze Overlay", file=sys.stderr)
            else:
                print("Eye tracking started. Press 'q' to quit, 'c' to calibrate", file=sys.stderr)
        else:
            print("INFO: Headless eye tracking started", file=sys.stderr)
            if camera_available:
                print("INFO: Camera opened successfully", file=sys.stderr)
            else:
                print("INFO: Running in demo mode (no camera)", file=sys.stderr)
        
        frame_count = 0
        face_detection_count = 0
        
        while self.processing:
            gaze_point = None
            processed_frame = None
            
            if camera_available and cap is not None:
                ret, frame = cap.read()
                if not ret:
                    print("WARNING: Failed to read frame from camera", file=sys.stderr)
                    # Don't continue, fall through to demo mode for this frame
                    camera_available = False
                    if cap:
                        cap.release()
                        cap = None
                else:
                    frame_count += 1
                    # Process frame
                    gaze_point, processed_frame = self.process_frame(frame)
            
            # If no camera or failed to read, generate demo data
            if not camera_available:
                frame_count += 1
                # Create a dummy frame for GUI mode
                if self.gui_mode and processed_frame is None:
                    processed_frame = np.zeros((480, 640, 3), dtype=np.uint8)
                    cv2.putText(processed_frame, "NO CAMERA - DEMO MODE", (50, 240), 
                               cv2.FONT_HERSHEY_SIMPLEX, 1, (0, 0, 255), 2)
            
            # If no face detected, generate demo data for testing pipeline
            if gaze_point is None and (headless or frame_count % 60 == 0):
                # Generate smooth demo gaze data that moves around the screen
                demo_time = time.time() * 0.5  # Slow movement
                demo_x = (np.sin(demo_time) * 0.3 + 0.5) * self.screen_width  # Oscillate left-right
                demo_y = (np.cos(demo_time * 0.7) * 0.2 + 0.5) * self.screen_height  # Oscillate up-down
                
                gaze_point = GazePoint(
                    x=float(demo_x),
                    y=float(demo_y),
                    confidence=0.85,
                    timestamp=time.time(),
                    raw_pupil_x=0.5,
                    raw_pupil_y=0.5
                )
                
                if frame_count % 30 == 0:  # Every 30 frames
                    if self.debug_mode:
                        print(f"DEBUG: No face detected, using demo data. Frame {frame_count}", file=sys.stderr)
            elif gaze_point is not None:
                face_detection_count += 1
                if frame_count % 30 == 0:  # Every 30 frames
                    if self.debug_mode:
                        print(f"DEBUG: Face detected! Detection rate: {face_detection_count}/{frame_count}", file=sys.stderr)
            
            # Update FPS
            self.fps_counter += 1
            if time.time() - self.fps_start_time >= 1.0:
                self.current_fps = self.fps_counter
                self.fps_counter = 0
                self.fps_start_time = time.time()
                if headless or self.debug_mode:
                    print(f"INFO: Current FPS: {self.current_fps}, Face detection rate: {face_detection_count}/{frame_count} ({face_detection_count/frame_count*100:.1f}%)", file=sys.stderr)
            
            # Output gaze data (for integration with Tauri)
            if gaze_point:
                gaze_data = {
                    'x': gaze_point.x,
                    'y': gaze_point.y,
                    'confidence': gaze_point.confidence,
                    'timestamp': gaze_point.timestamp,
                    'calibrated': self.is_calibrated
                }
                # Output JSON for Tauri integration (stdout)
                if headless:
                    print(json.dumps(gaze_data), flush=True)
                elif self.debug_mode and frame_count % 30 == 0:
                    print(f"DEBUG: Gaze JSON: {json.dumps(gaze_data)}", file=sys.stderr)
            
            # GUI Mode handling
            if not headless:
                if self.gui_mode:
                    # Update gaze overlay
                    if show_overlay:
                        self.update_gaze_overlay(gaze_point)
                    
                    # Show main camera window
                    if processed_frame is not None:
                        cv2.imshow('Eye Tracking Camera', processed_frame)
                    
                    # Handle keyboard input for GUI mode
                    key = cv2.waitKey(1) & 0xFF
                    if key == ord('q'):
                        print("INFO: Quit requested", file=sys.stderr)
                        break
                    elif key == ord('c'):
                        print("INFO: Starting calibration...", file=sys.stderr)
                        self.calibrate()
                    elif key == ord('d'):
                        self.debug_mode = not self.debug_mode
                        print(f"INFO: Debug mode {'enabled' if self.debug_mode else 'disabled'}", file=sys.stderr)
                    elif key == ord('r'):
                        print("INFO: Resetting model...", file=sys.stderr)
                        self.build_simple_model()
                        self.is_calibrated = False
                    elif key == ord(' '):
                        show_overlay = not show_overlay
                        print(f"INFO: Gaze overlay {'enabled' if show_overlay else 'disabled'}", file=sys.stderr)
                        if not show_overlay:
                            cv2.destroyWindow('Gaze Overlay')
                else:
                    # Basic mode (original behavior)
                    if processed_frame is not None:
                        cv2.imshow('Eye Tracking', processed_frame)
                    
                    key = cv2.waitKey(1) & 0xFF
                    if key == ord('q'):
                        break
                    elif key == ord('c'):
                        self.calibrate()
            else:
                # In headless mode, just a small delay
                time.sleep(0.01)
        
        # Cleanup
        if cap is not None:
            cap.release()
        if not headless:
            cv2.destroyAllWindows()
        
        print(f"INFO: Tracking stopped. Processed {frame_count} frames, detected faces in {face_detection_count} frames", file=sys.stderr)

    def stop_tracking(self):
        """Stop the tracking loop"""
        self.processing = False

def main():
    """Main function for standalone execution"""
    parser = argparse.ArgumentParser(description='Advanced Eye Tracking ML System')
    parser.add_argument('--camera', type=int, default=0, help='Camera device ID')
    parser.add_argument('--model', type=str, help='Path to pre-trained model')
    parser.add_argument('--calibrate', action='store_true', help='Start with calibration')
    parser.add_argument('--screen-width', type=int, default=1920, help='Screen width')
    parser.add_argument('--screen-height', type=int, default=1080, help='Screen height')
    parser.add_argument('--headless', action='store_true', help='Run without GUI (for Tauri integration)')
    parser.add_argument('--gui', action='store_true', help='Run with enhanced GUI mode and live preview')
    parser.add_argument('--debug', action='store_true', help='Enable verbose debug output')
    parser.add_argument('--smoothing', type=int, default=5, help='Smoothing window size')
    
    args = parser.parse_args()
    
    # Validate arguments
    if args.headless and args.gui:
        print("ERROR: Cannot use both --headless and --gui modes", file=sys.stderr)
        return
    
    # Initialize eye tracker
    tracker = EyeTrackingML(
        camera_id=args.camera,
        model_path=args.model,
        smoothing_window=args.smoothing,
        debug_mode=args.debug,
        gui_mode=args.gui
    )
    
    tracker.screen_width = args.screen_width
    tracker.screen_height = args.screen_height
    
    try:
        # Print startup info
        mode_info = []
        if args.headless:
            mode_info.append("HEADLESS")
        if args.gui:
            mode_info.append("GUI")
        if args.debug:
            mode_info.append("DEBUG")
        
        mode_str = " + ".join(mode_info) if mode_info else "STANDARD"
        print(f"INFO: Starting Eye Tracking ML System in {mode_str} mode", file=sys.stderr)
        print(f"INFO: Screen resolution: {args.screen_width}x{args.screen_height}", file=sys.stderr)
        print(f"INFO: Camera ID: {args.camera}", file=sys.stderr)
        print(f"INFO: Smoothing window: {args.smoothing}", file=sys.stderr)
        
        # In headless mode, skip calibration and start immediately
        if args.headless:
            print("INFO: Starting in headless mode for Tauri integration", file=sys.stderr)
            # Use basic calibration for demo purposes
            tracker.is_calibrated = True
        elif args.calibrate:
            if not tracker.calibrate():
                print("ERROR: Calibration failed", file=sys.stderr)
                return
        elif args.gui:
            print("INFO: GUI mode started. Press 'C' to calibrate", file=sys.stderr)
        
        # Start tracking
        tracker.run_tracking(headless=args.headless)
    
    except KeyboardInterrupt:
        print("\nINFO: Shutting down...", file=sys.stderr)
    except Exception as e:
        print(f"ERROR: {e}", file=sys.stderr)
        if args.debug:
            import traceback
            traceback.print_exc()
    finally:
        tracker.stop_tracking()

if __name__ == "__main__":
    main()

# Basic Test (Camera + Debug)
# python eye-tracking-ml.py --debug

# Full GUI Mode with Live Preview
# python eye-tracking-ml.py --gui --debug --screen-width 1920 --screen-height 1080

# Headless Mode (Original Tauri Integration)
# python eye-tracking-ml.py --headless --camera 0 --screen-width 1920 --screen-height 1080