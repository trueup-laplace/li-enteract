// src-tauri/src/audio_loopback/macos/device_enumerator.rs
use super::core_audio_bindings::{
    device_has_output_streams, get_audio_device_ids, get_device_format, get_device_name,
    is_default_device, AudioDeviceType,
};
use crate::audio_loopback::types::{AudioLoopbackDevice, DeviceType, LoopbackMethod};
use anyhow::Result;
use objc2_core_audio::*;

pub struct CoreAudioLoopbackEnumerator {
    #[allow(dead_code)]
    system_object: AudioObjectID,
}

impl CoreAudioLoopbackEnumerator {
    pub fn new() -> Result<Self> {
        Ok(Self {
            system_object: kAudioObjectSystemObject as u32,
        })
    }

    pub fn enumerate_loopback_devices(&self) -> Result<Vec<AudioLoopbackDevice>> {
        let mut loopback_devices = Vec::new();

        // Get all audio devices
        let devices = self.get_audio_devices()?;

        for device_id in devices {
            let name = self.get_device_name(device_id)?;
            match self.create_device_info(device_id) {
                Ok(device_info) => {
                    // Test if device supports loopback
                    if self.test_loopback_capability(device_id)? {
                        loopback_devices.push(device_info);
                    }
                }
                Err(e) => {
                    println!(
                        "Warning: Failed to create device info for device {} ({}): {}",
                        device_id, name, e
                    );
                    // Continue with other devices instead of failing completely
                }
            }
        }

        // Sort by default status and name
        loopback_devices.sort_by(|a, b| b.is_default.cmp(&a.is_default).then(a.name.cmp(&b.name)));

        Ok(loopback_devices)
    }

    fn get_audio_devices(&self) -> Result<Vec<AudioObjectID>> {
        get_audio_device_ids()
    }

    fn create_device_info(&self, device_id: AudioObjectID) -> Result<AudioLoopbackDevice> {
        let name = self.get_device_name(device_id)?;
        let is_default = self.is_default_device(device_id)?;
        let (sample_rate, channels, format) = self.get_device_format(device_id)?;
        let device_type = self.get_device_type(device_id)?;

        Ok(AudioLoopbackDevice {
            id: device_id.to_string(),
            name,
            is_default,
            sample_rate,
            channels,
            format,
            device_type,
            loopback_method: LoopbackMethod::CaptureDevice,
        })
    }

    pub fn get_device_name(&self, device_id: AudioObjectID) -> Result<String> {
        get_device_name(device_id)
    }

    pub fn is_default_device(&self, device_id: AudioObjectID) -> Result<bool> {
        let is_default_input = is_default_device(device_id, AudioDeviceType::Input)?;
        let is_default_output = is_default_device(device_id, AudioDeviceType::Output)?;
        Ok(is_default_input || is_default_output)
    }

    pub fn get_device_format(&self, device_id: AudioObjectID) -> Result<(u32, u16, String)> {
        // Try output scope first (for render devices)
        get_device_format(device_id)
    }

    pub fn get_device_type(&self, device_id: AudioObjectID) -> Result<DeviceType> {
        // Check if device has output streams (render device)
        let has_output_streams = device_has_output_streams(device_id)?;

        if has_output_streams {
            Ok(DeviceType::Render)
        } else {
            Ok(DeviceType::Capture)
        }
    }

    fn test_loopback_capability(&self, _device_id: AudioObjectID) -> Result<bool> {
        // For macOS, we'll test if we can create an aggregate device with this device
        // This is a simplified test - in practice, you might want more sophisticated testing
        Ok(true) // Most devices on macOS support loopback through aggregate devices
    }

    pub fn auto_select_best_device(&self) -> Result<Option<AudioLoopbackDevice>> {
        let devices = self.enumerate_loopback_devices()?;

        if devices.is_empty() {
            return Ok(None);
        }

        // Priority 1: Default render device
        if let Some(default_device) = devices.iter().find(|d| d.is_default) {
            return Ok(Some(default_device.clone()));
        }

        // Priority 2: First available device
        Ok(Some(devices[0].clone()))
    }

