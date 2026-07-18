use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct CrashDiagnosis {
    pub crash_detected: bool,
    pub crash_type: Option<String>,
    pub summary: String,
    pub details: Vec<String>,
    pub recommendations: Vec<CrashRecommendation>,
    pub crash_file: Option<String>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CrashRecommendation {
    pub title: String,
    pub description: String,
    pub action: String,
    pub priority: String,
}

pub fn analyze_crashes(instance_path: &Path) -> CrashDiagnosis {
    let mc_dir = resolve_mc_dir(instance_path);

    let crash_report = find_latest_crash_report(&mc_dir);
    let log_errors = scan_latest_log(&mc_dir);
    let jvm_crash = find_jvm_crash_log(instance_path);

    build_diagnosis(crash_report, log_errors, jvm_crash)
}

fn resolve_mc_dir(path: &Path) -> std::path::PathBuf {
    if path.join(".minecraft").exists() {
        path.join(".minecraft")
    } else if path.join("minecraft").exists() {
        path.join("minecraft")
    } else {
        path.to_path_buf()
    }
}

fn find_latest_crash_report(mc_dir: &Path) -> Option<(String, String)> {
    let crash_dir = mc_dir.join("crash-reports");
    if !crash_dir.exists() {
        return None;
    }

    let mut entries: Vec<_> = std::fs::read_dir(&crash_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".txt"))
        .collect();

    entries.sort_by(|a, b| {
        b.metadata()
            .and_then(|m| m.modified())
            .ok()
            .cmp(&a.metadata().and_then(|m| m.modified()).ok())
    });

    let latest = entries.first()?;
    let content = std::fs::read_to_string(latest.path()).ok()?;
    let filename = latest.file_name().to_string_lossy().to_string();
    Some((filename, content))
}

fn scan_latest_log(mc_dir: &Path) -> Vec<String> {
    let log_path = mc_dir.join("logs").join("latest.log");
    let content = match std::fs::read_to_string(&log_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut errors = Vec::new();
    for line in content.lines().rev().take(500) {
        let lower = line.to_lowercase();
        if lower.contains("outofmemoryerror")
            || lower.contains("insufficient memory")
            || lower.contains("paging file")
            || lower.contains("cannot allocate memory")
            || lower.contains("gc overhead limit")
            || lower.contains("exception_access_violation")
            || lower.contains("fatal error")
            || lower.contains("crash report")
            || (lower.contains("error") && lower.contains("java.lang"))
        {
            errors.push(line.trim().to_string());
        }
    }
    errors.truncate(20);
    errors
}

fn find_jvm_crash_log(instance_path: &Path) -> Option<String> {
    let entries = std::fs::read_dir(instance_path).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("hs_err_pid") && name.ends_with(".log") {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                return Some(content.lines().take(100).collect::<Vec<_>>().join("\n"));
            }
        }
    }
    None
}

