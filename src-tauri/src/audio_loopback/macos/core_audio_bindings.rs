use anyhow::{Context, Result};
use objc2_core_audio::*;
use objc2_core_audio_types::{
    kAudioFormatLinearPCM, AudioBuffer, AudioBufferList, AudioStreamBasicDescription,
    AudioTimeStamp,
};
use objc2_core_foundation::{
    CFDictionary, CFMutableArray, CFNumber, CFNumberType, CFRetained, CFString,
};
use std::ptr::NonNull;

const AUDIO_FORMAT_LINEAR_PCM: u32 = kAudioFormatLinearPCM;
const AUDIO_OBJECT_PROPERTY_SCOPE_INPUT: AudioObjectPropertyScope = kAudioObjectPropertyScopeInput;
const AUDIO_OBJECT_PROPERTY_SCOPE_OUTPUT: AudioObjectPropertyScope =
    kAudioObjectPropertyScopeOutput;

/// Stream direction enum matching the Objective-C++ implementation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StreamDirection {
    Output = 0, // matches C++ StreamDirection::output
    Input = 1,  // matches C++ StreamDirection::input
}

impl StreamDirection {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(StreamDirection::Output),
            1 => Some(StreamDirection::Input),
            _ => None,
        }
    }
}

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

pub fn get_audio_device_ids() -> Result<Vec<AudioObjectID>> {
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
        return Err(anyhow::anyhow!(
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
        return Err(anyhow::anyhow!(
            "Failed to get property data: {}",
            property_result
        ));
    }

    Ok(device_ids)
}

pub fn get_device_transport_type(device_id: AudioObjectID) -> Result<u32> {
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
        return Err(anyhow::anyhow!(
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
        return Err(anyhow::anyhow!(
            "Failed to get property data: {}",
            property_result
        ));
    }
    Ok(transport_type)
}

pub fn get_device_name(device_id: AudioObjectID) -> Result<String> {
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
        return Err(anyhow::anyhow!(
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
        return Err(anyhow::anyhow!(
            "Failed to get property data: {}",
            property_result
        ));
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
    pub fn new(id: AudioObjectID) -> Result<Self> {
        // Get the name of the aggregate device
        let name = get_device_name(id)
            .with_context(|| format!("Failed to get name for aggregate device {}", id))?;

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
    // fn update_device_list(&mut self) -> Result<()> {
    //     // Implementation for getting device list
    // }

    // fn update_tap_list(&mut self) -> Result<()> {
    //     // Implementation for getting tap list
    // }

    // fn register_listeners(&self) -> Result<()> {
    //     // Implementation for registering listeners
    // }

    pub fn add_sub_device(&self, uid: AudioObjectID) -> Result<()> {
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
            return Err(anyhow::anyhow!(
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
            return Err(anyhow::anyhow!(
                "Failed to get property data: {}",
                property_result
            ));
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
            return Err(anyhow::anyhow!(
                "Failed to set property data: {}",
                property_result
            ));
        }
        Ok(())
    }
}

pub fn destroy_aggregate_device(device_id: AudioObjectID) -> Result<()> {
    let name = get_device_name(device_id)
        .with_context(|| format!("Failed to get name for device {}", device_id))?;
    println!("Destroying aggregate device {}: {}", device_id, name);
    unsafe {
        AudioHardwareDestroyAggregateDevice(device_id);
    }
    println!("Destroyed aggregate device {}: {}", device_id, name);
    Ok(())
}

pub fn create_aggregate_device(device_name: String, device_uid: String) -> Result<AggregateDevice> {
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
        .with_context(|| format!("Failed to create aggregate device '{}'", device_name))
}

pub fn get_default_input_device() -> Result<AudioObjectID> {
    let device_ids = get_audio_device_ids()?;

    for device_id in device_ids {
        let name = get_device_name(device_id)
            .with_context(|| format!("Failed to get name for device {}", device_id))?;
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
            return Err(anyhow::anyhow!(
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
            return Err(anyhow::anyhow!(
                "Failed to get property data: {}",
                property_result
            ));
        }

        println!("Default input device flag: {}", default_input_device_flag);
        if default_input_device_flag != 0 {
            return Ok(device_id);
        }
    }

    Err(anyhow::anyhow!("No default input device found"))
}

pub enum AudioDeviceType {
    Input,
    Output,
}

pub fn is_default_device(device_id: AudioObjectID, device_type: AudioDeviceType) -> Result<bool> {
    // Check if this is the default output device
    let address = AudioObjectPropertyAddress {
        mSelector: match device_type {
            AudioDeviceType::Input => kAudioHardwarePropertyDefaultInputDevice,
            AudioDeviceType::Output => kAudioHardwarePropertyDefaultOutputDevice,
        },
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain,
    };

    let mut default_device = 0u32;
    let mut size = std::mem::size_of::<AudioObjectID>() as u32;

    let result = unsafe {
        AudioObjectGetPropertyData(
            kAudioObjectSystemObject as AudioObjectID,
            NonNull::from(&address),
            0,
            std::ptr::null(),
            NonNull::from(&mut size),
            NonNull::new(&mut default_device as *mut _ as *mut std::ffi::c_void).unwrap(),
        )
    };

    if result != 0 {
        return Err(anyhow::anyhow!(
            "Failed to get default device for device {}: {}",
            device_id,
            result
        ));
    }

    Ok(device_id == default_device)
}

pub fn get_device_format(device_id: AudioObjectID) -> Result<(u32, u16, String)> {
    // Try output scope first (for render devices)
    let output_address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyStreamFormat,
        mScope: kAudioObjectPropertyScopeOutput,
        mElement: kAudioObjectPropertyElementMain,
    };

    let mut format = unsafe { std::mem::zeroed::<AudioStreamBasicDescription>() };
    let mut size = std::mem::size_of::<AudioStreamBasicDescription>() as u32;

    let output_result = unsafe {
        AudioObjectGetPropertyData(
            device_id as AudioObjectID,
            NonNull::from(&output_address),
            0,
            std::ptr::null(),
            NonNull::from(&mut size),
            NonNull::new(&mut format as *mut _ as *mut std::ffi::c_void).unwrap(),
        )
    };

    // If output scope works, use it
    if output_result == 0 {
        let sample_rate = format.mSampleRate as u32;
        let channels = format.mChannelsPerFrame as u16;

        let format_str = match format.mFormatID {
            AUDIO_FORMAT_LINEAR_PCM => {
                if format.mBitsPerChannel == 32 {
                    "IEEE Float 32bit".to_string()
                } else if format.mBitsPerChannel == 16 {
                    "PCM 16bit".to_string()
                } else {
                    format!("PCM {}bit", format.mBitsPerChannel)
                }
            }
            _ => "Unknown Format".to_string(),
        };

        return Ok((sample_rate, channels, format_str));
    }

    // Try input scope (for capture devices)
    let input_address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyStreamFormat,
        mScope: kAudioObjectPropertyScopeInput,
        mElement: kAudioObjectPropertyElementMain,
    };

    let input_result = unsafe {
        AudioObjectGetPropertyData(
            device_id as AudioObjectID,
            NonNull::from(&input_address),
            0,
            std::ptr::null(),
            NonNull::from(&mut size),
            NonNull::new(&mut format as *mut _ as *mut std::ffi::c_void).unwrap(),
        )
    };

    if input_result != 0 {
        return Err(anyhow::anyhow!(
            "Failed to get device format for device {} (both output and input scopes failed): output={}, input={}",
            device_id,
            output_result,
            input_result
        ));
    }

    let sample_rate = format.mSampleRate as u32;
    let channels = format.mChannelsPerFrame as u16;

    let format_str = match format.mFormatID {
        AUDIO_FORMAT_LINEAR_PCM => {
            if format.mBitsPerChannel == 32 {
                "IEEE Float 32bit".to_string()
            } else if format.mBitsPerChannel == 16 {
                "PCM 16bit".to_string()
            } else {
                format!("PCM {}bit", format.mBitsPerChannel)
            }
        }
        _ => "Unknown Format".to_string(),
    };

    Ok((sample_rate, channels, format_str))
}

pub fn device_has_output_streams(device_id: AudioObjectID) -> Result<bool> {
    let address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyStreams,
        mScope: kAudioObjectPropertyScopeOutput,
        mElement: kAudioObjectPropertyElementMain,
    };

    let mut size = 0u32;
    let size_result = unsafe {
        AudioObjectGetPropertyDataSize(
            device_id as AudioObjectID,
            NonNull::from(&address),
            0,
            std::ptr::null(),
            NonNull::from(&mut size),
        )
    };
    if size_result != 0 {
        return Err(anyhow::anyhow!(
            "Failed to get property data size: {}",
            size_result
        ));
    }
    Ok(size > 0)
}

pub fn device_has_input_streams(device_id: AudioObjectID) -> Result<bool> {
    let address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyStreams,
        mScope: kAudioObjectPropertyScopeInput,
        mElement: kAudioObjectPropertyElementMain,
    };

    let mut size = 0u32;
    let size_result = unsafe {
        AudioObjectGetPropertyDataSize(
            device_id as AudioObjectID,
            NonNull::from(&address),
            0,
            std::ptr::null(),
            NonNull::from(&mut size),
        )
    };
    if size_result != 0 {
        return Err(anyhow::anyhow!(
            "Failed to get property data size: {}",
            size_result
        ));
    }
    Ok(size > 0)
}

/// Stream information for cataloging
#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub format: AudioStreamBasicDescription,
}

/// Device stream catalog with input and output streams
#[derive(Debug, Clone)]
pub struct DeviceStreamCatalog {
    pub input_streams: Vec<StreamInfo>,
    pub output_streams: Vec<StreamInfo>,
    pub sample_rate: f32,
}

impl DeviceStreamCatalog {
    pub fn new() -> Self {
        Self {
            input_streams: Vec::new(),
            output_streams: Vec::new(),
            sample_rate: 48000.0,
        }
    }

    pub fn get_input_stream_count(&self) -> usize {
        self.input_streams.len()
    }

    pub fn get_output_stream_count(&self) -> usize {
        self.output_streams.len()
    }

    pub fn get_sample_rate(&self) -> f32 {
        self.sample_rate
    }
}

/// Catalog all streams for a device
pub fn catalog_device_streams(device_id: AudioObjectID) -> Result<DeviceStreamCatalog> {
    println!("[CORE_AUDIO] Cataloging streams for device: {}", device_id);

    let mut catalog = DeviceStreamCatalog::new();

    if device_id == kAudioObjectUnknown {
        println!("[CORE_AUDIO] No device set, skipping stream cataloging");
        return Ok(catalog);
    }

    // Get input streams
    catalog_input_streams(device_id, &mut catalog)?;

    // Get output streams
    catalog_output_streams(device_id, &mut catalog)?;

    // Update sample rate from first input stream
    if let Some(first_input) = catalog.input_streams.first() {
        catalog.sample_rate = first_input.format.mSampleRate as f32;
        println!(
            "[CORE_AUDIO] Detected sample rate: {} Hz",
            catalog.sample_rate
        );
    }

    println!("[CORE_AUDIO] Stream cataloging complete:");
    println!(
        "[CORE_AUDIO]   - Input streams: {}",
        catalog.input_streams.len()
    );
    println!(
        "[CORE_AUDIO]   - Output streams: {}",
        catalog.output_streams.len()
    );

    Ok(catalog)
}

/// Catalog input streams for a device
fn catalog_input_streams(
    device_id: AudioObjectID,
    catalog: &mut DeviceStreamCatalog,
) -> Result<()> {
    catalog_streams_for_scope(device_id, kAudioObjectPropertyScopeInput, catalog)
}

/// Catalog output streams for a device
fn catalog_output_streams(
    device_id: AudioObjectID,
    catalog: &mut DeviceStreamCatalog,
) -> Result<()> {
    catalog_streams_for_scope(device_id, kAudioObjectPropertyScopeOutput, catalog)
}

/// Catalog streams for a specific scope (input or output)
fn catalog_streams_for_scope(
    device_id: AudioObjectID,
    scope: AudioObjectPropertyScope,
    catalog: &mut DeviceStreamCatalog,
) -> Result<()> {
    let scope_name = match scope {
        kAudioObjectPropertyScopeInput => "input",
        kAudioObjectPropertyScopeOutput => "output",
        _ => "unknown",
    };

    println!("[CORE_AUDIO] Cataloging {} streams...", scope_name);

    // Get stream list
    let stream_ids = get_device_stream_ids(device_id, scope)?;

    for (index, &stream_id) in stream_ids.iter().enumerate() {
        println!(
            "[CORE_AUDIO] Processing {} stream {} (ID: {})",
            scope_name, index, stream_id
        );

        // Get stream format
        if let Ok(format) = get_stream_format(stream_id) {
            // Get stream direction (like the C++ implementation)
            if let Ok(direction) = get_stream_direction(stream_id) {
                let stream_info = StreamInfo { format };

                match direction {
                    StreamDirection::Input => {
                        catalog.input_streams.push(stream_info);
                        println!(
                            "[CORE_AUDIO] Added input stream {}: {}Hz, {}ch",
                            index, format.mSampleRate, format.mChannelsPerFrame
                        );
                    }
                    StreamDirection::Output => {
                        catalog.output_streams.push(stream_info);
                        println!(
                            "[CORE_AUDIO] Added output stream {}: {}Hz, {}ch",
                            index, format.mSampleRate, format.mChannelsPerFrame
                        );
                    }
                }
            } else {
                println!(
                    "[CORE_AUDIO] Failed to get stream direction for stream {}",
                    stream_id
                );
            }
        } else {
            println!("[CORE_AUDIO] Failed to get format for stream {}", stream_id);
        }
    }

    Ok(())
}

/// Get stream IDs for a device and scope
fn get_device_stream_ids(
    device_id: AudioObjectID,
    scope: AudioObjectPropertyScope,
) -> Result<Vec<AudioObjectID>> {
    let property_address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyStreams,
        mScope: scope,
        mElement: kAudioObjectPropertyElementMain,
    };

    let mut size = 0u32;
    let size_result = unsafe {
        AudioObjectGetPropertyDataSize(
            device_id,
            std::ptr::NonNull::from(&property_address),
            0,
            std::ptr::null(),
            std::ptr::NonNull::from(&mut size),
        )
    };

    if size_result != 0 {
        return Err(anyhow::anyhow!(
            "Failed to get stream property data size: {}",
            size_result
        ));
    }

    if size == 0 {
        return Ok(Vec::new());
    }

    let stream_count = size as usize / std::mem::size_of::<AudioObjectID>();
    let mut stream_ids: Vec<AudioObjectID> = vec![0; stream_count];

    let data_result = unsafe {
        AudioObjectGetPropertyData(
            device_id,
            std::ptr::NonNull::from(&property_address),
            0,
            std::ptr::null(),
            std::ptr::NonNull::from(&mut size),
            std::ptr::NonNull::new(stream_ids.as_mut_ptr() as *mut std::ffi::c_void).unwrap(),
        )
    };

    if data_result != 0 {
        return Err(anyhow::anyhow!(
            "Failed to get stream property data: {}",
            data_result
        ));
    }

    Ok(stream_ids)
}

/// Get format for a specific stream
fn get_stream_format(stream_id: AudioObjectID) -> Result<AudioStreamBasicDescription> {
    let property_address = AudioObjectPropertyAddress {
        mSelector: kAudioStreamPropertyVirtualFormat,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain,
    };

    let mut format = unsafe { std::mem::zeroed::<AudioStreamBasicDescription>() };
    let mut size = std::mem::size_of::<AudioStreamBasicDescription>() as u32;

    let result = unsafe {
        AudioObjectGetPropertyData(
            stream_id,
            std::ptr::NonNull::from(&property_address),
            0,
            std::ptr::null(),
            std::ptr::NonNull::from(&mut size),
            std::ptr::NonNull::new(&mut format as *mut _ as *mut std::ffi::c_void).unwrap(),
        )
    };

    if result != 0 {
        return Err(anyhow::anyhow!(
            "Failed to get stream format for stream {}: {}",
            stream_id,
            result
        ));
    }

    Ok(format)
}

/// Get direction for a specific stream (using UInt32 like Objective-C++ implementation)
fn get_stream_direction(stream_id: AudioObjectID) -> Result<StreamDirection> {
    let property_address = AudioObjectPropertyAddress {
        mSelector: kAudioStreamPropertyDirection,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain,
    };

    let mut direction: u32 = 0;
    let mut size = std::mem::size_of::<u32>() as u32;

    let result = unsafe {
        AudioObjectGetPropertyData(
            stream_id,
            std::ptr::NonNull::from(&property_address),
            0,
            std::ptr::null(),
            std::ptr::NonNull::from(&mut size),
            std::ptr::NonNull::new(&mut direction as *mut _ as *mut std::ffi::c_void).unwrap(),
        )
    };

    if result != 0 {
        return Err(anyhow::anyhow!(
            "Failed to get stream direction for stream {}: {} (0x{:x})",
            stream_id,
            result,
            result
        ));
    }

    println!(
        "[CORE_AUDIO] Raw stream direction value: {} (0x{:x}) for stream {}",
        direction, direction, stream_id
    );

    StreamDirection::from_u32(direction).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown stream direction value: {} (0x{:x}) for stream {}",
            direction,
            direction,
            stream_id
        )
    })
}

