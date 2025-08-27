//! Aggregate device functionality for combining multiple audio sources
//! This module will implement the aggregate device functionality from AudioTapSample

use crate::types::{AudioCaptureResult, AggregateDeviceConfig};
use crate::macos::core_audio_bindings::*;
use std::collections::HashSet;

/// Aggregate device for combining multiple audio sources
pub struct AggregateDevice {
    device_id: AudioAggregateDeviceID,
    config: AggregateDeviceConfig,
    device_list: HashSet<String>,
    tap_list: HashSet<String>,
    device_list_address: AudioObjectPropertyAddress,
    tap_list_address: AudioObjectPropertyAddress,
    composition_address: AudioObjectPropertyAddress,
}

impl AggregateDevice {
    /// Create a new aggregate device
    pub fn new(device_id: AudioAggregateDeviceID) -> AudioCaptureResult<Self> {
        let device_list_address = create_property_address(
            AUDIO_AGGREGATE_DEVICE_PROPERTY_FULL_SUB_DEVICE_LIST,
            AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL,
            AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN,
        );
        
        let tap_list_address = create_property_address(
            AUDIO_AGGREGATE_DEVICE_PROPERTY_TAP_LIST,
            AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL,
            AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN,
        );
        
        let composition_address = create_property_address(
            AUDIO_AGGREGATE_DEVICE_PROPERTY_COMPOSITION,
            AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL,
            AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN,
        );
        
        let mut device = Self {
            device_id,
            config: AggregateDeviceConfig {
                name: String::new(),
                sub_devices: Vec::new(),
                taps: Vec::new(),
                is_private: false,
                auto_start: false,
                auto_stop: false,
            },
            device_list: HashSet::new(),
            tap_list: HashSet::new(),
            device_list_address,
            tap_list_address,
            composition_address,
        };
        
        // Initialize the device
        device.initialize()?;
        
        Ok(device)
    }
    
    /// Initialize the aggregate device
    fn initialize(&mut self) -> AudioCaptureResult<()> {
        // Get the device name
        self.config.name = self.get_device_name_internal()?;
        
        // Update device and tap lists
        self.update_device_list()?;
        self.update_tap_list()?;
        self.update_config()?;
        
        Ok(())
    }
    
    /// Get the device configuration
    pub fn get_config(&self) -> &AggregateDeviceConfig {
        &self.config
    }
    
    /// Set the device configuration
    pub fn set_config(&mut self, config: AggregateDeviceConfig) -> AudioCaptureResult<()> {
        // TODO: Implement aggregate device configuration update
        // This will include:
        // 1. Updating the device composition
        // 2. Setting sub-device and tap lists
        // 3. Handling device property updates
        
        self.config = config;
        Ok(())
    }
    
    /// Add a sub-device to the aggregate device
    pub fn add_sub_device(&mut self, device_uid: &str) -> AudioCaptureResult<()> {
        self.add_remove(device_uid, ModifyAction::Add, ListType::Device)
    }
    
    /// Remove a sub-device from the aggregate device
    pub fn remove_sub_device(&mut self, device_uid: &str) -> AudioCaptureResult<()> {
        self.add_remove(device_uid, ModifyAction::Remove, ListType::Device)
    }
    
    /// Add a tap to the aggregate device
    pub fn add_tap(&mut self, tap_uid: &str) -> AudioCaptureResult<()> {
        self.add_remove(tap_uid, ModifyAction::Add, ListType::Tap)
    }
    
    /// Remove a tap from the aggregate device
    pub fn remove_tap(&mut self, tap_uid: &str) -> AudioCaptureResult<()> {
        self.add_remove(tap_uid, ModifyAction::Remove, ListType::Tap)
    }
    
    /// Get the device name
    pub fn get_name(&self) -> AudioCaptureResult<String> {
        Ok(self.config.name.clone())
    }
    
    /// Set the device name
    pub fn set_name(&mut self, name: &str) -> AudioCaptureResult<()> {
        // TODO: Implement name setting using kAudioObjectPropertyName
        self.config.name = name.to_string();
        Ok(())
    }
    
