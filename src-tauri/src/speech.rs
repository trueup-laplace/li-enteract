use std::process::{Command, Stdio, Child};
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};
use serde_json;

// Whisper-rs imports for wake word detection
use std::path::PathBuf;
use std::fs;
use base64::{Engine as _, engine::general_purpose};
use tempfile::NamedTempFile;
use anyhow::Result;
use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub chunk_size: usize,
    pub silence_threshold: f32,
    pub silence_duration: f32,
    pub max_recording_duration: f32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            chunk_size: 1024,
            silence_threshold: 0.01,
            silence_duration: 2.0,
            max_recording_duration: 30.0,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WakeWordDetection {
    pub confidence: f32,
    pub timestamp: u64,
    pub audio_snippet: Vec<f32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpeechTranscription {
    pub text: String,
    pub confidence: f32,
    pub duration: f32,
    pub timestamp: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SpeechState {
    pub is_listening: bool,
    pub is_recording: bool,
    pub wake_word_detected: bool,
    pub last_detection: Option<WakeWordDetection>,
    pub last_transcription: Option<SpeechTranscription>,
    pub total_detections: u32,
}

impl Default for SpeechState {
    fn default() -> Self {
        Self {
            is_listening: false,
            is_recording: false,
            wake_word_detected: false,
            last_detection: None,
            last_transcription: None,
            total_detections: 0,
        }
    }
}

// Whisper-rs structures for compatibility
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WhisperModelConfig {
    pub model_size: String,
    pub language: Option<String>,
    pub enable_vad: bool,
    pub silence_threshold: f32,
    pub max_segment_length: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub confidence: f32,
    pub start_time: f32,
    pub end_time: f32,
    pub language: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WakeWordState {
    pub is_active: bool,
    pub is_listening: bool,
    pub last_detection: Option<WakeWordDetectionInfo>,
    pub total_detections: u32,
    pub whisper_activated: bool,
}

impl Default for WakeWordState {
    fn default() -> Self {
        Self {
            is_active: false,
            is_listening: false,
            last_detection: None,
            total_detections: 0,
            whisper_activated: false,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WakeWordDetectionInfo {
    pub confidence: f32,
    pub timestamp: u64,
    pub audio_length: usize,
}

pub struct SpeechManager {
    process: Option<Child>,
    config: AudioConfig,
    wake_word_receiver: Option<mpsc::UnboundedReceiver<WakeWordDetection>>,
    transcription_receiver: Option<mpsc::UnboundedReceiver<SpeechTranscription>>,
    state: SpeechState,
}

impl SpeechManager {
    pub fn new(config: AudioConfig) -> Self {
        Self {
            process: None,
            config,
            wake_word_receiver: None,
            transcription_receiver: None,
            state: SpeechState::default(),
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        // Create Python script
        let script_content = self.create_python_script();
        let script_path = std::env::temp_dir().join("aubrey_speech_detector.py");
        std::fs::write(&script_path, script_content)
            .map_err(|e| format!("Failed to write Python script: {}", e))?;

        // Find Python executable
        let python_cmd = if cfg!(target_os = "windows") {
            if Command::new("python").arg("--version").output().is_ok() {
                "python"
            } else if Command::new("python3").arg("--version").output().is_ok() {
                "python3"
            } else {
                return Err("Python not found. Please install Python 3.8+ and add it to PATH".to_string());
            }
        } else {
            if Command::new("python3").arg("--version").output().is_ok() {
                "python3"
            } else if Command::new("python").arg("--version").output().is_ok() {
                "python"
            } else {
                return Err("Python not found. Please install Python 3.8+ and add it to PATH".to_string());
            }
        };

        // Start Python process
        let mut cmd = Command::new(python_cmd);
        cmd.arg(&script_path)
           .arg("--sample-rate").arg(self.config.sample_rate.to_string())
           .arg("--silence-threshold").arg(self.config.silence_threshold.to_string())
           .arg("--silence-duration").arg(self.config.silence_duration.to_string())
           .arg("--max-duration").arg(self.config.max_recording_duration.to_string());

        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start Python audio process: {}", e))?;

        println!("🎤 Started always-on audio monitoring for 'Aubrey'");

        // Create channels
        let (wake_tx, wake_rx) = mpsc::unbounded_channel();
        let (trans_tx, trans_rx) = mpsc::unbounded_channel();

        // Spawn stdout reader thread
        if let Some(stdout) = child.stdout.take() {
            std::thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }

                        if trimmed.starts_with("WAKE_WORD:") {
                            if let Ok(detection) = serde_json::from_str::<WakeWordDetection>(&trimmed[10..]) {
                                if wake_tx.send(detection).is_err() {
                                    break;
                                }
                            }
                        } else if trimmed.starts_with("TRANSCRIPTION:") {
                            if let Ok(transcription) = serde_json::from_str::<SpeechTranscription>(&trimmed[14..]) {
                                if trans_tx.send(transcription).is_err() {
                                    break;
                                }
                            }
                        } else {
                            println!("Audio Debug: {}", trimmed);
                        }
                    }
                }
            });
        }

        // Spawn stderr reader thread
        if let Some(stderr) = child.stderr.take() {
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        println!("Audio Error: {}", line);
                    }
                }
            });
        }

        self.process = Some(child);
        self.wake_word_receiver = Some(wake_rx);
        self.transcription_receiver = Some(trans_rx);
        self.state.is_listening = true;

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(mut process) = self.process.take() {
            process.kill().map_err(|e| format!("Failed to kill audio process: {}", e))?;
            process.wait().map_err(|e| format!("Failed to wait for audio process: {}", e))?;
        }
        
        self.wake_word_receiver = None;
        self.transcription_receiver = None;
        self.state = SpeechState::default();
        
        println!("🎤 Stopped audio monitoring");
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.process.is_some()
    }

    pub fn get_state(&self) -> &SpeechState {
        &self.state
    }

    pub fn check_wake_word(&mut self) -> Option<WakeWordDetection> {
        if let Some(receiver) = &mut self.wake_word_receiver {
            if let Ok(detection) = receiver.try_recv() {
                self.state.wake_word_detected = true;
                self.state.is_recording = true;
                self.state.total_detections += 1;
                self.state.last_detection = Some(detection.clone());
                println!("🎤 Wake word 'Aubrey' detected! Starting recording...");
                return Some(detection);
            }
        }
        None
    }

    pub fn check_transcription(&mut self) -> Option<SpeechTranscription> {
        if let Some(receiver) = &mut self.transcription_receiver {
            if let Ok(transcription) = receiver.try_recv() {
                self.state.is_recording = false;
                self.state.wake_word_detected = false;
                self.state.last_transcription = Some(transcription.clone());
                println!("🎤 Transcription complete: '{}'", transcription.text);
                return Some(transcription);
            }
        }
        None
    }

    fn create_python_script(&self) -> String {
        r#"#!/usr/bin/env python3
import pyaudio
import numpy as np
import argparse
import sys
import json
import time
from collections import deque

class AubreyDetector:
    def __init__(self, sample_rate, silence_threshold, silence_duration, max_duration):
        self.sample_rate = sample_rate
        self.silence_threshold = silence_threshold
        self.silence_duration = silence_duration
        self.max_duration = max_duration
        
        # Audio setup
        self.chunk_size = 1024
        self.format = pyaudio.paFloat32
        self.channels = 1
        
        # State
        self.is_recording = False
        self.audio_buffer = deque(maxlen=int(sample_rate * 3))
        self.recording_buffer = []
        self.silence_start = None
        
        # Initialize PyAudio
        self.audio = pyaudio.PyAudio()
        
        # Open microphone stream
        self.stream = self.audio.open(
            format=self.format,
            channels=self.channels,
            rate=self.sample_rate,
            input=True,
            frames_per_buffer=self.chunk_size,
            stream_callback=self.audio_callback
        )
        
    def audio_callback(self, in_data, frame_count, time_info, status):
        audio_data = np.frombuffer(in_data, dtype=np.float32)
        
        # Add to circular buffer for wake word detection
        self.audio_buffer.extend(audio_data)
        
        # Check for wake word in buffer
        if not self.is_recording and len(self.audio_buffer) >= self.sample_rate:
            self.check_wake_word()
            
        # If recording, add to recording buffer
        if self.is_recording:
            self.recording_buffer.extend(audio_data)
            self.check_silence(audio_data)
            
            # Check max duration
            if len(self.recording_buffer) > self.sample_rate * self.max_duration:
                self.finish_recording()
                
        return (None, pyaudio.paContinue)
    
    def check_wake_word(self):
        """Basic wake word detection using audio features"""
        audio_chunk = np.array(list(self.audio_buffer)[-self.sample_rate:])
        
        # Simple energy-based detection
        energy = np.mean(np.abs(audio_chunk))
        
        # Basic voice activity detection
        if energy > 0.02:
            spectral_centroid = self.calculate_spectral_centroid(audio_chunk)
            
            # Voice frequency range check
            if 800 < spectral_centroid < 2000:
                confidence = min(energy * 10, 1.0)
                
                detection = {
                    "confidence": confidence,
                    "timestamp": int(time.time() * 1000),
                    "audio_snippet": audio_chunk[-1600:].tolist()
                }
                
                print(f"WAKE_WORD:{json.dumps(detection)}")
                sys.stdout.flush()
                
                self.start_recording()
    
    def calculate_spectral_centroid(self, audio_data):
        """Calculate spectral centroid for voice detection"""
        fft = np.fft.fft(audio_data)
        magnitude = np.abs(fft)
        length = len(magnitude)
        freqs = np.fft.fftfreq(length, 1/self.sample_rate)
        
        magnitude = magnitude[:length//2]
        freqs = freqs[:length//2]
        
        if np.sum(magnitude) == 0:
            return 0
            
        return np.sum(freqs * magnitude) / np.sum(magnitude)
    
    def start_recording(self):
        """Start recording after wake word detection"""
        self.is_recording = True
        self.recording_buffer = []
        self.silence_start = None
        print("Started recording after wake word", file=sys.stderr)
    
    def check_silence(self, audio_data):
        """Check for silence to end recording"""
        energy = np.mean(np.abs(audio_data))
        
        if energy < self.silence_threshold:
            if self.silence_start is None:
                self.silence_start = time.time()
            elif time.time() - self.silence_start > self.silence_duration:
                self.finish_recording()
        else:
            self.silence_start = None
    
    def finish_recording(self):
        """Process and transcribe the recorded audio"""
        if not self.recording_buffer:
            return
            
        print("Finishing recording, transcribing...", file=sys.stderr)
        
        # For now, return a mock transcription
        # In a real implementation, you'd use Whisper or another ASR
        transcription = {
            "text": "Mock transcription - implement Whisper here",
            "confidence": 0.8,
            "duration": len(self.recording_buffer) / self.sample_rate,
            "timestamp": int(time.time() * 1000)
        }
        
        print(f"TRANSCRIPTION:{json.dumps(transcription)}")
        sys.stdout.flush()
        
        # Reset state
        self.is_recording = False
        self.recording_buffer = []
        self.silence_start = None
    
    def run(self):
        """Main loop"""
        print("🎤 Listening for 'Aubrey'...", file=sys.stderr)
        self.stream.start_stream()
        
        try:
            while True:
                time.sleep(0.1)
        except KeyboardInterrupt:
            print("Stopping audio monitoring", file=sys.stderr)
        finally:
            self.cleanup()
    
    def cleanup(self):
        """Clean up resources"""
        if hasattr(self, 'stream'):
            self.stream.stop_stream()
            self.stream.close()
        if hasattr(self, 'audio'):
            self.audio.terminate()

def main():
    parser = argparse.ArgumentParser(description='Always-on Aubrey wake word detection')
    parser.add_argument('--sample-rate', type=int, default=16000)
    parser.add_argument('--silence-threshold', type=float, default=0.01)
    parser.add_argument('--silence-duration', type=float, default=2.0)
    parser.add_argument('--max-duration', type=float, default=30.0)
    
    args = parser.parse_args()
    
    detector = AubreyDetector(
        args.sample_rate,
        args.silence_threshold,
        args.silence_duration,
        args.max_duration
    )
    
    detector.run()

if __name__ == "__main__":
    main()
"#.to_string()
    }
}

// Global speech manager and whisper state
lazy_static::lazy_static! {
    static ref SPEECH_MANAGER: Arc<Mutex<Option<SpeechManager>>> = Arc::new(Mutex::new(None));
    static ref WHISPER_CONTEXT: Arc<Mutex<Option<WhisperContext>>> = Arc::new(Mutex::new(None));
    static ref MODEL_CACHE_DIR: PathBuf = {
        let mut cache_dir = std::env::temp_dir();
        cache_dir.push("enteract");
        cache_dir.push("whisper_models");
        cache_dir
    };
    static ref WAKE_WORD_STATE: Arc<Mutex<WakeWordState>> = Arc::new(Mutex::new(WakeWordState::default()));
}

// Always-on speech commands
#[tauri::command]
pub async fn start_always_on_speech() -> Result<String, String> {
    let mut manager = SPEECH_MANAGER.lock().unwrap();
    
    if let Some(existing) = manager.as_mut() {
        existing.stop()?;
    }
    
    let config = AudioConfig::default();
    let mut new_manager = SpeechManager::new(config);
    new_manager.start()?;
    
    *manager = Some(new_manager);
    
    Ok("Always-on speech detection started - listening for 'Aubrey'".to_string())
}

#[tauri::command]
pub async fn stop_always_on_speech() -> Result<String, String> {
    let mut manager = SPEECH_MANAGER.lock().unwrap();
    
    if let Some(existing) = manager.as_mut() {
        existing.stop()?;
        *manager = None;
        Ok("Always-on speech detection stopped".to_string())
    } else {
        Err("Speech detection not running".to_string())
    }
}

#[tauri::command]
pub async fn get_speech_state() -> Result<SpeechState, String> {
    let manager = SPEECH_MANAGER.lock().unwrap();
    
    if let Some(ref manager_instance) = *manager {
        Ok(manager_instance.get_state().clone())
    } else {
        Ok(SpeechState::default())
    }
}

#[tauri::command]
pub async fn check_for_wake_word() -> Result<Option<WakeWordDetection>, String> {
    let mut manager = SPEECH_MANAGER.lock().unwrap();
    
    if let Some(ref mut manager_instance) = manager.as_mut() {
        Ok(manager_instance.check_wake_word())
    } else {
        Err("Speech detection not running".to_string())
    }
}

#[tauri::command]
pub async fn check_for_transcription() -> Result<Option<SpeechTranscription>, String> {
    let mut manager = SPEECH_MANAGER.lock().unwrap();
    
    if let Some(ref mut manager_instance) = manager.as_mut() {
        Ok(manager_instance.check_transcription())
    } else {
        Err("Speech detection not running".to_string())
    }
}

// Whisper-rs compatibility commands for frontend
#[tauri::command]
pub async fn initialize_whisper_model(config: WhisperModelConfig) -> Result<String, String> {
    let model_path = get_or_download_model(&config.model_size).await?;
    
    let ctx = WhisperContext::new_with_params(
        model_path.to_str().ok_or("Invalid model path")?,
        WhisperContextParameters::default()
    ).map_err(|e| format!("Failed to initialize Whisper context: {}", e))?;
    
    let mut whisper_ctx = WHISPER_CONTEXT.lock().unwrap();
    *whisper_ctx = Some(ctx);
    
    Ok(format!("Whisper model '{}' initialized successfully", config.model_size))
}

#[tauri::command]
pub async fn transcribe_audio_base64(audio_data: String, config: WhisperModelConfig) -> Result<TranscriptionResult, String> {
    // Decode base64 audio data
    let audio_bytes = general_purpose::STANDARD
        .decode(&audio_data)
        .map_err(|e| format!("Failed to decode base64 audio: {}", e))?;
    
    // Create temporary file for audio
    let temp_file = NamedTempFile::with_suffix(".wav")
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    
    fs::write(temp_file.path(), audio_bytes)
        .map_err(|e| format!("Failed to write audio to temp file: {}", e))?;
    
    transcribe_audio_file(temp_file.path().to_string_lossy().to_string(), config).await
}

#[tauri::command]
pub async fn transcribe_audio_file(file_path: String, config: WhisperModelConfig) -> Result<TranscriptionResult, String> {
    // Ensure model is initialized
    let needs_init = {
        let whisper_ctx = WHISPER_CONTEXT.lock().unwrap();
        whisper_ctx.is_none()
    };
    
    if needs_init {
        initialize_whisper_model(config.clone()).await?;
    }
    
    // Load and preprocess audio
    let audio_data = load_audio_file(&file_path)?;
    
    // Get Whisper context
    let whisper_ctx = WHISPER_CONTEXT.lock().unwrap();
    let ctx = whisper_ctx.as_ref().ok_or("Whisper context not initialized")?;
    
    // Set up transcription parameters
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    
    if let Some(ref lang) = config.language {
        params.set_language(Some(lang));
    } else {
        params.set_language(Some("auto"));
    }
    
    params.set_translate(false);
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    
    // Run transcription
    let mut state = ctx.create_state().map_err(|e| format!("Failed to create state: {}", e))?;
    state.full(params, &audio_data)
        .map_err(|e| format!("Transcription failed: {}", e))?;
    
    // Extract results
    let num_segments = state.full_n_segments()
        .map_err(|e| format!("Failed to get segment count: {}", e))?;
    
    let mut full_text = String::new();
    let mut total_confidence = 0.0;
    let mut start_time: f32 = f32::MAX;
    let mut end_time: f32 = 0.0;
    
    for i in 0..num_segments {
        let segment_text = state.full_get_segment_text(i)
            .map_err(|e| format!("Failed to get segment text: {}", e))?;
        
        let segment_start = state.full_get_segment_t0(i)
            .map_err(|e| format!("Failed to get segment start time: {}", e))? as f32 / 100.0;
        
        let segment_end = state.full_get_segment_t1(i)
            .map_err(|e| format!("Failed to get segment end time: {}", e))? as f32 / 100.0;
        
        full_text.push_str(&segment_text);
        start_time = start_time.min(segment_start);
        end_time = end_time.max(segment_end);
        total_confidence += 1.0; // Whisper doesn't provide confidence scores directly
    }
    
    let avg_confidence = if num_segments > 0 { total_confidence / num_segments as f32 } else { 0.0 };
    
    Ok(TranscriptionResult {
        text: full_text.trim().to_string(),
        confidence: avg_confidence,
        start_time,
        end_time,
        language: config.language,
    })
}

#[tauri::command]
pub async fn check_whisper_model_availability(model_size: String) -> Result<bool, String> {
    let model_path = get_model_path(&model_size);
    Ok(model_path.exists())
}

#[tauri::command]
pub async fn download_whisper_model(model_size: String) -> Result<String, String> {
    let model_path = get_model_path(&model_size);
    if model_path.exists() {
        fs::remove_file(&model_path)
            .map_err(|e| format!("Failed to remove existing model: {}", e))?;
    }
    
    get_or_download_model(&model_size).await?;
    Ok(format!("Model '{}' downloaded successfully", model_size))
}

#[tauri::command]
pub async fn list_available_models() -> Result<Vec<String>, String> {
    Ok(vec![
        "tiny".to_string(),
        "base".to_string(),
        "small".to_string(),
        "medium".to_string(),
        "large".to_string(),
    ])
}

// Wake word detection commands for compatibility
#[tauri::command]
pub async fn start_wake_word_detection() -> Result<String, String> {
    let mut state = WAKE_WORD_STATE.lock().unwrap();
    state.is_active = true;
    state.is_listening = true;
    
    println!("Wake word detection started (compatibility mode)");
    Ok("Wake word detection started for 'Aubrey'".to_string())
}

#[tauri::command]
pub async fn stop_wake_word_detection() -> Result<String, String> {
    let mut state = WAKE_WORD_STATE.lock().unwrap();
    state.is_active = false;
    state.is_listening = false;
    state.whisper_activated = false;
    
    println!("Wake word detection stopped");
    Ok("Wake word detection stopped".to_string())
}

#[tauri::command]
pub async fn check_wake_word_detection() -> Result<Option<WakeWordDetectionInfo>, String> {
    let state = {
        let state_guard = WAKE_WORD_STATE.lock().unwrap();
        state_guard.clone()
    };
    
    if state.is_active {
        // Check the always-on speech system for wake word detections
        let mut manager = SPEECH_MANAGER.lock().unwrap();
        if let Some(ref mut manager_instance) = manager.as_mut() {
            if let Some(detection) = manager_instance.check_wake_word() {
                let detection_info = WakeWordDetectionInfo {
                    confidence: detection.confidence,
                    timestamp: detection.timestamp,
                    audio_length: detection.audio_snippet.len(),
                };
                
                // Update state
                {
                    let mut state_guard = WAKE_WORD_STATE.lock().unwrap();
                    state_guard.last_detection = Some(detection_info.clone());
                    state_guard.total_detections += 1;
                    state_guard.whisper_activated = true;
                }
                
                return Ok(Some(detection_info));
            }
        }
    }
    
    Ok(None)
}

#[tauri::command]
pub async fn get_wake_word_state() -> Result<WakeWordState, String> {
    let state = WAKE_WORD_STATE.lock().unwrap();
    Ok(state.clone())
}

#[tauri::command]
pub async fn reset_wake_word_stats() -> Result<String, String> {
    let mut state = WAKE_WORD_STATE.lock().unwrap();
    state.total_detections = 0;
    state.last_detection = None;
    state.whisper_activated = false;
    
    Ok("Wake word statistics reset".to_string())
}

// Helper functions for Whisper
async fn get_or_download_model(model_size: &str) -> Result<PathBuf, String> {
    let model_path = get_model_path(model_size);
    
    if !model_path.exists() || !is_valid_model_file(&model_path) {
        if model_path.exists() {
            fs::remove_file(&model_path)
                .map_err(|e| format!("Failed to remove invalid model: {}", e))?;
        }
        download_model(model_size).await?;
    }
    
    Ok(model_path)
}

fn is_valid_model_file(path: &PathBuf) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        metadata.len() > 1_000_000 // 1MB minimum
    } else {
        false
    }
}

