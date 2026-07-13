pub const MIGRATIONS: &str = r#"
CREATE TABLE IF NOT EXISTS devices (
    id TEXT PRIMARY KEY,
    hostname TEXT,
    os_name TEXT,
    os_version TEXT,
    cpu_model TEXT,
    cpu_cores INTEGER,
    cpu_threads INTEGER,
    gpu_model TEXT,
    gpu_vram_mb INTEGER,
    ram_total_mb INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS game_instances (
    id TEXT PRIMARY KEY,
    game_type TEXT NOT NULL DEFAULT 'minecraft',
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    launcher TEXT,
    minecraft_version TEXT,
    loader_type TEXT,
    loader_version TEXT,
    java_path TEXT,
    jvm_args TEXT,
    xmx_mb INTEGER,
    xms_mb INTEGER,
    mod_count INTEGER DEFAULT 0,
    last_played_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    instance_id TEXT NOT NULL,
    started_at TEXT NOT NULL,
    ended_at TEXT,
    duration_secs INTEGER,
    launch_method TEXT,
    cpu_avg_percent REAL,
    ram_avg_mb REAL,
    ram_peak_mb REAL,
    status TEXT NOT NULL DEFAULT 'active',
    notes TEXT,
    FOREIGN KEY (instance_id) REFERENCES game_instances(id)
);

CREATE TABLE IF NOT EXISTS recommendations (
    id TEXT PRIMARY KEY,
    instance_id TEXT,
    session_id TEXT,
    category TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'info',
    confidence TEXT NOT NULL DEFAULT 'medium',
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    evidence TEXT,
    expected_impact TEXT,
    risk_level TEXT NOT NULL DEFAULT 'low',
    action_type TEXT,
    action_data TEXT,
    rollback_data TEXT,
    status TEXT NOT NULL DEFAULT 'new',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (instance_id) REFERENCES game_instances(id),
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

CREATE TABLE IF NOT EXISTS rollback_points (
    id TEXT PRIMARY KEY,
    recommendation_id TEXT NOT NULL,
    file_path TEXT NOT NULL,
    original_content TEXT,
    original_hash TEXT,
    backup_path TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    restored_at TEXT,
    FOREIGN KEY (recommendation_id) REFERENCES recommendations(id)
);

CREATE TABLE IF NOT EXISTS process_observations (
    id TEXT PRIMARY KEY,
    session_id TEXT,
    name TEXT NOT NULL,
    pid INTEGER,
    cpu_percent REAL,
    ram_mb REAL,
    category TEXT,
    is_resource_hog INTEGER DEFAULT 0,
    observed_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

CREATE TABLE IF NOT EXISTS user_preferences (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS ignore_rules (
    id TEXT PRIMARY KEY,
    rule_type TEXT NOT NULL,
    pattern TEXT NOT NULL,
    reason TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS telemetry_summaries (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    minute_ts TEXT NOT NULL,
    cpu_avg REAL,
    ram_avg_mb REAL,
    ram_peak_mb REAL,
    hog_count INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);
"#;
