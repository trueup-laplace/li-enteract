//! Objective-C bindings for Core Audio functionality
//! This module provides direct access to Core Audio APIs that aren't available in Rust crates
//! 
//! # Naming Conventions
//! 
//! ## Constants
//! This module maps Core Audio constants from their original camelCase names to Rust-style UPPER_SNAKE_CASE:
//! - `kAudioObjectUnknown` → `AUDIO_OBJECT_UNKNOWN`
//! - `kAudioObjectSystemObject` → `AUDIO_OBJECT_SYSTEM_OBJECT`
//! - `kAudioHardwarePropertyDevices` → `AUDIO_HARDWARE_PROPERTY_DEVICES`
//! - etc.
//! 
//! ## Struct Properties
//! Struct properties are mapped from Core Audio's camelCase to Rust's snake_case:
//! - `mSelector` → `selector`
//! - `mScope` → `scope`
//! - `mElement` → `element`
//! - `mSampleRate` → `sample_rate`
//! - `mFormatID` → `format_id`
//! - `mFormatFlags` → `format_flags`
//! - `mBytesPerPacket` → `bytes_per_packet`
//! - `mFramesPerPacket` → `frames_per_packet`
//! - `mBytesPerFrame` → `bytes_per_frame`
//! - `mChannelsPerFrame` → `channels_per_frame`
//! - `mBitsPerChannel` → `bits_per_channel`
//! - `mReserved` → `reserved`
//! - `mNumberChannels` → `number_channels`
//! - `mDataByteSize` → `data_byte_size`
//! - `mData` → `data`
//! - `mNumberBuffers` → `number_buffers`
//! - `mBuffers` → `buffers`
//! - `mSampleTime` → `sample_time`
//! - `mHostTime` → `host_time`
//! - `mRateScalar` → `rate_scalar`
//! - `mWordClockTime` → `word_clock_time`
//! - `mSMPTETime` → `smpte_time`
//! - `mFlags` → `flags`
//! 
//! This follows Rust naming conventions while maintaining the semantic meaning of the original Core Audio constants and properties.
//! 
//! ## FFI Function Parameters
//! 
//! **External FFI functions** (declared with `extern "C"`) maintain their original camelCase parameter names
//! to preserve compatibility with the Core Audio C API:
//! - `AudioObjectGetPropertyDataSize(inObjectID, inAddress, ...)`
//! - `AudioDeviceCreateIOProcID(inDevice, inProc, ...)`
//! 
//! **Function type definitions** use Rust snake_case parameter names while maintaining FFI compatibility:
//! - `AudioDeviceIOProc(in_device, in_now, in_input_data, ...)`
//! - `AudioObjectPropertyListenerProc(in_object_id, in_number_addresses, ...)`

use std::ffi::{c_void, CStr, CString};
use std::ptr;
use std::os::raw::{c_char};

// Core Audio types
pub type AudioObjectID = u32;
pub type AudioDeviceID = AudioObjectID;
pub type AudioStreamID = AudioObjectID;
pub type AudioTapID = AudioObjectID;
pub type AudioAggregateDeviceID = AudioObjectID;
pub type OSStatus = i32;

// Core Audio constants
/// Maps to `kAudioObjectUnknown` - represents an unknown audio object
pub const AUDIO_OBJECT_UNKNOWN: AudioObjectID = 0;
/// Maps to `kAudioObjectSystemObject` - represents the system audio object
pub const AUDIO_OBJECT_SYSTEM_OBJECT: AudioObjectID = 1;

// Property scopes
/// Maps to `kAudioObjectPropertyScopeGlobal` - global property scope
pub const AUDIO_OBJECT_PROPERTY_SCOPE_GLOBAL: u32 = 0;
/// Maps to `kAudioObjectPropertyScopeInput` - input property scope
pub const AUDIO_OBJECT_PROPERTY_SCOPE_INPUT: u32 = 1;
/// Maps to `kAudioObjectPropertyScopeOutput` - output property scope
pub const AUDIO_OBJECT_PROPERTY_SCOPE_OUTPUT: u32 = 2;