    /// Get device name from Core Audio
    fn get_device_name_internal(&self) -> AudioCaptureResult<String> {
        let property_address = create_property_address(
            AUDIO_OBJECT_PROPERTY_NAME,
            AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL,
            AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN,
        );
        
        let mut property_size = std::mem::size_of::<CFStringRef>() as u32;
        let mut name_ref: CFStringRef = std::ptr::null_mut();
        
        let status = unsafe {
            AudioObjectGetPropertyData(
                self.device_id,
                &property_address,
                0,
                std::ptr::null(),
                &mut property_size,
                &mut name_ref as *mut _ as *mut std::ffi::c_void,
            )
        };
        
        if status != NO_ERR {
            return Err(crate::types::AudioCaptureError::CoreAudioError(
                format!("Failed to get device name: {}", status)
            ));
        }
        
        cf_string_to_string(name_ref)
            .map_err(|e| crate::types::AudioCaptureError::CoreAudioError(
                format!("Failed to convert device name: {}", e)
            ))
    }
    
    /// Update the device list from Core Audio
    fn update_device_list(&mut self) -> AudioCaptureResult<()> {
        let mut property_size = 0u32;
        let status = unsafe {
            AudioObjectGetPropertyDataSize(
                self.device_id,
                &self.device_list_address,
                0,
                std::ptr::null(),
                &mut property_size,
            )
        };
        
        if status != NO_ERR {
            return Err(crate::types::AudioCaptureError::CoreAudioError(
                format!("Failed to get device list size: {}", status)
            ));
        }
        
        if property_size == 0 {
            self.device_list.clear();
            return Ok(());
        }
        
        let mut list: CFArrayRef = std::ptr::null_mut();
        let status = unsafe {
            AudioObjectGetPropertyData(
                self.device_id,
                &self.device_list_address,
                0,
                std::ptr::null(),
                &mut property_size,
                &mut list as *mut _ as *mut std::ffi::c_void,
            )
        };
        
        if status != NO_ERR {
            return Err(crate::types::AudioCaptureError::CoreAudioError(
                format!("Failed to get device list: {}", status)
            ));
        }
        
        // Convert CFArray to Vec<String>
        let device_strings = cf_array_to_strings(list)
            .map_err(|e| crate::types::AudioCaptureError::CoreAudioError(
                format!("Failed to convert device list: {}", e)
            ))?;
        
        self.device_list.clear();
        for device_uid in device_strings {
            self.device_list.insert(device_uid);
        }
        
        // Update config
        self.config.sub_devices = self.device_list.iter().cloned().collect();
        
        Ok(())
    }
    
    /// Update the tap list from Core Audio
    fn update_tap_list(&mut self) -> AudioCaptureResult<()> {
        let mut property_size = 0u32;
        let status = unsafe {
            AudioObjectGetPropertyDataSize(
                self.device_id,
                &self.tap_list_address,
                0,
                std::ptr::null(),
                &mut property_size,
            )
        };
        
        if status != NO_ERR {
            return Err(crate::types::AudioCaptureError::CoreAudioError(
                format!("Failed to get tap list size: {}", status)
            ));
        }
        
        if property_size == 0 {
            self.tap_list.clear();
            return Ok(());
        }
        
        let mut list: CFArrayRef = std::ptr::null_mut();
        let status = unsafe {
            AudioObjectGetPropertyData(
                self.device_id,
                &self.tap_list_address,
                0,
                std::ptr::null(),
                &mut property_size,
                &mut list as *mut _ as *mut std::ffi::c_void,
            )
        };
        
        if status != NO_ERR {
            return Err(crate::types::AudioCaptureError::CoreAudioError(
                format!("Failed to get tap list: {}", status)
            ));
        }
        
        // Convert CFArray to Vec<String>
        let tap_strings = cf_array_to_strings(list)
            .map_err(|e| crate::types::AudioCaptureError::CoreAudioError(
                format!("Failed to convert tap list: {}", e)
            ))?;
        
        self.tap_list.clear();
        for tap_uid in tap_strings {
            self.tap_list.insert(tap_uid);
        }
        
        // Update config
        self.config.taps = self.tap_list.iter().cloned().collect();
        
        Ok(())
    }
    
