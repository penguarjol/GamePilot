import { useState, useEffect } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { useInvoke } from "@/hooks/useInvoke";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type {
  MinecraftInstance,
  SavedInstance,
  DiscoveredInstance,
  ModAnalysis,
  Recommendation,
  LaunchResult,
  ConfigAnalysis,
  ModpackHealth,
} from "@/types";

export function Minecraft() {
  const saved = useInvoke<SavedInstance[]>("get_saved_instances");
  const [selectedInstance, setSelectedInstance] = useState<MinecraftInstance | null>(null);
  const [modAnalysis, setModAnalysis] = useState<ModAnalysis | null>(null);
  const [recommendations, setRecommendations] = useState<Recommendation[]>([]);
  const [configAnalysis, setConfigAnalysis] = useState<ConfigAnalysis | null>(null);
  const [modpackHealth, setModpackHealth] = useState<ModpackHealth | null>(null);
  const [loading, setLoading] = useState<string | null>(null);
  const [launchResult, setLaunchResult] = useState<LaunchResult | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [discovered, setDiscovered] = useState<DiscoveredInstance[]>([]);
  const [discovering, setDiscovering] = useState(false);

  useEffect(() => {
    saved.execute();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const runAutoDetect = async () => {
    setDiscovering(true);
    try {
      const result = await invoke<DiscoveredInstance[]>("discover_all_instances");
      setDiscovered(result);
    } catch (err) {
      toast.error(`Detection failed: ${err}`);
    } finally {
      setDiscovering(false);
    }
  };

  const addFromDiscovered = async (inst: DiscoveredInstance) => {
    setDialogOpen(false);
    setLoading("Scanning instance...");
    try {
      const full = await invoke<MinecraftInstance>("scan_instance", {
        path: inst.path,
        launcher: inst.launcher,
      });
      setSelectedInstance(full);
      await invoke("save_instance", { instanceJson: JSON.stringify(full) });
      await saved.execute();
      toast.success(`Added "${inst.name}"`);
      await runAnalysis(full);
    } catch (err) {
      toast.error(String(err));
    } finally {
      setLoading(null);
    }
  };

  const browseForInstance = async () => {
    setDialogOpen(false);
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
      toast.success(`Added "${instance.name}"`);
      await runAnalysis(instance);
    } catch (err) {
      toast.error(String(err));
    } finally {
      setLoading(null);
    }
  };

  const selectSaved = async (inst: SavedInstance) => {
    setModAnalysis(null);
    setRecommendations([]);
    setConfigAnalysis(null);
    setModpackHealth(null);
    setLaunchResult(null);
    setLoading("Scanning instance...");
    try {
      const full = await invoke<MinecraftInstance>("scan_instance", {
        path: inst.path,
        launcher: inst.launcher ?? "Custom",
      });
      setSelectedInstance(full);
    } catch (err) {
      toast.error(String(err));
    } finally {
      setLoading(null);
    }
  };

  const deleteInstance = async (instId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await invoke("delete_instance", { instanceId: instId });
      if (selectedInstance?.id === instId) {
        setSelectedInstance(null);
        setModAnalysis(null);
        setRecommendations([]);
        setConfigAnalysis(null);
        setModpackHealth(null);
      }
      await saved.execute();
      toast.success("Instance removed");
    } catch (err) {
      toast.error(String(err));
    }
  };

  const runAnalysis = async (instance: MinecraftInstance) => {
    setLoading("Analyzing...");
    try {
      if (instance.mods_path) {
        const analysis = await invoke<ModAnalysis>("analyze_mods", {
          modsPath: instance.mods_path,
          loader: instance.loader_type,
        });
        setModAnalysis(analysis);

        const health = await invoke<ModpackHealth>("get_modpack_health", {
          modsPath: instance.mods_path,
          loader: instance.loader_type,
          hasConfigIssues: false,
        });
        setModpackHealth(health);
      }
      if (instance.path) {
        const config = await invoke<ConfigAnalysis>("analyze_configs", {
          instancePath: instance.path,
          modCount: instance.mod_count,
        });
        setConfigAnalysis(config);
      }
      const recs = await invoke<Recommendation[]>("get_recommendations", {
        instanceJson: JSON.stringify(instance),
      });
      setRecommendations(recs);
    } catch (err) {
      toast.error(String(err));
    } finally {
      setLoading(null);
    }
  };

  const analyzeInstance = () => {
    if (selectedInstance) runAnalysis(selectedInstance);
  };

  const launchInstance = async () => {
    if (!selectedInstance) return;
    setLoading("Launching...");
    try {
      const result = await invoke<LaunchResult>("launch_instance", {
        instanceId: selectedInstance.id,
        launcher: selectedInstance.launcher,
        instancePath: selectedInstance.path,
      });
      setLaunchResult(result);
      if (result.success) toast.success(result.message);
      else toast.error(result.message);
    } catch (err) {
      toast.error(String(err));
    } finally {
      setLoading(null);
    }
  };

  const updateRecStatus = async (recId: string, status: string) => {
    try {
      await invoke("update_recommendation_status", { recommendationId: recId, status });
      toast.success(`Status updated: ${status.replace("_", " ")}`);
    } catch (err) {
      toast.error(String(err));
    }
  };

  const riskColor = (score: number) =>
    score >= 70 ? "text-success" : score >= 40 ? "text-warning" : "text-destructive";

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold tracking-tight">Minecraft</h1>
        <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
          <DialogTrigger render={<Button />}>+ Add Instance</DialogTrigger>
          <DialogContent className="sm:max-w-lg">
            <DialogHeader>
              <DialogTitle>Add Instance</DialogTitle>
            </DialogHeader>
            <Tabs defaultValue="auto">
              <TabsList>
                <TabsTrigger value="auto">Auto-Detect</TabsTrigger>
                <TabsTrigger value="browse">Browse</TabsTrigger>
              </TabsList>
              <TabsContent value="auto" className="mt-4 space-y-3">
                <Button onClick={runAutoDetect} disabled={discovering} variant="secondary" className="w-full">
                  {discovering ? "Scanning..." : "Scan for Instances"}
                </Button>
                {discovered.length > 0 && (
                  <div className="max-h-60 overflow-y-auto space-y-2">
                    {discovered.map((inst) => (
                      <button
                        key={inst.path}
                        onClick={() => addFromDiscovered(inst)}
                        className="w-full text-left rounded-lg border border-border p-3 hover:bg-muted/50 transition-colors"
                      >
                        <div className="font-medium text-sm">{inst.name}</div>
                        <div className="text-xs text-muted-foreground mt-0.5">
                          {inst.launcher}
                          {inst.minecraft_version ? ` / ${inst.minecraft_version}` : ""}
                          {inst.mod_count > 0 ? ` / ${inst.mod_count} mods` : ""}
                        </div>
                      </button>
                    ))}
                  </div>
                )}
              </TabsContent>
              <TabsContent value="browse" className="mt-4 space-y-3">
                <p className="text-sm text-muted-foreground">
                  Select a Minecraft instance folder. Common locations:
                </p>
                <ul className="text-xs text-muted-foreground space-y-1 font-mono">
                  <li>CurseForge: ~/curseforge/minecraft/Instances/...</li>
                  <li>MultiMC: ~/.local/share/multimc/instances/...</li>
                  <li>Prism: ~/.local/share/PrismLauncher/instances/...</li>
                </ul>
                <Button onClick={browseForInstance} className="w-full">
                  Choose Folder
                </Button>
              </TabsContent>
            </Tabs>
          </DialogContent>
        </Dialog>
      </div>

      {loading && (
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <div className="h-3 w-3 rounded-full bg-primary animate-pulse" />
          {loading}
        </div>
      )}

      <div className="flex gap-6">
        {/* Left panel: instance list */}
        <div className="w-64 shrink-0 space-y-2">
          <h3 className="text-sm font-medium text-muted-foreground px-1">Instances</h3>
          {saved.data && saved.data.length > 0 ? (
            <div className="space-y-1">
              {saved.data.map((inst) => (
                <div
                  key={inst.id}
                  className={`group relative flex flex-col rounded-lg border px-3 py-2 cursor-pointer transition-colors ${
                    selectedInstance?.id === inst.id
                      ? "border-primary bg-primary/10"
                      : "border-border hover:bg-muted/50"
                  }`}
                  onClick={() => selectSaved(inst)}
                  onKeyDown={(e) => e.key === "Enter" && selectSaved(inst)}
                  tabIndex={0}
                  role="button"
                >
                  <span className="text-sm font-medium truncate pr-5">{inst.name}</span>
                  <span className="text-xs text-muted-foreground">
                    {inst.minecraft_version ?? "Unknown"}
                    {inst.loader_type ? ` - ${inst.loader_type}` : ""}
                    {inst.mod_count ? ` (${inst.mod_count} mods)` : ""}
                  </span>
                  <button
                    className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-destructive transition-opacity text-xs"
                    onClick={(e) => deleteInstance(inst.id, e)}
                    title="Remove instance"
                  >
                    x
                  </button>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground px-1">
              No instances. Click "Add Instance" to get started.
            </p>
          )}
        </div>

        {/* Right panel: instance detail */}
        <div className="flex-1 min-w-0 space-y-4">
          {selectedInstance ? (
            <>
              <Card>
                <CardHeader>
                  <div className="flex items-start justify-between">
                    <div>
                      <CardTitle className="text-lg">{selectedInstance.name}</CardTitle>
                      <p className="text-xs text-muted-foreground mt-1 font-mono truncate">
                        {selectedInstance.path}
                      </p>
                    </div>
                    <div className="flex gap-2 shrink-0">
                      <Button variant="secondary" onClick={analyzeInstance} disabled={!!loading}>
                        Analyze
                      </Button>
                      <Button onClick={launchInstance} disabled={!!loading}>
                        Launch
                      </Button>
                    </div>
                  </div>
                </CardHeader>
                <CardContent>
                  {launchResult && (
                    <div className={`mb-4 rounded-lg p-3 text-sm ${launchResult.success ? "bg-success/10 text-success" : "bg-destructive/10 text-destructive"}`}>
                      <strong>{launchResult.success ? "Launched" : "Failed"}</strong> — {launchResult.message}
                    </div>
                  )}
                  <div className="grid grid-cols-2 gap-6">
                    <div className="space-y-2 text-sm">
                      <h4 className="font-medium">Instance Info</h4>
                      <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1">
                        <dt className="text-muted-foreground">Version</dt>
                        <dd>{selectedInstance.minecraft_version ?? "Unknown"}</dd>
                        <dt className="text-muted-foreground">Loader</dt>
                        <dd>{selectedInstance.loader_type ?? "None"}{selectedInstance.loader_version ? ` (${selectedInstance.loader_version})` : ""}</dd>
                        <dt className="text-muted-foreground">Launcher</dt>
                        <dd>{selectedInstance.launcher}</dd>
                        <dt className="text-muted-foreground">Mods</dt>
                        <dd>{selectedInstance.mod_count}</dd>
                      </dl>
                    </div>
                    <div className="space-y-2 text-sm">
                      <h4 className="font-medium">JVM Settings</h4>
                      <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1">
                        <dt className="text-muted-foreground">Java</dt>
                        <dd className="font-mono text-xs truncate">{selectedInstance.java_path ?? "Default"}</dd>
                        <dt className="text-muted-foreground">Max RAM</dt>
                        <dd>{selectedInstance.xmx_mb ? `${selectedInstance.xmx_mb} MB` : "Not set"}</dd>
                        <dt className="text-muted-foreground">Min RAM</dt>
                        <dd>{selectedInstance.xms_mb ? `${selectedInstance.xms_mb} MB` : "Not set"}</dd>
                      </dl>
                    </div>
                  </div>
                </CardContent>
              </Card>

              {/* Health Score */}
              {modpackHealth && (
                <Card>
                  <CardHeader>
                    <CardTitle>Modpack Health</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="flex items-center gap-6">
                      <div className="text-center">
                        <span className={`text-4xl font-bold ${riskColor(modpackHealth.overall_score)}`}>
                          {modpackHealth.overall_score}
                        </span>
                        <span className="text-lg text-muted-foreground"> / 100</span>
                      </div>
                      <p className="text-sm text-muted-foreground flex-1">{modpackHealth.summary}</p>
                    </div>
                    <div className="mt-4 grid grid-cols-1 sm:grid-cols-2 gap-3">
                      {([
                        { label: "Memory", risk: modpackHealth.memory_risk },
                        { label: "Rendering", risk: modpackHealth.rendering_risk },
                        { label: "Startup", risk: modpackHealth.startup_risk },
                        { label: "Dependency", risk: modpackHealth.dependency_risk },
                        { label: "Optimization", risk: modpackHealth.optimization_score },
                      ] as const).map(({ label, risk }) => (
                        <div key={label} className="space-y-1">
                          <div className="flex justify-between text-xs">
                            <span className="text-muted-foreground">{label}</span>
                            <span className={riskColor(risk.score)}>{risk.score}</span>
                          </div>
                          <Progress value={risk.score} />
                        </div>
                      ))}
                    </div>
                  </CardContent>
                </Card>
              )}

              {/* Mod Analysis */}
              {modAnalysis && (
                <Card>
                  <CardHeader>
                    <CardTitle>
                      Mod Analysis ({modAnalysis.total_mods} mods, {modAnalysis.total_size_mb.toFixed(1)} MB)
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    {modAnalysis.detected_performance_mods.length > 0 && (
                      <div className="flex flex-wrap gap-1.5">
                        <span className="text-xs text-muted-foreground mr-1">Performance mods:</span>
                        {modAnalysis.detected_performance_mods.map((m) => (
                          <Badge key={m} variant="secondary">{m}</Badge>
                        ))}
                      </div>
                    )}
                    {modAnalysis.mods.length > 0 && (
                      <Table>
                        <TableHeader>
                          <TableRow>
                            <TableHead>File</TableHead>
                            <TableHead>Version</TableHead>
                            <TableHead className="text-right">Size</TableHead>
                          </TableRow>
                        </TableHeader>
                        <TableBody>
                          {modAnalysis.mods.slice(0, 50).map((mod, i) => (
                            <TableRow key={i}>
                              <TableCell className="font-medium">{mod.display_name ?? mod.file_name}</TableCell>
                              <TableCell className="text-muted-foreground">{mod.version ?? "-"}</TableCell>
                              <TableCell className="text-right">{(mod.size_bytes / 1024 / 1024).toFixed(1)} MB</TableCell>
                            </TableRow>
                          ))}
                        </TableBody>
                      </Table>
                    )}
                    {modAnalysis.mods.length > 50 && (
                      <p className="text-xs text-muted-foreground text-center">
                        ...and {modAnalysis.mods.length - 50} more
                      </p>
                    )}
                  </CardContent>
                </Card>
              )}

              {/* Config Recommendations */}
              {configAnalysis && configAnalysis.recommendations.length > 0 && (
                <Card>
                  <CardHeader>
                    <CardTitle>Config Recommendations</CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-3">
                    {configAnalysis.recommendations.map((cr, i) => (
                      <div key={i} className="rounded-lg border border-border p-3 space-y-1">
                        <div className="flex items-center justify-between">
                          <span className="font-mono text-xs">{cr.file}</span>
                          <Badge variant="outline">{cr.confidence}</Badge>
                        </div>
                        <div className="text-sm">
                          <span className="text-muted-foreground">{cr.key}: </span>
                          <span className="line-through text-muted-foreground">{cr.current_value}</span>
                          <span className="mx-1.5 text-muted-foreground">{"\u2192"}</span>
                          <span className="text-primary font-medium">{cr.recommended_value}</span>
                        </div>
                        <p className="text-xs text-muted-foreground">{cr.reason}</p>
                      </div>
                    ))}
                  </CardContent>
                </Card>
              )}

              {/* Recommendations */}
              {recommendations.length > 0 && (
                <Card>
                  <CardHeader>
                    <CardTitle>Recommendations</CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    {recommendations.map((r) => (
                      <div key={r.id} className="rounded-lg border border-border p-4 space-y-2">
                        <div className="flex flex-wrap gap-1.5">
                          <Badge variant={r.severity === "error" ? "destructive" : "secondary"}>{r.severity}</Badge>
                          <Badge variant="outline">{r.confidence}</Badge>
                          <Badge variant="outline">risk: {r.risk_level}</Badge>
                        </div>
                        <h4 className="font-medium text-sm">{r.title}</h4>
                        <p className="text-sm text-muted-foreground">{r.description}</p>
                        <p className="text-xs text-muted-foreground">{r.evidence}</p>
                        <div className="flex gap-2 pt-1">
                          <Button size="sm" onClick={() => updateRecStatus(r.id, "accepted")}>Accept</Button>
                          <Button size="sm" variant="secondary" onClick={() => updateRecStatus(r.id, "ignored_once")}>Ignore</Button>
                          <Button size="sm" variant="secondary" onClick={() => updateRecStatus(r.id, "deferred")}>Defer</Button>
                          {r.action_type === "open_link" && r.action_data && (
                            <a href={r.action_data} target="_blank" rel="noreferrer">
                              <Button size="sm" variant="secondary">Open Link</Button>
                            </a>
                          )}
                        </div>
                      </div>
                    ))}
                  </CardContent>
                </Card>
              )}
            </>
          ) : (
            <div className="flex flex-col items-center justify-center h-64 text-muted-foreground">
              <span className="text-4xl mb-3">{"\u25A3"}</span>
              <p className="text-sm">Select an instance or add a new one</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
