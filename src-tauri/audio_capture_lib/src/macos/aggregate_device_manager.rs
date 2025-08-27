//! Aggregate device management for macOS
//! This module provides high-level functions for working with aggregate devices

use crate::types::{AudioCaptureResult, AggregateDeviceConfig, AudioDevice, DeviceType};
use crate::macos::core_audio_enumerator::CoreAudioDeviceEnumerator;
use crate::macos::aggregate_device::AggregateDevice;
use crate::macos::core_audio_bindings::*;

/// Aggregate device manager for creating and configuring aggregate devices
pub struct AggregateDeviceManager {
    enumerator: CoreAudioDeviceEnumerator,
}

impl AggregateDeviceManager {
    /// Create a new aggregate device manager
    pub fn new() -> AudioCaptureResult<Self> {
        Ok(Self {
            enumerator: CoreAudioDeviceEnumerator::new()?,
        })
    }
    
    /// Get all available input devices (microphones)
    pub async fn get_available_input_devices(&self) -> AudioCaptureResult<Vec<AudioDevice>> {
        self.enumerator.get_input_devices().await
    }
    
    /// Get all available output devices (speakers)
    pub async fn get_available_output_devices(&self) -> AudioCaptureResult<Vec<AudioDevice>> {
        self.enumerator.get_output_devices().await
    }
    
    /// Get all available aggregate devices
    pub async fn get_available_aggregate_devices(&self) -> AudioCaptureResult<Vec<AudioDevice>> {
        self.enumerator.get_aggregate_devices().await
    }
    
    /// Get the default input device (microphone)
    pub async fn get_default_input_device(&self) -> AudioCaptureResult<Option<AudioDevice>> {
        self.enumerator.get_default_input_device().await
    }
    
    /// Get the default output device (speakers)
    pub async fn get_default_output_device(&self) -> AudioCaptureResult<Option<AudioDevice>> {
        self.enumerator.get_default_output_device().await
    }
    
    /// Find an existing aggregate device or create a new one
    pub async fn find_or_create_aggregate_device(&self, name: &str) -> AudioCaptureResult<AggregateDevice> {
        // First, try to find an existing aggregate device with the given name
        let aggregate_devices = self.get_available_aggregate_devices().await?;
        
        for device in aggregate_devices {
            if device.name == name {
                let device_id: AudioAggregateDeviceID = device.id.parse()
                    .map_err(|_| crate::types::AudioCaptureError::CoreAudioError(
                        "Invalid aggregate device ID".to_string()
                    ))?;
                return AggregateDevice::new(device_id);
            }
        }
        
        // If no device with that name exists, create one programmatically
        println!("[AggregateDeviceManager] Creating new aggregate device: {}", name);
        let device_id = self.create_aggregate_device(name).await?;
        AggregateDevice::new(device_id)
    }
    
    /// Create a new aggregate device programmatically
    async fn create_aggregate_device(&self, name: &str) -> AudioCaptureResult<AudioAggregateDeviceID> {
        use crate::macos::aggregate_device_creator;
        
        // Use the objc2-core-audio implementation
        let device_id = aggregate_device_creator::create_simple_aggregate_device(name)?;
        Ok(device_id)
    }
    
    /// Setup an aggregate device to capture default microphone input
    pub async fn setup_microphone_aggregate_device(&self, aggregate_device_name: &str) -> AudioCaptureResult<AggregateDevice> {
        use crate::macos::aggregate_device_creator;
        
        // Get the default input device (microphone)
        let default_input = self.get_default_input_device().await?
            .ok_or_else(|| crate::types::AudioCaptureError::CoreAudioError(
                "No default input device found".to_string()
            ))?;
        
        println!("[AggregateDeviceManager] Found default input device: {} ({})", default_input.name, default_input.uid);
        
        // Create the aggregate device with the default input device
        let device_id = aggregate_device_creator::create_microphone_aggregate_device(aggregate_device_name)?;
        let aggregate_device = AggregateDevice::new(device_id)?;
        
        println!("[AggregateDeviceManager] Created microphone aggregate device with default input");
        
        Ok(aggregate_device)
    }
    
    /// Setup an aggregate device with specific input and output devices
    pub async fn setup_custom_aggregate_device(
        &self,
        aggregate_device_name: &str,
        input_device_uid: &str,
        output_device_uid: Option<&str>,
    ) -> AudioCaptureResult<AggregateDevice> {
        // Find or create the aggregate device
        let mut aggregate_device = self.find_or_create_aggregate_device(aggregate_device_name).await?;
        
        // Add the input device
        aggregate_device.add_sub_device(input_device_uid)?;
        println!("[AggregateDeviceManager] Added input device: {}", input_device_uid);
        
        // Add the output device if specified
        if let Some(output_uid) = output_device_uid {
            aggregate_device.add_sub_device(output_uid)?;
            println!("[AggregateDeviceManager] Added output device: {}", output_uid);
        }
        
        Ok(aggregate_device)
    }
    
