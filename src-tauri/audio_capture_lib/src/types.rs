//! Core types and data structures for the audio capture library

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Audio capture state management
lazy_static::lazy_static! {
    pub static ref CAPTURE_STATE: Arc<Mutex<CaptureState>> = Arc::new(Mutex::new(CaptureState::default()));
}

/// Represents the current state of audio capture
#[derive(Default, Debug)]
pub struct CaptureState {
    pub is_capturing: bool,
    pub capture_handle: Option<tokio::task::JoinHandle<()>>,
    pub stop_tx: Option<mpsc::Sender<()>>,
}

/// Represents an audio device that can be used for capture
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub uid: String,
    pub is_default: bool,
    pub sample_rate: u32,
    pub channels: u16,
    pub format: String,
    pub device_type: DeviceType,
    pub capture_method: CaptureMethod,
}

/// Type of audio device
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceType {
    Render,    // Output device (speakers, headphones)
    Capture,   // Input device (microphone)
    Aggregate, // Virtual device combining multiple sources
}

/// Method used to capture audio from the device
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CaptureMethod {
    Direct,           // Direct capture (microphone)
    Loopback,         // Loopback capture (system audio)
    AudioTap,         // Audio tap on specific processes
    AggregateDevice,  // Aggregate device with taps
}

/// Configuration for audio capture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureConfig {
    pub device_id: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_size: usize,
    pub capture_method: CaptureMethod,
    pub enable_transcription: bool,
    pub transcription_buffer_duration: f32,
    pub transcription_interval_ms: u64,
    pub min_audio_length: f32,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            device_id: String::new(),
            sample_rate: 16000,
            channels: 1,
            buffer_size: 4096,
            capture_method: CaptureMethod::Direct,
            enable_transcription: true,
            transcription_buffer_duration: 4.0,
            transcription_interval_ms: 800,
            min_audio_length: 1.5,
        }
    }
}

/// Audio data chunk with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioChunk {
    pub device_id: String,
    pub audio_data: Vec<u8>,
    pub sample_rate: u32,
    pub channels: u16,
    pub level: f32,
    pub timestamp: i64,
    pub duration: u64,
    pub total_samples: u64,
}

/// Transcription result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub confidence: f32,
    pub start_time: f32,
    pub end_time: f32,
    pub timestamp: i64,
}

/// Audio tap configuration (macOS specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTapConfig {
    pub name: String,
    pub processes: Vec<String>,
    pub is_private: bool,
    pub is_process_restore_enabled: bool,
    pub mute: TapMute,
    pub mixdown: TapMixdown,
    pub exclusive: bool,
    pub device: Option<String>,
    pub stream_index: Option<u32>,
}

/// Tap mute behavior
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TapMute {
    Unmuted,
    Muted,
    MutedWithFeedback,
}

/// Tap mixdown configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TapMixdown {
    DeviceFormat,
    Mono,
    Stereo,
}

/// Aggregate device configuration (macOS specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateDeviceConfig {
    pub name: String,
    pub sub_devices: Vec<String>,
    pub taps: Vec<String>,
    pub is_private: bool,
    pub auto_start: bool,
    pub auto_stop: bool,
}

/// Error types for the audio capture library
#[derive(Debug, thiserror::Error)]
pub enum AudioCaptureError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    
    #[error("Failed to initialize audio capture: {0}")]
    InitializationFailed(String),
    
    #[error("Audio capture already in progress")]
    AlreadyCapturing,
    
    #[error("Audio capture not in progress")]
    NotCapturing,
    
    #[error("Core Audio error: {0}")]
    CoreAudioError(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for audio capture operations
pub type AudioCaptureResult<T> = Result<T, AudioCaptureError>;
