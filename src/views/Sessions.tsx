import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useInvoke } from "../hooks/useInvoke";
import type { Session, SessionReport } from "../types";

export function Sessions() {
  const sessions = useInvoke<Session[]>("get_sessions");
  const [selectedSession, setSelectedSession] = useState<Session | null>(null);
  const [report, setReport] = useState<SessionReport | null>(null);
  const [reportLoading, setReportLoading] = useState(false);
  const [reportError, setReportError] = useState<string | null>(null);

  useEffect(() => {
    sessions.execute();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

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
