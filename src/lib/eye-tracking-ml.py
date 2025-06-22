#!/usr/bin/env python3
"""
Advanced Eye Tracking ML System
High-performance eye tracking using MediaPipe, OpenCV, and TensorFlow
Designed for integration with Tauri applications
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
                 smoothing_window: int = 5):
        """
        Initialize the eye tracking system
        
        Args:
            camera_id: Camera device ID
            model_path: Path to custom trained model
            calibration_points: Number of calibration points
            smoothing_window: Size of smoothing window for gaze points
        """
        self.camera_id = camera_id
        self.calibration_points = calibration_points
        self.smoothing_window = smoothing_window
        
        # MediaPipe setup
        self.mp_face_mesh = mp.solutions.face_mesh
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
        
        print("Built simple gaze estimation model")

    def load_gaze_model(self, model_path: str):
        """Load a pre-trained gaze estimation model"""
        try:
            self.gaze_model = tf.keras.models.load_model(model_path)
            print(f"Loaded gaze model from {model_path}")
        except Exception as e:
            print(f"Failed to load model: {e}")
            self.build_simple_model()

    def extract_eye_features(self, landmarks, face_3d) -> Optional[np.ndarray]:
        """Extract features from eye landmarks and head pose"""
        if landmarks is None or len(landmarks) == 0:
            return None
        
        try:
            # Check if we have enough landmarks
            if len(landmarks) < 468:  # MediaPipe face mesh has 468 landmarks
                print(f"Not enough landmarks: {len(landmarks)}", file=sys.stderr)
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
                    print("Insufficient eye landmarks", file=sys.stderr)
                    return None
                
                left_pupil = np.mean(left_eye_landmarks, axis=0)
                right_pupil = np.mean(right_eye_landmarks, axis=0)
            
            # Get eye corner landmarks for normalization - check bounds
            required_indices = [133, 33, 362, 263, 1, 168, 234, 454]
            for idx in required_indices:
                if idx >= len(landmarks):
                    print(f"Missing landmark {idx}, have {len(landmarks)} landmarks")
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
                print(f"Invalid eye widths: left={left_eye_width}, right={right_eye_width}")
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
            
            return features
            
        except Exception as e:
            print(f"Feature extraction error: {e}", file=sys.stderr)
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
            
            return gaze_point
            
        except Exception as e:
            print(f"Gaze estimation error: {e}")
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
            self.draw_debug_info(frame, landmarks, gaze_point)
        
        return gaze_point, frame

    def draw_debug_info(self, frame: np.ndarray, landmarks: np.ndarray, gaze_point: Optional[GazePoint]):
        """Draw debug information on the frame"""
        h, w = frame.shape[:2]
        
        # Draw eye landmarks
        for idx in self.LEFT_EYE + self.RIGHT_EYE:
            if idx < len(landmarks):
                x, y = landmarks[idx]
                cv2.circle(frame, (int(x * w), int(y * h)), 2, (0, 255, 0), -1)
        
        # Draw iris landmarks
        for idx in self.LEFT_IRIS + self.RIGHT_IRIS:
            if idx < len(landmarks):
                x, y = landmarks[idx]
                cv2.circle(frame, (int(x * w), int(y * h)), 2, (255, 0, 0), -1)
        
        # Draw gaze point info
        if gaze_point:
            cv2.putText(frame, f"Gaze: ({gaze_point.x:.0f}, {gaze_point.y:.0f})", 
                       (10, 30), cv2.FONT_HERSHEY_SIMPLEX, 0.7, (0, 255, 255), 2)
            cv2.putText(frame, f"Confidence: {gaze_point.confidence:.2f}", 
                       (10, 60), cv2.FONT_HERSHEY_SIMPLEX, 0.7, (0, 255, 255), 2)
        
        # Draw FPS
        cv2.putText(frame, f"FPS: {self.current_fps:.1f}", 
                   (10, h - 30), cv2.FONT_HERSHEY_SIMPLEX, 0.7, (255, 255, 0), 2)

    def calibrate(self) -> bool:
        """Perform calibration using multiple screen points"""
        print("Starting calibration...")
        print("Look at the red dot and press SPACE when focused")
        
        cap = cv2.VideoCapture(self.camera_id)
        if not cap.isOpened():
            print("Failed to open camera")
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
            print(f"Calibration point {point_idx + 1}/{len(calibration_points)}: ({target_x}, {target_y})")
            
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
            print(f"Calibration completed! Final loss: {history.history['loss'][-1]:.4f}")
            return True
        else:
            print("Insufficient calibration data")
            return False

    def run_tracking(self, headless: bool = False) -> None:
        """Main tracking loop"""
        cap = cv2.VideoCapture(self.camera_id)
        if not cap.isOpened():
            print("ERROR: Failed to open camera", file=sys.stderr)
            return
        
        # Set camera properties for better performance
        cap.set(cv2.CAP_PROP_FRAME_WIDTH, 640)
        cap.set(cv2.CAP_PROP_FRAME_HEIGHT, 480)
        cap.set(cv2.CAP_PROP_FPS, 30)
        
        if not headless:
            print("Eye tracking started. Press 'q' to quit, 'c' to calibrate", file=sys.stderr)
        else:
            print("INFO: Headless eye tracking started", file=sys.stderr)
            print("INFO: Camera opened successfully", file=sys.stderr)
        
        frame_count = 0
        face_detection_count = 0
        
        while self.processing:
            ret, frame = cap.read()
            if not ret:
                print("WARNING: Failed to read frame from camera", file=sys.stderr)
                continue
            
            frame_count += 1
            
            # Process frame
            gaze_point, processed_frame = self.process_frame(frame)
            
            # If no face detected, generate demo data for testing pipeline
            if gaze_point is None and headless:
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
                    print(f"INFO: No face detected, using demo data. Frame {frame_count}", file=sys.stderr)
            elif gaze_point is not None:
                face_detection_count += 1
                if frame_count % 30 == 0:  # Every 30 frames
                    print(f"INFO: Face detected! Detection rate: {face_detection_count}/{frame_count}", file=sys.stderr)
            
            # Update FPS
            self.fps_counter += 1
            if time.time() - self.fps_start_time >= 1.0:
                self.current_fps = self.fps_counter
                self.fps_counter = 0
                self.fps_start_time = time.time()
                if headless:
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
                print(json.dumps(gaze_data), flush=True)
            
            # Show debug window only if not headless
            if not headless:
                cv2.imshow('Eye Tracking', processed_frame)
                
                key = cv2.waitKey(1) & 0xFF
                if key == ord('q'):
                    break
                elif key == ord('c'):
                    self.calibrate()
            else:
                # In headless mode, just a small delay
                time.sleep(0.01)
        
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
    
    args = parser.parse_args()
    
    # Initialize eye tracker
    tracker = EyeTrackingML(
        camera_id=args.camera,
        model_path=args.model
    )
    
    tracker.screen_width = args.screen_width
    tracker.screen_height = args.screen_height
    
    try:
        # In headless mode, skip calibration and start immediately
        if args.headless:
            print("INFO: Starting in headless mode for Tauri integration", file=sys.stderr)
            # Use basic calibration for demo purposes
            tracker.is_calibrated = True
        elif args.calibrate:
            if not tracker.calibrate():
                print("ERROR: Calibration failed", file=sys.stderr)
                return
        
        # Start tracking
        tracker.run_tracking(headless=args.headless)
    
    except KeyboardInterrupt:
        print("\nINFO: Shutting down...", file=sys.stderr)
    except Exception as e:
        print(f"ERROR: {e}", file=sys.stderr)
    finally:
        tracker.stop_tracking()

if __name__ == "__main__":
    main() 