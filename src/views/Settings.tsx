import { useState } from "react";

export function Settings() {
  const [retentionDays, setRetentionDays] = useState(30);
  const [saved, setSaved] = useState(false);

  const handleSave = () => {
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  return (
    <div className="view-content">
      <div className="view-header">
        <h1>Settings</h1>
      </div>

      <div className="settings-sections">
        <div className="card">
          <h3 className="card-title">Data Retention</h3>
          <div className="setting-row">
            <label htmlFor="retention">Keep session data for</label>
            <div className="setting-control">
              <select
                id="retention"
                value={retentionDays}
                onChange={(e) => setRetentionDays(Number(e.target.value))}
              >
                <option value={7}>7 days</option>
                <option value={14}>14 days</option>
                <option value={30}>30 days</option>
                <option value={60}>60 days</option>
                <option value={90}>90 days</option>
                <option value={365}>1 year</option>
              </select>
            </div>
          </div>
          <div className="setting-actions">
            <button className="btn btn-primary" onClick={handleSave}>
              Save
            </button>
            {saved && <span className="setting-saved">Settings saved</span>}
          </div>
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
