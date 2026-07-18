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
  display_refresh_hz: number;
  windows_gaming: WindowsGamingSettings | null;
}

export interface WindowsGamingSettings {
  game_mode_enabled: boolean | null;
  hardware_accelerated_gpu_scheduling: boolean | null;
  variable_refresh_rate: boolean | null;
}

export interface DiskInfo {
  name: string;
  mount_point: string;
  total_gb: number;
  free_gb: number;
  is_removable: boolean;
  storage_type: string;
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
  conflicts: ConflictWarning[];
  duplicates: DuplicateWarning[];
  total_size_mb: number;
}

export interface ConflictWarning {
  mod_a: string;
  mod_b: string;
  reason: string;
  severity: string;
}

export interface DuplicateWarning {
  category: string;
  installed_mods: string[];
  recommendation: string;
}

export interface LaunchProfile {
  id: string;
  instance_id: string;
  name: string;
  java_path: string | null;
  jvm_args: string | null;
  xmx_mb: number | null;
  xms_mb: number | null;
  pre_launch_actions: string | null;
  auto_apply: boolean;
  created_at: string;
  updated_at: string;
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
  avg_fps: number | null;
  low_1pct_fps: number | null;
  avg_tps: number | null;
  telemetry_points: number;
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

export interface OptimizationAction {
  id: string;
  recommendation_id: string | null;
  instance_id: string | null;
  action_type: string;
  description: string;
  file_path: string | null;
  old_value: string | null;
  new_value: string | null;
  status: string;
  applied_at: string;
  rolled_back_at: string | null;
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

export interface ModSearchResult {
  project_id: string;
  slug: string;
  title: string;
  description: string;
  author: string;
  downloads: number;
  icon_url: string | null;
  categories: string[];
  latest_version: string | null;
}

export interface ModVersion {
  id: string;
  version_number: string;
  name: string;
  game_versions: string[];
  loaders: string[];
  downloads: number;
  files: ModFile[];
}

export interface ModFile {
  url: string;
  filename: string;
  size: number;
  primary: boolean;
}

export interface InstallResult {
  success: boolean;
  filename: string;
  installed_path: string;
  message: string;
}

export interface GovernorStatus {
  mode: string;
  self_cpu: number;
  self_ram_mb: number;
  telemetry_interval_ms: number;
  game_running: boolean;
  vision_mode: string;
  capture_interval_ms: number;
}

export interface DetectedGame {
  game_id: string;
  game_name: string;
  process_name: string;
  window_title: string | null;
  is_running: boolean;
}

export interface RecommendationOutcome {
  recommendation_id: string;
  metric_name: string;
  improvement_percent: number | null;
  outcome: string;
  title: string;
  category: string;
}

export interface LogEvent {
  timestamp: string;
  level: string;
  message: string;
}

export interface TelemetrySummary {
  minute_ts: string;
  cpu_avg: number | null;
  ram_avg_mb: number | null;
  ram_peak_mb: number | null;
  hog_count: number | null;
  fps_avg: number | null;
  fps_low_1pct: number | null;
  tps_avg: number | null;
}

export interface OptimizationProfile {
  version: string;
  created_at: string;
  game: string;
  instance_name: string;
  minecraft_version: string | null;
  loader: string | null;
  jvm_settings: JvmProfile | null;
  config_changes: ProfileConfigChange[];
  recommended_mods: ProfileRecommendedMod[];
  health_score: number | null;
  hardware_summary: string | null;
}

export interface JvmProfile {
  xmx_mb: number | null;
  xms_mb: number | null;
  jvm_args: string | null;
  java_version: string | null;
}

export interface ProfileConfigChange {
  file: string;
  key: string;
  value: string;
  reason: string;
}

export interface ProfileRecommendedMod {
  name: string;
  modrinth_slug: string | null;
  reason: string;
}

export interface GameInfo {
  id: string;
  name: string;
  icon: string;
  installed: boolean;
  install_path: string | null;
}

export interface SteamGameInstance {
  id: string;
  game_id: string;
  name: string;
  path: string;
  version: string | null;
  last_played: string | null;
}

export interface PlayerStats {
  username: string;
  skills: SkillLevel[];
  total_level: number;
  total_xp: number;
}

export interface SkillLevel {
  name: string;
  level: number;
  xp: number;
  rank: number;
}

export interface CurrencyPrice {
  name: string;
  chaos_equivalent: number;
  change_percent: number;
}

export interface AmmoData {
  name: string;
  short_name: string;
  caliber: string;
  damage: number;
  penetration: number;
  armor_damage: number;
}

export interface ItemPrice {
  name: string;
  short_name: string;
  avg_24h_price: number;
  last_low_price: number;
}

export interface CrashDiagnosis {
  crash_detected: boolean;
  crash_type: string | null;
  summary: string;
  details: string[];
  recommendations: CrashRecommendation[];
  crash_file: string | null;
  timestamp: string | null;
}

export interface CrashRecommendation {
  title: string;
  description: string;
  action: string;
  priority: string;
}

export interface DiskAdvice {
  instance_drive: DriveStatus | null;
  best_drive: DriveRecommendation | null;
  paging_file: PagingFileStatus | null;
  warnings: DiskWarning[];
  recommendations: DiskRecommendation[];
}

export interface DriveStatus {
  mount_point: string;
  total_gb: number;
  free_gb: number;
  used_percent: number;
  is_critical: boolean;
  storage_type: string;
}

export interface DriveRecommendation {
  mount_point: string;
  free_gb: number;
  storage_type: string;
  reason: string;
}

export interface PagingFileStatus {
  current_size_mb: number;
  max_size_mb: number;
  drive: string;
  is_system_managed: boolean;
  is_adequate: boolean;
  recommended_size_mb: number;
}

export interface DiskWarning {
  severity: string;
  message: string;
}

export interface DiskRecommendation {
  title: string;
  description: string;
  action: string;
  priority: string;
  estimated_gain_gb: number | null;
}

export interface MigrationResult {
  success: boolean;
  old_path: string;
  new_path: string;
  files_copied: number;
  total_size_mb: number;
  message: string;
}

export interface BloatwareReport {
  temp_files: TempFileInfo[];
  total_temp_size_mb: number;
  startup_programs: StartupProgram[];
  cleanup_recommendations: CleanupRecommendation[];
}

export interface TempFileInfo {
  path: string;
  size_mb: number;
  category: string;
}

export interface StartupProgram {
  name: string;
  command: string;
  source: string;
}

export interface CleanupRecommendation {
  title: string;
  description: string;
  estimated_size_mb: number;
  action: string;
  safe: boolean;
}
