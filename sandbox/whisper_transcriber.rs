// src/whisper_transcriber.rs
use std::error::Error;
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;
use chrono::{DateTime, Local};
use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};

// Configuration matching your Python implementation
const WHISPER_SAMPLE_RATE: u32 = 16000;
const BUFFER_DURATION: f32 = 4.0;       // Reduced for faster processing
const OVERLAP_DURATION: f32 = 1.0;      // Reduced overlap
const MIN_AUDIO_LENGTH: f32 = 1.5;      // Shorter minimum length
const MIN_CONFIDENCE: f32 = 0.35;       // Slightly lower for speed
const PROCESSING_INTERVAL: f32 = 0.8;   // Faster processing trigger

pub struct RealtimeWhisperTranscriber {
    whisper_ctx: WhisperContext,
    audio_buffer: VecDeque<f32>,
    buffer_size: usize,
    overlap_size: usize,
    min_audio_samples: usize,
    last_transcription_time: Instant,
    
    // Performance tracking
    total_transcriptions: u32,
    total_processing_time: f32,
    processing_times: VecDeque<f32>,
    
    // Logging
    log_file: Option<File>,
}

impl RealtimeWhisperTranscriber {
    pub fn new(model_path: &str) -> Result<Self, Box<dyn Error>> {
        println!("🤖 Loading Whisper model: {}", model_path);
        println!("⚡ Optimizing for real-time performance...");
        
        // Load Whisper model with speed optimizations
        let whisper_ctx = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
        .map_err(|e| format!("Failed to load Whisper model: {}", e))?;
        
        println!("✅ Whisper model loaded successfully");
        
        // Calculate buffer sizes
        let buffer_size = (BUFFER_DURATION * WHISPER_SAMPLE_RATE as f32) as usize;
        let overlap_size = (OVERLAP_DURATION * WHISPER_SAMPLE_RATE as f32) as usize;
        let min_audio_samples = (MIN_AUDIO_LENGTH * WHISPER_SAMPLE_RATE as f32) as usize;
        
        println!("🔧 Real-time buffer configuration:");
        println!("   Buffer duration: {:.1}s ({} samples)", BUFFER_DURATION, buffer_size);
        println!("   Overlap duration: {:.1}s ({} samples)", OVERLAP_DURATION, overlap_size);
        println!("   Minimum audio length: {:.1}s ({} samples)", MIN_AUDIO_LENGTH, min_audio_samples);
        println!("   Processing interval: {:.1}s", PROCESSING_INTERVAL);
        
        // Setup logging
        let log_file = match File::create("realtime_transcription_log.txt") {
            Ok(mut file) => {
                let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
                writeln!(file, "=== REAL-TIME Whisper Transcription Log Started: {} ===\n", timestamp)?;
                println!("✅ Logging to: realtime_transcription_log.txt");
                Some(file)
            },
            Err(e) => {
                println!("⚠️ Could not create log file: {}", e);
                None
            }
        };
        
        Ok(Self {
            whisper_ctx,
            audio_buffer: VecDeque::with_capacity(buffer_size * 2),
            buffer_size,
            overlap_size,
            min_audio_samples,
            last_transcription_time: Instant::now(),
            total_transcriptions: 0,
            total_processing_time: 0.0,
            processing_times: VecDeque::with_capacity(10),
            log_file,
        })
    }
    
    pub fn add_audio_chunk(&mut self, audio_data: &[f32]) {
        // Add new audio data to buffer
        self.audio_buffer.extend(audio_data.iter());
        
        // Keep buffer size manageable
        while self.audio_buffer.len() > self.buffer_size * 2 {
            self.audio_buffer.pop_front();
        }
    }
    
    pub fn should_process(&self) -> bool {
        // Check if we have enough audio and enough time has passed
        self.audio_buffer.len() >= self.buffer_size &&
        self.last_transcription_time.elapsed().as_secs_f32() > PROCESSING_INTERVAL
    }
    
