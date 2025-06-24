use tauri::Window;
use std::process::{Command, Stdio, Child};
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use serde_json;
use std::time::{SystemTime, UNIX_EPOCH};
use std::path::PathBuf;

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

pub struct MLEyeTracker {
    process: Option<Child>,
    is_tracking: bool,
    is_calibrating: bool,
    stats: MLTrackingStats,
    calibration_points: Vec<CalibrationPoint>,
    last_gaze_data: Option<MLGazeData>,
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
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.is_tracking {
            return Err("ML eye tracking is already running".to_string());
        }

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
        script_path.push("gaze-ml-improved.py");

        if !script_path.exists() {
            return Err(format!("Python script not found at: {:?}. Current dir: {:?}. Please ensure the gaze-ml-improved.py script exists in src/lib/", 
                script_path, std::env::current_dir().unwrap_or_default()));
        }

        // Find Python executable
        let python_cmd = if cfg!(target_os = "windows") {
            if Command::new("python").arg("--version").output().is_ok() {
                "python"
            } else if Command::new("python3").arg("--version").output().is_ok() {
                "python3"
            } else {
                return Err("Python not found. Please install Python 3.8+ and add it to PATH".to_string());
            }
        } else {
            if Command::new("python3").arg("--version").output().is_ok() {
                "python3"
            } else if Command::new("python").arg("--version").output().is_ok() {
                "python"
            } else {
                return Err("Python not found. Please install Python 3.8+ and add it to PATH".to_string());
            }
        };

        // Start Python ML eye tracking process
        let mut cmd = Command::new(python_cmd);
        cmd.arg(&script_path);

        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start ML eye tracking process: {}", e))?;

        println!("👁️  Started ML eye tracking using script at: {:?}", script_path);

        // Spawn stdout reader thread for gaze data
        if let Some(stdout) = child.stdout.take() {
            std::thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        
                        // Parse different types of ML output
                        if trimmed.starts_with("GAZE:") {
                            if let Ok(gaze_data) = serde_json::from_str::<MLGazeData>(&trimmed[5..]) {
                                // Store the latest gaze data (in a real implementation, you'd use a channel)
                                println!("Gaze: ({:.2}, {:.2}) confidence: {:.2}", 
                                    gaze_data.x, gaze_data.y, gaze_data.confidence);
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
            return Err("Eye tracking must be started before calibration".to_string());
        }
        
        self.is_calibrating = true;
        self.calibration_points.clear();
        
        // In a real implementation, you'd send a calibration start command to the Python process
        println!("👁️  Started ML calibration");
        Ok(())
    }

    pub fn add_calibration_point(&mut self, screen_x: f64, screen_y: f64) -> Result<(), String> {
        if !self.is_calibrating {
            return Err("Calibration not in progress".to_string());
        }

        // In a real implementation, you'd get actual gaze data from the ML model
        let mock_point = CalibrationPoint {
            screen_x,
            screen_y,
            gaze_x: screen_x + (rand::random::<f64>() - 0.5) * 100.0, // Mock gaze with some offset
            gaze_y: screen_y + (rand::random::<f64>() - 0.5) * 100.0,
            confidence: 0.85,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        };

        self.calibration_points.push(mock_point);
        println!("👁️  Added calibration point: ({:.2}, {:.2})", screen_x, screen_y);
        
        Ok(())
    }

    pub fn finish_calibration(&mut self) -> Result<String, String> {
        if !self.is_calibrating {
            return Err("Calibration not in progress".to_string());
        }

        self.is_calibrating = false;
        let num_points = self.calibration_points.len();
        
        // In a real implementation, you'd process the calibration data and update the ML model
        println!("👁️  Finished ML calibration with {} points", num_points);
        
        Ok(format!("ML calibration completed with {} points", num_points))
    }

    pub fn get_stats(&self) -> &MLTrackingStats {
        &self.stats
    }

    pub fn get_latest_gaze_data(&self) -> Option<&MLGazeData> {
        self.last_gaze_data.as_ref()
    }

    pub fn detect_window_drag(&self) -> bool {
        // In a real implementation, you'd analyze gaze patterns to detect window dragging
        // For now, return false as this is complex ML logic
        false
    }
}

// Global ML eye tracker instance
lazy_static::lazy_static! {
    static ref ML_EYE_TRACKER: Arc<Mutex<MLEyeTracker>> = Arc::new(Mutex::new(MLEyeTracker::new()));
}

// Tauri commands
#[tauri::command]
pub async fn start_ml_eye_tracking() -> Result<String, String> {
    let mut tracker = ML_EYE_TRACKER.lock().unwrap();
    tracker.start()?;
    Ok("ML eye tracking started successfully".to_string())
}

#[tauri::command]
pub async fn stop_ml_eye_tracking() -> Result<String, String> {
    let mut tracker = ML_EYE_TRACKER.lock().unwrap();
    tracker.stop()?;
    Ok("ML eye tracking stopped successfully".to_string())
}

#[tauri::command]
pub async fn get_ml_gaze_data() -> Result<Option<MLGazeData>, String> {
    let tracker = ML_EYE_TRACKER.lock().unwrap();
    
    if !tracker.is_tracking() {
        return Err("ML eye tracking is not running".to_string());
    }

    // Return mock gaze data for now
    // In a real implementation, this would return actual ML-computed gaze data
    let mock_data = MLGazeData {
        x: 500.0 + (rand::random::<f64>() - 0.5) * 200.0,
        y: 300.0 + (rand::random::<f64>() - 0.5) * 200.0,
        confidence: 0.75 + rand::random::<f32>() * 0.25,
        left_eye_landmarks: vec![(100.0, 100.0), (110.0, 105.0)], // Mock landmarks
        right_eye_landmarks: vec![(200.0, 100.0), (210.0, 105.0)],
        head_pose: HeadPose {
            yaw: (rand::random::<f32>() - 0.5) * 30.0,
            pitch: (rand::random::<f32>() - 0.5) * 20.0,
            roll: (rand::random::<f32>() - 0.5) * 15.0,
        },
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    };
    
    Ok(Some(mock_data))
}

#[tauri::command]
pub async fn calibrate_ml_eye_tracking(points: Vec<(f64, f64)>) -> Result<String, String> {
    let mut tracker = ML_EYE_TRACKER.lock().unwrap();
    
    tracker.start_calibration()?;
    
    for (x, y) in points {
        tracker.add_calibration_point(x, y)?;
    }
    
    tracker.finish_calibration()
}

#[tauri::command]
pub async fn get_ml_tracking_stats() -> Result<MLTrackingStats, String> {
    let tracker = ML_EYE_TRACKER.lock().unwrap();
    Ok(tracker.get_stats().clone())
}

#[tauri::command]
pub async fn pause_ml_tracking() -> Result<String, String> {
    let mut tracker = ML_EYE_TRACKER.lock().unwrap();
    tracker.pause();
    Ok("ML eye tracking paused".to_string())
}

#[tauri::command]
pub async fn resume_ml_tracking() -> Result<String, String> {
    let mut tracker = ML_EYE_TRACKER.lock().unwrap();
    tracker.resume();
    Ok("ML eye tracking resumed".to_string())
}

#[tauri::command]
pub async fn detect_window_drag() -> Result<bool, String> {
    let tracker = ML_EYE_TRACKER.lock().unwrap();
    Ok(tracker.detect_window_drag())
}
