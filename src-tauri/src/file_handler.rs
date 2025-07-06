use serde::{Deserialize, Serialize};
use base64::Engine;
use std::io::Cursor;
use image::{ImageFormat, ImageReader};
use pdf_extract;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileUploadResult {
    pub file_id: String,
    pub file_name: String,
    pub file_size: u64,
    pub mime_type: String,
    pub base64_data: String,
    pub thumbnail: Option<String>,
    pub extracted_text: Option<String>,
    pub dimensions: Option<FileDimensions>,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileDimensions {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileValidationConfig {
    pub max_file_size: u64, // 50MB default
    pub allowed_image_types: Vec<String>,
    pub allowed_document_types: Vec<String>,
    pub max_files_per_message: u32,
}

impl Default for FileValidationConfig {
    fn default() -> Self {
        Self {
            max_file_size: 50 * 1024 * 1024, // 50MB
            allowed_image_types: vec![
                "image/png".to_string(),
                "image/jpeg".to_string(),
                "image/jpg".to_string(),
                "image/gif".to_string(),
                "image/webp".to_string(),
                "image/bmp".to_string(),
            ],
            allowed_document_types: vec![
                "application/pdf".to_string(),
                "text/plain".to_string(),
                "text/markdown".to_string(),
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
                "application/msword".to_string(),
            ],
            max_files_per_message: 10,
        }
    }
}

#[tauri::command]
pub async fn upload_file_base64(
    file_name: String,
    file_data: String, // Base64 encoded
    mime_type: String,
) -> Result<FileUploadResult, String> {
    println!("📁 Processing file upload: {} ({})", file_name, mime_type);
    
    let config = FileValidationConfig::default();
    let file_id = format!("file_{}", uuid::Uuid::new_v4());
    
    // Decode base64 data
    let file_bytes = base64::engine::general_purpose::STANDARD
        .decode(&file_data)
        .map_err(|e| format!("Invalid base64 data: {}", e))?;
    
    let file_size = file_bytes.len() as u64;
    
    // Validate file size
    if file_size > config.max_file_size {
        return Ok(FileUploadResult {
            file_id,
            file_name,
            file_size,
            mime_type,
            base64_data: String::new(),
            thumbnail: None,
            extracted_text: None,
            dimensions: None,
            success: false,
            error: Some(format!("File size ({} bytes) exceeds maximum allowed size ({} bytes)", 
                file_size, config.max_file_size)),
        });
    }
    
    // Validate MIME type
    let is_image = config.allowed_image_types.contains(&mime_type);
    let is_document = config.allowed_document_types.contains(&mime_type);
    
    if !is_image && !is_document {
        return Ok(FileUploadResult {
            file_id,
            file_name,
            file_size,
            mime_type: mime_type.clone(),
            base64_data: String::new(),
            thumbnail: None,
            extracted_text: None,
            dimensions: None,
            success: false,
            error: Some(format!("Unsupported file type: {}", mime_type)),
        });
    }
    
    let mut result = FileUploadResult {
        file_id,
        file_name,
        file_size,
        mime_type: mime_type.clone(),
        base64_data: file_data,
        thumbnail: None,
        extracted_text: None,
        dimensions: None,
        success: true,
        error: None,
    };
    
    // Process images
    if is_image {
        match process_image(&file_bytes) {
            Ok((dimensions, thumbnail)) => {
                result.dimensions = Some(dimensions);
                result.thumbnail = thumbnail;
            }
            Err(e) => {
                println!("⚠️ Failed to process image {}: {}", result.file_name, e);
                result.error = Some(format!("Image processing error: {}", e));
            }
        }
    }
    
    // Process documents
    if is_document {
        match extract_document_text(&file_bytes, &mime_type) {
            Ok(text) => {
                result.extracted_text = Some(text);
            }
            Err(e) => {
                println!("⚠️ Failed to extract text from {}: {}", result.file_name, e);
                // Don't fail the upload for text extraction errors
                result.extracted_text = Some(format!("Text extraction failed: {}", e));
            }
        }
    }
    
    println!("✅ File processed successfully: {}", result.file_name);
    Ok(result)
}

#[tauri::command]
pub async fn validate_file_upload(
    file_size: u64,
    mime_type: String,
) -> Result<bool, String> {
    let config = FileValidationConfig::default();
    
    if file_size > config.max_file_size {
        return Err(format!("File size ({} bytes) exceeds maximum allowed size ({} bytes)", 
            file_size, config.max_file_size));
    }
    
    let is_supported = config.allowed_image_types.contains(&mime_type) || 
                      config.allowed_document_types.contains(&mime_type);
    
    if !is_supported {
        return Err(format!("Unsupported file type: {}", mime_type));
    }
    
    Ok(true)
}

#[tauri::command]
pub async fn get_file_upload_config() -> FileValidationConfig {
    FileValidationConfig::default()
}

fn process_image(image_data: &[u8]) -> Result<(FileDimensions, Option<String>), String> {
    let img = ImageReader::new(Cursor::new(image_data))
        .with_guessed_format()
        .map_err(|e| format!("Failed to read image: {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode image: {}", e))?;
    
    let dimensions = FileDimensions {
        width: img.width(),
        height: img.height(),
    };
    
    // Create thumbnail (max 200x200)
    let thumbnail = create_thumbnail(&img)?;
    
    Ok((dimensions, Some(thumbnail)))
}

fn create_thumbnail(img: &image::DynamicImage) -> Result<String, String> {
    const THUMBNAIL_SIZE: u32 = 200;
    
    let thumbnail = img.thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE);
    
    let mut thumbnail_data = Vec::new();
    thumbnail
        .write_to(&mut Cursor::new(&mut thumbnail_data), ImageFormat::Png)
        .map_err(|e| format!("Failed to create thumbnail: {}", e))?;
    
    let thumbnail_base64 = base64::engine::general_purpose::STANDARD.encode(&thumbnail_data);
    Ok(thumbnail_base64)
}

fn extract_document_text(file_data: &[u8], mime_type: &str) -> Result<String, String> {
    match mime_type {
        "application/pdf" => extract_pdf_text(file_data),
        "text/plain" | "text/markdown" => {
            String::from_utf8(file_data.to_vec())
                .map_err(|e| format!("Failed to decode text file: {}", e))
        }
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
            extract_docx_text(file_data)
        }
        _ => Err(format!("Unsupported document type for text extraction: {}", mime_type)),
    }
}

