use std::process::{Command, Stdio, Child};
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
// Note: SystemTime and UNIX_EPOCH are used in calibration timestamps
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
        if self.is_running() {
            return Err("Speech recognition is already running".to_string());
        }

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
                        }
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
            process.kill().map_err(|e| format!("Failed to kill speech process: {}", e))?;
            process.wait().map_err(|e| format!("Failed to wait for speech process: {}", e))?;
        }
        
        self.wake_word_receiver = None;
        self.transcription_receiver = None;
        self.state = SpeechState::default();
        
        println!("🎤 Stopped speech recognition");
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.process.is_some() && self.state.is_listening
    }

    pub fn get_state(&self) -> &SpeechState {
        &self.state
    }

    pub fn check_wake_word(&mut self) -> Option<WakeWordDetection> {
        if let Some(receiver) = &mut self.wake_word_receiver {
            if let Ok(detection) = receiver.try_recv() {
                self.state.wake_word_detected = true;
                self.state.last_detection = Some(detection.clone());
                self.state.total_detections += 1;
                return Some(detection);
            }
        }
        None
    }

    pub fn check_transcription(&mut self) -> Option<SpeechTranscription> {
        if let Some(receiver) = &mut self.transcription_receiver {
            if let Ok(transcription) = receiver.try_recv() {
                self.state.last_transcription = Some(transcription.clone());
                return Some(transcription);
            }
        }
        None
    }

    fn create_python_script(&self) -> String {
        format!(r#"
#!/usr/bin/env python3
"""
Enhanced Aubrey Wake Word Detection with Transcription
Listens for "Aubrey" wake word and provides speech transcription
"""

import pyaudio
import wave
import numpy as np
import speech_recognition as sr
import io
import sys
import json
import time
import threading
import argparse
from collections import deque

class AubreyDetector:
    def __init__(self, sample_rate=16000, silence_threshold=0.01, silence_duration=2.0, max_duration=30.0):
        self.sample_rate = sample_rate
        self.silence_threshold = silence_threshold
        self.silence_duration = silence_duration
        self.max_duration = max_duration
        self.chunk_size = 1024
        
        self.audio = pyaudio.PyAudio()
        self.recognizer = sr.Recognizer()
        self.microphone = sr.Microphone(sample_rate=sample_rate)
        
        # Adjust for ambient noise
        with self.microphone as source:
            self.recognizer.adjust_for_ambient_noise(source, duration=1)
        
        print(f"🎤 Aubrey detector initialized (SR threshold: {{self.recognizer.energy_threshold}})", file=sys.stderr)
        
        self.is_running = False
        self.wake_word_detected = False
        
    def listen_for_wake_word(self):
        """Continuously listen for the wake word 'Aubrey'"""
        self.is_running = True
        
        while self.is_running:
            try:
                with self.microphone as source:
                    # Listen for audio with timeout
                    audio = self.recognizer.listen(source, timeout=1, phrase_time_limit=5)
                
                try:
                    # Use Google Speech Recognition for wake word detection
                    text = self.recognizer.recognize_google(audio).lower()
                    
                    if "aubrey" in text or "aubry" in text or "awbrey" in text:
                        detection = {{
                            "confidence": 0.85,
                            "timestamp": int(time.time() * 1000),
                            "audio_snippet": []  # Would contain audio data in real implementation
                        }}
                        
                        print(f"WAKE_WORD:{{json.dumps(detection)}}")
                        sys.stdout.flush()
                        
                        # Start transcription mode
                        self.start_transcription()
                        
                except sr.UnknownValueError:
                    # No speech detected, continue listening
                    pass
                except sr.RequestError as e:
                    print(f"Speech recognition error: {{e}}", file=sys.stderr)
                    time.sleep(1)
                    
            except sr.WaitTimeoutError:
                # Timeout, continue listening
                pass
            except Exception as e:
                print(f"Wake word detection error: {{e}}", file=sys.stderr)
                time.sleep(1)
    
    def start_transcription(self):
        """Start transcription after wake word detection"""
        print("🎤 Starting transcription...", file=sys.stderr)
        
        silence_counter = 0
        max_silence = int(self.silence_duration * (self.sample_rate / self.chunk_size))
        
        try:
            with self.microphone as source:
                audio = self.recognizer.listen(source, timeout=1, phrase_time_limit=self.max_duration)
            
            try:
                text = self.recognizer.recognize_google(audio)
                transcription = {{
                    "text": text,
                    "confidence": 0.9,
                    "duration": 2.0,  # Estimated duration
                    "timestamp": int(time.time() * 1000)
                }}
                
                print(f"TRANSCRIPTION:{{json.dumps(transcription)}}")
                sys.stdout.flush()
                
            except sr.UnknownValueError:
                print("Could not understand audio", file=sys.stderr)
            except sr.RequestError as e:
                print(f"Transcription error: {{e}}", file=sys.stderr)
                
        except Exception as e:
            print(f"Transcription error: {{e}}", file=sys.stderr)
    
    def stop(self):
        """Stop the detector"""
        self.is_running = False
        self.audio.terminate()

def main():
    parser = argparse.ArgumentParser(description='Aubrey Wake Word Detector')
    parser.add_argument('--sample-rate', type=int, default={sample_rate}, help='Sample rate')
    parser.add_argument('--silence-threshold', type=float, default={silence_threshold}, help='Silence threshold')
    parser.add_argument('--silence-duration', type=float, default={silence_duration}, help='Silence duration')
    parser.add_argument('--max-duration', type=float, default={max_duration}, help='Max recording duration')
    
    args = parser.parse_args()
    
    detector = AubreyDetector(
        sample_rate=args.sample_rate,
        silence_threshold=args.silence_threshold,
        silence_duration=args.silence_duration,
        max_duration=args.max_duration
    )
    
    try:
        detector.listen_for_wake_word()
    except KeyboardInterrupt:
        print("🛑 Stopping Aubrey detector...", file=sys.stderr)
    finally:
        detector.stop()

if __name__ == "__main__":
    main()
"#,
            sample_rate = self.config.sample_rate,
            silence_threshold = self.config.silence_threshold,
            silence_duration = self.config.silence_duration,
            max_duration = self.config.max_recording_duration
        )
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
    match SPEECH_MANAGER.lock() {
        Ok(mut manager_opt) => {
            let mut manager = SpeechManager::new(AudioConfig::default());
            manager.start()?;
            *manager_opt = Some(manager);
            Ok("Always-on speech recognition started".to_string())
        }
        Err(_) => Err("Failed to access speech manager".to_string())
    }
}

