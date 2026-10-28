use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarHandshake {
    pub port: u16,
    pub token: String,
    pub pid: u32,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarStatus {
    pub running: bool,
    pub port: Option<u16>,
    pub pid: Option<u32>,
    pub version: Option<String>,
    pub uptime_s: Option<f64>,
    pub last_error: Option<String>,
}

pub struct SidecarState {
    pub handshake: Mutex<Option<SidecarHandshake>>,
    shutting_down: AtomicBool,
    child_pid: Mutex<Option<u32>>,
    pub last_error: Mutex<Option<String>>,
}

impl SidecarState {
    pub fn new() -> Self {
        Self {
            handshake: Mutex::new(None),
            shutting_down: AtomicBool::new(false),
            child_pid: Mutex::new(None),
            last_error: Mutex::new(None),
        }
    }

    pub async fn base_url(&self) -> Option<String> {
        let hs = self.handshake.lock().await;
        hs.as_ref().map(|h| format!("http://127.0.0.1:{}", h.port))
    }

    pub async fn token(&self) -> Option<String> {
        let hs = self.handshake.lock().await;
        hs.as_ref().map(|h| h.token.clone())
    }

    pub async fn status(&self) -> SidecarStatus {
        let hs = self.handshake.lock().await;
        let err = self.last_error.lock().await;
        match hs.as_ref() {
            Some(h) => SidecarStatus {
                running: true,
                port: Some(h.port),
                pid: Some(h.pid),
                version: Some(h.version.clone()),
                uptime_s: None,
                last_error: None,
            },
            None => SidecarStatus {
                running: false,
                port: None,
                pid: None,
                version: None,
                uptime_s: None,
                last_error: err.clone(),
            },
        }
    }

    pub fn mark_shutdown(&self) {
        self.shutting_down.store(true, Ordering::SeqCst);
    }

    pub fn is_shutting_down(&self) -> bool {
        self.shutting_down.load(Ordering::SeqCst)
    }
}

pub async fn spawn_sidecar_with_retry(state: &SidecarState) -> Result<(), String> {
    // First: check if a sidecar is already running on common ports
    for port in [8017, 8018, 8019, 8020] {
        if let Ok(()) = try_connect_existing(state, port).await {
            log::info!("Connected to existing sidecar on port {}", port);
            return Ok(());
        }
    }

    let max_attempts = 2;
    let mut last_err = String::new();

    for attempt in 1..=max_attempts {
        log::info!("Sidecar spawn attempt {}/{}", attempt, max_attempts);
        match spawn_sidecar(state).await {
            Ok(()) => {
                *state.last_error.lock().await = None;
                return Ok(());
            }
            Err(e) => {
                last_err = e.clone();
                log::warn!("Sidecar attempt {} failed: {}", attempt, e);
                *state.last_error.lock().await = Some(e);
                if attempt < max_attempts {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    }

    let msg = format!("Sidecar not available: {}. Start it manually or check Settings > Clipper.", last_err);
    *state.last_error.lock().await = Some(msg.clone());
    Err(msg)
}

async fn try_connect_existing(state: &SidecarState, port: u16) -> Result<(), String> {
    let url = format!("http://127.0.0.1:{}/health", port);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build().map_err(|e| e.to_string())?;

    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err("Health check failed".to_string());
    }

    let handshake = SidecarHandshake {
        port,
        token: String::new(),
        pid: 0,
        version: "external".to_string(),
    };
    *state.handshake.lock().await = Some(handshake);
    *state.last_error.lock().await = None;
    Ok(())
}

async fn spawn_sidecar(state: &SidecarState) -> Result<(), String> {
    use std::process::Stdio;
    use tokio::io::AsyncBufReadExt;
    use tokio::process::Command;

    let token = uuid::Uuid::new_v4().to_string();
    let parent_pid = std::process::id();

    let (program, script_arg) = find_sidecar_binary()?;

    log::info!("Sidecar binary: {} {:?}", program, script_arg);

    let mut cmd = Command::new(&program);
    if let Some(script) = &script_arg {
        cmd.arg(script);
    }
    let mut child = cmd
        .args(["--port", "0", "--token", &token, "--parent-pid", &parent_pid.to_string()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("Failed to spawn '{}': {}", program, e))?;

    let pid = child.id().unwrap_or(0);
    *state.child_pid.lock().await = Some(pid);

    let stdout = child.stdout.take().ok_or("No stdout from sidecar")?;
    let mut reader = tokio::io::BufReader::new(stdout).lines();

    let handshake_line = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        reader.next_line(),
    )
    .await
    .map_err(|_| "Sidecar handshake timed out after 30s".to_string())?
    .map_err(|e| format!("Failed to read handshake: {}", e))?
    .ok_or("Sidecar closed stdout before handshake")?;

    let handshake: SidecarHandshake = serde_json::from_str(&handshake_line)
        .map_err(|e| format!("Invalid handshake JSON: {} (got: {})", e, &handshake_line[..handshake_line.len().min(200)]))?;

    log::info!("Sidecar started: port={}, pid={}, version={}", handshake.port, handshake.pid, handshake.version);

    poll_health(&handshake, 10).await?;

    *state.handshake.lock().await = Some(handshake);

    tokio::spawn(async move {
        let status = child.wait().await;
        log::warn!("Sidecar process exited: {:?}", status);
    });

    Ok(())
}

async fn poll_health(handshake: &SidecarHandshake, max_attempts: u32) -> Result<(), String> {
    let url = format!("http://127.0.0.1:{}/health", handshake.port);
    let client = reqwest::Client::new();

    for attempt in 1..=max_attempts {
        match client.get(&url).timeout(std::time::Duration::from_secs(3)).send().await {
            Ok(resp) if resp.status().is_success() => {
                log::info!("Sidecar health OK on attempt {}", attempt);
                return Ok(());
            }
            _ => {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        }
    }

    Err(format!("Sidecar health check failed after {} attempts on port {}", max_attempts, handshake.port))
}

pub async fn stop_sidecar(state: &SidecarState) {
    state.mark_shutdown();
    if let Some(pid) = state.child_pid.lock().await.take() {
        #[cfg(unix)]
        {
            let _ = std::process::Command::new("kill").args(["-TERM", &pid.to_string()]).output();
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            let _ = std::process::Command::new("kill").args(["-9", &pid.to_string()]).output();
        }
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("taskkill").args(["/T", "/F", "/PID", &pid.to_string()]).output();
        }
    }
    *state.handshake.lock().await = None;
    state.shutting_down.store(false, Ordering::SeqCst);
}

pub async fn restart_sidecar(state: &SidecarState) -> Result<(), String> {
    log::info!("Restarting sidecar...");
    stop_sidecar(state).await;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    spawn_sidecar_with_retry(state).await
}

fn find_sidecar_binary() -> Result<(String, Option<String>), String> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));

    if let Some(dir) = &exe_dir {
        #[cfg(windows)]
        let name = "clipper-sidecar-x86_64-pc-windows-msvc.exe";
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        let name = "clipper-sidecar-aarch64-apple-darwin";
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        let name = "clipper-sidecar-x86_64-apple-darwin";
        #[cfg(target_os = "linux")]
        let name = "clipper-sidecar-x86_64-unknown-linux-gnu";

        let bundled = dir.join(name);
        if bundled.exists() {
            return Ok((bundled.to_string_lossy().to_string(), None));
        }
    }

    let dev_candidates = ["sidecar/api.py", "../sidecar/api.py"];
    for candidate in &dev_candidates {
        let path = std::path::Path::new(candidate);
        if path.exists() {
            return Ok(("python3".to_string(), Some(path.to_string_lossy().to_string())));
        }
    }

    Err("Clipper sidecar not found. In dev: run from project root with Python deps installed. In production: the sidecar binary should be bundled.".to_string())
}

pub async fn sidecar_request(
    state: &SidecarState,
    method: &str,
    path: &str,
    body: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let base_url = state.base_url().await.ok_or("Sidecar not running")?;
    let token = state.token().await.unwrap_or_default();
    let url = format!("{}{}", base_url, path);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let req = match method {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        _ => return Err(format!("Unsupported method: {}", method)),
    };

    let req = req.header("Authorization", &token);
    let req = if let Some(b) = body { req.json(&b) } else { req };

    let resp = req.send().await.map_err(|e| format!("Sidecar request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Sidecar returned {}: {}", status, text));
    }

    resp.json::<serde_json::Value>().await
        .map_err(|e| format!("Failed to parse sidecar response: {}", e))
}