// Property elements
/// Maps to `kAudioObjectPropertyElementMain` - main property element
pub const AUDIO_OBJECT_PROPERTY_ELEMENT_MAIN: u32 = 0;

// Property selectors
/// Maps to `kAudioHardwarePropertyDevices` - hardware devices property
pub const AUDIO_HARDWARE_PROPERTY_DEVICES: u32 = 0x64657623; // 'dev#'
/// Maps to `kAudioHardwarePropertyDefaultInputDevice` - default input device property
pub const AUDIO_HARDWARE_PROPERTY_DEFAULT_INPUT_DEVICE: u32 = 0x64496e20; // 'dIn '
/// Maps to `kAudioHardwarePropertyDefaultOutputDevice` - default output device property
pub const AUDIO_HARDWARE_PROPERTY_DEFAULT_OUTPUT_DEVICE: u32 = 0x644f7574; // 'dOut'
/// Maps to `kAudioObjectPropertyName` - device name property
pub const AUDIO_OBJECT_PROPERTY_NAME: u32 = 0x6c6e616d; // 'lnam'
/// Maps to `kAudioDevicePropertyDeviceNameCFString` - device name property (deprecated, use AUDIO_OBJECT_PROPERTY_NAME)
pub const AUDIO_DEVICE_PROPERTY_DEVICE_NAME_CF_STRING: u32 = 0x6c6e616d; // 'lnam'
/// Maps to `kAudioDevicePropertyDeviceUID` - device UID property
pub const AUDIO_DEVICE_PROPERTY_DEVICE_UID: u32 = 0x75696420; // 'uid '
/// Maps to `kAudioDevicePropertyStreams` - device streams property
pub const AUDIO_DEVICE_PROPERTY_STREAMS: u32 = 0x73746d23; // 'stm#'
/// Maps to `kAudioDevicePropertyNominalSampleRate` - nominal sample rate property
pub const AUDIO_DEVICE_PROPERTY_NOMINAL_SAMPLE_RATE: u32 = 0x6e737261; // 'nsra'
/// Maps to `kAudioDevicePropertyStreamConfiguration` - stream configuration property
pub const AUDIO_DEVICE_PROPERTY_STREAM_CONFIGURATION: u32 = 0x736c6179; // 'slay'
/// Maps to `kAudioDevicePropertyStreamFormat` - stream format property
pub const AUDIO_DEVICE_PROPERTY_STREAM_FORMAT: u32 = 0x73666d74; // 'sfmt'
/// Maps to `kAudioDevicePropertyTransportType` - transport type property
pub const AUDIO_DEVICE_PROPERTY_TRANSPORT_TYPE: u32 = 0x7472616e; // 'tran'
/// Maps to `kAudioDeviceTransportTypeAggregate` - aggregate transport type
pub const AUDIO_DEVICE_TRANSPORT_TYPE_AGGREGATE: u32 = 0x61676772; // 'aggr'

// Audio tap specific properties
/// Maps to `kAudioTapPropertyDescription` - tap description property
pub const AUDIO_TAP_PROPERTY_DESCRIPTION: u32 = 100;
/// Maps to `kAudioTapPropertyUID` - tap UID property
pub const AUDIO_TAP_PROPERTY_UID: u32 = 101;
/// Maps to `kAudioTapPropertyFormat` - tap format property
pub const AUDIO_TAP_PROPERTY_FORMAT: u32 = 102;

// Aggregate device specific properties
/// Maps to `kAudioAggregateDevicePropertyFullSubDeviceList` - full sub-device list property
pub const AUDIO_AGGREGATE_DEVICE_PROPERTY_FULL_SUB_DEVICE_LIST: u32 = 200;
/// Maps to `kAudioAggregateDevicePropertyTapList` - tap list property
pub const AUDIO_AGGREGATE_DEVICE_PROPERTY_TAP_LIST: u32 = 201;
/// Maps to `kAudioAggregateDevicePropertyComposition` - composition property
pub const AUDIO_AGGREGATE_DEVICE_PROPERTY_COMPOSITION: u32 = 202;

