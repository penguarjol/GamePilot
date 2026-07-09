use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct DiscoveredLauncher {
    pub name: String,
    pub path: PathBuf,
    pub launcher_type: LauncherType,
}

#[derive(Debug, Clone, Serialize)]
pub enum LauncherType {
    PrismLauncher,
    MultiMC,
    CurseForge,
    Modrinth,
    ATLauncher,
    OfficialLauncher,
    Custom,
}

impl std::fmt::Display for LauncherType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LauncherType::PrismLauncher => write!(f, "Prism Launcher"),
            LauncherType::MultiMC => write!(f, "MultiMC"),
            LauncherType::CurseForge => write!(f, "CurseForge"),
            LauncherType::Modrinth => write!(f, "Modrinth App"),
            LauncherType::ATLauncher => write!(f, "ATLauncher"),
            LauncherType::OfficialLauncher => write!(f, "Official Launcher"),
            LauncherType::Custom => write!(f, "Custom"),
        }
    }
}

pub fn discover_launchers() -> Vec<DiscoveredLauncher> {
    let mut launchers = Vec::new();

    #[cfg(target_os = "windows")]
    {
        discover_windows_launchers(&mut launchers);
    }

    #[cfg(target_os = "macos")]
    {
        discover_macos_launchers(&mut launchers);
    }

    launchers
}

#[cfg(target_os = "windows")]
fn discover_windows_launchers(launchers: &mut Vec<DiscoveredLauncher>) {
    let appdata = std::env::var("APPDATA").unwrap_or_default();
    let local_appdata = std::env::var("LOCALAPPDATA").unwrap_or_default();

    let candidates = [
        (
            PathBuf::from(&appdata).join("PrismLauncher"),
            "Prism Launcher",
            LauncherType::PrismLauncher,
        ),
        (
            PathBuf::from(&appdata).join("PrismLauncher").join("instances"),
            "Prism Launcher",
            LauncherType::PrismLauncher,
        ),
        (
            PathBuf::from(&appdata).join("MultiMC"),
            "MultiMC",
            LauncherType::MultiMC,
        ),
        (
            PathBuf::from(&appdata).join(".minecraft"),
            "Official Launcher",
            LauncherType::OfficialLauncher,
        ),
        (
            PathBuf::from(&appdata).join("ATLauncher"),
            "ATLauncher",
            LauncherType::ATLauncher,
        ),
        (
            PathBuf::from(&local_appdata)
                .join("CurseForge")
                .join("Minecraft")
                .join("Instances"),
            "CurseForge",
            LauncherType::CurseForge,
        ),
        (
            PathBuf::from(&appdata)
                .join("com.modrinth.theseus")
                .join("profiles"),
            "Modrinth App",
            LauncherType::Modrinth,
        ),
    ];

    for (path, name, launcher_type) in candidates {
        if path.exists() {
            launchers.push(DiscoveredLauncher {
                name: name.to_string(),
                path,
                launcher_type,
            });
        }
    }
}

#[cfg(target_os = "macos")]
fn discover_macos_launchers(launchers: &mut Vec<DiscoveredLauncher>) {
    let home = std::env::var("HOME").unwrap_or_default();

    let candidates = [
        (
            PathBuf::from(&home)
                .join("Library")
                .join("Application Support")
                .join("PrismLauncher"),
            "Prism Launcher",
            LauncherType::PrismLauncher,
        ),
        (
            PathBuf::from(&home)
                .join("Library")
                .join("Application Support")
                .join("minecraft"),
            "Official Launcher",
            LauncherType::OfficialLauncher,
        ),
        (
            PathBuf::from(&home)
                .join("Library")
                .join("Application Support")
                .join("MultiMC"),
            "MultiMC",
            LauncherType::MultiMC,
        ),
        (
            PathBuf::from(&home)
                .join("Library")
                .join("Application Support")
                .join("ATLauncher"),
            "ATLauncher",
            LauncherType::ATLauncher,
        ),
    ];

    for (path, name, launcher_type) in candidates {
        if path.exists() {
            launchers.push(DiscoveredLauncher {
                name: name.to_string(),
                path,
                launcher_type,
            });
        }
    }
}

pub fn discover_instances_in_path(path: &PathBuf, launcher: &LauncherType) -> Vec<PathBuf> {
    let mut instances = Vec::new();

    match launcher {
        LauncherType::PrismLauncher | LauncherType::MultiMC => {
            let instances_dir = if path.join("instances").exists() {
                path.join("instances")
            } else {
                path.clone()
            };

            if let Ok(entries) = std::fs::read_dir(&instances_dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() && !p.file_name().unwrap_or_default().to_string_lossy().starts_with('.') {
                        if p.join(".minecraft").exists() || p.join("minecraft").exists() || p.join("mmc-pack.json").exists() || p.join("instance.cfg").exists() {
                            instances.push(p);
                        }
                    }
                }
            }
        }
        LauncherType::CurseForge => {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() {
                        if p.join("minecraftinstance.json").exists() || p.join("mods").exists() {
                            instances.push(p);
                        }
                    }
                }
            }
        }
        LauncherType::ATLauncher => {
            let instances_dir = path.join("instances");
            if let Ok(entries) = std::fs::read_dir(&instances_dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() && p.join("instance.json").exists() {
                        instances.push(p);
                    }
                }
            }
        }
        LauncherType::OfficialLauncher => {
            if path.join("versions").exists() {
                instances.push(path.clone());
            }
        }
        LauncherType::Modrinth => {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() && p.join("profile.json").exists() {
                        instances.push(p);
                    }
                }
            }
        }
        LauncherType::Custom => {
            instances.push(path.clone());
        }
    }

    instances
}
