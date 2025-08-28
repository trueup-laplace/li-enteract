# macOS Audio Loopback Implementation Plan

## Overview
This document outlines the incremental implementation strategy for the macOS audio loopback system, providing a clear checklist for each phase.

## Required Modules

### Core Modules (Must Implement)
- [x] **`macos/device_enumerator.rs`** - Device discovery and enumeration
- [x] **`macos/core_audio_bindings.rs`** - Core Audio API bindings
- [ ] **`macos/capture_engine.rs`** - Audio capture loop and I/O management

### Optional/Supporting Modules
- [x] **`macos/device_loader.rs`** - Device management utilities
- [x] **`macos/mod.rs`** - Module exports

## Implementation Phases

### ✅ Phase 1: Basic Device Enumeration (COMPLETED)
**Goal**: Get device listing working without audio capture

**Completed Tasks**:
- [x] `CoreAudioLoopbackEnumerator::new()`
- [x] `get_audio_devices()`
- [x] `get_device_name()`
- [x] `enumerate_loopback_devices()`
- [x] Basic error handling with anyhow
- [x] Tauri commands for device enumeration

**Test Strategy**:
```bash
# Test device enumeration
cargo test device_enumeration
# Or test via Tauri command
await invoke('enumerate_loopback_devices')
```

**Expected Output**: List of audio devices with names, sample rates, etc.

### 🔄 Phase 2: Device Format Detection (IN PROGRESS)
**Goal**: Understand device capabilities

**Completed Tasks**:
- [x] `get_device_format()` - Get sample rate, channels, format type
- [x] `get_device_type()` - Determine if device is render or capture
- [x] `is_default_device()` - Check if device is default input/output
- [x] `device_has_output_streams()` - Check for output capabilities
- [x] Basic device info creation in `create_device_info()`

**Current Tasks**:
- [ ] Enhanced format validation
- [ ] Device capability testing
- [ ] Format compatibility checking
- [ ] Extended test coverage

**Test Strategy**:
```bash
# Test format detection
await invoke('test_audio_device', { deviceId: "some_device_id" })
```

**Expected Output**: Device format information (sample rate, channels, format type)

### ⏳ Phase 3: Basic Audio Recorder (PENDING)
**Goal**: Create minimal audio recorder that can initialize

**Tasks**:
- [ ] `AudioRecorder::new()`
- [ ] `set_device_id()`
- [ ] `start_io()` / `stop_io()` (empty implementations)
- [ ] Basic Core Audio setup

**Test Strategy**:
```rust
#[test]
fn test_audio_recorder_creation() {
    let mut recorder = AudioRecorder::new();
    assert!(recorder.set_device_id(device_id).is_ok());
}
```

### ⏳ Phase 4: I/O Procedure Setup (PENDING)
**Goal**: Get Core Audio I/O procedure working

**Tasks**:
- [ ] `AudioRecorder::start_io()` (real implementation)
- [ ] `io_proc` callback function
- [ ] Basic buffer handling
- [ ] Audio unit setup and configuration

**Test Strategy**:
```rust
#[test]
fn test_io_proc_initialization() {
    let mut recorder = AudioRecorder::new();
    recorder.set_device_id(device_id)?;
    assert!(recorder.start_io().is_ok());
    // Should not crash
    std::thread::sleep(Duration::from_millis(100));
    assert!(recorder.stop_io().is_ok());
}
```

### ⏳ Phase 5: Audio Buffer Processing (PENDING)
**Goal**: Get audio data flowing through the system

**Tasks**:
- [ ] `WhisperManager::new()`
- [ ] `add_audio_data()`
- [ ] `get_audio_chunk()`
- [ ] `resample_audio()`
- [ ] Audio format conversion

**Test Strategy**:
```rust
#[test]
fn test_audio_buffer_flow() {
    let mut whisper_manager = WhisperManager::new();
    let test_samples = vec![0.1, 0.2, 0.3, 0.4, 0.5];
    whisper_manager.add_audio_data(test_samples);
    
    let chunk = whisper_manager.get_audio_chunk();
    assert!(!chunk.is_empty());
}
```

### ⏳ Phase 6: Integration with Existing Pipeline (PENDING)
**Goal**: Connect to your existing audio processing

**Tasks**:
- [ ] `run_audio_capture_loop_sync()` (minimal version)
- [ ] Integration with `process_audio_for_transcription`
- [ ] Event emission setup
- [ ] Error handling integration

**Test Strategy**:
```bash
# Test full pipeline
await invoke('start_audio_loopback_capture', { deviceId: "test_device" })
# Should start without errors
await invoke('stop_audio_loopback_capture')
```

### ⏳ Phase 7: Full Capture Loop (PENDING)
**Goal**: Complete audio capture with transcription

**Tasks**:
- [ ] Complete `run_audio_capture_loop_sync()`
- [ ] Event emission
- [ ] Error handling
- [ ] Cleanup procedures
- [ ] Performance optimization

## Testing Infrastructure

### 1. Unit Tests for Each Phase
- [x] Basic device enumeration tests
- [x] Error handling tests
- [x] Auto-select device tests
- [ ] Device format validation tests
- [ ] Audio recorder lifecycle tests
- [ ] Whisper manager buffer tests
- [ ] I/O procedure tests

### 2. Integration Tests
- [ ] Full capture pipeline tests
- [ ] End-to-end audio flow tests
- [ ] Error recovery tests

### 3. Manual Testing Commands
```bash
# Test each phase manually
pnpm tauri dev
# Then in browser console:
await invoke('enumerate_loopback_devices')
await invoke('auto_select_best_device')
await invoke('test_audio_device', { deviceId: "..." })
await invoke('start_audio_loopback_capture', { deviceId: "..." })
```

## Core Audio Integration Tips

### 1. Error Handling Strategy
```rust
// Wrap Core Audio calls in Result
fn safe_core_audio_call() -> Result<()> {
    unsafe {
        let result = SomeCoreAudioFunction();
        if result != kAudioHardwareNoError {
            return Err(anyhow::anyhow!("Core Audio error: {}", result));
        }
    }
    Ok(())
}
```

### 2. Memory Management
```rust
// Use RAII for Core Audio resources
impl Drop for AudioRecorder {
    fn drop(&mut self) {
        let _ = self.stop_io();
    }
}
```

### 3. Debugging Strategy
```rust
// Add extensive logging for each phase
println!("[PHASE1] Device enumeration: found {} devices", devices.len());
println!("[PHASE2] Device format: {}Hz, {}ch", sample_rate, channels);
println!("[PHASE3] Audio recorder created successfully");
```

## Current Status

**Completed**: Phase 1 (Device Enumeration)
**In Progress**: Phase 2 (Device Format Detection)
**Next**: Phase 3 (Basic Audio Recorder)

## Notes

- All Core Audio bindings are implemented using `objc2_core_audio`
- Error handling uses `anyhow` for consistent error propagation
- Tauri commands provide the interface for frontend integration
- Tests are written using Rust's built-in testing framework

## Next Steps

1. Complete Phase 2 tests and validation
2. Begin Phase 3 implementation
3. Set up continuous testing for each phase
4. Document any Core Audio specific issues encountered
