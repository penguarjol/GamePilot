export interface HardwareInfo {
  cpu_model: string;
  cpu_cores: number;
  cpu_threads: number;
  cpu_usage_percent: number;
  ram_total_mb: number;
  ram_used_mb: number;
  ram_available_mb: number;
  gpu_model: string;
  gpu_vram_mb: number;
  os_name: string;
  os_version: string;
  hostname: string;
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
