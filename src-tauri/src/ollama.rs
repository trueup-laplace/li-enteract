use serde::{Deserialize, Serialize};
use reqwest;
use std::collections::HashMap;
use tauri::{AppHandle, Emitter};
use futures_util::StreamExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
    pub digest: String,
    pub details: Option<ModelDetails>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelDetails {
    pub format: String,
    pub family: String,
    pub families: Option<Vec<String>>,
    pub parameter_size: String,
    pub quantization_level: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaModelsResponse {
    pub models: Vec<OllamaModel>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaStatus {
    pub status: String,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PullRequest {
    pub name: String,
    pub insecure: Option<bool>,
    pub stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    pub stream: Option<bool>,
    pub context: Option<Vec<i32>>,
    pub images: Option<Vec<String>>,
    pub system: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub model: String,
    pub created_at: String,
    pub response: String,
    pub done: bool,
    pub context: Option<Vec<i32>>,
    pub total_duration: Option<u64>,
    pub load_duration: Option<u64>,
    pub prompt_eval_count: Option<u32>,
    pub prompt_eval_duration: Option<u64>,
    pub eval_count: Option<u32>,
    pub eval_duration: Option<u64>,
}

const OLLAMA_BASE_URL: &str = "http://localhost:11434";

// System prompts for different agent types
const ENTERACT_AGENT_PROMPT: &str = r#"You are the Enteract Agent, a built-in private AI assistant integrated securely into the Enteract application. You are designed with zero security leaks and run completely locally.

Key principles:
- Prioritize user privacy and security above all else
- Provide helpful, accurate, and concise responses
- Focus on productivity and efficiency
- Never request external data or connections
- Always format responses in clean markdown
- Be direct and professional in your communication

You have access to the user's local system through the Enteract interface but maintain strict security boundaries. Respond helpfully while being mindful of security best practices."#;

const VISION_ANALYSIS_PROMPT: &str = r#"You are a specialized vision analysis agent. Analyze the provided screenshot/image with extreme detail and provide comprehensive insights.

Focus on:
- UI/UX elements and design patterns
- Text content and information hierarchy  
- Visual composition and layout
- Potential accessibility issues
- Areas for improvement or optimization
- Any notable patterns or anomalies

Provide your analysis in well-structured markdown format with clear headings and bullet points. Be thorough but organized in your response."#;

const DEEP_RESEARCH_PROMPT: &str = r#"You are a deep research specialist agent powered by advanced reasoning capabilities. Approach every query with systematic, thorough analysis.

Your approach:
- Break down complex problems into components
- Provide multi-layered analysis with reasoning chains
- Consider multiple perspectives and edge cases
- Offer evidence-based conclusions
- Structure responses with clear logical flow

Format all responses in comprehensive markdown with:
- Executive summary
- Detailed analysis sections
- Key findings and insights
- Actionable recommendations
- Supporting reasoning

Think step-by-step and show your reasoning process."#;

#[tauri::command]
pub async fn get_ollama_models() -> Result<Vec<OllamaModel>, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/tags", OLLAMA_BASE_URL);
    
    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<OllamaModelsResponse>().await {
                    Ok(models_response) => Ok(models_response.models),
                    Err(e) => Err(format!("Failed to parse models response: {}", e)),
                }
            } else {
                Err(format!("Ollama API error: {}", response.status()))
            }
        }
        Err(e) => Err(format!("Failed to connect to Ollama: {}. Make sure Ollama is running.", e)),
    }
}

#[tauri::command]
pub async fn get_ollama_status() -> Result<OllamaStatus, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/version", OLLAMA_BASE_URL);
    
    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<HashMap<String, String>>().await {
                    Ok(version_info) => Ok(OllamaStatus {
                        status: "running".to_string(),
                        version: version_info.get("version").cloned(),
                    }),
                    Err(_) => Ok(OllamaStatus {
                        status: "running".to_string(),
                        version: None,
                    }),
                }
            } else {
                Err(format!("Ollama API error: {}", response.status()))
            }
        }
        Err(_) => Ok(OllamaStatus {
            status: "not_running".to_string(),
            version: None,
        }),
    }
}

