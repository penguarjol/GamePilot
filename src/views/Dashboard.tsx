import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { useInvoke } from "@/hooks/useInvoke";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import type {
  HardwareInfo,
  ProcessInfo,
  DiscoveredLauncher,
  Recommendation,
  Session,
  SavedInstance,
  GameInfo,
  SelfMetrics,
  GovernorStatus,
} from "@/types";

function Skeleton({ className = "" }: { className?: string }) {
  return <div className={`animate-pulse rounded-md bg-muted ${className}`} />;
}

export function Dashboard() {
  const navigate = useNavigate();
  const hw = useInvoke<HardwareInfo>("get_hardware_info");
  const procs = useInvoke<ProcessInfo[]>("get_process_info");
  const launchers = useInvoke<DiscoveredLauncher[]>("discover_launchers");
  const sessions = useInvoke<Session[]>("get_sessions");
  const saved = useInvoke<SavedInstance[]>("get_saved_instances");
  const discoveredGames = useInvoke<GameInfo[]>("discover_all_games");
  const recs = useInvoke<Recommendation[]>("get_recommendations_for_path");
  const selfMetrics = useInvoke<SelfMetrics>("get_self_metrics");
  const governor = useInvoke<GovernorStatus>("get_governor_status");

  useEffect(() => {
    launchers.execute();
    sessions.execute();
    saved.execute();
    discoveredGames.execute();
    selfMetrics.execute();
    governor.execute();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    const sessionPoller = setInterval(async () => {
      try {
        await invoke("auto_detect_and_manage_session");
      } catch {
        /* polling failure is non-fatal */
      }
    }, 10000);
    return () => clearInterval(sessionPoller);
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
  const cpuPercent = hw.data ? Math.round(hw.data.cpu_usage_percent) : 0;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold tracking-tight">Dashboard</h1>
        <div className="flex items-center gap-2">
          <Button onClick={() => invoke("toggle_overlay")} variant="outline" size="sm">
            Toggle Overlay
          </Button>
          <Button onClick={runDiagnostics} disabled={hw.loading || procs.loading}>
            {hw.loading || procs.loading ? "Scanning..." : "Run Diagnostics"}
          </Button>
        </div>
      </div>

      {sessions.data?.some((s) => s.status === "active") && (
        <Card className="border-primary bg-primary/10 mb-4">
          <CardContent className="flex items-center justify-between py-3">
            <div className="flex items-center gap-2">
              <Badge variant="default">LIVE</Badge>
              <span className="text-sm">Gaming session in progress</span>
            </div>
            <Button variant="outline" size="sm" onClick={() => navigate("/sessions")}>
              View Session
            </Button>
          </CardContent>
        </Card>
      )}

      {hw.data?.disks.some((d) => d.free_gb < 15) && (
        <div className="rounded-lg border border-warning bg-warning/10 px-4 py-3 text-sm">
          <span className="font-medium">Low disk space warning:</span>{" "}
          {hw.data.disks
            .filter((d) => d.free_gb < 15)
            .map((d) => `${d.mount_point} (${d.free_gb.toFixed(1)} GB free)`)
            .join(", ")}
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
        {/* System Health */}
        <Card>
          <CardHeader>
            <CardTitle>System Health</CardTitle>
          </CardHeader>
          <CardContent>
            {hw.loading ? (
              <div className="space-y-3">
                <Skeleton className="h-4 w-full" />
                <Skeleton className="h-4 w-3/4" />
                <Skeleton className="h-4 w-1/2" />
              </div>
            ) : hw.data ? (
              <div className="space-y-4">
                <div className="space-y-2">
                  <div className="flex justify-between text-sm">
                    <span className="text-muted-foreground">CPU</span>
                    <span>{cpuPercent}%</span>
                  </div>
                  <Progress value={cpuPercent} />
                </div>
                <div className="space-y-2">
                  <div className="flex justify-between text-sm">
                    <span className="text-muted-foreground">RAM</span>
                    <span>{hw.data.ram_used_mb} / {hw.data.ram_total_mb} MB</span>
                  </div>
                  <Progress value={ramUsedPercent} />
                </div>
                <div className="pt-2 border-t border-border space-y-1 text-sm">
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">GPU</span>
                    <span className="text-right truncate max-w-[60%]">{hw.data.gpu_model}</span>
                  </div>
                  {hw.data.gpu_vram_mb > 0 && (
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">VRAM</span>
                      <span>{hw.data.gpu_vram_mb} MB</span>
                    </div>
                  )}
                  {hw.data.display_refresh_hz > 0 && (
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Display</span>
                      <span>{hw.data.display_refresh_hz} Hz</span>
                    </div>
                  )}
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">OS</span>
                    <span>{hw.data.os_name}</span>
                  </div>
                </div>
              </div>
            ) : hw.error ? (
              <p className="text-sm text-destructive">{hw.error}</p>
            ) : (
              <p className="text-sm text-muted-foreground">
                Click "Run Diagnostics" to scan your system
              </p>
            )}
          </CardContent>
        </Card>

        {/* Discovered Launchers */}
        <Card>
          <CardHeader>
            <CardTitle>Discovered Launchers</CardTitle>
          </CardHeader>
          <CardContent>
            {launchers.loading ? (
              <div className="space-y-2">
                <Skeleton className="h-4 w-full" />
                <Skeleton className="h-4 w-2/3" />
              </div>
            ) : launchers.data && launchers.data.length > 0 ? (
              <ul className="space-y-2">
                {launchers.data.map((l, i) => (
                  <li key={i} className="flex items-center justify-between text-sm">
                    <span className="font-medium">{l.name}</span>
                    <span className="text-muted-foreground text-xs truncate max-w-[50%]">{l.path}</span>
                  </li>
                ))}
              </ul>
            ) : (
              <p className="text-sm text-muted-foreground">No launchers detected</p>
            )}
          </CardContent>
        </Card>

        {/* Resource Hogs */}
        <Card>
          <CardHeader>
            <CardTitle>Resource Hogs</CardTitle>
          </CardHeader>
          <CardContent>
            {procs.data ? (
              resourceHogs.length > 0 ? (
                <ul className="space-y-3">
                  {resourceHogs.slice(0, 5).map((p) => (
                    <li key={p.pid} className="space-y-1">
                      <div className="flex items-center justify-between text-sm">
                        <span className="font-medium font-mono">{p.name}</span>
                        <div className="flex gap-2 text-xs text-muted-foreground">
                          <span>{p.ram_mb.toFixed(0)} MB</span>
                          <span>{p.cpu_percent.toFixed(1)}% CPU</span>
                        </div>
                      </div>
                      <p className="text-xs text-warning">{p.recommendation}</p>
                    </li>
                  ))}
                </ul>
              ) : (
                <p className="text-sm text-muted-foreground">No resource hogs detected</p>
              )
            ) : (
              <p className="text-sm text-muted-foreground">
                Run diagnostics to detect background processes
              </p>
            )}
          </CardContent>
        </Card>

        {/* Top Recommendations */}
        <Card>
          <CardHeader>
            <CardTitle>Top Recommendations</CardTitle>
          </CardHeader>
          <CardContent>
            {recs.data && recs.data.length > 0 ? (
              <ul className="space-y-3">
                {recs.data.slice(0, 3).map((r) => (
                  <li key={r.id} className="space-y-1">
                    <div className="flex gap-1.5">
                      <Badge variant={r.severity === "error" ? "destructive" : "secondary"}>
                        {r.severity}
                      </Badge>
                      <Badge variant="outline">{r.confidence}</Badge>
                    </div>
                    <p className="text-sm">{r.title}</p>
                  </li>
                ))}
              </ul>
            ) : (
              <p className="text-sm text-muted-foreground">
                Add and analyze an instance for recommendations
              </p>
            )}
          </CardContent>
        </Card>

        {/* Recent Sessions */}
        <Card>
          <CardHeader>
            <CardTitle>Recent Sessions</CardTitle>
          </CardHeader>
          <CardContent>
            {sessions.data && sessions.data.length > 0 ? (
              <ul className="space-y-2">
                {sessions.data.slice(0, 4).map((s) => (
                  <li key={s.id} className="flex items-center gap-2 text-sm">
                    <Badge variant={s.status === "active" ? "default" : "secondary"}>
                      {s.status}
                    </Badge>
                    <span className="text-muted-foreground">
                      {new Date(s.started_at).toLocaleDateString()}
                    </span>
                    {s.duration_secs != null && (
                      <span className="ml-auto text-xs text-muted-foreground">
                        {formatDuration(s.duration_secs)}
                      </span>
                    )}
                  </li>
                ))}
              </ul>
            ) : (
              <p className="text-sm text-muted-foreground">No sessions recorded yet</p>
            )}
          </CardContent>
        </Card>

        {/* Discovered Games */}
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0">
            <CardTitle>Games</CardTitle>
            {discoveredGames.data && (
              <Badge variant="secondary">
                {discoveredGames.data.length} detected
              </Badge>
            )}
          </CardHeader>
          <CardContent>
            {discoveredGames.loading ? (
              <div className="space-y-2">
                <Skeleton className="h-4 w-full" />
                <Skeleton className="h-4 w-2/3" />
              </div>
            ) : discoveredGames.data && discoveredGames.data.length > 0 ? (
              <div className="space-y-3">
                <ul className="space-y-2">
                  {discoveredGames.data.slice(0, 4).map((g) => (
                    <li key={g.id} className="flex items-center justify-between text-sm">
                      <span className="font-medium">{g.name}</span>
                      <Badge variant={g.installed ? "default" : "outline"}>
                        {g.installed ? "Installed" : "Not Found"}
                      </Badge>
                    </li>
                  ))}
                </ul>
                <Button
                  variant="outline"
                  size="sm"
                  className="w-full"
                  onClick={() => navigate("/library")}
                >
                  View Library
                </Button>
              </div>
            ) : (
              <p className="text-sm text-muted-foreground">No games detected</p>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Self-metrics footer */}
      {selfMetrics.data && (
        <div className="flex items-center gap-4 rounded-lg border border-border bg-muted/30 px-4 py-2 text-xs text-muted-foreground">
          <span className="font-medium text-foreground">GamePilot</span>
          <span>CPU: {selfMetrics.data.cpu_percent.toFixed(1)}%</span>
          <span>RAM: {selfMetrics.data.ram_mb.toFixed(0)} MB</span>
          {governor.data && (
            <>
              <span className="border-l border-border pl-4">Mode: {governor.data.mode}</span>
              <span>Sampling: {(governor.data.telemetry_interval_ms / 1000).toFixed(0)}s</span>
            </>
          )}
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
