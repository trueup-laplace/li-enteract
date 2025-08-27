//! Test example for working with existing aggregate devices

use audio_capture_lib::macos::aggregate_device_manager::factory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Testing Existing Aggregate Device Usage");
    println!("==========================================");
    
    // Get all available devices for UI selection
    println!("\n📋 Available devices for UI selection:");
    let ui_devices = factory::get_ui_device_list().await?;
    for (i, device) in ui_devices.iter().enumerate() {
        println!("  {}. {} (UID: {})", i + 1, device.name, device.uid);
        println!("     Type: {:?}, Default: {}", device.device_type, device.is_default);
    }
    
    // Look for existing aggregate devices
    println!("\n🔗 Looking for existing aggregate devices...");
    let aggregate_devices = factory::get_ui_device_list().await?;
    let aggregate_devices: Vec<_> = aggregate_devices.iter()
        .filter(|d| d.device_type == audio_capture_lib::types::DeviceType::Aggregate)
        .collect();
    
    if aggregate_devices.is_empty() {
        println!("❌ No aggregate devices found.");
        println!("\n📝 To create an aggregate device:");
        println!("   1. Open Audio MIDI Setup (Applications > Utilities > Audio MIDI Setup)");
        println!("   2. Click the '+' button at the bottom left");
        println!("   3. Select 'Create Aggregate Device'");
        println!("   4. Name it 'Callio Test Aggregate'");
        println!("   5. Add your microphone (Audio Device 91) to the aggregate device");
        println!("   6. Save the configuration");
        println!("\n   Then run this test again.");
    } else {
        println!("✅ Found {} existing aggregate device(s):", aggregate_devices.len());
        for (i, device) in aggregate_devices.iter().enumerate() {
            println!("  {}. {} (UID: {})", i + 1, device.name, device.uid);
        }
        
        // Try to work with the first aggregate device
        if let Some(aggregate_device) = aggregate_devices.first() {
            println!("\n🔧 Testing with aggregate device: {}", aggregate_device.name);
            
            // Try to create a custom aggregate device with this name
            match factory::create_custom_aggregate_device(
                &aggregate_device.name,
                "device_91", // Default microphone UID
                None, // No output device
            ).await {
                Ok(device) => {
                    println!("✅ Successfully worked with aggregate device!");
                    println!("   Device ID: {}", device.get_device_id());
                    println!("   Name: {}", device.get_name().unwrap_or_else(|_| "Unknown".to_string()));
                    
                    let config = device.get_config();
                    println!("   Sub-devices: {:?}", config.sub_devices);
                    println!("   Taps: {:?}", config.taps);
                    println!("   Private: {}", config.is_private);
                    println!("   Auto-start: {}", config.auto_start);
                }
                Err(e) => {
                    println!("❌ Failed to work with aggregate device: {}", e);
                }
            }
        }
    }
    
    println!("\n✅ Test completed!");
    Ok(())
}
