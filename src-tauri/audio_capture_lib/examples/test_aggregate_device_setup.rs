//! Test example for aggregate device setup with default microphone

use audio_capture_lib::macos::aggregate_device_manager::factory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Testing Aggregate Device Setup with Default Microphone");
    println!("=====================================================");
    
    // Get all available devices for UI selection
    println!("\n📋 Available devices for UI selection:");
    let ui_devices = factory::get_ui_device_list().await?;
    for (i, device) in ui_devices.iter().enumerate() {
        println!("  {}. {} (UID: {})", i + 1, device.name, device.uid);
        println!("     Type: {:?}, Default: {}", device.device_type, device.is_default);
    }
    
    // Get only input devices
    println!("\n🎤 Available input devices:");
    let input_devices = factory::get_ui_input_devices().await?;
    for (i, device) in input_devices.iter().enumerate() {
        println!("  {}. {} (UID: {})", i + 1, device.name, device.uid);
    }
    
    // Try to create an aggregate device with default microphone
    println!("\n🔗 Setting up aggregate device with default microphone...");
    match factory::create_microphone_aggregate_device().await {
        Ok(aggregate_device) => {
            println!("✅ Successfully created aggregate device!");
            println!("   Device ID: {}", aggregate_device.get_device_id());
            println!("   Name: {}", aggregate_device.get_name().unwrap_or_else(|_| "Unknown".to_string()));
            
            let config = aggregate_device.get_config();
            println!("   Sub-devices: {:?}", config.sub_devices);
            println!("   Taps: {:?}", config.taps);
            println!("   Private: {}", config.is_private);
            println!("   Auto-start: {}", config.auto_start);
        }
        Err(e) => {
            println!("❌ Failed to create aggregate device: {}", e);
            println!("   This is normal if no aggregate devices exist in Audio MIDI Setup.");
            println!("   Please create an aggregate device named 'Callio Microphone Aggregate' in Audio MIDI Setup first.");
        }
    }
    
    println!("\n✅ Test completed!");
    Ok(())
}