    /// Update the device configuration from Core Audio
    fn update_config(&mut self) -> AudioCaptureResult<()> {
        let mut property_size = 0u32;
        let status = unsafe {
            AudioObjectGetPropertyDataSize(
                self.device_id,
                &self.composition_address,
                0,
                std::ptr::null(),
                &mut property_size,
            )
        };
        
        if status != NO_ERR {
            return Err(crate::types::AudioCaptureError::CoreAudioError(
                format!("Failed to get composition size: {}", status)
            ));
        }
        
        if property_size == 0 {
            return Ok(());
        }
        
        let mut composition: CFDictionaryRef = std::ptr::null_mut();
        let status = unsafe {
            AudioObjectGetPropertyData(
                self.device_id,
                &self.composition_address,
                0,
                std::ptr::null(),
                &mut property_size,
                &mut composition as *mut _ as *mut std::ffi::c_void,
            )
        };
        
        if status != NO_ERR {
            return Err(crate::types::AudioCaptureError::CoreAudioError(
                format!("Failed to get composition: {}", status)
            ));
        }
        
        // TODO: Parse composition dictionary to update config
        // For now, we'll just keep the default values
        
        Ok(())
    }
    
    /// Add or remove a subdevice or subtap from the aggregate device
    fn add_remove(&mut self, uid: &str, action: ModifyAction, list_type: ListType) -> AudioCaptureResult<()> {
        let property_address = match list_type {
            ListType::Device => &self.device_list_address,
            ListType::Tap => &self.tap_list_address,
        };
        
        let mut property_size = 0u32;
        let status = unsafe {
            AudioObjectGetPropertyDataSize(
                self.device_id,
                &*property_address,
                0,
                std::ptr::null(),
                &mut property_size,
            )
        };
        
        if status != NO_ERR {
            return Err(crate::types::AudioCaptureError::CoreAudioError(
                format!("Failed to get list size: {}", status)
            ));
        }
        
        let mut list: CFArrayRef = std::ptr::null_mut();
        let status = unsafe {
            AudioObjectGetPropertyData(
                self.device_id,
                &*property_address,
                0,
                std::ptr::null(),
                &mut property_size,
                &mut list as *mut _ as *mut std::ffi::c_void,
            )
        };
        
        if status != NO_ERR {
            return Err(crate::types::AudioCaptureError::CoreAudioError(
                format!("Failed to get list: {}", status)
            ));
        }
        
        // Convert current list to Vec<String>
        let mut current_list = cf_array_to_strings(list)
            .map_err(|e| crate::types::AudioCaptureError::CoreAudioError(
                format!("Failed to convert list: {}", e)
            ))?;
        
        match action {
            ModifyAction::Add => {
                if !current_list.contains(&uid.to_string()) {
                    current_list.push(uid.to_string());
                }
            }
            ModifyAction::Remove => {
                current_list.retain(|x| x != uid);
            }
        }
        
        // Create new CFArray from updated list
        let new_list = create_cf_array_from_strings(&current_list)
            .map_err(|e| crate::types::AudioCaptureError::CoreAudioError(
                format!("Failed to create new list: {}", e)
            ))?;
        
        // Set the updated list back to Core Audio
        let new_property_size = std::mem::size_of::<CFArrayRef>() as u32;
        let status = unsafe {
            AudioObjectSetPropertyData(
                self.device_id,
                &*property_address,
                0,
                std::ptr::null(),
                new_property_size,
                &new_list as *const _ as *const std::ffi::c_void,
            )
        };
        
        if status != NO_ERR {
            return Err(crate::types::AudioCaptureError::CoreAudioError(
                format!("Failed to set updated list: {}", status)
            ));
        }
        
        // Update our internal state
        match list_type {
            ListType::Device => {
                self.update_device_list()?;
            }
            ListType::Tap => {
                self.update_tap_list()?;
            }
        }
        
        Ok(())
    }
    