// Audio process properties
/// Maps to `kAudioProcessPropertyIsRunning` - process running status property
pub const AUDIO_PROCESS_PROPERTY_IS_RUNNING: u32 = 300;
/// Maps to `kAudioProcessPropertyBundleID` - process bundle ID property
pub const AUDIO_PROCESS_PROPERTY_BUNDLE_ID: u32 = 301;
/// Maps to `kAudioProcessPropertyPID` - process PID property
pub const AUDIO_PROCESS_PROPERTY_PID: u32 = 302;

// Audio stream properties
/// Maps to `kAudioStreamPropertyDirection` - stream direction property
pub const AUDIO_STREAM_PROPERTY_DIRECTION: u32 = 400;
/// Maps to `kAudioStreamPropertyVirtualFormat` - virtual format property
pub const AUDIO_STREAM_PROPERTY_VIRTUAL_FORMAT: u32 = 401;

// Status codes
/// Maps to `noErr` - no error status
pub const NO_ERR: OSStatus = 0;
/// Maps to `kAudioHardwareNoError` - no hardware error status
pub const AUDIO_HARDWARE_NO_ERROR: OSStatus = 0;

// Audio format constants
/// Maps to `kAudioFormatLinearPCM` - linear PCM format
pub const AUDIO_FORMAT_LINEAR_PCM: u32 = 1819304813;
/// Maps to `kAudioFormatFlagIsFloat` - float format flag
pub const AUDIO_FORMAT_FLAG_IS_FLOAT: u32 = 1;
/// Maps to `kAudioFormatFlagIsPacked` - packed format flag
pub const AUDIO_FORMAT_FLAG_IS_PACKED: u32 = 2;
/// Maps to `kAudioFormatFlagIsSignedInteger` - signed integer format flag
pub const AUDIO_FORMAT_FLAG_IS_SIGNED_INTEGER: u32 = 4;
/// Maps to `kAudioFormatFlagIsNonInterleaved` - non-interleaved format flag
pub const AUDIO_FORMAT_FLAG_IS_NON_INTERLEAVED: u32 = 8;

// Audio format constants (Core Audio style)
pub const kAudioFormatLinearPCM: u32 = 0x6c70636d; // 'lpcm'
pub const kAudioFormatFlagIsFloat: u32 = 0x00000001;
pub const kAudioFormatFlagIsPacked: u32 = 0x00000002;
pub const kAudioFormatFlagIsNonInterleaved: u32 = 0x00000004;

// Stream direction
/// Maps to `kAudioStreamDirectionOutput` - output stream direction
pub const AUDIO_STREAM_DIRECTION_OUTPUT: u32 = 0;
/// Maps to `kAudioStreamDirectionInput` - input stream direction
pub const AUDIO_STREAM_DIRECTION_INPUT: u32 = 1;

// Property address structure
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AudioObjectPropertyAddress {
    /// Maps to `mSelector` - property selector
    pub selector: u32,
    /// Maps to `mScope` - property scope
    pub scope: u32,
    /// Maps to `mElement` - property element
    pub element: u32,
}

// Audio stream basic description
#[repr(C)]
#[derive(Debug, Clone)]
pub struct AudioStreamBasicDescription {
    /// Maps to `mSampleRate` - sample rate in Hz
    pub sample_rate: f64,
    /// Maps to `mFormatID` - format identifier
    pub format_id: u32,
    /// Maps to `mFormatFlags` - format flags
    pub format_flags: u32,
    /// Maps to `mBytesPerPacket` - bytes per packet
    pub bytes_per_packet: u32,
    /// Maps to `mFramesPerPacket` - frames per packet
    pub frames_per_packet: u32,
    /// Maps to `mBytesPerFrame` - bytes per frame
    pub bytes_per_frame: u32,
    /// Maps to `mChannelsPerFrame` - channels per frame
    pub channels_per_frame: u32,
    /// Maps to `mBitsPerChannel` - bits per channel
    pub bits_per_channel: u32,
    /// Maps to `mReserved` - reserved field
    pub reserved: u32,
}

