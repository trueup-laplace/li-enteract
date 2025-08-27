//! Aggregate device creation using Core Foundation

use crate::types::AudioCaptureResult;
use crate::macos::core_audio_bindings::{AudioObjectID, AudioHardwareCreateAggregateDevice, NO_ERR};
use core_foundation::{
    string::CFString,
    dictionary::CFDictionary,
    base::TCFType,
};

/// Create a simple aggregate device using Core Foundation
pub fn create_simple_aggregate_device(name: &str) -> AudioCaptureResult<u32> {
    // Step 1: Create CFString for the device name
    let name_cf_string = CFString::new(name);
    
    // Step 2: Create a unique UID
    let uid = format!("{}-uid", name);
    let uid_cf_string = CFString::new(&uid);
    
    // Step 3: Create CFDictionary with name and UID
    let name_key_cf_string = CFString::new("name");
    let uid_key_cf_string = CFString::new("uid");
    
    let pairs = vec![(name_key_cf_string, name_cf_string), (uid_key_cf_string, uid_cf_string)];
    let description = CFDictionary::from_CFType_pairs(&pairs);
    
    // Step 4: Call AudioHardwareCreateAggregateDevice
    let mut device_id: AudioObjectID = 0;
    let status = unsafe {
        AudioHardwareCreateAggregateDevice(
            description.as_CFTypeRef() as *mut std::ffi::c_void,
            &mut device_id,
        )
    };
    
    if status == NO_ERR {
        Ok(device_id)
    } else {
        Err(crate::types::AudioCaptureError::CoreAudioError(
            format!("Failed to create aggregate device '{}': status {}", name, status)
        ))
    }
}

/// Create an aggregate device with default input device
pub fn create_microphone_aggregate_device(name: &str) -> AudioCaptureResult<u32> {
    // For now, create a simple aggregate device
    // In the future, we can add sub-devices here
    create_simple_aggregate_device(name)
}

/// Create an "All Audio" aggregate device (like Swift MainView)
pub fn create_all_audio_aggregate_device(name: &str) -> AudioCaptureResult<u32> {
    // For now, create a simple aggregate device
    // In the future, we can add system audio devices and taps here
    create_simple_aggregate_device(name)
}
