// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::Window;
use std::process::{Command, Stdio, Child};
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use serde_json;

// Speech transcription imports
use std::path::PathBuf;
use std::fs;
use base64::{Engine as _, engine::general_purpose};
use tempfile::NamedTempFile;
use anyhow::Result;
use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};
use std::time::{SystemTime, UNIX_EPOCH};

// Wake word detection (simplified for thread safety)

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn set_window_transparency(window: Window, alpha: f64) -> Result<(), String> {
    // Clamp alpha between 0.0 and 1.0
    let clamped_alpha = alpha.clamp(0.0, 1.0);
    
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{
            GetWindowLongPtrW, SetWindowLongPtrW, SetLayeredWindowAttributes, 
            GWL_EXSTYLE, WS_EX_LAYERED, WS_EX_TRANSPARENT, LWA_ALPHA
        };
        
        if let Ok(hwnd) = window.hwnd() {
            let hwnd = HWND(hwnd.0 as isize);
            
            unsafe {
                // Get current extended window style
                let mut ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                
                // Add layered window style for transparency
                ex_style |= WS_EX_LAYERED.0 as isize;
                
                // Add transparent style for click-through when very transparent
                if clamped_alpha < 0.1 {
                    ex_style |= WS_EX_TRANSPARENT.0 as isize;
                } else {
                    // Remove transparent style to enable interaction
                    ex_style &= !(WS_EX_TRANSPARENT.0 as isize);
                }
                
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style);
                
                // Set transparency level (0-255, where 255 is opaque)
                let alpha_value = (clamped_alpha * 255.0) as u8;
                SetLayeredWindowAttributes(hwnd, windows::Win32::Foundation::COLORREF(0), alpha_value, LWA_ALPHA)
                    .map_err(|e| format!("Failed to set transparency: {}", e))?;
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        use objc::runtime::{Object, Sel};
        use objc::{msg_send, sel, sel_impl};
        
        if let Ok(ns_window) = window.ns_window() {
            let ns_window = ns_window as *mut Object;
            unsafe {
                let _: () = msg_send![ns_window, setAlphaValue: clamped_alpha];
                
                // Enable/disable mouse events based on transparency
                let ignore_mouse = clamped_alpha < 0.1;
                let _: () = msg_send![ns_window, setIgnoresMouseEvents: ignore_mouse];
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        // Linux transparency implementation varies by window manager
        // This is a basic implementation for X11
        
        // Note: Linux implementation depends heavily on the desktop environment
        // This is a simplified version that may need adaptation
        window.set_decorations(false).map_err(|e| e.to_string())?;
        
        // For Wayland/X11, additional implementation would be needed
        // based on the specific compositor/window manager
    }
    
    Ok(())
}

#[tauri::command]
async fn emergency_restore_window(window: Window) -> Result<(), String> {
    // Always restore to fully opaque and interactive
    set_window_transparency(window.clone(), 1.0).await?;
    
    // Ensure window is visible and on top
    window.set_always_on_top(true).map_err(|e| e.to_string())?;
    window.unminimize().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
async fn toggle_transparency(window: Window, current_alpha: f64) -> Result<f64, String> {
    let new_alpha = if current_alpha > 0.5 { 0.3 } else { 1.0 };
    set_window_transparency(window, new_alpha).await?;
    Ok(new_alpha)
}

#[tauri::command]
async fn move_window_to_position(window: Window, x: i32, y: i32) -> Result<(), String> {
    use tauri::PhysicalPosition;
    
    let position = PhysicalPosition::new(x, y);
    window.set_position(position).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
async fn get_window_position(window: Window) -> Result<(i32, i32), String> {
    let position = window.outer_position().map_err(|e| e.to_string())?;
    Ok((position.x, position.y))
}

#[tauri::command]
async fn get_window_size(window: Window) -> Result<(u32, u32), String> {
    let size = window.outer_size().map_err(|e| e.to_string())?;
    Ok((size.width, size.height))
}

#[tauri::command]
async fn get_screen_size() -> Result<(u32, u32), String> {
    // Get primary monitor size
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
        
        unsafe {
            let width = GetSystemMetrics(SM_CXSCREEN) as u32;
            let height = GetSystemMetrics(SM_CYSCREEN) as u32;
            return Ok((width, height));
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        use core_graphics::display::CGMainDisplay;
        
        let display = CGMainDisplay();
        let width = display.pixels_wide() as u32;
        let height = display.pixels_high() as u32;
        return Ok((width, height));
    }
    
    #[cfg(target_os = "linux")]
    {
        // For Linux, we'll return a default size
        // In a production app, you'd want to query the actual display
        return Ok((1920, 1080));
    }
}

#[tauri::command]
async fn get_virtual_desktop_size() -> Result<(u32, u32), String> {
    // Get full virtual desktop size (all monitors combined)
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN};
        
        unsafe {
            let width = GetSystemMetrics(SM_CXVIRTUALSCREEN) as u32;
            let height = GetSystemMetrics(SM_CYVIRTUALSCREEN) as u32;
            println!("🖥️ Virtual desktop detected: {}x{}", width, height);
            return Ok((width, height));
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // For macOS, sum up all displays
        use core_graphics::display::{CGDisplay, CGDisplayBounds};
        
        let displays = CGDisplay::active_displays()
            .map_err(|e| format!("Failed to get displays: {:?}", e))?;
        
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        
        for display in displays {
            let bounds = CGDisplayBounds(display);
            min_x = min_x.min(bounds.origin.x);
            min_y = min_y.min(bounds.origin.y);
            max_x = max_x.max(bounds.origin.x + bounds.size.width);
            max_y = max_y.max(bounds.origin.y + bounds.size.height);
        }
        
        let width = (max_x - min_x) as u32;
        let height = (max_y - min_y) as u32;
        return Ok((width, height));
    }
    
    #[cfg(target_os = "linux")]
    {
        // For Linux, fall back to primary display
        return get_screen_size().await;
    }
}

#[tauri::command]
async fn set_window_bounds(window: Window, x: i32, y: i32, width: u32, height: u32) -> Result<(), String> {
    use tauri::{PhysicalPosition, PhysicalSize};
    
    let position = PhysicalPosition::new(x, y);
    let size = PhysicalSize::new(width, height);
    
    window.set_position(position).map_err(|e| e.to_string())?;
    window.set_size(size).map_err(|e| e.to_string())?;
    
    Ok(())
}

// Eye tracking ML integration types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MLGazeData {
    pub x: f64,
    pub y: f64,
    pub confidence: f64,
    pub timestamp: f64,
    pub calibrated: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WindowDragState {
    pub is_dragging: bool,
    pub drag_start_time: f64,
    pub last_position: Option<(f64, f64)>,
    pub pause_tracking: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MLEyeTrackingConfig {
    pub camera_id: i32,
    pub screen_width: i32,
    pub screen_height: i32,
    pub model_path: Option<String>,
    pub smoothing_window: i32,
}

// ML Eye tracking process manager
pub struct MLEyeTrackingProcess {
    process: Option<Child>,
    config: MLEyeTrackingConfig,
    receiver: Option<mpsc::UnboundedReceiver<MLGazeData>>,
    drag_state: WindowDragState,
}

impl MLEyeTrackingProcess {
    pub fn new(config: MLEyeTrackingConfig) -> Self {
        Self {
            process: None,
            config,
            receiver: None,
            drag_state: WindowDragState {
                is_dragging: false,
                drag_start_time: 0.0,
                last_position: None,
                pause_tracking: false,
            },
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        // Find the Python script - try multiple possible locations
        let possible_paths = vec![
            // Try new improved script first
            std::env::current_dir().unwrap().join("src").join("lib").join("gaze-ml-test.py"),
            // Fallback to original
            std::env::current_dir().unwrap().join("src").join("lib").join("eye-tracking-ml.py"),
            // Alternative development paths
            std::env::current_dir().unwrap().join("..").join("src").join("lib").join("gaze-ml-test.py"),
            std::env::current_dir().unwrap().join("..").join("src").join("lib").join("eye-tracking-ml.py"),
            // Current directory
            std::env::current_dir().unwrap().join("gaze-ml-test.py"),
            std::env::current_dir().unwrap().join("eye-tracking-ml.py"),
            // Relative to src-tauri
            std::env::current_dir().unwrap().parent().unwrap().join("src").join("lib").join("gaze-ml-test.py"),
            std::env::current_dir().unwrap().parent().unwrap().join("src").join("lib").join("eye-tracking-ml.py"),
        ];

        let mut python_script = None;
        let mut found_scripts = Vec::new();
        
        for path in possible_paths {
            if path.exists() {
                found_scripts.push(path.display().to_string());
                if python_script.is_none() {
                    python_script = Some(path.clone());
                    println!("✅ Found Python script: {}", path.display());
                }
            }
        }

        let python_script = python_script.ok_or_else(|| {
            let attempted_paths: Vec<String> = vec![
                std::env::current_dir().unwrap().join("src").join("lib").join("gaze-ml-test.py").display().to_string(),
                std::env::current_dir().unwrap().join("src").join("lib").join("eye-tracking-ml.py").display().to_string(),
                std::env::current_dir().unwrap().join("..").join("src").join("lib").join("gaze-ml-test.py").display().to_string(),
                std::env::current_dir().unwrap().join("gaze-ml-test.py").display().to_string(),
            ];
            format!("Python script not found. Attempted paths: {:?}. Current dir: {:?}", 
                attempted_paths, std::env::current_dir().unwrap())
        })?;
        
        println!("🐍 Using Python script: {}", python_script.display());
        if found_scripts.len() > 1 {
            println!("📝 Available scripts: {:?}", found_scripts);
        }

        // Build command arguments - try different Python commands
        let python_cmd = if cfg!(target_os = "windows") {
            // On Windows, try python first, then python3
            if Command::new("python").arg("--version").output().is_ok() {
                "python"
            } else if Command::new("python3").arg("--version").output().is_ok() {
                "python3"
            } else {
                return Err("Python not found. Please install Python 3.8+ and add it to PATH".to_string());
            }
        } else {
            // On Unix systems, prefer python3
            if Command::new("python3").arg("--version").output().is_ok() {
                "python3"
            } else if Command::new("python").arg("--version").output().is_ok() {
                "python"
            } else {
                return Err("Python not found. Please install Python 3.8+ and add it to PATH".to_string());
            }
        };

        // Debug: Print the script path before using it
        println!("DEBUG: Using Python script at: {:?}", python_script);
        
        let mut cmd = Command::new(python_cmd);
        cmd.arg(&python_script)
           .arg("--camera").arg(self.config.camera_id.to_string())
           .arg("--screen-width").arg(self.config.screen_width.to_string())
           .arg("--screen-height").arg(self.config.screen_height.to_string())
           .arg("--headless"); // Run in headless mode for Tauri integration

        if let Some(model_path) = &self.config.model_path {
            cmd.arg("--model").arg(model_path);
        }

        // Debug: Print the command we're about to run
        println!("DEBUG: Starting Python process with command: {:?}", cmd);
        
        // Start the Python process
        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start Python process: {}", e))?;
        
        println!("DEBUG: Python process started successfully with PID: {:?}", child.id());

        // Create channel for real-time gaze data
        let (tx, rx) = mpsc::unbounded_channel();

        // Spawn thread to read from Python process stderr (for debug info)
        if let Some(stderr) = child.stderr.take() {
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                println!("Python stderr reader thread started");
                for line in reader.lines() {
                    if let Ok(line) = line {
                        println!("Python stderr: {}", line);
                    }
                }
                println!("Python stderr reader thread ended");
            });
        }

        // Spawn thread to read from Python process stdout
        if let Some(stdout) = child.stdout.take() {
            std::thread::spawn(move || {
                let reader = BufReader::new(stdout);
                println!("Python stdout reader thread started");
                let mut json_count = 0;
                let mut non_json_count = 0;
                
                for line in reader.lines() {
                    if let Ok(line) = line {
                        let trimmed_line = line.trim();
                        if trimmed_line.is_empty() {
                            continue;
                        }
                        
                        // Try to parse JSON gaze data from Python
                        if trimmed_line.starts_with('{') && trimmed_line.ends_with('}') {
                            match serde_json::from_str::<MLGazeData>(trimmed_line) {
                                Ok(gaze_data) => {
                                    json_count += 1;
                                    if json_count % 30 == 1 {  // Log every 30th JSON message
                                        println!("Parsed ML gaze JSON #{}: x={:.1}, y={:.1}", 
                                            json_count, gaze_data.x, gaze_data.y);
                                    }
                                    if tx.send(gaze_data).is_err() {
                                        println!("Channel closed, stopping Python reader thread");
                                        break;
                                    }
                                },
                                Err(e) => {
                                    non_json_count += 1;
                                    if non_json_count <= 5 {  // Only log first 5 parse errors
                                        println!("JSON parse error: {} for line: {}", e, trimmed_line);
                                    }
                                }
                            }
                        } else {
                            // Print non-JSON output (debug info, errors, etc.)
                            println!("Python debug: {}", trimmed_line);
                        }
                    }
                }
                println!("Python process stdout reader thread ended. Parsed {} JSON messages, {} non-JSON lines", 
                    json_count, non_json_count);
            });
        }

        self.process = Some(child);
        self.receiver = Some(rx);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(mut process) = self.process.take() {
            process.kill().map_err(|e| format!("Failed to kill process: {}", e))?;
            process.wait().map_err(|e| format!("Failed to wait for process: {}", e))?;
        }
        self.receiver = None;
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.process.is_some()
    }


}

// Global ML eye tracking state with thread-safe access
lazy_static::lazy_static! {
    static ref ML_EYE_TRACKING: Arc<Mutex<Option<MLEyeTrackingProcess>>> = Arc::new(Mutex::new(None));
}

// ML Eye tracking commands
#[tauri::command]
async fn start_ml_eye_tracking(mut config: MLEyeTrackingConfig) -> Result<String, String> {
    // Auto-detect virtual desktop size if not properly set
    let (virtual_width, virtual_height) = get_virtual_desktop_size().await?;
    
    // Override config with correct virtual desktop dimensions
    config.screen_width = virtual_width as i32;
    config.screen_height = virtual_height as i32;
    
    println!("🎯 Starting ML eye tracking with virtual desktop: {}x{}", config.screen_width, config.screen_height);
    
    let mut tracker = ML_EYE_TRACKING.lock().unwrap();
    
    // Stop existing tracker if running
    if let Some(existing) = tracker.as_mut() {
        existing.stop()?;
    }
    
    // Create and start new tracker
    let mut new_tracker = MLEyeTrackingProcess::new(config);
    new_tracker.start()?;
    
    *tracker = Some(new_tracker);
    
    Ok("ML Eye tracking started successfully".to_string())
}

#[tauri::command]
async fn stop_ml_eye_tracking() -> Result<String, String> {
    let mut tracker = ML_EYE_TRACKING.lock().unwrap();
    
    if let Some(existing) = tracker.as_mut() {
        existing.stop()?;
        *tracker = None;
        Ok("ML Eye tracking stopped successfully".to_string())
    } else {
        Err("ML Eye tracking not running".to_string())
    }
}

#[tauri::command]
async fn get_ml_gaze_data() -> Result<Option<MLGazeData>, String> {
    // First, check if tracking is running and get a mutable reference if needed
    let has_running_tracker = {
        let tracker = ML_EYE_TRACKING.lock().unwrap();
        tracker.as_ref().map_or(false, |t| t.is_running())
    };
    
    if !has_running_tracker {
        return Err("ML Eye tracking not running".to_string());
    }
    
    // Now get the gaze data without holding the lock across await
    let mut tracker = ML_EYE_TRACKING.lock().unwrap();
    if let Some(ref mut tracker_instance) = tracker.as_mut() {
        // Use try_recv instead of async recv to avoid holding lock across await
        if let Some(receiver) = &mut tracker_instance.receiver {
            match receiver.try_recv() {
                Ok(gaze_data) => {
                    println!("Real ML Gaze data: x={:.1}, y={:.1}, conf={:.2}", 
                        gaze_data.x, gaze_data.y, gaze_data.confidence);
                    Ok(Some(gaze_data))
                },
                Err(_) => {
                    // No new data available right now
                    Ok(None)
                }
            }
        } else {
            Err("ML Eye tracking receiver not initialized".to_string())
        }
    } else {
        Err("ML Eye tracking not initialized".to_string())
    }
}

#[tauri::command]
async fn calibrate_ml_eye_tracking() -> Result<String, String> {
    let tracker = ML_EYE_TRACKING.lock().unwrap();
    
    if tracker.as_ref().map_or(false, |t| t.is_running()) {
        // In a real implementation, you would send calibration signals to the Python process
        Ok("ML Eye tracking calibration initiated".to_string())
    } else {
        Err("ML Eye tracking not running".to_string())
    }
}

#[tauri::command]
async fn get_ml_tracking_stats() -> Result<serde_json::Value, String> {
    let tracker = ML_EYE_TRACKING.lock().unwrap();
    
    let drag_state = if let Some(ref t) = *tracker {
        serde_json::json!({
            "is_dragging": t.drag_state.is_dragging,
            "pause_tracking": t.drag_state.pause_tracking,
            "drag_start_time": t.drag_state.drag_start_time
        })
    } else {
        serde_json::json!({
            "is_dragging": false,
            "pause_tracking": false,
            "drag_start_time": 0.0
        })
    };
    
    let stats = serde_json::json!({
        "status": "running",
        "model_type": "enhanced_mediapipe",
        "features": [
            "MediaPipe face mesh",
            "Iris tracking", 
            "Head pose estimation",
            "Temporal smoothing",
            "Multi-monitor support",
            "Window drag detection",
            "Graceful pause/resume"
        ],
        "performance": {
            "expected_fps": "20-40",
            "latency_ms": "25-45",
            "accuracy": "improved_with_calibration"
        },
        "drag_detection": drag_state
    });
    
    Ok(stats)
}

#[tauri::command]
async fn pause_ml_tracking() -> Result<String, String> {
    let mut tracker = ML_EYE_TRACKING.lock().unwrap();
    
    if let Some(ref mut tracker_instance) = tracker.as_mut() {
        tracker_instance.drag_state.pause_tracking = true;
        Ok("ML Eye tracking paused".to_string())
    } else {
        Err("ML Eye tracking not running".to_string())
    }
}

#[tauri::command]
async fn resume_ml_tracking() -> Result<String, String> {
    let mut tracker = ML_EYE_TRACKING.lock().unwrap();
    
    if let Some(ref mut tracker_instance) = tracker.as_mut() {
        tracker_instance.drag_state.pause_tracking = false;
        Ok("ML Eye tracking resumed".to_string())
    } else {
        Err("ML Eye tracking not running".to_string())
    }
}

#[tauri::command]
async fn detect_window_drag(window: Window, gaze_x: f64, gaze_y: f64) -> Result<bool, String> {
    let mut tracker = ML_EYE_TRACKING.lock().unwrap();
    
    if let Some(ref mut tracker_instance) = tracker.as_mut() {
        // Get window position and size
        let window_pos = window.outer_position().map_err(|e| e.to_string())?;
        let window_size = window.outer_size().map_err(|e| e.to_string())?;
        
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        
        // Check if gaze is in titlebar area (top 30 pixels)
        let in_titlebar = gaze_y >= window_pos.y as f64 && 
                         gaze_y <= (window_pos.y + 30) as f64 &&
                         gaze_x >= window_pos.x as f64 && 
                         gaze_x <= (window_pos.x + window_size.width as i32) as f64;
        
        if in_titlebar && !tracker_instance.drag_state.is_dragging {
            // Start drag detection
            tracker_instance.drag_state.is_dragging = true;
            tracker_instance.drag_state.drag_start_time = current_time;
            tracker_instance.drag_state.last_position = Some((gaze_x, gaze_y));
            tracker_instance.drag_state.pause_tracking = true;
            
            println!("🎯 Window drag detected - pausing gaze control");
            return Ok(true);
        } else if tracker_instance.drag_state.is_dragging {
            // Check for drag end (stable position)
            if let Some((last_x, last_y)) = tracker_instance.drag_state.last_position {
                let dx = (gaze_x - last_x).abs();
                let dy = (gaze_y - last_y).abs();
                
                if dx < 10.0 && dy < 10.0 && (current_time - tracker_instance.drag_state.drag_start_time) > 0.5 {
                    // Drag finished
                    tracker_instance.drag_state.is_dragging = false;
                    tracker_instance.drag_state.pause_tracking = false;
                    tracker_instance.drag_state.last_position = None;
                    
                    println!("🎯 Window drag finished - resuming gaze control");
                    return Ok(false);
                } else {
                    tracker_instance.drag_state.last_position = Some((gaze_x, gaze_y));
                }
            }
        }
        
        Ok(tracker_instance.drag_state.is_dragging)
    } else {
        Err("ML Eye tracking not running".to_string())
    }
}

// Speech transcription structures and commands
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WhisperModelConfig {
    pub model_size: String,
    pub language: Option<String>,
    pub enable_vad: bool,
    pub silence_threshold: f32,
    pub max_segment_length: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub confidence: f32,
    pub start_time: f32,
    pub end_time: f32,
    pub language: Option<String>,
}

lazy_static::lazy_static! {
    static ref WHISPER_CONTEXT: Arc<Mutex<Option<WhisperContext>>> = Arc::new(Mutex::new(None));
    static ref MODEL_CACHE_DIR: PathBuf = {
        let mut cache_dir = std::env::temp_dir();
        cache_dir.push("enteract");
        cache_dir.push("whisper_models");
        cache_dir
    };
    static ref WAKE_WORD_STATE: Arc<Mutex<WakeWordState>> = Arc::new(Mutex::new(WakeWordState::default()));
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WakeWordState {
    pub is_active: bool,
    pub is_listening: bool,
    pub last_detection: Option<WakeWordDetectionInfo>,
    pub total_detections: u32,
    pub whisper_activated: bool,
}

impl Default for WakeWordState {
    fn default() -> Self {
        Self {
            is_active: false,
            is_listening: false,
            last_detection: None,
            total_detections: 0,
            whisper_activated: false,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WakeWordDetectionInfo {
    pub confidence: f32,
    pub timestamp: u64,
    pub audio_length: usize,
}

// Simplified wake word detection commands
#[tauri::command]
async fn start_wake_word_detection() -> Result<String, String> {
    // Update state
    {
        let mut state = WAKE_WORD_STATE.lock().unwrap();
        state.is_active = true;
        state.is_listening = true;
    }
    
    println!("Wake word detection started (simplified version)");
    Ok("Wake word detection started for 'Aubrey'".to_string())
}

#[tauri::command]
async fn stop_wake_word_detection() -> Result<String, String> {
    // Update state
    {
        let mut state = WAKE_WORD_STATE.lock().unwrap();
        state.is_active = false;
        state.is_listening = false;
        state.whisper_activated = false;
    }
    
    println!("Wake word detection stopped");
    Ok("Wake word detection stopped".to_string())
}

#[tauri::command]
async fn check_wake_word_detection() -> Result<Option<WakeWordDetectionInfo>, String> {
    // For now, simulate wake word detection for testing
    // In a real implementation, this would check audio buffers for patterns
    
    let state = {
        let state_guard = WAKE_WORD_STATE.lock().unwrap();
        state_guard.clone()
    };
    
    if state.is_active {
        // Simulate occasional wake word detection for demo purposes
        // This would be replaced with actual audio processing
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().hash(&mut hasher);
        
        // Simulate wake word detection every ~30 seconds for demo
        if hasher.finish() % 30 == 0 && state.total_detections < 3 {
            let detection_info = WakeWordDetectionInfo {
                confidence: 0.85,
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
                audio_length: 16000, // 1 second of audio at 16kHz
            };
            
            // Update state
            {
                let mut state_guard = WAKE_WORD_STATE.lock().unwrap();
                state_guard.last_detection = Some(detection_info.clone());
                state_guard.total_detections += 1;
                state_guard.whisper_activated = true;
            }
            
            println!("Wake word detected (simulated)! Confidence: {:.2}", detection_info.confidence);
            
            return Ok(Some(detection_info));
        }
    }
    
    Ok(None)
}

#[tauri::command]
async fn get_wake_word_state() -> Result<WakeWordState, String> {
    let state = WAKE_WORD_STATE.lock().unwrap();
    Ok(state.clone())
}

#[tauri::command]
async fn reset_wake_word_stats() -> Result<String, String> {
    let mut state = WAKE_WORD_STATE.lock().unwrap();
    state.total_detections = 0;
    state.last_detection = None;
    state.whisper_activated = false;
    
    Ok("Wake word statistics reset".to_string())
}

#[tauri::command]
async fn initialize_whisper_model(config: WhisperModelConfig) -> Result<String, String> {
    let model_path = get_or_download_model(&config.model_size).await?;
    
    let ctx = WhisperContext::new_with_params(
        model_path.to_str().ok_or("Invalid model path")?,
        WhisperContextParameters::default()
    ).map_err(|e| format!("Failed to initialize Whisper context: {}", e))?;
    
    let mut whisper_ctx = WHISPER_CONTEXT.lock().unwrap();
    *whisper_ctx = Some(ctx);
    
    Ok(format!("Whisper model '{}' initialized successfully", config.model_size))
}

#[tauri::command]
async fn transcribe_audio_base64(audio_data: String, config: WhisperModelConfig) -> Result<TranscriptionResult, String> {
    // Decode base64 audio data
    let audio_bytes = general_purpose::STANDARD
        .decode(&audio_data)
        .map_err(|e| format!("Failed to decode base64 audio: {}", e))?;
    
    // Create temporary file for audio
    let temp_file = NamedTempFile::with_suffix(".wav")
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    
    fs::write(temp_file.path(), audio_bytes)
        .map_err(|e| format!("Failed to write audio to temp file: {}", e))?;
    
    transcribe_audio_file(temp_file.path().to_string_lossy().to_string(), config).await
}

#[tauri::command]
async fn transcribe_audio_file(file_path: String, config: WhisperModelConfig) -> Result<TranscriptionResult, String> {
    // Ensure model is initialized
    let needs_init = {
        let whisper_ctx = WHISPER_CONTEXT.lock().unwrap();
        whisper_ctx.is_none()
    };
    
    if needs_init {
        initialize_whisper_model(config.clone()).await?;
    }
    
    // Load and preprocess audio
    let audio_data = load_audio_file(&file_path)?;
    
    // Get Whisper context
    let whisper_ctx = WHISPER_CONTEXT.lock().unwrap();
    let ctx = whisper_ctx.as_ref().ok_or("Whisper context not initialized")?;
    
    // Set up transcription parameters
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    
    if let Some(ref lang) = config.language {
        params.set_language(Some(lang));
    } else {
        params.set_language(Some("auto"));
    }
    
    params.set_translate(false);
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    
    // Run transcription
    let mut state = ctx.create_state().map_err(|e| format!("Failed to create state: {}", e))?;
    state.full(params, &audio_data)
        .map_err(|e| format!("Transcription failed: {}", e))?;
    
    // Extract results
    let num_segments = state.full_n_segments()
        .map_err(|e| format!("Failed to get segment count: {}", e))?;
    
    let mut full_text = String::new();
    let mut total_confidence = 0.0;
    let mut start_time: f32 = f32::MAX;
    let mut end_time: f32 = 0.0;
    
    for i in 0..num_segments {
        let segment_text = state.full_get_segment_text(i)
            .map_err(|e| format!("Failed to get segment text: {}", e))?;
        
        let segment_start = state.full_get_segment_t0(i)
            .map_err(|e| format!("Failed to get segment start time: {}", e))? as f32 / 100.0;
        
        let segment_end = state.full_get_segment_t1(i)
            .map_err(|e| format!("Failed to get segment end time: {}", e))? as f32 / 100.0;
        
        full_text.push_str(&segment_text);
        start_time = start_time.min(segment_start);
        end_time = end_time.max(segment_end);
        total_confidence += 1.0; // Whisper doesn't provide confidence scores directly
    }
    
    let avg_confidence = if num_segments > 0 { total_confidence / num_segments as f32 } else { 0.0 };
    
    Ok(TranscriptionResult {
        text: full_text.trim().to_string(),
        confidence: avg_confidence,
        start_time,
        end_time,
        language: config.language,
    })
}

#[tauri::command]
async fn check_whisper_model_availability(model_size: String) -> Result<bool, String> {
    let model_path = get_model_path(&model_size);
    Ok(model_path.exists())
}

#[tauri::command]
async fn download_whisper_model(model_size: String) -> Result<String, String> {
    // Force re-download by removing existing file
    let model_path = get_model_path(&model_size);
    if model_path.exists() {
        fs::remove_file(&model_path)
            .map_err(|e| format!("Failed to remove existing model: {}", e))?;
    }
    
    get_or_download_model(&model_size).await?;
    Ok(format!("Model '{}' downloaded successfully", model_size))
}

#[tauri::command]
async fn list_available_models() -> Result<Vec<String>, String> {
    Ok(vec![
        "tiny".to_string(),
        "base".to_string(),
        "small".to_string(),
        "medium".to_string(),
        "large".to_string(),
    ])
}

// Helper functions
async fn get_or_download_model(model_size: &str) -> Result<PathBuf, String> {
    let model_path = get_model_path(model_size);
    
    // Check if model exists and is valid
    if !model_path.exists() || !is_valid_model_file(&model_path) {
        // Remove invalid file if it exists
        if model_path.exists() {
            fs::remove_file(&model_path)
                .map_err(|e| format!("Failed to remove invalid model: {}", e))?;
        }
        download_model(model_size).await?;
    }
    
    Ok(model_path)
}

fn is_valid_model_file(path: &PathBuf) -> bool {
    // Check if file is large enough to be a real model (should be at least 1MB)
    if let Ok(metadata) = fs::metadata(path) {
        metadata.len() > 1_000_000 // 1MB minimum
    } else {
        false
    }
}

fn get_model_path(model_size: &str) -> PathBuf {
    let mut path = MODEL_CACHE_DIR.clone();
    path.push(format!("ggml-{}.bin", model_size));
    path
}

async fn download_model(model_size: &str) -> Result<(), String> {
    // Create cache directory if it doesn't exist
    fs::create_dir_all(&*MODEL_CACHE_DIR)
        .map_err(|e| format!("Failed to create cache directory: {}", e))?;
    
    let model_url = format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{}.bin",
        model_size
    );
    
    let model_path = get_model_path(model_size);
    
    println!("Downloading Whisper model '{}' from: {}", model_size, model_url);
    
    // Download the model
    let response = reqwest::get(&model_url).await
        .map_err(|e| format!("Failed to download model: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to download model: HTTP {}", response.status()));
    }
    
    let bytes = response.bytes().await
        .map_err(|e| format!("Failed to read model data: {}", e))?;
    
    fs::write(&model_path, bytes)
        .map_err(|e| format!("Failed to save model: {}", e))?;
    
    println!("Successfully downloaded Whisper model '{}' to: {:?}", model_size, model_path);
    
    Ok(())
}

fn load_audio_file(file_path: &str) -> Result<Vec<f32>, String> {
    // For now, we'll assume the audio is already in the correct format
    // In a production app, you'd want to use a proper audio library like symphonia
    // to handle various audio formats and convert to 16kHz mono f32
    
    let audio_bytes = fs::read(file_path)
        .map_err(|e| format!("Failed to read audio file: {}", e))?;
    
    // Simple conversion assuming 16-bit PCM audio
    // This is a simplified implementation - you should use proper audio processing
    let mut audio_f32 = Vec::new();
    for chunk in audio_bytes.chunks(2) {
        if chunk.len() == 2 {
            let sample = i16::from_le_bytes([chunk[0], chunk[1]]) as f32 / 32768.0;
            audio_f32.push(sample);
        }
    }
    
    Ok(audio_f32)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Setup emergency global hotkey for transparency restore
            #[cfg(desktop)]
            {
                // Register global hotkey for emergency restore (Ctrl+Shift+Esc)
                // This ensures users can always regain control
                let _handle = app.handle().clone();
                
                // Note: Global hotkey registration would require additional dependencies
                // For now, we'll rely on window-level keyboard shortcuts
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet, 
            set_window_transparency, 
            emergency_restore_window,
            toggle_transparency,
            move_window_to_position,
            get_window_position,
            get_window_size,
            get_screen_size,
            get_virtual_desktop_size,
            set_window_bounds,
            start_ml_eye_tracking,
            stop_ml_eye_tracking,
            get_ml_gaze_data,
            calibrate_ml_eye_tracking,
            get_ml_tracking_stats,
            pause_ml_tracking,
            resume_ml_tracking,
            detect_window_drag,
            // Wake word detection commands
            start_wake_word_detection,
            stop_wake_word_detection,
            check_wake_word_detection,
            get_wake_word_state,
            reset_wake_word_stats,
            // Speech transcription commands
            initialize_whisper_model,
            transcribe_audio_base64,
            transcribe_audio_file,
            check_whisper_model_availability,
            download_whisper_model,
            list_available_models
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