fn extract_pdf_text(pdf_data: &[u8]) -> Result<String, String> {
    match pdf_extract::extract_text_from_mem(pdf_data) {
        Ok(text) => {
            if text.trim().is_empty() {
                Err("PDF contains no extractable text".to_string())
            } else {
                Ok(text)
            }
        }
        Err(e) => Err(format!("Failed to extract PDF text: {}", e)),
    }
}

fn extract_docx_text(_docx_data: &[u8]) -> Result<String, String> {
    // For now, return a placeholder - DOCX text extraction would require additional dependencies
    // This could be enhanced with a proper DOCX parser
    Err("DOCX text extraction not yet implemented".to_string())
}

#[tauri::command]
pub async fn process_clipboard_image(image_base64: String) -> Result<FileUploadResult, String> {
    let _file_id = format!("clipboard_{}", uuid::Uuid::new_v4());
    let file_name = format!("clipboard_image_{}.png", chrono::Utc::now().timestamp());
    
    upload_file_base64(file_name, image_base64, "image/png".to_string()).await
}

// Add utility for cleanup if needed
#[tauri::command]
pub async fn cleanup_temp_files() -> Result<(), String> {
    // Implementation for cleaning up temporary files if we store them locally
    // For now, we're keeping everything in memory/base64
    Ok(())
} 