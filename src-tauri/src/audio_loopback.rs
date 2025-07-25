// src-tauri/src/audio_loopback.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use tauri::{AppHandle, Emitter};

// Re-use types from your existing implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioLoopbackDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub sample_rate: u32,
    pub channels: u16,
    pub format: String,
    pub device_type: DeviceType,
    pub loopback_method: LoopbackMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceType {
    Render,
    Capture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoopbackMethod {
    RenderLoopback,
    CaptureDevice,
    StereoMix,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDeviceSettings {
    #[serde(alias = "selected_loopback_device")]
    pub selectedLoopbackDevice: Option<String>,
    #[serde(alias = "loopback_enabled")]
    pub loopbackEnabled: bool,
    #[serde(alias = "buffer_size")]
    pub bufferSize: u32,
    #[serde(alias = "sample_rate")]
    pub sampleRate: u32,
}

impl Default for AudioDeviceSettings {
    fn default() -> Self {
        Self {
            selectedLoopbackDevice: None,
            loopbackEnabled: false,
            bufferSize: 4096,
            sampleRate: 16000,
        }
    }
}

// Import your existing device enumeration logic
// This integrates the WASAPI enumeration code from the sandbox
mod device_enumerator {
    use super::*;
    use std::error::Error;
    
    // Note: In a real implementation, you would add the required dependencies:
    // [dependencies]
    // wasapi = "0.4"
    // windows = { version = "0.58", features = [...] }
    
    // For now, this provides a working stub that can be expanded
    pub struct WASAPILoopbackEnumerator;
    
    impl WASAPILoopbackEnumerator {
        pub fn new() -> Result<Self, Box<dyn Error>> {
            // TODO: Initialize COM with initialize_mta()
            // TODO: Create DeviceCollection for Render and Capture directions
            // This would match the sandbox implementation exactly
            Ok(Self)
        }
        
        pub fn enumerate_loopback_devices(&self) -> Result<Vec<AudioLoopbackDevice>, Box<dyn Error>> {
            // TODO: Implement the comprehensive scan from sandbox/device_enumerator.rs
            // This should include:
            // 1. scan_render_devices with render loopback capability
            // 2. scan_capture_devices with loopback-style names  
            // 3. scan_stereo_mix_devices for traditional loopback
            
            let mut devices = Vec::new();
            
            // Simulated devices - replace with actual WASAPI enumeration
            devices.push(AudioLoopbackDevice {
                id: "default_render_loopback".to_string(),
                name: "Speakers (Realtek High Definition Audio)".to_string(),
                is_default: true,
                sample_rate: 48000,
                channels: 2,
                format: "IEEE Float 32bit".to_string(),
                device_type: DeviceType::Render,
                loopback_method: LoopbackMethod::RenderLoopback,
            });
            
            devices.push(AudioLoopbackDevice {
                id: "stereo_mix_capture".to_string(),
                name: "Stereo Mix (Realtek High Definition Audio)".to_string(),
                is_default: false,
                sample_rate: 44100,
                channels: 2,
                format: "PCM 16bit".to_string(),
                device_type: DeviceType::Capture,
                loopback_method: LoopbackMethod::StereoMix,
            });
            
            println!("🔊 Simulated enumeration found {} devices", devices.len());
            Ok(devices)
        }
        
        pub fn auto_select_best_device(&self) -> Result<Option<AudioLoopbackDevice>, Box<dyn Error>> {
            let devices = self.enumerate_loopback_devices()?;
            
            // Priority logic from sandbox implementation:
            // 1. Default render device with render loopback
            for device in &devices {
                if device.is_default && 
                   matches!(device.device_type, DeviceType::Render) && 
                   matches!(device.loopback_method, LoopbackMethod::RenderLoopback) {
                    println!("✓ Auto-selected: Default render device with loopback: {}", device.name);
                    return Ok(Some(device.clone()));
                }
            }
            
            // 2. Any render device with loopback
            for device in &devices {
                if matches!(device.device_type, DeviceType::Render) && 
                   matches!(device.loopback_method, LoopbackMethod::RenderLoopback) {
                    println!("✓ Auto-selected: Render device with loopback: {}", device.name);
                    return Ok(Some(device.clone()));
                }
            }
            
            // 3. Stereo Mix device
            for device in &devices {
                if matches!(device.loopback_method, LoopbackMethod::StereoMix) {
                    println!("✓ Auto-selected: Stereo Mix device: {}", device.name);
                    return Ok(Some(device.clone()));
                }
            }
            
            // 4. Any capture device that might work
            for device in &devices {
                if matches!(device.device_type, DeviceType::Capture) {
                    println!("✓ Auto-selected: Capture device: {}", device.name);
                    return Ok(Some(device.clone()));
                }
            }
            
            if let Some(device) = devices.first() {
                println!("⚠ No optimal device found, using first available: {}", device.name);
                return Ok(Some(device.clone()));
            }
            
            println!("❌ No suitable loopback devices found");
            Ok(None)
        }
        
        pub fn test_device_capability(&self, device_id: &str) -> bool {
            // TODO: Implement actual device testing from sandbox
            // This would call try_initialize_render_loopback or try_initialize_capture_device
            // For now, return true for non-empty device IDs
            let result = !device_id.is_empty();
            println!("🧪 Testing device capability for {}: {}", device_id, if result { "✅ SUCCESS" } else { "❌ FAILED" });
            result
        }
    }
}

// Settings storage (simple file-based for now)
use std::path::PathBuf;
use std::fs;

fn get_settings_path() -> Result<PathBuf, Box<dyn Error>> {
    let app_data = dirs::config_dir()
        .ok_or("Could not find config directory")?;
    let app_dir = app_data.join("enteract");
    
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)?;
    }
    
    Ok(app_dir.join("audio_settings.json"))
}

