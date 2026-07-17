import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface DetectedGame {
  game_id: string;
  game_name: string;
  process_name: string;
  window_title: string | null;
  is_running: boolean;
}

interface SelfMetrics {
  cpu_percent: number;
  ram_mb: number;
}

interface ProcessInfo {
  name: string;
  ram_mb: number;
  is_resource_hog: boolean;
}

export function Overlay() {
  const [game, setGame] = useState<DetectedGame | null>(null);
  const [metrics, setMetrics] = useState<SelfMetrics | null>(null);
  const [visible, setVisible] = useState(true);
  const [hogs, setHogs] = useState<string[]>([]);

  useEffect(() => {
    const poll = async () => {
      try {
        const detected = await invoke<DetectedGame | null>("detect_running_game");
        setGame(detected);
      } catch {
        /* command may not exist yet */
      }
      try {
        const m = await invoke<SelfMetrics>("get_self_metrics");
        setMetrics(m);
      } catch {
        /* non-critical */
      }
      try {
        const procs = await invoke<ProcessInfo[]>("get_process_info");
        setHogs(
          procs
            .filter((p) => p.is_resource_hog)
            .map((p) => `${p.name} (${Math.round(p.ram_mb)} MB)`)
        );
      } catch {
        /* non-critical */
      }
    };

    poll();
    const interval = setInterval(poll, 5000);
    return () => clearInterval(interval);
  }, []);

  if (!visible) return null;

  return (
    <div
      style={{
        position: "fixed",
        top: 8,
        right: 8,
        width: 320,
        fontFamily: "'Geist Variable', 'Segoe UI', system-ui, sans-serif",
        fontSize: 13,
        color: "#e8edf5",
        pointerEvents: "auto",
        display: "flex",
        flexDirection: "column",
        gap: 8,
      }}
    >
      {/* Game Status */}
      <div
        style={{
          background: "rgba(15, 20, 30, 0.85)",
          borderRadius: 8,
          padding: "10px 14px",
          backdropFilter: "blur(12px)",
          border: "1px solid rgba(0, 212, 170, 0.3)",
        }}
      >
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          <span style={{ fontWeight: 600, color: "#00d4aa" }}>GamePilot</span>
          <button
            onClick={() => setVisible(false)}
            style={{
              background: "none",
              border: "none",
              color: "#6b7280",
              cursor: "pointer",
              fontSize: 16,
              padding: 0,
            }}
          >
            x
          </button>
        </div>
        {game ? (
          <div style={{ marginTop: 6, fontSize: 12 }}>
            <span style={{ color: "#10b981" }}>Playing: </span>
            <span>{game.game_name}</span>
          </div>
        ) : (
          <div style={{ marginTop: 6, fontSize: 12, color: "#6b7280" }}>No game detected</div>
        )}
      </div>

      {/* Performance */}
      {metrics && (
        <div
          style={{
            background: "rgba(15, 20, 30, 0.85)",
            borderRadius: 8,
            padding: "10px 14px",
            backdropFilter: "blur(12px)",
            border: "1px solid rgba(255, 255, 255, 0.08)",
          }}
        >
          <div style={{ fontSize: 11, color: "#6b7280", marginBottom: 4 }}>GamePilot Usage</div>
          <div style={{ display: "flex", gap: 16, fontSize: 12 }}>
            <span>CPU: {metrics.cpu_percent.toFixed(1)}%</span>
            <span>RAM: {metrics.ram_mb.toFixed(0)} MB</span>
          </div>
        </div>
      )}

      {/* Resource Hogs */}
      {hogs.length > 0 && (
        <div
          style={{
            background: "rgba(30, 20, 10, 0.85)",
            borderRadius: 8,
            padding: "10px 14px",
            backdropFilter: "blur(12px)",
            border: "1px solid rgba(245, 166, 35, 0.3)",
          }}
        >
          <div style={{ fontSize: 11, color: "#f5a623", marginBottom: 4 }}>Resource Hogs</div>
          {hogs.slice(0, 3).map((h, i) => (
            <div key={i} style={{ fontSize: 12, color: "#e8edf5" }}>
              {h}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