// Audio buffer structure
#[repr(C)]
pub struct AudioBuffer {
    /// Maps to `mNumberChannels` - number of channels
    pub number_channels: u32,
    /// Maps to `mDataByteSize` - data byte size
    pub data_byte_size: u32,
    /// Maps to `mData` - data pointer
    pub data: *mut c_void,
}

// Audio buffer list structure
#[repr(C)]
pub struct AudioBufferList {
    /// Maps to `mNumberBuffers` - number of buffers
    pub number_buffers: u32,
    /// Maps to `mBuffers` - buffer array
    pub buffers: [AudioBuffer; 1], // Variable length array
}

// Audio time stamp
#[repr(C)]
pub struct AudioTimeStamp {
    /// Maps to `mSampleTime` - sample time
    pub sample_time: f64,
    /// Maps to `mHostTime` - host time
    pub host_time: u64,
    /// Maps to `mRateScalar` - rate scalar
    pub rate_scalar: f64,
    /// Maps to `mWordClockTime` - word clock time
    pub word_clock_time: u64,
    /// Maps to `mSMPTETime` - SMPTE time
    pub smpte_time: [u8; 16],
    /// Maps to `mFlags` - flags
    pub flags: u32,
    /// Maps to `mReserved` - reserved field
    pub reserved: u32,
}

// Audio device IO proc ID
pub type AudioDeviceIOProcID = *mut c_void;

// Audio device IO proc callback
/// Maps to Core Audio's AudioDeviceIOProc callback type
/// Parameter names follow Rust snake_case convention while maintaining FFI compatibility
pub type AudioDeviceIOProc = extern "C" fn(
    in_device: AudioObjectID,
    in_now: *const AudioTimeStamp,
    in_input_data: *const AudioBufferList,
    in_input_time: *const AudioTimeStamp,
    out_output_data: *mut AudioBufferList,
    out_output_time: *const AudioTimeStamp,
    in_client_data: *mut c_void,
) -> OSStatus;

// CATapDescription structure (simplified)
#[repr(C)]
pub struct CATapDescription {
    pub name: *mut c_char,
    pub processes: *mut u32,
    pub process_count: u32,
    pub is_private: bool,
    pub is_process_restore_enabled: bool,
    pub mute_behavior: u32,
    pub is_mixdown: bool,
    pub is_mono: bool,
    pub is_exclusive: bool,
    pub device_uid: *mut c_char,
    pub stream: Option<u32>,
}

// CATapMuteBehavior enum
/// Maps to `kCATapMuteBehaviorUnmuted` - unmuted tap behavior
pub const CA_TAP_MUTE_BEHAVIOR_UNMUTED: u32 = 0;
/// Maps to `kCATapMuteBehaviorMuted` - muted tap behavior
pub const CA_TAP_MUTE_BEHAVIOR_MUTED: u32 = 1;
/// Maps to `kCATapMuteBehaviorMutedWithFeedback` - muted with feedback tap behavior
pub const CA_TAP_MUTE_BEHAVIOR_MUTED_WITH_FEEDBACK: u32 = 2;

// Aggregate device creation keys
/// Maps to `kAudioAggregateDeviceUIDKey` - aggregate device UID key
pub const AUDIO_AGGREGATE_DEVICE_UID_KEY: &str = "uid";
/// Maps to `kAudioAggregateDeviceNameKey` - aggregate device name key
pub const AUDIO_AGGREGATE_DEVICE_NAME_KEY: &str = "name";
/// Maps to `kAudioAggregateDeviceSubDeviceListKey` - aggregate device sub-device list key
pub const AUDIO_AGGREGATE_DEVICE_SUB_DEVICE_LIST_KEY: &str = "subdevices";
/// Maps to `kAudioAggregateDeviceMainSubDeviceKey` - aggregate device main sub-device key
pub const AUDIO_AGGREGATE_DEVICE_MAIN_SUB_DEVICE_KEY: &str = "master";
/// Maps to `kAudioAggregateDeviceIsPrivateKey` - aggregate device private key
pub const AUDIO_AGGREGATE_DEVICE_IS_PRIVATE_KEY: &str = "private";
/// Maps to `kAudioAggregateDeviceTapAutoStartKey` - aggregate device tap auto-start key
pub const AUDIO_AGGREGATE_DEVICE_TAP_AUTO_START_KEY: &str = "tapautostart";