#[tauri::command]
pub async fn pull_ollama_model(model_name: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/pull", OLLAMA_BASE_URL);
    
    let request = PullRequest {
        name: model_name.clone(),
        insecure: Some(false),
        stream: Some(false),
    };
    
    match client.post(&url).json(&request).send().await {
        Ok(response) => {
            if response.status().is_success() {
                Ok(format!("Successfully started pulling model: {}", model_name))
            } else {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                Err(format!("Failed to pull model: {}", error_text))
            }
        }
        Err(e) => Err(format!("Failed to connect to Ollama: {}", e)),
    }
}

#[tauri::command]
pub async fn delete_ollama_model(model_name: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/delete", OLLAMA_BASE_URL);
    
    let request = serde_json::json!({
        "name": model_name
    });
    
    match client.delete(&url).json(&request).send().await {
        Ok(response) => {
            if response.status().is_success() {
                Ok(format!("Successfully deleted model: {}", model_name))
            } else {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                Err(format!("Failed to delete model: {}", error_text))
            }
        }
        Err(e) => Err(format!("Failed to connect to Ollama: {}", e)),
    }
}

#[tauri::command]
pub async fn generate_ollama_response(model: String, prompt: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/generate", OLLAMA_BASE_URL);
    
    let request = GenerateRequest {
        model,
        prompt,
        stream: Some(false),
        context: None,
        images: None,
        system: None,
    };
    
    match client.post(&url).json(&request).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<GenerateResponse>().await {
                    Ok(generate_response) => Ok(generate_response.response),
                    Err(e) => Err(format!("Failed to parse response: {}", e)),
                }
            } else {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                Err(format!("Generation failed: {}", error_text))
            }
        }
        Err(e) => Err(format!("Failed to connect to Ollama: {}", e)),
    }
}