    pub fn process_audio(&mut self) -> Result<Option<String>, Box<dyn Error>> {
        if !self.should_process() {
            return Ok(None);
        }
        
        // Extract audio data for processing
        let audio_data: Vec<f32> = self.audio_buffer.iter().copied().collect();
        
        if audio_data.len() < self.min_audio_samples {
            return Ok(None);
        }
        
        // Use smaller chunk for speed (matching Python implementation)
        let max_samples = std::cmp::min(audio_data.len(), self.buffer_size);
        let audio_chunk = &audio_data[audio_data.len() - max_samples..];
        
        // Quick quality check (RMS level)
        let rms = (audio_chunk.iter().map(|&x| x * x).sum::<f32>() / audio_chunk.len() as f32).sqrt();
        if rms < 0.003 {  // Equivalent to ~100/32768 from Python
            return Ok(None);
        }
        
        let audio_duration = audio_chunk.len() as f32 / WHISPER_SAMPLE_RATE as f32;
        
        println!("🎤 Processing {:.1}s of audio ({} samples)", audio_duration, audio_chunk.len());
        
        // SPEED-OPTIMIZED Whisper processing
        let start_time = Instant::now();
        
        // Create params optimized for speed (matching Python settings)
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        
        // Speed optimizations matching Python implementation
        params.set_n_threads(6);                    // MORE threads for tiny model
        params.set_translate(false);                // No translation
        params.set_language(None);                  // Auto-detect language
        params.set_print_special(false);            // No special tokens
        params.set_print_progress(false);           // No progress output
        params.set_print_realtime(false);           // No realtime printing
        params.set_print_timestamps(false);         // No timestamps for speed
        params.set_suppress_blank(true);            // Suppress blank outputs
        params.set_suppress_non_speech_tokens(true); // Suppress non-speech
        params.set_temperature(0.0);                // No randomness for consistency
        params.set_max_len(0);                      // No length limit
        params.set_token_timestamps(false);         // No token timestamps for speed
        
        // Process audio through Whisper
        let mut state = self.whisper_ctx.create_state()
            .map_err(|e| format!("Failed to create Whisper state: {}", e))?;
        
        state.full(params, audio_chunk)
            .map_err(|e| format!("Whisper processing failed: {}", e))?;
        
        let processing_time = start_time.elapsed().as_secs_f32();
        let real_time_factor = processing_time / audio_duration;
        
        // Track performance
        self.processing_times.push_back(processing_time);
        if self.processing_times.len() > 10 {
            self.processing_times.pop_front();
        }
        self.total_processing_time += processing_time;
        
        // Extract transcription
        let num_segments = state.full_n_segments()
            .map_err(|e| format!("Failed to get segment count: {}", e))?;
        
        let mut transcription_parts = Vec::new();
        let mut total_confidence = 0.0;
        let mut confidence_count = 0;
        
        for i in 0..num_segments {
            let segment = state.full_get_segment_text(i)
                .map_err(|e| format!("Failed to get segment text: {}", e))?;
            
            let text = segment.trim();
            if !text.is_empty() && text.len() > 1 {
                transcription_parts.push(text.to_string());
                
                // Try to get confidence (if available)
                // Note: whisper-rs might not expose confidence directly
                // This is a simplified approach
                confidence_count += 1;
            }
        }
        
        if transcription_parts.is_empty() {
            self.update_last_transcription_time();
            return Ok(None);
        }
        
        let full_text = transcription_parts.join(" ");
        
        // Calculate approximate confidence (simplified)
        let confidence = if confidence_count > 0 {
            // Since whisper-rs doesn't easily expose confidence, we'll estimate based on text quality
            self.estimate_confidence(&full_text)
        } else {
            None
        };
        
        // Quality filtering (matching Python implementation)
        if !self.is_quality_ok(&full_text, confidence) {
            println!("⚡ Filtered low quality (conf: {:.3})", confidence.unwrap_or(0.0));
            self.update_last_transcription_time();
            return Ok(None);
        }
        
        // Log transcription
        self.log_transcription(&full_text, confidence, processing_time, real_time_factor)?;
        
        self.total_transcriptions += 1;
        self.update_last_transcription_time();
        
        // Aggressive buffer management for real-time (matching Python)
        let samples_to_remove = audio_chunk.len() - self.overlap_size;
        for _ in 0..std::cmp::min(samples_to_remove, self.audio_buffer.len()) {
            self.audio_buffer.pop_front();
        }
        
        Ok(Some(full_text))
    }
    
