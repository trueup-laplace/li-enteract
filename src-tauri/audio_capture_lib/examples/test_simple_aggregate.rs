//! Simple test for aggregate device creation

use audio_capture_lib::macos::aggregate_device_creator::create_simple_aggregate_device;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Testing Simple Aggregate Device Creation");
    println!("===========================================");
    
    // Test creating a simple aggregate device
    println!("\n📝 Creating simple aggregate device...");
    match create_simple_aggregate_device("Test Simple Device") {
        Ok(device_id) => {
            println!("   ✅ Successfully created aggregate device: {}", device_id);
        }
        Err(e) => {
            println!("   ❌ Failed to create aggregate device: {}", e);
        }
    }
    
    println!("\n✅ Simple aggregate device test completed!");
    Ok(())
}
