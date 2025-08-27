// src-tauri/src/audio_loopback/macos_device_enumerator.rs
use crate::audio_loopback::types::*;
use anyhow::Result;
use core_audio::{
    audio_unit::{AudioUnit, IOType, Scope},
    audio_unit::render_callback::{self, data},
    sys::{
        kAudioHardwarePropertyDevices, kAudioObjectPropertyElementMain,
        kAudioObjectPropertyScopeGlobal, kAudioObjectSystemObject,
        AudioObjectGetPropertyData, AudioObjectGetPropertyDataSize,
        AudioObjectID, AudioObjectPropertyAddress,
    },
};

// macOS Core Audio Device Enumerator Implementation
pub struct CoreAudioLoopbackEnumerator;

impl CoreAudioLoopbackEnumerator {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
    
    pub fn enumerate_loopback_devices(&self) -> Result<Vec<AudioLoopbackDevice>> {
        let mut loopback_devices = Vec::new();
        
        // Get all audio devices
        let device_ids = self.get_audio_device_ids()?;
        
        for device_id in device_ids {
            if let Ok(device_info) = self.create_device_info(device_id) {
                // On macOS, we can capture system audio using various methods
                // For now, we'll include all output devices as potential loopback sources
                if device_info.device_type == DeviceType::Render {
                    loopback_devices.push(device_info);
                }
            }
        }
        
        // Remove duplicates by ID
        loopback_devices.sort_by(|a, b| a.id.cmp(&b.id));
        loopback_devices.dedup_by(|a, b| a.id == b.id);
        
        Ok(loopback_devices)
    }
    
    fn get_audio_device_ids(&self) -> Result<Vec<AudioObjectID>> {
        let mut property_size = 0u32;
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioHardwarePropertyDevices,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
        };
        
        let status = unsafe {
            AudioObjectGetPropertyDataSize(
                kAudioObjectSystemObject,
                &property_address,
                0,
                std::ptr::null(),
                &mut property_size,
            )
        };
        
        if status != 0 {
            return Err(anyhow::anyhow!("Failed to get property data size"));
        }
        
        let device_count = property_size / std::mem::size_of::<AudioObjectID>() as u32;
        let mut device_ids = vec![0u32; device_count as usize];
        
        let status = unsafe {
            AudioObjectGetPropertyData(
                kAudioObjectSystemObject,
                &property_address,
                0,
                std::ptr::null(),
                &mut property_size,
                device_ids.as_mut_ptr() as *mut _,
            )
        };
        
        if status != 0 {
            return Err(anyhow::anyhow!("Failed to get device IDs"));
        }
        
        Ok(device_ids)
    }
    
    fn create_device_info(&self, device_id: AudioObjectID) -> Result<AudioLoopbackDevice> {
        // Get device name
        let name = self.get_device_name(device_id)?;
        
        // Get device UID
        let uid = self.get_device_uid(device_id)?;
        
        // Check if this is an output device
        let device_type = if self.has_output_streams(device_id)? {
            DeviceType::Render
        } else {
            DeviceType::Capture
        };
        
        // Get default device info
        let is_default = self.is_default_device(device_id, &device_type)?;
        
        // Get device format info
        let (sample_rate, channels, format) = self.get_device_format(device_id)?;
        
        Ok(AudioLoopbackDevice {
            id: uid,
            name,
            is_default,
            sample_rate,
            channels,
            format,
            device_type,
            loopback_method: LoopbackMethod::RenderLoopback,
        })
    }
    
    fn get_device_name(&self, device_id: AudioObjectID) -> Result<String> {
        // This is a simplified implementation
        // In a full implementation, you'd use Core Audio APIs to get the actual device name
        Ok(format!("Core Audio Device {}", device_id))
    }
    
    fn get_device_uid(&self, device_id: AudioObjectID) -> Result<String> {
        // This is a simplified implementation
        // In a full implementation, you'd use Core Audio APIs to get the actual device UID
        Ok(format!("device_{}", device_id))
    }
    
    fn has_output_streams(&self, _device_id: AudioObjectID) -> Result<bool> {
        // Simplified implementation - assume all devices have output streams
        // In a full implementation, you'd check the actual device capabilities
        Ok(true)
    }
    
    fn is_default_device(&self, _device_id: AudioObjectID, _device_type: &DeviceType) -> Result<bool> {
        // Simplified implementation - assume first device is default
        // In a full implementation, you'd check against the actual default device
        Ok(false)
    }
    
    fn get_device_format(&self, _device_id: AudioObjectID) -> Result<(u32, u16, String)> {
        // Simplified implementation with default values
        // In a full implementation, you'd query the actual device format
        Ok((48000, 2, "PCM 16bit".to_string()))
    }
    
    pub fn auto_select_best_device(&self) -> Result<Option<AudioLoopbackDevice>> {
        let devices = self.enumerate_loopback_devices()?;
        
        if devices.is_empty() {
            return Ok(None);
        }
        
        // Return the first available device
        Ok(Some(devices[0].clone()))
    }
    
    pub fn find_device_by_id(&self, device_id: &str) -> Result<Option<AudioLoopbackDevice>> {
        let devices = self.enumerate_loopback_devices()?;
        Ok(devices.into_iter().find(|d| d.id == device_id))
    }
    
    pub fn test_render_loopback_capability(&self, _device_id: AudioObjectID) -> bool {
        // Simplified implementation - assume all devices support loopback
        // In a full implementation, you'd test the actual device capabilities
        true
    }
    
    pub fn test_capture_device_capability(&self, _device_id: AudioObjectID) -> bool {
        // Simplified implementation - assume all devices support capture
        // In a full implementation, you'd test the actual device capabilities
        true
    }
}

// Tauri Commands
#[tauri::command]
pub async fn enumerate_loopback_devices() -> Result<Vec<AudioLoopbackDevice>, String> {
    match CoreAudioLoopbackEnumerator::new() {
        Ok(enumerator) => {
            match enumerator.enumerate_loopback_devices() {
                Ok(devices) => Ok(devices),
                Err(e) => Err(format!("Failed to enumerate audio devices: {}", e))
            }
        },
        Err(e) => Err(format!("Failed to initialize audio enumerator: {}", e))
    }
}

#[tauri::command]
pub async fn auto_select_best_device() -> Result<Option<AudioLoopbackDevice>, String> {
    match CoreAudioLoopbackEnumerator::new() {
        Ok(enumerator) => {
            match enumerator.auto_select_best_device() {
                Ok(device) => Ok(device),
                Err(e) => Err(format!("Failed to auto-select device: {}", e))
            }
        },
        Err(e) => Err(format!("Failed to initialize audio enumerator: {}", e))
    }
}

#[tauri::command]
pub async fn test_audio_device(device_id: String) -> Result<bool, String> {
    match CoreAudioLoopbackEnumerator::new() {
        Ok(enumerator) => {
            match enumerator.find_device_by_id(&device_id) {
                Ok(Some(device_info)) => {
                    // Simplified test - assume device works if we can find it
                    Ok(true)
                },
                Ok(None) => Ok(false),
                Err(e) => Err(format!("Failed to find device: {}", e))
            }
        },
        Err(e) => Err(format!("Failed to test audio device: {}", e))
    }
}
