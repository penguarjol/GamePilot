import { useEffect } from "react";
import { useInvoke } from "../hooks/useInvoke";
import type {
  HardwareInfo,
  ProcessInfo,
  DiscoveredLauncher,
  Recommendation,
  Session,
  SavedInstance,
  SelfMetrics,
} from "../types";

export function Dashboard() {
  const hw = useInvoke<HardwareInfo>("get_hardware_info");
  const procs = useInvoke<ProcessInfo[]>("get_process_info");
  const launchers = useInvoke<DiscoveredLauncher[]>("discover_launchers");
  const sessions = useInvoke<Session[]>("get_sessions");
  const saved = useInvoke<SavedInstance[]>("get_saved_instances");
  const recs = useInvoke<Recommendation[]>("get_recommendations_for_path");
  const selfMetrics = useInvoke<SelfMetrics>("get_self_metrics");

  useEffect(() => {
    launchers.execute();
    sessions.execute();
    saved.execute();
    selfMetrics.execute();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const runDiagnostics = async () => {
    await Promise.all([hw.execute(), procs.execute()]);
  };

  useEffect(() => {
    if (saved.data && saved.data.length > 0) {
      const inst = saved.data[0];
      recs.execute({
        instancePath: inst.path,
        launcher: inst.launcher ?? "Custom",
      });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [saved.data]);

  const resourceHogs = procs.data?.filter((p) => p.is_resource_hog) ?? [];
  const ramUsedPercent = hw.data
    ? Math.round((hw.data.ram_used_mb / hw.data.ram_total_mb) * 100)
    : 0;
  const lowDisks = hw.data?.disks.filter((d) => d.free_gb < 10) ?? [];

  return (
    <div className="view-content">
      <div className="view-header">
        <h1>Dashboard</h1>
        <button
          className="btn btn-primary"
          onClick={runDiagnostics}
          disabled={hw.loading || procs.loading}
        >
          {hw.loading || procs.loading ? (
            <span className="spinner" />
          ) : null}
          Run Diagnostics
        </button>
      </div>

      <div className="dashboard-grid">
        <div className="card">
          <h3 className="card-title">System Health</h3>
          {hw.data ? (
            <div className="hw-summary">
              <div className="hw-row">
                <span className="hw-label">CPU</span>
                <span className="hw-value">{hw.data.cpu_model}</span>
              </div>
              <div className="hw-row">
                <span className="hw-label">Cores / Threads</span>
                <span className="hw-value">
                  {hw.data.cpu_cores} / {hw.data.cpu_threads}
                </span>
              </div>
              <div className="hw-row">
                <span className="hw-label">CPU Usage</span>
                <span className="hw-value">{hw.data.cpu_usage_percent.toFixed(1)}%</span>
              </div>
              {hw.data.cpu_freq_mhz > 0 && (
                <div className="hw-row">
                  <span className="hw-label">CPU Frequency</span>
                  <span className="hw-value">{(hw.data.cpu_freq_mhz / 1000).toFixed(2)} GHz</span>
                </div>
              )}
              <div className="hw-row">
                <span className="hw-label">RAM</span>
                <span className="hw-value">
                  {hw.data.ram_used_mb} / {hw.data.ram_total_mb} MB ({ramUsedPercent}%)
                </span>
              </div>
              <div className="hw-row">
                <span className="hw-label">GPU</span>
                <span className="hw-value">{hw.data.gpu_model}</span>
              </div>
              <div className="hw-row">
                <span className="hw-label">OS</span>
                <span className="hw-value">
                  {hw.data.os_name} {hw.data.os_version}
                </span>
              </div>
            </div>
          ) : hw.error ? (
            <div className="error-state">{hw.error}</div>
          ) : (
            <div className="empty-state">
              <span className="empty-state-icon">{"\u2699"}</span>
              <span>Click "Run Diagnostics" to scan your system</span>
            </div>
          )}
        </div>

        <div className="card">
          <h3 className="card-title">Discovered Launchers</h3>
          {launchers.loading ? (
            <div className="loading-center"><span className="spinner" /></div>
          ) : launchers.data && launchers.data.length > 0 ? (
            <ul className="launcher-list">
              {launchers.data.map((l, i) => (
                <li key={i} className="launcher-item">
                  <span className="launcher-name">{l.name}</span>
                  <span className="launcher-path">{l.path}</span>
                </li>
              ))}
            </ul>
          ) : (
            <div className="empty-state">
              <span className="empty-state-icon">{"\u25A3"}</span>
              <span>No launchers detected</span>
            </div>
          )}
        </div>

        <div className="card">
          <h3 className="card-title">Resource Hogs</h3>
          {procs.data ? (
            resourceHogs.length > 0 ? (
              <ul className="hog-list">
                {resourceHogs.slice(0, 5).map((p) => (
                  <li key={p.pid} className="hog-item">
                    <div className="hog-name">{p.name}</div>
                    <div className="hog-stats">
                      <span>{p.ram_mb.toFixed(0)} MB</span>
                      <span>{p.cpu_percent.toFixed(1)}% CPU</span>
                    </div>
                    <div className="hog-rec">{p.recommendation}</div>
                  </li>
                ))}
              </ul>
            ) : (
              <div className="empty-state">
                <span>No resource hogs detected</span>
              </div>
            )
          ) : procs.error ? (
            <div className="error-state">{procs.error}</div>
          ) : (
            <div className="empty-state">
              <span className="empty-state-icon">{"\u26A0"}</span>
              <span>Run diagnostics to detect background processes</span>
            </div>
          )}
        </div>

        <div className="card">
          <h3 className="card-title">Top Recommendations</h3>
          {recs.data && recs.data.length > 0 ? (
            <ul className="rec-preview-list">
              {recs.data.slice(0, 3).map((r) => (
                <li key={r.id} className="rec-preview-item">
                  <div className="rec-preview-header">
                    <span className={`badge badge-${r.severity}`}>{r.severity}</span>
                    <span className={`badge badge-${r.confidence}`}>{r.confidence}</span>
                  </div>
                  <div className="rec-preview-title">{r.title}</div>
                </li>
              ))}
            </ul>
          ) : (
            <div className="empty-state">
              <span className="empty-state-icon">{"\u2691"}</span>
              <span>Add and analyze an instance for recommendations</span>
            </div>
          )}
        </div>

        <div className="card">
          <h3 className="card-title">Recent Sessions</h3>
          {sessions.data && sessions.data.length > 0 ? (
            <ul className="session-preview-list">
              {sessions.data.slice(0, 4).map((s) => (
                <li key={s.id} className="session-preview-item">
                  <span className={`badge badge-${s.status === "active" ? "success" : "info"}`}>
                    {s.status}
                  </span>
                  <span className="session-preview-date">
                    {new Date(s.started_at).toLocaleDateString()}
                  </span>
                  {s.duration_secs != null && (
                    <span className="session-preview-duration">
                      {formatDuration(s.duration_secs)}
                    </span>
                  )}
                </li>
              ))}
            </ul>
          ) : (
            <div className="empty-state">
              <span className="empty-state-icon">{"\u25F7"}</span>
              <span>No sessions recorded yet</span>
            </div>
          )}
        </div>

        <div className="card">
          <h3 className="card-title">Saved Instances</h3>
          {saved.data && saved.data.length > 0 ? (
            <ul className="saved-preview-list">
              {saved.data.slice(0, 4).map((inst) => (
                <li key={inst.id} className="saved-preview-item">
                  <span className="saved-name">{inst.name}</span>
                  <span className="saved-meta">
                    {inst.minecraft_version ?? "Unknown version"}
                    {inst.loader_type ? ` / ${inst.loader_type}` : ""}
                  </span>
                </li>
              ))}
            </ul>
          ) : (
            <div className="empty-state">
              <span className="empty-state-icon">{"\u25A3"}</span>
              <span>No instances saved</span>
            </div>
          )}
        </div>
      </div>

      {lowDisks.length > 0 && (
        <div className="disk-warning-banner">
          <strong>Low disk space:</strong>
          {lowDisks.map((d) => (
            <span key={d.mount_point} className="disk-warning-item">
              {d.name || d.mount_point} — {d.free_gb.toFixed(1)} GB free
            </span>
          ))}
        </div>
      )}

      {selfMetrics.data && (
        <div className="self-metrics-footer">
          <span className="self-metrics-label">GamePilot</span>
          <span className="self-metrics-stat">CPU: {selfMetrics.data.cpu_percent.toFixed(1)}%</span>
          <span className="self-metrics-stat">RAM: {selfMetrics.data.ram_mb.toFixed(0)} MB</span>
        </div>
      )}
    </div>
  );
}

function formatDuration(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}
