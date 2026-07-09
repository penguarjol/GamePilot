use serde::Serialize;
use sysinfo::System;

#[derive(Debug, Clone, Serialize)]
pub struct HardwareInfo {
    pub cpu_model: String,
    pub cpu_cores: u32,
    pub cpu_threads: u32,
    pub cpu_usage_percent: f32,
    pub ram_total_mb: u64,
    pub ram_used_mb: u64,
    pub ram_available_mb: u64,
    pub gpu_model: String,
    pub gpu_vram_mb: u64,
    pub os_name: String,
    pub os_version: String,
    pub hostname: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: u32,
    pub cpu_percent: f32,
    pub ram_mb: f64,
    pub category: String,
    pub is_resource_hog: bool,
    pub recommendation: String,
}

const RESOURCE_HOG_SIGNATURES: &[(&str, &str, &str)] = &[
    ("chrome", "Browser — high RAM usage", "Consider closing unused tabs"),
    ("firefox", "Browser — high RAM usage", "Consider closing unused tabs"),
    ("msedge", "Browser — high RAM usage", "Consider closing unused tabs"),
    ("obs64", "Streaming/Recording — GPU/CPU overhead", "Pause recording if not needed"),
    ("obs", "Streaming/Recording — GPU/CPU overhead", "Pause recording if not needed"),
    ("discord", "Communication — potential GPU overhead if screen sharing", "Disable screen sharing during gameplay"),
    ("onedrive", "Cloud sync — disk contention", "Pause sync during gameplay"),
    ("dropbox", "Cloud sync — disk contention", "Pause sync during gameplay"),
    ("steam", "Game platform — potential download/update activity", "Pause downloads"),
    ("steamwebhelper", "Steam browser — RAM usage", "Close Steam browser tabs"),
    ("corsair", "RGB software — CPU polling", "Close during gameplay"),
    ("icue", "RGB software — CPU polling", "Close during gameplay"),
    ("razer", "RGB/peripheral software — CPU polling", "Close during gameplay"),
    ("wallpaper", "Wallpaper engine — GPU usage", "Pause during gameplay"),
    ("msmpeng", "Windows Defender scan", "Schedule scans for idle time"),
    ("antimalware", "Antivirus scan", "Schedule scans for idle time"),
];

pub fn collect_hardware_info() -> HardwareInfo {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu_model = sys
        .cpus()
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Unknown CPU".to_string());

    let cpu_cores = sys.physical_core_count().unwrap_or(0) as u32;
    let cpu_threads = sys.cpus().len() as u32;
    let cpu_usage = sys.global_cpu_info().cpu_usage();

    let ram_total_mb = sys.total_memory() / (1024 * 1024);
    let ram_used_mb = sys.used_memory() / (1024 * 1024);
    let ram_available_mb = sys.available_memory() / (1024 * 1024);

    HardwareInfo {
        cpu_model,
        cpu_cores,
        cpu_threads,
        cpu_usage_percent: cpu_usage,
        ram_total_mb,
        ram_used_mb,
        ram_available_mb,
        gpu_model: detect_gpu_model(),
        gpu_vram_mb: 0,
        os_name: System::name().unwrap_or_else(|| "Unknown".to_string()),
        os_version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
        hostname: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
    }
}

pub fn collect_process_info() -> Vec<ProcessInfo> {
    let mut sys = System::new_all();
    sys.refresh_all();
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_all();

    let mut processes: Vec<ProcessInfo> = sys
        .processes()
        .values()
        .map(|p| {
            let name = p.name().to_lowercase();
            let ram_mb = p.memory() as f64 / (1024.0 * 1024.0);
            let cpu = p.cpu_usage();

            let (category, is_hog, rec) = categorize_process(&name, ram_mb, cpu);

            ProcessInfo {
                name: p.name().to_string(),
                pid: p.pid().as_u32(),
                cpu_percent: cpu,
                ram_mb,
                category,
                is_resource_hog: is_hog,
                recommendation: rec,
            }
        })
        .filter(|p| p.ram_mb > 50.0 || p.cpu_percent > 1.0)
        .collect();

    processes.sort_by(|a, b| {
        b.ram_mb
            .partial_cmp(&a.ram_mb)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    processes.truncate(30);
    processes
}

fn categorize_process(name: &str, ram_mb: f64, cpu: f32) -> (String, bool, String) {
    for (pattern, category, recommendation) in RESOURCE_HOG_SIGNATURES {
        if name.contains(pattern) {
            let is_hog = ram_mb > 500.0 || cpu > 5.0;
            return (category.to_string(), is_hog, recommendation.to_string());
        }
    }

    if ram_mb > 1000.0 {
        return (
            "High memory usage".to_string(),
            true,
            "Consider closing if not needed during gameplay".to_string(),
        );
    }
    if cpu > 15.0 {
        return (
            "High CPU usage".to_string(),
            true,
            "May impact game performance".to_string(),
        );
    }

    ("System/Other".to_string(), false, String::new())
}

fn detect_gpu_model() -> String {
    #[cfg(target_os = "macos")]
    {
        "GPU detection available on Windows".to_string()
    }
    #[cfg(target_os = "windows")]
    {
        detect_gpu_windows()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        "Unknown GPU".to_string()
    }
}

#[cfg(target_os = "windows")]
fn detect_gpu_windows() -> String {
    use std::process::Command;
    let output = Command::new("wmic")
        .args(["path", "win32_videocontroller", "get", "name"])
        .output();
    match output {
        Ok(out) => {
            let text = String::from_utf8_lossy(&out.stdout);
            text.lines()
                .nth(1)
                .map(|l| l.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "Unknown GPU".to_string())
        }
        Err(_) => "Unknown GPU".to_string(),
    }
}
