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
    fps_avg REAL,
    fps_low_1pct REAL,
    tps_avg REAL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

CREATE TABLE IF NOT EXISTS optimization_actions (
    id TEXT PRIMARY KEY,
    recommendation_id TEXT,
    instance_id TEXT,
    action_type TEXT NOT NULL,
    description TEXT NOT NULL,
    file_path TEXT,
    old_value TEXT,
    new_value TEXT,
    status TEXT NOT NULL DEFAULT 'applied',
    applied_at TEXT NOT NULL DEFAULT (datetime('now')),
    rolled_back_at TEXT,
    FOREIGN KEY (recommendation_id) REFERENCES recommendations(id),
    FOREIGN KEY (instance_id) REFERENCES game_instances(id)
);

CREATE TABLE IF NOT EXISTS launch_profiles (
    id TEXT PRIMARY KEY,
    instance_id TEXT NOT NULL,
    name TEXT NOT NULL DEFAULT 'Default',
    java_path TEXT,
    jvm_args TEXT,
    xmx_mb INTEGER,
    xms_mb INTEGER,
    pre_launch_actions TEXT,
    auto_apply INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (instance_id) REFERENCES game_instances(id)
);

CREATE TABLE IF NOT EXISTS hardware_snapshots (
    id TEXT PRIMARY KEY,
    session_id TEXT,
    device_id TEXT,
    cpu_usage REAL,
    ram_total_mb INTEGER,
    ram_used_mb INTEGER,
    ram_available_mb INTEGER,
    gpu_model TEXT,
    snapshot_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

CREATE TABLE IF NOT EXISTS recommendation_outcomes (
    id TEXT PRIMARY KEY,
    recommendation_id TEXT NOT NULL,
    session_before_id TEXT,
    session_after_id TEXT,
    metric_name TEXT NOT NULL,
    value_before REAL,
    value_after REAL,
    improvement_percent REAL,
    outcome TEXT NOT NULL DEFAULT 'unknown',
    recorded_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (recommendation_id) REFERENCES recommendations(id)
);

-- Clipper: Core clip storage
CREATE TABLE IF NOT EXISTS clips (
    id TEXT PRIMARY KEY,
    session_id TEXT,
    game_id TEXT,
    source_video TEXT NOT NULL,
    clip_path TEXT,
    start_time_ms INTEGER NOT NULL,
    end_time_ms INTEGER NOT NULL,
    duration_ms INTEGER,
    status TEXT NOT NULL DEFAULT 'detected',
    composite_score REAL,
    decima_confidence REAL,
    thumbnail_path TEXT,
    title TEXT,
    transcript TEXT,
    rationale TEXT,
    mood_scores TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

-- Clipper: Detection signals per clip
CREATE TABLE IF NOT EXISTS clip_signals (
    id TEXT PRIMARY KEY,
    clip_id TEXT NOT NULL,
    signal_type TEXT NOT NULL,
    signal_value REAL,
    signal_data TEXT,
    timestamp_ms INTEGER,
    FOREIGN KEY (clip_id) REFERENCES clips(id)
);

-- Clipper: Scoring results
CREATE TABLE IF NOT EXISTS clip_scores (
    id TEXT PRIMARY KEY,
    clip_id TEXT NOT NULL,
    scorer TEXT NOT NULL,
    score_level TEXT,
    confidence REAL,
    escalated INTEGER DEFAULT 0,
    rationale TEXT,
    scored_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (clip_id) REFERENCES clips(id)
);

-- Clipper: User feedback
CREATE TABLE IF NOT EXISTS clip_feedback (
    id TEXT PRIMARY KEY,
    clip_id TEXT NOT NULL,
    action TEXT NOT NULL,
    rating INTEGER,
    rationale_agreed INTEGER,
    written_reason TEXT,
    feedback_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (clip_id) REFERENCES clips(id)
);

-- Clipper: Processing jobs
CREATE TABLE IF NOT EXISTS processing_jobs (
    id TEXT PRIMARY KEY,
    source_video TEXT NOT NULL,
    session_id TEXT,
    game_id TEXT,
    status TEXT NOT NULL DEFAULT 'queued',
    progress REAL DEFAULT 0,
    clips_found INTEGER DEFAULT 0,
    clips_approved INTEGER DEFAULT 0,
    error TEXT,
    started_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Clipper: Distribution log
CREATE TABLE IF NOT EXISTS distribution_log (
    id TEXT PRIMARY KEY,
    clip_id TEXT NOT NULL,
    platform TEXT NOT NULL,
    post_id TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    scheduled_at TEXT,
    posted_at TEXT,
    postiz_post_id TEXT,
    engagement_data TEXT,
    FOREIGN KEY (clip_id) REFERENCES clips(id)
);

-- Clipper: Scoring config per game+profile
CREATE TABLE IF NOT EXISTS scoring_config (
    id TEXT PRIMARY KEY,
    game_id TEXT NOT NULL,
    profile_id TEXT NOT NULL,
    decima_tau REAL DEFAULT 0.8,
    weight_audio REAL DEFAULT 0.15,
    weight_scene REAL DEFAULT 0.10,
    weight_yolo REAL DEFAULT 0.35,
    weight_whisper REAL DEFAULT 0.10,
    weight_vlm REAL DEFAULT 0.30,
    weight_trend REAL DEFAULT 0.0,
    min_clip_duration_ms INTEGER DEFAULT 10000,
    max_clip_duration_ms INTEGER DEFAULT 60000,
    clip_padding_before_ms INTEGER DEFAULT 3000,
    clip_padding_after_ms INTEGER DEFAULT 2000,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Clipper: Recording config
CREATE TABLE IF NOT EXISTS recording_config (
    id TEXT PRIMARY KEY DEFAULT 'default',
    use_builtin_recorder INTEGER DEFAULT 0,
    watch_folders TEXT,
    encoder TEXT DEFAULT 'h264',
    quality_preset TEXT DEFAULT 'high',
    replay_buffer_seconds INTEGER DEFAULT 30,
    hotkey_clip TEXT DEFAULT 'F10',
    hotkey_bookmark TEXT DEFAULT 'F8',
    storage_limit_gb INTEGER DEFAULT 100,
    per_game_overrides TEXT
);

-- Clipper: Compositions (editor state)
CREATE TABLE IF NOT EXISTS compositions (
    id TEXT PRIMARY KEY,
    clip_id TEXT NOT NULL,
    recipe_id TEXT,
    composition_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (clip_id) REFERENCES clips(id)
);

-- Clipper: Recipes
CREATE TABLE IF NOT EXISTS recipes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    is_builtin INTEGER DEFAULT 0,
    recipe_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
"#;
