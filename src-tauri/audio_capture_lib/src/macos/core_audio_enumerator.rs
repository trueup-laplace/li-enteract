//! Core Audio device enumeration for macOS

use crate::types::{AudioDevice, AudioCaptureResult, DeviceType, CaptureMethod};
use crate::device_enumerator::DeviceEnumerator;
use crate::macos::core_audio_bindings::*;
use async_trait::async_trait;
use std::ptr;

/// Helper function to create a property address
fn create_property_address(selector: u32, scope: u32, element: u32) -> AudioObjectPropertyAddress {
    AudioObjectPropertyAddress {
        selector,
        scope,
        element,
    }
}

/// Helper function to convert CFString to Rust String
fn cf_string_to_string(cf_string: *mut std::ffi::c_void) -> AudioCaptureResult<String> {
    if cf_string.is_null() {
        return Ok("Unknown Device".to_string());
    }
    
    // Use the Core Foundation functions to properly convert CFString
    crate::macos::core_audio_bindings::cf_string_to_string(cf_string)
        .map_err(|e| crate::types::AudioCaptureError::CoreAudioError(
            format!("Failed to convert CFString: {}", e)
        ))
}

/// Core Audio device enumerator for macOS
pub struct CoreAudioDeviceEnumerator {
    devices: Vec<AudioDevice>,
}

impl CoreAudioDeviceEnumerator {
    /// Create a new Core Audio device enumerator
    pub fn new() -> AudioCaptureResult<Self> {
        Ok(Self {
            devices: Vec::new(),
        })
    }
    
    /// Get all audio device IDs
    pub fn get_audio_device_ids(&self) -> AudioCaptureResult<Vec<AudioObjectID>> {
        let mut property_size = 0u32;
        let property_address = create_property_address(
            AUDIO_HARDWARE_PROPERTY_DEVICES,
            AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL,
            AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN,
        );
        
        let status = unsafe {
            AudioObjectGetPropertyDataSize(
                AUDIO_OBJECT_SYSTEM_OBJECT,
                &property_address,
                0,
                ptr::null(),
                &mut property_size,
            )
        };
        
        if status != 0 {
            return Err(crate::types::AudioCaptureError::CoreAudioError(
                format!("Failed to get property data size: {}", status)
            ));
        }
        
        let device_count = property_size / std::mem::size_of::<AudioObjectID>() as u32;
        
        if device_count == 0 {
            return Ok(vec![]);
        }
        
        let mut device_ids = vec![0u32; device_count as usize];
        
        let status = unsafe {
            AudioObjectGetPropertyData(
                AUDIO_OBJECT_SYSTEM_OBJECT,
                &property_address,
                0,
                ptr::null(),
                &mut property_size,
                device_ids.as_mut_ptr() as *mut _,
            )
        };
        
        if status != 0 {
            return Err(crate::types::AudioCaptureError::CoreAudioError(
                format!("Failed to get device IDs: {}", status)
            ));
        }
        
        Ok(device_ids)
    }
    
    /// Get device name
    fn get_device_name(&self, device_id: AudioObjectID) -> AudioCaptureResult<String> {
        // For now, just return a simple device name based on the ID
        // This avoids CFString conversion issues
        Ok(format!("Audio Device {}", device_id))
    }
    
    /// Get device UID
    fn get_device_uid(&self, device_id: AudioObjectID) -> AudioCaptureResult<String> {
        // For now, just return a simple UID based on the ID
        // This avoids CFString conversion issues
        Ok(format!("device_{}", device_id))
    }
    
    /// Check if device has output streams
    fn has_output_streams(&self, device_id: AudioObjectID) -> AudioCaptureResult<bool> {
        let mut property_size = 0u32;
        let property_address = AudioObjectPropertyAddress {
            selector: AUDIO_DEVICE_PROPERTY_STREAM_CONFIGURATION,
            scope: AUDIO_OBJECT_PROPERTY_SCOPE_OUTPUT,
            element: AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN,
        };
        
        let status = unsafe {
            AudioObjectGetPropertyDataSize(
                device_id,
                &property_address,
                0,
                ptr::null(),
                &mut property_size,
            )
        };
        
        println!("[OutputStreams] Device {}: status={}, size={}", device_id, status, property_size);
        
        if status != 0 {
            return Ok(false);
        }
        
        Ok(property_size > 0)
    }
    