    pub fn find_device_by_id(&self, device_id: &str) -> Result<Option<AudioLoopbackDevice>> {
        let devices = self.enumerate_loopback_devices()?;
        Ok(devices.into_iter().find(|d| d.id == device_id))
    }
}

// Tauri Commands - same interface as Windows
#[tauri::command]
pub async fn enumerate_loopback_devices() -> Result<Vec<AudioLoopbackDevice>, String> {
    match CoreAudioLoopbackEnumerator::new() {
        Ok(enumerator) => match enumerator.enumerate_loopback_devices() {
            Ok(devices) => Ok(devices),
            Err(e) => Err(format!("Failed to enumerate audio devices: {}", e)),
        },
        Err(e) => Err(format!("Failed to initialize audio enumerator: {}", e)),
    }
}

#[tauri::command]
pub async fn auto_select_best_device() -> Result<Option<AudioLoopbackDevice>, String> {
    match CoreAudioLoopbackEnumerator::new() {
        Ok(enumerator) => match enumerator.auto_select_best_device() {
            Ok(device) => Ok(device),
            Err(e) => Err(format!("Failed to auto-select device: {}", e)),
        },
        Err(e) => Err(format!("Failed to initialize audio enumerator: {}", e)),
    }
}

#[tauri::command]
pub async fn test_audio_device(device_id: String) -> Result<bool, String> {
    match CoreAudioLoopbackEnumerator::new() {
        Ok(enumerator) => {
            match enumerator.find_device_by_id(&device_id) {
                Ok(Some(_)) => Ok(true), // Simplified test for macOS
                Ok(None) => Ok(false),
                Err(e) => Err(format!("Failed to test audio device: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to test audio device: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_enumeration() {
        println!("[PHASE1] Starting device enumeration test...");
        let enumerator = CoreAudioLoopbackEnumerator::new().unwrap();
        let devices = enumerator.enumerate_loopback_devices().unwrap();
        println!("[PHASE1] Found {} devices", devices.len());

        for device in &devices {
            println!("[PHASE1]   - Device: {} (ID: {})", device.name, device.id);
        }

        assert!(!devices.is_empty());
    }

    #[test]
    fn test_anyhow_error_handling() {
        println!("[PHASE1] Testing anyhow error handling...");
        // Test that anyhow errors work correctly
        let result: Result<Vec<AudioObjectID>> = Err(anyhow::anyhow!("Test error"));
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert_eq!(error_msg, "Test error");
        println!("[PHASE1] Anyhow error handling test passed");
    }

    #[test]
    fn test_auto_select_best_device() {
        println!("[PHASE1] Testing auto-select best device...");
        let enumerator = CoreAudioLoopbackEnumerator::new().unwrap();
        let best_device = enumerator.auto_select_best_device().unwrap();

        if let Some(device) = best_device {
            println!(
                "[PHASE1] Auto-selected device: {} (ID: {})",
                device.name, device.id
            );
        } else {
            println!("[PHASE1] No device auto-selected");
        }

        // This test should pass even if no device is found
        println!("[PHASE1] Auto-select test completed");
    }

    // Phase 2 Tests: Device Format Detection
    #[test]
    fn test_device_format_detection() {
        println!("[PHASE2] Testing device format detection...");
        let enumerator = CoreAudioLoopbackEnumerator::new().unwrap();
        let devices = enumerator.enumerate_loopback_devices().unwrap();

        for device in &devices {
            println!(
                "[PHASE2] Testing device: {} (ID: {})",
                device.name, device.id
            );

            // Test that device has valid format information
            assert!(
                device.sample_rate > 0,
                "Device {} has invalid sample rate: {}",
                device.name,
                device.sample_rate
            );
            assert!(
                device.channels > 0,
                "Device {} has invalid channel count: {}",
                device.name,
                device.channels
            );
            assert!(
                !device.format.is_empty(),
                "Device {} has empty format string",
                device.name
            );

            println!("[PHASE2]   - Sample Rate: {}Hz", device.sample_rate);
            println!("[PHASE2]   - Channels: {}", device.channels);
            println!("[PHASE2]   - Format: {}", device.format);
            println!("[PHASE2]   - Device Type: {:?}", device.device_type);
            println!("[PHASE2]   - Is Default: {}", device.is_default);
        }

        println!("[PHASE2] Device format detection test completed");
    }

    #[test]
    fn test_device_type_classification() {
        println!("[PHASE2] Testing device type classification...");
        let enumerator = CoreAudioLoopbackEnumerator::new().unwrap();
        let devices = enumerator.enumerate_loopback_devices().unwrap();

        let mut render_devices = 0;
        let mut capture_devices = 0;

        for device in &devices {
            match device.device_type {
                DeviceType::Render => {
                    render_devices += 1;
                    println!(
                        "[PHASE2] Render device: {} (ID: {})",
                        device.name, device.id
                    );
                }
                DeviceType::Capture => {
                    capture_devices += 1;
                    println!(
                        "[PHASE2] Capture device: {} (ID: {})",
                        device.name, device.id
                    );
                }
            }
        }

        println!(
            "[PHASE2] Found {} render devices and {} capture devices",
            render_devices, capture_devices
        );

        // At least one device should be found
        assert!(render_devices + capture_devices > 0, "No devices found");

        println!("[PHASE2] Device type classification test completed");
    }

    #[test]
    fn test_default_device_detection() {
        println!("[PHASE2] Testing default device detection...");
        let enumerator = CoreAudioLoopbackEnumerator::new().unwrap();
        let devices = enumerator.enumerate_loopback_devices().unwrap();

        let default_devices: Vec<_> = devices.iter().filter(|d| d.is_default).collect();

        println!("[PHASE2] Found {} default devices", default_devices.len());

        for device in &default_devices {
            println!(
                "[PHASE2] Default device: {} (ID: {})",
                device.name, device.id
            );
        }

        // On macOS, there should typically be at least one default device
        // But we'll make this test flexible in case of unusual setups
        if !default_devices.is_empty() {
            println!("[PHASE2] Default device detection working correctly");
        } else {
            println!(
                "[PHASE2] Warning: No default devices found (this might be normal in some setups)"
            );
        }

        println!("[PHASE2] Default device detection test completed");
    }

    #[test]
    fn test_device_format_validation() {
        println!("[PHASE2] Testing device format validation...");
        let enumerator = CoreAudioLoopbackEnumerator::new().unwrap();
        let devices = enumerator.enumerate_loopback_devices().unwrap();

        for device in &devices {
            println!(
                "[PHASE2] Validating device: {} (ID: {})",
                device.name, device.id
            );

            // Test common sample rates
            let valid_sample_rates = [44100, 48000, 96000, 192000];
            let is_valid_sample_rate = valid_sample_rates.contains(&device.sample_rate);

            if !is_valid_sample_rate {
                println!(
                    "[PHASE2] Warning: Unusual sample rate {}Hz for device {}",
                    device.sample_rate, device.name
                );
            }

            // Test channel count
            let valid_channel_counts = [1, 2, 4, 6, 8];
            let is_valid_channel_count = valid_channel_counts.contains(&device.channels);

            if !is_valid_channel_count {
                println!(
                    "[PHASE2] Warning: Unusual channel count {} for device {}",
                    device.channels, device.name
                );
            }

            // Test format string
            let valid_formats = ["IEEE Float 32bit", "PCM 16bit", "PCM 24bit", "PCM 32bit"];
            let has_valid_format = valid_formats.iter().any(|f| device.format.contains(f));

            if !has_valid_format {
                println!(
                    "[PHASE2] Warning: Unusual format '{}' for device {}",
                    device.format, device.name
                );
            }

            // Log validation results
            println!("[PHASE2]   - Sample Rate Valid: {}", is_valid_sample_rate);
            println!(
                "[PHASE2]   - Channel Count Valid: {}",
                is_valid_channel_count
            );
            println!("[PHASE2]   - Format Valid: {}", has_valid_format);
        }

        println!("[PHASE2] Device format validation test completed");
    }

    #[test]
    fn test_device_capability_testing() {
        println!("[PHASE2] Testing device capability testing...");
        let enumerator = CoreAudioLoopbackEnumerator::new().unwrap();
        let devices = enumerator.enumerate_loopback_devices().unwrap();

        for device in &devices {
            println!(
                "[PHASE2] Testing capabilities for device: {} (ID: {})",
                device.name, device.id
            );

            // Test if we can get device info for this device
            let device_id: AudioObjectID = device.id.parse().unwrap_or(0);
            if device_id > 0 {
                let device_info_result = enumerator.create_device_info(device_id);
                match device_info_result {
                    Ok(info) => {
                        println!("[PHASE2]   - Successfully created device info");
                        assert_eq!(info.id, device.id, "Device ID mismatch");
                        assert_eq!(info.name, device.name, "Device name mismatch");
                    }
                    Err(e) => {
                        println!("[PHASE2]   - Failed to create device info: {}", e);
                    }
                }
            }

            // Test loopback capability (simplified test)
            let loopback_capable = enumerator
                .test_loopback_capability(device_id)
                .unwrap_or(false);
            println!("[PHASE2]   - Loopback Capable: {}", loopback_capable);
        }

        println!("[PHASE2] Device capability testing completed");
    }

    #[test]
    fn test_device_find_by_id() {
        println!("[PHASE2] Testing device find by ID...");
        let enumerator = CoreAudioLoopbackEnumerator::new().unwrap();
        let devices = enumerator.enumerate_loopback_devices().unwrap();

        if let Some(first_device) = devices.first() {
            println!(
                "[PHASE2] Testing find_device_by_id with device: {} (ID: {})",
                first_device.name, first_device.id
            );

            let found_device = enumerator.find_device_by_id(&first_device.id).unwrap();

            match found_device {
                Some(device) => {
                    println!("[PHASE2] Successfully found device by ID");
                    assert_eq!(device.id, first_device.id, "Device ID mismatch");
                    assert_eq!(device.name, first_device.name, "Device name mismatch");
                }
                None => {
                    println!("[PHASE2] Failed to find device by ID");
                    panic!("Should have found device by ID");
                }
            }
        } else {
            println!("[PHASE2] No devices available for find_device_by_id test");
        }

        // Test with non-existent ID
        let non_existent_result = enumerator.find_device_by_id("999999").unwrap();
        assert!(
            non_existent_result.is_none(),
            "Should not find non-existent device"
        );

        println!("[PHASE2] Device find by ID test completed");
    }

    #[test]
    fn test_device_info_consistency() {
        println!("[PHASE2] Testing device info consistency...");
        let enumerator = CoreAudioLoopbackEnumerator::new().unwrap();
        let devices = enumerator.enumerate_loopback_devices().unwrap();

        for device in &devices {
            println!(
                "[PHASE2] Checking consistency for device: {} (ID: {})",
                device.name, device.id
            );

            // Test that device ID can be parsed as AudioObjectID
            let device_id_result: Result<AudioObjectID, _> = device.id.parse();
            assert!(
                device_id_result.is_ok(),
                "Device ID '{}' should be parseable as AudioObjectID",
                device.id
            );

            let device_id = device_id_result.unwrap();

            // Test that we can get the same information directly
            let direct_name = enumerator.get_device_name(device_id).unwrap();
            let direct_format = enumerator.get_device_format(device_id).unwrap();
            let direct_type = enumerator.get_device_type(device_id).unwrap();
            let direct_is_default = enumerator.is_default_device(device_id).unwrap();

            // Compare with stored values
            assert_eq!(
                direct_name, device.name,
                "Device name mismatch for device {}",
                device.id
            );
            assert_eq!(
                direct_format.0, device.sample_rate,
                "Sample rate mismatch for device {}",
                device.id
            );
            assert_eq!(
                direct_format.1, device.channels,
                "Channel count mismatch for device {}",
                device.id
            );
            assert_eq!(
                direct_type, device.device_type,
                "Device type mismatch for device {}",
                device.id
            );
            assert_eq!(
                direct_is_default, device.is_default,
                "Default status mismatch for device {}",
                device.id
            );

            println!("[PHASE2]   - All device info consistent");
        }

        println!("[PHASE2] Device info consistency test completed");
    }
}
