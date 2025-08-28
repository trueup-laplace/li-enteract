// src-tauri/src/audio_loopback/macos/audio_recorder.rs
use crate::audio_loopback::macos::core_audio_bindings::{
    catalog_device_streams, get_device_name_safe, DeviceStreamCatalog, StreamInfo,
};
use anyhow::Result;
use atomic_float::AtomicF32;
use objc2_core_audio::*;
use objc2_core_audio_types::AudioStreamBasicDescription;
use std::sync::{atomic::AtomicBool, Mutex};
use tauri::AppHandle;

pub struct AudioRecorder {
    device_id: AudioObjectID,
    stream_catalog: Mutex<DeviceStreamCatalog>,
    recording_enabled: AtomicBool,
    loopback_enabled: AtomicBool,
    io_proc_id: Mutex<Option<AudioDeviceIOProcID>>,
    current_sample_rate: AtomicF32,
    app_handle: Option<AppHandle>,
}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            device_id: kAudioObjectUnknown,
            stream_catalog: Mutex::new(DeviceStreamCatalog::new()),
            recording_enabled: AtomicBool::new(false),
            loopback_enabled: AtomicBool::new(false),
            io_proc_id: Mutex::new(None),
            current_sample_rate: AtomicF32::new(48000.0),
            app_handle: None,
        }
    }

    // Add basic methods for testing
    pub fn set_app_handle(&mut self, app_handle: AppHandle) {
        self.app_handle = Some(app_handle);
    }

    pub fn set_device_id(&mut self, device_id: AudioObjectID) -> Result<()> {
        println!("[AudioRecorder] Setting device ID: {}", device_id);

        // Get device name for debugging
        if device_id != kAudioObjectUnknown {
            match get_device_name_safe(device_id) {
                Ok(name) => println!("[AudioRecorder] Device name: {}", name),
                Err(e) => println!("[AudioRecorder] Failed to get device name: {}", e),
            }
        }

        self.device_id = device_id;
        Ok(())
    }

    pub fn get_device_id(&self) -> AudioObjectID {
        self.device_id
    }

    pub fn get_current_sample_rate(&self) -> f32 {
        self.current_sample_rate
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn is_recording_enabled(&self) -> bool {
        self.recording_enabled
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn is_loopback_enabled(&self) -> bool {
        self.loopback_enabled
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn get_input_stream_count(&self) -> usize {
        self.stream_catalog.lock().unwrap().get_input_stream_count()
    }

    pub fn get_output_stream_count(&self) -> usize {
        self.stream_catalog
            .lock()
            .unwrap()
            .get_output_stream_count()
    }

    /// Catalog device streams using Core Audio bindings
    pub fn catalog_device_streams(&mut self) -> Result<()> {
        println!(
            "[AudioRecorder] Cataloging streams for device: {}",
            self.device_id
        );

        if self.device_id == kAudioObjectUnknown {
            println!("[AudioRecorder] No device set, skipping stream cataloging");
            return Ok(());
        }

        // Use the Core Audio bindings to catalog streams
        let catalog = catalog_device_streams(self.device_id)?;

        // Update our internal catalog (no data copying, just moving ownership)
        {
            let mut internal_catalog = self.stream_catalog.lock().unwrap();
            *internal_catalog = catalog;
        }

        // Update sample rate
        let sample_rate = self.stream_catalog.lock().unwrap().get_sample_rate();
        self.current_sample_rate
            .store(sample_rate, std::sync::atomic::Ordering::Relaxed);

        Ok(())
    }

    /// Adapt to a new device - catalog streams and update state
    pub fn adapt_to_device(&mut self, device_id: AudioObjectID) -> Result<()> {
        println!("[AudioRecorder] Adapting to device: {}", device_id);

        // Stop any existing IO
        self.stop_io()?;

        // Set new device ID
        self.set_device_id(device_id)?;

        // Catalog device streams
        self.catalog_device_streams()?;

        // TODO: Register listeners for device changes
        // self.register_listeners()?;

        // TODO: Restart IO if needed
        // if self.is_recording_enabled() {
        //     self.start_recording()?;
        // } else if self.is_loopback_enabled() {
        //     self.start_io()?;
        // }

        Ok(())
    }

    /// Stop IO (placeholder for now)
    fn stop_io(&self) -> Result<()> {
        // TODO: Implement actual IO stopping
        println!("[AudioRecorder] Stopping IO (placeholder)");
        Ok(())
    }

    /// Get input stream formats (returns owned copies for safety)
    pub fn get_input_stream_formats(&self) -> Vec<AudioStreamBasicDescription> {
        let catalog = self.stream_catalog.lock().unwrap();
        catalog
            .input_streams
            .iter()
            .map(|stream| stream.format)
            .collect()
    }

    /// Get output stream formats (returns owned copies for safety)
    pub fn get_output_stream_formats(&self) -> Vec<AudioStreamBasicDescription> {
        let catalog = self.stream_catalog.lock().unwrap();
        catalog
            .output_streams
            .iter()
            .map(|stream| stream.format)
            .collect()
    }

    /// Get stream info for detailed access
    pub fn get_input_stream_info(&self) -> Vec<StreamInfo> {
        let catalog = self.stream_catalog.lock().unwrap();
        catalog.input_streams.clone()
    }

    /// Get stream info for detailed access
    pub fn get_output_stream_info(&self) -> Vec<StreamInfo> {
        let catalog = self.stream_catalog.lock().unwrap();
        catalog.output_streams.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_loopback::macos::device_enumerator::CoreAudioLoopbackEnumerator;
    use crate::audio_loopback::DeviceType;
    use std::sync::Arc;

    /// Test 1: Basic AudioRecorder creation and initialization
    #[test]
    fn test_audio_recorder_creation() {
        println!("[AUDIO_RECORDER_TEST] Testing AudioRecorder creation...");

        let recorder = AudioRecorder::new();

        // Test default values
        assert_eq!(recorder.get_device_id(), kAudioObjectUnknown);
        assert_eq!(recorder.get_current_sample_rate(), 48000.0);
        assert!(!recorder.is_recording_enabled());
        assert!(!recorder.is_loopback_enabled());
        assert_eq!(recorder.get_input_stream_count(), 0);
        assert_eq!(recorder.get_output_stream_count(), 0);

        println!("[AUDIO_RECORDER_TEST] ✓ AudioRecorder creation test passed");
    }

    /// Test 2: Device ID setting and retrieval
    #[test]
    fn test_device_id_management() {
        println!("[AUDIO_RECORDER_TEST] Testing device ID management...");

        let mut recorder = AudioRecorder::new();

        // Test setting device ID
        let test_device_id = 12345;
        assert!(recorder.set_device_id(test_device_id).is_ok());
        assert_eq!(recorder.get_device_id(), test_device_id);

        // Test setting same device ID (should not error)
        assert!(recorder.set_device_id(test_device_id).is_ok());

        // Test setting different device ID
        let new_device_id = 67890;
        assert!(recorder.set_device_id(new_device_id).is_ok());
        assert_eq!(recorder.get_device_id(), new_device_id);

        println!("[AUDIO_RECORDER_TEST] ✓ Device ID management test passed");
    }

    /// Test 3: Sample rate management
    #[test]
    fn test_sample_rate_management() {
        println!("[AUDIO_RECORDER_TEST] Testing sample rate management...");

        let recorder = AudioRecorder::new();

        // Test default sample rate
        assert_eq!(recorder.get_current_sample_rate(), 48000.0);

        // Test that sample rate is atomic and thread-safe
        let recorder_arc = Arc::new(recorder);
        let recorder_clone = recorder_arc.clone();

        let handle = std::thread::spawn(move || {
            // This would test atomic operations if we had a setter
            recorder_clone.get_current_sample_rate()
        });

        let result = handle.join().unwrap();
        assert_eq!(result, 48000.0);

        println!("[AUDIO_RECORDER_TEST] ✓ Sample rate management test passed");
    }

    /// Test 4: Recording and loopback state management
    #[test]
    fn test_recording_loopback_states() {
        println!("[AUDIO_RECORDER_TEST] Testing recording and loopback states...");

        let recorder = AudioRecorder::new();

        // Test initial states
        assert!(!recorder.is_recording_enabled());
        assert!(!recorder.is_loopback_enabled());

        // Test that states are atomic and thread-safe
        let recorder_arc = Arc::new(recorder);
        let recorder_clone = recorder_arc.clone();

        let handle = std::thread::spawn(move || {
            recorder_clone.is_recording_enabled() && recorder_clone.is_loopback_enabled()
        });

        let result = handle.join().unwrap();
        assert!(!result); // Both should be false

        println!("[AUDIO_RECORDER_TEST] ✓ Recording and loopback states test passed");
    }

    /// Test 5: Stream list management
    #[test]
    fn test_stream_list_management() {
        println!("[AUDIO_RECORDER_TEST] Testing stream list management...");

        let recorder = AudioRecorder::new();

        // Test initial empty stream lists
        assert_eq!(recorder.get_input_stream_count(), 0);
        assert_eq!(recorder.get_output_stream_count(), 0);

        println!("[AUDIO_RECORDER_TEST] ✓ Stream list management test passed");
    }

    /// Test 6: Integration with device enumerator
    #[test]
    fn test_device_enumerator_integration() {
        println!("[AUDIO_RECORDER_TEST] Testing device enumerator integration...");

        // Test that we can enumerate devices and set device IDs
        match CoreAudioLoopbackEnumerator::new() {
            Ok(enumerator) => {
                match enumerator.enumerate_loopback_devices() {
                    Ok(devices) => {
                        if !devices.is_empty() {
                            // Test with first available device
                            let first_device = &devices[0];
                            let device_id: AudioObjectID = first_device.id.parse().unwrap();

                            let mut recorder = AudioRecorder::new();
                            assert!(recorder.set_device_id(device_id).is_ok());
                            assert_eq!(recorder.get_device_id(), device_id);

                            println!(
                                "[AUDIO_RECORDER_TEST] ✓ Tested with real device: {} (ID: {})",
                                first_device.name, device_id
                            );
                        } else {
                            println!("[AUDIO_RECORDER_TEST] ⚠ No devices available for testing");
                        }
                    }
                    Err(e) => {
                        println!("[AUDIO_RECORDER_TEST] ⚠ Failed to enumerate devices: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("[AUDIO_RECORDER_TEST] ⚠ Failed to create enumerator: {}", e);
            }
        }

        println!("[AUDIO_RECORDER_TEST] ✓ Device enumerator integration test completed");
    }

    /// Test 7: Thread safety
    #[test]
    fn test_thread_safety() {
        println!("[AUDIO_RECORDER_TEST] Testing thread safety...");

        let recorder_arc = Arc::new(AudioRecorder::new());
        let mut handles = Vec::new();

        // Spawn multiple threads to test concurrent access
        for i in 0..5 {
            let recorder_clone = recorder_arc.clone();
            let handle = std::thread::spawn(move || {
                // Test concurrent reads
                let device_id = recorder_clone.get_device_id();
                let sample_rate = recorder_clone.get_current_sample_rate();
                let recording = recorder_clone.is_recording_enabled();
                let loopback = recorder_clone.is_loopback_enabled();

                (device_id, sample_rate, recording, loopback, i)
            });
            handles.push(handle);
        }

        // Collect results
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.join().unwrap());
        }

        // Verify all threads got consistent results
        for (device_id, sample_rate, recording, loopback, _) in &results {
            assert_eq!(*device_id, kAudioObjectUnknown);
            assert_eq!(*sample_rate, 48000.0);
            assert!(!*recording);
            assert!(!*loopback);
        }

        println!("[AUDIO_RECORDER_TEST] ✓ Thread safety test passed");
    }

    /// Test 8: Error handling
    #[test]
    fn test_error_handling() {
        println!("[AUDIO_RECORDER_TEST] Testing error handling...");

        let mut recorder = AudioRecorder::new();

        // Test setting invalid device ID (should not panic)
        let invalid_device_id = 0; // kAudioObjectUnknown
        assert!(recorder.set_device_id(invalid_device_id).is_ok());

        // Test setting very large device ID (should not panic)
        let large_device_id = u32::MAX;
        assert!(recorder.set_device_id(large_device_id).is_ok());

        println!("[AUDIO_RECORDER_TEST] ✓ Error handling test passed");
    }

    /// Test 9: Memory management
    #[test]
    fn test_memory_management() {
        println!("[AUDIO_RECORDER_TEST] Testing memory management...");

        // Test that AudioRecorder can be dropped without issues
        {
            let _recorder = AudioRecorder::new();
            // Recorder should be dropped here
        }

        // Test that multiple recorders can coexist
        let recorders: Vec<AudioRecorder> = (0..10).map(|_| AudioRecorder::new()).collect();
        assert_eq!(recorders.len(), 10);

        // Test that recorders can be moved
        let mut recorders_vec = Vec::new();
        for i in 0..5 {
            let mut recorder = AudioRecorder::new();
            recorder.set_device_id(i).unwrap();
            recorders_vec.push(recorder);
        }

        for (i, recorder) in recorders_vec.iter().enumerate() {
            assert_eq!(recorder.get_device_id(), i as AudioObjectID);
        }

        println!("[AUDIO_RECORDER_TEST] ✓ Memory management test passed");
    }

    /// Test 10: Stream cataloging functionality
    #[test]
    fn test_stream_cataloging() {
        println!("[AUDIO_RECORDER_TEST] Testing stream cataloging...");

        let mut recorder = AudioRecorder::new();

        // Test with no device set
        assert!(recorder.catalog_device_streams().is_ok());
        assert_eq!(recorder.get_input_stream_count(), 0);
        assert_eq!(recorder.get_output_stream_count(), 0);

        // Test with real device
        match CoreAudioLoopbackEnumerator::new() {
            Ok(enumerator) => {
                match enumerator.enumerate_loopback_devices() {
                    Ok(devices) => {
                        if !devices.is_empty() {
                            let first_device = &devices[0];
                            let device_id: AudioObjectID = first_device.id.parse().unwrap();

                            recorder.set_device_id(device_id).unwrap();
                            assert!(recorder.catalog_device_streams().is_ok());

                            println!(
                                "[AUDIO_RECORDER_TEST] Device: {} (ID: {})",
                                first_device.name, device_id
                            );
                            println!(
                                "[AUDIO_RECORDER_TEST] Input streams: {}",
                                recorder.get_input_stream_count()
                            );
                            println!(
                                "[AUDIO_RECORDER_TEST] Output streams: {}",
                                recorder.get_output_stream_count()
                            );
                            println!(
                                "[AUDIO_RECORDER_TEST] Sample rate: {}Hz",
                                recorder.get_current_sample_rate()
                            );

                            // Validate stream counts based on device type
                            match first_device.device_type {
                                DeviceType::Render => {
                                    // Render devices should have output streams
                                    assert!(
                                        recorder.get_output_stream_count() > 0,
                                        "Render device should have output streams"
                                    );
                                }
                                DeviceType::Capture => {
                                    // Capture devices should have input streams
                                    assert!(
                                        recorder.get_input_stream_count() > 0,
                                        "Capture device should have input streams"
                                    );
                                }
                            }

                            // Sample rate should be valid
                            assert!(
                                recorder.get_current_sample_rate() > 0.0,
                                "Sample rate should be positive"
                            );
                        } else {
                            println!("[AUDIO_RECORDER_TEST] ⚠ No devices available for stream cataloging test");
                        }
                    }
                    Err(e) => {
                        println!("[AUDIO_RECORDER_TEST] ⚠ Failed to enumerate devices: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("[AUDIO_RECORDER_TEST] ⚠ Failed to create enumerator: {}", e);
            }
        }

        println!("[AUDIO_RECORDER_TEST] ✓ Stream cataloging test completed");
    }

    /// Test 11: Device adaptation
    #[test]
    fn test_device_adaptation() {
        println!("[AUDIO_RECORDER_TEST] Testing device adaptation...");

        let mut recorder = AudioRecorder::new();

        // Test adaptation with no device
        assert!(recorder.adapt_to_device(kAudioObjectUnknown).is_ok());
        assert_eq!(recorder.get_device_id(), kAudioObjectUnknown);
        assert_eq!(recorder.get_input_stream_count(), 0);
        assert_eq!(recorder.get_output_stream_count(), 0);

        // Test adaptation with real device
        match CoreAudioLoopbackEnumerator::new() {
            Ok(enumerator) => {
                match enumerator.enumerate_loopback_devices() {
                    Ok(devices) => {
                        if !devices.is_empty() {
                            let first_device = &devices[0];
                            let device_id: AudioObjectID = first_device.id.parse().unwrap();

                            assert!(recorder.adapt_to_device(device_id).is_ok());
                            assert_eq!(recorder.get_device_id(), device_id);

                            // Should have cataloged streams
                            let input_count = recorder.get_input_stream_count();
                            let output_count = recorder.get_output_stream_count();
                            assert!(
                                input_count > 0 || output_count > 0,
                                "Device should have at least one stream"
                            );

                            println!(
                                "[AUDIO_RECORDER_TEST] Adapted to device: {} (ID: {})",
                                first_device.name, device_id
                            );
                            println!("[AUDIO_RECORDER_TEST] Streams after adaptation: {} input, {} output", 
                                     input_count, output_count);
                        } else {
                            println!(
                                "[AUDIO_RECORDER_TEST] ⚠ No devices available for adaptation test"
                            );
                        }
                    }
                    Err(e) => {
                        println!("[AUDIO_RECORDER_TEST] ⚠ Failed to enumerate devices: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("[AUDIO_RECORDER_TEST] ⚠ Failed to create enumerator: {}", e);
            }
        }

        println!("[AUDIO_RECORDER_TEST] ✓ Device adaptation test completed");
    }

    /// Test 12: Stream format access performance
    #[test]
    fn test_stream_format_access() {
        println!("[AUDIO_RECORDER_TEST] Testing stream format access...");

        let mut recorder = AudioRecorder::new();

        match CoreAudioLoopbackEnumerator::new() {
            Ok(enumerator) => {
                match enumerator.enumerate_loopback_devices() {
                    Ok(devices) => {
                        if !devices.is_empty() {
                            let first_device = &devices[0];
                            let device_id: AudioObjectID = first_device.id.parse().unwrap();

                            recorder.set_device_id(device_id).unwrap();
                            recorder.catalog_device_streams().unwrap();

                            // Test format access (should be fast, no copying)
                            let input_formats = recorder.get_input_stream_formats();
                            let output_formats = recorder.get_output_stream_formats();

                            println!(
                                "[AUDIO_RECORDER_TEST] Input formats: {}",
                                input_formats.len()
                            );
                            println!(
                                "[AUDIO_RECORDER_TEST] Output formats: {}",
                                output_formats.len()
                            );

                            // Validate formats
                            for (i, format) in input_formats.iter().enumerate() {
                                assert!(
                                    format.mSampleRate > 0.0,
                                    "Input format {} should have positive sample rate",
                                    i
                                );
                                assert!(
                                    format.mChannelsPerFrame > 0,
                                    "Input format {} should have positive channel count",
                                    i
                                );
                            }

                            for (i, format) in output_formats.iter().enumerate() {
                                assert!(
                                    format.mSampleRate > 0.0,
                                    "Output format {} should have positive sample rate",
                                    i
                                );
                                assert!(
                                    format.mChannelsPerFrame > 0,
                                    "Output format {} should have positive channel count",
                                    i
                                );
                            }
                        }
                    }
                    Err(e) => {
                        println!("[AUDIO_RECORDER_TEST] ⚠ Failed to enumerate devices: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("[AUDIO_RECORDER_TEST] ⚠ Failed to create enumerator: {}", e);
            }
        }

        println!("[AUDIO_RECORDER_TEST] ✓ Stream format access test completed");
    }
}
