use super::core_audio_bindings::{
    create_aggregate_device, destroy_aggregate_device, get_audio_device_ids,
    get_default_input_device, get_device_name, get_device_transport_type, AggregateDevice,
};
use objc2_core_audio::{kAudioDeviceTransportTypeAggregate, AudioObjectID};

pub struct DeviceLists {
    pub real_device_list: Vec<AudioObjectID>,
    pub aggregate_device_list: Vec<AggregateDevice>,
}

pub fn load_devices() -> Result<DeviceLists, String> {
    let mut device_lists = DeviceLists {
        real_device_list: Vec::new(),
        aggregate_device_list: Vec::new(),
    };

    let device_ids: Vec<AudioObjectID> =
        get_audio_device_ids().map_err(|e| format!("Failed to get audio device IDs: {}", e))?;

    for device_id in device_ids {
        println!("Device ID: {}", device_id);
        let transport_type = get_device_transport_type(device_id)
            .map_err(|e| format!("Failed to get device transport type: {}", e))?;
        if transport_type == kAudioDeviceTransportTypeAggregate {
            println!("Device is an aggregate device");
            device_lists
                .aggregate_device_list
                .push(AggregateDevice::new(device_id)?);
        } else {
            println!("Device is a real device");
            device_lists.real_device_list.push(device_id);
        }
    }

    Ok(device_lists)
}

pub fn clean_own_aggregate_devices() -> Result<(), String> {
    let device_ids = get_audio_device_ids()?;

    for device_id in device_ids {
        let transport_type = get_device_transport_type(device_id)?;
        if transport_type == kAudioDeviceTransportTypeAggregate {
            let name = get_device_name(device_id)?;
            // Destroy only our own aggregate devices.
            if name.to_lowercase().contains("enteract") {
                destroy_aggregate_device(device_id)?;
            }
        }
    }

    Ok(())
}

pub fn create_microphone_aggregate_device() -> Result<AggregateDevice, String> {
    let uuid = uuid::Uuid::new_v4().to_string();
    let device_name = format!("Enteract Microphone Aggregate Device {}", uuid);
    let device = create_aggregate_device(device_name, format!("enteract-microphone-{}", uuid))?;
    let default_input_device = get_default_input_device()?;
    device.add_sub_device(default_input_device)?;
    Ok(device)
}
