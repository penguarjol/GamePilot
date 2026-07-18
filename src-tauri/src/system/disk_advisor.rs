use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DiskAdvice {
    pub instance_drive: Option<DriveStatus>,
    pub best_drive: Option<DriveRecommendation>,
    pub paging_file: Option<PagingFileStatus>,
    pub warnings: Vec<DiskWarning>,
    pub recommendations: Vec<DiskRecommendation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DriveStatus {
    pub mount_point: String,
    pub total_gb: f64,
    pub free_gb: f64,
    pub used_percent: f64,
    pub is_critical: bool,
    pub storage_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DriveRecommendation {
    pub mount_point: String,
    pub free_gb: f64,
    pub storage_type: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PagingFileStatus {
    pub current_size_mb: u64,
    pub max_size_mb: u64,
    pub drive: String,
    pub is_system_managed: bool,
    pub is_adequate: bool,
    pub recommended_size_mb: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskWarning {
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskRecommendation {
    pub title: String,
    pub description: String,
    pub action: String,
    pub priority: String,
    pub estimated_gain_gb: Option<f64>,
}

pub fn analyze_disk_for_instance(instance_path: &str, xmx_mb: u32) -> DiskAdvice {
    let disks = crate::hardware::collect_hardware_info().disks;

    let instance_drive = disks.iter().find(|d| {
        instance_path
            .to_lowercase()
            .starts_with(&d.mount_point.to_lowercase())
    });

    let instance_status = instance_drive.map(|d| DriveStatus {
        mount_point: d.mount_point.clone(),
        total_gb: d.total_gb,
        free_gb: d.free_gb,
        used_percent: if d.total_gb > 0.0 {
            (d.total_gb - d.free_gb) / d.total_gb * 100.0
        } else {
            0.0
        },
        is_critical: d.free_gb < 20.0,
        storage_type: d.storage_type.clone(),
    });

    let mut warnings = Vec::new();
    let mut recommendations = Vec::new();

    // Detect OS drive anti-pattern: instance on a small system drive
    if let Some(ref status) = instance_status {
        let is_os_drive = status.mount_point.to_uppercase().starts_with("C");
        let is_small_drive = status.total_gb < 512.0;
        if is_os_drive && is_small_drive {
            warnings.push(DiskWarning {
                severity: "warning".to_string(),
                message: format!(
                    "This instance is on your OS drive ({}, {:.0} GB). \
                     On a system drive this size, Minecraft modpacks and their paging file \
                     requirements can fill the drive and cause crashes. \
                     Move this instance to a data drive for better stability.",
                    status.mount_point, status.total_gb
                ),
            });
        }
    }

    if let Some(ref status) = instance_status {
        if status.free_gb < 10.0 {
            warnings.push(DiskWarning {
                severity: "critical".to_string(),
                message: format!(
                    "{} has only {:.1} GB free. Minecraft and Java need disk space for paging, temp files, and world saves.",
                    status.mount_point, status.free_gb
                ),
            });
        } else if status.free_gb < 20.0 {
            warnings.push(DiskWarning {
                severity: "warning".to_string(),
                message: format!(
                    "{} has {:.1} GB free. Consider freeing space or moving the instance to a drive with more room.",
                    status.mount_point, status.free_gb
                ),
            });
        }
    }

    let best_drive = disks
        .iter()
        .filter(|d| {
            instance_drive
                .map(|id| d.mount_point != id.mount_point)
                .unwrap_or(true)
        })
        .filter(|d| d.free_gb > 50.0 && !d.is_removable)
        .max_by(|a, b| {
            let a_score = a.free_gb + if a.storage_type == "SSD" { 1000.0 } else { 0.0 };
            let b_score = b.free_gb + if b.storage_type == "SSD" { 1000.0 } else { 0.0 };
            a_score
                .partial_cmp(&b_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|d| DriveRecommendation {
            mount_point: d.mount_point.clone(),
            free_gb: d.free_gb,
            storage_type: d.storage_type.clone(),
            reason: if instance_drive
                .map(|id| id.free_gb < 20.0)
                .unwrap_or(false)
            {
                format!(
                    "{} has {:.0} GB free ({}) — moving your instance here would resolve disk space issues",
                    d.mount_point, d.free_gb, d.storage_type
                )
            } else {
                format!(
                    "{} has {:.0} GB free ({}) — a good alternative location",
                    d.mount_point, d.free_gb, d.storage_type
                )
            },
        });

    if instance_status
        .as_ref()
        .map(|s| s.is_critical)
        .unwrap_or(false)
    {
        if let Some(ref best) = best_drive {
            recommendations.push(DiskRecommendation {
                title: format!("Move instance to {}", best.mount_point),
                description: format!(
                    "Your instance is on a nearly full drive. {} has {:.0} GB free and is {}.",
                    best.mount_point, best.free_gb, best.storage_type
                ),
                action: "migrate_instance".to_string(),
                priority: "critical".to_string(),
                estimated_gain_gb: None,
            });
        }
    }

    let paging_file = detect_paging_file(xmx_mb);

    if let Some(ref pf) = paging_file {
        if !pf.is_adequate {
            recommendations.push(DiskRecommendation {
                title: "Increase paging file size".to_string(),
                description: format!(
                    "Your paging file is {} MB but your Minecraft allocation is {} MB. \
                     Windows needs a paging file at least 1.5x your RAM allocation. \
                     Recommended: {} MB.",
                    pf.current_size_mb, xmx_mb, pf.recommended_size_mb
                ),
                action: "open_virtual_memory".to_string(),
                priority: "high".to_string(),
                estimated_gain_gb: None,
            });
        }
    }

    recommendations.push(DiskRecommendation {
        title: "Run Windows Disk Cleanup".to_string(),
        description: "Clear temporary files, browser cache, and old Windows updates to free disk space.".to_string(),
        action: "open_disk_cleanup".to_string(),
        priority: "medium".to_string(),
        estimated_gain_gb: Some(5.0),
    });

    DiskAdvice {
        instance_drive: instance_status,
        best_drive,
        paging_file,
        warnings,
        recommendations,
    }
}

fn detect_paging_file(xmx_mb: u32) -> Option<PagingFileStatus> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;

        let output = std::process::Command::new("wmic")
            .args([
                "pagefile",
                "get",
                "CurrentUsage,MaximumSize,Name",
                "/format:csv",
            ])
            .creation_flags(0x08000000)
            .output()
            .ok()?;

        let text = String::from_utf8_lossy(&output.stdout);
        let data_line = text
            .lines()
            .filter(|l| !l.trim().is_empty())
            .find(|l| !l.starts_with("Node"))?;

        let parts: Vec<&str> = data_line.split(',').collect();
        // CSV: Node, CurrentUsage, MaximumSize, Name
        let current = parts
            .get(1)
            .and_then(|v| v.trim().parse::<u64>().ok())
            .unwrap_or(0);
        let max_size = parts
            .get(2)
            .and_then(|v| v.trim().parse::<u64>().ok())
            .unwrap_or(0);
        let drive = parts
            .get(3)
            .and_then(|v| v.trim().chars().next())
            .map(|c| format!("{}:", c))
            .unwrap_or_default();

        let recommended = (xmx_mb as u64 * 3) / 2;

        Some(PagingFileStatus {
            current_size_mb: current,
            max_size_mb: max_size,
            drive,
            is_system_managed: max_size == 0,
            // System-managed is usually OK if drive has space
            is_adequate: max_size >= recommended || max_size == 0,
            recommended_size_mb: recommended,
        })
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = xmx_mb;
        None
    }
}
