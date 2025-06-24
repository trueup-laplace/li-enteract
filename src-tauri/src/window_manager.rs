use tauri::Window;
use tauri::{PhysicalPosition, PhysicalSize};

#[tauri::command]
pub async fn move_window_to_position(window: Window, x: i32, y: i32) -> Result<(), String> {
    let position = PhysicalPosition::new(x, y);
    window.set_position(position).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn get_window_position(window: Window) -> Result<(i32, i32), String> {
    let position = window.outer_position().map_err(|e| e.to_string())?;
    Ok((position.x, position.y))
}

#[tauri::command]
pub async fn get_window_size(window: Window) -> Result<(u32, u32), String> {
    let size = window.outer_size().map_err(|e| e.to_string())?;
    Ok((size.width, size.height))
}

#[tauri::command]
pub async fn get_screen_size() -> Result<(u32, u32), String> {
    // Get primary monitor size
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
        
        unsafe {
            let width = GetSystemMetrics(SM_CXSCREEN) as u32;
            let height = GetSystemMetrics(SM_CYSCREEN) as u32;
            return Ok((width, height));
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        use core_graphics::display::CGMainDisplay;
        
        let display = CGMainDisplay();
        let width = display.pixels_wide() as u32;
        let height = display.pixels_high() as u32;
        return Ok((width, height));
    }
    
    #[cfg(target_os = "linux")]
    {
        // For Linux, we'll return a default size
        // In a production app, you'd want to query the actual display
        return Ok((1920, 1080));
    }
}

#[tauri::command]
pub async fn get_virtual_desktop_size() -> Result<(u32, u32), String> {
    // Get full virtual desktop size (all monitors combined)
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN};
        
        unsafe {
            let width = GetSystemMetrics(SM_CXVIRTUALSCREEN) as u32;
            let height = GetSystemMetrics(SM_CYVIRTUALSCREEN) as u32;
            println!("🖥️ Virtual desktop detected: {}x{}", width, height);
            return Ok((width, height));
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // For macOS, sum up all displays
        use core_graphics::display::{CGDisplay, CGDisplayBounds};
        
        let displays = CGDisplay::active_displays()
            .map_err(|e| format!("Failed to get displays: {:?}", e))?;
        
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        
        for display in displays {
            let bounds = CGDisplayBounds(display);
            min_x = min_x.min(bounds.origin.x);
            min_y = min_y.min(bounds.origin.y);
            max_x = max_x.max(bounds.origin.x + bounds.size.width);
            max_y = max_y.max(bounds.origin.y + bounds.size.height);
        }
        
        let width = (max_x - min_x) as u32;
        let height = (max_y - min_y) as u32;
        return Ok((width, height));
    }
    
    #[cfg(target_os = "linux")]
    {
        // For Linux, fall back to primary display
        return get_screen_size().await;
    }
}

#[tauri::command]
pub async fn set_window_bounds(window: Window, x: i32, y: i32, width: u32, height: u32) -> Result<(), String> {
    let position = PhysicalPosition::new(x, y);
    let size = PhysicalSize::new(width, height);
    
    window.set_position(position).map_err(|e| e.to_string())?;
    window.set_size(size).map_err(|e| e.to_string())?;
    
    Ok(())
} 