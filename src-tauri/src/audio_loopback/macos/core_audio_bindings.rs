use objc2_core_audio::*;
use objc2_core_foundation::{
    CFDictionary, CFMutableArray, CFNumber, CFNumberType, CFRetained, CFString,
};
use std::ptr::NonNull;

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

pub fn get_audio_device_ids() -> Result<Vec<AudioObjectID>, String> {
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
            NonNull::from(&device_list_address),
            0,
            std::ptr::null(),
            NonNull::from(&mut property_size),
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
            NonNull::from(&device_list_address),
            0,
            std::ptr::null(),
            NonNull::from(&mut property_size),
            NonNull::new(device_ids.as_mut_ptr() as *mut std::ffi::c_void).unwrap(),
        )
    };
    println!("Found {} devices: {:?}", device_ids.len(), device_ids);

    if property_result != 0 {
        return Err(format!("Failed to get property data: {}", property_result));
    }

    Ok(device_ids)
}

pub fn get_device_transport_type(device_id: AudioObjectID) -> Result<u32, String> {
    let mut transport_type_size = 0u32;
    let property_address = get_property_address(
        kAudioDevicePropertyTransportType,
        kAudioObjectPropertyScopeGlobal,
        kAudioObjectPropertyElementMain,
    );
    let property_size_result = unsafe {
        AudioObjectGetPropertyDataSize(
            device_id,
            NonNull::from(&property_address),
            0,
            std::ptr::null(),
            NonNull::from(&mut transport_type_size),
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
            NonNull::from(&property_address),
            0,
            std::ptr::null(),
            NonNull::from(&mut transport_type_size),
            NonNull::new((&mut transport_type as *mut u32) as *mut std::ffi::c_void).unwrap(),
        )
    };

    println!("Device {} transport type: {}", device_id, transport_type);

    if property_result != 0 {
        return Err(format!("Failed to get property data: {}", property_result));
    }
    Ok(transport_type)
}