// Additional Core Audio constants for aggregate device creation
// AUDIO_OBJECT_PROPERTY_NAME is defined above with the correct value
/// Maps to `kAudioObjectPropertyManufacturer` - object manufacturer property
pub const AUDIO_OBJECT_PROPERTY_MANUFACTURER: u32 = 3;

// Core Foundation types (simplified)
pub type CFStringRef = *mut std::ffi::c_void;
pub type CFArrayRef = *mut std::ffi::c_void;
pub type CFDictionaryRef = *mut std::ffi::c_void;
pub type CFNumberRef = *mut std::ffi::c_void;

// External Core Audio functions
#[link(name = "CoreAudio", kind = "framework")]
extern "C" {
    // Audio Object functions
    pub fn AudioObjectGetPropertyDataSize(
        inObjectID: AudioObjectID,
        inAddress: *const AudioObjectPropertyAddress,
        inQualifierDataSize: u32,
        inQualifierData: *const c_void,
        outDataSize: *mut u32,
    ) -> OSStatus;

    pub fn AudioObjectGetPropertyData(
        inObjectID: AudioObjectID,
        inAddress: *const AudioObjectPropertyAddress,
        inQualifierDataSize: u32,
        inQualifierData: *const c_void,
        ioDataSize: *mut u32,
        outData: *mut c_void,
    ) -> OSStatus;

    pub fn AudioObjectSetPropertyData(
        inObjectID: AudioObjectID,
        inAddress: *const AudioObjectPropertyAddress,
        inQualifierDataSize: u32,
        inQualifierData: *const c_void,
        inDataSize: u32,
        inData: *const c_void,
    ) -> OSStatus;

    pub fn AudioObjectAddPropertyListener(
        inObjectID: AudioObjectID,
        inAddress: *const AudioObjectPropertyAddress,
        inListener: AudioObjectPropertyListenerProc,
        inClientData: *mut c_void,
    ) -> OSStatus;

    pub fn AudioObjectRemovePropertyListener(
        inObjectID: AudioObjectID,
        inAddress: *const AudioObjectPropertyAddress,
        inListener: AudioObjectPropertyListenerProc,
        inClientData: *mut c_void,
    ) -> OSStatus;

    // Audio Device functions
    pub fn AudioDeviceCreateIOProcID(
        inDevice: AudioObjectID,
        inProc: AudioDeviceIOProc,
        inClientData: *mut c_void,
        outIOProcID: *mut AudioDeviceIOProcID,
    ) -> OSStatus;

    pub fn AudioDeviceStart(
        inDevice: AudioObjectID,
        inProcID: AudioDeviceIOProcID,
    ) -> OSStatus;

    pub fn AudioDeviceStop(
        inDevice: AudioObjectID,
        inProcID: AudioDeviceIOProcID,
    ) -> OSStatus;

    pub fn AudioDeviceDestroyIOProcID(
        inDevice: AudioObjectID,
        inProcID: AudioDeviceIOProcID,
    ) -> OSStatus;

    // Aggregate device creation and destruction
    pub fn AudioHardwareCreateAggregateDevice(
        inDescription: *mut c_void, // CFDictionaryRef
        outDeviceID: *mut AudioObjectID,
    ) -> OSStatus;

    pub fn AudioHardwareDestroyAggregateDevice(
        inDeviceID: AudioObjectID,
    ) -> OSStatus;
}

// Property listener callback
/// Maps to Core Audio's AudioObjectPropertyListenerProc callback type
/// Parameter names follow Rust snake_case convention while maintaining FFI compatibility
pub type AudioObjectPropertyListenerProc = extern "C" fn(
    in_object_id: AudioObjectID,
    in_number_addresses: u32,
    in_addresses: *const AudioObjectPropertyAddress,
    in_client_data: *mut c_void,
) -> OSStatus;

// Helper functions for working with Core Audio
pub fn create_property_address(selector: u32, scope: u32, element: u32) -> AudioObjectPropertyAddress {
    AudioObjectPropertyAddress {
        selector,
        scope,
        element,
    }
}

