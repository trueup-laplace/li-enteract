//! Device enumeration functionality for discovering audio devices

use crate::types::{AudioDevice, AudioCaptureResult};
use async_trait::async_trait;

/// Trait for enumerating audio devices on different platforms
#[async_trait]
pub trait DeviceEnumerator {
    /// Enumerate all available audio devices
    async fn enumerate_devices(&self) -> AudioCaptureResult<Vec<AudioDevice>>;
    
    /// Find a specific device by ID
    async fn find_device_by_id(&self, device_id: &str) -> AudioCaptureResult<Option<AudioDevice>>;
    
    /// Get the default device for a specific type
    async fn get_default_device(&self, device_type: crate::types::DeviceType) -> AudioCaptureResult<Option<AudioDevice>>;
    
    /// Refresh the device list
    async fn refresh_devices(&mut self) -> AudioCaptureResult<()>;
}

/// Factory function to create the appropriate device enumerator for the current platform
pub fn create_device_enumerator() -> AudioCaptureResult<Box<dyn DeviceEnumerator + Send + Sync>> {
    #[cfg(target_os = "macos")]
    {
        use crate::macos::CoreAudioDeviceEnumerator;
        Ok(Box::new(CoreAudioDeviceEnumerator::new()?))
    }
    
    #[cfg(target_os = "windows")]
    {
        use crate::windows::WASAPIDeviceEnumerator;
        Ok(Box::new(WASAPIDeviceEnumerator::new()?))
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err(crate::types::AudioCaptureError::Other(
            "Unsupported platform".to_string()
        ))
    }
}

/// Utility functions for device enumeration
pub mod utils {
    use super::*;
    
    /// Filter devices by type
    pub fn filter_devices_by_type(
        devices: &[AudioDevice], 
        device_type: crate::types::DeviceType
    ) -> Vec<AudioDevice> {
        devices.iter()
            .filter(|device| device.device_type == device_type)
            .cloned()
            .collect()
    }
    
    /// Filter devices by capture method
    pub fn filter_devices_by_capture_method(
        devices: &[AudioDevice], 
        capture_method: crate::types::CaptureMethod
    ) -> Vec<AudioDevice> {
        devices.iter()
            .filter(|device| device.capture_method == capture_method)
            .cloned()
            .collect()
    }
    
    /// Find devices that support a specific sample rate
    pub fn filter_devices_by_sample_rate(
        devices: &[AudioDevice], 
        sample_rate: u32
    ) -> Vec<AudioDevice> {
        devices.iter()
            .filter(|device| device.sample_rate == sample_rate)
            .cloned()
            .collect()
    }
    
    /// Sort devices by name
    pub fn sort_devices_by_name(devices: &mut [AudioDevice]) {
        devices.sort_by(|a, b| a.name.cmp(&b.name));
    }
    
    /// Sort devices by ID
    pub fn sort_devices_by_id(devices: &mut [AudioDevice]) {
        devices.sort_by(|a, b| a.id.cmp(&b.id));
    }
}