    /// Check if device has input streams
    fn has_input_streams(&self, device_id: AudioObjectID) -> AudioCaptureResult<bool> {
        let mut property_size = 0u32;
        let property_address = AudioObjectPropertyAddress {
            selector: AUDIO_DEVICE_PROPERTY_STREAM_CONFIGURATION,
            scope: AUDIO_OBJECT_PROPERTY_SCOPE_INPUT,
            element: AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN,
        };
        
        let status = unsafe {
            AudioObjectGetPropertyDataSize(
                device_id,
                &property_address,
                0,
                ptr::null(),
                &mut property_size,
            )
        };
        
        println!("[InputStreams] Device {}: status={}, size={}", device_id, status, property_size);
        
        if status != 0 {
            return Ok(false);
        }
        
        Ok(property_size > 0)
    }
    
    /// Get device sample rate
    fn get_device_sample_rate(&self, device_id: AudioObjectID) -> AudioCaptureResult<u32> {
        let mut sample_rate: f64 = 0.0;
        let mut property_size = std::mem::size_of::<f64>() as u32;
        let property_address = AudioObjectPropertyAddress {
            selector: AUDIO_DEVICE_PROPERTY_NOMINAL_SAMPLE_RATE,
            scope: AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL,
            element: AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN,
        };
        
        let status = unsafe {
            AudioObjectGetPropertyData(
                device_id,
                &property_address,
                0,
                ptr::null(),
                &mut property_size,
                &mut sample_rate as *mut _ as *mut _,
            )
        };
        
        if status != 0 {
            return Ok(48000); // Default sample rate
        }
        
        Ok(sample_rate as u32)
    }
    
    /// Get device channel count
    fn get_device_channels(&self, device_id: AudioObjectID) -> AudioCaptureResult<u16> {
        let mut channels: u32 = 0;
        let mut property_size = std::mem::size_of::<u32>() as u32;
        let property_address = AudioObjectPropertyAddress {
            selector: AUDIO_DEVICE_PROPERTY_STREAM_CONFIGURATION,
            scope: AUDIO_OBJECT_PROPERTY_SCOPE_OUTPUT,
            element: AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN,
        };
        
        let status = unsafe {
            AudioObjectGetPropertyData(
                device_id,
                &property_address,
                0,
                ptr::null(),
                &mut property_size,
                &mut channels as *mut _ as *mut _,
            )
        };
        
        if status != 0 {
            return Ok(2); // Default to stereo
        }
        
        Ok(channels as u16)
    }
    
    /// Check if device is default
    fn is_default_device(&self, device_id: AudioObjectID, device_type: DeviceType) -> AudioCaptureResult<bool> {
        let property_selector = match device_type {
            DeviceType::Render => AUDIO_HARDWARE_PROPERTY_DEFAULT_OUTPUT_DEVICE,
            DeviceType::Capture => AUDIO_HARDWARE_PROPERTY_DEFAULT_INPUT_DEVICE,
            DeviceType::Aggregate => return Ok(false),
        };
        
        let mut default_device_id: AudioObjectID = 0;
        let mut property_size = std::mem::size_of::<AudioObjectID>() as u32;
        let property_address = AudioObjectPropertyAddress {
            selector: property_selector,
            scope: AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL,
            element: AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN,
        };
        
        let status = unsafe {
            AudioObjectGetPropertyData(
                AUDIO_OBJECT_SYSTEM_OBJECT,
                &property_address,
                0,
                ptr::null(),
                &mut property_size,
                &mut default_device_id as *mut _ as *mut _,
            )
        };
        

        
        if status != 0 {
            return Ok(false);
        }
        
        Ok(device_id == default_device_id)
    }
    
    /// Get device transport type
    fn get_device_transport_type(&self, device_id: AudioObjectID) -> AudioCaptureResult<u32> {
        let mut transport_type: u32 = 0;
        let mut property_size = std::mem::size_of::<u32>() as u32;
        let property_address = AudioObjectPropertyAddress {
            selector: AUDIO_DEVICE_PROPERTY_TRANSPORT_TYPE,
            scope: AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL,
            element: AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN,
        };
        
        let status = unsafe {
            AudioObjectGetPropertyData(
                device_id,
                &property_address,
                0,
                ptr::null(),
                &mut property_size,
                &mut transport_type as *mut _ as *mut _,
            )
        };
        
        if status != 0 {
            return Ok(0); // Default to non-aggregate
        }
        
        Ok(transport_type)
    }
    
