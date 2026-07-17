use serde::Serialize;
use xcap::Monitor;

#[derive(Debug, Clone, Serialize)]
pub struct CaptureResult {
    pub width: u32,
    pub height: u32,
    pub window_title: String,
    pub timestamp: String,
    pub has_image: bool,
}

/// Capture the primary monitor screen.
pub fn capture_screen() -> Result<(CaptureResult, Vec<u8>), String> {
    let monitors = Monitor::all().map_err(|e| format!("Monitor enumeration failed: {}", e))?;

    let monitor = monitors
        .into_iter()
        .next()
        .ok_or_else(|| "No monitors found".to_string())?;

    let image = monitor
        .capture_image()
        .map_err(|e| format!("Screen capture failed: {}", e))?;

    let width = image.width();
    let height = image.height();

    let mut png_bytes = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_bytes);
    let dynamic = xcap::image::DynamicImage::ImageRgba8(image);
    dynamic
        .write_to(&mut cursor, xcap::image::ImageFormat::Png)
        .map_err(|e| format!("PNG encode failed: {}", e))?;

    let monitor_name = monitor
        .name()
        .unwrap_or_else(|_| "Unknown".to_string());

    Ok((
        CaptureResult {
            width,
            height,
            window_title: monitor_name,
            timestamp: chrono::Utc::now().to_rfc3339(),
            has_image: true,
        },
        png_bytes,
    ))
}

/// Get the title of the foreground window (for game detection).
pub fn get_foreground_window_title() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        get_foreground_title_windows()
    }
    #[cfg(target_os = "macos")]
    {
        get_foreground_title_macos()
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        None
    }
}

#[cfg(target_os = "windows")]
fn get_foreground_title_windows() -> Option<String> {
    use std::os::windows::process::CommandExt;
    let output = std::process::Command::new("powershell")
        .args([
            "-Command",
            "(Get-Process | Where-Object {$_.MainWindowHandle -ne 0} | Sort-Object -Property CPU -Descending | Select-Object -First 1).MainWindowTitle",
        ])
        .creation_flags(0x08000000)
        .output()
        .ok()?;
    let title = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if title.is_empty() {
        None
    } else {
        Some(title)
    }
}

#[cfg(target_os = "macos")]
fn get_foreground_title_macos() -> Option<String> {
    let output = std::process::Command::new("osascript")
        .args([
            "-e",
            "tell application \"System Events\" to get name of first process whose frontmost is true",
        ])
        .output()
        .ok()?;
    let title = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if title.is_empty() {
        None
    } else {
        Some(title)
    }
}
