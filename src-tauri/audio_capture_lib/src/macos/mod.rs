//! macOS-specific audio capture implementation using Core Audio

pub mod core_audio_bindings;
pub mod core_audio_enumerator;
pub mod core_audio_capture;
pub mod audio_tap;
pub mod aggregate_device;
pub mod aggregate_device_manager;
pub mod aggregate_device_creator;
pub mod audio_streamer;

// Re-export main types
pub use core_audio_bindings::*;
pub use core_audio_enumerator::CoreAudioDeviceEnumerator;
pub use core_audio_capture::CoreAudioCaptureEngine;
pub use audio_tap::AudioTap;
pub use aggregate_device::AggregateDevice;

pub use aggregate_device_manager::AggregateDeviceManager;
pub use audio_streamer::{AudioStreamer, AudioBuffer, AudioStreamConfig, factory as audio_streamer_factory};
