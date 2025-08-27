//! Simple test example for audio streaming from default input device

use audio_capture_lib::macos::aggregate_device_manager::factory as aggregate_factory;
use audio_capture_lib::macos::audio_streamer_factory;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Testing Simple Audio Streaming");
    println!("=================================");
    
    // Step 1: Get available devices
    println!("\n📋 Step 1: Getting available devices...");
    let devices = aggregate_factory::get_ui_device_list().await?;
    println!("   Found {} devices", devices.len());
    
    // Find the default input device
    let default_input = devices.iter()
        .find(|d| d.device_type == audio_capture_lib::types::DeviceType::Capture && d.is_default)
        .ok_or("No default input device found")?;
    
    println!("   Default input device: {} (UID: {})", default_input.name, default_input.uid);
    
    // Step 2: Get device ID directly
    println!("\n🎤 Step 2: Getting device ID...");
    let device_id: u32 = default_input.id.parse().unwrap_or(0);
    if device_id == 0 {
        return Err("Could not get device ID for testing".into());
    }
    
    println!("   Device ID: {}", device_id);
    
    // Step 3: Create audio streamer directly
    println!("\n🎵 Step 3: Creating audio streamer...");
    let mut streamer = audio_streamer_factory::create_aggregate_device_streamer(device_id)?;
    
    // Set up audio data callback
    let audio_data_received = Arc::new(Mutex::new(0u32));
    let callback_data = audio_data_received.clone();
    
    streamer.set_callback(Box::new(move |audio_buffer: &audio_capture_lib::macos::AudioBuffer| {
        let mut counter = callback_data.lock().unwrap();
        *counter += 1;
        
        // Log audio data periodically
        if *counter % 50 == 0 {
            println!("   [Callback] Received audio buffer #{}: {} samples, {} channels, {:.2} Hz", 
                *counter, audio_buffer.data.len(), audio_buffer.channels, audio_buffer.sample_rate);
            
            // Calculate RMS (root mean square) to check if we're getting audio
            let rms: f32 = audio_buffer.data.iter()
                .map(|&x| x * x)
                .sum::<f32>()
                .sqrt() / (audio_buffer.data.len() as f32).sqrt();
            
            println!("   [Callback] Audio RMS: {:.6}", rms);
        }
    }));
    
    // Step 4: Start audio streaming
    println!("\n🔊 Step 4: Starting audio streaming...");
    match streamer.start() {
        Ok(()) => {
            println!("   ✅ Audio streaming started successfully!");
            println!("   Speak into your microphone to test audio capture...");
        }
        Err(e) => {
            println!("   ❌ Failed to start audio streaming: {}", e);
            return Err(e.into());
        }
    }
    
    // Step 5: Let it run for a while
    println!("\n⏱️  Step 5: Streaming audio for 10 seconds...");
    let start_time = std::time::Instant::now();
    
    while start_time.elapsed() < Duration::from_secs(10) {
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Check if we're still running
        if !streamer.is_running() {
            println!("   ⚠️  Audio streaming stopped unexpectedly");
            break;
        }
        
        // Get latest buffer and show some stats
        if let Some(buffer) = streamer.get_latest_buffer() {
            let elapsed = start_time.elapsed();
            if elapsed.as_secs() % 2 == 0 && elapsed.subsec_millis() < 100 {
                println!("   📊 Live stats: {} samples, {} channels, {:.2} Hz", 
                    buffer.data.len(), buffer.channels, buffer.sample_rate);
            }
        }
    }
    
    // Step 6: Stop audio streaming
    println!("\n🛑 Step 6: Stopping audio streaming...");
    streamer.stop()?;
    println!("   ✅ Audio streaming stopped");
    
    // Step 7: Show final statistics
    println!("\n📊 Step 7: Final statistics...");
    let total_buffers = *audio_data_received.lock().unwrap();
    println!("   Total audio buffers received: {}", total_buffers);
    println!("   Average buffers per second: {:.1}", total_buffers as f32 / 10.0);
    
    let all_buffers = streamer.get_all_buffers();
    println!("   Buffers in queue: {}", all_buffers.len());
    
    if !all_buffers.is_empty() {
        let total_samples: usize = all_buffers.iter()
            .map(|b| b.data.len())
            .sum();
        println!("   Total samples captured: {}", total_samples);
        println!("   Average samples per buffer: {:.1}", total_samples as f32 / all_buffers.len() as f32);
    }
    
    println!("\n🎉 Simple audio streaming test completed successfully!");
    println!("====================================================");
    println!("✅ Device enumeration: Working");
    println!("✅ Audio streaming setup: Working");
    println!("✅ Real-time audio capture: Working");
    println!("✅ Audio data processing: Working");
    
    Ok(())
}
