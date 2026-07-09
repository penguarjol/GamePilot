use serde::Serialize;
use sysinfo::{Disks, System};

#[derive(Debug, Clone, Serialize)]
pub struct HardwareInfo {
    pub cpu_model: String,
    pub cpu_cores: u32,
    pub cpu_threads: u32,
    pub cpu_usage_percent: f32,
    pub cpu_freq_mhz: u64,
    pub ram_total_mb: u64,
    pub ram_used_mb: u64,
    pub ram_available_mb: u64,
    pub gpu_model: String,
    pub gpu_vram_mb: u64,
    pub gpu_driver_version: String,
    pub disks: Vec<DiskInfo>,
    pub os_name: String,
    pub os_version: String,
    pub hostname: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_gb: f64,
    pub free_gb: f64,
    pub is_removable: bool,
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

#[derive(Debug, Clone, Serialize)]
pub struct TelemetrySample {
    pub timestamp: String,
    pub cpu_percent: f32,
    pub ram_used_mb: u64,
    pub ram_available_mb: u64,
    pub top_processes: Vec<ProcessSnapshot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessSnapshot {
    pub name: String,
    pub pid: u32,
    pub cpu_percent: f32,
    pub ram_mb: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SelfMetrics {
    pub cpu_percent: f32,
    pub ram_mb: f64,
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
    ("nvidiacontainer", "NVIDIA telemetry — CPU overhead", "Can be disabled if not using GeForce Experience"),
    ("lghub", "Logitech Hub — peripheral software", "Close during gameplay if not needed"),
    ("epicgameslauncher", "Epic Games — background updates", "Pause downloads"),
    ("battlenet", "Battle.net — background updates", "Pause downloads"),
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
    let cpu_freq = sys.cpus().first().map(|c| c.frequency()).unwrap_or(0);

    let ram_total_mb = sys.total_memory() / (1024 * 1024);
    let ram_used_mb = sys.used_memory() / (1024 * 1024);
    let ram_available_mb = sys.available_memory() / (1024 * 1024);

    let disks_info = Disks::new_with_refreshed_list();
    let disks: Vec<DiskInfo> = disks_info
        .iter()
        .map(|d| DiskInfo {
            name: d.name().to_string_lossy().to_string(),
            mount_point: d.mount_point().to_string_lossy().to_string(),
            total_gb: d.total_space() as f64 / (1024.0 * 1024.0 * 1024.0),
            free_gb: d.available_space() as f64 / (1024.0 * 1024.0 * 1024.0),
            is_removable: d.is_removable(),
        })
        .collect();

    let (gpu_model, gpu_vram, gpu_driver) = detect_gpu_details();

    HardwareInfo {
        cpu_model,
        cpu_cores,
        cpu_threads,
        cpu_usage_percent: cpu_usage,
        cpu_freq_mhz: cpu_freq,
        ram_total_mb,
        ram_used_mb,
        ram_available_mb,
        gpu_model,
        gpu_vram_mb: gpu_vram,
        gpu_driver_version: gpu_driver,
        disks,
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

pub fn collect_telemetry_sample() -> TelemetrySample {
    let mut sys = System::new_all();
    sys.refresh_all();
    std::thread::sleep(std::time::Duration::from_millis(100));
    sys.refresh_all();

    let top_processes: Vec<ProcessSnapshot> = {
        let mut procs: Vec<_> = sys
            .processes()
            .values()
            .filter(|p| p.memory() > 50 * 1024 * 1024 || p.cpu_usage() > 2.0)
            .map(|p| ProcessSnapshot {
                name: p.name().to_string(),
                pid: p.pid().as_u32(),
                cpu_percent: p.cpu_usage(),
                ram_mb: p.memory() as f64 / (1024.0 * 1024.0),
            })
            .collect();
        procs.sort_by(|a, b| {
            b.cpu_percent
                .partial_cmp(&a.cpu_percent)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        procs.truncate(10);
        procs
    };

    TelemetrySample {
        timestamp: chrono::Utc::now().to_rfc3339(),
        cpu_percent: sys.global_cpu_info().cpu_usage(),
        ram_used_mb: sys.used_memory() / (1024 * 1024),
        ram_available_mb: sys.available_memory() / (1024 * 1024),
        top_processes,
    }
}

pub fn collect_self_metrics() -> SelfMetrics {
    let mut sys = System::new_all();
    sys.refresh_all();
    let pid = sysinfo::get_current_pid().unwrap();

    match sys.process(pid) {
        Some(p) => SelfMetrics {
            cpu_percent: p.cpu_usage(),
            ram_mb: p.memory() as f64 / (1024.0 * 1024.0),
        },
        None => SelfMetrics {
            cpu_percent: 0.0,
            ram_mb: 0.0,
        },
    }
}

pub fn is_process_running(process_name: &str) -> bool {
    let mut sys = System::new_all();
    sys.refresh_all();
    let lower = process_name.to_lowercase();
    sys.processes()
        .values()
        .any(|p| p.name().to_lowercase().contains(&lower))
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

fn detect_gpu_details() -> (String, u64, String) {
    #[cfg(target_os = "windows")]
    {
        detect_gpu_windows()
    }
    #[cfg(target_os = "macos")]
    {
        ("GPU detection available on Windows".to_string(), 0, String::new())
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        ("Unknown GPU".to_string(), 0, String::new())
    }
}

#[cfg(target_os = "windows")]
fn detect_gpu_windows() -> (String, u64, String) {
    use std::process::Command;

    let name = Command::new("wmic")
        .args(["path", "win32_videocontroller", "get", "name"])
        .output()
        .ok()
        .and_then(|out| {
            let text = String::from_utf8_lossy(&out.stdout).to_string();
            text.lines()
                .nth(1)
                .map(|l| l.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| "Unknown GPU".to_string());

    let vram = Command::new("wmic")
        .args(["path", "win32_videocontroller", "get", "adapterram"])
        .output()
        .ok()
        .and_then(|out| {
            let text = String::from_utf8_lossy(&out.stdout).to_string();
            text.lines()
                .nth(1)
                .and_then(|l| l.trim().parse::<u64>().ok())
                .map(|bytes| bytes / (1024 * 1024))
        })
        .unwrap_or(0);

    let driver = Command::new("wmic")
        .args(["path", "win32_videocontroller", "get", "driverversion"])
        .output()
        .ok()
        .and_then(|out| {
            let text = String::from_utf8_lossy(&out.stdout).to_string();
            text.lines()
                .nth(1)
                .map(|l| l.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_default();

    (name, vram, driver)
}
