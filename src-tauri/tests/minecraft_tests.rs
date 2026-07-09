use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
}

#[test]
fn test_parse_prism_instance() {
    let path = fixtures_dir().join("prism-instance");
    let instance = gamepilot_app_lib::minecraft::instance::parse_instance(&path, "Prism Launcher");

    assert_eq!(instance.launcher, "Prism Launcher");
    assert_eq!(
        instance.minecraft_version.as_deref(),
        Some("1.21.1")
    );
    assert_eq!(
        instance.loader_type.as_deref(),
        Some("NeoForge")
    );
    assert_eq!(
        instance.loader_version.as_deref(),
        Some("21.1.77")
    );
    assert!(instance.xmx_mb == Some(8192));
    assert!(instance.xms_mb == Some(4096));
}

#[test]
fn test_parse_prism_mods() {
    let mods_path = fixtures_dir()
        .join("prism-instance")
        .join(".minecraft")
        .join("mods");
    let analysis = gamepilot_app_lib::minecraft::mods::analyze_mods(&mods_path, Some("NeoForge"));

    assert_eq!(analysis.total_mods, 10);
    assert!(
        analysis.detected_performance_mods.contains(&"ModernFix".to_string()),
        "Should detect ModernFix"
    );
    assert!(
        analysis.detected_performance_mods.contains(&"FerriteCore".to_string()),
        "Should detect FerriteCore"
    );
    assert!(
        analysis.detected_performance_mods.contains(&"Sodium".to_string()),
        "Should detect Sodium"
    );
}

#[test]
fn test_parse_curseforge_instance() {
    let path = fixtures_dir().join("curseforge-instance");
    let instance = gamepilot_app_lib::minecraft::instance::parse_instance(&path, "CurseForge");

    assert_eq!(instance.launcher, "CurseForge");
    assert_eq!(
        instance.minecraft_version.as_deref(),
        Some("1.20.1")
    );
    assert_eq!(instance.loader_type.as_deref(), Some("Forge"));
}

#[test]
fn test_parse_modrinth_instance() {
    let path = fixtures_dir().join("modrinth-instance");
    let instance =
        gamepilot_app_lib::minecraft::instance::parse_instance(&path, "Modrinth App");

    assert_eq!(instance.launcher, "Modrinth App");
    assert_eq!(
        instance.minecraft_version.as_deref(),
        Some("1.21.1")
    );
    assert_eq!(
        instance.loader_type.as_deref(),
        Some("Fabric")
    );
}

#[test]
fn test_parse_empty_instance() {
    let path = fixtures_dir().join("empty-instance");
    let instance = gamepilot_app_lib::minecraft::instance::parse_instance(&path, "Custom");

    assert_eq!(instance.launcher, "Custom");
    assert!(instance.minecraft_version.is_none());
    assert!(instance.loader_type.is_none());
    assert_eq!(instance.mod_count, 0);
}

#[test]
fn test_manual_folder_selection() {
    let path = fixtures_dir().join("manual-folder");
    let instance = gamepilot_app_lib::minecraft::instance::parse_instance(&path, "Custom");

    assert_eq!(instance.mod_count, 5);
    assert!(instance.mods_path.is_some());
    assert!(instance.config_path.is_some());
}

#[test]
fn test_manual_folder_mod_analysis() {
    let mods_path = fixtures_dir().join("manual-folder").join("mods");
    let analysis = gamepilot_app_lib::minecraft::mods::analyze_mods(&mods_path, Some("Fabric"));

    assert_eq!(analysis.total_mods, 5);
    assert!(
        analysis
            .detected_performance_mods
            .contains(&"Sodium".to_string()),
        "Should detect Sodium"
    );
    assert!(
        analysis
            .detected_performance_mods
            .contains(&"Lithium".to_string()),
        "Should detect Lithium"
    );

    let missing_names: Vec<&str> = analysis
        .missing_performance_mods
        .iter()
        .map(|m| m.mod_name.as_str())
        .collect();
    assert!(
        missing_names.contains(&"ModernFix"),
        "Should recommend ModernFix (missing)"
    );
    assert!(
        missing_names.contains(&"FerriteCore"),
        "Should recommend FerriteCore (missing)"
    );
}

#[test]
fn test_recommendations_generation() {
    let hw = gamepilot_app_lib::hardware::HardwareInfo {
        cpu_model: "Test CPU".to_string(),
        cpu_cores: 8,
        cpu_threads: 16,
        cpu_usage_percent: 25.0,
        ram_total_mb: 16384,
        ram_used_mb: 8192,
        ram_available_mb: 8192,
        gpu_model: "Test GPU".to_string(),
        gpu_vram_mb: 8192,
        os_name: "Windows".to_string(),
        os_version: "11".to_string(),
        hostname: "test-pc".to_string(),
    };

    let instance = gamepilot_app_lib::minecraft::instance::MinecraftInstance {
        id: "test".to_string(),
        name: "Test Instance".to_string(),
        path: std::path::PathBuf::from("/test"),
        launcher: "Prism Launcher".to_string(),
        minecraft_version: Some("1.21.1".to_string()),
        loader_type: Some("NeoForge".to_string()),
        loader_version: Some("21.1.77".to_string()),
        mods_path: None,
        mod_count: 250,
        config_path: None,
        resource_packs_path: None,
        shader_packs_path: None,
        java_path: None,
        jvm_args: None,
        xmx_mb: None,
        xms_mb: None,
    };

    let recs = gamepilot_app_lib::minecraft::rules::generate_recommendations(&hw, &instance, None);

    assert!(recs.len() >= 3, "Should generate at least 3 recommendations, got {}", recs.len());

    let categories: Vec<&str> = recs.iter().map(|r| r.category.as_str()).collect();
    assert!(
        categories.contains(&"java_jvm"),
        "Should have JVM recommendations"
    );
}

#[test]
fn test_database_operations() {
    let db = gamepilot_app_lib::db::Database::open_in_memory().unwrap();

    let conn = db.conn();
    conn.execute(
        "INSERT INTO game_instances (id, name, path, launcher) VALUES ('test1', 'Test', '/test', 'Prism')",
        [],
    )
    .unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM game_instances",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}
