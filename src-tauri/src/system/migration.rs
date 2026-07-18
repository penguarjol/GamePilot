use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct MigrationResult {
    pub success: bool,
    pub old_path: String,
    pub new_path: String,
    pub files_copied: usize,
    pub total_size_mb: f64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MigrationProgress {
    pub files_copied: usize,
    pub total_files: usize,
    pub bytes_copied: u64,
    pub current_file: String,
}

pub fn migrate_instance(
    source_path: &Path,
    target_dir: &Path,
) -> Result<MigrationResult, String> {
    if !source_path.exists() {
        return Err(format!(
            "Source path does not exist: {}",
            source_path.display()
        ));
    }

    let instance_name = source_path
        .file_name()
        .ok_or("Invalid source path")?
        .to_string_lossy()
        .to_string();

    let new_path = target_dir.join(&instance_name);

    if new_path.exists() {
        return Err(format!(
            "Target already exists: {}",
            new_path.display()
        ));
    }

    let total_files = count_files(source_path)?;
    let (files_copied, bytes_copied) = copy_dir_recursive(source_path, &new_path)?;

    let verify_count = count_files(&new_path)?;
    if verify_count != total_files {
        return Err(format!(
            "Verification failed: source has {} files but copy has {}. The copy at {} was NOT deleted — verify manually.",
            total_files, verify_count, new_path.display()
        ));
    }

    let size_mb = bytes_copied as f64 / (1024.0 * 1024.0);

    Ok(MigrationResult {
        success: true,
        old_path: source_path.to_string_lossy().to_string(),
        new_path: new_path.to_string_lossy().to_string(),
        files_copied,
        total_size_mb: size_mb,
        message: format!(
            "Successfully migrated '{}' to {}. {} files ({:.1} MB) copied and verified.",
            instance_name,
            target_dir.display(),
            files_copied,
            size_mb
        ),
    })
}

fn count_files(dir: &Path) -> Result<usize, String> {
    let mut count = 0;
    for entry in walkdir::WalkDir::new(dir).into_iter().flatten() {
        if entry.file_type().is_file() {
            count += 1;
        }
    }
    Ok(count)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(usize, u64), String> {
    std::fs::create_dir_all(dst).map_err(|e| format!("Cannot create target dir: {}", e))?;

    let mut files_copied = 0;
    let mut bytes_copied: u64 = 0;

    for entry in walkdir::WalkDir::new(src).into_iter() {
        let entry = entry.map_err(|e| format!("Walk error: {}", e))?;
        let relative = entry
            .path()
            .strip_prefix(src)
            .map_err(|e| format!("Path error: {}", e))?;
        let target = dst.join(relative);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)
                .map_err(|e| format!("mkdir error: {}", e))?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            let size = std::fs::copy(entry.path(), &target)
                .map_err(|e| format!("Copy failed for {}: {}", relative.display(), e))?;
            files_copied += 1;
            bytes_copied += size;
        }
    }

    Ok((files_copied, bytes_copied))
}

/// Delete the old instance after successful migration (caller must confirm with user first).
pub fn delete_old_instance(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    std::fs::remove_dir_all(path)
        .map_err(|e| format!("Failed to remove old instance: {}", e))
}
