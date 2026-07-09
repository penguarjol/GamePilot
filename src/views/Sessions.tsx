import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useInvoke } from "@/hooks/useInvoke";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type { Session, SessionReport, TelemetrySample } from "@/types";

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
        } catch { /* polling errors are non-fatal */ }
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
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold tracking-tight">Sessions</h1>
        <Button variant="secondary" onClick={() => sessions.execute()} disabled={sessions.loading}>
          Refresh
        </Button>
      </div>

      {/* Live telemetry bar */}
      {hasActiveSession && (
        <Card className="border-primary/50">
          <CardContent className="py-3">
            <div className="flex items-center gap-4 flex-wrap">
              <Badge>LIVE</Badge>
              {gameRunning !== null && (
                <Badge variant={gameRunning ? "default" : "destructive"}>
                  {gameRunning ? "Game Running" : "Game Stopped"}
                </Badge>
              )}
              {telemetry && (
                <>
                  <div className="text-sm">
                    <span className="text-muted-foreground">CPU </span>
                    <span className="font-medium">{telemetry.cpu_percent.toFixed(1)}%</span>
                  </div>
                  <div className="text-sm">
                    <span className="text-muted-foreground">RAM </span>
                    <span className="font-medium">{telemetry.ram_used_mb.toFixed(0)} MB</span>
                  </div>
                  <div className="text-sm">
                    <span className="text-muted-foreground">Free </span>
                    <span className="font-medium">{telemetry.ram_available_mb.toFixed(0)} MB</span>
                  </div>
                  {telemetry.top_processes.length > 0 && (
                    <div className="text-xs text-muted-foreground ml-auto">
                      Top: {telemetry.top_processes.slice(0, 3).map((p) =>
                        `${p.name} (${p.cpu_percent.toFixed(0)}%)`
                      ).join(", ")}
                    </div>
                  )}
                </>
              )}
            </div>
          </CardContent>
        </Card>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-[1fr_400px] gap-6">
        {/* Session list */}
        <Card>
          <CardContent className="pt-4">
            {sessions.loading ? (
              <div className="space-y-2">
                {Array.from({ length: 4 }).map((_, i) => (
                  <div key={i} className="h-8 animate-pulse rounded bg-muted" />
                ))}
              </div>
            ) : sessions.data && sessions.data.length > 0 ? (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Status</TableHead>
                    <TableHead>Started</TableHead>
                    <TableHead>Duration</TableHead>
                    <TableHead>Method</TableHead>
                    <TableHead>Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {sessions.data.map((s) => (
                    <TableRow
                      key={s.id}
                      className={`cursor-pointer ${selectedSession?.id === s.id ? "bg-muted/50" : ""}`}
                      onClick={() => selectSession(s)}
                    >
                      <TableCell>
                        <Badge variant={s.status === "active" ? "default" : "secondary"}>
                          {s.status}
                        </Badge>
                      </TableCell>
                      <TableCell>{formatDate(s.started_at)}</TableCell>
                      <TableCell>{s.duration_secs != null ? formatDuration(s.duration_secs) : "-"}</TableCell>
                      <TableCell>{s.launch_method ?? "-"}</TableCell>
                      <TableCell>
                        {s.status === "active" && (
                          <Button
                            variant="destructive"
                            size="xs"
                            onClick={(e) => { e.stopPropagation(); endSession(s); }}
                          >
                            End
                          </Button>
                        )}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            ) : (
              <div className="flex flex-col items-center justify-center h-40 text-muted-foreground">
                <span className="text-3xl mb-2">{"\u25F7"}</span>
                <p className="text-sm">No sessions recorded</p>
                <p className="text-xs">Launch a Minecraft instance to start recording</p>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Session detail */}
        {selectedSession && (
          <Card>
            <CardHeader>
              <CardTitle>Session Report</CardTitle>
            </CardHeader>
            <CardContent>
              {reportLoading ? (
                <div className="space-y-2">
                  {Array.from({ length: 6 }).map((_, i) => (
                    <div key={i} className="h-4 animate-pulse rounded bg-muted" />
                  ))}
                </div>
              ) : reportError ? (
                <p className="text-sm text-destructive">{reportError}</p>
              ) : report ? (
                <div className="space-y-4">
                  <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-sm">
                    <dt className="text-muted-foreground">Status</dt>
                    <dd>
                      <Badge variant={report.session.status === "active" ? "default" : "secondary"}>
                        {report.session.status}
                      </Badge>
                    </dd>
                    <dt className="text-muted-foreground">Started</dt>
                    <dd>{formatDate(report.session.started_at)}</dd>
                    {report.session.ended_at && (
                      <>
                        <dt className="text-muted-foreground">Ended</dt>
                        <dd>{formatDate(report.session.ended_at)}</dd>
                      </>
                    )}
                    {report.session.duration_secs != null && (
                      <>
                        <dt className="text-muted-foreground">Duration</dt>
                        <dd>{formatDuration(report.session.duration_secs)}</dd>
                      </>
                    )}
                    {report.session.cpu_avg_percent != null && (
                      <>
                        <dt className="text-muted-foreground">Avg CPU</dt>
                        <dd>{report.session.cpu_avg_percent.toFixed(1)}%</dd>
                      </>
                    )}
                    {report.session.ram_avg_mb != null && (
                      <>
                        <dt className="text-muted-foreground">Avg RAM</dt>
                        <dd>{report.session.ram_avg_mb.toFixed(0)} MB</dd>
                      </>
                    )}
                    {report.session.ram_peak_mb != null && (
                      <>
                        <dt className="text-muted-foreground">Peak RAM</dt>
                        <dd>{report.session.ram_peak_mb.toFixed(0)} MB</dd>
                      </>
                    )}
                    <dt className="text-muted-foreground">Recs Applied</dt>
                    <dd>{report.recommendations_applied}</dd>
                    <dt className="text-muted-foreground">Observations</dt>
                    <dd>{report.process_observations}</dd>
                  </dl>
                  <div className="border-t border-border pt-3">
                    <p className="text-sm"><strong>Summary:</strong> {report.summary}</p>
                  </div>
                </div>
              ) : null}
            </CardContent>
          </Card>
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
