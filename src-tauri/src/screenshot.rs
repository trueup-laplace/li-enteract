use screenshots::Screen;
use base64::Engine;
use serde::{Deserialize, Serialize};

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
        
    // Get all screens
    let screens = Screen::all().ok_or("Failed to get screens")?;
        
    // Use the primary screen (first one)
    let screen = screens.into_iter().next()
        .ok_or("No screens found")?;
        
    println!("📸 Found screen: {}x{}", screen.display_info.width, screen.display_info.height);
        
    // Capture the screenshot
    let image = screen.capture().ok_or("Failed to capture screen")?;
        
    // Get the raw image data
    let width = image.width();
    let height = image.height();
    let buffer = image.buffer();
        
    // Create an ImageBuffer from the raw data (assuming RGBA format)
    let img_buffer = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(width, height, buffer.as_slice())
        .ok_or("Failed to create image buffer from raw data")?;
        
    // Convert to PNG bytes
    let mut png_data = Vec::new();
    img_buffer.write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode PNG: {}", e))?;
        
    // Encode to base64
    let base64_image = base64::engine::general_purpose::STANDARD.encode(&png_data);
        
    println!("📸 Screenshot captured: {}x{} ({} bytes)", width, height, png_data.len());
        
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
        
    // Get all screens
    let screens = Screen::all().ok_or("Failed to get screens")?;
        
    // Use the primary screen
    let screen = screens.into_iter().next()
        .ok_or("No screens found")?;
        
    // Capture the full screenshot first
    let full_image = screen.capture().ok_or("Failed to capture screen")?;
        
    // Get the raw image data
    let full_width = full_image.width();
    let full_height = full_image.height();
    let buffer = full_image.buffer();
        
    // Create an ImageBuffer from the raw data and clone it to own the data
    let img_buffer = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(full_width, full_height, buffer.to_vec())
        .ok_or("Failed to create image buffer from raw data")?;
        
    // Crop to the specified area - this now owns the data
    let cropped = image::imageops::crop_imm(&img_buffer, x as u32, y as u32, width, height);
        
    // Convert to PNG bytes
    let mut png_data = Vec::new();
    cropped.to_image().write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode PNG: {}", e))?;
        
    // Encode to base64
    let base64_image = base64::engine::general_purpose::STANDARD.encode(&png_data);
        
    println!("📸 Area screenshot captured: {}x{} ({} bytes)", width, height, png_data.len());
        
    Ok(ScreenshotResult {
        image_base64: base64_image,
        width,
        height,
        format: "png".to_string(),
    })
}