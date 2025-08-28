// src-tauri/src/audio_loopback/macos/audio_recorder.rs
use crate::audio_loopback::macos::core_audio_bindings::{
    catalog_device_streams, get_device_name_safe, DeviceStreamCatalog, StreamInfo,
};
use anyhow::{Context, Result};
use atomic_float::AtomicF32;
use objc2_core_audio::*;
use objc2_core_audio_types::{
    AudioBuffer, AudioBufferList, AudioStreamBasicDescription, AudioTimeStamp,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{atomic::AtomicBool, Mutex};
use tauri::AppHandle;

/// Audio processor trait for processing captured audio
pub trait AudioProcessor {
    fn process_audio(&self, samples: Vec<f32>, sample_rate: f32) -> Result<()>;
}

/// Simple WAV file writer
struct WavFileWriter {
    file: std::fs::File,
    data_size: u32,
    sample_rate: u32,
    channels: u16,
    bits_per_sample: u16,
}

impl WavFileWriter {
    fn new(
        file_path: &PathBuf,
        sample_rate: u32,
        channels: u16,
        bits_per_sample: u16,
    ) -> Result<Self> {
        let mut file = std::fs::File::create(file_path)?;

        // Write WAV header
        Self::write_wav_header(&mut file, sample_rate, channels, bits_per_sample)?;

        Ok(Self {
            file,
            data_size: 0,
            sample_rate,
            channels,
            bits_per_sample,
        })
    }

    fn write_wav_header(
        file: &mut std::fs::File,
        sample_rate: u32,
        channels: u16,
        bits_per_sample: u16,
    ) -> Result<()> {
        use std::io::Write;

        // RIFF header
        file.write_all(b"RIFF")?;
        file.write_all(&[0, 0, 0, 0])?; // File size (to be filled later)
        file.write_all(b"WAVE")?;

        // fmt chunk
        file.write_all(b"fmt ")?;
        file.write_all(&16u32.to_le_bytes())?; // fmt chunk size
        file.write_all(&1u16.to_le_bytes())?; // PCM format
        file.write_all(&channels.to_le_bytes())?; // channels
        file.write_all(&sample_rate.to_le_bytes())?; // sample rate
        file.write_all(
            &((sample_rate * channels as u32 * bits_per_sample as u32) / 8).to_le_bytes(),
        )?; // byte rate
        file.write_all(&((channels * bits_per_sample) / 8).to_le_bytes())?; // block align
        file.write_all(&bits_per_sample.to_le_bytes())?; // bits per sample

        // data chunk
        file.write_all(b"data")?;
        file.write_all(&[0, 0, 0, 0])?; // data size (to be filled later)

        Ok(())
    }

    fn write_audio_data(&mut self, audio_data: &[f32]) -> Result<()> {
        use std::io::Write;

        // Convert f32 samples to the target bit depth
        let bytes_per_sample = self.bits_per_sample as usize / 8;
        let mut buffer = Vec::with_capacity(audio_data.len() * bytes_per_sample);

        for &sample in audio_data {
            // Clamp sample to [-1.0, 1.0] range
            let clamped_sample = sample.max(-1.0).min(1.0);

            match self.bits_per_sample {
                16 => {
                    // Convert to 16-bit PCM
                    let pcm_sample = (clamped_sample * 32767.0) as i16;
                    buffer.extend_from_slice(&pcm_sample.to_le_bytes());
                }
                24 => {
                    // Convert to 24-bit PCM
                    let pcm_sample = (clamped_sample * 8388607.0) as i32;
                    buffer.extend_from_slice(&pcm_sample.to_le_bytes()[..3]);
                }
                32 => {
                    // Convert to 32-bit PCM
                    let pcm_sample = (clamped_sample * 2147483647.0) as i32;
                    buffer.extend_from_slice(&pcm_sample.to_le_bytes());
                }
                _ => {
                    return Err(anyhow::anyhow!(
                        "Unsupported bits per sample: {}",
                        self.bits_per_sample
                    ));
                }
            }
        }

        self.file.write_all(&buffer)?;
        self.data_size += buffer.len() as u32;

        Ok(())
    }

    fn finalize(&mut self) -> Result<()> {
        use std::io::{Seek, SeekFrom, Write};

        // Update file size in RIFF header
        let file_size = 36 + self.data_size; // 36 bytes for header + data size
        self.file.seek(SeekFrom::Start(4))?;
        self.file.write_all(&file_size.to_le_bytes())?;

        // Update data size in data chunk
        self.file.seek(SeekFrom::Start(40))?;
        self.file.write_all(&self.data_size.to_le_bytes())?;

        Ok(())
    }
}

/// Audio recorder for capturing audio from Core Audio devices
pub struct AudioRecorder {
    device_id: Mutex<AudioObjectID>,
    stream_catalog: Mutex<DeviceStreamCatalog>,
    audio_processor: Mutex<Option<Box<dyn AudioProcessor + Send + Sync>>>,
    recording_enabled: AtomicBool,
    loopback_enabled: AtomicBool,
    io_proc_id: Mutex<Option<AudioDeviceIOProcID>>,
    frame_counter: Mutex<u64>,
    recording_files: Mutex<HashMap<u32, WavFileWriter>>,
    recording_paths: Mutex<HashMap<u32, PathBuf>>,
}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            device_id: Mutex::new(kAudioObjectUnknown),
            stream_catalog: Mutex::new(DeviceStreamCatalog::new()),
            audio_processor: Mutex::new(None),
            recording_enabled: AtomicBool::new(false),
            loopback_enabled: AtomicBool::new(false),
            io_proc_id: Mutex::new(None),
            frame_counter: Mutex::new(0),
            recording_files: Mutex::new(HashMap::new()),
            recording_paths: Mutex::new(HashMap::new()),
        }
    }

    // Add basic methods for testing
    pub fn set_app_handle(&mut self, app_handle: AppHandle) {
        // self.app_handle = Some(app_handle); // This field was removed
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

        *self.device_id.lock().unwrap() = device_id;
        Ok(())
    }

    pub fn get_device_id(&self) -> AudioObjectID {
        *self.device_id.lock().unwrap()
    }

    pub fn get_current_sample_rate(&self) -> f32 {
        self.stream_catalog.lock().unwrap().get_sample_rate()
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
            self.get_device_id()
        );

        if self.get_device_id() == kAudioObjectUnknown {
            println!("[AudioRecorder] No device set, skipping stream cataloging");
            return Ok(());
        }

        // Use the Core Audio bindings to catalog streams
        let catalog = catalog_device_streams(self.get_device_id())?;

        // Update our internal catalog (no data copying, just moving ownership)
        {
            let mut internal_catalog = self.stream_catalog.lock().unwrap();
            *internal_catalog = catalog;
        }

        // Update sample rate
        let sample_rate = self.stream_catalog.lock().unwrap().get_sample_rate();
        // self.current_sample_rate // This field was removed
        // .store(sample_rate, std::sync::atomic::Ordering::Relaxed);

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

    /// IO Proc callback function - called by Core Audio for each audio buffer
    /// This matches the signature expected by the Core Audio framework
    pub unsafe extern "C-unwind" fn audio_io_proc(
        _in_device: AudioObjectID,
        _in_now: std::ptr::NonNull<AudioTimeStamp>,
        in_input_data: std::ptr::NonNull<AudioBufferList>,
        _in_input_time: std::ptr::NonNull<AudioTimeStamp>,
        out_output_data: std::ptr::NonNull<AudioBufferList>,
        _in_output_time: std::ptr::NonNull<AudioTimeStamp>,
        in_client_data: *mut std::ffi::c_void,
    ) -> i32 {
        // Get the AudioRecorder instance from client data
        let recorder = &*(in_client_data as *const AudioRecorder);

        // Get input buffer information
        let input_data = in_input_data.as_ref();
        let number_input_buffers = input_data.mNumberBuffers;
        let number_frames_to_record = if number_input_buffers > 0 {
            let first_buffer = &input_data.mBuffers[0];
            first_buffer.mDataByteSize
                / (first_buffer.mNumberChannels * std::mem::size_of::<f32>() as u32)
        } else {
            0
        };

        // Get output buffer information
        let output_data = out_output_data.as_ref();
        let number_output_buffers = output_data.mNumberBuffers;
        let _number_frames_to_output = if number_output_buffers > 0 {
            let first_buffer = &output_data.mBuffers[0];
            first_buffer.mDataByteSize
                / (first_buffer.mNumberChannels * std::mem::size_of::<f32>() as u32)
        } else {
            0
        };

        // Process each input buffer
        for index in 0..number_input_buffers {
            let buffer = &input_data.mBuffers[index as usize];

            // Check if recording is enabled
            if recorder.is_recording_enabled() {
                // Write audio buffer to recording files
                if let Err(e) =
                    recorder.write_audio_buffer_to_files(buffer, index, number_frames_to_record)
                {
                    println!("[AudioRecorder] Error writing to recording file: {}", e);
                }
            }

            // Check if loopback is enabled
            if recorder.is_loopback_enabled() && index < number_output_buffers {
                // Copy input data to output buffer for loopback
                // Use unsafe to access the mutable output buffer
                unsafe {
                    let output_buffer = &mut (*out_output_data.as_ptr()).mBuffers[index as usize];
                    if buffer.mDataByteSize <= output_buffer.mDataByteSize {
                        std::ptr::copy_nonoverlapping(
                            buffer.mData as *const u8,
                            output_buffer.mData as *mut u8,
                            buffer.mDataByteSize as usize,
                        );
                    }
                }
            }

            // Process audio data for our audio processor (replacing Whisper transcription)
            if !buffer.mData.is_null() {
                // Convert audio buffer to float array
                let float_data = buffer.mData as *const f32;
                let total_samples = buffer.mDataByteSize / std::mem::size_of::<f32>() as u32;
                let channels = buffer.mNumberChannels;
                let frames = total_samples / channels;

                // Debug logging (every 100 frames to avoid spam)
                static FRAME_COUNTER: std::sync::atomic::AtomicU64 =
                    std::sync::atomic::AtomicU64::new(0);
                let frame_count = FRAME_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                if frame_count % 100 == 0 {
                    println!(
                        "[AudioRecorder] Audio: {} frames, {} channels, {} total samples",
                        frames, channels, total_samples
                    );

                    // Log some sample values to verify we have audio
                    if total_samples > 0 {
                        let mut max_value = 0.0f32;
                        let samples_to_check = std::cmp::min(total_samples, 100);
                        for i in 0..samples_to_check {
                            // Only check first 100 samples to avoid performance impact
                            let sample = *float_data.offset(i as isize);
                            max_value = max_value.max(sample.abs());
                        }
                        println!("[AudioRecorder] Audio buffer max amplitude: {}", max_value);
                    }
                }

                // Convert to mono samples (similar to Objective-C++ implementation)
                let mut mono_samples = Vec::with_capacity(frames as usize);
                for frame in 0..frames {
                    let mut sample = 0.0f32;

                    if channels == 1 {
                        // Mono: use the single channel
                        sample = *float_data.offset(frame as isize);
                    } else if channels == 2 {
                        // Stereo: average left and right channels
                        let left = *float_data.offset((frame * 2) as isize);
                        let right = *float_data.offset((frame * 2 + 1) as isize);
                        sample = (left + right) * 0.5f32;
                    } else {
                        // Multi-channel: average all channels
                        for ch in 0..channels {
                            sample += *float_data.offset((frame * channels + ch) as isize);
                        }
                        sample /= channels as f32;
                    }

                    mono_samples.push(sample);
                }

                // Send audio data to our audio processor (replacing Whisper manager)
                recorder.process_audio_data(mono_samples);
            }
        }

        // Return success (kAudioHardwareNoError = 0)
        0
    }

    /// Process audio data through the audio processor
    fn process_audio_data(&self, samples: Vec<f32>) {
        // Get the current sample rate
        let sample_rate = self.get_current_sample_rate();

        // Try to get the audio processor and process the audio
        if let Ok(processor_guard) = self.audio_processor.lock() {
            if let Some(processor) = processor_guard.as_ref() {
                if let Err(e) = processor.process_audio(samples, sample_rate) {
                    println!("[AudioRecorder] Error processing audio: {}", e);
                }
            }
        }
    }

    /// Start IO Proc for audio capture
    pub fn start_io(&mut self) -> Result<()> {
        use crate::audio_loopback::macos::core_audio_bindings::{
            create_io_proc_id, start_audio_device, AudioDeviceIOProc,
        };

        println!("[AudioRecorder] Starting IO");

        // Create IO Proc ID
        let io_proc_id = create_io_proc_id(
            self.get_device_id(),
            Self::audio_io_proc as AudioDeviceIOProc,
            self as *const _ as *mut std::ffi::c_void,
        )?;

        // Store the IO Proc ID
        if let Ok(mut io_proc_guard) = self.io_proc_id.lock() {
            *io_proc_guard = Some(io_proc_id);
        }

        // Start the audio device
        start_audio_device(self.get_device_id(), io_proc_id)?;

        println!("[AudioRecorder] IO started successfully");
        Ok(())
    }

    /// Stop IO Proc
    pub fn stop_io(&mut self) -> Result<()> {
        use crate::audio_loopback::macos::core_audio_bindings::{
            destroy_io_proc_id, stop_audio_device,
        };

        println!("[AudioRecorder] Stopping IO");

        // Get the IO Proc ID
        let io_proc_id = if let Ok(io_proc_guard) = self.io_proc_id.lock() {
            io_proc_guard.clone()
        } else {
            return Err(anyhow::anyhow!("Failed to lock IO Proc ID"));
        };

        if let Some(proc_id) = io_proc_id {
            // Stop the audio device
            stop_audio_device(self.get_device_id(), proc_id)?;

            // Destroy the IO Proc ID
            destroy_io_proc_id(self.get_device_id(), proc_id)?;

            // Clear the stored IO Proc ID
            if let Ok(mut io_proc_guard) = self.io_proc_id.lock() {
                *io_proc_guard = None;
            }
        }

        println!("[AudioRecorder] IO stopped successfully");
        Ok(())
    }

    /// Enable recording
    pub fn enable_recording(&mut self) {
        self.recording_enabled
            .store(true, std::sync::atomic::Ordering::Relaxed);
        println!("[AudioRecorder] Recording enabled");
    }

    /// Disable recording
    pub fn disable_recording(&mut self) {
        self.recording_enabled
            .store(false, std::sync::atomic::Ordering::Relaxed);
        println!("[AudioRecorder] Recording disabled");
    }

    /// Enable loopback
    pub fn enable_loopback(&mut self) {
        self.loopback_enabled
            .store(true, std::sync::atomic::Ordering::Relaxed);
        println!("[AudioRecorder] Loopback enabled");
    }

    /// Disable loopback
    pub fn disable_loopback(&mut self) {
        self.loopback_enabled
            .store(false, std::sync::atomic::Ordering::Relaxed);
        println!("[AudioRecorder] Loopback disabled");
    }

    /// Set audio processor
    pub fn set_audio_processor(&mut self, processor: Box<dyn AudioProcessor + Send + Sync>) {
        if let Ok(mut processor_guard) = self.audio_processor.lock() {
            *processor_guard = Some(processor);
            println!("[AudioRecorder] Audio processor set");
        }
    }

    /// Set up recording files for all input streams
    pub fn setup_recording_files(&mut self, output_dir: &PathBuf) -> Result<()> {
        println!(
            "[AudioRecorder] Setting up recording files in: {}",
            output_dir.display()
        );

        // Ensure output directory exists
        std::fs::create_dir_all(output_dir)?;

        // Get stream catalog
        let catalog = self.stream_catalog.lock().unwrap();
        let input_streams = &catalog.input_streams;

        if input_streams.is_empty() {
            return Err(anyhow::anyhow!("No input streams available for recording"));
        }

        // Generate timestamp for unique filenames
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();

        // Create recording files for each input stream
        for (index, stream) in input_streams.iter().enumerate() {
            let filename = format!("recording_{}_stream_{}.wav", timestamp, index);
            let file_path = output_dir.join(&filename);

            println!(
                "[AudioRecorder] Creating recording file: {}",
                file_path.display()
            );

            // Create WAV file writer
            let sample_rate = stream.format.mSampleRate as u32;
            let channels = stream.format.mChannelsPerFrame as u16;
            let bits_per_sample = stream.format.mBitsPerChannel as u16;

            match WavFileWriter::new(&file_path, sample_rate, channels, bits_per_sample) {
                Ok(writer) => {
                    // Store the writer and path
                    if let Ok(mut files_guard) = self.recording_files.lock() {
                        files_guard.insert(index as u32, writer);
                    }
                    if let Ok(mut paths_guard) = self.recording_paths.lock() {
                        paths_guard.insert(index as u32, file_path);
                    }
                    println!(
                        "[AudioRecorder] Created WAV recording file for stream {} ({} Hz, {} ch, {} bit)",
                        index, sample_rate, channels, bits_per_sample
                    );
                }
                Err(e) => {
                    println!(
                        "[AudioRecorder] Failed to create recording file for stream {}: {}",
                        index, e
                    );
                }
            }
        }

        Ok(())
    }

    /// Write audio buffer to recording files
    fn write_audio_buffer_to_files(
        &self,
        buffer: &AudioBuffer,
        stream_index: u32,
        _frames: u32,
    ) -> Result<()> {
        // Get the recording file for this stream
        if let Ok(mut files_guard) = self.recording_files.lock() {
            if let Some(writer) = files_guard.get_mut(&stream_index) {
                // Convert audio buffer to f32 samples
                let audio_data = unsafe {
                    let ptr = buffer.mData as *const f32;
                    let samples_count = buffer.mDataByteSize as usize / std::mem::size_of::<f32>();
                    std::slice::from_raw_parts(ptr, samples_count)
                };

                // Write audio data to WAV file
                if let Err(e) = writer.write_audio_data(audio_data) {
                    println!("[AudioRecorder] Error writing audio data: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Clean up recording files
    pub fn cleanup_recording_files(&mut self) -> Result<()> {
        println!("[AudioRecorder] Cleaning up recording files");

        // Finalize and close all recording files
        if let Ok(mut files_guard) = self.recording_files.lock() {
            for (&stream_index, writer) in files_guard.iter_mut() {
                if let Err(e) = writer.finalize() {
                    println!(
                        "[AudioRecorder] Error finalizing file for stream {}: {}",
                        stream_index, e
                    );
                }
                println!(
                    "[AudioRecorder] Finalized recording file for stream {}",
                    stream_index
                );
            }
            files_guard.clear();
        }

        // Clear paths
        if let Ok(mut paths_guard) = self.recording_paths.lock() {
            paths_guard.clear();
        }

        println!("[AudioRecorder] Recording files cleaned up");
        Ok(())
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
