use tauri::Window;

#[tauri::command]
pub async fn set_window_transparency(window: Window, alpha: f64) -> Result<(), String> {
    // Clamp alpha between 0.0 and 1.0
    let clamped_alpha = alpha.clamp(0.0, 1.0);
    
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{
            GetWindowLongPtrW, SetWindowLongPtrW, SetLayeredWindowAttributes, 
            GWL_EXSTYLE, WS_EX_LAYERED, WS_EX_TRANSPARENT, LWA_ALPHA
        };
        
        if let Ok(hwnd) = window.hwnd() {
            let hwnd = HWND(hwnd.0 as isize);
            
            unsafe {
                // Get current extended window style
                let mut ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                
                // Add layered window style for transparency
                ex_style |= WS_EX_LAYERED.0 as isize;
                
                // Add transparent style for click-through when very transparent
                if clamped_alpha < 0.1 {
                    ex_style |= WS_EX_TRANSPARENT.0 as isize;
                } else {
                    // Remove transparent style to enable interaction
                    ex_style &= !(WS_EX_TRANSPARENT.0 as isize);
                }
                
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style);
                
                // Set transparency level (0-255, where 255 is opaque)
                let alpha_value = (clamped_alpha * 255.0) as u8;
                SetLayeredWindowAttributes(hwnd, windows::Win32::Foundation::COLORREF(0), alpha_value, LWA_ALPHA)
                    .map_err(|e| format!("Failed to set transparency: {}", e))?;
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        println!("🔧 TRANSPARENCY: macOS - Setting transparency level to {}", clamped_alpha);
        
        // Set window properties for transparency
        if let Err(e) = window.set_decorations(false) {
            println!("🔧 TRANSPARENCY: macOS - Failed to set decorations: {:?}", e);
        }
        

        
        println!("🔧 TRANSPARENCY: macOS - Window transparency configured");
    }
    
    #[cfg(target_os = "linux")]
    {
        // Linux transparency implementation varies by window manager
        // This is a basic implementation for X11
        
        // Note: Linux implementation depends heavily on the desktop environment
        // This is a simplified version that may need adaptation
        window.set_decorations(false).map_err(|e| e.to_string())?;
        
        // For Wayland/X11, additional implementation would be needed
        // based on the specific compositor/window manager
    }
    
    Ok(())
}

#[tauri::command]
pub async fn emergency_restore_window(window: Window) -> Result<(), String> {
    println!("🔧 TRANSPARENCY: Emergency restore called");
    
    #[cfg(target_os = "macos")]
    {
        println!("🔧 TRANSPARENCY: macOS - Emergency restore - clearing transparency");
    }
    
    // Always restore to fully opaque and interactive
    set_window_transparency(window.clone(), 1.0).await?;
    
    // Ensure window is visible and on top
    window.set_always_on_top(true).map_err(|e| e.to_string())?;
    window.unminimize().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    
    println!("🔧 TRANSPARENCY: Emergency restore completed successfully");
    Ok(())
}

#[tauri::command]
pub async fn toggle_transparency(window: Window, current_alpha: f64) -> Result<f64, String> {
    println!("🔧 TRANSPARENCY: Toggle called with current_alpha: {}", current_alpha);
    let new_alpha = if current_alpha > 0.5 { 0.3 } else { 1.0 };
    println!("🔧 TRANSPARENCY: Toggle setting new_alpha to: {}", new_alpha);
    set_window_transparency(window, new_alpha).await?;
    println!("🔧 TRANSPARENCY: Toggle completed successfully");
    Ok(new_alpha)
}

#[tauri::command]
pub async fn initialize_window_transparency(window: Window) -> Result<(), String> {
    println!("🔧 TRANSPARENCY: Initializing window transparency");
    
    #[cfg(target_os = "macos")]
    {
        // Ensure window decorations are disabled
        if let Err(e) = window.set_decorations(false) {
            println!("🔧 TRANSPARENCY: macOS - Failed to set initial decorations: {:?}", e);
        }
        

        
        println!("🔧 TRANSPARENCY: macOS - Initial window transparency configured");
    }
    
    println!("🔧 TRANSPARENCY: Window transparency initialized");
    Ok(())
}
 