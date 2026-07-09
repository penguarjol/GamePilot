import { useState, useEffect } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { useInvoke } from "../hooks/useInvoke";
import type {
  MinecraftInstance,
  SavedInstance,
  DiscoveredLauncher,
  ModAnalysis,
  Recommendation,
  LaunchResult,
} from "../types";

export function Minecraft() {
  const saved = useInvoke<SavedInstance[]>("get_saved_instances");
  const launchers = useInvoke<DiscoveredLauncher[]>("discover_launchers");
  const [selectedInstance, setSelectedInstance] = useState<MinecraftInstance | null>(null);
  const [modAnalysis, setModAnalysis] = useState<ModAnalysis | null>(null);
  const [recommendations, setRecommendations] = useState<Recommendation[]>([]);
  const [loading, setLoading] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [launchResult, setLaunchResult] = useState<LaunchResult | null>(null);

  useEffect(() => {
    saved.execute();
    launchers.execute();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const addInstance = async () => {
    setError(null);
    const folder = await open({ directory: true, title: "Select Minecraft instance folder" });
    if (!folder) return;

    setLoading("Scanning instance...");
    try {
      const instance = await invoke<MinecraftInstance>("scan_instance", {
        path: folder,
        launcher: "Custom",
      });
      setSelectedInstance(instance);
      await invoke("save_instance", { instanceJson: JSON.stringify(instance) });
      await saved.execute();
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(null);
    }
  };

  const selectSaved = async (inst: SavedInstance) => {
    setError(null);
    setModAnalysis(null);
    setRecommendations([]);
    setLaunchResult(null);
    setLoading("Scanning instance...");
    try {
      const full = await invoke<MinecraftInstance>("scan_instance", {
        path: inst.path,
        launcher: inst.launcher ?? "Custom",
      });
      setSelectedInstance(full);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(null);
    }
  };

  const analyzeInstance = async () => {
    if (!selectedInstance) return;
    setError(null);
    setLoading("Analyzing mods...");
    try {
      if (selectedInstance.mods_path) {
        const analysis = await invoke<ModAnalysis>("analyze_mods", {
          modsPath: selectedInstance.mods_path,
          loader: selectedInstance.loader_type,
        });
        setModAnalysis(analysis);
      }
      const recs = await invoke<Recommendation[]>("get_recommendations", {
        instanceJson: JSON.stringify(selectedInstance),
      });
      setRecommendations(recs);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(null);
    }
  };

  const launchInstance = async () => {
    if (!selectedInstance) return;
    setError(null);
    setLoading("Launching...");
    try {
      const result = await invoke<LaunchResult>("launch_instance", {
        instanceId: selectedInstance.id,
        launcher: selectedInstance.launcher,
        instancePath: selectedInstance.path,
      });
      setLaunchResult(result);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(null);
    }
  };

  return (
    <div className="view-content">
      <div className="view-header">
        <h1>Minecraft</h1>
        <button className="btn btn-primary" onClick={addInstance}>
          + Add Instance
        </button>
      </div>

      {error && <div className="error-state">{error}</div>}
      {loading && (
        <div className="loading-banner">
          <span className="spinner" />
          <span>{loading}</span>
        </div>
      )}

      <div className="mc-layout">
        <div className="mc-sidebar">
          <h3 className="section-title">Instances</h3>
          {saved.data && saved.data.length > 0 ? (
            <ul className="instance-list">
              {saved.data.map((inst) => (
                <li
                  key={inst.id}
                  className={`instance-item${selectedInstance?.id === inst.id ? " active" : ""}`}
                  onClick={() => selectSaved(inst)}
                  onKeyDown={(e) => e.key === "Enter" && selectSaved(inst)}
                  tabIndex={0}
                  role="button"
                >
                  <div className="instance-name">{inst.name}</div>
                  <div className="instance-meta">
                    {inst.minecraft_version ?? "Unknown"}
                    {inst.loader_type ? ` - ${inst.loader_type}` : ""}
                    {inst.mod_count ? ` (${inst.mod_count} mods)` : ""}
                  </div>
                </li>
              ))}
            </ul>
          ) : (
            <div className="empty-state">
              <span>No instances</span>
              <span className="empty-state-hint">Click "Add Instance" to get started</span>
            </div>
          )}

          {launchers.data && launchers.data.length > 0 && (
            <>
              <h3 className="section-title">Detected Launchers</h3>
              <ul className="launcher-mini-list">
                {launchers.data.map((l, i) => (
                  <li key={i} className="launcher-mini">{l.name}</li>
                ))}
              </ul>
            </>
          )}
        </div>

        <div className="mc-detail">
          {selectedInstance ? (
            <>
              <div className="detail-header">
                <div>
                  <h2>{selectedInstance.name}</h2>
                  <span className="detail-path">{selectedInstance.path}</span>
                </div>
                <div className="detail-actions">
                  <button className="btn btn-secondary" onClick={analyzeInstance}>
                    Analyze
                  </button>
                  <button className="btn btn-primary" onClick={launchInstance}>
                    Launch
                  </button>
                </div>
              </div>

              {launchResult && (
                <div className={`launch-result ${launchResult.success ? "success" : "failure"}`}>
                  <strong>{launchResult.success ? "Launched" : "Failed"}</strong>
                  <span>{launchResult.message}</span>
                </div>
              )}

              <div className="detail-grid">
                <div className="detail-section">
                  <h4>Instance Info</h4>
                  <dl className="detail-dl">
                    <dt>Version</dt>
                    <dd>{selectedInstance.minecraft_version ?? "Unknown"}</dd>
                    <dt>Loader</dt>
                    <dd>
                      {selectedInstance.loader_type ?? "None"}
                      {selectedInstance.loader_version ? ` (${selectedInstance.loader_version})` : ""}
                    </dd>
                    <dt>Launcher</dt>
                    <dd>{selectedInstance.launcher}</dd>
                    <dt>Mods</dt>
                    <dd>{selectedInstance.mod_count}</dd>
                  </dl>
                </div>

                <div className="detail-section">
                  <h4>JVM Settings</h4>
                  <dl className="detail-dl">
                    <dt>Java Path</dt>
                    <dd className="mono">{selectedInstance.java_path ?? "Default"}</dd>
                    <dt>Max RAM (Xmx)</dt>
                    <dd>{selectedInstance.xmx_mb ? `${selectedInstance.xmx_mb} MB` : "Not set"}</dd>
                    <dt>Min RAM (Xms)</dt>
                    <dd>{selectedInstance.xms_mb ? `${selectedInstance.xms_mb} MB` : "Not set"}</dd>
                    <dt>JVM Args</dt>
                    <dd className="mono jvm-args">
                      {selectedInstance.jvm_args ?? "Default"}
                    </dd>
                  </dl>
                </div>
              </div>

              {modAnalysis && (
                <div className="detail-section">
                  <h4>Mod Analysis ({modAnalysis.total_mods} mods, {modAnalysis.total_size_mb.toFixed(1)} MB)</h4>

                  {modAnalysis.detected_performance_mods.length > 0 && (
                    <div className="perf-mods">
                      <span className="perf-label">Performance mods detected:</span>
                      {modAnalysis.detected_performance_mods.map((m) => (
                        <span key={m} className="badge badge-success">{m}</span>
                      ))}
                    </div>
                  )}

                  {modAnalysis.mods.length > 0 && (
                    <div className="mod-table-wrap">
                      <table>
                        <thead>
                          <tr>
                            <th>File</th>
                            <th>Version</th>
                            <th>Size</th>
                          </tr>
                        </thead>
                        <tbody>
                          {modAnalysis.mods.slice(0, 50).map((mod, i) => (
                            <tr key={i}>
                              <td>{mod.display_name ?? mod.file_name}</td>
                              <td>{mod.version ?? "-"}</td>
                              <td>{(mod.size_bytes / 1024 / 1024).toFixed(1)} MB</td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                      {modAnalysis.mods.length > 50 && (
                        <p className="table-overflow">
                          ...and {modAnalysis.mods.length - 50} more
                        </p>
                      )}
                    </div>
                  )}
                </div>
              )}

              {recommendations.length > 0 && (
                <div className="detail-section">
                  <h4>Recommendations</h4>
                  <ul className="rec-list">
                    {recommendations.map((r) => (
                      <li key={r.id} className="rec-card">
                        <div className="rec-badges">
                          <span className={`badge badge-${r.severity}`}>{r.severity}</span>
                          <span className={`badge badge-${r.confidence}`}>{r.confidence} confidence</span>
                          <span className={`badge badge-${r.risk_level === "none" ? "none" : r.risk_level}`}>
                            risk: {r.risk_level}
                          </span>
                        </div>
                        <div className="rec-title">{r.title}</div>
                        <div className="rec-desc">{r.description}</div>
                        <div className="rec-evidence">{r.evidence}</div>
                        {r.action_type === "open_link" && r.action_data && (
                          <a
                            href={r.action_data}
                            target="_blank"
                            rel="noreferrer"
                            className="btn btn-secondary btn-sm"
                          >
                            Open Link
                          </a>
                        )}
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </>
          ) : (
            <div className="empty-state">
              <span className="empty-state-icon">{"\u25A3"}</span>
              <span>Select an instance or add a new one</span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