    fn estimate_confidence(&self, text: &str) -> Option<f32> {
        // Simple heuristic for confidence based on text characteristics
        if text.len() < 3 {
            return Some(0.1);
        }
        
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return Some(0.1);
        }
        
        // Check for repetition (lower confidence)
        let unique_words = words.iter().collect::<std::collections::HashSet<_>>();
        let uniqueness_ratio = unique_words.len() as f32 / words.len() as f32;
        
        // Check for common filler words or artifacts
        let filler_count = words.iter()
            .filter(|&&word| {
                let w = word.to_lowercase();
                w == "uh" || w == "um" || w == "ah" || w.len() == 1
            })
            .count();
        
        let filler_ratio = filler_count as f32 / words.len() as f32;
        
        // Estimate confidence
        let base_confidence = 0.7;
        let confidence = base_confidence * uniqueness_ratio * (1.0 - filler_ratio);
        
        Some(confidence.clamp(0.1, 0.95))
    }
    
    fn is_quality_ok(&self, text: &str, confidence: Option<f32>) -> bool {
        if text.len() < 2 {
            return false;
        }
        
        if let Some(conf) = confidence {
            if conf < MIN_CONFIDENCE {
                return false;
            }
        }
        
        // Simple repetition check (matching Python implementation)
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() > 4 {
            let unique_words = words.iter().collect::<std::collections::HashSet<_>>();
            let unique_ratio = unique_words.len() as f32 / words.len() as f32;
            if unique_ratio < 0.3 {
                return false;
            }
        }
        
        true
    }
    
    fn log_transcription(&mut self, text: &str, confidence: Option<f32>, processing_time: f32, real_time_factor: f32) -> Result<(), Box<dyn Error>> {
        let timestamp = Local::now().format("%H:%M:%S%.3f");
        
        // Console output with speed indicator
        let speed_indicator = if real_time_factor < 0.5 {
            "⚡"
        } else if real_time_factor < 1.0 {
            "🚀"
        } else {
            "🎯"
        };
        
        let mut console_parts = vec![format!("{} [{}] {}", speed_indicator, timestamp, text)];
        if let Some(conf) = confidence {
            console_parts.push(format!("({:.3})", conf));
        }
        console_parts.push(format!("({:.2}s)", processing_time));
        console_parts.push(format!("({:.2}x)", real_time_factor));
        
        println!("{}", console_parts.join(" "));
        
        // File logging
        if let Some(ref mut file) = self.log_file {
            let mut log_parts = vec![format!("[{}] TRANSCRIPTION: {}", timestamp, text)];
            if let Some(conf) = confidence {
                log_parts.push(format!("(conf: {:.3})", conf));
            }
            log_parts.push(format!("({:.2}s)", processing_time));
            log_parts.push(format!("(RTF: {:.2}x)", real_time_factor));
            
            writeln!(file, "{}", log_parts.join(" "))?;
            file.flush()?;
        }
        
        Ok(())
    }
    
    fn update_last_transcription_time(&mut self) {
        self.last_transcription_time = Instant::now();
    }
    
    pub fn get_performance_stats(&self) -> (f32, f32, u32) {
        let avg_processing_time = if self.total_transcriptions > 0 {
            self.total_processing_time / self.total_transcriptions as f32
        } else {
            0.0
        };
        
        let avg_rtf = if !self.processing_times.is_empty() {
            let sum: f32 = self.processing_times.iter().sum();
            (sum / self.processing_times.len() as f32) / (BUFFER_DURATION * 0.8)
        } else {
            0.0
        };
        
        (avg_processing_time, avg_rtf, self.total_transcriptions)
    }
}