    /// Create an "All Audio" aggregate device (like the Swift code)
    pub async fn create_all_audio_aggregate_device(&self, name: &str) -> AudioCaptureResult<AggregateDevice> {
        use crate::macos::aggregate_device_creator;
        
        // Create the aggregate device using objc2-core-audio
        let device_id = aggregate_device_creator::create_all_audio_aggregate_device(name)?;
        let aggregate_device = AggregateDevice::new(device_id)?;
        
        println!("[AggregateDeviceManager] Created All Audio aggregate device: {} (ID: {})", name, device_id);
        Ok(aggregate_device)
    }
    
    /// Add a tap to an aggregate device
    pub async fn add_tap_to_aggregate_device(&self, aggregate_device: &mut AggregateDevice, tap_uid: &str) -> AudioCaptureResult<()> {
        aggregate_device.add_tap(tap_uid)?;
        println!("[AggregateDeviceManager] Added tap {} to aggregate device", tap_uid);
        Ok(())
    }
    
    /// Get a list of all devices with readable names for UI selection
    pub async fn get_device_list_for_ui(&self) -> AudioCaptureResult<Vec<AudioDevice>> {
        let mut all_devices = Vec::new();
        
        // Get input devices
        let input_devices = self.get_available_input_devices().await?;
        for mut device in input_devices {
            device.name = format!("🎤 {}", device.name);
            all_devices.push(device);
        }
        
        // Get output devices
        let output_devices = self.get_available_output_devices().await?;
        for mut device in output_devices {
            device.name = format!("🔊 {}", device.name);
            all_devices.push(device);
        }
        
        // Get aggregate devices
        let aggregate_devices = self.get_available_aggregate_devices().await?;
        for mut device in aggregate_devices {
            device.name = format!("🔗 {}", device.name);
            all_devices.push(device);
        }
        
        Ok(all_devices)
    }
    
    /// Create an aggregate device configuration for the given devices
    pub async fn create_aggregate_config(
        &self,
        name: &str,
        input_device_uid: &str,
        output_device_uid: Option<&str>,
        is_private: bool,
        auto_start: bool,
    ) -> AudioCaptureResult<AggregateDeviceConfig> {
        let mut sub_devices = vec![input_device_uid.to_string()];
        
        if let Some(output_uid) = output_device_uid {
            sub_devices.push(output_uid.to_string());
        }
        
        Ok(AggregateDeviceConfig {
            name: name.to_string(),
            sub_devices,
            taps: Vec::new(),
            is_private,
            auto_start,
            auto_stop: false,
        })
    }
}

/// Factory functions for common aggregate device operations
pub mod factory {
    use super::*;
    
    /// Create a simple aggregate device with default microphone
    pub async fn create_microphone_aggregate_device() -> AudioCaptureResult<AggregateDevice> {
        let manager = AggregateDeviceManager::new()?;
        manager.setup_microphone_aggregate_device("Callio Microphone Aggregate").await
    }
    
    /// Create an aggregate device with custom configuration
    pub async fn create_custom_aggregate_device(
        name: &str,
        input_device_uid: &str,
        output_device_uid: Option<&str>,
    ) -> AudioCaptureResult<AggregateDevice> {
        let manager = AggregateDeviceManager::new()?;
        manager.setup_custom_aggregate_device(name, input_device_uid, output_device_uid).await
    }
    
    /// Get all available devices for UI selection
    pub async fn get_ui_device_list() -> AudioCaptureResult<Vec<AudioDevice>> {
        let manager = AggregateDeviceManager::new()?;
        manager.get_device_list_for_ui().await
    }
    
    /// Get only input devices for UI selection
    pub async fn get_ui_input_devices() -> AudioCaptureResult<Vec<AudioDevice>> {
        let manager = AggregateDeviceManager::new()?;
        let devices = manager.get_available_input_devices().await?;
        Ok(devices.into_iter()
            .map(|mut d| {
                d.name = format!("🎤 {}", d.name);
                d
            })
            .collect())
    }
    
    /// Create an "All Audio" aggregate device (like the Swift MainView)
    pub async fn create_all_audio_aggregate_device(name: &str) -> AudioCaptureResult<AggregateDevice> {
        let manager = AggregateDeviceManager::new()?;
        manager.create_all_audio_aggregate_device(name).await
    }
    
    /// Add a tap to an aggregate device
    pub async fn add_tap_to_aggregate_device(aggregate_device: &mut AggregateDevice, tap_uid: &str) -> AudioCaptureResult<()> {
        let manager = AggregateDeviceManager::new()?;
        manager.add_tap_to_aggregate_device(aggregate_device, tap_uid).await
    }
}
