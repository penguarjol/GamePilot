import { useEffect } from "react";
import { useInvoke } from "../hooks/useInvoke";
import type { HardwareInfo, ProcessInfo, JavaInstallation } from "../types";

export function Diagnostics() {
  const hw = useInvoke<HardwareInfo>("get_hardware_info");
  const procs = useInvoke<ProcessInfo[]>("get_process_info");
  const java = useInvoke<JavaInstallation[]>("detect_java");

  useEffect(() => {
    hw.execute();
    procs.execute();
    java.execute();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const refresh = async () => {
    await Promise.all([hw.execute(), procs.execute(), java.execute()]);
  };

  const ramPercent = hw.data
    ? Math.round((hw.data.ram_used_mb / hw.data.ram_total_mb) * 100)
    : 0;

  return (
    <div className="view-content">
      <div className="view-header">
        <h1>Diagnostics</h1>
        <button
          className="btn btn-primary"
          onClick={refresh}
          disabled={hw.loading}
        >
          {hw.loading ? <span className="spinner" /> : null}
          Refresh
        </button>
      </div>

      <div className="diag-grid">
        <div className="card">
          <h3 className="card-title">Hardware</h3>
          {hw.data ? (
            <dl className="detail-dl">
              <dt>Hostname</dt>
              <dd>{hw.data.hostname}</dd>
              <dt>OS</dt>
              <dd>{hw.data.os_name} {hw.data.os_version}</dd>
              <dt>CPU</dt>
              <dd>{hw.data.cpu_model}</dd>
              <dt>Cores / Threads</dt>
              <dd>{hw.data.cpu_cores} / {hw.data.cpu_threads}</dd>
              <dt>CPU Usage</dt>
              <dd>
                <div className="bar-container">
                  <div
                    className="bar-fill"
                    style={{ width: `${Math.min(hw.data.cpu_usage_percent, 100)}%` }}
                  />
                  <span className="bar-label">{hw.data.cpu_usage_percent.toFixed(1)}%</span>
                </div>
              </dd>
              <dt>RAM</dt>
              <dd>
                <div className="bar-container">
                  <div
                    className={`bar-fill${ramPercent > 85 ? " bar-warn" : ""}`}
                    style={{ width: `${ramPercent}%` }}
                  />
                  <span className="bar-label">
                    {hw.data.ram_used_mb} / {hw.data.ram_total_mb} MB ({ramPercent}%)
                  </span>
                </div>
              </dd>
              <dt>Available RAM</dt>
              <dd>{hw.data.ram_available_mb} MB</dd>
              <dt>GPU</dt>
              <dd>{hw.data.gpu_model}</dd>
              {hw.data.gpu_vram_mb > 0 && (
                <>
                  <dt>VRAM</dt>
                  <dd>{hw.data.gpu_vram_mb} MB</dd>
                </>
              )}
            </dl>
          ) : hw.error ? (
            <div className="error-state">{hw.error}</div>
          ) : (
            <div className="loading-center"><span className="spinner spinner-lg" /></div>
          )}
        </div>

        <div className="card">
          <h3 className="card-title">Java Installations</h3>
          {java.data && java.data.length > 0 ? (
            <table>
              <thead>
                <tr>
                  <th>Version</th>
                  <th>Vendor</th>
                  <th>64-bit</th>
                  <th>Path</th>
                </tr>
              </thead>
              <tbody>
                {java.data.map((j, i) => (
                  <tr key={i}>
                    <td>{j.version ?? "Unknown"}</td>
                    <td>{j.vendor ?? "Unknown"}</td>
                    <td>{j.is_64bit ? "Yes" : "No"}</td>
                    <td className="mono path-cell">{j.path}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          ) : java.loading ? (
            <div className="loading-center"><span className="spinner" /></div>
          ) : (
            <div className="empty-state">
              <span>No Java installations found</span>
            </div>
          )}
        </div>
      </div>

      <div className="card" style={{ marginTop: "var(--space-lg)" }}>
        <h3 className="card-title">
          Processes
          {procs.data && (
            <span className="card-title-count">({procs.data.length})</span>
          )}
        </h3>
        {procs.data ? (
          <div className="table-scroll">
            <table>
              <thead>
                <tr>
                  <th>Name</th>
                  <th>PID</th>
                  <th>CPU %</th>
                  <th>RAM (MB)</th>
                  <th>Category</th>
                  <th>Status</th>
                  <th>Recommendation</th>
                </tr>
              </thead>
              <tbody>
                {procs.data.map((p) => (
                  <tr key={p.pid} className={p.is_resource_hog ? "row-hog" : ""}>
                    <td className="mono">{p.name}</td>
                    <td>{p.pid}</td>
                    <td className={p.cpu_percent > 15 ? "text-warn" : ""}>
                      {p.cpu_percent.toFixed(1)}
                    </td>
                    <td className={p.ram_mb > 1000 ? "text-warn" : ""}>
                      {p.ram_mb.toFixed(0)}
                    </td>
                    <td>{p.category}</td>
                    <td>
                      {p.is_resource_hog ? (
                        <span className="badge badge-warning">hog</span>
                      ) : (
                        <span className="badge badge-none">ok</span>
                      )}
                    </td>
                    <td>{p.recommendation || "-"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : procs.error ? (
          <div className="error-state">{procs.error}</div>
        ) : (
          <div className="loading-center"><span className="spinner spinner-lg" /></div>
        )}
      </div>
    </div>
  );
}
