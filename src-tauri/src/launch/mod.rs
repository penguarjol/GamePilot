use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct LaunchResult {
    pub success: bool,
    pub method: String,
    pub message: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LaunchProfile {
    pub instance_id: String,
    pub launcher: String,
    pub instance_path: String,
    pub java_path: Option<String>,
    pub jvm_args: Option<String>,
}

pub fn launch_instance(profile: &LaunchProfile) -> LaunchResult {
    let launcher = profile.launcher.to_lowercase();

    match launcher.as_str() {
        "prism launcher" | "multimc" => launch_via_launcher(profile),
        "curseforge" => launch_via_curseforge(profile),
        _ => launch_via_folder_open(profile),
    }
}

fn launch_via_launcher(profile: &LaunchProfile) -> LaunchResult {
    let session_id = uuid::Uuid::new_v4().to_string();

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        use std::process::Command;

        let launcher_cmd = match profile.launcher.to_lowercase().as_str() {
            l if l.contains("prism") => "prismlauncher",
            l if l.contains("multimc") => "multimc",
            _ => return launch_via_folder_open(profile),
        };

        let instance_name = Path::new(&profile.instance_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        match Command::new(launcher_cmd)
            .args(["--launch", &instance_name])
            .creation_flags(0x08000000)
            .spawn()
        {
            Ok(_) => LaunchResult {
                success: true,
                method: format!("Launched via {}", profile.launcher),
                message: format!(
                    "Instance '{}' launched through {}.",
                    instance_name, profile.launcher
                ),
                session_id: Some(session_id),
            },
            Err(e) => {
                log::warn!("Launcher command failed: {}, falling back to folder open", e);
                launch_via_folder_open(profile)
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        LaunchResult {
            success: true,
            method: "Delegated (dev mode)".to_string(),
            message: format!(
                "On Windows, this would launch via {}. \
                 Open the instance folder to launch manually.",
                profile.launcher
            ),
            session_id: Some(session_id),
        }
    }
}

fn launch_via_curseforge(profile: &LaunchProfile) -> LaunchResult {
    let session_id = uuid::Uuid::new_v4().to_string();
    let instance_name = Path::new(&profile.instance_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        use std::process::Command;

        // Strategy 1: Use curseforge:// URI protocol to launch the specific instance.
        // CurseForge registers this protocol handler on install.
        // The run-instance URI format triggers a direct game launch.
        let instance_id_from_manifest = read_curseforge_instance_id(&profile.instance_path);

        if let Some(cf_instance_id) = instance_id_from_manifest {
            let uri = format!("curseforge://run-instance/minecraft/{}", cf_instance_id);
            match Command::new("cmd")
                .args(["/C", "start", "", &uri])
                .creation_flags(0x08000000)
                .spawn()
            {
                Ok(_) => {
                    return LaunchResult {
                        success: true,
                        method: "CurseForge URI".to_string(),
                        message: format!(
                            "Launching '{}' via CurseForge. The game should start shortly.",
                            instance_name
                        ),
                        session_id: Some(session_id),
                    };
                }
                Err(e) => log::warn!("CurseForge URI launch failed: {}", e),
            }
        }

        // Strategy 2: Open CurseForge app directly and let the user click play.
        let cf_paths = [
            format!(
                "{}\\Programs\\CurseForge\\CurseForge.exe",
                std::env::var("LOCALAPPDATA").unwrap_or_default()
            ),
            format!(
                "{}\\Overwolf\\CurseForge.exe",
                std::env::var("LOCALAPPDATA").unwrap_or_default()
            ),
            format!(
                "{}\\CurseForge\\CurseForge.exe",
                std::env::var("PROGRAMFILES").unwrap_or_default()
            ),
        ];

        for cf_exe in &cf_paths {
            if std::path::Path::new(cf_exe).exists() {
                match Command::new(cf_exe)
                    .creation_flags(0x08000000)
                    .spawn()
                {
                    Ok(_) => {
                        return LaunchResult {
                            success: true,
                            method: "CurseForge App".to_string(),
                            message: format!(
                                "Opened CurseForge app. Navigate to '{}' and click Play.",
                                instance_name
                            ),
                            session_id: Some(session_id),
                        };
                    }
                    Err(_) => continue,
                }
            }
        }

        // Strategy 3: Fall back to opening the instance folder.
        launch_via_folder_open(profile)
    }

    #[cfg(not(target_os = "windows"))]
    {
        LaunchResult {
            success: true,
            method: "Delegated (dev mode)".to_string(),
            message: format!(
                "On Windows, this would launch '{}' via CurseForge.",
                instance_name
            ),
            session_id: Some(session_id),
        }
    }
}

/// Read the CurseForge instance ID from minecraftinstance.json.
/// CurseForge stores an "installedModpack.addonID" or "baseModLoaderPath"
/// that can be used to construct a launch URI.
fn read_curseforge_instance_id(instance_path: &str) -> Option<String> {
    let manifest_path = Path::new(instance_path).join("minecraftinstance.json");
    let content = std::fs::read_to_string(&manifest_path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;

    // Try the installedModpack path first — the addonID is CurseForge's project ID
    if let Some(addon_id) = v
        .get("installedModpack")
        .and_then(|m| m.get("addonID"))
        .and_then(|id| id.as_u64())
    {
        return Some(addon_id.to_string());
    }

    // Fall back to the instance folder name as an identifier
    Path::new(instance_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
}

fn launch_via_folder_open(profile: &LaunchProfile) -> LaunchResult {
    let session_id = uuid::Uuid::new_v4().to_string();
    let path = Path::new(&profile.instance_path);

    if !path.exists() {
        return LaunchResult {
            success: false,
            method: "folder_open".to_string(),
            message: format!("Instance path does not exist: {}", profile.instance_path),
            session_id: None,
        };
    }

    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer")
            .arg(&profile.instance_path)
            .spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(&profile.instance_path)
            .spawn();
    }

    LaunchResult {
        success: true,
        method: "folder_open".to_string(),
        message: format!(
            "Opened instance folder. Launch Minecraft from your preferred launcher \
             using the instance at: {}",
            profile.instance_path
        ),
        session_id: Some(session_id),
    }
}
