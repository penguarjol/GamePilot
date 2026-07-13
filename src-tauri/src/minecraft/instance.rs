use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::path::{Path, PathBuf};

fn instance_id_from_path(path: &Path) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(path.to_string_lossy().as_bytes());
    let hash = hasher.finalize();
    format!("inst-{}", hex::encode(&hash[..12]))
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct MinecraftInstance {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub launcher: String,
    pub minecraft_version: Option<String>,
    pub loader_type: Option<String>,
    pub loader_version: Option<String>,
    pub mods_path: Option<PathBuf>,
    pub mod_count: usize,
    pub config_path: Option<PathBuf>,
    pub resource_packs_path: Option<PathBuf>,
    pub shader_packs_path: Option<PathBuf>,
    pub java_path: Option<String>,
    pub jvm_args: Option<String>,
    pub xmx_mb: Option<u32>,
    pub xms_mb: Option<u32>,
}


#[derive(Debug, Deserialize)]
struct MmcPack {
    components: Option<Vec<MmcComponent>>,
}

#[derive(Debug, Deserialize)]
struct MmcComponent {
    uid: Option<String>,
    version: Option<String>,
}

pub fn parse_instance(path: &Path, launcher: &str) -> MinecraftInstance {
    let id = instance_id_from_path(path);
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let mc_dir = find_minecraft_dir(path);
    let mods_path = find_dir(&mc_dir, "mods");
    let config_path = find_dir(&mc_dir, "config");
    let resource_packs_path = find_dir(&mc_dir, "resourcepacks");
    let shader_packs_path = find_dir(&mc_dir, "shaderpacks");

    let mod_count = mods_path
        .as_ref()
        .and_then(|p| std::fs::read_dir(p).ok())
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let name = e.file_name().to_string_lossy().to_lowercase();
                    name.ends_with(".jar") || name.ends_with(".zip")
                })
                .count()
        })
        .unwrap_or(0);

    let (mc_version, loader_type, loader_version) = detect_version_and_loader(path, &mc_dir);
    let (java_path, jvm_args, xmx_mb, xms_mb) = detect_jvm_config(path, launcher);

    MinecraftInstance {
        id,
        name,
        path: path.to_path_buf(),
        launcher: launcher.to_string(),
        minecraft_version: mc_version,
        loader_type,
        loader_version,
        mods_path,
        mod_count,
        config_path,
        resource_packs_path,
        shader_packs_path,
        java_path,
        jvm_args,
        xmx_mb,
        xms_mb,
    }
}

fn find_minecraft_dir(path: &Path) -> PathBuf {
    if path.join(".minecraft").exists() {
        path.join(".minecraft")
    } else if path.join("minecraft").exists() {
        path.join("minecraft")
    } else {
        path.to_path_buf()
    }
}

fn find_dir(base: &Path, name: &str) -> Option<PathBuf> {
    let p = base.join(name);
    if p.exists() && p.is_dir() {
        Some(p)
    } else {
        None
    }
}

fn detect_version_and_loader(
    instance_path: &Path,
    mc_dir: &Path,
) -> (Option<String>, Option<String>, Option<String>) {
    if let Some(result) = try_parse_mmc_pack(instance_path) {
        return result;
    }

    if let Some(result) = try_parse_curseforge_manifest(mc_dir) {
        return result;
    }

    if let Some(result) = try_parse_modrinth_profile(instance_path) {
        return result;
    }

    if let Some(result) = try_detect_from_version_json(mc_dir) {
        return result;
    }

    (None, None, None)
}

fn try_parse_mmc_pack(path: &Path) -> Option<(Option<String>, Option<String>, Option<String>)> {
    let mmc_pack_path = path.join("mmc-pack.json");
    if !mmc_pack_path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&mmc_pack_path).ok()?;
    let pack: MmcPack = serde_json::from_str(&content).ok()?;

    let components = pack.components?;
    let mut mc_version = None;
    let mut loader_type = None;
    let mut loader_version = None;

    for comp in &components {
        let uid = comp.uid.as_deref().unwrap_or_default();
        match uid {
            "net.minecraft" => mc_version = comp.version.clone(),
            "net.minecraftforge" => {
                loader_type = Some("Forge".to_string());
                loader_version = comp.version.clone();
            }
            "net.neoforged" => {
                loader_type = Some("NeoForge".to_string());
                loader_version = comp.version.clone();
            }
            "net.fabricmc.fabric-loader" => {
                loader_type = Some("Fabric".to_string());
                loader_version = comp.version.clone();
            }
            "org.quiltmc.quilt-loader" => {
                loader_type = Some("Quilt".to_string());
                loader_version = comp.version.clone();
            }
            _ => {}
        }
    }

    Some((mc_version, loader_type, loader_version))
}

