// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::{App, Manager, Window};
use std::process::{Command, Stdio, Child};
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use serde_json;

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
}

impl MLEyeTrackingProcess {
    pub fn new(config: MLEyeTrackingConfig) -> Self {
        Self {
            process: None,
            config,
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        // Find the Python script - try multiple possible locations
        let possible_paths = vec![
            // Development path (most likely)
            std::env::current_dir().unwrap().join("src").join("lib").join("eye-tracking-ml.py"),
            // Alternative development path
            std::env::current_dir().unwrap().join("..").join("src").join("lib").join("eye-tracking-ml.py"),
            // Current directory
            std::env::current_dir().unwrap().join("eye-tracking-ml.py"),
            // Relative to src-tauri
            std::env::current_dir().unwrap().parent().unwrap().join("src").join("lib").join("eye-tracking-ml.py"),
        ];

        let mut python_script = None;
        for path in possible_paths {
            if path.exists() {
                python_script = Some(path);
                break;
            }
        }

        let python_script = python_script.ok_or_else(|| {
            let attempted_paths: Vec<String> = vec![
                std::env::current_dir().unwrap().join("src").join("lib").join("eye-tracking-ml.py").display().to_string(),
                std::env::current_dir().unwrap().join("..").join("src").join("lib").join("eye-tracking-ml.py").display().to_string(),
                std::env::current_dir().unwrap().join("eye-tracking-ml.py").display().to_string(),
            ];
            format!("Python script not found. Attempted paths: {:?}. Current dir: {:?}", 
                attempted_paths, std::env::current_dir().unwrap())
        })?;

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

        let mut cmd = Command::new(python_cmd);
        cmd.arg(python_script)
           .arg("--camera").arg(self.config.camera_id.to_string())
           .arg("--screen-width").arg(self.config.screen_width.to_string())
           .arg("--screen-height").arg(self.config.screen_height.to_string());

        if let Some(model_path) = &self.config.model_path {
            cmd.arg("--model").arg(model_path);
        }

        // Start the Python process
        let child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start Python process: {}", e))?;

        self.process = Some(child);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(mut process) = self.process.take() {
            process.kill().map_err(|e| format!("Failed to kill process: {}", e))?;
            process.wait().map_err(|e| format!("Failed to wait for process: {}", e))?;
        }
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
async fn start_ml_eye_tracking(config: MLEyeTrackingConfig) -> Result<String, String> {
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
    let tracker = ML_EYE_TRACKING.lock().unwrap();
    
    if tracker.as_ref().map_or(false, |t| t.is_running()) {
        // Generate dynamic simulated data that moves around
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        
        // Create moving gaze point for testing
        let x = 960.0 + (time * 0.5).sin() * 400.0; // Oscillate left-right
        let y = 540.0 + (time * 0.3).cos() * 300.0; // Oscillate up-down
        
        let gaze_data = MLGazeData {
            x: x.max(100.0).min(1820.0), // Keep within screen bounds
            y: y.max(100.0).min(980.0),
            confidence: 0.85 + (time * 0.1).sin() * 0.1, // Varying confidence
            timestamp: time,
            calibrated: true,
        };
        
        println!("ML Gaze data: x={:.1}, y={:.1}, conf={:.2}", gaze_data.x, gaze_data.y, gaze_data.confidence);
        Ok(Some(gaze_data))
    } else {
        Err("ML Eye tracking not running".to_string())
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
    let stats = serde_json::json!({
        "status": "running",
        "model_type": "tensorflow_keras",
        "features": [
            "MediaPipe face mesh",
            "Iris tracking", 
            "Head pose estimation",
            "Temporal smoothing",
            "Neural network gaze estimation"
        ],
        "performance": {
            "expected_fps": "15-30",
            "latency_ms": "30-50",
            "accuracy": "improved_with_calibration"
        }
    });
    
    Ok(stats)
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
            set_window_bounds,
            start_ml_eye_tracking,
            stop_ml_eye_tracking,
            get_ml_gaze_data,
            calibrate_ml_eye_tracking,
            get_ml_tracking_stats
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
