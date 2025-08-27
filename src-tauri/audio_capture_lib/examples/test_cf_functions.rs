//! Test example for Core Foundation functions

use audio_capture_lib::macos::core_audio_bindings::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Testing Core Foundation Functions");
    println!("====================================");
    
    // Test 1: Create a simple CFString
    println!("\n📝 Test 1: Creating CFString...");
    match create_cf_string("Test String") {
        Ok(cf_string) => {
            println!("   ✅ Successfully created CFString: {:p}", cf_string);
        }
        Err(e) => {
            println!("   ❌ Failed to create CFString: {}", e);
            return Ok(());
        }
    }
    
    // Test 2: Create a CFNumber
    println!("\n🔢 Test 2: Creating CFNumber...");
    match create_cf_number_int(42) {
        Ok(cf_number) => {
            println!("   ✅ Successfully created CFNumber: {:p}", cf_number);
        }
        Err(e) => {
            println!("   ❌ Failed to create CFNumber: {}", e);
            return Ok(());
        }
    }
    
    // Test 3: Create a CFDictionary
    println!("\n📚 Test 3: Creating CFDictionary...");
    let cf_name = match create_cf_string("Test Device") {
        Ok(s) => s,
        Err(e) => {
            println!("   ❌ Failed to create CFString for name: {}", e);
            return Ok(());
        }
    };
    
    let cf_uid = match create_cf_string("test.device.123") {
        Ok(s) => s,
        Err(e) => {
            println!("   ❌ Failed to create CFString for UID: {}", e);
            return Ok(());
        }
    };
    
    let pairs = vec![
        (AUDIO_AGGREGATE_DEVICE_NAME_KEY, cf_name),
        (AUDIO_AGGREGATE_DEVICE_UID_KEY, cf_uid),
    ];
    
    match create_cf_dictionary_from_pairs(&pairs) {
        Ok(cf_dict) => {
            println!("   ✅ Successfully created CFDictionary: {:p}", cf_dict);
        }
        Err(e) => {
            println!("   ❌ Failed to create CFDictionary: {}", e);
            return Ok(());
        }
    }
    
    // Test 4: Try to create an aggregate device with the CFDictionary
    println!("\n🔗 Test 4: Creating Aggregate Device...");
    let cf_name = match create_cf_string("Test Aggregate Device") {
        Ok(s) => s,
        Err(e) => {
            println!("   ❌ Failed to create CFString for name: {}", e);
            return Ok(());
        }
    };
    
    let cf_uid = match create_cf_string("test.aggregate.device.123") {
        Ok(s) => s,
        Err(e) => {
            println!("   ❌ Failed to create CFString for UID: {}", e);
            return Ok(());
        }
    };
    
    let pairs = vec![
        (AUDIO_AGGREGATE_DEVICE_NAME_KEY, cf_name),
        (AUDIO_AGGREGATE_DEVICE_UID_KEY, cf_uid),
    ];
    
    let cf_description = match create_cf_dictionary_from_pairs(&pairs) {
        Ok(dict) => dict,
        Err(e) => {
            println!("   ❌ Failed to create CFDictionary for description: {}", e);
            return Ok(());
        }
    };
    
    let mut device_id: AudioAggregateDeviceID = 0;
    let status = unsafe {
        AudioHardwareCreateAggregateDevice(
            cf_description,
            &mut device_id,
        )
    };
    
    if status == NO_ERR {
        println!("   ✅ Successfully created aggregate device: {}", device_id);
    } else {
        println!("   ❌ Failed to create aggregate device: {}", status);
    }
    
    println!("\n✅ Core Foundation function tests completed!");
    Ok(())
}
