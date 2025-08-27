// src-tauri/src/audio_loopback/mod.rs
// Cross-platform audio loopback module with platform-specific implementations

pub mod types;
pub mod audio_processor;
pub mod quality_filter;
pub mod settings;

// Platform-specific modules
#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "macos")]
pub mod macos;

// Re-export main types and functions
pub use types::{CAPTURE_STATE, CaptureState, AudioLoopbackDevice, DeviceType, LoopbackMethod, AudioDeviceSettings};
pub use audio_processor::*;
pub use settings::*;

// Platform-specific re-exports
#[cfg(target_os = "windows")]
pub use windows::*;
#[cfg(target_os = "macos")]
pub use macos::*;
