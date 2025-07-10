use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: i32,
    pub text: String,
    pub sender: String, // 'user' | 'assistant' | 'transcription' | 'system'
    pub timestamp: String, // ISO 8601 string
    pub is_interim: Option<bool>,
    pub confidence: Option<f64>,
    pub source: Option<String>,
    pub attachments: Option<Vec<MessageAttachment>>,
    pub thinking: Option<ThinkingProcess>,
    #[serde(rename = "messageType")]
    pub message_type: Option<String>,
    pub metadata: Option<MessageMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAttachment {
    pub id: String,
    #[serde(rename = "type")]
    pub attachment_type: String,
    pub name: String,
    pub size: i64,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub url: Option<String>,
    #[serde(rename = "base64Data")]
    pub base64_data: Option<String>,
    pub thumbnail: Option<String>,
    #[serde(rename = "extractedText")]
    pub extracted_text: Option<String>,
    pub dimensions: Option<FileDimensions>,
    #[serde(rename = "uploadProgress")]
    pub upload_progress: Option<i32>,
    #[serde(rename = "uploadStatus")]
    pub upload_status: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDimensions {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingProcess {
    #[serde(rename = "isVisible")]
    pub is_visible: bool,
    pub content: String,
    #[serde(rename = "isStreaming")]
    pub is_streaming: bool,
    pub steps: Option<Vec<ThinkingStep>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingStep {
    pub id: String,
    pub title: String,
    pub content: String,
    pub timestamp: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    #[serde(rename = "agentType")]
    pub agent_type: Option<String>,
    pub model: Option<String>,
    pub tokens: Option<i32>,
    #[serde(rename = "processingTime")]
    pub processing_time: Option<f64>,
    #[serde(rename = "analysisType")]
    pub analysis_type: Option<Vec<String>>,
    #[serde(rename = "searchQueries")]
    pub search_queries: Option<Vec<String>>,
    pub sources: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub title: String,
    pub history: Vec<ChatMessage>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(rename = "modelId")]
    pub model_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveChatsPayload {
    pub chats: Vec<ChatSession>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoadChatsResponse {
    pub chats: Vec<ChatSession>,
}

fn get_chats_file_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app_handle   
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    // Ensure the directory exists
    if !app_data_dir.exists() {
        fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to create app data directory: {}", e))?;
    }

    Ok(app_data_dir.join("user_chat_sessions.json"))
}

#[tauri::command]
pub fn save_chat_sessions(
    app_handle: AppHandle,
    payload: SaveChatsPayload,
) -> Result<(), String> {
    let file_path = get_chats_file_path(&app_handle)?;
    
    let json_content = serde_json::to_string_pretty(&payload.chats)
        .map_err(|e| format!("Failed to serialize chat sessions: {}", e))?;

    fs::write(&file_path, json_content)
        .map_err(|e| format!("Failed to write chat sessions file: {}", e))?;

    println!("Chat sessions saved to: {:?}", file_path);
    Ok(())
}

#[tauri::command]
pub fn load_chat_sessions(app_handle: AppHandle) -> Result<LoadChatsResponse, String> {
    let file_path = get_chats_file_path(&app_handle)?;

    if !file_path.exists() {
        println!("Chat sessions file does not exist, returning empty list");
        return Ok(LoadChatsResponse {
            chats: Vec::new(),
        });
    }

    let file_content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read chat sessions file: {}", e))?;

    let chats: Vec<ChatSession> = serde_json::from_str(&file_content)
        .map_err(|e| format!("Failed to deserialize chat sessions: {}", e))?;

    println!("Loaded {} chat sessions from: {:?}", chats.len(), file_path);
    Ok(LoadChatsResponse { chats })
} 