fn try_parse_curseforge_manifest(
    mc_dir: &Path,
) -> Option<(Option<String>, Option<String>, Option<String>)> {
    let manifest_path = mc_dir.join("minecraftinstance.json");
    if !manifest_path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&manifest_path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;

    let mc_version = v
        .get("gameVersion")
        .or_else(|| v.get("baseModLoader").and_then(|l| l.get("minecraftVersion")))
        .and_then(|v| v.as_str())
        .map(String::from);

    let loader_name = v
        .get("baseModLoader")
        .and_then(|l| l.get("name"))
        .and_then(|v| v.as_str())
        .map(String::from);

    let loader_type = loader_name.as_deref().map(|n| {
        let lower = n.to_lowercase();
        if lower.contains("neoforge") {
            "NeoForge".to_string()
        } else if lower.contains("forge") {
            "Forge".to_string()
        } else if lower.contains("fabric") {
            "Fabric".to_string()
        } else if lower.contains("quilt") {
            "Quilt".to_string()
        } else {
            n.to_string()
        }
    });

    let loader_version = v
        .get("baseModLoader")
        .and_then(|l| l.get("forgeVersion").or_else(|| l.get("name")))
        .and_then(|v| v.as_str())
        .map(String::from);

    Some((mc_version, loader_type, loader_version))
}

fn try_parse_modrinth_profile(
    path: &Path,
) -> Option<(Option<String>, Option<String>, Option<String>)> {
    let profile_path = path.join("profile.json");
    if !profile_path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&profile_path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;

    let mc_version = v
        .get("game_version")
        .and_then(|v| v.as_str())
        .map(String::from);

    let loader_type = v
        .get("loader")
        .and_then(|v| v.as_str())
        .map(|s| {
            let lower = s.to_lowercase();
            if lower.contains("neoforge") {
                "NeoForge".to_string()
            } else if lower.contains("forge") {
                "Forge".to_string()
            } else if lower.contains("fabric") {
                "Fabric".to_string()
            } else if lower.contains("quilt") {
                "Quilt".to_string()
            } else {
                s.to_string()
            }
        });

    let loader_version = v
        .get("loader_version")
        .and_then(|v| v.as_str())
        .map(String::from);

    Some((mc_version, loader_type, loader_version))
}

fn try_detect_from_version_json(
    mc_dir: &Path,
) -> Option<(Option<String>, Option<String>, Option<String>)> {
    let versions_dir = mc_dir.join("versions");
    if !versions_dir.exists() {
        return None;
    }

    if let Ok(entries) = std::fs::read_dir(&versions_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                let json_file = p.join(format!(
                    "{}.json",
                    p.file_name().unwrap_or_default().to_string_lossy()
                ));
                if json_file.exists() {
                    if let Ok(content) = std::fs::read_to_string(&json_file) {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                            let id = v.get("id").and_then(|v| v.as_str()).map(String::from);
                            return Some((id, None, None));
                        }
                    }
                }
            }
        }
    }
    None
}

fn detect_jvm_config(
    instance_path: &Path,
    _launcher: &str,
) -> (Option<String>, Option<String>, Option<u32>, Option<u32>) {
    if let Some(cfg) = parse_instance_cfg(instance_path) {
        return cfg;
    }
    (None, None, None, None)
}

fn parse_instance_cfg(
    path: &Path,
) -> Option<(Option<String>, Option<String>, Option<u32>, Option<u32>)> {
    let cfg_path = path.join("instance.cfg");
    if !cfg_path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&cfg_path).ok()?;

    let mut java_path = None;
    let mut jvm_args = None;
    let mut xmx_mb = None;
    let mut xms_mb = None;

    for line in content.lines() {
        let line = line.trim();
        if let Some((key, value)) = line.split_once('=') {
            match key.trim() {
                "JavaPath" => java_path = Some(value.trim().to_string()),
                "JvmArgs" => jvm_args = Some(value.trim().to_string()),
                "MaxMemAlloc" => xmx_mb = value.trim().parse().ok(),
                "MinMemAlloc" => xms_mb = value.trim().parse().ok(),
                _ => {}
            }
        }
    }

    Some((java_path, jvm_args, xmx_mb, xms_mb))
}
