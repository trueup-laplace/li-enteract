use serde::{Deserialize, Serialize};
use reqwest;
use std::collections::HashMap;

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