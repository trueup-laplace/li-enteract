use tauri::Window;
use std::process::{Command, Stdio, Child};
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use serde_json;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MLGazeData {
    pub x: f64,
    pub y: f64,
    pub confidence: f32,
    pub left_eye_landmarks: Vec<(f32, f32)>,
    pub right_eye_landmarks: Vec<(f32, f32)>,
    pub head_pose: HeadPose,
    pub timestamp: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HeadPose {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MLTrackingStats {
    pub total_frames_processed: u32,
    pub average_confidence: f32,
    pub frames_per_second: f32,
    pub tracking_duration: f64,
    pub last_update: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CalibrationPoint {
    pub screen_x: f64,
    pub screen_y: f64,
    pub gaze_x: f64,
    pub gaze_y: f64,
    pub confidence: f32,
    pub timestamp: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MLEyeTrackingConfig {
    pub camera_id: i32,
    pub screen_width: u32,
    pub screen_height: u32,
    pub smoothing_window: u32,
    pub confidence_threshold: f32,
    pub kalman_process_noise: f32,
    pub kalman_measurement_noise: f32,
    pub adaptive_smoothing: bool,
}

// Global eye tracker instance
lazy_static::lazy_static! {
    static ref EYE_TRACKER: Arc<Mutex<MLEyeTracker>> = Arc::new(Mutex::new(MLEyeTracker::new()));
}

fn get_eye_tracker() -> &'static Arc<Mutex<MLEyeTracker>> {
    &EYE_TRACKER
}

pub struct MLEyeTracker {
    process: Option<Child>,
    is_tracking: bool,
    is_calibrating: bool,
    stats: MLTrackingStats,
    calibration_points: Vec<CalibrationPoint>,
    last_gaze_data: Option<MLGazeData>,
    config: Option<MLEyeTrackingConfig>,
}

impl MLEyeTracker {
    pub fn new() -> Self {
        Self {
            process: None,
            is_tracking: false,
            is_calibrating: false,
            stats: MLTrackingStats {
                total_frames_processed: 0,
                average_confidence: 0.0,
                frames_per_second: 0.0,
                tracking_duration: 0.0,
                last_update: 0,
            },
            calibration_points: Vec::new(),
            last_gaze_data: None,
            config: None,
        }
    }

    pub fn start(&mut self, config: MLEyeTrackingConfig) -> Result<(), String> {
        if self.is_tracking {
            return Err("ML eye tracking is already running".to_string());
        }

        self.config = Some(config);

        // Get the project root directory (parent of src-tauri)
        let mut script_path = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;
        
        // If we're in src-tauri, go up one level to get to project root
        if script_path.file_name().and_then(|n| n.to_str()) == Some("src-tauri") {
            script_path.pop();
        }
        
        // Navigate to the src/lib directory from the project root
        script_path.push("src");
        script_path.push("lib");
        script_path.push("gaze-tracker-application.py");

        if !script_path.exists() {
            return Err(format!("Python script not found at: {:?}. Current dir: {:?}. Please ensure the gaze-tracker-application.py script exists in src/lib/", 
                script_path, std::env::current_dir().unwrap_or_default()));
        }

        // Find Python executable
        let python_cmd = if cfg!(target_os = "windows") {
            if Command::new("python").arg("--version").output().is_ok() { "python" } 
            else if Command::new("python3").arg("--version").output().is_ok() { "python3" } 
            else { return Err("Python not found. Please install Python 3.8+ and add it to PATH".to_string()); }
        } else {
            if Command::new("python3").arg("--version").output().is_ok() { "python3" }
            else if Command::new("python").arg("--version").output().is_ok() { "python" }
            else { return Err("Python not found. Please install Python 3.8+ and add it to PATH".to_string()); }
        };

        // Start Python ML eye tracking process with config
        let mut cmd = Command::new(python_cmd);
        cmd.arg(&script_path)
           .arg("--camera-id").arg(self.config.as_ref().unwrap().camera_id.to_string())
           .arg("--screen-width").arg(self.config.as_ref().unwrap().screen_width.to_string())
           .arg("--screen-height").arg(self.config.as_ref().unwrap().screen_height.to_string());

        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start ML eye tracking process: {}", e))?;

        println!("👁️  Started ML eye tracking using script at: {:?}", script_path);

        // Clone the main eye tracker instance for thread-safe access
        let eye_tracker_clone = Arc::clone(&EYE_TRACKER);

        // Spawn stdout reader thread for gaze data
        if let Some(stdout) = child.stdout.take() {
            std::thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        let trimmed = line.trim();
                        if trimmed.is_empty() { continue; }
                        
                        if trimmed.starts_with("GAZE:") {
                            if let Ok(gaze_data) = serde_json::from_str::<MLGazeData>(&trimmed[5..]) {
                                if let Ok(mut tracker) = eye_tracker_clone.lock() {
                                    tracker.update_gaze_data(gaze_data);
                                }
                            }
                        } else if trimmed.starts_with("CALIBRATION:") {
                            if let Ok(cal_point) = serde_json::from_str::<CalibrationPoint>(&trimmed[12..]) {
                                println!("Calibration point: ({:.2}, {:.2})", cal_point.screen_x, cal_point.screen_y);
                            }
                        } else {
                            println!("ML Debug: {}", trimmed);
                        }
                    }
                }
            });
        }

        // Spawn stderr reader thread
        if let Some(stderr) = child.stderr.take() {
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        println!("ML Error: {}", line);
                    }
                }
            });
        }

        self.process = Some(child);
        self.is_tracking = true;
        
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(mut process) = self.process.take() {
            process.kill().map_err(|e| format!("Failed to kill ML process: {}", e))?;
            process.wait().map_err(|e| format!("Failed to wait for ML process: {}", e))?;
        }
        
        self.is_tracking = false;
        self.is_calibrating = false;
        self.last_gaze_data = None;
        
        println!("👁️  Stopped ML eye tracking");
        Ok(())
    }

    pub fn is_tracking(&self) -> bool {
        self.is_tracking
    }

    pub fn pause(&mut self) {
        if self.is_tracking {
            // In a real implementation, you'd send a pause command to the Python process
            println!("👁️  Paused ML eye tracking");
        }
    }

    pub fn resume(&mut self) {
        if self.is_tracking {
            // In a real implementation, you'd send a resume command to the Python process
            println!("👁️  Resumed ML eye tracking");
        }
    }

    pub fn start_calibration(&mut self) -> Result<(), String> {
        if !self.is_tracking {
            return Err("Cannot start calibration: tracking not active".to_string());
        }
        
        self.is_calibrating = true;
        self.calibration_points.clear();
        
        println!("🎯 Started calibration");
        Ok(())
    }

    pub fn add_calibration_point(&mut self, screen_x: f64, screen_y: f64) -> Result<(), String> {
        if !self.is_calibrating {
            return Err("Calibration not active".to_string());
        }

        // Get current gaze data for calibration
        if let Some(gaze_data) = &self.last_gaze_data {
            let cal_point = CalibrationPoint {
                screen_x,
                screen_y,
                gaze_x: gaze_data.x,
                gaze_y: gaze_data.y,
                confidence: gaze_data.confidence,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            };
            
            self.calibration_points.push(cal_point);
            println!("📍 Added calibration point: ({:.1}, {:.1})", screen_x, screen_y);
        } else {
            return Err("No gaze data available for calibration".to_string());
        }

        Ok(())
    }

    pub fn finish_calibration(&mut self) -> Result<String, String> {
        if !self.is_calibrating {
            return Err("Calibration not active".to_string());
        }
        
        self.is_calibrating = false;
        
        let point_count = self.calibration_points.len();
        println!("✅ Calibration completed with {} points", point_count);
        
        Ok(format!("Calibration completed with {} points", point_count))
    }

    pub fn get_stats(&self) -> &MLTrackingStats {
        &self.stats
    }

    pub fn get_latest_gaze_data(&self) -> Option<&MLGazeData> {
        self.last_gaze_data.as_ref()
    }

    pub fn update_gaze_data(&mut self, gaze_data: MLGazeData) {
        self.last_gaze_data = Some(gaze_data);
        self.stats.total_frames_processed += 1;
        self.stats.last_update = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
    }

    pub fn detect_window_drag(&self) -> bool {
        // Placeholder for window drag detection logic
        // In real implementation, this would analyze gaze patterns
        false
    }
}

