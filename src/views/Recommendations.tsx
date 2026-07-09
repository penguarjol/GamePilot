import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useInvoke } from "../hooks/useInvoke";
import type { Recommendation, SavedInstance, RollbackPoint, ConfigAnalysis, ConfigRecommendation } from "../types";

type FilterCategory = "all" | "java_jvm" | "modpack";
type FilterSeverity = "all" | "warning" | "info" | "error";
type FilterStatus = "all" | "new" | "applied" | "ignored" | "deferred";

export function Recommendations() {
  const saved = useInvoke<SavedInstance[]>("get_saved_instances");
  const [recommendations, setRecommendations] = useState<Recommendation[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [filterCategory, setFilterCategory] = useState<FilterCategory>("all");
  const [filterSeverity, setFilterSeverity] = useState<FilterSeverity>("all");
  const [filterStatus, setFilterStatus] = useState<FilterStatus>("all");
  const [appliedIds, setAppliedIds] = useState<Set<string>>(new Set());
  const [statusMap, setStatusMap] = useState<Record<string, string>>({});
  const [configRecs, setConfigRecs] = useState<ConfigRecommendation[]>([]);

  useEffect(() => {
    saved.execute();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (saved.data && saved.data.length > 0) {
      loadRecommendations(saved.data[0]);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [saved.data]);

  const loadRecommendations = async (instance: SavedInstance) => {
    setLoading(true);
    setError(null);
    try {
      const recs = await invoke<Recommendation[]>("get_recommendations", {
        instanceJson: JSON.stringify(instance),
      });
      setRecommendations(recs);

      try {
        const config = await invoke<ConfigAnalysis>("analyze_configs", {
          instancePath: instance.path,
          modCount: instance.mod_count ?? 0,
        });
        setConfigRecs(config.recommendations);
      } catch {
        setConfigRecs([]);
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const applyRecommendation = async (rec: Recommendation) => {
    if (rec.action_type === "open_link" && rec.action_data) {
      window.open(rec.action_data, "_blank");
      setAppliedIds((prev) => new Set(prev).add(rec.id));
      return;
    }

    if (rec.action_type === "set_jvm_arg" && rec.action_data) {
      setAppliedIds((prev) => new Set(prev).add(rec.id));
    }
  };

  const rollback = async (rec: Recommendation) => {
    if (!rec.action_data) return;
    try {
      const rp: RollbackPoint = JSON.parse(rec.action_data);
      await invoke("rollback_file", { rollbackPointJson: JSON.stringify(rp) });
      setAppliedIds((prev) => {
        const next = new Set(prev);
        next.delete(rec.id);
        return next;
      });
    } catch {
      // Rollback data not available for this recommendation type
    }
  };

  const updateStatus = async (recId: string, status: string) => {
    await invoke("update_recommendation_status", { recommendationId: recId, status });
    setStatusMap((prev) => ({ ...prev, [recId]: status }));
    if (status === "applied") {
      setAppliedIds((prev) => new Set(prev).add(recId));
    }
  };

  const getRecStatus = (recId: string): string => {
    if (statusMap[recId]) return statusMap[recId];
    if (appliedIds.has(recId)) return "applied";
    return "new";
  };

  const filtered = recommendations.filter((r) => {
    if (filterCategory !== "all" && r.category !== filterCategory) return false;
    if (filterSeverity !== "all" && r.severity !== filterSeverity) return false;
    if (filterStatus !== "all" && getRecStatus(r.id) !== filterStatus) return false;
    return true;
  });

  return (
    <div className="view-content">
      <div className="view-header">
        <h1>Recommendations</h1>
        {saved.data && saved.data.length > 0 && (
          <select
            className="instance-select"
            onChange={(e) => {
              const inst = saved.data!.find((s) => s.id === e.target.value);
              if (inst) loadRecommendations(inst);
            }}
          >
            {saved.data.map((s) => (
              <option key={s.id} value={s.id}>{s.name}</option>
            ))}
          </select>
        )}
      </div>

      <div className="filter-bar">
        <label>
          Category:
          <select
            value={filterCategory}
            onChange={(e) => setFilterCategory(e.target.value as FilterCategory)}
          >
            <option value="all">All</option>
            <option value="java_jvm">JVM</option>
            <option value="modpack">Modpack</option>
          </select>
        </label>
        <label>
          Severity:
          <select
            value={filterSeverity}
            onChange={(e) => setFilterSeverity(e.target.value as FilterSeverity)}
          >
            <option value="all">All</option>
            <option value="warning">Warning</option>
            <option value="info">Info</option>
            <option value="error">Error</option>
          </select>
        </label>
        <div className="status-filter-btns">
          {(["all", "new", "applied", "ignored", "deferred"] as FilterStatus[]).map((s) => (
            <button
              key={s}
              className={`btn btn-sm${filterStatus === s ? " btn-primary" : " btn-secondary"}`}
              onClick={() => setFilterStatus(s)}
            >
              {s === "all" ? "All" : s.charAt(0).toUpperCase() + s.slice(1)}
            </button>
          ))}
        </div>
        <span className="filter-count">{filtered.length} recommendation{filtered.length !== 1 ? "s" : ""}</span>
      </div>

      {error && <div className="error-state">{error}</div>}

      {loading ? (
        <div className="loading-center"><span className="spinner spinner-lg" /></div>
      ) : filtered.length > 0 ? (
        <ul className="rec-full-list">
          {filtered.map((r) => (
            <li key={r.id} className="rec-full-card card">
              <div className="rec-full-header">
                <div className="rec-badges">
                  <span className={`badge badge-${r.severity}`}>{r.severity}</span>
                  <span className={`badge badge-${r.confidence}`}>{r.confidence}</span>
                  <span className={`badge badge-${r.risk_level === "none" ? "none" : r.risk_level}`}>
                    risk: {r.risk_level}
                  </span>
                  <span className="badge badge-none">{r.category}</span>
                  {appliedIds.has(r.id) && (
                    <span className="badge badge-success">applied</span>
                  )}
                </div>
              </div>
              <h3 className="rec-full-title">{r.title}</h3>
              <p className="rec-full-desc">{r.description}</p>
              <div className="rec-full-evidence">
                <strong>Evidence:</strong> {r.evidence}
              </div>
              <div className="rec-full-impact">
                <strong>Expected impact:</strong> {r.expected_impact}
              </div>
              <div className="rec-full-actions">
                {r.action_type && !appliedIds.has(r.id) && (
                  <button
                    className="btn btn-primary btn-sm"
                    onClick={() => applyRecommendation(r)}
                  >
                    {r.action_type === "open_link" ? "Open Link" : "Apply"}
                  </button>
                )}
                {appliedIds.has(r.id) && r.action_type !== "open_link" && (
                  <button
                    className="btn btn-danger btn-sm"
                    onClick={() => rollback(r)}
                  >
                    Rollback
                  </button>
                )}
                {getRecStatus(r.id) === "new" && (
                  <>
                    <button
                      className="btn btn-secondary btn-sm"
                      onClick={() => updateStatus(r.id, "ignored")}
                    >
                      Ignore Once
                    </button>
                    <button
                      className="btn btn-secondary btn-sm"
                      onClick={() => {
                        updateStatus(r.id, "ignored");
                        invoke("add_ignore_rule", { ruleType: "recommendation", pattern: r.title });
                      }}
                    >
                      Ignore Always
                    </button>
                    <button
                      className="btn btn-secondary btn-sm"
                      onClick={() => updateStatus(r.id, "deferred")}
                    >
                      Defer
                    </button>
                  </>
                )}
                {getRecStatus(r.id) !== "new" && (
                  <span className={`badge badge-status-${getRecStatus(r.id)}`}>
                    {getRecStatus(r.id)}
                  </span>
                )}
              </div>
            </li>
          ))}
        </ul>
      ) : (
        <div className="empty-state">
          <span className="empty-state-icon">{"\u2691"}</span>
          <span>
            {recommendations.length === 0
              ? "No recommendations yet. Add and analyze a Minecraft instance first."
              : "No recommendations match current filters."}
          </span>
        </div>
      )}

      {configRecs.length > 0 && (
        <div className="detail-section" style={{ marginTop: "var(--space-xl)" }}>
          <h3 className="card-title">Config Recommendations</h3>
          <ul className="config-rec-list">
            {configRecs.map((cr, i) => (
              <li key={i} className="config-rec-card card">
                <div className="config-rec-header">
                  <span className="mono">{cr.file}</span>
                  <span className={`badge badge-${cr.confidence}`}>{cr.confidence}</span>
                </div>
                <div className="config-rec-body">
                  <span className="config-rec-key">{cr.key}:</span>
                  <span className="config-rec-change">
                    {cr.current_value} → {cr.recommended_value}
                  </span>
                </div>
                <div className="config-rec-reason">{cr.reason}</div>
                <div className="config-rec-impact">Impact: {cr.impact}</div>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
