//! Audio streaming from aggregate devices using Core Audio

use crate::types::AudioCaptureResult;
use crate::macos::core_audio_bindings::*;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Audio buffer for streaming
#[derive(Debug, Clone)]
pub struct AudioBuffer {
    pub data: Vec<f32>,
    pub channels: u32,
    pub sample_rate: f64,
    pub timestamp: std::time::Instant,
}

/// Audio streaming configuration
#[derive(Debug, Clone)]
pub struct AudioStreamConfig {
    pub sample_rate: f64,
    pub channels: u32,
    pub buffer_size: usize,
    pub format: AudioStreamBasicDescription,
}

/// Callback for audio data
pub type AudioDataCallback = Box<dyn Fn(&AudioBuffer) + Send + Sync>;

/// Audio streamer for aggregate devices
pub struct AudioStreamer {
    device_id: AudioObjectID,
    io_proc_id: Option<AudioDeviceIOProcID>,
    config: AudioStreamConfig,
    callback: Option<AudioDataCallback>,
    is_running: bool,
    audio_buffer: Arc<Mutex<VecDeque<AudioBuffer>>>,
    // UI integration
    last_emit: Instant,
    transcription_buffer: Vec<f32>,
    last_transcription: Instant,
}

impl AudioStreamer {
    /// Create a new audio streamer for the given device
    pub fn new(device_id: AudioObjectID) -> AudioCaptureResult<Self> {
        let config = AudioStreamConfig {
            sample_rate: 48000.0,
            channels: 2,
            buffer_size: 1024,
            format: AudioStreamBasicDescription {
                sample_rate: 48000.0,
                format_id: kAudioFormatLinearPCM,
                format_flags: kAudioFormatFlagIsFloat | kAudioFormatFlagIsPacked | kAudioFormatFlagIsNonInterleaved,
                bytes_per_packet: 4,
                frames_per_packet: 1,
                bytes_per_frame: 4,
                channels_per_frame: 2,
                bits_per_channel: 32,
                reserved: 0,
            },
        };

        Ok(Self {
            device_id,
            io_proc_id: None,
            config,
            callback: None,
            is_running: false,
            audio_buffer: Arc::new(Mutex::new(VecDeque::new())),
            // UI integration
            last_emit: Instant::now(),
            transcription_buffer: Vec::new(),
            last_transcription: Instant::now(),
        })
    }

    /// Set the audio data callback
    pub fn set_callback(&mut self, callback: AudioDataCallback) {
        self.callback = Some(callback);
    }

    /// Start audio streaming
    pub fn start(&mut self) -> AudioCaptureResult<()> {
        if self.is_running {
            return Ok(());
        }

        println!("[AudioStreamer] Starting audio streaming from device {}", self.device_id);

        // Create the IO procedure
        let io_proc_id = unsafe {
            let mut proc_id: AudioDeviceIOProcID = std::ptr::null_mut();
            let status = AudioDeviceCreateIOProcID(
                self.device_id,
                Self::audio_io_proc,
                self as *mut _ as *mut std::ffi::c_void,
                &mut proc_id,
            );
            
            if status != NO_ERR {
                return Err(crate::types::AudioCaptureError::CoreAudioError(
                    format!("Failed to create IO procedure: {}", status)
                ));
            }
            proc_id
        };

        self.io_proc_id = Some(io_proc_id);

        // Start the device
        let status = unsafe {
            AudioDeviceStart(self.device_id, io_proc_id)
        };

        if status != NO_ERR {
            return Err(crate::types::AudioCaptureError::CoreAudioError(
                format!("Failed to start audio device: {}", status)
            ));
        }

        self.is_running = true;
        println!("[AudioStreamer] Audio streaming started successfully");
        Ok(())
    }

    /// Stop audio streaming
    pub fn stop(&mut self) -> AudioCaptureResult<()> {
        if !self.is_running {
            return Ok(());
        }

        println!("[AudioStreamer] Stopping audio streaming");

        if let Some(io_proc_id) = self.io_proc_id {
            unsafe {
                AudioDeviceStop(self.device_id, io_proc_id);
                AudioDeviceDestroyIOProcID(self.device_id, io_proc_id);
            }
        }

        self.io_proc_id = None;
        self.is_running = false;
        println!("[AudioStreamer] Audio streaming stopped");
        Ok(())
    }

    /// Check if streaming is active
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Get the latest audio buffer
    pub fn get_latest_buffer(&self) -> Option<AudioBuffer> {
        self.audio_buffer.lock().ok()?.pop_back()
    }

    /// Get all available audio buffers
    pub fn get_all_buffers(&self) -> Vec<AudioBuffer> {
        self.audio_buffer.lock()
            .map(|mut buffer| buffer.drain(..).collect())
            .unwrap_or_default()
    }

