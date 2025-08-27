//! Audio Capture Library
//! 
//! This library provides a cross-platform interface for capturing audio from various sources
//! including microphones, system audio, and application-specific audio streams.
//! 
//! The library is designed to work with Core Audio on macOS and can be integrated
//! with Tauri applications.

pub mod types;
pub mod device_enumerator;
pub mod capture_engine;
pub mod audio_processor;

#[cfg(target_os = "macos")]
pub mod macos;

// Re-export main types and traits
pub use types::*;
pub use device_enumerator::{DeviceEnumerator, create_device_enumerator};
pub use capture_engine::{CaptureEngine, AudioCaptureManager};
pub use audio_processor::*;

// Platform-specific re-exports
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_processor_resample() {
        let input_samples = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = audio_processor::AudioProcessor::resample_audio(&input_samples, 48000, 16000);
        assert!(result.is_ok());
        
        let resampled = result.unwrap();
        assert!(!resampled.is_empty());
    }

    #[test]
    fn test_audio_processor_stereo_to_mono() {
        let stereo_samples = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let result = audio_processor::AudioProcessor::stereo_to_mono(&stereo_samples);
        assert!(result.is_ok());
        
        let mono = result.unwrap();
        assert_eq!(mono.len(), 3);
        assert_eq!(mono[0], 1.5); // (1.0 + 2.0) / 2
    }

    #[test]
    fn test_capture_config_default() {
        let config = CaptureConfig::default();
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.channels, 1);
        assert_eq!(config.buffer_size, 4096);
    }
}
