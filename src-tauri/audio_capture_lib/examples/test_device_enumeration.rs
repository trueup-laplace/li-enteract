//! Test example for device enumeration functionality

use audio_capture_lib::macos::core_audio_enumerator::CoreAudioDeviceEnumerator;
use audio_capture_lib::DeviceEnumerator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🔧 Testing Device Enumeration");
    println!("=============================");
    
    // Create device enumerator
    let enumerator = CoreAudioDeviceEnumerator::new()?;
    let devices = enumerator.enumerate_devices().await?;
    
    println!("\nFound {} audio device(s):", devices.len());
    for (i, device) in devices.iter().enumerate() {
        println!("  {}. {} (ID: {})", i + 1, device.name, device.id);
        println!("     UID: {}", device.uid);
        println!("     Type: {:?}", device.device_type);
        println!("     Sample Rate: {}Hz", device.sample_rate);
        println!("     Channels: {}", device.channels);
        println!("     Format: {}", device.format);
        println!("     Default: {}", device.is_default);
        println!("     Capture Method: {:?}", device.capture_method);
        println!();
    }
    
    // Look for specific device types
    let input_devices: Vec<_> = devices.iter()
        .filter(|d| d.device_type == audio_capture_lib::types::DeviceType::Capture)
        .collect();
    
    let output_devices: Vec<_> = devices.iter()
        .filter(|d| d.device_type == audio_capture_lib::types::DeviceType::Render)
        .collect();
    
    let aggregate_devices: Vec<_> = devices.iter()
        .filter(|d| d.device_type == audio_capture_lib::types::DeviceType::Aggregate)
        .collect();
    
    println!("Device Summary:");
    println!("  Input devices: {}", input_devices.len());
    println!("  Output devices: {}", output_devices.len());
    println!("  Aggregate devices: {}", aggregate_devices.len());
    
    if !input_devices.is_empty() {
        println!("\nInput devices:");
        for device in input_devices {
            println!("  - {} ({})", device.name, device.uid);
        }
    }
    
    if !output_devices.is_empty() {
        println!("\nOutput devices:");
        for device in output_devices {
            println!("  - {} ({})", device.name, device.uid);
        }
    }
    
    if !aggregate_devices.is_empty() {
        println!("\nAggregate devices:");
        for device in aggregate_devices {
            println!("  - {} ({})", device.name, device.uid);
        }
    }
    
    println!("\n✅ Device enumeration test completed successfully!");
    
    Ok(())
}