#[tauri::command]
pub async fn stop_always_on_speech() -> Result<String, String> {
    match SPEECH_MANAGER.lock() {
        Ok(mut manager_opt) => {
            if let Some(manager) = manager_opt.as_mut() {
                manager.stop()?;
            }
            *manager_opt = None;
            Ok("Always-on speech recognition stopped".to_string())
        }
        Err(_) => Err("Failed to access speech manager".to_string())
    }
}

#[tauri::command]
pub async fn get_speech_state() -> Result<SpeechState, String> {
    match SPEECH_MANAGER.lock() {
        Ok(manager_opt) => {
            if let Some(manager) = manager_opt.as_ref() {
                Ok(manager.get_state().clone())
            } else {
                Ok(SpeechState::default())
            }
        }
        Err(_) => Err("Failed to access speech manager".to_string())
    }
}

#[tauri::command]
pub async fn check_for_wake_word() -> Result<Option<WakeWordDetection>, String> {
    match SPEECH_MANAGER.lock() {
        Ok(mut manager_opt) => {
            if let Some(manager) = manager_opt.as_mut() {
                Ok(manager.check_wake_word())
            } else {
                Ok(None)
            }
        }
        Err(_) => Err("Failed to access speech manager".to_string())
    }
}

#[tauri::command]
pub async fn check_for_transcription() -> Result<Option<SpeechTranscription>, String> {
    match SPEECH_MANAGER.lock() {
        Ok(mut manager_opt) => {
            if let Some(manager) = manager_opt.as_mut() {
                Ok(manager.check_transcription())
            } else {
                Ok(None)
            }
        }
        Err(_) => Err("Failed to access speech manager".to_string())
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
    start_always_on_speech().await
}

#[tauri::command]
pub async fn stop_wake_word_detection() -> Result<String, String> {
    stop_always_on_speech().await
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
