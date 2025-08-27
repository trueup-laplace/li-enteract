//! macOS-specific audio capture implementation using Core Audio

pub mod core_audio_bindings;
pub mod core_audio_enumerator;
pub mod core_audio_capture;
pub mod audio_tap;
pub mod aggregate_device;

// Re-export main types
pub use core_audio_bindings::*;
pub use core_audio_enumerator::CoreAudioDeviceEnumerator;
pub use core_audio_capture::CoreAudioCaptureEngine;
pub use audio_tap::AudioTap;
pub use aggregate_device::AggregateDevice;
