// Modified simple_audio_capture.rs to include Whisper transcription
use std::error::Error;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::Write;
use wasapi::{initialize_mta, Direction, ShareMode, DeviceCollection};

mod device_enumerator;
mod whisper_transcriber;  // Add this module

use device_enumerator::{WASAPILoopbackEnumerator, LoopbackDevice, DeviceType, LoopbackMethod};
use whisper_transcriber::RealtimeWhisperTranscriber;

const CAPTURE_DURATION_SECONDS: u64 = 300;  // Extended to 5 minutes for transcription testing
const WHISPER_SAMPLE_RATE: u32 = 16000;

struct AudioCaptureWithTranscription {
    device: LoopbackDevice,
    is_running: Arc<Mutex<bool>>,
    transcriber: Option<RealtimeWhisperTranscriber>,
}

impl AudioCaptureWithTranscription {
    fn new(device: LoopbackDevice, model_path: Option<&str>) -> Result<Self, Box<dyn Error>> {
        let transcriber = if let Some(path) = model_path {
            println!("🤖 Initializing Whisper transcription...");
            match RealtimeWhisperTranscriber::new(path) {
                Ok(t) => {
                    println!("✅ Whisper transcriber ready");
                    Some(t)
                }
                Err(e) => {
                    println!("⚠️ Could not initialize Whisper: {}", e);
                    println!("📝 Continuing with audio capture only...");
                    None
                }
            }
        } else {
            println!("📝 No Whisper model specified, audio capture only");
            None
        };
        
        Ok(Self {
            device,
            is_running: Arc::new(Mutex::new(false)),
            transcriber,
        })
    }
    
    // ... (keep your existing process_audio_chunk and calculate_audio_level methods)
    
    fn process_audio_chunk(&self, audio_data: &[f32]) -> Vec<f32> {
        let mut processed = audio_data.to_vec();
        
        // Convert stereo to mono if needed
        if self.device.channels == 2 {
            processed = processed.chunks(2)
                .map(|chunk| {
                    if chunk.len() == 2 {
                        (chunk[0] + chunk[1]) * 0.5
                    } else {
                        chunk[0]
                    }
                })
                .collect();
        }
        
        // Simple resampling for common rates
        if self.device.sample_rate != WHISPER_SAMPLE_RATE {
            if self.device.sample_rate == 48000 && WHISPER_SAMPLE_RATE == 16000 {
                processed = processed.iter().step_by(3).copied().collect();
            } else if self.device.sample_rate == 44100 && WHISPER_SAMPLE_RATE == 16000 {
                let factor = self.device.sample_rate as f32 / WHISPER_SAMPLE_RATE as f32;
                processed = (0..processed.len())
                    .step_by(factor as usize)
                    .map(|i| processed.get(i).copied().unwrap_or(0.0))
                    .collect();
            }
        }
        
        processed
    }
    
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
    
    // ... (keep your existing find_wasapi_device method)
    
