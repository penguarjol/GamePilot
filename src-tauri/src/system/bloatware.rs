use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct BloatwareReport {
    pub temp_files: Vec<TempFileInfo>,
    pub total_temp_size_mb: f64,
    pub startup_programs: Vec<StartupProgram>,
    pub cleanup_recommendations: Vec<CleanupRecommendation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TempFileInfo {
    pub path: String,
    pub size_mb: f64,
    pub category: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StartupProgram {
    pub name: String,
    pub command: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CleanupRecommendation {
    pub title: String,
    pub description: String,
    pub estimated_size_mb: f64,
    pub action: String,
    pub safe: bool,
}

pub fn scan_bloatware() -> BloatwareReport {
    let temp_files = scan_temp_files();
    let total_temp: f64 = temp_files.iter().map(|t| t.size_mb).sum();
    let startup = scan_startup_programs();
    let recs = generate_cleanup_recommendations(&temp_files, total_temp);

    BloatwareReport {
        temp_files,
        total_temp_size_mb: total_temp,
        startup_programs: startup,
        cleanup_recommendations: recs,
    }
}

fn scan_temp_files() -> Vec<TempFileInfo> {
    let mut results = Vec::new();

    let temp_dirs = vec![
        (std::env::var("TEMP").unwrap_or_default(), "Windows Temp"),
        (std::env::var("TMP").unwrap_or_default(), "Windows TMP"),
        (
            format!(
                "{}\\AppData\\Local\\Temp",
                std::env::var("USERPROFILE").unwrap_or_default()
            ),
            "User Temp",
        ),
        (
            format!(
                "{}\\AppData\\Local\\Google\\Chrome\\User Data\\Default\\Cache",
                std::env::var("USERPROFILE").unwrap_or_default()
            ),
            "Chrome Cache",
        ),
        (
            format!(
                "{}\\AppData\\Local\\Microsoft\\Edge\\User Data\\Default\\Cache",
                std::env::var("USERPROFILE").unwrap_or_default()
            ),
            "Edge Cache",
        ),
        (
            format!(
                "{}\\AppData\\Local\\Discord\\Cache",
                std::env::var("USERPROFILE").unwrap_or_default()
            ),
            "Discord Cache",
        ),
    ];

    for (path, category) in &temp_dirs {
        if path.is_empty() {
            continue;
        }
        let dir = std::path::Path::new(path);
        if dir.exists() {
            let size = dir_size(dir);
            if size > 1.0 {
                results.push(TempFileInfo {
                    path: path.clone(),
                    size_mb: size,
                    category: category.to_string(),
                });
            }
        }
    }

    results.sort_by(|a, b| {
        b.size_mb
            .partial_cmp(&a.size_mb)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results
}

fn dir_size(path: &std::path::Path) -> f64 {
    let mut total: u64 = 0;
    for entry in walkdir::WalkDir::new(path)
        .max_depth(3)
        .into_iter()
        .flatten()
    {
        if entry.file_type().is_file() {
            total += entry.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }
    total as f64 / (1024.0 * 1024.0)
}

fn scan_startup_programs() -> Vec<StartupProgram> {
    #[cfg(target_os = "windows")]
    {
        scan_startup_windows()
    }
    #[cfg(not(target_os = "windows"))]
    {
        Vec::new()
    }
}

#[cfg(target_os = "windows")]
fn scan_startup_windows() -> Vec<StartupProgram> {
    use std::os::windows::process::CommandExt;

    let output = std::process::Command::new("wmic")
        .args(["startup", "get", "Caption,Command,Location", "/format:csv"])
        .creation_flags(0x08000000)
        .output()
        .ok();

    let mut programs = Vec::new();

    if let Some(out) = output {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines().skip(1) {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                let name = parts.get(1).unwrap_or(&"").trim().to_string();
                let command = parts.get(2).unwrap_or(&"").trim().to_string();
                let source = parts.get(3).unwrap_or(&"").trim().to_string();
                if !name.is_empty() {
                    programs.push(StartupProgram {
                        name,
                        command,
                        source,
                    });
                }
            }
        }
    }

    programs
}

fn generate_cleanup_recommendations(
    temp_files: &[TempFileInfo],
    total_mb: f64,
) -> Vec<CleanupRecommendation> {
    let mut recs = Vec::new();

    if total_mb > 1000.0 {
        recs.push(CleanupRecommendation {
            title: "Clear temporary files".to_string(),
            description: format!(
                "{:.1} GB of temporary files detected. Running Disk Cleanup can recover most of this space.",
                total_mb / 1024.0
            ),
            estimated_size_mb: total_mb * 0.7,
            action: "open_disk_cleanup".to_string(),
            safe: true,
        });
    }

    for tf in temp_files {
        if tf.size_mb > 500.0 && tf.category.contains("Cache") {
            recs.push(CleanupRecommendation {
                title: format!("Clear {} ({:.1} GB)", tf.category, tf.size_mb / 1024.0),
                description: format!(
                    "Browser/app cache at {} is using {:.0} MB. Clearing it is safe and frees significant space.",
                    tf.path, tf.size_mb
                ),
                estimated_size_mb: tf.size_mb,
                action: "info".to_string(),
                safe: true,
            });
        }
    }

    recs
}
