use objc2_core_audio::*;
use objc2_core_foundation::CFString;
use std::error::Error;

fn get_property_address(
    selector: AudioObjectPropertySelector,
    scope: AudioObjectPropertyScope,
    element: AudioObjectPropertyElement,
) -> AudioObjectPropertyAddress {
    AudioObjectPropertyAddress {
        mSelector: selector,
        mScope: scope,
        mElement: element,
    }
}

fn get_audio_device_ids() -> Result<Vec<AudioObjectID>, String> {
    let mut property_size = 0u32;
    let device_list_address = get_property_address(
        kAudioHardwarePropertyDevices,
        kAudioObjectPropertyScopeGlobal,
        kAudioObjectPropertyElementMain,
    );

    // Get size
    let property_size_result = unsafe {
        AudioObjectGetPropertyDataSize(
            kAudioObjectSystemObject as AudioObjectID,
            std::ptr::NonNull::from(&device_list_address),
            0,
            std::ptr::null(),
            std::ptr::NonNull::from(&mut property_size),
        )
    };

    if property_size_result != 0 {
        return Err(format!(
            "Failed to get property data size: {}",
            property_size_result
        ));
    }

    let device_count = property_size as usize / std::mem::size_of::<AudioObjectID>();
    println!("Device count: {}", device_count);
    let mut device_ids: Vec<AudioObjectID> = vec![0; device_count];

    // Get data
    let property_result = unsafe {
        AudioObjectGetPropertyData(
            kAudioObjectSystemObject as AudioObjectID,
            std::ptr::NonNull::from(&device_list_address),
            0,
            std::ptr::null(),
            std::ptr::NonNull::from(&mut property_size),
            std::ptr::NonNull::new(device_ids.as_mut_ptr() as *mut std::ffi::c_void).unwrap(),
        )
    };
    println!("Found {} devices: {:?}", device_ids.len(), device_ids);

    if property_result != 0 {
        return Err(format!("Failed to get property data: {}", property_result));
    }

    Ok(device_ids)
}

fn get_device_transport_type(device_id: AudioObjectID) -> Result<u32, String> {
    let mut transport_type_size = 0u32;
    let property_address = get_property_address(
        kAudioDevicePropertyTransportType,
        kAudioObjectPropertyScopeGlobal,
        kAudioObjectPropertyElementMain,
    );
    let property_size_result = unsafe {
        AudioObjectGetPropertyDataSize(
            device_id,
            std::ptr::NonNull::from(&property_address),
            0,
            std::ptr::null(),
            std::ptr::NonNull::from(&mut transport_type_size),
        )
    };
    if property_size_result != 0 {
        return Err(format!(
            "Failed to get property data size: {}",
            property_size_result
        ));
    }
    let mut transport_type: u32 = 0;
    let property_result = unsafe {
        AudioObjectGetPropertyData(
            device_id,
            std::ptr::NonNull::from(&property_address),
            0,
            std::ptr::null(),
            std::ptr::NonNull::from(&mut transport_type_size),
            std::ptr::NonNull::new((&mut transport_type as *mut u32) as *mut std::ffi::c_void)
                .unwrap(),
        )
    };
    if property_result != 0 {
        return Err(format!("Failed to get property data: {}", property_result));
    }
    Ok(transport_type)
}

fn get_device_name(device_id: AudioObjectID) -> Result<String, String> {
    let mut name_size = 0u32;
    let property_address = get_property_address(
        kAudioDevicePropertyDeviceNameCFString,
        kAudioObjectPropertyScopeGlobal,
        kAudioObjectPropertyElementMain,
    );

    // Get the size of the property data
    let property_size_result = unsafe {
        AudioObjectGetPropertyDataSize(
            device_id,
            std::ptr::NonNull::from(&property_address),
            0,
            std::ptr::null(),
            std::ptr::NonNull::from(&mut name_size),
        )
    };
    if property_size_result != 0 {
        return Err(format!(
            "Failed to get property data size: {}",
            property_size_result
        ));
    }

    // Create a buffer to hold the CFString pointer
    let mut cf_string_ptr: *const CFString = std::ptr::null();
    let property_result = unsafe {
        AudioObjectGetPropertyData(
            device_id,
            std::ptr::NonNull::from(&property_address),
            0,
            std::ptr::null(),
            std::ptr::NonNull::from(&mut name_size),
            std::ptr::NonNull::new(
                &mut cf_string_ptr as *mut *const CFString as *mut std::ffi::c_void,
            )
            .unwrap(),
        )
    };

    if property_result != 0 {
        return Err(format!("Failed to get property data: {}", property_result));
    }

    let name = if !cf_string_ptr.is_null() {
        unsafe { (*cf_string_ptr).to_string() }
    } else {
        String::from("Unknown Device")
    };

    println!("Device {} name: {}", device_id, name);
    Ok(name)
}

fn destroy_aggregate_device(device_id: AudioObjectID) -> Result<(), String> {
    let name = get_device_name(device_id)?;
    println!("Destroying aggregate device {}: {}", device_id, name);
    unsafe {
        AudioHardwareDestroyAggregateDevice(device_id);
    }
    println!("Destroyed aggregate device {}: {}", device_id, name);
    Ok(())
}

fn clean_existing_aggregate_devices_unsafe() -> Result<(), String> {
    let device_ids = get_audio_device_ids()?;

    for device_id in device_ids {
        let transport_type = get_device_transport_type(device_id)?;
        if transport_type == kAudioDeviceTransportTypeAggregate {
            destroy_aggregate_device(device_id)?;
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("[cleanup_script] Starting aggregate device cleanup...");

    // Call the cleanup function
    match clean_existing_aggregate_devices_unsafe() {
        Ok(()) => {
            println!("[cleanup_script] Successfully cleaned up aggregate devices");
            Ok(())
        }
        Err(e) => {
            eprintln!(
                "[cleanup_script] Error cleaning up aggregate devices: {}",
                e
            );
            Err(e.into())
        }
    }
}
