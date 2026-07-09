import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useInvoke } from "../hooks/useInvoke";
import type { Recommendation, SavedInstance, RollbackPoint } from "../types";

type FilterCategory = "all" | "java_jvm" | "modpack";
type FilterSeverity = "all" | "warning" | "info" | "error";

export function Recommendations() {
  const saved = useInvoke<SavedInstance[]>("get_saved_instances");
  const [recommendations, setRecommendations] = useState<Recommendation[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [filterCategory, setFilterCategory] = useState<FilterCategory>("all");
  const [filterSeverity, setFilterSeverity] = useState<FilterSeverity>("all");
  const [appliedIds, setAppliedIds] = useState<Set<string>>(new Set());

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

  const filtered = recommendations.filter((r) => {
    if (filterCategory !== "all" && r.category !== filterCategory) return false;
    if (filterSeverity !== "all" && r.severity !== filterSeverity) return false;
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
    </div>
  );
}