fn build_diagnosis(
    crash_report: Option<(String, String)>,
    log_errors: Vec<String>,
    jvm_crash: Option<String>,
) -> CrashDiagnosis {
    let mut recommendations = Vec::new();
    let mut details = Vec::new();
    let mut crash_type = None;
    let mut crash_file = None;

    let all_text = format!(
        "{} {} {}",
        crash_report.as_ref().map(|(_, c)| c.as_str()).unwrap_or(""),
        log_errors.join(" "),
        jvm_crash.as_deref().unwrap_or("")
    )
    .to_lowercase();

    if let Some((ref name, _)) = crash_report {
        crash_file = Some(name.clone());
    }

    if all_text.contains("outofmemoryerror") || all_text.contains("insufficient memory") {
        crash_type = Some("Out of Memory".to_string());
        details.push("Java ran out of memory (OutOfMemoryError).".to_string());

        recommendations.push(CrashRecommendation {
            title: "Increase RAM allocation".to_string(),
            description: "Your modpack needs more RAM than currently allocated. Increase Xmx in JVM settings.".to_string(),
            action: "apply_jvm".to_string(),
            priority: "critical".to_string(),
        });

        if all_text.contains("metaspace") {
            details.push("Specifically ran out of Metaspace (class loading area).".to_string());
            recommendations.push(CrashRecommendation {
                title: "Increase Metaspace".to_string(),
                description: "Add -XX:MaxMetaspaceSize=512m to JVM arguments.".to_string(),
                action: "apply_jvm".to_string(),
                priority: "high".to_string(),
            });
        }
    }

    if all_text.contains("paging file")
        || all_text.contains("pagefile")
        || all_text.contains("commit limit")
    {
        crash_type = Some("Paging File Exhausted".to_string());
        details.push(
            "Windows virtual memory (paging file) is too small or the disk it lives on is full."
                .to_string(),
        );

        recommendations.push(CrashRecommendation {
            title: "Free disk space on C: drive".to_string(),
            description: "The paging file needs room to expand. Clear temporary files or move large applications to another drive.".to_string(),
            action: "disk_cleanup".to_string(),
            priority: "critical".to_string(),
        });
        recommendations.push(CrashRecommendation {
            title: "Move instance to a drive with more space".to_string(),
            description: "If your Minecraft instance is on a nearly full drive, moving it to a drive with more free space prevents paging file issues.".to_string(),
            action: "migrate_instance".to_string(),
            priority: "high".to_string(),
        });
        recommendations.push(CrashRecommendation {
            title: "Increase paging file size".to_string(),
            description: "Open System Properties > Advanced > Performance Settings > Advanced > Virtual Memory. Set a custom paging file on a drive with space (e.g., D:).".to_string(),
            action: "open_settings".to_string(),
            priority: "high".to_string(),
        });
    }

    if all_text.contains("gc overhead limit") {
        crash_type = crash_type.or(Some("GC Overhead".to_string()));
        details.push(
            "Java spent too much time garbage collecting. The heap is nearly full.".to_string(),
        );
        recommendations.push(CrashRecommendation {
            title: "Increase RAM and optimize GC flags".to_string(),
            description: "Increase Xmx by 2-4 GB and ensure G1GC is enabled with tuned parameters."
                .to_string(),
            action: "apply_jvm".to_string(),
            priority: "high".to_string(),
        });
    }

    if all_text.contains("mixin") && all_text.contains("conflict") {
        crash_type = crash_type.or(Some("Mod Conflict".to_string()));
        details.push(
            "A mixin conflict between mods was detected. Two mods are trying to modify the same game code.".to_string(),
        );
        recommendations.push(CrashRecommendation {
            title: "Check for incompatible mods".to_string(),
            description:
                "Run mod analysis to detect known conflicts. Try removing recently added mods."
                    .to_string(),
            action: "analyze_mods".to_string(),
            priority: "high".to_string(),
        });
    }

    if all_text.contains("mod")
        && (all_text.contains("requires")
            || all_text.contains("missing")
            || all_text.contains("not found"))
    {
        crash_type = crash_type.or(Some("Missing Dependency".to_string()));
        details.push("A mod is missing a required dependency.".to_string());
        recommendations.push(CrashRecommendation {
            title: "Check mod dependencies".to_string(),
            description: "A required library or dependency mod is missing. Check the crash report for the specific mod name.".to_string(),
            action: "analyze_mods".to_string(),
            priority: "high".to_string(),
        });
    }

    if all_text.contains("exception_access_violation") {
        crash_type = crash_type.or(Some("Access Violation".to_string()));
        details.push(
            "A native code crash occurred (EXCEPTION_ACCESS_VIOLATION). This can be caused by GPU drivers, bad JVM, or native mod libraries.".to_string(),
        );
        recommendations.push(CrashRecommendation {
            title: "Update GPU drivers".to_string(),
            description:
                "Access violations are often caused by outdated GPU drivers. Update to the latest version."
                    .to_string(),
            action: "open_link".to_string(),
            priority: "high".to_string(),
        });
    }

    for error in &log_errors {
        let prefix = &error[..error.len().min(50)];
        if !details.iter().any(|d| d.contains(prefix)) {
            details.push(error.clone());
        }
    }

    let crash_detected = crash_type.is_some() || !log_errors.is_empty();
    let summary = if let Some(ref ct) = crash_type {
        format!("Crash detected: {}", ct)
    } else if !log_errors.is_empty() {
        format!("{} error(s) found in game log", log_errors.len())
    } else {
        "No crashes or errors detected.".to_string()
    };

    CrashDiagnosis {
        crash_detected,
        crash_type,
        summary,
        details,
        recommendations,
        crash_file,
        timestamp: Some(chrono::Utc::now().to_rfc3339()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_no_crashes_on_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let diag = analyze_crashes(tmp.path());
        assert!(!diag.crash_detected);
        assert!(diag.crash_type.is_none());
        assert!(diag.recommendations.is_empty());
    }

    #[test]
    fn test_detects_oom_in_log() {
        let tmp = TempDir::new().unwrap();
        let logs_dir = tmp.path().join("logs");
        fs::create_dir_all(&logs_dir).unwrap();
        fs::write(
            logs_dir.join("latest.log"),
            "[12:00:00] [Server thread/ERROR]: java.lang.OutOfMemoryError: Java heap space\n",
        )
        .unwrap();

        let diag = analyze_crashes(tmp.path());
        assert!(diag.crash_detected);
        assert_eq!(diag.crash_type.as_deref(), Some("Out of Memory"));
        assert!(diag.recommendations.iter().any(|r| r.action == "apply_jvm"));
    }

    #[test]
    fn test_detects_crash_report_file() {
        let tmp = TempDir::new().unwrap();
        let crash_dir = tmp.path().join("crash-reports");
        fs::create_dir_all(&crash_dir).unwrap();
        fs::write(
            crash_dir.join("crash-2024-01-01_12.00.00-server.txt"),
            "---- Minecraft Crash Report ----\nDescription: Exception in server tick loop\njava.lang.OutOfMemoryError: GC overhead limit exceeded\n",
        ).unwrap();

        let diag = analyze_crashes(tmp.path());
        assert!(diag.crash_detected);
        assert!(diag.crash_file.is_some());
        assert!(
            diag.crash_type.as_deref() == Some("Out of Memory")
                || diag.crash_type.as_deref() == Some("GC Overhead")
        );
    }

    #[test]
    fn test_detects_access_violation() {
        let tmp = TempDir::new().unwrap();
        let logs_dir = tmp.path().join("logs");
        fs::create_dir_all(&logs_dir).unwrap();
        fs::write(
            logs_dir.join("latest.log"),
            "[12:00:00] [Render thread/FATAL]: EXCEPTION_ACCESS_VIOLATION (0xc0000005)\n",
        )
        .unwrap();

        let diag = analyze_crashes(tmp.path());
        assert!(diag.crash_detected);
        assert_eq!(diag.crash_type.as_deref(), Some("Access Violation"));
        assert!(diag
            .recommendations
            .iter()
            .any(|r| r.title.contains("GPU drivers")));
    }

    #[test]
    fn test_detects_jvm_crash_log() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("hs_err_pid12345.log"),
            "# A fatal error has been detected by the Java Runtime Environment:\n#\n# EXCEPTION_ACCESS_VIOLATION\n",
        ).unwrap();

        let diag = analyze_crashes(tmp.path());
        assert!(diag.crash_detected);
        assert_eq!(diag.crash_type.as_deref(), Some("Access Violation"));
    }

    #[test]
    fn test_resolves_dot_minecraft_subdir() {
        let tmp = TempDir::new().unwrap();
        let mc_dir = tmp.path().join(".minecraft").join("logs");
        fs::create_dir_all(&mc_dir).unwrap();
        fs::write(
            mc_dir.join("latest.log"),
            "[12:00:00] [main/ERROR]: Fatal error: insufficient memory\n",
        )
        .unwrap();

        let diag = analyze_crashes(tmp.path());
        assert!(diag.crash_detected);
        assert_eq!(diag.crash_type.as_deref(), Some("Out of Memory"));
    }
}