fn get_general_settings_path() -> Result<PathBuf, Box<dyn Error>> {
    let app_data = dirs::config_dir()
        .ok_or("Could not find config directory")?;
    let app_dir = app_data.join("enteract");
    
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)?;
    }
    
    Ok(app_dir.join("general_settings.json"))
}

// Tauri Commands
#[tauri::command]
pub async fn enumerate_loopback_devices() -> Result<Vec<AudioLoopbackDevice>, String> {
    match device_enumerator::WASAPILoopbackEnumerator::new() {
        Ok(enumerator) => {
            match enumerator.enumerate_loopback_devices() {
                Ok(devices) => {
                    println!("🔊 Found {} loopback devices", devices.len());
                    Ok(devices)
                },
                Err(e) => Err(format!("Failed to enumerate audio devices: {}", e))
            }
        },
        Err(e) => Err(format!("Failed to initialize audio enumerator: {}", e))
    }
}

#[tauri::command]
pub async fn auto_select_best_device() -> Result<Option<AudioLoopbackDevice>, String> {
    match device_enumerator::WASAPILoopbackEnumerator::new() {
        Ok(enumerator) => {
            match enumerator.auto_select_best_device() {
                Ok(device) => Ok(device),
                Err(e) => Err(format!("Failed to auto-select device: {}", e))
            }
        },
        Err(e) => Err(format!("Failed to initialize audio enumerator: {}", e))
    }
}

#[tauri::command]
pub async fn test_audio_device(device_id: String) -> Result<bool, String> {
    match device_enumerator::WASAPILoopbackEnumerator::new() {
        Ok(enumerator) => {
            let result = enumerator.test_device_capability(&device_id);
            if result {
                println!("✅ Audio device test successful: {}", device_id);
            } else {
                println!("❌ Audio device test failed: {}", device_id);
            }
            Ok(result)
        },
        Err(e) => Err(format!("Failed to test audio device: {}", e))
    }
}

#[tauri::command]
pub async fn save_audio_settings(settings: AudioDeviceSettings) -> Result<(), String> {
    let settings_path = get_settings_path()
        .map_err(|e| format!("Failed to get settings path: {}", e))?;
    
    let json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    
    fs::write(settings_path, json)
        .map_err(|e| format!("Failed to write settings file: {}", e))?;
    
    println!("💾 Audio settings saved");
    Ok(())
}