pub fn get_device_name(device_id: AudioObjectID) -> Result<String, String> {
    let mut name_size = 0u32;
    let property_address = get_property_address(
        kAudioObjectPropertyName,
        kAudioObjectPropertyScopeGlobal,
        kAudioObjectPropertyElementMain,
    );

    // Get the size of the property data
    let property_size_result = unsafe {
        AudioObjectGetPropertyDataSize(
            device_id,
            NonNull::from(&property_address),
            0,
            std::ptr::null(),
            NonNull::from(&mut name_size),
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
            NonNull::from(&property_address),
            0,
            std::ptr::null(),
            NonNull::from(&mut name_size),
            NonNull::new(&mut cf_string_ptr as *mut *const CFString as *mut std::ffi::c_void)
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

pub struct AggregateDevice {
    id: AudioObjectID,
    name: String,
    device_list: Vec<AudioObjectID>,
    tap_list: Vec<AudioObjectID>,
}

impl AggregateDevice {
    pub fn new(id: AudioObjectID) -> Result<Self, String> {
        // Get the name of the aggregate device
        let name = get_device_name(id)?;

        // TODO: Fill out the device and tap lists
        // self.update_device_list()
        // self.update_tap_list()

        // TODO: Register listeners
        // self.register_listeners()

        Ok(AggregateDevice {
            id,
            name,
            device_list: Vec::new(), // TODO: implement device list
            tap_list: Vec::new(),    // TODO: implement tap list
        })
    }

    // TODO: Implement these methods later
    // fn update_device_list(&mut self) -> Result<(), String> {
    //     // Implementation for getting device list
    // }

    // fn update_tap_list(&mut self) -> Result<(), String> {
    //     // Implementation for getting tap list
    // }

    // fn register_listeners(&self) -> Result<(), String> {
    //     // Implementation for registering listeners
    // }

    pub fn add_sub_device(&self, uid: AudioObjectID) -> Result<(), String> {
        let mut property_size = 0u32;
        let property_address = get_property_address(
            kAudioAggregateDevicePropertyFullSubDeviceList,
            kAudioObjectPropertyScopeGlobal,
            kAudioObjectPropertyElementMain,
        );
        let property_size_result = unsafe {
            AudioObjectGetPropertyDataSize(
                self.id,
                NonNull::from(&property_address),
                0,
                std::ptr::null(),
                NonNull::from(&mut property_size),
            )
        };
        if property_size_result != 0 {
            return Err(format!(
                "Failed to get property data size: {}",
                property_size_result
            ));
        }
        let mut list = CFMutableArray::<CFNumber>::empty();
        let property_result = unsafe {
            AudioObjectGetPropertyData(
                self.id,
                NonNull::from(&property_address),
                0,
                std::ptr::null(),
                NonNull::from(&mut property_size),
                NonNull::new(
                    list.as_ref() as *const CFMutableArray<CFNumber> as *mut std::ffi::c_void
                )
                .unwrap(),
            )
        };
        if property_result != 0 {
            return Err(format!("Failed to get property data: {}", property_result));
        }
        let uid_value = uid as i32;
        let cf_number = unsafe {
            CFNumber::new(
                None,
                CFNumberType::SInt32Type,
                &uid_value as *const i32 as *const std::ffi::c_void,
            )
            .unwrap()
        };
        list.append(cf_number.as_ref());
        property_size += std::mem::size_of::<AudioObjectID>() as u32;
        let property_result = unsafe {
            AudioObjectSetPropertyData(
                self.id,
                NonNull::from(&property_address),
                0,
                std::ptr::null(),
                property_size,
                NonNull::new(
                    list.as_ref() as *const CFMutableArray<CFNumber> as *mut std::ffi::c_void
                )
                .unwrap(),
            )
        };
        if property_result != 0 {
            return Err(format!("Failed to set property data: {}", property_result));
        }
        Ok(())
    }
}

pub fn destroy_aggregate_device(device_id: AudioObjectID) -> Result<(), String> {
    let name = get_device_name(device_id)?;
    println!("Destroying aggregate device {}: {}", device_id, name);
    unsafe {
        AudioHardwareDestroyAggregateDevice(device_id);
    }
    println!("Destroyed aggregate device {}: {}", device_id, name);
    Ok(())
}

pub fn create_aggregate_device(
    device_name: String,
    device_uid: String,
) -> Result<AggregateDevice, String> {
    let description: CFRetained<CFDictionary<CFString, CFString>> = CFDictionary::from_slices(
        &[
            CFString::from_str(kAudioAggregateDeviceNameKey.to_str().unwrap()).as_ref(),
            CFString::from_str(kAudioAggregateDeviceUIDKey.to_str().unwrap()).as_ref(),
        ],
        &[
            CFString::from_str(&device_name).as_ref(),
            CFString::from_str(&device_uid).as_ref(),
        ],
    );
    println!("Description: {:?}", description);
    let mut device_id: AudioObjectID = 0;
    unsafe {
        AudioHardwareCreateAggregateDevice(
            description.as_ref() as &CFDictionary,
            NonNull::from(&mut device_id),
        );
    }
    AggregateDevice::new(device_id)
}

pub fn get_default_input_device() -> Result<AudioObjectID, String> {
    let device_ids = get_audio_device_ids()?;

    for device_id in device_ids {
        let name = get_device_name(device_id)?;
        println!("Device {} name: {}", device_id, name);

        let property_address = get_property_address(
            kAudioHardwarePropertyDefaultInputDevice,
            kAudioObjectPropertyScopeGlobal,
            kAudioObjectPropertyElementMain,
        );
        let mut property_size = 0u32;
        let property_size_result = unsafe {
            AudioObjectGetPropertyDataSize(
                kAudioObjectSystemObject as AudioObjectID,
                NonNull::from(&property_address),
                0,
                std::ptr::null(),
                NonNull::from(&mut property_size),
            )
        };
        if property_size_result != 0 {
            return Err(format!(
                "Failed to get property data size: {}",
                property_size_result
            ));
        }
        let mut default_input_device_flag = 0;
        let property_result = unsafe {
            AudioObjectGetPropertyData(
                kAudioObjectSystemObject as AudioObjectID,
                NonNull::from(&property_address),
                0,
                std::ptr::null(),
                NonNull::from(&mut property_size),
                NonNull::new(&mut default_input_device_flag as *mut i32 as *mut std::ffi::c_void)
                    .unwrap(),
            )
        };

        if property_result != 0 {
            return Err(format!("Failed to get property data: {}", property_result));
        }

        println!("Default input device flag: {}", default_input_device_flag);
        if default_input_device_flag != 0 {
            return Ok(device_id);
        }
    }

    Err(format!("No default input device found"))
}
