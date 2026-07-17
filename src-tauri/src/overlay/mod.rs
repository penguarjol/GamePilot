use tauri::Manager;

pub fn toggle_overlay(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("overlay") {
        if window.is_visible().unwrap_or(false) {
            window.hide().map_err(|e| format!("Hide failed: {}", e))?;
        } else {
            window.show().map_err(|e| format!("Show failed: {}", e))?;
        }
    }
    Ok(())
}

pub fn show_overlay(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("overlay") {
        window.show().map_err(|e| format!("Show failed: {}", e))?;
    }
    Ok(())
}

pub fn hide_overlay(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("overlay") {
        window.hide().map_err(|e| format!("Hide failed: {}", e))?;
    }
    Ok(())
}
