// src-tauri/src/audio_loopback.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use wasapi::{DeviceCollection, Direction, Device, ShareMode, get_default_device, initialize_mta};
use base64::prelude::*;

// Audio capture state management
lazy_static::lazy_static! {
    static ref CAPTURE_STATE: Arc<Mutex<CaptureState>> = Arc::new(Mutex::new(CaptureState::default()));
}

#[derive(Default)]
struct CaptureState {
    is_capturing: bool,
    capture_handle: Option<tokio::task::JoinHandle<()>>,
    stop_tx: Option<mpsc::Sender<()>>,
}

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

// WASAPI Device Enumerator Implementation
pub struct WASAPILoopbackEnumerator {
    render_collection: DeviceCollection,
    capture_collection: DeviceCollection,
}

impl WASAPILoopbackEnumerator {
    pub fn new() -> Result<Self> {
        // Initialize COM for WASAPI
        initialize_mta()
            .map_err(|e| anyhow::anyhow!("Failed to initialize COM: {:?}", e))?;
            
        let render_collection = DeviceCollection::new(&Direction::Render)
            .map_err(|e| anyhow::anyhow!("Failed to create render device collection: {:?}", e))?;
        let capture_collection = DeviceCollection::new(&Direction::Capture)
            .map_err(|e| anyhow::anyhow!("Failed to create capture device collection: {:?}", e))?;
        
        Ok(Self { 
            render_collection,
            capture_collection,
        })
    }
    
    pub fn enumerate_loopback_devices(&self) -> Result<Vec<AudioLoopbackDevice>> {
        println!("\n=== WASAPI LOOPBACK DEVICE SCAN ===");
        
        let mut loopback_devices = Vec::new();
        
        // Get default devices for comparison
        let default_render = get_default_device(&Direction::Render).ok();
        let default_capture = get_default_device(&Direction::Capture).ok();
        
        let default_render_id = default_render.as_ref().and_then(|d| d.get_id().ok()).unwrap_or_default();
        let default_capture_id = default_capture.as_ref().and_then(|d| d.get_id().ok()).unwrap_or_default();
        
        // Strategy 1: Try render devices with loopback
        println!("🔍 Scanning render devices with loopback capability");
        if let Ok(render_devices) = self.scan_render_devices(&default_render_id) {
            loopback_devices.extend(render_devices);
        }
        
        // Strategy 2: Try capture devices that might be loopback
        println!("🔍 Scanning capture devices with loopback names");
        if let Ok(capture_devices) = self.scan_capture_devices(&default_capture_id) {
            loopback_devices.extend(capture_devices);
        }
        
        // Strategy 3: Look for stereo mix and similar devices
        println!("🔍 Scanning for Stereo Mix and system audio devices");
        if let Ok(stereo_devices) = self.scan_stereo_mix_devices(&default_capture_id) {
            loopback_devices.extend(stereo_devices);
        }
        
        // Remove duplicates by ID
        loopback_devices.sort_by(|a, b| a.id.cmp(&b.id));
        loopback_devices.dedup_by(|a, b| a.id == b.id);
        
        if loopback_devices.is_empty() {
            println!("❌ No suitable loopback devices found!");
        } else {
            println!("\n✅ Found {} loopback device(s)", loopback_devices.len());
        }
        
        Ok(loopback_devices)
    }
    
    fn scan_render_devices(&self, default_id: &str) -> Result<Vec<AudioLoopbackDevice>> {
        let device_count = self.render_collection.get_nbr_devices()
            .map_err(|e| anyhow::anyhow!("Failed to get render device count: {:?}", e))?;
        let mut devices = Vec::new();
        
        for i in 0..device_count {
            if let Ok(device) = self.render_collection.get_device_at_index(i) {
                if let Ok(device_info) = self.create_render_device_info(&device, default_id) {
                    if self.test_render_loopback_capability(&device) {
                        println!("  ✓ Render loopback: {}", device_info.name);
                        devices.push(device_info);
                    }
                }
            }
        }
        
        Ok(devices)
    }
    
