use serde::{Deserialize, Serialize};
use std::path::Path;

const MODRINTH_API: &str = "https://api.modrinth.com/v2";
const USER_AGENT: &str = "GamePilot/0.3.0 (github.com/penguarjol/GamePilot)";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModSearchResult {
    pub project_id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub downloads: u64,
    pub icon_url: Option<String>,
    pub categories: Vec<String>,
    pub latest_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModVersion {
    pub id: String,
    pub version_number: String,
    pub name: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub downloads: u64,
    pub files: Vec<ModFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModFile {
    pub url: String,
    pub filename: String,
    pub size: u64,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstallResult {
    pub success: bool,
    pub filename: String,
    pub installed_path: String,
    pub message: String,
}

/// Search Modrinth for mods compatible with a given MC version and loader.
pub async fn search_mods(
    query: &str,
    mc_version: Option<&str>,
    loader: Option<&str>,
    limit: u32,
) -> Result<Vec<ModSearchResult>, String> {
    let client = reqwest::Client::new();

    let mut facets = vec![vec!["project_type:mod".to_string()]];
    if let Some(ver) = mc_version {
        facets.push(vec![format!("versions:{}", ver)]);
    }
    if let Some(ldr) = loader {
        facets.push(vec![format!("categories:{}", ldr.to_lowercase())]);
    }
    let facets_json = serde_json::to_string(&facets).unwrap_or_default();

    let resp = client
        .get(format!("{}/search", MODRINTH_API))
        .header("User-Agent", USER_AGENT)
        .query(&[
            ("query", query),
            ("facets", &facets_json),
            ("limit", &limit.to_string()),
            ("index", "relevance"),
        ])
        .send()
        .await
        .map_err(|e| format!("Search failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Modrinth API error: {}", resp.status()));
    }

    let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;

    let hits = body
        .get("hits")
        .and_then(|h| h.as_array())
        .cloned()
        .unwrap_or_default();

    let results: Vec<ModSearchResult> = hits
        .iter()
        .filter_map(|hit| {
            Some(ModSearchResult {
                project_id: hit.get("project_id")?.as_str()?.to_string(),
                slug: hit.get("slug")?.as_str()?.to_string(),
                title: hit.get("title")?.as_str()?.to_string(),
                description: hit
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("")
                    .to_string(),
                author: hit
                    .get("author")
                    .and_then(|a| a.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                downloads: hit.get("downloads").and_then(|d| d.as_u64()).unwrap_or(0),
                icon_url: hit.get("icon_url").and_then(|u| u.as_str()).map(String::from),
                categories: hit
                    .get("categories")
                    .and_then(|c| c.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default(),
                latest_version: hit
                    .get("latest_version")
                    .and_then(|v| v.as_str())
                    .map(String::from),
            })
        })
        .collect();

    Ok(results)
}

/// Get available versions for a specific mod, filtered by MC version and loader.
pub async fn get_mod_versions(
    project_id: &str,
    mc_version: Option<&str>,
    loader: Option<&str>,
) -> Result<Vec<ModVersion>, String> {
    let client = reqwest::Client::new();

    let mut url = format!("{}/project/{}/version", MODRINTH_API, project_id);
    let mut params = Vec::new();
    if let Some(ver) = mc_version {
        params.push(format!("game_versions=[%22{}%22]", ver));
    }
    if let Some(ldr) = loader {
        params.push(format!("loaders=[%22{}%22]", ldr.to_lowercase()));
    }
    if !params.is_empty() {
        url = format!("{}?{}", url, params.join("&"));
    }

    let resp = client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .map_err(|e| format!("Version fetch failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Modrinth API error: {}", resp.status()));
    }

    let versions: Vec<serde_json::Value> =
        resp.json().await.map_err(|e| format!("Parse error: {}", e))?;

    let results: Vec<ModVersion> = versions
        .iter()
        .filter_map(|v| {
            let files: Vec<ModFile> = v
                .get("files")
                .and_then(|f| f.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|f| {
                            Some(ModFile {
                                url: f.get("url")?.as_str()?.to_string(),
                                filename: f.get("filename")?.as_str()?.to_string(),
                                size: f.get("size").and_then(|s| s.as_u64()).unwrap_or(0),
                                primary: f
                                    .get("primary")
                                    .and_then(|p| p.as_bool())
                                    .unwrap_or(false),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            Some(ModVersion {
                id: v.get("id")?.as_str()?.to_string(),
                version_number: v.get("version_number")?.as_str()?.to_string(),
                name: v
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string(),
                game_versions: v
                    .get("game_versions")
                    .and_then(|g| g.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default(),
                loaders: v
                    .get("loaders")
                    .and_then(|l| l.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default(),
                downloads: v.get("downloads").and_then(|d| d.as_u64()).unwrap_or(0),
                files,
            })
        })
        .collect();

    Ok(results)
}

/// Download and install a mod file into an instance's mods folder.
pub async fn install_mod(
    download_url: &str,
    filename: &str,
    mods_dir: &Path,
) -> Result<InstallResult, String> {
    std::fs::create_dir_all(mods_dir).map_err(|e| format!("Cannot create mods dir: {}", e))?;

    let dest = mods_dir.join(filename);

    if dest.exists() {
        return Ok(InstallResult {
            success: false,
            filename: filename.to_string(),
            installed_path: dest.to_string_lossy().to_string(),
            message: format!("{} already exists in mods folder", filename),
        });
    }

    let client = reqwest::Client::new();
    let resp = client
        .get(download_url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .map_err(|e| format!("Download failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Download error: HTTP {}", resp.status()));
    }

    let bytes = resp.bytes().await.map_err(|e| format!("Read error: {}", e))?;

    std::fs::write(&dest, &bytes).map_err(|e| format!("Write failed: {}", e))?;

    Ok(InstallResult {
        success: true,
        filename: filename.to_string(),
        installed_path: dest.to_string_lossy().to_string(),
        message: format!(
            "Installed {} ({:.1} MB)",
            filename,
            bytes.len() as f64 / (1024.0 * 1024.0)
        ),
    })
}

/// Remove a mod from the mods folder by renaming it to `.disabled`.
pub fn remove_mod(mods_dir: &Path, filename: &str) -> Result<String, String> {
    let mod_path = mods_dir.join(filename);
    if !mod_path.exists() {
        return Err(format!("{} not found", filename));
    }

    let disabled_name = format!("{}.disabled", filename);
    let disabled_path = mods_dir.join(&disabled_name);
    std::fs::rename(&mod_path, &disabled_path)
        .map_err(|e| format!("Failed to disable mod: {}", e))?;

    Ok(format!(
        "Disabled {} (renamed to {})",
        filename, disabled_name
    ))
}

/// Re-enable a previously disabled mod.
pub fn enable_mod(mods_dir: &Path, disabled_filename: &str) -> Result<String, String> {
    let disabled_path = mods_dir.join(disabled_filename);
    if !disabled_path.exists() {
        return Err(format!("{} not found", disabled_filename));
    }

    let enabled_name = disabled_filename.trim_end_matches(".disabled");
    let enabled_path = mods_dir.join(enabled_name);
    std::fs::rename(&disabled_path, &enabled_path)
        .map_err(|e| format!("Failed to enable mod: {}", e))?;

    Ok(format!("Enabled {}", enabled_name))
}
