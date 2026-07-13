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
import type { Session, SessionReport, TelemetrySample, Recommendation, GovernorStatus, LogEvent } from "@/types";

export function Sessions() {
  const sessions = useInvoke<Session[]>("get_sessions");
  const [selectedSession, setSelectedSession] = useState<Session | null>(null);
  const [report, setReport] = useState<SessionReport | null>(null);
  const [reportLoading, setReportLoading] = useState(false);
  const [reportError, setReportError] = useState<string | null>(null);
  const [telemetry, setTelemetry] = useState<TelemetrySample | null>(null);
  const [gameRunning, setGameRunning] = useState<boolean | null>(null);
  const [governorStatus, setGovernorStatus] = useState<GovernorStatus | null>(null);
  const [logEvents, setLogEvents] = useState<LogEvent[]>([]);
  const [logOpen, setLogOpen] = useState(false);
  const pollRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const samplesRef = useRef<{ cpu: number[]; ram: number[] }>({ cpu: [], ram: [] });
  const logPosRef = useRef(0);
  const lastSummaryRef = useRef(0);
  const sessionsDataRef = useRef(sessions.data);
  useEffect(() => { sessionsDataRef.current = sessions.data; }, [sessions.data]);

  useEffect(() => {
    sessions.execute();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const hasActiveSession = sessions.data?.some((s) => s.status === "active") ?? false;

  useEffect(() => {
    if (!hasActiveSession) {
      setTelemetry(null);
      setGameRunning(null);
      setGovernorStatus(null);
      setLogEvents([]);
      logPosRef.current = 0;
      lastSummaryRef.current = 0;
      return;
    }

    samplesRef.current = { cpu: [], ram: [] };
    logPosRef.current = 0;
    lastSummaryRef.current = Date.now();
    let cancelled = false;

    const schedulePoll = (delayMs: number) => {
      if (cancelled) return;
      pollRef.current = setTimeout(poll, delayMs);
    };

    const poll = async () => {
      if (cancelled) return;

      let intervalMs = 5000;

      let isPaused = false;
      try {
        const gov = await invoke<GovernorStatus>("get_governor_status");
        if (!cancelled) {
          setGovernorStatus(gov);
          intervalMs = gov.telemetry_interval_ms;
          isPaused = gov.mode === "Paused";
        }
      } catch { /* non-fatal */ }

      if (isPaused) {
        schedulePoll(intervalMs);
        return;
      }

      try {
        const sample = await invoke<TelemetrySample>("get_telemetry_sample");
        if (!cancelled) {
          setTelemetry(sample);
          samplesRef.current.cpu.push(sample.cpu_percent);
          samplesRef.current.ram.push(sample.ram_used_mb);
        }
      } catch { /* non-fatal */ }

      const active = sessionsDataRef.current?.find((s) => s.status === "active");

      // Store telemetry summary every 60s
      if (active && Date.now() - lastSummaryRef.current >= 60_000) {
        const { cpu, ram } = samplesRef.current;
        if (cpu.length > 0) {
          const cpuAvg = cpu.reduce((a, b) => a + b, 0) / cpu.length;
          const ramAvg = ram.reduce((a, b) => a + b, 0) / ram.length;
          const ramPeak = Math.max(...ram);
          await invoke("store_telemetry_summary", {
            sessionId: active.id,
            cpuAvg,
            ramAvg,
            ramPeak,
            hogCount: 0,
          }).catch(() => {});
          samplesRef.current = { cpu: [], ram: [] };
          lastSummaryRef.current = Date.now();
        }
      }

      // Tail game log
      if (active) {
        try {
          const [events, newPos] = await invoke<[LogEvent[], number]>("tail_game_log", {
            instancePath: active.instance_id,
            fromPos: logPosRef.current,
          });
          logPosRef.current = newPos;
          if (!cancelled && events.length > 0) {
            setLogEvents((prev) => [...prev, ...events].slice(-50));
          }
        } catch { /* non-fatal */ }
      }

      try {
        const running = await invoke<boolean>("is_game_running", { processName: "java" });
        if (!cancelled) setGameRunning(running);
        if (!running && active) {
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
          try { await sessions.execute(); } catch { /* preserved by useInvoke */ }
          return;
        }
      } catch { /* non-fatal */ }

      schedulePoll(intervalMs);
    };

    poll();

    return () => {
      cancelled = true;
      if (pollRef.current) clearTimeout(pollRef.current);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [hasActiveSession]);

  const [previousSession, setPreviousSession] = useState<Session | null>(null);
  const [nextSteps, setNextSteps] = useState<string[]>([]);

  const selectSession = async (session: Session) => {
    setSelectedSession(session);
    setReport(null);
    setReportError(null);
    setReportLoading(true);
    setPreviousSession(null);
    setNextSteps([]);
    try {
      const r = await invoke<SessionReport>("get_session_report", {
        sessionId: session.id,
      });
      setReport(r);

      const allSessions = sessions.data ?? [];
      const instanceSessions = allSessions
        .filter((s) => s.instance_id === session.instance_id && s.id !== session.id)
        .sort((a, b) => new Date(b.started_at).getTime() - new Date(a.started_at).getTime());
      const sessionTime = new Date(session.started_at).getTime();
      const prev = instanceSessions.find(
        (s) => new Date(s.started_at).getTime() < sessionTime
      );
      setPreviousSession(prev ?? null);

      const steps: string[] = [];
      if (r.session.cpu_avg_percent != null && r.session.cpu_avg_percent > 70) {
        steps.push("Consider closing background applications before next session");
      }
      if (r.session.duration_secs != null && r.session.duration_secs < 600) {
        steps.push(
          `Session lasted only ${formatDuration(r.session.duration_secs)} — check for crashes in Minecraft logs`
        );
      }
      try {
        const recs = await invoke<Recommendation[]>("get_recommendations_for_path", {
          instancePath: "",
          launcher: "Custom",
        }).catch(() => [] as Recommendation[]);
        if (recs.length > 0 && r.recommendations_applied === 0) {
          steps.push(`You have ${recs.length} unreviewed recommendations for this instance`);
        }
      } catch {
        // non-fatal
      }
      setNextSteps(steps);
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

  const handleRefresh = async () => {
    try { await sessions.execute(); } catch { /* useInvoke preserves data */ }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold tracking-tight">Sessions</h1>
        <Button variant="secondary" onClick={handleRefresh} disabled={sessions.loading}>
          Refresh
        </Button>
      </div>

      {/* Live telemetry bar */}
      {hasActiveSession && (
        <div className="space-y-3">
          {governorStatus?.mode === "Paused" && (
            <Card className="border-warning/50">
              <CardContent className="py-3">
                <p className="text-sm text-warning">
                  Telemetry paused — GamePilot is throttling to preserve game performance
                </p>
              </CardContent>
            </Card>
          )}
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
                {governorStatus && governorStatus.mode !== "Paused" && (
                  <div className="text-xs text-muted-foreground ml-auto border-l border-border pl-3">
                    Governor: {governorStatus.mode} / {(governorStatus.telemetry_interval_ms / 1000).toFixed(0)}s interval
                  </div>
                )}
              </div>
            </CardContent>
          </Card>

          {/* Game Log */}
          <Card className="bg-zinc-950 border-zinc-800">
            <CardContent className="py-0">
              <button
                onClick={() => setLogOpen((v) => !v)}
                className="w-full flex items-center justify-between py-3 text-sm text-zinc-300"
              >
                <span className="font-medium">Game Log</span>
                <span className="text-xs text-zinc-500">{logOpen ? "collapse" : "expand"} ({logEvents.length} events)</span>
              </button>
              {logOpen && (
                <div className="pb-3 max-h-60 overflow-y-auto font-mono text-xs space-y-0.5">
                  {logEvents.length === 0 ? (
                    <p className="text-zinc-500 py-2">No log events captured yet</p>
                  ) : (
                    logEvents.map((evt, i) => (
                      <div
                        key={i}
                        className={`py-0.5 px-1 rounded ${
                          evt.level === "ERROR"
                            ? "text-red-400 bg-red-950/30"
                            : evt.level === "WARN"
                              ? "text-yellow-400 bg-yellow-950/20"
                              : "text-zinc-400"
                        }`}
                      >
                        <span className="text-zinc-600 mr-2">{evt.timestamp}</span>
                        <span className="mr-2">[{evt.level}]</span>
                        <span>{evt.message}</span>
                      </div>
                    ))
                  )}
                </div>
              )}
            </CardContent>
          </Card>
        </div>
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

                  {/* Session Comparison */}
                  {previousSession && (
                    <div className="border-t border-border pt-3 space-y-2">
                      <h4 className="text-sm font-medium">Compared to Previous Session</h4>
                      <Table>
                        <TableHeader>
                          <TableRow>
                            <TableHead>Metric</TableHead>
                            <TableHead>This Session</TableHead>
                            <TableHead>Previous</TableHead>
                            <TableHead>Change</TableHead>
                          </TableRow>
                        </TableHeader>
                        <TableBody>
                          {report.session.cpu_avg_percent != null && previousSession.cpu_avg_percent != null && (
                            <TableRow>
                              <TableCell className="text-muted-foreground">CPU avg</TableCell>
                              <TableCell>{report.session.cpu_avg_percent.toFixed(1)}%</TableCell>
                              <TableCell>{previousSession.cpu_avg_percent.toFixed(1)}%</TableCell>
                              <TableCell>
                                <MetricDelta
                                  current={report.session.cpu_avg_percent}
                                  previous={previousSession.cpu_avg_percent}
                                  lowerIsBetter
                                />
                              </TableCell>
                            </TableRow>
                          )}
                          {report.session.ram_avg_mb != null && previousSession.ram_avg_mb != null && (
                            <TableRow>
                              <TableCell className="text-muted-foreground">RAM avg</TableCell>
                              <TableCell>{report.session.ram_avg_mb.toFixed(0)} MB</TableCell>
                              <TableCell>{previousSession.ram_avg_mb.toFixed(0)} MB</TableCell>
                              <TableCell>
                                <MetricDelta
                                  current={report.session.ram_avg_mb}
                                  previous={previousSession.ram_avg_mb}
                                  lowerIsBetter
                                />
                              </TableCell>
                            </TableRow>
                          )}
                          {report.session.duration_secs != null && previousSession.duration_secs != null && (
                            <TableRow>
                              <TableCell className="text-muted-foreground">Duration</TableCell>
                              <TableCell>{formatDuration(report.session.duration_secs)}</TableCell>
                              <TableCell>{formatDuration(previousSession.duration_secs)}</TableCell>
                              <TableCell>
                                <MetricDelta
                                  current={report.session.duration_secs}
                                  previous={previousSession.duration_secs}
                                  lowerIsBetter={false}
                                />
                              </TableCell>
                            </TableRow>
                          )}
                        </TableBody>
                      </Table>
                    </div>
                  )}

                  {/* Actionable Next Steps */}
                  {nextSteps.length > 0 && (
                    <div className="border-t border-border pt-3 space-y-2">
                      <h4 className="text-sm font-medium">Next Steps</h4>
                      <ul className="space-y-1.5">
                        {nextSteps.map((step, i) => (
                          <li key={i} className="text-sm text-muted-foreground flex gap-2">
                            <span className="shrink-0 text-foreground font-medium">{"\u2022"}</span>
                            {step}
                          </li>
                        ))}
                      </ul>
                    </div>
                  )}

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

function MetricDelta({
  current,
  previous,
  lowerIsBetter,
}: {
  current: number;
  previous: number;
  lowerIsBetter: boolean;
}) {
  if (previous === 0) return <span className="text-xs text-muted-foreground">-</span>;
  const pctChange = Math.round(((current - previous) / previous) * 100);
  const isImproved = lowerIsBetter ? pctChange < 0 : pctChange > 0;
  const label = pctChange > 0 ? `+${pctChange}%` : `${pctChange}%`;
  return (
    <span className={`text-xs font-medium ${isImproved ? "text-green-500" : pctChange === 0 ? "text-muted-foreground" : "text-destructive"}`}>
      {label}
    </span>
  );
}