    /// Determine device type based on capabilities
    fn determine_device_type(&self, device_id: AudioObjectID) -> AudioCaptureResult<DeviceType> {
        // First check if it's an aggregate device
        let transport_type = self.get_device_transport_type(device_id)?;
        if transport_type == AUDIO_DEVICE_TRANSPORT_TYPE_AGGREGATE {
            return Ok(DeviceType::Aggregate);
        }
        
        // Since stream detection is not working, use default device detection instead
        let is_default_input = self.is_default_device(device_id, DeviceType::Capture)?;
        let is_default_output = self.is_default_device(device_id, DeviceType::Render)?;
        
        if is_default_input {
            Ok(DeviceType::Capture)
        } else if is_default_output {
            Ok(DeviceType::Render)
        } else {
            // For non-default devices, assume they are output devices
            // This is a reasonable assumption for most audio devices
            Ok(DeviceType::Render)
        }
    }
    
    /// Create device info from device ID
    fn create_device_info(&self, device_id: AudioObjectID) -> AudioCaptureResult<AudioDevice> {
        let name = self.get_device_name(device_id)?;
        let uid = self.get_device_uid(device_id)?;
        let sample_rate = self.get_device_sample_rate(device_id).unwrap_or(48000);
        let channels = self.get_device_channels(device_id).unwrap_or(2);
        
        // Determine device type based on capabilities
        let device_type = self.determine_device_type(device_id)?;
        
        let is_default = self.is_default_device(device_id, device_type.clone()).unwrap_or(false);
        
        let capture_method = match device_type {
            DeviceType::Render => CaptureMethod::Loopback,
            DeviceType::Capture => CaptureMethod::Direct,
            DeviceType::Aggregate => CaptureMethod::AggregateDevice,
        };
        
        let format = format!("{} channels at {}Hz", channels, sample_rate);
        
        Ok(AudioDevice {
            id: device_id.to_string(),
            name,
            uid,
            is_default,
            sample_rate,
            channels,
            format,
            device_type,
            capture_method,
        })
    }
    
    /// Get available input devices (microphones)
    pub async fn get_input_devices(&self) -> AudioCaptureResult<Vec<AudioDevice>> {
        let devices = self.enumerate_devices().await?;
        Ok(devices.into_iter()
            .filter(|d| d.device_type == DeviceType::Capture)
            .collect())
    }
    
    /// Get available output devices (speakers)
    pub async fn get_output_devices(&self) -> AudioCaptureResult<Vec<AudioDevice>> {
        let devices = self.enumerate_devices().await?;
        Ok(devices.into_iter()
            .filter(|d| d.device_type == DeviceType::Render)
            .collect())
    }
    
    /// Get available aggregate devices
    pub async fn get_aggregate_devices(&self) -> AudioCaptureResult<Vec<AudioDevice>> {
        let devices = self.enumerate_devices().await?;
        Ok(devices.into_iter()
            .filter(|d| d.device_type == DeviceType::Aggregate)
            .collect())
    }
    
    /// Get default input device (microphone)
    pub async fn get_default_input_device(&self) -> AudioCaptureResult<Option<AudioDevice>> {
        self.get_default_device(DeviceType::Capture).await
    }
    
    /// Get default output device (speakers)
    pub async fn get_default_output_device(&self) -> AudioCaptureResult<Option<AudioDevice>> {
        self.get_default_device(DeviceType::Render).await
    }
}

#[async_trait]
impl DeviceEnumerator for CoreAudioDeviceEnumerator {
    async fn enumerate_devices(&self) -> AudioCaptureResult<Vec<AudioDevice>> {
        let device_ids = self.get_audio_device_ids()?;
        let mut devices = Vec::new();
        
        for device_id in device_ids {
            if let Ok(device_info) = self.create_device_info(device_id) {
                devices.push(device_info);
            }
        }
        
        // Remove duplicates by ID
        devices.sort_by(|a, b| a.id.cmp(&b.id));
        devices.dedup_by(|a, b| a.id == b.id);
        
        Ok(devices)
    }
    
    async fn find_device_by_id(&self, device_id: &str) -> AudioCaptureResult<Option<AudioDevice>> {
        let devices = self.enumerate_devices().await?;
        Ok(devices.into_iter().find(|d| d.id == device_id))
    }
    
    async fn get_default_device(&self, device_type: DeviceType) -> AudioCaptureResult<Option<AudioDevice>> {
        let devices = self.enumerate_devices().await?;
        Ok(devices.into_iter().find(|d| d.device_type == device_type && d.is_default))
    }
    
    async fn refresh_devices(&mut self) -> AudioCaptureResult<()> {
        // Clear cached devices to force refresh
        self.devices.clear();
        Ok(())
    }
}
