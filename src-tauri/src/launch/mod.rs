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
        "prism launcher" | "multimc" => launch_via_launcher(profile, "prismlauncher"),
        "curseforge" => launch_via_launcher(profile, "curseforge"),
        _ => launch_via_folder_open(profile),
    }
}

fn launch_via_launcher(profile: &LaunchProfile, _launcher_exe: &str) -> LaunchResult {
    let session_id = uuid::Uuid::new_v4().to_string();

    #[cfg(target_os = "windows")]
    {
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
