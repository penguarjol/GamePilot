use serde::Serialize;
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[repr(u8)]
pub enum GovernorMode {
    Normal = 0,
    Lite = 1,
    Minimal = 2,
    Paused = 3,
}

impl From<u8> for GovernorMode {
    fn from(v: u8) -> Self {
        match v {
            1 => GovernorMode::Lite,
            2 => GovernorMode::Minimal,
            3 => GovernorMode::Paused,
            _ => GovernorMode::Normal,
        }
    }
}

static GOVERNOR_MODE: AtomicU8 = AtomicU8::new(0);
static VISION_MODE: AtomicU8 = AtomicU8::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[repr(u8)]
pub enum VisionMode {
    Active = 0,
    Reduced = 1,
    Minimal = 2,
    Disabled = 3,
}

impl From<u8> for VisionMode {
    fn from(v: u8) -> Self {
        match v {
            1 => VisionMode::Reduced,
            2 => VisionMode::Minimal,
            3 => VisionMode::Disabled,
            _ => VisionMode::Active,
        }
    }
}

pub fn current_vision_mode() -> VisionMode {
    VisionMode::from(VISION_MODE.load(Ordering::Relaxed))
}

pub fn set_vision_mode(mode: VisionMode) {
    VISION_MODE.store(mode as u8, Ordering::Relaxed);
}

pub fn capture_interval_ms() -> u64 {
    match current_vision_mode() {
        VisionMode::Active => 3000,
        VisionMode::Reduced => 10000,
        VisionMode::Minimal => 30000,
        VisionMode::Disabled => u64::MAX,
    }
}

pub fn current_mode() -> GovernorMode {
    GovernorMode::from(GOVERNOR_MODE.load(Ordering::Relaxed))
}

pub fn set_mode(mode: GovernorMode) {
    GOVERNOR_MODE.store(mode as u8, Ordering::Relaxed);
}

pub struct Budget {
    pub cpu_limit: f32,
    pub ram_limit_mb: f64,
}

pub fn budget_for_mode(mode: GovernorMode) -> Budget {
    match mode {
        GovernorMode::Normal => Budget { cpu_limit: 3.0, ram_limit_mb: 500.0 },
        GovernorMode::Lite => Budget { cpu_limit: 1.0, ram_limit_mb: 250.0 },
        GovernorMode::Minimal => Budget { cpu_limit: 0.5, ram_limit_mb: 200.0 },
        GovernorMode::Paused => Budget { cpu_limit: 0.25, ram_limit_mb: 150.0 },
    }
}

pub fn evaluate(self_cpu: f32, self_ram_mb: f64, game_running: bool) -> GovernorMode {
    if !game_running {
        set_mode(GovernorMode::Normal);
        set_vision_mode(VisionMode::Active);
        return GovernorMode::Normal;
    }

    let mode = if self_cpu > 3.0 || self_ram_mb > 500.0 {
        GovernorMode::Paused
    } else if self_cpu > 1.0 || self_ram_mb > 250.0 {
        GovernorMode::Minimal
    } else if self_cpu > 0.5 || self_ram_mb > 200.0 {
        GovernorMode::Lite
    } else {
        GovernorMode::Normal
    };

    let vision = if self_cpu > 2.0 {
        VisionMode::Disabled
    } else if self_cpu > 1.0 {
        VisionMode::Minimal
    } else {
        VisionMode::Reduced
    };

    set_mode(mode);
    set_vision_mode(vision);
    mode
}

pub fn telemetry_interval_ms() -> u64 {
    match current_mode() {
        GovernorMode::Normal => 5000,
        GovernorMode::Lite => 10000,
        GovernorMode::Minimal => 30000,
        GovernorMode::Paused => 60000,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GovernorStatus {
    pub mode: String,
    pub self_cpu: f32,
    pub self_ram_mb: f64,
    pub telemetry_interval_ms: u64,
    pub game_running: bool,
    pub vision_mode: String,
    pub capture_interval_ms: u64,
}
