import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useInvoke } from "../hooks/useInvoke";
import type { IgnoreRule } from "../types";

export function Settings() {
  const [theme, setTheme] = useState<"dark" | "light">("dark");
  const [showOnboarding, setShowOnboarding] = useState(false);
  const [deleteConfirm, setDeleteConfirm] = useState(false);
  const ignoreRules = useInvoke<IgnoreRule[]>("get_ignore_rules");

  useEffect(() => {
    ignoreRules.execute();
    initPreferences();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const initPreferences = async () => {
    try {
      const savedTheme = await invoke<string | null>("get_preference", { key: "theme" });
      if (savedTheme === "light" || savedTheme === "dark") {
        setTheme(savedTheme);
        document.body.classList.toggle("light-theme", savedTheme === "light");
      }
      const onboarded = await invoke<string | null>("get_preference", { key: "onboarding_complete" });
      if (onboarded === null) {
        setShowOnboarding(true);
      }
    } catch { /* preferences may not be initialized yet */ }
  };

  const toggleTheme = async () => {
    const next = theme === "dark" ? "light" : "dark";
    setTheme(next);
    document.body.classList.toggle("light-theme", next === "light");
    await invoke("set_preference", { key: "theme", value: next });
  };

  const completeOnboarding = async () => {
    setShowOnboarding(false);
    await invoke("set_preference", { key: "onboarding_complete", value: "true" });
  };

  const removeRule = async (ruleId: string) => {
    await invoke("remove_ignore_rule", { ruleId });
    await ignoreRules.execute();
  };

  const deleteAllData = async () => {
    await invoke("delete_all_data");
    setDeleteConfirm(false);
  };

  return (
    <div className="view-content">
      {showOnboarding && (
        <div className="onboarding-overlay">
          <div className="onboarding-modal card">
            <h2>Welcome to GamePilot</h2>
            <p>
              GamePilot runs entirely on your machine. All performance data, session history,
              and recommendations are stored locally. Nothing is sent to external servers.
            </p>
            <ul className="onboarding-points">
              <li>System diagnostics stay on your device</li>
              <li>Mod analysis is performed locally</li>
              <li>You can delete all data at any time from Settings</li>
            </ul>
            <button className="btn btn-primary" onClick={completeOnboarding}>
              Get Started
            </button>
          </div>
        </div>
      )}

      <div className="view-header">
        <h1>Settings</h1>
      </div>

      <div className="settings-sections">
        <div className="card">
          <h3 className="card-title">Appearance</h3>
          <div className="setting-row">
            <label>Theme</label>
            <button className="btn btn-secondary" onClick={toggleTheme}>
              {theme === "dark" ? "Switch to Light" : "Switch to Dark"}
            </button>
          </div>
        </div>

        <div className="card">
          <h3 className="card-title">Data Retention</h3>
          <p className="setting-note">
            Session data, recommendations, and diagnostics are stored locally on disk.
            Use the danger zone below to clear all stored data.
          </p>
        </div>

        <div className="card">
          <h3 className="card-title">
            Ignore Rules
            {ignoreRules.data && (
              <span className="card-title-count">({ignoreRules.data.length})</span>
            )}
          </h3>
          {ignoreRules.data && ignoreRules.data.length > 0 ? (
            <table>
              <thead>
                <tr>
                  <th>Type</th>
                  <th>Pattern</th>
                  <th>Reason</th>
                  <th>Created</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                {ignoreRules.data.map((rule) => (
                  <tr key={rule.id}>
                    <td>{rule.rule_type}</td>
                    <td className="mono">{rule.pattern}</td>
                    <td>{rule.reason ?? "-"}</td>
                    <td>{new Date(rule.created_at).toLocaleDateString()}</td>
                    <td>
                      <button
                        className="btn btn-danger btn-sm"
                        onClick={() => removeRule(rule.id)}
                      >
                        Remove
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          ) : (
            <div className="empty-state">
              <span>No ignore rules configured</span>
            </div>
          )}
        </div>

        <div className="card card-danger">
          <h3 className="card-title">Danger Zone</h3>
          {!deleteConfirm ? (
            <div className="setting-row">
              <span>Delete all stored data (sessions, recommendations, preferences)</span>
              <button className="btn btn-danger" onClick={() => setDeleteConfirm(true)}>
                Delete All Data
              </button>
            </div>
          ) : (
            <div className="setting-row">
              <span className="text-warn">This action cannot be undone.</span>
              <div className="detail-actions">
                <button className="btn btn-danger" onClick={deleteAllData}>
                  Confirm Delete
                </button>
                <button className="btn btn-secondary" onClick={() => setDeleteConfirm(false)}>
                  Cancel
                </button>
              </div>
            </div>
          )}
        </div>

        <div className="card">
          <h3 className="card-title">About</h3>
          <dl className="detail-dl">
            <dt>Application</dt>
            <dd>GamePilot</dd>
            <dt>Version</dt>
            <dd>0.1.0</dd>
            <dt>Runtime</dt>
            <dd>Tauri 2 + React 19</dd>
            <dt>Purpose</dt>
            <dd>
              Minecraft performance analysis and optimization.
              Scans instances, analyzes mods, provides JVM tuning
              recommendations, and tracks gaming sessions.
            </dd>
          </dl>
        </div>

        <div className="card">
          <h3 className="card-title">Keyboard Shortcuts</h3>
          <table className="shortcuts-table">
            <tbody>
              <tr>
                <td><kbd>1</kbd>-<kbd>6</kbd></td>
                <td>Switch between views</td>
              </tr>
              <tr>
                <td><kbd>R</kbd></td>
                <td>Refresh current view data</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
