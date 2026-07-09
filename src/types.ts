export interface HardwareInfo {
  cpu_model: string;
  cpu_cores: number;
  cpu_threads: number;
  cpu_usage_percent: number;
  cpu_freq_mhz: number;
  ram_total_mb: number;
  ram_used_mb: number;
  ram_available_mb: number;
  gpu_model: string;
  gpu_vram_mb: number;
  gpu_driver_version: string;
  disks: DiskInfo[];
  os_name: string;
  os_version: string;
  hostname: string;
}

export interface DiskInfo {
  name: string;
  mount_point: string;
  total_gb: number;
  free_gb: number;
  is_removable: boolean;
}

export interface ProcessInfo {
  name: string;
  pid: number;
  cpu_percent: number;
  ram_mb: number;
  category: string;
  is_resource_hog: boolean;
  recommendation: string;
}

export interface TelemetrySample {
  timestamp: string;
  cpu_percent: number;
  ram_used_mb: number;
  ram_available_mb: number;
  top_processes: ProcessSnapshot[];
}

export interface ProcessSnapshot {
  name: string;
  pid: number;
  cpu_percent: number;
  ram_mb: number;
}

export interface SelfMetrics {
  cpu_percent: number;
  ram_mb: number;
}

export interface DiscoveredLauncher {
  name: string;
  path: string;
  launcher_type: string;
}

export interface MinecraftInstance {
  id: string;
  name: string;
  path: string;
  launcher: string;
  minecraft_version: string | null;
  loader_type: string | null;
  loader_version: string | null;
  mods_path: string | null;
  mod_count: number;
  config_path: string | null;
  resource_packs_path: string | null;
  shader_packs_path: string | null;
  java_path: string | null;
  jvm_args: string | null;
  xmx_mb: number | null;
  xms_mb: number | null;
}

export interface ModInfo {
  file_name: string;
  mod_id: string | null;
  display_name: string | null;
  version: string | null;
  size_bytes: number;
}

export interface ModAnalysis {
  total_mods: number;
  mods: ModInfo[];
  detected_performance_mods: string[];
  missing_performance_mods: PerformanceModRecommendation[];
  total_size_mb: number;
}

export interface PerformanceModRecommendation {
  mod_name: string;
  mod_id: string;
  reason: string;
  expected_impact: string;
  confidence: string;
  url: string;
  loaders: string[];
}

export interface Recommendation {
  id: string;
  category: string;
  severity: string;
  confidence: string;
  title: string;
  description: string;
  evidence: string;
  expected_impact: string;
  risk_level: string;
  action_type: string | null;
  action_data: string | null;
}

export interface ConfigAnalysis {
  options: OptionsAnalysis | null;
  server_properties: ServerPropertiesAnalysis | null;
  recommendations: ConfigRecommendation[];
}

export interface OptionsAnalysis {
  render_distance: number | null;
  simulation_distance: number | null;
  max_framerate: number | null;
  graphics_level: string | null;
  gui_scale: number | null;
  vsync: boolean | null;
  entity_shadows: boolean | null;
  fullscreen: boolean | null;
  raw_entries: Record<string, string>;
}

export interface ServerPropertiesAnalysis {
  view_distance: number | null;
  simulation_distance: number | null;
  max_players: number | null;
  spawn_protection: number | null;
  max_tick_time: number | null;
  network_compression_threshold: number | null;
  raw_entries: Record<string, string>;
}

export interface ConfigRecommendation {
  file: string;
  key: string;
  current_value: string;
  recommended_value: string;
  reason: string;
  impact: string;
  confidence: string;
}

export interface ModpackHealth {
  overall_score: number;
  memory_risk: RiskScore;
  rendering_risk: RiskScore;
  startup_risk: RiskScore;
  dependency_risk: RiskScore;
  optimization_score: RiskScore;
  summary: string;
}

export interface RiskScore {
  score: number;
  label: string;
  detail: string;
}

export interface JavaInstallation {
  path: string;
  version: string | null;
  vendor: string | null;
  is_64bit: boolean;
}

export interface LaunchResult {
  success: boolean;
  method: string;
  message: string;
  session_id: string | null;
}

export interface Session {
  id: string;
  instance_id: string;
  started_at: string;
  ended_at: string | null;
  duration_secs: number | null;
  launch_method: string | null;
  cpu_avg_percent: number | null;
  ram_avg_mb: number | null;
  ram_peak_mb: number | null;
  status: string;
  notes: string | null;
}

export interface SessionReport {
  session: Session;
  recommendations_applied: number;
  process_observations: number;
  summary: string;
}

export interface RollbackPoint {
  id: string;
  recommendation_id: string;
  file_path: string;
  original_hash: string;
  backup_path: string;
  created_at: string;
}

export interface SavedInstance {
  id: string;
  name: string;
  path: string;
  launcher: string | null;
  minecraft_version: string | null;
  loader_type: string | null;
  loader_version: string | null;
  mod_count: number | null;
  last_played_at: string | null;
}

export interface IgnoreRule {
  id: string;
  rule_type: string;
  pattern: string;
  reason: string | null;
  created_at: string;
}

export interface DiscoveredInstance {
  name: string;
  path: string;
  launcher: string;
  minecraft_version: string | null;
  loader_type: string | null;
  mod_count: number;
}

export interface DiscoveredInstance {
  name: string;
  path: string;
  launcher: string;
  minecraft_version: string | null;
  loader_type: string | null;
  mod_count: number;
}
