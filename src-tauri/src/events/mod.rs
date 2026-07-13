use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum GamePilotEvent {
    GameDiscovered { game_id: String, name: String },
    InstanceDiscovered { instance_id: String, name: String, launcher: String },
    InstanceAdded { instance_id: String, name: String },
    InstanceRemoved { instance_id: String },
    GameLaunchRequested { instance_id: String },
    GameLaunched { instance_id: String, session_id: String, method: String },
    GameExited { instance_id: String, session_id: String },
    RecommendationCreated { recommendation_id: String, title: String, category: String },
    RecommendationStatusChanged { recommendation_id: String, old_status: String, new_status: String },
    OptimizationApplied { recommendation_id: String, file_path: String },
    OptimizationRolledBack { recommendation_id: String, file_path: String },
    ModInstalled { instance_id: String, mod_name: String, filename: String },
    ModRemoved { instance_id: String, mod_name: String },
    TelemetryUpdate { session_id: String, cpu_percent: f32, ram_used_mb: u64 },
    GovernorModeChanged { old_mode: String, new_mode: String },
    ProcessAlertDetected { process_name: String, ram_mb: f64, cpu_percent: f32 },
}

/// Emit an event to the frontend via Tauri's event system.
pub fn emit(app: &AppHandle, event: &GamePilotEvent) {
    let event_name = match event {
        GamePilotEvent::GameDiscovered { .. } => "game_discovered",
        GamePilotEvent::InstanceDiscovered { .. } => "instance_discovered",
        GamePilotEvent::InstanceAdded { .. } => "instance_added",
        GamePilotEvent::InstanceRemoved { .. } => "instance_removed",
        GamePilotEvent::GameLaunchRequested { .. } => "game_launch_requested",
        GamePilotEvent::GameLaunched { .. } => "game_launched",
        GamePilotEvent::GameExited { .. } => "game_exited",
        GamePilotEvent::RecommendationCreated { .. } => "recommendation_created",
        GamePilotEvent::RecommendationStatusChanged { .. } => "recommendation_status_changed",
        GamePilotEvent::OptimizationApplied { .. } => "optimization_applied",
        GamePilotEvent::OptimizationRolledBack { .. } => "optimization_rolled_back",
        GamePilotEvent::ModInstalled { .. } => "mod_installed",
        GamePilotEvent::ModRemoved { .. } => "mod_removed",
        GamePilotEvent::TelemetryUpdate { .. } => "telemetry_update",
        GamePilotEvent::GovernorModeChanged { .. } => "governor_mode_changed",
        GamePilotEvent::ProcessAlertDetected { .. } => "process_alert",
    };

    let _ = app.emit(event_name, event);
}
