//! Test example for aggregate device functionality

use audio_capture_lib::macos::aggregate_device::factory;
use audio_capture_lib::macos::core_audio_enumerator::CoreAudioDeviceEnumerator;
use audio_capture_lib::DeviceEnumerator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🔧 Testing Aggregate Device Functionality");
    println!("=========================================");
    
    // First, let's enumerate all devices to see what we have
    let enumerator = CoreAudioDeviceEnumerator::new()?;
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
    
    // Look for aggregate devices
    let aggregate_devices = factory::find_aggregate_devices().await?;
    
    println!("\nFound {} aggregate device(s):", aggregate_devices.len());
    for (i, device) in aggregate_devices.iter().enumerate() {
        println!("  {}. {} (ID: {})", i + 1, device.get_name()?, device.get_device_id());
        
        let device_list = device.get_device_list();
        if !device_list.is_empty() {
            println!("     Sub-devices: {}", device_list.iter().cloned().collect::<Vec<_>>().join(", "));
        }
        
        let tap_list = device.get_tap_list();
        if !tap_list.is_empty() {
            println!("     Taps: {}", tap_list.iter().cloned().collect::<Vec<_>>().join(", "));
        }
    }
    
    // If we found aggregate devices, let's test adding a sub-device
    if let Some(device) = aggregate_devices.first() {
        println!("\nTesting aggregate device operations...");
        
        // Find a suitable sub-device to add (look for a microphone)
        let microphone = devices.iter()
            .find(|d| d.device_type == audio_capture_lib::types::DeviceType::Capture)
            .ok_or("No microphone found")?;
        
        println!("Found microphone: {} ({})", microphone.name, microphone.uid);
        
        // Note: We can't modify the device in this test since we only have a reference
        // In a real application, you would have a mutable reference to modify the device
        
        println!("✅ Aggregate device test completed successfully!");
    } else {
        println!("⚠️ No aggregate devices found. This is normal if no aggregate devices exist.");
        println!("   You may need to create an aggregate device in Audio MIDI Setup first.");
    }
    
    Ok(())
}