/// Get device name with error handling
pub fn get_device_name_safe(device_id: AudioObjectID) -> Result<String> {
    if device_id == kAudioObjectUnknown {
        return Ok("Unknown Device".to_string());
    }

    get_device_name(device_id)
}

/// Audio buffer information for processing
#[derive(Debug, Clone)]
pub struct AudioBufferInfo {
    pub number_input_buffers: u32,
    pub number_output_buffers: u32,
    pub frames_per_buffer: u32,
    pub channels_per_frame: u32,
    pub sample_rate: f32,
}

impl AudioBufferInfo {
    pub fn from_buffer_lists(
        in_input_data: *const AudioBufferList,
        out_output_data: *const AudioBufferList,
        sample_rate: f32,
    ) -> Self {
        let mut number_input_buffers = 0u32;
        let mut number_output_buffers = 0u32;
        let mut frames_per_buffer = 0u32;
        let mut channels_per_frame = 0u32;

        // Process input buffers
        if !in_input_data.is_null() {
            unsafe {
                let input_data = &*in_input_data;
                number_input_buffers = input_data.mNumberBuffers;

                if number_input_buffers > 0 {
                    let buffer = &input_data.mBuffers[0];
                    channels_per_frame = buffer.mNumberChannels;
                    frames_per_buffer = buffer.mDataByteSize
                        / (channels_per_frame * std::mem::size_of::<f32>() as u32);
                }
            }
        }

        // Process output buffers
        if !out_output_data.is_null() {
            unsafe {
                let output_data = &*out_output_data;
                number_output_buffers = output_data.mNumberBuffers;
            }
        }

        Self {
            number_input_buffers,
            number_output_buffers,
            frames_per_buffer,
            channels_per_frame,
            sample_rate,
        }
    }
}