#[tauri::command]
pub async fn load_audio_settings() -> Result<Option<AudioDeviceSettings>, String> {
    let settings_path = get_settings_path()
        .map_err(|e| format!("Failed to get settings path: {}", e))?;
    
    if !settings_path.exists() {
        return Ok(None);
    }
    
    let json = fs::read_to_string(settings_path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;
    
    let settings: AudioDeviceSettings = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse settings: {}", e))?;
    
    println!("📂 Audio settings loaded");
    Ok(Some(settings))
}

#[tauri::command]
pub async fn save_general_settings(settings: HashMap<String, serde_json::Value>) -> Result<(), String> {
    let settings_path = get_general_settings_path()
        .map_err(|e| format!("Failed to get settings path: {}", e))?;
    
    let json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    
    fs::write(settings_path, json)
        .map_err(|e| format!("Failed to write settings file: {}", e))?;
    
    println!("💾 General settings saved");
    Ok(())
}

#[tauri::command]
pub async fn load_general_settings() -> Result<Option<HashMap<String, serde_json::Value>>, String> {
    let settings_path = get_general_settings_path()
        .map_err(|e| format!("Failed to get settings path: {}", e))?;
    
    if !settings_path.exists() {
        return Ok(None);
    }
    
    let json = fs::read_to_string(settings_path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;
    
    let settings: HashMap<String, serde_json::Value> = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse settings: {}", e))?;
    
    println!("📂 General settings loaded");
    Ok(Some(settings))
}

// Audio capture functionality for conversational interface
#[tauri::command]
pub async fn start_audio_loopback_capture(
    device_id: String,
    app_handle: AppHandle
) -> Result<String, String> {
    // This would start your real-time audio capture
    // Using your existing AudioCaptureWithTranscription logic
    
    println!("🎤 Starting audio loopback capture for device: {}", device_id);
    
    // Start capture in background thread
    let app_handle_clone = app_handle.clone();
    tokio::spawn(async move {
        // Use your existing capture loop here
        // Emit audio data events to the frontend
        
        // Example of emitting audio events:
        loop {
            // Capture audio chunk
            // let audio_chunk = capture_audio_chunk().await;
            
            // Emit to frontend for processing
            if let Err(e) = app_handle_clone.emit("audio-chunk", serde_json::json!({
                "device_id": device_id,
                "audio_data": "base64_encoded_audio", // Your actual audio data
                "timestamp": chrono::Utc::now().timestamp_millis()
            })) {
                eprintln!("Failed to emit audio chunk: {}", e);
                break;
            }
            
            // Sleep for appropriate interval based on buffer size
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });
    
    Ok("Audio capture started".to_string())
}

#[tauri::command]
pub async fn stop_audio_loopback_capture() -> Result<(), String> {
    println!("⏹️ Stopping audio loopback capture");
    // Stop the capture loop
    Ok(())
}

// Audio processing for transcription
#[tauri::command]
pub async fn process_audio_for_transcription(
    audio_data: Vec<u8>,
    sample_rate: u32
) -> Result<String, String> {
    // This would process the audio through Whisper
    // Using your existing whisper_transcriber logic
    
    println!("🤖 Processing audio for transcription ({} bytes at {} Hz)", 
             audio_data.len(), sample_rate);
    
    // Placeholder - integrate with your existing Whisper implementation
    Ok("Transcribed text would go here".to_string())
}

// Integration functions for your existing device enumeration
pub fn integrate_device_enumeration() -> Result<(), Box<dyn Error>> {
    // This function would integrate your existing device_enumerator.rs logic
    // You can copy your WASAPILoopbackEnumerator implementation here
    // and replace the placeholder implementation above
    
    println!("🔧 Integrating WASAPI device enumeration...");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_enumerate_devices() {
        let result = enumerate_loopback_devices().await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_settings_save_load() {
        let settings = AudioDeviceSettings {
            selectedLoopbackDevice: Some("test_device".to_string()),
            loopbackEnabled: true,
            bufferSize: 8192,
            sampleRate: 48000,
        };
        
        assert!(save_audio_settings(settings.clone()).await.is_ok());
        
        let loaded = load_audio_settings().await.unwrap();
        assert!(loaded.is_some());
        
        let loaded_settings = loaded.unwrap();
        assert_eq!(loaded_settings.selected_loopback_device, settings.selected_loopback_device);
        assert_eq!(loaded_settings.loopback_enabled, settings.loopback_enabled);
    }
}