// src-tauri/src/audio_loopback/macos/mod.rs
// macOS-specific audio loopback implementation using Core Audio

pub mod audio_recorder;
pub mod capture_engine;
pub mod core_audio_bindings;
pub mod device_enumerator;
pub mod device_loader;

// Include tests module for comprehensive Phase 2 testing
#[cfg(test)]
pub mod tests;

pub use capture_engine::*;
pub use device_enumerator::*;
