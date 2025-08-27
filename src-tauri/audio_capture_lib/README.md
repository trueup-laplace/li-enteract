# Audio Capture Library

A cross-platform Rust library for capturing audio from various sources including microphones, system audio, and application-specific audio streams. This library is designed to work with Core Audio on macOS and can be integrated with Tauri applications.

## Features

- **Cross-platform audio device enumeration**
- **Multiple capture methods**: Direct capture, loopback capture, audio taps, and aggregate devices
- **Real-time audio processing**: Resampling, format conversion, and audio analysis
- **Transcription support**: Built-in support for audio transcription workflows
- **Async/await support**: Modern async Rust patterns for non-blocking audio capture
- **Tauri integration**: Designed to work seamlessly with Tauri applications

## Architecture

The library is organized into several modules:

- **`types`**: Core data structures and enums
- **`device_enumerator`**: Platform-agnostic device discovery
- **`capture_engine`**: Audio capture management and callbacks
- **`audio_processor`**: Audio processing utilities (resampling, format conversion)
- **`macos/`**: macOS-specific Core Audio implementations

## Quick Start

### Basic Usage

```rust
use audio_capture_lib::{
    AudioCaptureManager, CaptureConfig, CaptureMethod,
    capture_engine::utils::{create_debug_audio_callback, create_debug_transcription_callback},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create capture manager
    let mut capture_manager = AudioCaptureManager::new();
    
    // Enumerate devices
    let enumerator = audio_capture_lib::device_enumerator::create_device_enumerator()?;
    let devices = enumerator.enumerate_devices().await?;
    
    // Find a suitable device
    let device = devices.iter()
        .find(|d| d.device_type == audio_capture_lib::types::DeviceType::Capture)
        .ok_or("No suitable device found")?;
    
    // Configure capture
    let config = CaptureConfig {
        device_id: device.id.clone(),
        sample_rate: 16000,
        channels: 1,
        buffer_size: 4096,
        capture_method: CaptureMethod::Direct,
        enable_transcription: true,
        transcription_buffer_duration: 4.0,
        transcription_interval_ms: 800,
        min_audio_length: 1.5,
    };
    
    // Start capture
    capture_manager.start_capture(
        config,
        Some(create_debug_audio_callback()),
        Some(create_debug_transcription_callback()),
    ).await?;
    
    // ... capture is running ...
    
    // Stop capture
    capture_manager.stop_capture().await?;
    
    Ok(())
}
```

### Tauri Integration

To integrate with your Tauri app, add the library as a dependency in your `Cargo.toml`:

```toml
[dependencies]
audio_capture_lib = { path = "./audio_capture_lib" }
```

Then use it in your Tauri commands:

```rust
use audio_capture_lib::{AudioCaptureManager, CaptureConfig, CaptureMethod};
use tauri::AppHandle;

#[tauri::command]
pub async fn start_audio_capture(
    device_id: String,
    app_handle: AppHandle,
) -> Result<String, String> {
    let mut capture_manager = AudioCaptureManager::new();
    
    let config = CaptureConfig {
        device_id,
        sample_rate: 16000,
        channels: 1,
        buffer_size: 4096,
        capture_method: CaptureMethod::Direct,
        enable_transcription: true,
        transcription_buffer_duration: 4.0,
        transcription_interval_ms: 800,
        min_audio_length: 1.5,
    };
    
    // Create callbacks that emit to Tauri events
    let audio_callback = Box::new(move |chunk| {
        let app_handle = app_handle.clone();
        tokio::spawn(async move {
            let _ = app_handle.emit("audio-chunk", chunk);
        });
    });
    
    capture_manager.start_capture(
        config,
        Some(audio_callback),
        None,
    ).await.map_err(|e| e.to_string())?;
    
    Ok("Audio capture started".to_string())
}
```

## Capture Methods

The library supports several capture methods:

### Direct Capture
For microphones and input devices:
```rust
let config = CaptureConfig {
    capture_method: CaptureMethod::Direct,
    // ... other config
};
```

### Loopback Capture
For system audio (speakers, headphones):
```rust
let config = CaptureConfig {
    capture_method: CaptureMethod::Loopback,
    // ... other config
};
```

### Audio Taps
For capturing audio from specific applications:
```rust
let config = CaptureConfig {
    capture_method: CaptureMethod::AudioTap,
    // ... other config
};
```

### Aggregate Devices
For combining multiple audio sources:
```rust
let config = CaptureConfig {
    capture_method: CaptureMethod::AggregateDevice,
    // ... other config
};
```

## Audio Processing

The library includes utilities for audio processing:

```rust
use audio_capture_lib::audio_processor::AudioProcessor;

// Resample audio
let resampled = AudioProcessor::resample_audio(&samples, 48000, 16000)?;

// Convert stereo to mono
let mono = AudioProcessor::stereo_to_mono(&stereo_samples)?;

// Calculate audio level
let level_db = AudioProcessor::calculate_level_db(&samples);

// Detect silence
let is_silent = AudioProcessor::detect_silence(&samples, 0.001);
```

## Error Handling

The library uses a custom error type for comprehensive error handling:

```rust
use audio_capture_lib::types::AudioCaptureError;

match result {
    Ok(data) => { /* handle success */ },
    Err(AudioCaptureError::DeviceNotFound(id)) => {
        println!("Device {} not found", id);
    },
    Err(AudioCaptureError::CoreAudioError(msg)) => {
        println!("Core Audio error: {}", msg);
    },
    Err(e) => {
        println!("Other error: {}", e);
    },
}
```

## Building

To build the library:

```bash
cd audio_capture_lib
cargo build
```

To run the example:

```bash
cargo run --example basic_capture
```

## Dependencies

The library depends on several Rust crates:

- **`core_audio`**: Core Audio bindings for macOS
- **`cpal`**: Cross-platform audio I/O
- **`tokio`**: Async runtime
- **`serde`**: Serialization
- **`tracing`**: Logging

## Platform Support

Currently supported platforms:
- **macOS**: Full Core Audio support with all capture methods
- **Windows**: Basic WASAPI support (planned)
- **Linux**: Basic ALSA support (planned)

## Contributing

This library is designed to be extensible. To add support for new platforms or capture methods:

1. Implement the `DeviceEnumerator` trait for your platform
2. Implement the `CaptureEngine` trait for your capture method
3. Add platform-specific modules following the existing pattern

## License

This library is part of the larger project and follows the same licensing terms.
