import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { useInvoke } from "@/hooks/useInvoke";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type { HardwareInfo, ProcessInfo, JavaInstallation } from "@/types";

export function Diagnostics() {
  const hw = useInvoke<HardwareInfo>("get_hardware_info");
  const procs = useInvoke<ProcessInfo[]>("get_process_info");
  const java = useInvoke<JavaInstallation[]>("detect_java");
  const [hasScanned, setHasScanned] = useState(false);

  const runScan = async () => {
    setHasScanned(true);
    await Promise.all([hw.execute(), procs.execute(), java.execute()]);
  };

  const ramPercent = hw.data
    ? Math.round((hw.data.ram_used_mb / hw.data.ram_total_mb) * 100)
    : 0;

  const ignoreProcess = async (name: string) => {
    try {
      await invoke("add_ignore_rule", {
        ruleType: "process",
        pattern: name,
        reason: "Ignored from diagnostics",
      });
      toast.success(`"${name}" added to ignore rules`);
    } catch (err) {
      toast.error(String(err));
    }
  };

  if (!hasScanned) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <h1 className="text-2xl font-bold tracking-tight">Diagnostics</h1>
        </div>
        <div className="flex flex-col items-center justify-center h-80 text-muted-foreground space-y-4">
          <span className="text-5xl">{"\u2699"}</span>
          <p className="text-sm">Run a full system scan to view hardware, processes, and Java installations.</p>
          <Button size="lg" onClick={runScan}>Run Scan</Button>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold tracking-tight">Diagnostics</h1>
        <Button onClick={runScan} disabled={hw.loading}>
          {hw.loading ? "Scanning..." : "Refresh"}
        </Button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {/* Hardware */}
        <Card>
          <CardHeader>
            <CardTitle>Hardware</CardTitle>
          </CardHeader>
          <CardContent>
            {hw.loading ? (
              <div className="space-y-2">
                {Array.from({ length: 6 }).map((_, i) => (
                  <div key={i} className="h-4 animate-pulse rounded bg-muted" />
                ))}
              </div>
            ) : hw.data ? (
              <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-sm">
                <dt className="text-muted-foreground">Hostname</dt>
                <dd>{hw.data.hostname}</dd>
                <dt className="text-muted-foreground">OS</dt>
                <dd>{hw.data.os_name} {hw.data.os_version}</dd>
                <dt className="text-muted-foreground">CPU</dt>
                <dd>{hw.data.cpu_model}</dd>
                <dt className="text-muted-foreground">Cores / Threads</dt>
                <dd>{hw.data.cpu_cores} / {hw.data.cpu_threads}</dd>
                <dt className="text-muted-foreground">CPU Usage</dt>
                <dd className="space-y-1">
                  <Progress value={Math.min(hw.data.cpu_usage_percent, 100)} />
                  <span className="text-xs text-muted-foreground">{hw.data.cpu_usage_percent.toFixed(1)}%</span>
                </dd>
                <dt className="text-muted-foreground">RAM</dt>
                <dd className="space-y-1">
                  <Progress value={ramPercent} />
                  <span className="text-xs text-muted-foreground">{hw.data.ram_used_mb} / {hw.data.ram_total_mb} MB ({ramPercent}%)</span>
                </dd>
                <dt className="text-muted-foreground">Available RAM</dt>
                <dd>{hw.data.ram_available_mb} MB</dd>
                <dt className="text-muted-foreground">GPU</dt>
                <dd>{hw.data.gpu_model}</dd>
                {hw.data.gpu_vram_mb > 0 && (
                  <>
                    <dt className="text-muted-foreground">VRAM</dt>
                    <dd>{hw.data.gpu_vram_mb} MB</dd>
                  </>
                )}
                {hw.data.gpu_driver_version && (
                  <>
                    <dt className="text-muted-foreground">GPU Driver</dt>
                    <dd>{hw.data.gpu_driver_version}</dd>
                  </>
                )}
                {hw.data.cpu_freq_mhz > 0 && (
                  <>
                    <dt className="text-muted-foreground">CPU Frequency</dt>
                    <dd>{(hw.data.cpu_freq_mhz / 1000).toFixed(2)} GHz</dd>
                  </>
                )}
              </dl>
            ) : hw.error ? (
              <p className="text-sm text-destructive">{hw.error}</p>
            ) : null}
          </CardContent>
        </Card>

        {/* Java Installations */}
        <Card>
          <CardHeader>
            <CardTitle>Java Installations</CardTitle>
          </CardHeader>
          <CardContent>
            {java.loading ? (
              <div className="space-y-2">
                {Array.from({ length: 3 }).map((_, i) => (
                  <div key={i} className="h-4 animate-pulse rounded bg-muted" />
                ))}
              </div>
            ) : java.data && java.data.length > 0 ? (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Version</TableHead>
                    <TableHead>Vendor</TableHead>
                    <TableHead>64-bit</TableHead>
                    <TableHead>Path</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {java.data.map((j, i) => (
                    <TableRow key={i}>
                      <TableCell>{j.version ?? "Unknown"}</TableCell>
                      <TableCell>{j.vendor ?? "Unknown"}</TableCell>
                      <TableCell>{j.is_64bit ? "Yes" : "No"}</TableCell>
                      <TableCell className="font-mono text-xs max-w-[200px] truncate">{j.path}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            ) : (
              <p className="text-sm text-muted-foreground">No Java installations found</p>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Disks */}
      {hw.data && hw.data.disks.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>Disks</CardTitle>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Mount</TableHead>
                  <TableHead>Total (GB)</TableHead>
                  <TableHead>Free (GB)</TableHead>
                  <TableHead>Used</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {hw.data.disks.map((d, i) => {
                  const usedPercent = d.total_gb > 0 ? Math.round(((d.total_gb - d.free_gb) / d.total_gb) * 100) : 0;
                  return (
                    <TableRow key={i}>
                      <TableCell>{d.name || "-"}</TableCell>
                      <TableCell className="font-mono text-xs">{d.mount_point}</TableCell>
                      <TableCell>{d.total_gb.toFixed(1)}</TableCell>
                      <TableCell className={d.free_gb < 10 ? "text-warning" : ""}>
                        {d.free_gb.toFixed(1)}
                      </TableCell>
                      <TableCell className="w-40">
                        <div className="flex items-center gap-2">
                          <Progress value={usedPercent} />
                          <span className="text-xs text-muted-foreground w-10">{usedPercent}%</span>
                        </div>
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      )}

      {/* Processes */}
      <Card>
        <CardHeader>
          <CardTitle>
            Processes
            {procs.data && <span className="text-muted-foreground font-normal ml-2">({procs.data.length})</span>}
          </CardTitle>
        </CardHeader>
        <CardContent>
          {procs.loading ? (
            <div className="space-y-2">
              {Array.from({ length: 5 }).map((_, i) => (
                <div key={i} className="h-4 animate-pulse rounded bg-muted" />
              ))}
            </div>
          ) : procs.data ? (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>PID</TableHead>
                  <TableHead>CPU %</TableHead>
                  <TableHead>RAM (MB)</TableHead>
                  <TableHead>Category</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {procs.data.map((p) => (
                  <TableRow key={p.pid}>
                    <TableCell className="font-mono text-xs">{p.name}</TableCell>
                    <TableCell>{p.pid}</TableCell>
                    <TableCell className={p.cpu_percent > 15 ? "text-warning" : ""}>
                      {p.cpu_percent.toFixed(1)}
                    </TableCell>
                    <TableCell className={p.ram_mb > 1000 ? "text-warning" : ""}>
                      {p.ram_mb.toFixed(0)}
                    </TableCell>
                    <TableCell>{p.category}</TableCell>
                    <TableCell>
                      <Badge variant={p.is_resource_hog ? "destructive" : "secondary"}>
                        {p.is_resource_hog ? "hog" : "ok"}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      <Button
                        variant="ghost"
                        size="xs"
                        onClick={() => ignoreProcess(p.name)}
                      >
                        Always Ignore
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          ) : procs.error ? (
            <p className="text-sm text-destructive">{procs.error}</p>
          ) : null}
        </CardContent>
      </Card>
    </div>
  );
}