/// Convert audio buffer to mono float samples
pub fn convert_audio_buffer_to_mono(buffer: &AudioBuffer, frames: u32, channels: u32) -> Vec<f32> {
    let mut mono_samples = Vec::with_capacity(frames as usize);

    unsafe {
        let float_data = buffer.mData as *const f32;

        for frame in 0..frames {
            let mut sample = 0.0f32;

            match channels {
                1 => {
                    // Mono: use the single channel
                    sample = *float_data.offset(frame as isize);
                }
                2 => {
                    // Stereo: average left and right channels
                    let left = *float_data.offset((frame * 2) as isize);
                    let right = *float_data.offset((frame * 2 + 1) as isize);
                    sample = (left + right) * 0.5f32;
                }
                _ => {
                    // Multi-channel: average all channels
                    for ch in 0..channels {
                        sample += *float_data.offset((frame * channels + ch) as isize);
                    }
                    sample /= channels as f32;
                }
            }

            mono_samples.push(sample);
        }
    }

    mono_samples
}

/// IO Proc callback function type (matching the crate's expected signature)
pub type AudioDeviceIOProc = unsafe extern "C-unwind" fn(
    inDevice: AudioObjectID,
    inNow: std::ptr::NonNull<AudioTimeStamp>,
    inInputData: std::ptr::NonNull<AudioBufferList>,
    inInputTime: std::ptr::NonNull<AudioTimeStamp>,
    outOutputData: std::ptr::NonNull<AudioBufferList>,
    inOutputTime: std::ptr::NonNull<AudioTimeStamp>,
    inClientData: *mut std::ffi::c_void,
) -> i32;

