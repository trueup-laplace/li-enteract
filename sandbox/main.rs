use std::error::Error;
use std::env;
use wasapi::initialize_mta;

mod device_enumerator;
use device_enumerator::WASAPILoopbackEnumerator;

fn show_help() {
    println!("=== WASAPI Loopback Audio Tools (Enhanced) ===");
    println!();
    println!("This version includes robust fallback strategies for Windows audio driver inconsistencies.");
    println!();
    println!("Usage:");
    println!("  cargo run --bin device_enum           - List available loopback devices");
    println!("  cargo run --bin audio_capture         - Start robust real-time capture");
    println!();
    println!("Or use this main binary:");
    println!("  cargo run -- devices                  - List devices with all methods");
    println!("  cargo run -- transcribe [model_path]  - Start transcription (placeholder)");
    println!();
    println!("Examples:");
    println!("  cargo run -- devices");
    println!("  cargo run -- transcribe models/ggml-tiny.bin");
    println!();
    println!("Device Detection Strategies:");
    println!("  1. Render devices with loopback capability");
    println!("  2. Capture devices with loopback-style names");
    println!("  3. Stereo Mix and system audio capture devices");
    println!("  4. Emergency fallback methods");
    println!();
    println!("Troubleshooting:");
    println!("  - Run as Administrator if device detection fails");
    println!("  - Enable 'Stereo Mix' in Windows Sound settings");
    println!("  - Update audio drivers if no devices found");
}

fn list_devices() -> Result<(), Box<dyn Error>> {
    initialize_mta()
        .map_err(|e| format!("Failed to initialize COM: {}", e))?;
    
    println!("=== ENHANCED WASAPI Loopback Device Enumerator ===");
    println!("Using comprehensive device detection with fallback strategies...\n");
    
    let enumerator = WASAPILoopbackEnumerator::new()
        .map_err(|e| format!("Failed to create enhanced enumerator: {}", e))?;
    
    // List all devices with detailed information
    if let Err(e) = enumerator.list_devices_detailed() {
        println!("❌ Error listing devices: {}", e);
        return Err(e);
    }
    
    // Auto-select best device
    match enumerator.auto_select_best_device()? {
        Some(device) => {
            println!("\n=== RECOMMENDED DEVICE ===");
            println!("Device: {}", device.name);
            println!("Type: {:?}", device.device_type);
            println!("Method: {:?}", device.loopback_method);
            println!("Format: {} Hz, {} channels, {}", 
                device.sample_rate, device.channels, device.format);
            println!("Default: {}", if device.is_default { "Yes" } else { "No" });
            
            println!("\n💡 This device should work with the audio capture tool!");
        }
        None => {
            println!("\n❌ No suitable loopback devices found!");
            println!("\n🔧 Troubleshooting Steps:");
            println!("1. Enable 'Stereo Mix' in Windows Sound Control Panel:");
            println!("   - Right-click sound icon → Open Sound settings");
            println!("   - Click 'Sound Control Panel' → Recording tab");
            println!("   - Right-click empty area → Show Disabled Devices");
            println!("   - Enable 'Stereo Mix' if available");
            println!();
            println!("2. Update your audio drivers");
            println!("3. Try running as Administrator");
            println!("4. Check Windows Audio service is running");
            println!();
            println!("Some audio drivers (especially newer ones) may not expose");
            println!("traditional loopback interfaces for security reasons.");
        }
    }
    
    Ok(())
}

fn simulate_transcription(model_path: &str) -> Result<(), Box<dyn Error>> {
    initialize_mta()
        .map_err(|e| format!("Failed to initialize COM: {}", e))?;
    
    println!("=== SIMULATED TRANSCRIPTION MODE ===");
    println!("Model path: {}", model_path);
    println!("Note: This is a simulation. For full transcription, implement whisper-rs integration.\n");
    
    // First, check if we have suitable devices
    let enumerator = WASAPILoopbackEnumerator::new()
        .map_err(|e| format!("Failed to create enhanced enumerator: {}", e))?;
    
    match enumerator.auto_select_best_device()? {
        Some(device) => {
            println!("✅ Found suitable audio device: {}", device.name);
            println!("   Type: {:?}", device.device_type);
            println!("   Method: {:?}", device.loopback_method);
            println!("   Format: {} Hz, {} channels", device.sample_rate, device.channels);
            
            println!("\n🎤 Would start audio capture from this device...");
            println!("🤖 Would process audio through Whisper model: {}", model_path);
            println!("📝 Would output real-time transcriptions...");
            
            println!("\n💡 To implement full transcription:");
            println!("1. Add whisper-rs dependency");
            println!("2. Load the Whisper model: {}", model_path);
            println!("3. Use the robust audio capture from audio_capture.rs");
            println!("4. Process audio chunks through Whisper");
            println!("5. Output transcriptions in real-time");
            
        }
        None => {
            println!("❌ No suitable audio devices found for transcription!");
            println!("Please run 'cargo run -- devices' to troubleshoot device detection.");
        }
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    
    // If no arguments provided, default to listing devices
    if args.len() < 2 {
        println!("No command specified, defaulting to device listing...\n");
        return list_devices();
    }
    
    match args[1].as_str() {
        "devices" => {
            list_devices()?;
        }
        "transcribe" => {
            let model_path = if args.len() > 2 {
                &args[2]
            } else {
                "models/ggml-tiny.bin"
            };
            
            simulate_transcription(model_path)?;
        }
        "help" | "--help" | "-h" => {
            show_help();
        }
        _ => {
            println!("Unknown command: {}", args[1]);
            println!("Use 'help' to see available commands.");
            show_help();
        }
    }
    
    Ok(())
}