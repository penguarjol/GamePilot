use std::path::{Path, PathBuf};

pub fn detect_java_installations() -> Vec<JavaInstallation> {
    let mut javas = Vec::new();

    #[cfg(target_os = "windows")]
    {
        detect_java_windows(&mut javas);
    }

    #[cfg(target_os = "macos")]
    {
        detect_java_macos(&mut javas);
    }

    javas
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct JavaInstallation {
    pub path: PathBuf,
    pub version: Option<String>,
    pub vendor: Option<String>,
    pub is_64bit: bool,
}

fn create_hidden_command(program: &Path) -> std::process::Command {
    #[allow(unused_mut)]
    let mut cmd = std::process::Command::new(program);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    cmd
}

#[cfg(target_os = "windows")]
fn detect_java_windows(javas: &mut Vec<JavaInstallation>) {
    let search_paths = vec![
        std::env::var("JAVA_HOME").ok().map(PathBuf::from),
        Some(PathBuf::from("C:\\Program Files\\Java")),
        Some(PathBuf::from("C:\\Program Files\\Eclipse Adoptium")),
        Some(PathBuf::from("C:\\Program Files\\Microsoft")),
        Some(PathBuf::from("C:\\Program Files\\Zulu")),
    ];

    for maybe_path in search_paths.into_iter().flatten() {
        if maybe_path.exists() {
            if maybe_path.join("bin").join("java.exe").exists() {
                if let Some(info) = probe_java(&maybe_path.join("bin").join("java.exe")) {
                    javas.push(info);
                }
            } else if let Ok(entries) = std::fs::read_dir(&maybe_path) {
                for entry in entries.flatten() {
                    let java_exe = entry.path().join("bin").join("java.exe");
                    if java_exe.exists() {
                        if let Some(info) = probe_java(&java_exe) {
                            javas.push(info);
                        }
                    }
                }
            }
        }
    }

    let where_path = Path::new("C:\\Windows\\System32\\where.exe");
    if let Ok(output) = create_hidden_command(where_path).arg("java").output() {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            let p = PathBuf::from(line.trim());
            if p.exists() && !javas.iter().any(|j| j.path == p) {
                if let Some(info) = probe_java(&p) {
                    javas.push(info);
                }
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn detect_java_macos(javas: &mut Vec<JavaInstallation>) {
    let search_paths = vec![
        std::env::var("JAVA_HOME").ok().map(PathBuf::from),
        Some(PathBuf::from("/Library/Java/JavaVirtualMachines")),
    ];

    for maybe_path in search_paths.into_iter().flatten() {
        if maybe_path.join("bin").join("java").exists() {
            if let Some(info) = probe_java(&maybe_path.join("bin").join("java")) {
                javas.push(info);
            }
        } else if let Ok(entries) = std::fs::read_dir(&maybe_path) {
            for entry in entries.flatten() {
                let java_bin = entry
                    .path()
                    .join("Contents")
                    .join("Home")
                    .join("bin")
                    .join("java");
                if java_bin.exists() {
                    if let Some(info) = probe_java(&java_bin) {
                        javas.push(info);
                    }
                }
            }
        }
    }

    if let Ok(output) = create_hidden_command(Path::new("/usr/bin/which")).arg("java").output() {
        let text = String::from_utf8_lossy(&output.stdout);
        let p = PathBuf::from(text.trim());
        if p.exists() && !javas.iter().any(|j| j.path == p) {
            if let Some(info) = probe_java(&p) {
                javas.push(info);
            }
        }
    }
}

fn probe_java(java_path: &Path) -> Option<JavaInstallation> {
    let output = create_hidden_command(java_path)
        .arg("-version")
        .output()
        .ok()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let version = stderr
        .lines()
        .next()
        .and_then(|line| {
            line.split('"')
                .nth(1)
                .map(String::from)
        });

    let vendor = stderr.lines().nth(1).map(|l| l.trim().to_string());
    let is_64bit = stderr.contains("64-Bit");

    Some(JavaInstallation {
        path: java_path.to_path_buf(),
        version,
        vendor,
        is_64bit,
    })
}
