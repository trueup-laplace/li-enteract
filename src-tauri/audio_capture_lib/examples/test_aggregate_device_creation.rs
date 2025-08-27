//! Test example for aggregate device creation only

use audio_capture_lib::macos::aggregate_device_creator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Testing Aggregate Device Creation");
    println!("====================================");
    
    // Test 1: Create a simple aggregate device
    println!("\n🎤 Test 1: Creating simple aggregate device...");
    match aggregate_device_creator::create_simple_aggregate_device("Test Aggregate Device") {
        Ok(device_id) => {
            println!("   ✅ Successfully created aggregate device with ID: {}", device_id);
        }
        Err(e) => {
            println!("   ❌ Failed to create aggregate device: {}", e);
        }
    }
    
    // Test 2: Create a microphone aggregate device
    println!("\n🎤 Test 2: Creating microphone aggregate device...");
    match aggregate_device_creator::create_microphone_aggregate_device("Callio Microphone Aggregate") {
        Ok(device_id) => {
            println!("   ✅ Successfully created microphone aggregate device with ID: {}", device_id);
        }
        Err(e) => {
            println!("   ❌ Failed to create microphone aggregate device: {}", e);
        }
    }
    
    // Test 3: Create an "All Audio" aggregate device
    println!("\n🎤 Test 3: Creating All Audio aggregate device...");
    match aggregate_device_creator::create_all_audio_aggregate_device("Callio All Audio Aggregate") {
        Ok(device_id) => {
            println!("   ✅ Successfully created All Audio aggregate device with ID: {}", device_id);
        }
        Err(e) => {
            println!("   ❌ Failed to create All Audio aggregate device: {}", e);
        }
    }
    
    println!("\n🎉 Aggregate device creation test completed!");
    println!("=============================================");
    
    Ok(())
}
