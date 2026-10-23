use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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
}

pub struct SidecarState {
    handshake: Mutex<Option<SidecarHandshake>>,
    shutting_down: AtomicBool,
    child_pid: Mutex<Option<u32>>,
}

impl SidecarState {
    pub fn new() -> Self {
        Self {
            handshake: Mutex::new(None),
            shutting_down: AtomicBool::new(false),
            child_pid: Mutex::new(None),
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
        match hs.as_ref() {
            Some(h) => SidecarStatus {
                running: true,
                port: Some(h.port),
                pid: Some(h.pid),
                version: Some(h.version.clone()),
                uptime_s: None,
            },
            None => SidecarStatus {
                running: false,
                port: None,
                pid: None,
                version: None,
                uptime_s: None,
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

pub async fn spawn_sidecar(state: &SidecarState) -> Result<(), String> {
    use std::process::Stdio;
    use tokio::io::AsyncBufReadExt;
    use tokio::process::Command;

    let token = uuid::Uuid::new_v4().to_string();
    let parent_pid = std::process::id();

    let sidecar_path = find_sidecar_binary()?;

    let mut child = Command::new(&sidecar_path)
        .args([
            "--port", "0",
            "--token", &token,
            "--parent-pid", &parent_pid.to_string(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("Failed to spawn sidecar: {}", e))?;

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
        .map_err(|e| format!("Invalid handshake JSON: {} — got: {}", e, handshake_line))?;

    log::info!(
        "Sidecar started: port={}, pid={}, version={}",
        handshake.port, handshake.pid, handshake.version
    );

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
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                log::info!("Sidecar health check passed on attempt {}", attempt);
                return Ok(());
            }
            _ => {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        }
    }

    Err(format!("Sidecar health check failed after {} attempts", max_attempts))
}

pub async fn stop_sidecar(state: &SidecarState) {
    state.mark_shutdown();
    if let Some(pid) = state.child_pid.lock().await.take() {
        #[cfg(unix)]
        {
            let _ = std::process::Command::new("kill")
                .args(["-TERM", &pid.to_string()])
                .output();
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            let _ = std::process::Command::new("kill")
                .args(["-9", &pid.to_string()])
                .output();
        }
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("taskkill")
                .args(["/T", "/F", "/PID", &pid.to_string()])
                .output();
        }
    }
    *state.handshake.lock().await = None;
}

fn find_sidecar_binary() -> Result<String, String> {
    let candidates = [
        "sidecar/api.py",
        "../sidecar/api.py",
    ];

    for candidate in &candidates {
        let path = std::path::Path::new(candidate);
        if path.exists() {
            return Ok(format!("python3 {}", path.display()));
        }
    }

    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));

    if let Some(dir) = exe_dir {
        let bundled = dir.join("clipper-sidecar");
        if bundled.exists() {
            return Ok(bundled.to_string_lossy().to_string());
        }
        #[cfg(windows)]
        {
            let bundled_exe = dir.join("clipper-sidecar.exe");
            if bundled_exe.exists() {
                return Ok(bundled_exe.to_string_lossy().to_string());
            }
        }
    }

    Err("Sidecar binary not found. Run from project root or bundle with PyInstaller.".to_string())
}

pub async fn sidecar_request(
    state: &SidecarState,
    method: &str,
    path: &str,
    body: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let base_url = state.base_url().await
        .ok_or("Sidecar not running")?;
    let token = state.token().await.unwrap_or_default();
    let url = format!("{}{}", base_url, path);

    let client = reqwest::Client::new();
    let req = match method {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        _ => return Err(format!("Unsupported method: {}", method)),
    };

    let req = req.header("Authorization", &token);
    let req = if let Some(b) = body {
        req.json(&b)
    } else {
        req
    };

    let resp = req.send().await
        .map_err(|e| format!("Sidecar request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Sidecar returned {}: {}", status, text));
    }

    resp.json::<serde_json::Value>().await
        .map_err(|e| format!("Failed to parse sidecar response: {}", e))
}
