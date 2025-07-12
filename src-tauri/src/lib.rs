// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

// Import our modules
mod transparency;
mod window_manager;
mod eye_tracking;
mod speech;
mod ollama;
mod screenshot;
mod file_handler;
mod data_store;

// Re-export the commands from modules
use transparency::{set_window_transparency, emergency_restore_window, toggle_transparency};
use window_manager::{
    move_window_to_position, get_window_position, get_window_size, get_screen_size,
    get_virtual_desktop_size, get_monitor_layout, set_window_bounds
};
use eye_tracking::{
    start_ml_eye_tracking, stop_ml_eye_tracking, get_ml_gaze_data, calibrate_ml_eye_tracking,
    get_ml_tracking_stats, pause_ml_tracking, resume_ml_tracking, detect_window_drag
};
use speech::{
    initialize_whisper_model, transcribe_audio_base64, transcribe_audio_file,
    check_whisper_model_availability, download_whisper_model, list_available_models
};
use ollama::{
    get_ollama_models, get_ollama_status, pull_ollama_model, delete_ollama_model,
    generate_ollama_response, generate_ollama_response_stream, get_ollama_model_info,
    generate_enteract_agent_response, generate_vision_analysis, generate_deep_research
};
use screenshot::{capture_screenshot, capture_screenshot_area};
use file_handler::{
    upload_file_base64, validate_file_upload, get_file_upload_config,
    process_clipboard_image, cleanup_temp_files
};
use data_store::{save_chat_sessions, load_chat_sessions};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Setup emergency global hotkey for transparency restore
            #[cfg(desktop)]
            {
                // Register global hotkey for emergency restore (Ctrl+Shift+Esc)
                // This ensures users can always regain control
                let _handle = app.handle().clone();
                
                // Note: Global hotkey registration would require additional dependencies
                // For now, we'll rely on window-level keyboard shortcuts
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Basic commands
            greet,
            
            // Transparency commands
            set_window_transparency, 
            emergency_restore_window,
            toggle_transparency,
            
            // Window management commands
            move_window_to_position,
            get_window_position,
            get_window_size,
            get_screen_size,
            get_virtual_desktop_size,
            get_monitor_layout,
            set_window_bounds,
            
            // Eye tracking commands
            start_ml_eye_tracking,
            stop_ml_eye_tracking,
            get_ml_gaze_data,
            calibrate_ml_eye_tracking,
            get_ml_tracking_stats,
            pause_ml_tracking,
            resume_ml_tracking,
            detect_window_drag,
            
            // Whisper transcription commands
            initialize_whisper_model,
            transcribe_audio_base64,
            transcribe_audio_file,
            check_whisper_model_availability,
            download_whisper_model,
            list_available_models,
            
            // Ollama commands
            get_ollama_models,
            get_ollama_status,
            pull_ollama_model,
            delete_ollama_model,
            generate_ollama_response,
            generate_ollama_response_stream,
            get_ollama_model_info,
            
            // Agent commands
            generate_enteract_agent_response,
            generate_vision_analysis,
            generate_deep_research,
            
            // Screenshot commands
            capture_screenshot,
            capture_screenshot_area,
            
            // File upload commands
            upload_file_base64,
            validate_file_upload,
            get_file_upload_config,
            process_clipboard_image,
            cleanup_temp_files,

            // Chat session commands
            save_chat_sessions,
            load_chat_sessions
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
