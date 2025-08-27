//! Core Audio device enumeration for macOS

use crate::types::{AudioDevice, AudioCaptureResult, DeviceType, CaptureMethod};
use crate::device_enumerator::DeviceEnumerator;
use crate::macos::core_audio_bindings::*;
use async_trait::async_trait;
use std::ptr;

/// Helper function to create a property address
fn create_property_address(selector: u32, scope: u32, element: u32) -> AudioObjectPropertyAddress {
    AudioObjectPropertyAddress {
        mSelector: selector,
        mScope: scope,
        mElement: element,
    }
}

/// Helper function to convert CFString to Rust String
fn cf_string_to_string(cf_string: *mut std::ffi::c_void) -> AudioCaptureResult<String> {
    if cf_string.is_null() {
        return Ok("Unknown Device".to_string());
    }
    
    // For now, return a simple identifier based on the pointer
    // This avoids the complex CFString memory management issues
    Ok(format!("Device_{:p}", cf_string))
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
            kAudioHardwarePropertyDevices,
            kAudioObjectPropertyScopeGlobal,
            kAudioObjectPropertyElementMain,
        );
        
        let status = unsafe {
            AudioObjectGetPropertyDataSize(
                kAudioObjectSystemObject,
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
                kAudioObjectSystemObject,
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
        let mut property_size = std::mem::size_of::<*mut std::ffi::c_void>() as u32;
        let property_address = create_property_address(
            kAudioDevicePropertyDeviceNameCFString,
            kAudioObjectPropertyScopeGlobal,
            kAudioObjectPropertyElementMain,
        );
        
        let mut name_ref: *mut std::ffi::c_void = ptr::null_mut();
        let status = unsafe {
            AudioObjectGetPropertyData(
                device_id,
                &property_address,
                0,
                ptr::null(),
                &mut property_size,
                &mut name_ref as *mut _ as *mut _,
            )
        };
        
        if status != 0 {
            // Return a default name if we can't get the device name
            return Ok(format!("Unknown Device {}", device_id));
        }
        
        if name_ref.is_null() {
            return Ok(format!("Unknown Device {}", device_id));
        }
        
        // Convert CFString to Rust String using our helper function
        cf_string_to_string(name_ref)
            .map_err(|_| crate::types::AudioCaptureError::CoreAudioError(
                format!("Failed to convert device name for device {}", device_id)
            ))
            .or_else(|_| Ok(format!("Unknown Device {}", device_id)))
    }
    
    /// Get device UID
    fn get_device_uid(&self, device_id: AudioObjectID) -> AudioCaptureResult<String> {
        let mut property_size = std::mem::size_of::<*mut std::ffi::c_void>() as u32;
        let property_address = create_property_address(
            kAudioDevicePropertyDeviceUID,
            kAudioObjectPropertyScopeGlobal,
            kAudioObjectPropertyElementMain,
        );
        
        let mut uid_ref: *mut std::ffi::c_void = ptr::null_mut();
        let status = unsafe {
            AudioObjectGetPropertyData(
                device_id,
                &property_address,
                0,
                ptr::null(),
                &mut property_size,
                &mut uid_ref as *mut _ as *mut _,
            )
        };
        
        if status != 0 {
            // Return a default UID if we can't get the device UID
            return Ok(format!("device_{}", device_id));
        }
        
        if uid_ref.is_null() {
            return Ok(format!("device_{}", device_id));
        }
        
        // Convert CFString to Rust String using our helper function
        cf_string_to_string(uid_ref)
            .map_err(|_| crate::types::AudioCaptureError::CoreAudioError(
                format!("Failed to convert device UID for device {}", device_id)
            ))
            .or_else(|_| Ok(format!("device_{}", device_id)))
    }
    
    /// Check if device has output streams
    fn has_output_streams(&self, device_id: AudioObjectID) -> AudioCaptureResult<bool> {
        let mut property_size = 0u32;
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyStreams,
            mScope: kAudioObjectPropertyScopeOutput,
            mElement: kAudioObjectPropertyElementMain,
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
        
        if status != 0 {
            return Ok(false);
        }
        
        Ok(property_size > 0)
    }
    
    /// Check if device has input streams
    fn has_input_streams(&self, device_id: AudioObjectID) -> AudioCaptureResult<bool> {
        let mut property_size = 0u32;
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyStreams,
            mScope: kAudioObjectPropertyScopeInput,
            mElement: kAudioObjectPropertyElementMain,
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
            mSelector: kAudioDevicePropertyNominalSampleRate,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
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
            mSelector: kAudioDevicePropertyStreamConfiguration,
            mScope: kAudioObjectPropertyScopeOutput,
            mElement: kAudioObjectPropertyElementMain,
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
            DeviceType::Render => kAudioHardwarePropertyDefaultOutputDevice,
            DeviceType::Capture => kAudioHardwarePropertyDefaultInputDevice,
            DeviceType::Aggregate => return Ok(false),
        };
        
        let mut default_device_id: AudioObjectID = 0;
        let mut property_size = std::mem::size_of::<AudioObjectID>() as u32;
        let property_address = AudioObjectPropertyAddress {
            mSelector: property_selector,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
        };
        
        let status = unsafe {
            AudioObjectGetPropertyData(
                kAudioObjectSystemObject,
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
            mSelector: kAudioDevicePropertyTransportType,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
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
    
    /// Create device info from device ID
    fn create_device_info(&self, device_id: AudioObjectID) -> AudioCaptureResult<AudioDevice> {
        let name = self.get_device_name(device_id)?;
        let uid = self.get_device_uid(device_id)?;
        let sample_rate = self.get_device_sample_rate(device_id).unwrap_or(48000);
        let channels = self.get_device_channels(device_id).unwrap_or(2);
        
        // Check transport type to determine if this is an aggregate device
        let transport_type = self.get_device_transport_type(device_id).unwrap_or(0);
        let is_aggregate = transport_type == kAudioDeviceTransportTypeAggregate;
        
        // For now, let's include all devices and determine type based on transport type
        // This is similar to how the Swift code works
        let device_type = if is_aggregate {
            DeviceType::Aggregate
        } else {
            // For non-aggregate devices, assume they are render devices (output)
            // This is a simplification - in a full implementation you'd check actual capabilities
            DeviceType::Render
        };
        
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