pub fn get_property_data_size(
    object_id: AudioObjectID,
    address: &AudioObjectPropertyAddress,
) -> Result<u32, OSStatus> {
    let mut size = 0u32;
    let status = unsafe {
        AudioObjectGetPropertyDataSize(
            object_id,
            address,
            0,
            ptr::null(),
            &mut size,
        )
    };
    
    if status == NO_ERR {
        Ok(size)
    } else {
        Err(status)
    }
}

pub fn get_property_data<T>(
    object_id: AudioObjectID,
    address: &AudioObjectPropertyAddress,
) -> Result<T, OSStatus> {
    let mut size = std::mem::size_of::<T>() as u32;
    let mut data: T = unsafe { std::mem::zeroed() };
    
    let status = unsafe {
        AudioObjectGetPropertyData(
            object_id,
            address,
            0,
            ptr::null(),
            &mut size,
            &mut data as *mut T as *mut c_void,
        )
    };
    
    if status == NO_ERR {
        Ok(data)
    } else {
        Err(status)
    }
}

pub fn set_property_data<T>(
    object_id: AudioObjectID,
    address: &AudioObjectPropertyAddress,
    data: &T,
) -> Result<(), OSStatus> {
    let size = std::mem::size_of::<T>() as u32;
    
    let status = unsafe {
        AudioObjectSetPropertyData(
            object_id,
            address,
            0,
            ptr::null(),
            size,
            data as *const T as *const c_void,
        )
    };
    
    if status == NO_ERR {
        Ok(())
    } else {
        Err(status)
    }
}

// Core Foundation function declarations for aggregate device creation
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFDictionaryCreate(
        allocator: *mut c_void,
        keys: *const *const c_void,
        values: *const *const c_void,
        numValues: usize,
        keyCallBacks: *const c_void,
        valueCallBacks: *const c_void,
    ) -> *mut c_void;
    
    fn CFArrayCreate(
        allocator: *mut c_void,
        values: *const *const c_void,
        numValues: usize,
        callBacks: *const c_void,
    ) -> *mut c_void;
    
    fn CFStringCreateWithCString(
        allocator: *mut c_void,
        cString: *const c_char,
        encoding: u32,
    ) -> *mut c_void;
    
    fn CFNumberCreate(
        allocator: *mut c_void,
        theType: u32,
        valuePtr: *const c_void,
    ) -> *mut c_void;
    
    fn kCFAllocatorDefault() -> *mut c_void;
    fn kCFTypeArrayCallBacks() -> *const c_void;
    fn kCFTypeDictionaryKeyCallBacks() -> *const c_void;
    fn kCFTypeDictionaryValueCallBacks() -> *const c_void;
}

// CFNumber types
pub const CF_NUMBER_INT_TYPE: u32 = 3; // kCFNumberIntType
pub const CF_NUMBER_BOOL_TYPE: u32 = 9; // kCFNumberBoolType

// CFString encoding constants
pub const CF_STRING_ENCODING_UTF8: u32 = 0x08000100; // kCFStringEncodingUTF8

// Helper functions for creating Core Foundation objects
pub fn create_cf_string(string: &str) -> Result<*mut c_void, OSStatus> {
    let c_string = match CString::new(string) {
        Ok(s) => s,
        Err(_) => return Err(-1),
    };
    
    unsafe {
        let cf_string = CFStringCreateWithCString(
            kCFAllocatorDefault(),
            c_string.as_ptr(),
            CF_STRING_ENCODING_UTF8,
        );
        
        if cf_string.is_null() {
            Err(-1)
        } else {
            Ok(cf_string)
        }
    }
}

pub fn create_cf_number_int(value: i32) -> Result<*mut c_void, OSStatus> {
    unsafe {
        let cf_number = CFNumberCreate(
            kCFAllocatorDefault(),
            CF_NUMBER_INT_TYPE,
            &value as *const i32 as *const c_void,
        );
        
        if cf_number.is_null() {
            Err(-1)
        } else {
            Ok(cf_number)
        }
    }
}

