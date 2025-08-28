// src-tauri/src/audio_loopback/types.rs
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

// Audio capture state management
lazy_static::lazy_static! {
    pub static ref CAPTURE_STATE: Arc<Mutex<CaptureState>> = Arc::new(Mutex::new(CaptureState::default()));
}

#[derive(Default)]
pub struct CaptureState {
    pub is_capturing: bool,
    pub capture_handle: Option<tokio::task::JoinHandle<()>>,
    pub stop_tx: Option<mpsc::Sender<()>>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
