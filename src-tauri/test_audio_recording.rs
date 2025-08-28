//! Interactive test script for audio recording functionality
//!
//! This script tests the complete audio capture pipeline including:
//! - Device enumeration and selection
//! - Stream cataloging
//! - File recording setup
//! - Audio capture and file writing
//! - Cleanup

use std::{
    io::{self, Write},
    path::PathBuf,
    thread,
    time::Duration,
};

use anyhow::Result;
use objc2_core_audio_types::AudioStreamBasicDescription;

// Import our audio modules
use enteract_lib::audio_loopback::macos::{
    audio_recorder::AudioRecorder,
    core_audio_bindings::{
        device_has_input_streams, device_has_output_streams, get_audio_device_ids, get_device_name,
    },
};

/// Simple audio processor for testing
struct TestAudioProcessor {
    file_path: PathBuf,
    sample_count: std::sync::atomic::AtomicU64,
}

impl TestAudioProcessor {
    fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            sample_count: std::sync::atomic::AtomicU64::new(0),
        }
    }
}

impl enteract_lib::audio_loopback::macos::audio_recorder::AudioProcessor for TestAudioProcessor {
    fn process_audio(&self, samples: Vec<f32>, sample_rate: f32) -> Result<()> {
        let count = self
            .sample_count
            .fetch_add(samples.len() as u64, std::sync::atomic::Ordering::Relaxed);

        // Log every 1000 samples to avoid spam
        if count % 1000 < samples.len() as u64 {
            println!(
                "[TestAudioProcessor] Processed {} samples (total: {}), sample_rate: {}",
                samples.len(),
                count,
                sample_rate
            );

            // Log some sample values to verify we have audio
            if !samples.is_empty() {
                let max_amplitude = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
                println!("[TestAudioProcessor] Max amplitude: {}", max_amplitude);
            }
        }

        Ok(())
    }
}

/// Simple device info for testing
#[derive(Debug)]
struct TestDevice {
    id: u32,
    name: String,
    has_input: bool,
    has_output: bool,
}

fn enumerate_all_audio_devices() -> Result<Vec<TestDevice>> {
    let device_ids = get_audio_device_ids()?;
    let mut devices = Vec::new();

    for device_id in device_ids {
        match get_device_name(device_id) {
            Ok(name) => {
                let has_output = device_has_output_streams(device_id).unwrap_or(false);
                let has_input = device_has_input_streams(device_id).unwrap_or(false);

                devices.push(TestDevice {
                    id: device_id,
                    name,
                    has_input,
                    has_output,
                });
            }
            Err(e) => {
                println!(
                    "Warning: Failed to get name for device {}: {}",
                    device_id, e
                );
            }
        }
    }

    Ok(devices)
}

fn main() -> Result<()> {
    println!("🎵 Audio Recording Test Script");
    println!("==============================");

    // Step 1: Enumerate all available devices (both input and output)
    println!("\n1. Enumerating all audio devices...");
    let devices = enumerate_all_audio_devices()?;

    if devices.is_empty() {
        println!("❌ No audio devices found!");
        return Ok(());
    }

    println!("✅ Found {} audio devices:", devices.len());
    for (i, device) in devices.iter().enumerate() {
        let capabilities = if device.has_input && device.has_output {
            "Input/Output"
        } else if device.has_input {
            "Input"
        } else if device.has_output {
            "Output"
        } else {
            "Unknown"
        };
        println!(
            "   {}. {} (ID: {}) - {}",
            i + 1,
            device.name,
            device.id,
            capabilities
        );
    }

    // Step 2: Let user select a device
    print!("\nSelect a device (1-{}): ", devices.len());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let device_index: usize = input.trim().parse().unwrap_or(1) - 1;

    if device_index >= devices.len() {
        println!("❌ Invalid device selection!");
        return Ok(());
    }

    let selected_device = &devices[device_index];
    let device_id: u32 = selected_device.id;

    println!(
        "✅ Selected device: {} (ID: {}) - {}",
        selected_device.name,
        device_id,
        if selected_device.has_input && selected_device.has_output {
            "Input/Output"
        } else if selected_device.has_input {
            "Input"
        } else if selected_device.has_output {
            "Output"
        } else {
            "Unknown"
        }
    );

    // Step 3: Create AudioRecorder and set up device
    println!("\n2. Setting up AudioRecorder...");
    let mut recorder = AudioRecorder::new();
    recorder.set_device_id(device_id)?;

    // Step 4: Catalog device streams
    println!("\n3. Cataloging device streams...");
    recorder.catalog_device_streams()?;

    let input_count = recorder.get_input_stream_count();
    let output_count = recorder.get_output_stream_count();
    println!(
        "✅ Device streams: {} input, {} output",
        input_count, output_count
    );

    if input_count == 0 {
        println!("❌ No input streams available for recording!");
        println!("   This device may not support audio input or may need different configuration.");
        return Ok(());
    }

    // Step 5: Set up file recording
    println!("\n4. Setting up file recording...");

    // Create output directory
    let output_dir = PathBuf::from("test_recordings");
    std::fs::create_dir_all(&output_dir)?;

    // Set up recording files in the AudioRecorder
    recorder.setup_recording_files(&output_dir)?;

    println!(
        "✅ Recording files will be saved to: {}",
        output_dir.display()
    );

    // Step 6: Set up audio processor for testing
    println!("\n5. Setting up test audio processor...");
    let test_processor = TestAudioProcessor::new(output_dir.clone());
    recorder.set_audio_processor(Box::new(test_processor));

    // Step 7: Enable recording and loopback
    println!("\n6. Enabling recording and loopback...");
    recorder.enable_recording();
    recorder.enable_loopback();

    // Step 8: Start audio capture
    println!("\n7. Starting audio capture...");
    recorder.start_io()?;

    println!("✅ Audio capture started!");
    println!("🎤 Speak into your microphone or play some audio...");
    println!("⏱️  Recording for 10 seconds...");

    // Step 9: Record for 10 seconds
    thread::sleep(Duration::from_secs(10));

    // Step 10: Stop recording
    println!("\n8. Stopping audio capture...");
    recorder.stop_io()?;
    recorder.disable_recording();
    recorder.disable_loopback();

    // Clean up recording files
    recorder.cleanup_recording_files()?;

    println!("✅ Audio capture stopped!");

    // Step 11: Summary
    println!("\n📊 Recording Summary:");
    println!("   Device: {}", selected_device.name);
    println!("   Input streams: {}", input_count);
    println!("   Output streams: {}", output_count);
    println!("   Recording duration: 10 seconds");
    println!("   Output directory: {}", output_dir.display());

    // Step 12: Check if files were created
    let mut file_count = 0;
    for entry in std::fs::read_dir(&output_dir)? {
        if let Ok(entry) = entry {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("wav") {
                file_count += 1;
                let metadata = std::fs::metadata(entry.path())?;
                println!(
                    "   File: {} ({} bytes)",
                    entry.file_name().to_string_lossy(),
                    metadata.len()
                );
            }
        }
    }

    if file_count > 0 {
        println!("✅ {} recording file(s) created successfully!", file_count);
    } else {
        println!("❌ No recording files were created!");
    }

    println!("\n🎉 Test completed!");
    println!("Check the console output above for audio processing details.");

    Ok(())
}
