use xcap::Monitor;
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenshotResult {
    pub image_base64: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
}

#[tauri::command]
pub async fn capture_screenshot() -> Result<ScreenshotResult, String> {
    println!("📸 Capturing screenshot...");
    
    // Get all monitors
    let monitors = Monitor::all().map_err(|e| format!("Failed to get monitors: {}", e))?;
    
    // Use the primary monitor or first one if no primary found
    let monitor = monitors
        .into_iter()
        .find(|m| m.is_primary().unwrap_or(false))
        .or_else(|| Monitor::all().ok()?.into_iter().next())
        .ok_or("No monitors found")?;
    
    println!("📸 Found monitor: {}x{}", 
        monitor.width().unwrap_or(0), 
        monitor.height().unwrap_or(0)
    );
    
    // Capture the screenshot
    let image = monitor.capture_image()
        .map_err(|e| format!("Failed to capture monitor: {}", e))?;
    
    let width = image.width();
    let height = image.height();
    
    println!("📸 Captured image: {}x{}", width, height);
    
    // Convert to PNG bytes
    let mut png_data = Vec::new();
    image.write_to(&mut Cursor::new(&mut png_data), xcap::image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode PNG: {}", e))?;
    
    // Encode to base64
    let base64_image = base64::engine::general_purpose::STANDARD.encode(&png_data);
    
    println!("✅ Screenshot captured successfully: {}x{}, {} bytes", width, height, png_data.len());
    
    Ok(ScreenshotResult {
        image_base64: base64_image,
        width,
        height,
        format: "png".to_string(),
    })
}

#[tauri::command]
pub async fn capture_screenshot_area(x: i32, y: i32, width: u32, height: u32) -> Result<ScreenshotResult, String> {
    println!("📸 Capturing screenshot area: {}x{} at ({}, {})", width, height, x, y);
    
    // Get all monitors
    let monitors = Monitor::all().map_err(|e| format!("Failed to get monitors: {}", e))?;
    
    // Find the monitor that contains the specified coordinates
    let monitor = monitors
        .into_iter()
        .find(|m| {
            if let (Ok(mx), Ok(my), Ok(mw), Ok(mh)) = (m.x(), m.y(), m.width(), m.height()) {
                x >= mx && y >= my && x < (mx + mw as i32) && y < (my + mh as i32)
            } else {
                false
            }
        })
        .or_else(|| Monitor::all().ok()?.into_iter().next())
        .ok_or("No suitable monitor found for the specified coordinates")?;
    
    // Convert coordinates to monitor-relative
    let monitor_x = monitor.x().unwrap_or(0);
    let monitor_y = monitor.y().unwrap_or(0);
    let relative_x = x - monitor_x;
    let relative_y = y - monitor_y;
    
    println!("📸 Using monitor at ({}, {}), relative capture at ({}, {})", 
        monitor_x, monitor_y, relative_x, relative_y);
    
    // Capture the specified region
    let image = monitor.capture_region(
        relative_x.max(0) as u32, 
        relative_y.max(0) as u32, 
        width, 
        height
    ).map_err(|e| format!("Failed to capture region: {}", e))?;
    
    let captured_width = image.width();
    let captured_height = image.height();
    
    println!("📸 Captured region: {}x{}", captured_width, captured_height);
    
    // Convert to PNG bytes
    let mut png_data = Vec::new();
    image.write_to(&mut Cursor::new(&mut png_data), xcap::image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode PNG: {}", e))?;
    
    // Encode to base64
    let base64_image = base64::engine::general_purpose::STANDARD.encode(&png_data);
    
    println!("✅ Screenshot area captured successfully: {}x{}, {} bytes", 
        captured_width, captured_height, png_data.len());
    
    Ok(ScreenshotResult {
        image_base64: base64_image,
        width: captured_width,
        height: captured_height,
        format: "png".to_string(),
    })
}