// Tauri command implementations with proper error handling
#[tauri::command]
pub async fn start_ml_eye_tracking(config: MLEyeTrackingConfig) -> Result<String, String> {
    match get_eye_tracker().lock() {
        Ok(mut tracker) => {
            tracker.start(config)?;
            Ok("ML Eye tracking started successfully".to_string())
        }
        Err(_) => Err("Failed to access eye tracker".to_string())
    }
}

#[tauri::command]
pub async fn stop_ml_eye_tracking() -> Result<String, String> {
    match get_eye_tracker().lock() {
        Ok(mut tracker) => {
            tracker.stop()?;
            Ok("ML Eye tracking stopped successfully".to_string())
        }
        Err(_) => Err("Failed to access eye tracker".to_string())
    }
}

#[tauri::command]
pub async fn get_ml_gaze_data() -> Result<Option<MLGazeData>, String> {
    match get_eye_tracker().lock() {
        Ok(tracker) => {
            if let Some(gaze_data) = tracker.get_latest_gaze_data() {
                Ok(Some(gaze_data.clone()))
            } else {
                Ok(None)
            }
        }
        Err(_) => Err("Failed to access eye tracker".to_string())
    }
}

#[tauri::command]
pub async fn calibrate_ml_eye_tracking() -> Result<String, String> {
    match get_eye_tracker().lock() {
        Ok(mut tracker) => {
            tracker.start_calibration()?;
            // Auto-calibration with predefined points would go here
            tracker.finish_calibration()
        }
        Err(_) => Err("Failed to access eye tracker".to_string())
    }
}

#[tauri::command]
pub async fn get_ml_tracking_stats() -> Result<MLTrackingStats, String> {
    match get_eye_tracker().lock() {
        Ok(tracker) => Ok(tracker.get_stats().clone()),
        Err(_) => Err("Failed to access eye tracker".to_string())
    }
}

#[tauri::command]
pub async fn pause_ml_tracking() -> Result<String, String> {
    match get_eye_tracker().lock() {
        Ok(mut tracker) => {
            tracker.pause();
            Ok("ML tracking paused".to_string())
        }
        Err(_) => Err("Failed to access eye tracker".to_string())
    }
}

#[tauri::command]
pub async fn resume_ml_tracking() -> Result<String, String> {
    match get_eye_tracker().lock() {
        Ok(mut tracker) => {
            tracker.resume();
            Ok("ML tracking resumed".to_string())
        }
        Err(_) => Err("Failed to access eye tracker".to_string())
    }
}

#[tauri::command]
pub async fn detect_window_drag() -> Result<bool, String> {
    match get_eye_tracker().lock() {
        Ok(tracker) => Ok(tracker.detect_window_drag()),
        Err(_) => Err("Failed to access eye tracker".to_string())
    }
}