fn get_model_path(model_size: &str) -> PathBuf {
    let mut path = MODEL_CACHE_DIR.clone();
    path.push(format!("ggml-{}.bin", model_size));
    path
}

async fn download_model(model_size: &str) -> Result<(), String> {
    fs::create_dir_all(&*MODEL_CACHE_DIR)
        .map_err(|e| format!("Failed to create cache directory: {}", e))?;
    
    let model_url = format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{}.bin",
        model_size
    );
    
    let model_path = get_model_path(model_size);
    
    println!("Downloading Whisper model '{}' from: {}", model_size, model_url);
    
    let response = reqwest::get(&model_url).await
        .map_err(|e| format!("Failed to download model: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to download model: HTTP {}", response.status()));
    }
    
    let bytes = response.bytes().await
        .map_err(|e| format!("Failed to read model data: {}", e))?;
    
    fs::write(&model_path, bytes)
        .map_err(|e| format!("Failed to save model: {}", e))?;
    
    println!("Successfully downloaded Whisper model '{}' to: {:?}", model_size, model_path);
    
    Ok(())
}

fn load_audio_file(file_path: &str) -> Result<Vec<f32>, String> {
    let audio_bytes = fs::read(file_path)
        .map_err(|e| format!("Failed to read audio file: {}", e))?;
    
    let mut audio_f32 = Vec::new();
    for chunk in audio_bytes.chunks(2) {
        if chunk.len() == 2 {
            let sample = i16::from_le_bytes([chunk[0], chunk[1]]) as f32 / 32768.0;
            audio_f32.push(sample);
        }
    }
    
    Ok(audio_f32)
}
