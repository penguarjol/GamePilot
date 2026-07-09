import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useInvoke } from "../hooks/useInvoke";
import type { Session, SessionReport, TelemetrySample } from "../types";

export function Sessions() {
  const sessions = useInvoke<Session[]>("get_sessions");
  const [selectedSession, setSelectedSession] = useState<Session | null>(null);
  const [report, setReport] = useState<SessionReport | null>(null);
  const [reportLoading, setReportLoading] = useState(false);
  const [reportError, setReportError] = useState<string | null>(null);
  const [telemetry, setTelemetry] = useState<TelemetrySample | null>(null);
  const [gameRunning, setGameRunning] = useState<boolean | null>(null);
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const samplesRef = useRef<{ cpu: number[]; ram: number[] }>({ cpu: [], ram: [] });

  useEffect(() => {
    sessions.execute();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const hasActiveSession = sessions.data?.some((s) => s.status === "active") ?? false;

  useEffect(() => {
    if (hasActiveSession) {
      samplesRef.current = { cpu: [], ram: [] };
      const poll = async () => {
        try {
          const sample = await invoke<TelemetrySample>("get_telemetry_sample");
          setTelemetry(sample);
          samplesRef.current.cpu.push(sample.cpu_percent);
          samplesRef.current.ram.push(sample.ram_used_mb);
        } catch { /* ignore polling errors */ }
        try {
          const running = await invoke<boolean>("is_game_running", { processName: "java" });
          setGameRunning(running);
          if (!running) {
            const active = sessions.data?.find((s) => s.status === "active");
            if (active) {
              const { cpu, ram } = samplesRef.current;
              if (cpu.length > 0) {
                const cpuAvg = cpu.reduce((a, b) => a + b, 0) / cpu.length;
                const ramAvg = ram.reduce((a, b) => a + b, 0) / ram.length;
                const ramPeak = Math.max(...ram);
                await invoke("store_session_telemetry", {
                  sessionId: active.id,
                  cpuAvg,
                  ramAvg,
                  ramPeak,
                }).catch(() => {});
              }
              await invoke<Session>("end_session", { sessionId: active.id });
              await sessions.execute();
            }
          }
        } catch { /* ignore */ }
      };
      poll();
      pollRef.current = setInterval(poll, 5000);
    } else {
      setTelemetry(null);
      setGameRunning(null);
    }
    return () => {
      if (pollRef.current) clearInterval(pollRef.current);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [hasActiveSession]);

  const selectSession = async (session: Session) => {
    setSelectedSession(session);
    setReport(null);
    setReportError(null);
    setReportLoading(true);
    try {
      const r = await invoke<SessionReport>("get_session_report", {
        sessionId: session.id,
      });
      setReport(r);
    } catch (err) {
      setReportError(String(err));
    } finally {
      setReportLoading(false);
    }
  };

  const endSession = async (session: Session) => {
    try {
      await invoke<Session>("end_session", { sessionId: session.id });
      await sessions.execute();
      if (selectedSession?.id === session.id) {
        setSelectedSession(null);
        setReport(null);
      }
    } catch (err) {
      setReportError(String(err));
    }
  };

  return (
    <div className="view-content">
      <div className="view-header">
        <h1>Sessions</h1>
        <button
          className="btn btn-secondary"
          onClick={() => sessions.execute()}
          disabled={sessions.loading}
        >
          Refresh
        </button>
      </div>

      <div className="sessions-layout">
        {hasActiveSession && (
          <div className="telemetry-live-bar card">
            <div className="telemetry-header">
              <span className="badge badge-success">LIVE</span>
              {gameRunning !== null && (
                <span className={`badge badge-${gameRunning ? "success" : "warning"}`}>
                  {gameRunning ? "Game Running" : "Game Stopped"}
                </span>
              )}
            </div>
            {telemetry && (
              <div className="telemetry-stats">
                <div className="telemetry-stat">
                  <span className="telemetry-stat-label">CPU</span>
                  <span className="telemetry-stat-value">{telemetry.cpu_percent.toFixed(1)}%</span>
                </div>
                <div className="telemetry-stat">
                  <span className="telemetry-stat-label">RAM Used</span>
                  <span className="telemetry-stat-value">{telemetry.ram_used_mb.toFixed(0)} MB</span>
                </div>
                <div className="telemetry-stat">
                  <span className="telemetry-stat-label">RAM Free</span>
                  <span className="telemetry-stat-value">{telemetry.ram_available_mb.toFixed(0)} MB</span>
                </div>
                {telemetry.top_processes.length > 0 && (
                  <div className="telemetry-procs">
                    <span className="telemetry-stat-label">Top:</span>
                    {telemetry.top_processes.slice(0, 3).map((p) => (
                      <span key={p.pid} className="telemetry-proc">
                        {p.name} ({p.cpu_percent.toFixed(0)}% / {p.ram_mb.toFixed(0)}MB)
                      </span>
                    ))}
                  </div>
                )}
              </div>
            )}
          </div>
        )}

        <div className="sessions-list-panel">
          {sessions.loading ? (
            <div className="loading-center"><span className="spinner" /></div>
          ) : sessions.data && sessions.data.length > 0 ? (
            <table>
              <thead>
                <tr>
                  <th>Status</th>
                  <th>Started</th>
                  <th>Duration</th>
                  <th>Method</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                {sessions.data.map((s) => (
                  <tr
                    key={s.id}
                    className={`session-row${selectedSession?.id === s.id ? " active" : ""}`}
                    onClick={() => selectSession(s)}
                  >
                    <td>
                      <span className={`badge badge-${s.status === "active" ? "success" : s.status === "completed" ? "info" : "none"}`}>
                        {s.status}
                      </span>
                    </td>
                    <td>{formatDate(s.started_at)}</td>
                    <td>{s.duration_secs != null ? formatDuration(s.duration_secs) : "-"}</td>
                    <td>{s.launch_method ?? "-"}</td>
                    <td>
                      {s.status === "active" && (
                        <button
                          className="btn btn-danger btn-sm"
                          onClick={(e) => { e.stopPropagation(); endSession(s); }}
                        >
                          End
                        </button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          ) : (
            <div className="empty-state">
              <span className="empty-state-icon">{"\u25F7"}</span>
              <span>No sessions recorded</span>
              <span className="empty-state-hint">Launch a Minecraft instance to start recording sessions</span>
            </div>
          )}
        </div>

        {selectedSession && (
          <div className="session-detail-panel card">
            <h3>Session Report</h3>
            {reportLoading ? (
              <div className="loading-center"><span className="spinner" /></div>
            ) : reportError ? (
              <div className="error-state">{reportError}</div>
            ) : report ? (
              <div className="report-content">
                <dl className="detail-dl">
                  <dt>Instance ID</dt>
                  <dd className="mono">{report.session.instance_id}</dd>
                  <dt>Status</dt>
                  <dd>
                    <span className={`badge badge-${report.session.status === "active" ? "success" : "info"}`}>
                      {report.session.status}
                    </span>
                  </dd>
                  <dt>Started</dt>
                  <dd>{formatDate(report.session.started_at)}</dd>
                  {report.session.ended_at && (
                    <>
                      <dt>Ended</dt>
                      <dd>{formatDate(report.session.ended_at)}</dd>
                    </>
                  )}
                  {report.session.duration_secs != null && (
                    <>
                      <dt>Duration</dt>
                      <dd>{formatDuration(report.session.duration_secs)}</dd>
                    </>
                  )}
                  {report.session.cpu_avg_percent != null && (
                    <>
                      <dt>Avg CPU</dt>
                      <dd>{report.session.cpu_avg_percent.toFixed(1)}%</dd>
                    </>
                  )}
                  {report.session.ram_avg_mb != null && (
                    <>
                      <dt>Avg RAM</dt>
                      <dd>{report.session.ram_avg_mb.toFixed(0)} MB</dd>
                    </>
                  )}
                  {report.session.ram_peak_mb != null && (
                    <>
                      <dt>Peak RAM</dt>
                      <dd>{report.session.ram_peak_mb.toFixed(0)} MB</dd>
                    </>
                  )}
                  <dt>Recommendations Applied</dt>
                  <dd>{report.recommendations_applied}</dd>
                  <dt>Process Observations</dt>
                  <dd>{report.process_observations}</dd>
                </dl>
                <div className="report-summary">
                  <strong>Summary:</strong> {report.summary}
                </div>
              </div>
            ) : null}
          </div>
        )}
      </div>
    </div>
  );
}

function formatDate(iso: string): string {
  const d = new Date(iso);
  return d.toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function formatDuration(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = secs % 60;
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m ${s}s`;
  return `${s}s`;
}