    fn scan_capture_devices(&self, default_id: &str) -> Result<Vec<AudioLoopbackDevice>> {
        let device_count = self.capture_collection.get_nbr_devices()
            .map_err(|e| anyhow::anyhow!("Failed to get capture device count: {:?}", e))?;
        let mut devices = Vec::new();
        
        for i in 0..device_count {
            if let Ok(device) = self.capture_collection.get_device_at_index(i) {
                if let Ok(name) = device.get_friendlyname() {
                    if self.is_potential_loopback_capture_device(&name) {
                        if let Ok(device_info) = self.create_capture_device_info(&device, default_id) {
                            if self.test_capture_device_capability(&device) {
                                println!("  ✓ Capture loopback: {}", device_info.name);
                                devices.push(device_info);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(devices)
    }
    
    fn scan_stereo_mix_devices(&self, default_id: &str) -> Result<Vec<AudioLoopbackDevice>> {
        let device_count = self.capture_collection.get_nbr_devices()
            .map_err(|e| anyhow::anyhow!("Failed to get stereo mix device count: {:?}", e))?;
        let mut devices = Vec::new();
        
        for i in 0..device_count {
            if let Ok(device) = self.capture_collection.get_device_at_index(i) {
                if let Ok(name) = device.get_friendlyname() {
                    if self.is_stereo_mix_device(&name) {
                        if let Ok(mut device_info) = self.create_capture_device_info(&device, default_id) {
                            device_info.loopback_method = LoopbackMethod::StereoMix;
                            if self.test_capture_device_capability(&device) {
                                println!("  ✓ Stereo Mix: {}", device_info.name);
                                devices.push(device_info);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(devices)
    }
    
    fn is_potential_loopback_capture_device(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        name_lower.contains("loopback") || 
        name_lower.contains("what u hear") ||
        name_lower.contains("what you hear") ||
        name_lower.contains("wave") ||
        name_lower.contains("mix") ||
        (name_lower.contains("speakers") && name_lower.contains("capture"))
    }
    
    fn is_stereo_mix_device(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        name_lower.contains("stereo mix") ||
        name_lower.contains("stereomix") ||
        name_lower.contains("wave") ||
        name_lower.contains("what u hear") ||
        name_lower.contains("what you hear")
    }
    
    fn create_render_device_info(&self, device: &Device, default_id: &str) -> Result<AudioLoopbackDevice> {
        let id = device.get_id()
            .map_err(|e| anyhow::anyhow!("Failed to get device ID: {:?}", e))?;
        let name = device.get_friendlyname().unwrap_or_else(|_| "Unknown Render Device".to_string());
        let is_default = id == default_id;
        
        let (sample_rate, channels, format) = self.get_device_format(device)?;
        
        Ok(AudioLoopbackDevice {
            id,
            name,
            is_default,
            sample_rate,
            channels,
            format,
            device_type: DeviceType::Render,
            loopback_method: LoopbackMethod::RenderLoopback,
        })
    }
    
    fn create_capture_device_info(&self, device: &Device, default_id: &str) -> Result<AudioLoopbackDevice> {
        let id = device.get_id()
            .map_err(|e| anyhow::anyhow!("Failed to get device ID: {:?}", e))?;
        let name = device.get_friendlyname().unwrap_or_else(|_| "Unknown Capture Device".to_string());
        let is_default = id == default_id;
        
        let (sample_rate, channels, format) = self.get_device_format(device)?;
        
        Ok(AudioLoopbackDevice {
            id,
            name,
            is_default,
            sample_rate,
            channels,
            format,
            device_type: DeviceType::Capture,
            loopback_method: LoopbackMethod::CaptureDevice,
        })
    }
    
    fn get_device_format(&self, device: &Device) -> Result<(u32, u16, String)> {
        let audio_client = device.get_iaudioclient()
            .map_err(|e| anyhow::anyhow!("Failed to get audio client: {:?}", e))?;
        let format = audio_client.get_mixformat()
            .map_err(|e| anyhow::anyhow!("Failed to get mix format: {:?}", e))?;
        
        let sample_rate = format.get_samplespersec();
        let channels = format.get_nchannels();
        let bits_per_sample = format.get_bitspersample();
        
        let format_str = if bits_per_sample == 32 {
            "IEEE Float 32bit".to_string()
        } else if bits_per_sample == 24 {
            "PCM 24bit".to_string()
        } else if bits_per_sample == 16 {
            "PCM 16bit".to_string()
        } else {
            format!("PCM {}bit", bits_per_sample)
        };
        
        Ok((sample_rate, channels, format_str))
    }
    
    fn test_render_loopback_capability(&self, device: &Device) -> bool {
        match self.try_initialize_render_loopback(device) {
            Ok(_) => true,
            Err(_) => false
        }
    }
    
    fn test_capture_device_capability(&self, device: &Device) -> bool {
        match self.try_initialize_capture_device(device) {
            Ok(_) => true,
            Err(_) => false
        }
    }
    
    fn try_initialize_render_loopback(&self, device: &Device) -> Result<()> {
        let mut audio_client = device.get_iaudioclient()
            .map_err(|e| anyhow::anyhow!("Failed to get audio client: {:?}", e))?;
        let mix_format = audio_client.get_mixformat()
            .map_err(|e| anyhow::anyhow!("Failed to get mix format: {:?}", e))?;
        
        // Try to initialize the audio client in loopback mode
        let result = audio_client.initialize_client(
            &mix_format,
            10_000_000, // 1 second buffer in 100ns units
            &Direction::Render,
            &ShareMode::Shared,
            true, // use_loopback
        );
        
        result.map_err(|e| anyhow::anyhow!("WASAPI Error: {:?}", e))
    }
    
    fn try_initialize_capture_device(&self, device: &Device) -> Result<()> {
        let mut audio_client = device.get_iaudioclient()
            .map_err(|e| anyhow::anyhow!("Failed to get audio client: {:?}", e))?;
        let mix_format = audio_client.get_mixformat()
            .map_err(|e| anyhow::anyhow!("Failed to get mix format: {:?}", e))?;
        
        // Try to initialize as regular capture device (no loopback flag)
        let result = audio_client.initialize_client(
            &mix_format,
            10_000_000, // 1 second buffer in 100ns units
            &Direction::Capture,
            &ShareMode::Shared,
            false, // no loopback for capture devices
        );
        
        result.map_err(|e| anyhow::anyhow!("Failed to initialize capture device: {:?}", e))
    }
    
    pub fn auto_select_best_device(&self) -> Result<Option<AudioLoopbackDevice>> {
        let devices = self.enumerate_loopback_devices()?;
        
        if devices.is_empty() {
            return Ok(None);
        }
        
        // Priority 1: Default render device with render loopback
        if let Some(default_render) = devices.iter().find(|d| 
            d.is_default && 
            matches!(d.device_type, DeviceType::Render) && 
            matches!(d.loopback_method, LoopbackMethod::RenderLoopback)
        ) {
            println!("\n✓ Auto-selected: Default render device with loopback: {}", default_render.name);
            return Ok(Some(default_render.clone()));
        }
        
        // Priority 2: Any render device with loopback
        if let Some(render_device) = devices.iter().find(|d| 
            matches!(d.device_type, DeviceType::Render) && 
            matches!(d.loopback_method, LoopbackMethod::RenderLoopback)
        ) {
            println!("\n✓ Auto-selected: Render device with loopback: {}", render_device.name);
            return Ok(Some(render_device.clone()));
        }
        
        // Priority 3: Stereo Mix device
        if let Some(stereo_mix) = devices.iter().find(|d| 
            matches!(d.loopback_method, LoopbackMethod::StereoMix)
        ) {
            println!("\n✓ Auto-selected: Stereo Mix device: {}", stereo_mix.name);
            return Ok(Some(stereo_mix.clone()));
        }
        
        // Priority 4: Any capture device that might work
        if let Some(capture_device) = devices.iter().find(|d| 
            matches!(d.device_type, DeviceType::Capture)
        ) {
            println!("\n✓ Auto-selected: Capture device: {}", capture_device.name);
            return Ok(Some(capture_device.clone()));
        }
        
        // Fallback: First available device
        println!("\n⚠ No optimal device found, using first available: {}", devices[0].name);
        Ok(Some(devices[0].clone()))
    }
    
    pub fn find_device_by_id(&self, device_id: &str) -> Result<Option<AudioLoopbackDevice>> {
        let devices = self.enumerate_loopback_devices()?;
        Ok(devices.into_iter().find(|d| d.id == device_id))
    }
}

// Settings storage
use std::path::PathBuf;
use std::fs;

fn get_settings_path() -> Result<PathBuf> {
    let app_data = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
    let app_dir = app_data.join("enteract");
    
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)?;
    }
    
    Ok(app_dir.join("audio_settings.json"))
}

fn get_general_settings_path() -> Result<PathBuf> {
    let app_data = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
    let app_dir = app_data.join("enteract");
    
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)?;
    }
    
    Ok(app_dir.join("general_settings.json"))
}

// Tauri Commands
#[tauri::command]
pub async fn enumerate_loopback_devices() -> Result<Vec<AudioLoopbackDevice>, String> {
    match WASAPILoopbackEnumerator::new() {
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
    match WASAPILoopbackEnumerator::new() {
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
    match WASAPILoopbackEnumerator::new() {
        Ok(enumerator) => {
            // Find the device and test it properly
            match enumerator.find_device_by_id(&device_id) {
                Ok(Some(device_info)) => {
                    // Test based on device type
                    let result = match device_info.device_type {
                        DeviceType::Render => {
                            // Find and test render device
                            if let Ok(device_collection) = DeviceCollection::new(&Direction::Render) {
                                let device_count = device_collection.get_nbr_devices().unwrap_or(0);
                                for i in 0..device_count {
                                    if let Ok(device) = device_collection.get_device_at_index(i) {
                                        if let Ok(id) = device.get_id() {
                                            if id == device_id {
                                                return Ok(enumerator.test_render_loopback_capability(&device));
                                            }
                                        }
                                    }
                                }
                            }
                            false
                        },
                        DeviceType::Capture => {
                            // Find and test capture device
                            if let Ok(device_collection) = DeviceCollection::new(&Direction::Capture) {
                                let device_count = device_collection.get_nbr_devices().unwrap_or(0);
                                for i in 0..device_count {
                                    if let Ok(device) = device_collection.get_device_at_index(i) {
                                        if let Ok(id) = device.get_id() {
                                            if id == device_id {
                                                return Ok(enumerator.test_capture_device_capability(&device));
                                            }
                                        }
                                    }
                                }
                            }
                            false
                        }
                    };
                    
                    if result {
                        println!("✅ Audio device test successful: {}", device_id);
                    } else {
                        println!("❌ Audio device test failed: {}", device_id);
                    }
                    Ok(result)
                },
                Ok(None) => {
                    println!("❌ Device not found: {}", device_id);
                    Ok(false)
                },
                Err(e) => Err(format!("Failed to find device: {}", e))
            }
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

// Audio capture functionality with real-time processing
#[tauri::command]
pub async fn start_audio_loopback_capture(
    device_id: String,
    app_handle: AppHandle
) -> Result<String, String> {
    // Check if already capturing
    {
        let state = CAPTURE_STATE.lock().unwrap();
        if state.is_capturing {
            return Err("Audio capture already in progress".to_string());
        }
    }
    
    println!("🎤 Starting audio loopback capture for device: {}", device_id);
    
    // Create stop channel
    let (stop_tx, stop_rx) = mpsc::channel::<()>(1);
    
    // Start capture in background thread (not async because WASAPI types are not Send)
    let app_handle_clone = app_handle.clone();
    let device_id_clone = device_id.clone();
    
    let handle = tokio::task::spawn_blocking(move || {
        if let Err(e) = run_audio_capture_loop_sync(device_id_clone, app_handle_clone, stop_rx) {
            eprintln!("Audio capture error: {}", e);
        }
    });
    
    // Update state
    {
        let mut state = CAPTURE_STATE.lock().unwrap();
        state.is_capturing = true;
        state.capture_handle = Some(handle);
        state.stop_tx = Some(stop_tx);
    }
    
    Ok("Audio capture started".to_string())
}

#[tauri::command]
pub async fn stop_audio_loopback_capture() -> Result<(), String> {
    println!("⏹️ Stopping audio loopback capture");
    
    let (stop_tx, handle) = {
        let mut state = CAPTURE_STATE.lock().unwrap();
        state.is_capturing = false;
        (state.stop_tx.take(), state.capture_handle.take())
    };
    
    // Send stop signal
    if let Some(tx) = stop_tx {
        let _ = tx.send(()).await;
    }
    
    // Wait for task to complete
    if let Some(handle) = handle {
        let _ = handle.await;
    }
    
    Ok(())
}

// Audio processing for transcription
#[tauri::command]
pub async fn process_audio_for_transcription(
    audio_data: Vec<u8>,
    sample_rate: u32
) -> Result<String, String> {
    println!("🤖 Processing audio for transcription ({} bytes at {} Hz)", 
             audio_data.len(), sample_rate);
    
    // Convert audio data to f32 samples
    let _f32_samples: Vec<f32> = audio_data.chunks_exact(2)
        .map(|chunk| {
            let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
            sample as f32 / 32768.0
        })
        .collect();
    
    // TODO: Integrate with Whisper model
    // For now, return a placeholder
    Ok("Transcription would appear here".to_string())
}

// Audio capture loop implementation (synchronous because WASAPI types are not Send)
fn run_audio_capture_loop_sync(
    device_id: String,
    app_handle: AppHandle,
    mut stop_rx: mpsc::Receiver<()>
) -> Result<()> {
    // Initialize COM for this thread
    initialize_mta()
        .map_err(|e| anyhow::anyhow!("Failed to initialize COM: {:?}", e))?;
    
    // Find the device
    let enumerator = WASAPILoopbackEnumerator::new()?;
    let device_info = enumerator.find_device_by_id(&device_id)?
        .ok_or_else(|| anyhow::anyhow!("Device not found"))?;
    
    // Find the WASAPI device
    let wasapi_device = find_wasapi_device(&device_info)?;
    
    // Setup audio client
    let mut audio_client = wasapi_device.get_iaudioclient()
        .map_err(|e| anyhow::anyhow!("Failed to get audio client: {:?}", e))?;
    let format = audio_client.get_mixformat()
        .map_err(|e| anyhow::anyhow!("Failed to get mix format: {:?}", e))?;
    let (_, min_time) = audio_client.get_periods()
        .map_err(|e| anyhow::anyhow!("Failed to get periods: {:?}", e))?;
    
    // Determine direction and loopback based on device type
    let (direction, use_loopback) = match device_info.device_type {
        DeviceType::Render => (Direction::Render, true),
        DeviceType::Capture => (Direction::Capture, false),
    };
    
    // Initialize for loopback capture
    audio_client.initialize_client(
        &format,
        min_time,
        &direction,
        &ShareMode::Shared,
        use_loopback,
    ).map_err(|e| anyhow::anyhow!("Failed to initialize audio client: {:?}", e))?;
    
    let capture_client = audio_client.get_audiocaptureclient()
        .map_err(|e| anyhow::anyhow!("Failed to get capture client: {:?}", e))?;
    let h_event = audio_client.set_get_eventhandle()
        .map_err(|e| anyhow::anyhow!("Failed to get event handle: {:?}", e))?;
    
    println!("✅ Audio capture initialized successfully!");
    println!("📊 Format: {} Hz, {} channels, {} bit", 
             format.get_samplespersec(), 
             format.get_nchannels(),
             format.get_bitspersample());
    
    // Validate audio format before starting
    let bits_per_sample = format.get_bitspersample();
    let channels = format.get_nchannels();
    let sample_rate = format.get_samplespersec();
    
    if bits_per_sample != 16 && bits_per_sample != 32 {
        return Err(anyhow::anyhow!("Unsupported bits per sample: {}. Only 16 and 32 bit formats are supported.", bits_per_sample));
    }
    
    if channels == 0 || channels > 8 {
        return Err(anyhow::anyhow!("Invalid channel count: {}. Must be between 1 and 8.", channels));
    }
    
    if sample_rate < 8000 || sample_rate > 192000 {
        return Err(anyhow::anyhow!("Invalid sample rate: {} Hz. Must be between 8kHz and 192kHz.", sample_rate));
    }
    
    // Start the stream
    audio_client.start_stream()
        .map_err(|e| anyhow::anyhow!("Failed to start stream: {:?}", e))?;
    
    let start_time = Instant::now();
    let mut total_samples = 0u64;
    let mut last_emit = Instant::now();
    
    // Main capture loop
    loop {
        // Check for stop signal
        if stop_rx.try_recv().is_ok() {
            break;
        }
        
        // Wait for audio data with timeout
        match h_event.wait_for_event(100) {
            Ok(_) => {},
            Err(_) => {
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
        }
        
        // Get available frames
        let frames_available = match capture_client.get_next_nbr_frames() {
            Ok(Some(frames)) if frames > 0 => frames,
            _ => {
                std::thread::sleep(Duration::from_millis(1));
                continue;
            }
        };
        
        // Read audio data
        let bits_per_sample = format.get_bitspersample();
        let channels = format.get_nchannels();
        let bytes_per_sample = bits_per_sample / 8;
        let bytes_per_frame = bytes_per_sample * channels as u16;
        
        // Safety checks for buffer calculations
        if frames_available == 0 || bytes_per_frame == 0 {
            std::thread::sleep(Duration::from_millis(10));
            continue;
        }
        
        let buffer_size = frames_available as usize * bytes_per_frame as usize;
        
        // Sanity check: prevent excessive buffer sizes
        if buffer_size > 1_048_576 { // 1MB limit
            eprintln!("⚠️ Audio buffer size too large: {} bytes, skipping", buffer_size);
            std::thread::sleep(Duration::from_millis(10));
            continue;
        }
        
        let mut buffer = vec![0u8; buffer_size];
        
        let (frames_read, _flags) = match capture_client.read_from_device(bytes_per_frame as usize, &mut buffer) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("❌ Failed to read from audio device: {:?}", e);
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
        };
        
        if frames_read == 0 {
            continue;
        }
        
        // Process audio data with bounds checking
        let actual_bytes = frames_read as usize * bytes_per_frame as usize;
        
        // Safety check: ensure actual_bytes doesn't exceed buffer size
        let actual_bytes = if actual_bytes > buffer.len() {
            eprintln!("⚠️ Audio data size ({}) exceeds buffer size ({}), truncating", actual_bytes, buffer.len());
            buffer.len()
        } else {
            actual_bytes
        };
        
        let audio_data = &buffer[..actual_bytes];
        
        // Convert to f32 and process (mono conversion, resampling)
        let processed_audio = process_audio_chunk(
            audio_data,
            bits_per_sample,
            channels,
            format.get_samplespersec(),
            device_info.sample_rate
        );
        
        total_samples += processed_audio.len() as u64;
        
        // Emit audio chunk event periodically (every 100ms)
        let now = Instant::now();
        if now.duration_since(last_emit) > Duration::from_millis(100) {
            // Convert to PCM16 for frontend
            let pcm16_data: Vec<i16> = processed_audio.iter()
                .map(|&sample| (sample * 32767.0).clamp(-32768.0, 32767.0) as i16)
                .collect();
            
            // Convert to bytes
            let audio_bytes: Vec<u8> = pcm16_data.iter()
                .flat_map(|&sample| sample.to_le_bytes())
                .collect();
            
            // Calculate audio level
            let level = calculate_audio_level(&processed_audio);
            
            // Emit to frontend
            let _ = app_handle.emit("audio-chunk", serde_json::json!({
                "deviceId": device_id,
                "audioData": base64::prelude::BASE64_STANDARD.encode(&audio_bytes),
                "sampleRate": device_info.sample_rate,
                "channels": 1, // Always mono after processing
                "level": level,
                "timestamp": chrono::Utc::now().timestamp_millis(),
                "duration": start_time.elapsed().as_secs(),
                "totalSamples": total_samples
            }));
            
            last_emit = now;
        }
    }
    
    // Cleanup
    let _ = audio_client.stop_stream();
    println!("Audio capture stopped");
    
    Ok(())
}

// Helper function to find WASAPI device
fn find_wasapi_device(device_info: &AudioLoopbackDevice) -> Result<Device> {
    let direction = match device_info.device_type {
        DeviceType::Render => Direction::Render,
        DeviceType::Capture => Direction::Capture,
    };
    
    let device_collection = DeviceCollection::new(&direction)
        .map_err(|e| anyhow::anyhow!("Failed to create device collection: {:?}", e))?;
    let device_count = device_collection.get_nbr_devices()
        .map_err(|e| anyhow::anyhow!("Failed to get device count: {:?}", e))?;
    
    for i in 0..device_count {
        if let Ok(device) = device_collection.get_device_at_index(i) {
            if let Ok(id) = device.get_id() {
                if id == device_info.id {
                    return Ok(device);
                }
            }
        }
    }
    
    Err(anyhow::anyhow!("Could not find device with ID: {}", device_info.id))
}

// Audio processing functions
fn process_audio_chunk(
    audio_data: &[u8],
    bits_per_sample: u16,
    channels: u16,
    input_sample_rate: u32,
    output_sample_rate: u32
) -> Vec<f32> {
    // Safety check: ensure audio_data is not empty
    if audio_data.is_empty() {
        return Vec::new();
    }
    
    // Convert bytes to f32 samples
    let mut f32_samples = Vec::new();
    
    match bits_per_sample {
        32 => {
            // Safety check: ensure data length is multiple of 4
            if audio_data.len() % 4 != 0 {
                eprintln!("⚠️ Audio data length ({}) not multiple of 4 for 32-bit samples", audio_data.len());
                return Vec::new();
            }
            
            for chunk in audio_data.chunks_exact(4) {
                let bytes = [chunk[0], chunk[1], chunk[2], chunk[3]];
                let sample = f32::from_le_bytes(bytes);
                if sample.is_finite() && sample.abs() <= 2.0 {
                    f32_samples.push(sample);
                } else {
                    f32_samples.push(0.0);
                }
            }
        },
        16 => {
            // Safety check: ensure data length is multiple of 2
            if audio_data.len() % 2 != 0 {
                eprintln!("⚠️ Audio data length ({}) not multiple of 2 for 16-bit samples", audio_data.len());
                return Vec::new();
            }
            
            for chunk in audio_data.chunks_exact(2) {
                let bytes = [chunk[0], chunk[1]];
                let sample_i16 = i16::from_le_bytes(bytes);
                let sample_f32 = sample_i16 as f32 / 32768.0;
                f32_samples.push(sample_f32);
            }
        },
        _ => {
            eprintln!("⚠️ Unsupported bits per sample: {}", bits_per_sample);
            return Vec::new();
        }
    }
    
    // Convert stereo to mono if needed
    if channels == 2 {
        f32_samples = f32_samples.chunks(2)
            .map(|chunk| {
                if chunk.len() == 2 {
                    (chunk[0] + chunk[1]) * 0.5
                } else {
                    chunk[0]
                }
            })
            .collect();
    }
    
    // Simple resampling
    if input_sample_rate != output_sample_rate {
        if input_sample_rate == 48000 && output_sample_rate == 16000 {
            // 48kHz to 16kHz - take every 3rd sample
            f32_samples = f32_samples.iter().step_by(3).copied().collect();
        } else if input_sample_rate == 44100 && output_sample_rate == 16000 {
            // 44.1kHz to 16kHz
            let factor = input_sample_rate as f32 / output_sample_rate as f32;
            f32_samples = (0..f32_samples.len())
                .step_by(factor as usize)
                .filter_map(|i| f32_samples.get(i).copied())
                .collect();
        }
        // Add more resampling cases as needed
    }
    
    f32_samples
}

fn calculate_audio_level(audio_data: &[f32]) -> f32 {
    if audio_data.is_empty() {
        return -60.0;
    }
    
    let rms = (audio_data.iter().map(|&x| x * x).sum::<f32>() / audio_data.len() as f32).sqrt();
    
    if rms > 0.0 {
        20.0 * rms.log10().max(-60.0)
    } else {
        -60.0
    }
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
        assert_eq!(loaded_settings.selectedLoopbackDevice, settings.selectedLoopbackDevice);
        assert_eq!(loaded_settings.loopbackEnabled, settings.loopbackEnabled);
    }
}