    pub fn start_capture_with_transcription(&mut self) -> Result<(), Box<dyn Error>> {
        println!("🎤 Starting audio capture with real-time transcription");
        println!("📊 Sample Rate: {} Hz -> {} Hz", self.device.sample_rate, WHISPER_SAMPLE_RATE);
        println!("🎧 Channels: {}", self.device.channels);
        println!("🔧 Device Type: {:?}", self.device.device_type);
        println!("🔧 Loopback Method: {:?}", self.device.loopback_method);
        
        if self.transcriber.is_some() {
            println!("🤖 Whisper transcription: ENABLED");
            println!("⚡ Target: <1.0x real-time factor");
        } else {
            println!("📝 Whisper transcription: DISABLED (audio capture only)");
        }
        
        println!("⏱️  Duration: {} seconds", CAPTURE_DURATION_SECONDS);
        
        *self.is_running.lock().unwrap() = true;
        
        // Find the WASAPI device
        let wasapi_device = self.find_wasapi_device()
            .map_err(|e| format!("Device lookup failed: {}", e))?;
        
        // Setup audio client (use your existing setup code)
        let mut corrected_audio_client = wasapi_device.get_iaudioclient()
            .map_err(|e| format!("Failed to get corrected audio client: {}", e))?;
        
        let format = corrected_audio_client.get_mixformat()
            .map_err(|e| format!("Failed to get mix format: {}", e))?;
        
        let (_, min_time) = corrected_audio_client.get_periods()
            .map_err(|e| format!("Failed to get periods: {}", e))?;
        
        // Initialize for loopback capture
        corrected_audio_client.initialize_client(
            &format,
            min_time,
            &Direction::Capture,
            &ShareMode::Shared,
            true,
        ).map_err(|e| format!("Corrected initialization failed: {:?}", e))?;
        
        let capture_client = corrected_audio_client.get_audiocaptureclient()
            .map_err(|e| format!("Failed to get corrected capture client: {:?}", e))?;
        
        let h_event = corrected_audio_client.set_get_eventhandle()
            .map_err(|e| format!("Failed to get event handle: {:?}", e))?;
        
        println!("✅ Audio capture initialized successfully!");
        
        // Create output file for raw audio
        let mut raw_file = File::create("captured_audio.raw")
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        corrected_audio_client.start_stream()
            .map_err(|e| format!("Failed to start stream: {:?}", e))?;
        
        println!("\n🎵 CAPTURING AUDIO WITH REAL-TIME TRANSCRIPTION...");
        if self.transcriber.is_some() {
            println!("🤖 Whisper processing will start when sufficient audio is buffered");
            println!("📝 Transcriptions will appear below:");
        }
        println!("⚡ Processing audio in real-time");
        println!("📁 Saving raw audio to: captured_audio.raw");
        println!("Press Ctrl+C to stop early\n");
        
        let start_time = Instant::now();
        let mut total_samples = 0;
        let mut audio_chunks = Vec::new();
        let mut last_level_report = Instant::now();
        let mut error_count = 0;
        let max_errors = 10;
        
        // Setup Ctrl+C handler
        let is_running_clone = Arc::clone(&self.is_running);
        ctrlc::set_handler(move || {
            println!("\n🛑 Ctrl+C received, stopping capture...");
            *is_running_clone.lock().unwrap() = false;
        }).map_err(|e| format!("Failed to set Ctrl+C handler: {}", e))?;
        
        // Main capture loop with transcription
        while *self.is_running.lock().unwrap() {
            if start_time.elapsed().as_secs() >= CAPTURE_DURATION_SECONDS {
                break;
            }
            
            if error_count >= max_errors {
                println!("❌ Too many errors ({}), stopping capture", error_count);
                break;
            }
            
            // Wait for audio data
            match h_event.wait_for_event(100) {
                Ok(_) => {},
                Err(_) => {
                    thread::sleep(Duration::from_millis(1));
                    continue;
                }
            }
            
            // Get available frames
            let frames_available = match capture_client.get_next_nbr_frames() {
                Ok(Some(frames)) => {
                    if frames == 0 {
                        thread::sleep(Duration::from_millis(1));
                        continue;
                    }
                    if frames > 2048 { 2048 } else { frames }
                },
                Ok(None) => {
                    thread::sleep(Duration::from_millis(1));
                    continue;
                }
                Err(_) => {
                    error_count += 1;
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
            };
            
            // Read audio data (use your existing working code)
            let bits_per_sample = format.get_bitspersample();
            let channels = format.get_nchannels();
            let bytes_per_sample = bits_per_sample / 8;
            let bytes_per_frame = bytes_per_sample * channels as u16;
            
            let safe_buffer_size = std::cmp::max(
                frames_available as usize * bytes_per_frame as usize,
                4096
            );
            let mut buffer = vec![0u8; safe_buffer_size];
            
            let (frames_read, _flags) = match capture_client.read_from_device(bytes_per_frame as usize, &mut buffer) {
                Ok((frames, flags)) => {
                    if error_count > 0 {
                        error_count = std::cmp::max(0, error_count - 1);
                    }
                    (frames, flags)
                },
                Err(_) => {
                    error_count += 1;
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
            };
            
            if frames_read == 0 {
                continue;
            }
            
            // Convert to f32 samples
            let actual_bytes = frames_read as usize * bytes_per_frame as usize;
            let audio_data = if actual_bytes <= buffer.len() {
                &buffer[..actual_bytes]
            } else {
                error_count += 1;
                continue;
            };
            
            let mut f32_samples = Vec::new();
            match bits_per_sample {
                32 => {
                    for chunk in audio_data.chunks_exact(4) {
                        if chunk.len() == 4 {
                            let bytes = [chunk[0], chunk[1], chunk[2], chunk[3]];
                            let sample = f32::from_le_bytes(bytes);
                            if sample.is_finite() && sample.abs() <= 2.0 {
                                f32_samples.push(sample);
                            } else {
                                f32_samples.push(0.0);
                            }
                        }
                    }
                },
                16 => {
                    for chunk in audio_data.chunks_exact(2) {
                        if chunk.len() == 2 {
                            let bytes = [chunk[0], chunk[1]];
                            let sample_i16 = i16::from_le_bytes(bytes);
                            let sample_f32 = sample_i16 as f32 / 32768.0;
                            f32_samples.push(sample_f32);
                        }
                    }
                },
                _ => {
                    error_count += 1;
                    continue;
                }
            }
            
            if f32_samples.is_empty() {
                continue;
            }
            
            // Process audio
            let actual_device = LoopbackDevice {
                id: self.device.id.clone(),
                name: self.device.name.clone(),
                is_default: self.device.is_default,
                sample_rate: format.get_samplespersec(),
                channels: format.get_nchannels(),
                format: format!("{} bit", format.get_bitspersample()),
                device_type: self.device.device_type.clone(),
                loopback_method: self.device.loopback_method.clone(),
            };
            
            // let temp_capture = AudioCaptureWithTranscription::new(actual_device, None)?;
            let processed_audio = self.process_audio_chunk(&f32_samples);
            total_samples += processed_audio.len();
            
            // Add to Whisper transcriber if available
            if let Some(ref mut transcriber) = self.transcriber {
                transcriber.add_audio_chunk(&processed_audio);
                
                // Try to process transcription
                match transcriber.process_audio() {
                    Ok(Some(transcription)) => {
                        // Transcription was successful and logged by the transcriber
                    },
                    Ok(None) => {
                        // No transcription ready yet or filtered out
                    },
                    Err(e) => {
                        println!("⚠️ Transcription error: {}", e);
                    }
                }
            }
            
            // Save to file
            for &sample in &processed_audio {
                let sample_i16 = (sample * 32767.0).max(-32768.0).min(32767.0) as i16;
                if let Err(e) = raw_file.write_all(&sample_i16.to_le_bytes()) {
                    println!("⚠️ File write error: {}", e);
                    break;
                }
            }
            
            audio_chunks.push(processed_audio.clone());
            
            // Periodic reporting
            let now = Instant::now();
            if now.duration_since(last_level_report) > Duration::from_secs(3) {
                let level = self.calculate_audio_level(&processed_audio);
                let elapsed = start_time.elapsed().as_secs();
                
                let status_msg = if let Some(ref transcriber) = self.transcriber {
                    let (avg_time, avg_rtf, total_transcriptions) = transcriber.get_performance_stats();
                    format!("🎵 [{:02}s] Level: {:.1}dB | Samples: {} | Transcriptions: {} | Avg RTF: {:.2}x | Frames: {}", 
                        elapsed, level, total_samples, total_transcriptions, avg_rtf, frames_read)
                } else {
                    format!("🎵 [{:02}s] Level: {:.1}dB | Samples: {} | Chunks: {} | Errors: {} | Frames: {}", 
                        elapsed, level, total_samples, audio_chunks.len(), error_count, frames_read)
                };
                
                println!("{}", status_msg);
                last_level_report = now;
            }
        }
        
        // Cleanup
        match corrected_audio_client.stop_stream() {
            Ok(_) => {},
            Err(_) => println!("⚠️ Error stopping stream"),
        }
        println!("\n⏹️ Audio capture stopped");
        
        // Final statistics
        let total_duration = start_time.elapsed().as_secs_f32();
        let estimated_audio_duration = total_samples as f32 / WHISPER_SAMPLE_RATE as f32;
        
        println!("\n=== CAPTURE STATISTICS ===");
        println!("Capture Duration: {:.1}s", total_duration);
        println!("Audio Duration: {:.1}s", estimated_audio_duration);
        println!("Total Samples: {}", total_samples);
        println!("Total Chunks: {}", audio_chunks.len());
        println!("Total Errors: {}", error_count);
        
        if let Some(ref transcriber) = self.transcriber {
            let (avg_processing_time, avg_rtf, total_transcriptions) = transcriber.get_performance_stats();
            
            println!("\n=== TRANSCRIPTION STATISTICS ===");
            println!("Total Transcriptions: {}", total_transcriptions);
            println!("Average Processing Time: {:.2}s", avg_processing_time);
            println!("Average Real-time Factor: {:.2}x", avg_rtf);
            
            if avg_rtf < 1.0 {
                println!("🚀 ACHIEVED REAL-TIME PERFORMANCE!");
            } else if avg_rtf < 1.5 {
                println!("⚡ NEAR REAL-TIME PERFORMANCE");
            } else {
                println!("🔧 Consider optimizations for better real-time performance");
            }
            
            println!("📝 Transcription log saved to: realtime_transcription_log.txt");
        }
        
        if error_count < max_errors / 2 {
            println!("✅ Capture completed successfully!");
        } else {
            println!("⚠️ Capture completed with some errors");
        }
        
        println!("✅ Audio capture complete! Raw audio saved to: captured_audio.raw");
        println!("💡 To use with external Whisper: convert to WAV format at 16kHz mono");
        
        Ok(())
    }
    
    // Keep your existing find_wasapi_device method here
    fn find_wasapi_device(&self) -> Result<wasapi::Device, Box<dyn Error>> {
        match self.device.device_type {
            DeviceType::Render => {
                let device_collection = DeviceCollection::new(&Direction::Render)
                    .map_err(|e| format!("Failed to create render device collection: {}", e))?;
                let device_count = device_collection.get_nbr_devices()
                    .map_err(|e| format!("Failed to get render device count: {}", e))?;
                
                for i in 0..device_count {
                    if let Ok(device) = device_collection.get_device_at_index(i) {
                        if let Ok(id) = device.get_id() {
                            if id == self.device.id {
                                return Ok(device);
                            }
                        }
                    }
                }
            },
            DeviceType::Capture => {
                let device_collection = DeviceCollection::new(&Direction::Capture)
                    .map_err(|e| format!("Failed to create capture device collection: {}", e))?;
                let device_count = device_collection.get_nbr_devices()
                    .map_err(|e| format!("Failed to get capture device count: {}", e))?;
                
                for i in 0..device_count {
                    if let Ok(device) = device_collection.get_device_at_index(i) {
                        if let Ok(id) = device.get_id() {
                            if id == self.device.id {
                                return Ok(device);
                            }
                        }
                    }
                }
            }
        }
        
        Err(format!("Could not find device with ID: {}", self.device.id).into())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    initialize_mta()
        .map_err(|e| format!("Failed to initialize COM: {}", e))?;
    
    println!("=== WASAPI Audio Capture with Real-time Whisper Transcription ===");
    println!("🚀 Rust implementation matching Python real-time performance\n");
    
    // Parse command line arguments for Whisper model
    let args: Vec<String> = std::env::args().collect();
    let model_path = if args.len() > 1 {
        Some(args[1].as_str())
    } else {
        // Try default model locations
        if std::path::Path::new("models/ggml-tiny.bin").exists() {
            Some("models/ggml-tiny.bin")
        } else if std::path::Path::new("ggml-tiny.bin").exists() {
            Some("ggml-tiny.bin")
        } else {
            println!("⚠️ No Whisper model found. Usage:");
            println!("   cargo run --bin audio_capture [model_path]");
            println!("   cargo run --bin audio_capture models/ggml-tiny.bin");
            println!("   cargo run --bin audio_capture ggml-tiny.bin");
            println!("\n📥 Download Whisper models from:");
            println!("   https://huggingface.co/ggerganov/whisper.cpp/tree/main");
            println!("\n📝 Continuing with audio capture only (no transcription)...\n");
            None
        }
    };
    
    if let Some(path) = model_path {
        println!("🤖 Using Whisper model: {}", path);
    }
    
    // Find and select device using enhanced enumeration
    let enumerator = WASAPILoopbackEnumerator::new()
        .map_err(|e| format!("Failed to create device enumerator: {}", e))?;
    
    let selected_device = match enumerator.auto_select_best_device()? {
        Some(device) => {
            println!("🎯 Selected device: {} [{:?} via {:?}]", 
                device.name, device.device_type, device.loopback_method);
            device
        },
        None => {
            println!("❌ No suitable loopback devices found!");
            println!("💡 Try enabling 'Stereo Mix' in Windows Sound settings");
            println!("💡 Or check if your audio drivers support loopback");
            return Ok(());
        }
    };
    
    // Create capture instance with optional transcription
    let mut capture = AudioCaptureWithTranscription::new(selected_device, model_path)?;
    
    match capture.start_capture_with_transcription() {
        Ok(_) => {
            println!("🎉 Audio capture with transcription completed successfully!");
        },
        Err(e) => {
            println!("❌ Audio capture failed: {}", e);
            println!("\n🔧 Troubleshooting tips:");
            println!("1. Try running as Administrator");
            println!("2. Check Windows Sound settings for enabled devices");
            println!("3. Enable 'Stereo Mix' if available");
            println!("4. Update your audio drivers");
            println!("5. For Whisper issues, verify model file path and format");
            return Err(e);
        }
    }
    
    Ok(())
}