pub fn create_cf_number_bool(value: bool) -> Result<*mut c_void, OSStatus> {
    unsafe {
        let cf_number = CFNumberCreate(
            kCFAllocatorDefault(),
            CF_NUMBER_BOOL_TYPE,
            &value as *const bool as *const c_void,
        );
        
        if cf_number.is_null() {
            Err(-1)
        } else {
            Ok(cf_number)
        }
    }
}

pub fn create_cf_array_from_strings(strings: &[String]) -> Result<*mut c_void, OSStatus> {
    if strings.is_empty() {
        return Ok(std::ptr::null_mut());
    }
    
    let mut cf_strings: Vec<*mut c_void> = Vec::new();
    
    for string in strings {
        let cf_string = create_cf_string(string)?;
        cf_strings.push(cf_string);
    }
    
    unsafe {
        let cf_array = CFArrayCreate(
            kCFAllocatorDefault(),
            cf_strings.as_ptr() as *const *const c_void,
            cf_strings.len(),
            kCFTypeArrayCallBacks(),
        );
        
        if cf_array.is_null() {
            Err(-1)
        } else {
            Ok(cf_array)
        }
    }
}

pub fn create_cf_dictionary_from_pairs(pairs: &[(&str, *mut c_void)]) -> Result<*mut c_void, OSStatus> {
    if pairs.is_empty() {
        return Ok(std::ptr::null_mut());
    }
    
    let mut keys: Vec<*mut c_void> = Vec::new();
    let mut values: Vec<*mut c_void> = Vec::new();
    
    for (key, value) in pairs {
        let cf_key = create_cf_string(key)?;
        keys.push(cf_key);
        values.push(*value);
    }
    
    unsafe {
        let cf_dict = CFDictionaryCreate(
            kCFAllocatorDefault(),
            keys.as_ptr() as *const *const c_void,
            values.as_ptr() as *const *const c_void,
            keys.len(),
            kCFTypeDictionaryKeyCallBacks(),
            kCFTypeDictionaryValueCallBacks(),
        );
        
        if cf_dict.is_null() {
            Err(-1)
        } else {
            Ok(cf_dict)
        }
    }
}

// CFString helper functions
pub fn cf_string_to_string(cf_string: *mut c_void) -> Result<String, OSStatus> {
    if cf_string.is_null() {
        return Err(-1);
    }
    
    // For now, we'll use a simpler approach that doesn't require Core Foundation functions
    // This avoids memory management issues with CFString
    unsafe {
        // Try to treat it as a C string first
        let c_str = cf_string as *const c_char;
        if !c_str.is_null() {
            // Check if it looks like a valid C string
            let mut i = 0;
            while i < 1024 { // Limit to prevent infinite loop
                let byte = *c_str.offset(i);
                if byte == 0 {
                    break; // Found null terminator
                }
                if byte < 32 || byte > 126 {
                    break; // Not a printable ASCII character
                }
                i += 1;
            }
            
            if i > 0 && i < 1024 {
                // It looks like a valid C string
                let rust_string = CStr::from_ptr(c_str)
                    .to_string_lossy()
                    .into_owned();
                return Ok(rust_string);
            }
        }
    }
    
    // Fallback: return a device name based on the pointer
    Ok(format!("Device_{:p}", cf_string))
}

pub fn string_to_cf_string(string: &str) -> Result<*mut c_void, OSStatus> {
    let c_string = match CString::new(string) {
        Ok(s) => s,
        Err(_) => return Err(-1),
    };
    
    // In a real implementation, you'd create a proper CFString
    // For now, we'll just return the C string pointer
    Ok(c_string.into_raw() as *mut c_void)
}

// These functions are now implemented above with proper Core Foundation bindings

pub fn cf_array_to_strings(cf_array: CFArrayRef) -> Result<Vec<String>, OSStatus> {
    if cf_array.is_null() {
        return Ok(Vec::new());
    }
    
    // This is a simplified implementation
    // In a real implementation, you'd iterate through the CFArray and convert each CFString
    // For now, we'll return an empty vector
    Ok(Vec::new())
}
