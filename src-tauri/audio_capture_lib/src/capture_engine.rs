//! Audio capture engine for managing audio capture sessions

use crate::types::{AudioCaptureResult, CaptureConfig, AudioChunk, TranscriptionResult};
use async_trait::async_trait;
use tokio::sync::mpsc;
use std::sync::Arc;

/// Callback for receiving audio chunks
pub type AudioChunkCallback = Box<dyn Fn(AudioChunk) + Send + Sync>;

/// Callback for receiving transcription results
pub type TranscriptionCallback = Box<dyn Fn(TranscriptionResult) + Send + Sync>;

/// Trait for audio capture engines
#[async_trait]
pub trait CaptureEngine: Send + Sync {
    /// Start audio capture with the given configuration
    async fn start_capture(
        &mut self,
        config: CaptureConfig,
        audio_callback: Option<AudioChunkCallback>,
        transcription_callback: Option<TranscriptionCallback>,
    ) -> AudioCaptureResult<()>;
    
    /// Stop audio capture
    async fn stop_capture(&mut self) -> AudioCaptureResult<()>;
    
    /// Check if capture is currently active
    fn is_capturing(&self) -> bool;
    
    /// Get the current capture configuration
    fn get_config(&self) -> Option<&CaptureConfig>;
    
    /// Update capture configuration (requires restart)
    async fn update_config(&mut self, config: CaptureConfig) -> AudioCaptureResult<()>;
}

/// Main audio capture manager
pub struct AudioCaptureManager {
    engine: Option<Box<dyn CaptureEngine>>,
    audio_callback: Option<AudioChunkCallback>,
    transcription_callback: Option<TranscriptionCallback>,
}

impl AudioCaptureManager {
    /// Create a new audio capture manager
    pub fn new() -> Self {
        Self {
            engine: None,
            audio_callback: None,
            transcription_callback: None,
        }
    }
    
    /// Create a capture engine for the current platform
    pub fn create_engine() -> AudioCaptureResult<Box<dyn CaptureEngine>> {
        #[cfg(target_os = "macos")]
        {
            use crate::macos::CoreAudioCaptureEngine;
            Ok(Box::new(CoreAudioCaptureEngine::new()?))
        }
        
        #[cfg(target_os = "windows")]
        {
            use crate::windows::WASAPICaptureEngine;
            Ok(Box::new(WASAPICaptureEngine::new()?))
        }
        
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            Err(crate::types::AudioCaptureError::Other(
                "Unsupported platform".to_string()
            ))
        }
    }
    
    /// Start audio capture
    pub async fn start_capture(
        &mut self,
        config: CaptureConfig,
        audio_callback: Option<AudioChunkCallback>,
        transcription_callback: Option<TranscriptionCallback>,
    ) -> AudioCaptureResult<()> {
        // Create engine if not exists
        if self.engine.is_none() {
            self.engine = Some(Self::create_engine()?);
        }
        
        // Store callbacks
        self.audio_callback = audio_callback;
        self.transcription_callback = transcription_callback;
        
        // Start capture
        if let Some(engine) = &mut self.engine {
            engine.start_capture(
                config,
                self.audio_callback.take(),
                self.transcription_callback.take(),
            ).await?;
        }
        
        Ok(())
    }
    
    /// Stop audio capture
    pub async fn stop_capture(&mut self) -> AudioCaptureResult<()> {
        if let Some(engine) = &mut self.engine {
            engine.stop_capture().await?;
        }
        Ok(())
    }
    
    /// Check if capture is active
    pub fn is_capturing(&self) -> bool {
        self.engine.as_ref().map(|e| e.is_capturing()).unwrap_or(false)
    }
    
    /// Get current configuration
    pub fn get_config(&self) -> Option<&CaptureConfig> {
        self.engine.as_ref().and_then(|e| e.get_config())
    }
    
    /// Update configuration
    pub async fn update_config(&mut self, config: CaptureConfig) -> AudioCaptureResult<()> {
        if let Some(engine) = &mut self.engine {
            engine.update_config(config).await?;
        }
        Ok(())
    }
}

impl Default for AudioCaptureManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for audio capture
pub mod utils {
    use super::*;
    
    /// Create a simple audio chunk callback that prints to console
    pub fn create_debug_audio_callback() -> AudioChunkCallback {
        Box::new(|chunk: AudioChunk| {
            tracing::debug!(
                "Audio chunk: device={}, samples={}, level={:.2}dB, timestamp={}",
                chunk.device_id,
                chunk.audio_data.len(),
                chunk.level,
                chunk.timestamp
            );
        })
    }
    
    /// Create a simple transcription callback that prints to console
    pub fn create_debug_transcription_callback() -> TranscriptionCallback {
        Box::new(|result: TranscriptionResult| {
            tracing::info!(
                "Transcription: '{}' (confidence: {:.2}, time: {:.2}s-{:.2}s)",
                result.text,
                result.confidence,
                result.start_time,
                result.end_time
            );
        })
    }
    
    /// Create a callback that sends audio chunks to a channel
    pub fn create_channel_audio_callback(
        tx: mpsc::Sender<AudioChunk>
    ) -> AudioChunkCallback {
        Box::new(move |chunk: AudioChunk| {
            let tx = tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(chunk).await;
            });
        })
    }
    
    /// Create a callback that sends transcription results to a channel
    pub fn create_channel_transcription_callback(
        tx: mpsc::Sender<TranscriptionResult>
    ) -> TranscriptionCallback {
        Box::new(move |result: TranscriptionResult| {
            let tx = tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(result).await;
            });
        })
    }
}