    /// Process audio data to match UI expectations (PCM16, 16kHz)
    fn process_audio_for_ui(&self, audio_data: &[f32], input_sample_rate: f64) -> Vec<u8> {
        // Convert float samples to PCM16
        let pcm16_samples: Vec<i16> = audio_data.iter()
            .map(|&sample| (sample * 32767.0).clamp(-32768.0, 32767.0) as i16)
            .collect();
        
        // Convert to bytes (little-endian)
        pcm16_samples.iter()
            .flat_map(|&sample| sample.to_le_bytes())
            .collect()
    }

    /// Calculate audio level in dB
    fn calculate_audio_level(&self, audio_data: &[f32]) -> f32 {
        if audio_data.is_empty() {
            return -60.0;
        }
        
        let rms = (audio_data.iter().map(|&x| x * x).sum::<f32>() / audio_data.len() as f32).sqrt();
        
        if rms > 0.0 {
            20.0 * rms.log10().max(-60.0)
        } else {
            -60.0
        }
    }

    /// Audio IO procedure callback
    extern "C" fn audio_io_proc(
        _device_id: AudioObjectID,
        _in_now: *const AudioTimeStamp,
        in_input_data: *const AudioBufferList,
        _in_input_time: *const AudioTimeStamp,
        _out_output_data: *mut AudioBufferList,
        _in_output_time: *const AudioTimeStamp,
        in_client_data: *mut std::ffi::c_void,
    ) -> OSStatus {
        let streamer = unsafe { &mut *(in_client_data as *mut AudioStreamer) };
        
        if in_input_data.is_null() {
            return NO_ERR;
        }

        let input_data = unsafe { &*in_input_data };
        
        // Process each input buffer
        for i in 0..input_data.number_buffers {
            let buffer = unsafe { &input_data.buffers[i as usize] };
            
            if buffer.data.is_null() || buffer.data_byte_size == 0 {
                continue;
            }

            // Convert audio data to float samples
            let samples = unsafe {
                std::slice::from_raw_parts(
                    buffer.data as *const f32,
                    buffer.data_byte_size as usize / std::mem::size_of::<f32>()
                )
            };

            // Create audio buffer
            let audio_buffer = AudioBuffer {
                data: samples.to_vec(),
                channels: buffer.number_channels,
                sample_rate: streamer.config.sample_rate,
                timestamp: std::time::Instant::now(),
            };

            // Store in buffer queue
            if let Ok(mut buffer_queue) = streamer.audio_buffer.lock() {
                buffer_queue.push_back(audio_buffer.clone());
                
                // Keep only the last 10 buffers to prevent memory growth
                while buffer_queue.len() > 10 {
                    buffer_queue.pop_front();
                }
            }

            // Call the callback if set
            if let Some(callback) = &streamer.callback {
                callback(&audio_buffer);
            }

            // UI Integration: Process audio for transcription
            let now = Instant::now();
            
            // Add to transcription buffer (matching UI expectations)
            streamer.transcription_buffer.extend_from_slice(&audio_buffer.data);
            
            // Keep transcription buffer manageable (4 seconds at 48kHz = 192k samples)
            let max_buffer_size = (streamer.config.sample_rate * 4.0) as usize;
            if streamer.transcription_buffer.len() > max_buffer_size {
                let excess = streamer.transcription_buffer.len() - max_buffer_size;
                streamer.transcription_buffer.drain(0..excess);
            }
            
            // Emit audio chunks periodically (every 100ms)
            if now.duration_since(streamer.last_emit) > Duration::from_millis(100) {
                let pcm16_bytes = streamer.process_audio_for_ui(&audio_buffer.data, audio_buffer.sample_rate);
                let level = streamer.calculate_audio_level(&audio_buffer.data);
                
                // Note: In a real implementation, we would emit events here
                // For now, we'll just log the audio data
                println!("[AudioStreamer] Audio chunk: {} bytes, level: {:.1}dB, {} samples", 
                    pcm16_bytes.len(), level, audio_buffer.data.len());
                
                streamer.last_emit = now;
            }

            // Debug logging (every 100th buffer to avoid spam)
            static mut COUNTER: u32 = 0;
            unsafe {
                COUNTER += 1;
                if COUNTER % 100 == 0 {
                    println!("[AudioStreamer] Processed {} audio buffers, latest: {} samples, {} channels", 
                        COUNTER, samples.len(), buffer.number_channels);
                }
            }
        }

        NO_ERR
    }
}

impl Drop for AudioStreamer {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// Factory functions for audio streaming
pub mod factory {
    use super::*;

    /// Create an audio streamer for an aggregate device
    pub fn create_aggregate_device_streamer(device_id: AudioObjectID) -> AudioCaptureResult<AudioStreamer> {
        AudioStreamer::new(device_id)
    }

    /// Create an audio streamer with a callback
    pub fn create_streamer_with_callback<F>(
        device_id: AudioObjectID,
        callback: F,
    ) -> AudioCaptureResult<AudioStreamer>
    where
        F: Fn(&AudioBuffer) + Send + Sync + 'static,
    {
        let mut streamer = AudioStreamer::new(device_id)?;
        streamer.set_callback(Box::new(callback));
        Ok(streamer)
    }
}
