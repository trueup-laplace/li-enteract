//! Test example for getting default audio devices

use audio_capture_lib::macos::core_audio_enumerator::CoreAudioDeviceEnumerator;
use audio_capture_lib::macos::core_audio_bindings::*;
use std::ptr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Testing Default Device Detection");
    println!("==================================");
    
    // Get default input device
    let mut default_input_id: AudioObjectID = 0;
    let mut property_size = std::mem::size_of::<AudioObjectID>() as u32;
    let property_address = AudioObjectPropertyAddress {
        selector: AUDIO_HARDWARE_PROPERTY_DEFAULT_INPUT_DEVICE,
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
            &mut default_input_id as *mut _ as *mut _,
        )
    };
    
    println!("Default input device: ID={}, status={}", default_input_id, status);
    
    // Get default output device
    let mut default_output_id: AudioObjectID = 0;
    let property_address = AudioObjectPropertyAddress {
        selector: AUDIO_HARDWARE_PROPERTY_DEFAULT_OUTPUT_DEVICE,
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
            &mut default_output_id as *mut _ as *mut _,
        )
    };
    
    println!("Default output device: ID={}, status={}", default_output_id, status);
    
    // Get all devices
    let enumerator = CoreAudioDeviceEnumerator::new()?;
    let device_ids = enumerator.get_audio_device_ids()?;
    println!("All device IDs: {:?}", device_ids);
    
    println!("\n✅ Test completed!");
    Ok(())
}
