# Aggregate Device Cleanup Script

This script cleans up all existing aggregate audio devices on macOS. It's useful for removing test devices or cleaning up after development sessions.

## Usage

### Option 1: Using the shell script (Recommended)
```bash
./cleanup_aggregate_devices.sh
```

### Option 2: Building and running manually
```bash
cd src-tauri
cargo build --bin cleanup_aggregate_devices
./target/debug/cleanup_aggregate_devices
```

## What it does

The script:
1. Scans all audio devices on the system
2. Identifies aggregate devices (devices with transport type `kAudioDeviceTransportTypeAggregate`)
3. Destroys each aggregate device using Core Audio APIs
4. Provides detailed logging of the cleanup process

## Safety

⚠️ **Warning**: This script will destroy ALL aggregate audio devices on your system, including:
- Manually created aggregate devices
- Test devices created during development
- Any other aggregate devices

Make sure you don't have any important aggregate devices before running this script.

## Output Example

```
[cleanup_script] Starting aggregate device cleanup...
Device count: 17
Found 17 devices: [101, 98, 105, 91, 84, 111, 112, 113, 114, 110, 106, 96, 74, 109, 107, 104, 108]
Device 101 transport type: 1735554416
Device 101 name: Sample Aggregate Audio Device
Destroying aggregate device 101: Sample Aggregate Audio Device
Destroyed aggregate device 101: Sample Aggregate Audio Device
...
[cleanup_script] Successfully cleaned up aggregate devices
[cleanup_script] Cleanup completed!
```

## Files

- `cleanup_aggregate_devices.sh` - Shell script wrapper
- `src-tauri/src/cleanup_aggregate_devices.rs` - Rust binary source
- `src-tauri/Cargo.toml` - Updated to include the binary target
