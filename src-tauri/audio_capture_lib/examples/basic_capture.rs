//! Basic example of using the audio capture library

use audio_capture_lib::{
    AudioCaptureManager, CaptureConfig, CaptureMethod,
    capture_engine::utils::{create_debug_audio_callback, create_debug_transcription_callback},
};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🎤 Audio Capture Library Example");
    println!("=================================");
    
    // Create capture manager
    let mut capture_manager = AudioCaptureManager::new();
    
    // Enumerate available devices
    let enumerator = audio_capture_lib::device_enumerator::create_device_enumerator()?;
    let devices = enumerator.enumerate_devices().await?;
    
    println!("\nAvailable audio devices:");
    for device in &devices {
        println!("  - {} ({}): {} at {}Hz", 
            device.name, 
            device.id, 
            device.format, 
            device.sample_rate
        );
    }
    
    // Find a suitable device for capture
    let capture_device = devices.iter()
        .find(|d| d.device_type == audio_capture_lib::types::DeviceType::Capture)
        .or_else(|| devices.first())
        .ok_or("No suitable audio device found")?;
    
    println!("\nSelected device: {} ({})", capture_device.name, capture_device.id);
    
    // Configure capture
    let config = CaptureConfig {
        device_id: capture_device.id.clone(),
        sample_rate: 16000,
        channels: 1,
        buffer_size: 4096,
        capture_method: CaptureMethod::Direct,
        enable_transcription: true,
        transcription_buffer_duration: 4.0,
        transcription_interval_ms: 800,
        min_audio_length: 1.5,
    };
    
    // Create callbacks
    let audio_callback = create_debug_audio_callback();
    let transcription_callback = create_debug_transcription_callback();
    
    println!("\nStarting audio capture...");
    println!("Press Ctrl+C to stop");
    
    // Start capture
    capture_manager.start_capture(
        config,
        Some(audio_callback),
        Some(transcription_callback),
    ).await?;
    
    // Wait for user to stop
    tokio::signal::ctrl_c().await?;
    
    println!("\nStopping audio capture...");
    capture_manager.stop_capture().await?;
    
    println!("Capture stopped successfully!");
    Ok(())
}