    /// Refresh the device and tap lists from Core Audio
    pub fn refresh(&mut self) -> AudioCaptureResult<()> {
        self.update_device_list()?;
        self.update_tap_list()?;
        self.update_config()?;
        Ok(())
    }
    
    /// Get the current device list
    pub fn get_device_list(&self) -> &HashSet<String> {
        &self.device_list
    }
    
    /// Get the current tap list
    pub fn get_tap_list(&self) -> &HashSet<String> {
        &self.tap_list
    }
    
    /// Check if a device is in the device list
    pub fn has_device(&self, device_uid: &str) -> bool {
        self.device_list.contains(device_uid)
    }
    
    /// Check if a tap is in the tap list
    pub fn has_tap(&self, tap_uid: &str) -> bool {
        self.tap_list.contains(tap_uid)
    }
    
    /// Get the device ID
    pub fn get_device_id(&self) -> AudioAggregateDeviceID {
        self.device_id
    }
}

/// Action for modifying device/tap lists
#[derive(Debug, Clone, Copy)]
enum ModifyAction {
    Add,
    Remove,
}

/// Type of list to modify
#[derive(Debug, Clone, Copy)]
enum ListType {
    Device,
    Tap,
}

/// Factory functions for creating and managing aggregate devices
pub mod factory {
    use super::*;
    use crate::macos::core_audio_enumerator::CoreAudioDeviceEnumerator;
    use crate::device_enumerator::DeviceEnumerator;
    
    /// Create a new aggregate device with the given configuration
    pub async fn create_aggregate_device(
        name: &str,
        sub_devices: &[String],
        taps: &[String],
        is_private: bool,
        auto_start: bool,
    ) -> AudioCaptureResult<AggregateDevice> {
        // First, we need to find an existing aggregate device or create one
        // For now, we'll look for existing aggregate devices
        let enumerator = CoreAudioDeviceEnumerator::new()?;
        let devices = enumerator.enumerate_devices().await?;
        
        // Find an aggregate device
        let aggregate_device = devices.iter()
            .find(|d| d.device_type == crate::types::DeviceType::Aggregate)
            .ok_or_else(|| crate::types::AudioCaptureError::Other(
                "No aggregate device found".to_string()
            ))?;
        
        // Parse the device ID
        let device_id: AudioAggregateDeviceID = aggregate_device.id.parse()
            .map_err(|_| crate::types::AudioCaptureError::InvalidConfiguration(
                "Invalid aggregate device ID".to_string()
            ))?;
        
        // Create the aggregate device
        let mut device = AggregateDevice::new(device_id)?;
        
        // Configure it with the provided settings
        for sub_device in sub_devices {
            device.add_sub_device(sub_device)?;
        }
        
        for tap in taps {
            device.add_tap(tap)?;
        }
        
        // Set the name
        device.set_name(name)?;
        
        Ok(device)
    }
    
    /// Find existing aggregate devices
    pub async fn find_aggregate_devices() -> AudioCaptureResult<Vec<AggregateDevice>> {
        let enumerator = CoreAudioDeviceEnumerator::new()?;
        let devices = enumerator.enumerate_devices().await?;
        
        let mut aggregate_devices = Vec::new();
        
        for device in devices {
            if device.device_type == crate::types::DeviceType::Aggregate {
                let device_id: AudioAggregateDeviceID = device.id.parse()
                    .map_err(|_| crate::types::AudioCaptureError::InvalidConfiguration(
                        "Invalid aggregate device ID".to_string()
                    ))?;
                
                if let Ok(aggregate_device) = AggregateDevice::new(device_id) {
                    aggregate_devices.push(aggregate_device);
                }
            }
        }
        
        Ok(aggregate_devices)
    }
}