/// Create an IO Proc ID for a device (matching Objective-C++ AudioDeviceCreateIOProcID call)
pub fn create_io_proc_id(
    device_id: AudioObjectID,
    io_proc: AudioDeviceIOProc,
    client_data: *mut std::ffi::c_void,
) -> Result<AudioDeviceIOProcID> {
    let mut io_proc_id: AudioDeviceIOProcID = None;

    let result = unsafe {
        AudioDeviceCreateIOProcID(
            device_id,
            Some(io_proc),
            client_data,
            std::ptr::NonNull::new(&mut io_proc_id).unwrap(),
        )
    };

    if result != 0 {
        return Err(anyhow::anyhow!(
            "Failed to create IO Proc ID for device {}: {} (0x{:x})",
            device_id,
            result,
            result
        ));
    }

    Ok(io_proc_id)
}

/// Start audio device IO (matching Objective-C++ AudioDeviceStart call)
pub fn start_audio_device(device_id: AudioObjectID, io_proc_id: AudioDeviceIOProcID) -> Result<()> {
    let result = unsafe { AudioDeviceStart(device_id, io_proc_id) };

    if result != 0 {
        return Err(anyhow::anyhow!(
            "Failed to start audio device {}: {} (0x{:x})",
            device_id,
            result,
            result
        ));
    }

    Ok(())
}

/// Stop audio device IO (matching Objective-C++ AudioDeviceStop call)
pub fn stop_audio_device(device_id: AudioObjectID, io_proc_id: AudioDeviceIOProcID) -> Result<()> {
    let result = unsafe { AudioDeviceStop(device_id, io_proc_id) };

    if result != 0 {
        return Err(anyhow::anyhow!(
            "Failed to stop audio device {}: {} (0x{:x})",
            device_id,
            result,
            result
        ));
    }

    Ok(())
}

/// Destroy an IO Proc ID (matching Objective-C++ AudioDeviceDestroyIOProcID call)
pub fn destroy_io_proc_id(device_id: AudioObjectID, io_proc_id: AudioDeviceIOProcID) -> Result<()> {
    let result = unsafe { AudioDeviceDestroyIOProcID(device_id, io_proc_id) };

    if result != 0 {
        return Err(anyhow::anyhow!(
            "Failed to destroy IO Proc ID for device {}: {} (0x{:x})",
            device_id,
            result,
            result
        ));
    }

    Ok(())
}
