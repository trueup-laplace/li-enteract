# Always-On Speech Recognition with "Aubrey" Wake Word

This document describes the new speech recognition system implemented for Enteract, which replaces the previous whisper-rs implementation with a Python-based always-on microphone system.

## Overview

The new speech system provides:
- **Always-on microphone listening** for the wake word "Aubrey"
- **Automatic recording** after wake word detection 
- **Silence detection** to automatically stop recording
- **Speech transcription** of the recorded audio
- **Clean modular architecture** separated from other app functionality

## Architecture

### Rust Side (Tauri)
- **`speech.rs`** - Main speech management module
- **`SpeechManager`** - Handles Python process lifecycle
- **Tauri commands** - Interface for frontend to control speech functionality

### Python Side
- **`aubrey_speech_detector.py`** - Generated Python script for audio processing
- **PyAudio** - Microphone access and real-time audio streaming
- **NumPy** - Audio signal processing and analysis
- **Whisper** - Speech transcription (in the full implementation)

## Installation

### 1. Install Python Dependencies

**Windows:**
```bash
install_speech_deps.bat
```

**Linux/macOS:**
```bash
./install_speech_deps.sh
```

**Manual installation:**
```bash
pip install pyaudio numpy openai-whisper scipy
```

### 2. Build the Application

```bash
npm run tauri dev
# or for production
npm run tauri build
```

## Usage

### Frontend Integration

The speech system provides these Tauri commands:

```typescript
// Start always-on speech detection
await invoke('start_always_on_speech');

// Stop speech detection
await invoke('stop_always_on_speech');

// Get current speech state
const state = await invoke('get_speech_state');

// Check for wake word detection
const detection = await invoke('check_for_wake_word');

// Check for completed transcription
const transcription = await invoke('check_for_transcription');
```

### Speech Flow

1. **Start Listening**: Call `start_always_on_speech()`
2. **Wake Word Detection**: Say "Aubrey" - triggers recording
3. **Recording**: Speak your command/query
4. **Silence Detection**: Stop speaking - recording ends automatically
5. **Transcription**: Get the transcribed text via `check_for_transcription()`

## Configuration

The speech system can be configured via the `AudioConfig` struct:

```rust
pub struct AudioConfig {
    pub sample_rate: u32,          // Default: 16000 Hz
    pub chunk_size: usize,         // Default: 1024
    pub silence_threshold: f32,    // Default: 0.01
    pub silence_duration: f32,     // Default: 2.0 seconds
    pub max_recording_duration: f32, // Default: 30.0 seconds
}
```

## Data Structures

### Wake Word Detection
```rust
pub struct WakeWordDetection {
    pub confidence: f32,
    pub timestamp: u64,
    pub audio_snippet: Vec<f32>,
}
```

### Speech Transcription
```rust
pub struct SpeechTranscription {
    pub text: String,
    pub confidence: f32,
    pub duration: f32,
    pub timestamp: u64,
}
```

### Speech State
```rust
pub struct SpeechState {
    pub is_listening: bool,
    pub is_recording: bool,
    pub wake_word_detected: bool,
    pub last_detection: Option<WakeWordDetection>,
    pub last_transcription: Option<SpeechTranscription>,
    pub total_detections: u32,
}
```

## Technical Details

### Wake Word Detection
Currently uses a simple energy-based detection with spectral analysis:
- **Energy threshold**: Detects voice activity
- **Spectral centroid**: Filters for human voice frequency range (800-2000 Hz)
- **Confidence scoring**: Based on audio energy levels

For production use, consider integrating:
- [Picovoice Porcupine](https://picovoice.ai/platform/porcupine/) for better wake word accuracy
- [WebRTC VAD](https://github.com/wiseman/py-webrtcvad) for improved voice activity detection

### Audio Processing
- **Sample Rate**: 16kHz (standard for speech recognition)
- **Format**: 32-bit float audio data
- **Channels**: Mono (single channel)
- **Buffering**: 3-second circular buffer for wake word detection

### Transcription
The current implementation includes a placeholder for transcription. To enable full transcription:

1. The Python script already includes Whisper integration (commented out)
2. Uncomment the Whisper transcription code in the Python script
3. Ensure Whisper model is downloaded (happens automatically on first use)

## Troubleshooting

### Common Issues

**"Python not found"**
- Install Python 3.8+ and add to PATH
- Try running `python --version` or `python3 --version`

**"PyAudio installation failed"**
- Windows: Install Visual Studio Build Tools
- Linux: Install PortAudio dev headers (`sudo apt-get install portaudio19-dev`)
- macOS: Install with Homebrew (`brew install portaudio`)

**"No microphone detected"**
- Check microphone permissions in system settings
- Ensure microphone is not in use by other applications
- Try running the Python script directly to test audio access

### Debug Mode

To debug the Python audio processing:
- Check the Tauri console for "Audio Debug:" messages
- Python errors will appear as "Audio Error:" in the console
- The generated Python script is saved to temp directory for inspection

## Future Improvements

1. **Better Wake Word Detection**
   - Integrate Picovoice Porcupine or similar library
   - Train custom wake word model for "Aubrey"

2. **Enhanced Audio Processing**
   - Noise cancellation
   - Automatic gain control
   - Multiple microphone support

3. **Advanced Transcription**
   - Real-time streaming transcription
   - Custom vocabulary and language models
   - Confidence-based filtering

4. **Performance Optimization**
   - Reduce audio processing latency
   - Optimize memory usage for long-running sessions
   - Background processing without blocking UI

## Migration from Whisper-rs

The new Python-based system replaces the previous whisper-rs implementation:

### Removed Dependencies
- `whisper-rs`
- `anyhow`
- `base64`
- `tempfile`
- `reqwest`

### Removed Commands
- `initialize_whisper_model`
- `transcribe_audio_base64`
- `transcribe_audio_file`
- `check_whisper_model_availability`
- `download_whisper_model`
- `list_available_models`
- `start_wake_word_detection`
- `stop_wake_word_detection`
- `check_wake_word_detection`
- `get_wake_word_state`
- `reset_wake_word_stats`

### New Commands
- `start_always_on_speech`
- `stop_always_on_speech`
- `get_speech_state`
- `check_for_wake_word`
- `check_for_transcription`

This new implementation provides a much more responsive and user-friendly speech interface with true always-on capabilities. 