#[tauri::command]
pub async fn generate_ollama_response_stream(
    app_handle: AppHandle,
    model: String,
    prompt: String,
    session_id: String,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/generate", OLLAMA_BASE_URL);
    
    let request = GenerateRequest {
        model: model.clone(),
        prompt: prompt.clone(),
        stream: Some(true),
        context: None,
        images: None,
        system: None,
    };
    
    println!("🚀 Starting streaming generation for session: {}", session_id);
    
    // Emit start event
    if let Err(e) = app_handle.emit(&format!("ollama-stream-{}", session_id), serde_json::json!({
        "type": "start",
        "model": model,
        "prompt": prompt
    })) {
        return Err(format!("Failed to emit start event: {}", e));
    }
    
    match client.post(&url).json(&request).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let mut stream = response.bytes_stream();
                let mut buffer = Vec::new();
                
                while let Some(chunk_result) = stream.next().await {
                    match chunk_result {
                        Ok(chunk) => {
                            buffer.extend_from_slice(&chunk);
                            
                            // Process complete lines from buffer
                            while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
                                let line = buffer.drain(..=newline_pos).collect::<Vec<u8>>();
                                let line_str = String::from_utf8_lossy(&line[..line.len()-1]); // Remove newline
                                
                                if line_str.trim().is_empty() {
                                    continue;
                                }
                                
                                // Parse JSON response
                                match serde_json::from_str::<GenerateResponse>(&line_str) {
                                    Ok(response_chunk) => {
                                        // Emit chunk event
                                        if let Err(e) = app_handle.emit(&format!("ollama-stream-{}", session_id), serde_json::json!({
                                            "type": "chunk",
                                            "text": response_chunk.response,
                                            "done": response_chunk.done
                                        })) {
                                            eprintln!("Failed to emit chunk event: {}", e);
                                        }
                                        
                                        // If done, break the loop
                                        if response_chunk.done {
                                            println!("✅ Streaming completed for session: {}", session_id);
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to parse streaming response: {} - Line: {}", e, line_str);
                                        continue;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            let error_msg = format!("Stream error: {}", e);
                            eprintln!("{}", error_msg);
                            
                            // Emit error event
                            if let Err(emit_err) = app_handle.emit(&format!("ollama-stream-{}", session_id), serde_json::json!({
                                "type": "error",
                                "error": error_msg
                            })) {
                                eprintln!("Failed to emit error event: {}", emit_err);
                            }
                            
                            return Err(error_msg);
                        }
                    }
                }
                
                // Emit completion event
                if let Err(e) = app_handle.emit(&format!("ollama-stream-{}", session_id), serde_json::json!({
                    "type": "complete"
                })) {
                    eprintln!("Failed to emit complete event: {}", e);
                }
                
                Ok(())
            } else {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                let error_msg = format!("Generation failed: {}", error_text);
                
                // Emit error event
                if let Err(e) = app_handle.emit(&format!("ollama-stream-{}", session_id), serde_json::json!({
                    "type": "error",
                    "error": error_msg
                })) {
                    eprintln!("Failed to emit error event: {}", e);
                }
                
                Err(error_msg)
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to connect to Ollama: {}", e);
            
            // Emit error event
            if let Err(emit_err) = app_handle.emit(&format!("ollama-stream-{}", session_id), serde_json::json!({
                "type": "error",
                "error": error_msg
            })) {
                eprintln!("Failed to emit error event: {}", emit_err);
            }
            
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn generate_enteract_agent_response(
    app_handle: AppHandle,
    prompt: String,
    session_id: String,
) -> Result<(), String> {
    let model = "gemma3:1b-it-qat".to_string();
    generate_agent_response_stream(app_handle, model, prompt, ENTERACT_AGENT_PROMPT.to_string(), session_id).await
}

#[tauri::command]
pub async fn generate_vision_analysis(
    app_handle: AppHandle,
    prompt: String,
    image_base64: String,
    session_id: String,
) -> Result<(), String> {
    let model = "qwen2.5vl".to_string();
    let full_prompt = format!("Screenshot Analysis Request:\n\n{}", prompt);
    
    generate_agent_response_stream_with_image(
        app_handle, 
        model, 
        full_prompt, 
        VISION_ANALYSIS_PROMPT.to_string(),
        image_base64,
        session_id
    ).await
}

#[tauri::command]
pub async fn generate_deep_research(
    app_handle: AppHandle,
    prompt: String,
    session_id: String,
) -> Result<(), String> {
    let model = "deepseek-r1".to_string();
    let full_prompt = format!("Deep Research Query:\n\n{}", prompt);
    
    generate_agent_response_stream(app_handle, model, full_prompt, DEEP_RESEARCH_PROMPT.to_string(), session_id).await
}

// Helper function for streaming with system prompt
async fn generate_agent_response_stream(
    app_handle: AppHandle,
    model: String,
    prompt: String,
    system_prompt: String,
    session_id: String,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/generate", OLLAMA_BASE_URL);
    
    let request = GenerateRequest {
        model: model.clone(),
        prompt,
        stream: Some(true),
        context: None,
        images: None,
        system: Some(system_prompt),
    };
    
    println!("🤖 Starting {} agent streaming for session: {}", model, session_id);
    
    // Emit start event
    if let Err(e) = app_handle.emit(&format!("ollama-stream-{}", session_id), serde_json::json!({
        "type": "start",
        "model": model,
        "agent_type": "enteract"
    })) {
        return Err(format!("Failed to emit start event: {}", e));
    }
    
    stream_ollama_response(app_handle, client, url, request, session_id).await
}

// Helper function for streaming with image
async fn generate_agent_response_stream_with_image(
    app_handle: AppHandle,
    model: String,
    prompt: String,
    system_prompt: String,
    image_base64: String,
    session_id: String,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/generate", OLLAMA_BASE_URL);
    
    let request = GenerateRequest {
        model: model.clone(),
        prompt,
        stream: Some(true),
        context: None,
        images: Some(vec![image_base64]),
        system: Some(system_prompt),
    };
    
    println!("👁️ Starting {} vision analysis for session: {}", model, session_id);
    
    // Emit start event
    if let Err(e) = app_handle.emit(&format!("ollama-stream-{}", session_id), serde_json::json!({
        "type": "start",
        "model": model,
        "agent_type": "vision"
    })) {
        return Err(format!("Failed to emit start event: {}", e));
    }
    
    stream_ollama_response(app_handle, client, url, request, session_id).await
}

// Shared streaming logic
async fn stream_ollama_response(
    app_handle: AppHandle,
    client: reqwest::Client,
    url: String,
    request: GenerateRequest,
    session_id: String,
) -> Result<(), String> {
    match client.post(&url).json(&request).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let mut stream = response.bytes_stream();
                let mut buffer = Vec::new();
                
                while let Some(chunk_result) = stream.next().await {
                    match chunk_result {
                        Ok(chunk) => {
                            buffer.extend_from_slice(&chunk);
                            
                            // Process complete lines from buffer
                            while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
                                let line = buffer.drain(..=newline_pos).collect::<Vec<u8>>();
                                let line_str = String::from_utf8_lossy(&line[..line.len()-1]);
                                
                                if line_str.trim().is_empty() {
                                    continue;
                                }
                                
                                match serde_json::from_str::<GenerateResponse>(&line_str) {
                                    Ok(response_chunk) => {
                                        if let Err(e) = app_handle.emit(&format!("ollama-stream-{}", session_id), serde_json::json!({
                                            "type": "chunk",
                                            "text": response_chunk.response,
                                            "done": response_chunk.done
                                        })) {
                                            eprintln!("Failed to emit chunk event: {}", e);
                                        }
                                        
                                        if response_chunk.done {
                                            println!("✅ Agent streaming completed for session: {}", session_id);
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to parse streaming response: {} - Line: {}", e, line_str);
                                        continue;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            let error_msg = format!("Stream error: {}", e);
                            eprintln!("{}", error_msg);
                            
                            if let Err(emit_err) = app_handle.emit(&format!("ollama-stream-{}", session_id), serde_json::json!({
                                "type": "error",
                                "error": error_msg
                            })) {
                                eprintln!("Failed to emit error event: {}", emit_err);
                            }
                            
                            return Err(error_msg);
                        }
                    }
                }
                
                // Emit completion event
                if let Err(e) = app_handle.emit(&format!("ollama-stream-{}", session_id), serde_json::json!({
                    "type": "complete"
                })) {
                    eprintln!("Failed to emit complete event: {}", e);
                }
                
                Ok(())
            } else {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                let error_msg = format!("Generation failed: {}", error_text);
                
                if let Err(e) = app_handle.emit(&format!("ollama-stream-{}", session_id), serde_json::json!({
                    "type": "error",
                    "error": error_msg
                })) {
                    eprintln!("Failed to emit error event: {}", e);
                }
                
                Err(error_msg)
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to connect to Ollama: {}", e);
            
            if let Err(emit_err) = app_handle.emit(&format!("ollama-stream-{}", session_id), serde_json::json!({
                "type": "error",
                "error": error_msg
            })) {
                eprintln!("Failed to emit error event: {}", emit_err);
            }
            
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn get_ollama_model_info(model_name: String) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/show", OLLAMA_BASE_URL);
    
    let request = serde_json::json!({
        "name": model_name
    });
    
    match client.post(&url).json(&request).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(model_info) => Ok(model_info),
                    Err(e) => Err(format!("Failed to parse model info: {}", e)),
                }
            } else {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                Err(format!("Failed to get model info: {}", error_text))
            }
        }
        Err(e) => Err(format!("Failed to connect to Ollama: {}", e)),
    }
} 