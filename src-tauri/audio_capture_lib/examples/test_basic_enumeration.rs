//! Basic test for device enumeration - just get device IDs

use audio_capture_lib::macos::core_audio_enumerator::CoreAudioDeviceEnumerator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🔧 Testing Basic Device Enumeration");
    println!("===================================");
    
    // Create device enumerator
    let enumerator = CoreAudioDeviceEnumerator::new()?;
    
    // Just get the device IDs first
    let device_ids = enumerator.get_audio_device_ids()?;
    
    println!("\nFound {} device ID(s):", device_ids.len());
    for (i, device_id) in device_ids.iter().enumerate() {
        println!("  {}. Device ID: {}", i + 1, device_id);
    }
    
    println!("\n✅ Basic device enumeration test completed successfully!");
    
    Ok(())
}
