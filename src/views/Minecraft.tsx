import { useState, useEffect } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { writeTextFile, readTextFile } from "@tauri-apps/plugin-fs";
import { toast } from "sonner";
import { useInvoke } from "@/hooks/useInvoke";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
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
  ConfigRecommendation,
  ModpackHealth,
  RollbackPoint,
  ProcessInfo,
  ModSearchResult,
  ModVersion,
  ModFile,
  InstallResult,
  OptimizationProfile,
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
  const [recStatusMap, setRecStatusMap] = useState<Record<string, string>>({});
  const [appliedChanges, setAppliedChanges] = useState<RollbackPoint[]>([]);
  const [preLaunchOpen, setPreLaunchOpen] = useState(false);
  const [resourceHogs, setResourceHogs] = useState<ProcessInfo[]>([]);

  const [modSearchQuery, setModSearchQuery] = useState("");
  const [modSearchResults, setModSearchResults] = useState<ModSearchResult[]>([]);
  const [modSearching, setModSearching] = useState(false);
  const [installDialogOpen, setInstallDialogOpen] = useState(false);
  const [pendingInstall, setPendingInstall] = useState<{ mod: ModSearchResult; file: ModFile } | null>(null);
  const [installing, setInstalling] = useState(false);
  const [quickInstalling, setQuickInstalling] = useState<Record<string, boolean>>({});

  const [shareDialogOpen, setShareDialogOpen] = useState(false);
  const [shareProfileJson, setShareProfileJson] = useState("");
  const [shareLoading, setShareLoading] = useState(false);
  const [importJson, setImportJson] = useState("");
  const [importedProfile, setImportedProfile] = useState<OptimizationProfile | null>(null);
  const [discordWebhookUrl, setDiscordWebhookUrl] = useState("");
  const [discordSending, setDiscordSending] = useState(false);

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
      setRecStatusMap((prev) => ({ ...prev, [recId]: status }));
      toast.success(`Recommendation ${status.replace("_", " ")}`);
    } catch (err) {
      toast.error(`Failed: ${err}`);
    }
  };

  const applyConfig = async (cr: ConfigRecommendation) => {
    if (!selectedInstance) return;
    try {
      const rp = await invoke<RollbackPoint>("apply_config_change_auto", {
        instancePath: selectedInstance.path,
        filename: cr.file,
        key: cr.key,
        newValue: cr.recommended_value,
        recommendationId: `config-${cr.key}`,
      });
      setAppliedChanges((prev) => [...prev, rp]);
      toast.success(`Applied: ${cr.key} = ${cr.recommended_value}`);
      const config = await invoke<ConfigAnalysis>("analyze_configs", {
        instancePath: selectedInstance.path,
        modCount: selectedInstance.mod_count,
      });
      setConfigAnalysis(config);
    } catch (err) {
      toast.error(`Failed: ${err}`);
    }
  };

  const rollbackChange = async (rp: RollbackPoint) => {
    if (!selectedInstance) return;
    try {
      await invoke("rollback_file", { rollbackJson: JSON.stringify(rp) });
      setAppliedChanges((prev) => prev.filter((c) => c.id !== rp.id));
      toast.success(`Rolled back: ${rp.file_path}`);
      const config = await invoke<ConfigAnalysis>("analyze_configs", {
        instancePath: selectedInstance.path,
        modCount: selectedInstance.mod_count,
      });
      setConfigAnalysis(config);
    } catch (err) {
      toast.error(`Rollback failed: ${err}`);
    }
  };

  const handleLaunchClick = async () => {
    if (!selectedInstance) return;
    const unaddressedRecs = recommendations.filter((r) => !recStatusMap[r.id]).slice(0, 3);
    let hogs: ProcessInfo[] = [];
    try {
      const procs = await invoke<ProcessInfo[]>("get_process_info");
      hogs = procs.filter((p) => p.is_resource_hog);
      setResourceHogs(hogs);
    } catch {
      setResourceHogs([]);
    }

    if (unaddressedRecs.length > 0 || hogs.length > 0) {
      setPreLaunchOpen(true);
    } else {
      await launchInstance();
    }
  };

  const formatDownloads = (n: number): string => {
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
    if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
    return String(n);
  };

  const searchMods = async () => {
    if (!modSearchQuery.trim() || !selectedInstance) return;
    setModSearching(true);
    try {
      const results = await invoke<ModSearchResult[]>("search_modrinth_mods", {
        query: modSearchQuery.trim(),
        mcVersion: selectedInstance.minecraft_version ?? undefined,
        loader: selectedInstance.loader_type ?? undefined,
        limit: 20,
      });
      setModSearchResults(results);
    } catch (err) {
      toast.error(`Search failed: ${err}`);
    } finally {
      setModSearching(false);
    }
  };

  const prepareInstall = async (mod: ModSearchResult) => {
    if (!selectedInstance) return;
    try {
      const versions = await invoke<ModVersion[]>("get_modrinth_mod_versions", {
        projectId: mod.project_id,
        mcVersion: selectedInstance.minecraft_version ?? undefined,
        loader: selectedInstance.loader_type ?? undefined,
      });
      if (versions.length === 0) {
        toast.error("No compatible versions found");
        return;
      }
      const latest = versions[0];
      const primary = latest.files.find((f) => f.primary) ?? latest.files[0];
      if (!primary) {
        toast.error("No downloadable file found");
        return;
      }
      setPendingInstall({ mod, file: primary });
      setInstallDialogOpen(true);
    } catch (err) {
      toast.error(`Failed to fetch versions: ${err}`);
    }
  };

  const confirmInstall = async () => {
    if (!pendingInstall || !selectedInstance?.mods_path) return;
    setInstalling(true);
    try {
      const result = await invoke<InstallResult>("install_modrinth_mod", {
        downloadUrl: pendingInstall.file.url,
        filename: pendingInstall.file.filename,
        modsDir: selectedInstance.mods_path,
      });
      if (result.success) {
        toast.success(`Installed ${result.filename}`);
        setInstallDialogOpen(false);
        setPendingInstall(null);
        await runAnalysis(selectedInstance);
      } else {
        toast.error(result.message);
      }
    } catch (err) {
      toast.error(`Install failed: ${err}`);
    } finally {
      setInstalling(false);
    }
  };

  const toggleMod = async (filename: string, disabled: boolean) => {
    if (!selectedInstance?.mods_path) return;
    try {
      if (disabled) {
        await invoke<string>("enable_mod", { modsDir: selectedInstance.mods_path, filename });
        toast.success(`Enabled ${filename.replace(".disabled", "")}`);
      } else {
        await invoke<string>("remove_mod", { modsDir: selectedInstance.mods_path, filename });
        toast.success(`Disabled ${filename}`);
      }
      await runAnalysis(selectedInstance);
    } catch (err) {
      toast.error(`Failed: ${err}`);
    }
  };

  const quickInstallMod = async (modName: string) => {
    if (!selectedInstance?.mods_path) return;
    setQuickInstalling((prev) => ({ ...prev, [modName]: true }));
    try {
      const results = await invoke<ModSearchResult[]>("search_modrinth_mods", {
        query: modName,
        mcVersion: selectedInstance.minecraft_version ?? undefined,
        loader: selectedInstance.loader_type ?? undefined,
        limit: 5,
      });
      if (results.length === 0) {
        toast.error(`"${modName}" not found on Modrinth`);
        return;
      }
      const match = results[0];
      const versions = await invoke<ModVersion[]>("get_modrinth_mod_versions", {
        projectId: match.project_id,
        mcVersion: selectedInstance.minecraft_version ?? undefined,
        loader: selectedInstance.loader_type ?? undefined,
      });
      if (versions.length === 0) {
        toast.error(`No compatible version of "${modName}" found`);
        return;
      }
      const primary = versions[0].files.find((f) => f.primary) ?? versions[0].files[0];
      if (!primary) {
        toast.error("No downloadable file");
        return;
      }
      const result = await invoke<InstallResult>("install_modrinth_mod", {
        downloadUrl: primary.url,
        filename: primary.filename,
        modsDir: selectedInstance.mods_path,
      });
      if (result.success) {
        toast.success(`Installed ${result.filename}`);
        await runAnalysis(selectedInstance);
      } else {
        toast.error(result.message);
      }
    } catch (err) {
      toast.error(`Quick install failed: ${err}`);
    } finally {
      setQuickInstalling((prev) => ({ ...prev, [modName]: false }));
    }
  };

  const openShareDialog = async () => {
    if (!selectedInstance) return;
    setShareLoading(true);
    setShareDialogOpen(true);
    try {
      const json = await invoke<string>("export_optimization_profile", {
        instancePath: selectedInstance.path,
        launcher: selectedInstance.launcher,
      });
      setShareProfileJson(json);
    } catch (err) {
      toast.error(`Failed to generate profile: ${err}`);
      setShareProfileJson("");
    } finally {
      setShareLoading(false);
    }
  };

  const copyProfileToClipboard = async () => {
    try {
      await navigator.clipboard.writeText(shareProfileJson);
      toast.success("Profile copied to clipboard");
    } catch {
      toast.error("Failed to copy");
    }
  };

  const saveProfileToFile = async () => {
    try {
      const filePath = await save({
        defaultPath: `${selectedInstance?.name ?? "profile"}.gamepilot`,
        filters: [{ name: "GamePilot Profile", extensions: ["gamepilot", "json"] }],
      });
      if (filePath) {
        await writeTextFile(filePath, shareProfileJson);
        toast.success("Profile saved");
      }
    } catch (err) {
      toast.error(`Save failed: ${err}`);
    }
  };

  const handleShareToDiscord = async () => {
    if (!selectedInstance || !discordWebhookUrl.trim()) return;
    setDiscordSending(true);
    try {
      await invoke("share_to_discord", {
        webhookUrl: discordWebhookUrl.trim(),
        instancePath: selectedInstance.path,
        launcher: selectedInstance.launcher,
      });
      toast.success("Shared to Discord");
    } catch (err) {
      toast.error(`Discord share failed: ${err}`);
    } finally {
      setDiscordSending(false);
    }
  };

  const handleImportFromText = async () => {
    if (!importJson.trim()) return;
    try {
      const profile = await invoke<OptimizationProfile>("import_optimization_profile", {
        json: importJson.trim(),
      });
      setImportedProfile(profile);
    } catch (err) {
      toast.error(`Import failed: ${err}`);
    }
  };

  const handleImportFromFile = async () => {
    try {
      const filePath = await open({
        filters: [{ name: "GamePilot Profile", extensions: ["gamepilot", "json"] }],
      });
      if (!filePath) return;
      const content = await readTextFile(filePath as string);
      const profile = await invoke<OptimizationProfile>("import_optimization_profile", {
        json: content,
      });
      setImportJson(content);
      setImportedProfile(profile);
    } catch (err) {
      toast.error(`Import failed: ${err}`);
    }
  };

  const parseJvmActionData = (actionData: string) => {
    let xmxMb: number | undefined;
    let xmsMb: number | undefined;
    const otherArgs: string[] = [];

    for (const token of actionData.split(/\s+/)) {
      const xmxMatch = token.match(/^-Xmx(\d+)m$/i);
      const xmsMatch = token.match(/^-Xms(\d+)m$/i);
      if (xmxMatch) xmxMb = parseInt(xmxMatch[1], 10);
      else if (xmsMatch) xmsMb = parseInt(xmsMatch[1], 10);
      else if (token.trim()) otherArgs.push(token);
    }

    return { xmxMb, xmsMb, jvmArgs: otherArgs.length > 0 ? otherArgs.join(" ") : undefined };
  };

  const applyJvmSettings = async (rec: Recommendation) => {
    if (!selectedInstance || !rec.action_data) return;
    const { xmxMb, xmsMb, jvmArgs } = parseJvmActionData(rec.action_data);
    try {
      const rp = await invoke<RollbackPoint>("apply_jvm_settings", {
        instancePath: selectedInstance.path,
        xmxMb: xmxMb ?? null,
        xmsMb: xmsMb ?? null,
        jvmArgs: jvmArgs ?? null,
        javaPath: null,
        recommendationId: rec.id,
      });
      setAppliedChanges((prev) => [...prev, rp]);
      setRecStatusMap((prev) => ({ ...prev, [rec.id]: "applied" }));
      toast.success("JVM settings applied");
      const refreshed = await invoke<MinecraftInstance>("scan_instance", {
        path: selectedInstance.path,
        launcher: selectedInstance.launcher,
      });
      setSelectedInstance(refreshed);
    } catch (err) {
      toast.error(`Failed to apply JVM settings: ${err}`);
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
                      <Button variant="secondary" onClick={openShareDialog} disabled={!!loading}>
                        Share
                      </Button>
                      <Button onClick={handleLaunchClick} disabled={!!loading}>
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
                    {modAnalysis.missing_performance_mods.length > 0 && (
                      <div className="space-y-2">
                        <span className="text-xs text-muted-foreground">Recommended performance mods:</span>
                        {modAnalysis.missing_performance_mods.map((rec) => (
                          <div key={rec.mod_id} className="flex items-center justify-between rounded-lg border border-border p-2">
                            <div className="min-w-0 flex-1">
                              <span className="text-sm font-medium">{rec.mod_name}</span>
                              <p className="text-xs text-muted-foreground truncate">{rec.reason}</p>
                            </div>
                            <Button
                              size="sm"
                              variant="secondary"
                              disabled={quickInstalling[rec.mod_name]}
                              onClick={() => quickInstallMod(rec.mod_name)}
                            >
                              {quickInstalling[rec.mod_name] ? "Installing..." : "Install from Modrinth"}
                            </Button>
                          </div>
                        ))}
                      </div>
                    )}
                    {modAnalysis.mods.length > 0 && (
                      <Table>
                        <TableHeader>
                          <TableRow>
                            <TableHead>File</TableHead>
                            <TableHead>Version</TableHead>
                            <TableHead>Status</TableHead>
                            <TableHead className="text-right">Size</TableHead>
                            <TableHead className="text-right">Actions</TableHead>
                          </TableRow>
                        </TableHeader>
                        <TableBody>
                          {modAnalysis.mods.slice(0, 50).map((mod, i) => {
                            const isDisabled = mod.file_name.endsWith(".disabled");
                            return (
                              <TableRow key={i}>
                                <TableCell className="font-medium">{mod.display_name ?? mod.file_name}</TableCell>
                                <TableCell className="text-muted-foreground">{mod.version ?? "-"}</TableCell>
                                <TableCell>
                                  {isDisabled ? (
                                    <Badge variant="outline">disabled</Badge>
                                  ) : (
                                    <Badge variant="secondary" className="bg-emerald-500/10 text-emerald-600">active</Badge>
                                  )}
                                </TableCell>
                                <TableCell className="text-right">{(mod.size_bytes / 1024 / 1024).toFixed(1)} MB</TableCell>
                                <TableCell className="text-right">
                                  {isDisabled ? (
                                    <Button size="xs" variant="secondary" onClick={() => toggleMod(mod.file_name, true)}>
                                      Enable
                                    </Button>
                                  ) : (
                                    <Button size="xs" variant="destructive" onClick={() => toggleMod(mod.file_name, false)}>
                                      Disable
                                    </Button>
                                  )}
                                </TableCell>
                              </TableRow>
                            );
                          })}
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

              {/* Mod Store */}
              {selectedInstance?.mods_path && (
                <Card>
                  <CardHeader>
                    <CardTitle>Mod Store</CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div className="flex gap-2">
                      <input
                        type="text"
                        value={modSearchQuery}
                        onChange={(e) => setModSearchQuery(e.target.value)}
                        onKeyDown={(e) => e.key === "Enter" && searchMods()}
                        placeholder="Search Modrinth..."
                        className="flex-1 h-8 rounded-lg border border-border bg-background px-3 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
                      />
                      <Button onClick={searchMods} disabled={modSearching || !modSearchQuery.trim()}>
                        {modSearching ? "Searching..." : "Search"}
                      </Button>
                    </div>

                    {modSearchResults.length > 0 && (
                      <div className="grid grid-cols-1 lg:grid-cols-2 gap-3">
                        {modSearchResults.map((mod) => (
                          <Card key={mod.project_id} size="sm">
                            <CardContent className="flex gap-3 pt-3">
                              {mod.icon_url ? (
                                <img
                                  src={mod.icon_url}
                                  alt=""
                                  className="h-10 w-10 rounded-md object-cover shrink-0"
                                />
                              ) : (
                                <div className="h-10 w-10 rounded-md bg-muted shrink-0" />
                              )}
                              <div className="min-w-0 flex-1 space-y-1">
                                <div className="flex items-center justify-between gap-2">
                                  <span className="text-sm font-medium truncate">{mod.title}</span>
                                  <span className="text-xs text-muted-foreground shrink-0">
                                    {formatDownloads(mod.downloads)} downloads
                                  </span>
                                </div>
                                <p className="text-xs text-muted-foreground">by {mod.author}</p>
                                <p className="text-xs text-muted-foreground line-clamp-2">{mod.description}</p>
                                <div className="flex items-center justify-between pt-1">
                                  <div className="flex flex-wrap gap-1">
                                    {mod.categories.slice(0, 3).map((cat) => (
                                      <Badge key={cat} variant="outline" className="text-[10px]">{cat}</Badge>
                                    ))}
                                  </div>
                                  <Button size="xs" onClick={() => prepareInstall(mod)}>
                                    Install
                                  </Button>
                                </div>
                              </div>
                            </CardContent>
                          </Card>
                        ))}
                      </div>
                    )}
                  </CardContent>
                </Card>
              )}

              {/* Install Confirmation Dialog */}
              <Dialog open={installDialogOpen} onOpenChange={setInstallDialogOpen}>
                <DialogContent className="sm:max-w-md">
                  <DialogHeader>
                    <DialogTitle>Confirm Install</DialogTitle>
                    <DialogDescription>
                      Review the file before installing.
                    </DialogDescription>
                  </DialogHeader>
                  {pendingInstall && (
                    <div className="space-y-3">
                      <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-2 text-sm">
                        <dt className="text-muted-foreground">Mod</dt>
                        <dd className="font-medium">{pendingInstall.mod.title}</dd>
                        <dt className="text-muted-foreground">File</dt>
                        <dd className="font-mono text-xs truncate">{pendingInstall.file.filename}</dd>
                        <dt className="text-muted-foreground">Size</dt>
                        <dd>{(pendingInstall.file.size / 1024 / 1024).toFixed(2)} MB</dd>
                        <dt className="text-muted-foreground">Source</dt>
                        <dd className="text-xs truncate">Modrinth</dd>
                      </dl>
                    </div>
                  )}
                  <DialogFooter>
                    <Button variant="secondary" onClick={() => setInstallDialogOpen(false)} disabled={installing}>
                      Cancel
                    </Button>
                    <Button onClick={confirmInstall} disabled={installing}>
                      {installing ? "Installing..." : "Install"}
                    </Button>
                  </DialogFooter>
                </DialogContent>
              </Dialog>

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
                        <div className="pt-1">
                          <Button size="sm" onClick={() => applyConfig(cr)}>
                            Apply
                          </Button>
                        </div>
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
                    {recommendations.map((r) => {
                      const status = recStatusMap[r.id];
                      return (
                        <div
                          key={r.id}
                          className={`rounded-lg border border-border p-4 space-y-2 transition-opacity ${status ? "opacity-60" : ""}`}
                        >
                          <div className="flex flex-wrap gap-1.5">
                            <Badge variant={r.severity === "error" ? "destructive" : "secondary"}>{r.severity}</Badge>
                            <Badge variant="outline">{r.confidence}</Badge>
                            <Badge variant="outline">risk: {r.risk_level}</Badge>
                            {status && (
                              <Badge variant="default">{status.replace("_", " ")}</Badge>
                            )}
                          </div>
                          <h4 className="font-medium text-sm">{r.title}</h4>
                          <p className="text-sm text-muted-foreground">{r.description}</p>
                          <p className="text-xs text-muted-foreground">{r.evidence}</p>
                          {!status && (
                            <div className="flex gap-2 pt-1">
                              <Button size="sm" onClick={() => updateRecStatus(r.id, "accepted")}>Accept</Button>
                              <Button size="sm" variant="secondary" onClick={() => updateRecStatus(r.id, "ignored_once")}>Ignore</Button>
                              <Button size="sm" variant="secondary" onClick={() => updateRecStatus(r.id, "deferred")}>Defer</Button>
                              {r.action_type === "set_jvm_arg" && r.action_data && (
                                <Button size="sm" variant="secondary" onClick={() => applyJvmSettings(r)}>
                                  Apply JVM Settings
                                </Button>
                              )}
                              {r.action_type === "open_link" && r.action_data && (
                                <a href={r.action_data} target="_blank" rel="noreferrer">
                                  <Button size="sm" variant="secondary">Open Link</Button>
                                </a>
                              )}
                            </div>
                          )}
                        </div>
                      );
                    })}
                  </CardContent>
                </Card>
              )}

              {/* Applied Changes (Rollback) */}
              {appliedChanges.length > 0 && (
                <Card>
                  <CardHeader>
                    <CardTitle>Applied Changes</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <Table>
                      <TableHeader>
                        <TableRow>
                          <TableHead>File</TableHead>
                          <TableHead>Applied</TableHead>
                          <TableHead className="text-right">Action</TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {appliedChanges.map((rp) => (
                          <TableRow key={rp.id}>
                            <TableCell className="font-mono text-xs">{rp.file_path}</TableCell>
                            <TableCell className="text-xs text-muted-foreground">
                              {new Date(rp.created_at).toLocaleString(undefined, {
                                month: "short",
                                day: "numeric",
                                hour: "2-digit",
                                minute: "2-digit",
                              })}
                            </TableCell>
                            <TableCell className="text-right">
                              <Button size="sm" variant="destructive" onClick={() => rollbackChange(rp)}>
                                Rollback
                              </Button>
                            </TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  </CardContent>
                </Card>
              )}

              {/* Share Dialog */}
              <Dialog open={shareDialogOpen} onOpenChange={setShareDialogOpen}>
                <DialogContent className="sm:max-w-xl max-h-[80vh] overflow-y-auto">
                  <DialogHeader>
                    <DialogTitle>Share Optimization Profile</DialogTitle>
                    <DialogDescription>
                      Export or import optimization profiles for {selectedInstance?.name}.
                    </DialogDescription>
                  </DialogHeader>
                  <Tabs defaultValue="export">
                    <TabsList>
                      <TabsTrigger value="export">Export</TabsTrigger>
                      <TabsTrigger value="import">Import</TabsTrigger>
                    </TabsList>
                    <TabsContent value="export" className="mt-4 space-y-4">
                      {shareLoading ? (
                        <p className="text-sm text-muted-foreground">Generating profile...</p>
                      ) : shareProfileJson ? (
                        <>
                          <pre className="max-h-48 overflow-auto rounded-lg border border-border bg-muted/50 p-3 text-xs font-mono">
                            {shareProfileJson}
                          </pre>
                          <div className="flex gap-2">
                            <Button size="sm" onClick={copyProfileToClipboard}>
                              Copy to Clipboard
                            </Button>
                            <Button size="sm" variant="secondary" onClick={saveProfileToFile}>
                              Save as File
                            </Button>
                          </div>
                          <div className="space-y-2 pt-2 border-t border-border">
                            <h4 className="text-sm font-medium">Share to Discord</h4>
                            <div className="flex gap-2">
                              <input
                                type="text"
                                value={discordWebhookUrl}
                                onChange={(e) => setDiscordWebhookUrl(e.target.value)}
                                placeholder="Discord webhook URL"
                                className="flex-1 h-8 rounded-lg border border-border bg-background px-3 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
                              />
                              <Button
                                size="sm"
                                onClick={handleShareToDiscord}
                                disabled={discordSending || !discordWebhookUrl.trim()}
                              >
                                {discordSending ? "Sending..." : "Send"}
                              </Button>
                            </div>
                          </div>
                        </>
                      ) : (
                        <p className="text-sm text-muted-foreground">
                          No profile data available.
                        </p>
                      )}
                    </TabsContent>
                    <TabsContent value="import" className="mt-4 space-y-4">
                      <div className="space-y-2">
                        <textarea
                          value={importJson}
                          onChange={(e) => setImportJson(e.target.value)}
                          placeholder="Paste a .gamepilot JSON profile here..."
                          className="w-full h-32 rounded-lg border border-border bg-background px-3 py-2 text-xs font-mono placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring resize-none"
                        />
                        <div className="flex gap-2">
                          <Button
                            size="sm"
                            onClick={handleImportFromText}
                            disabled={!importJson.trim()}
                          >
                            Parse
                          </Button>
                          <Button size="sm" variant="secondary" onClick={handleImportFromFile}>
                            Browse for File
                          </Button>
                        </div>
                      </div>
                      {importedProfile && (
                        <div className="rounded-lg border border-border p-3 space-y-2">
                          <h4 className="text-sm font-medium">Imported Profile</h4>
                          <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-sm">
                            <dt className="text-muted-foreground">Instance</dt>
                            <dd>{importedProfile.instance_name}</dd>
                            <dt className="text-muted-foreground">Version</dt>
                            <dd>{importedProfile.minecraft_version ?? "Unknown"}</dd>
                            <dt className="text-muted-foreground">Loader</dt>
                            <dd>{importedProfile.loader ?? "None"}</dd>
                            {importedProfile.health_score != null && (
                              <>
                                <dt className="text-muted-foreground">Health</dt>
                                <dd>{importedProfile.health_score}/100</dd>
                              </>
                            )}
                            {importedProfile.jvm_settings?.xmx_mb != null && (
                              <>
                                <dt className="text-muted-foreground">RAM</dt>
                                <dd>{importedProfile.jvm_settings.xmx_mb} MB</dd>
                              </>
                            )}
                          </dl>
                          {importedProfile.recommended_mods.length > 0 && (
                            <div className="space-y-1">
                              <span className="text-xs text-muted-foreground">
                                Recommended mods ({importedProfile.recommended_mods.length}):
                              </span>
                              <div className="flex flex-wrap gap-1">
                                {importedProfile.recommended_mods.map((m) => (
                                  <Badge key={m.name} variant="secondary">
                                    {m.name}
                                  </Badge>
                                ))}
                              </div>
                            </div>
                          )}
                        </div>
                      )}
                    </TabsContent>
                  </Tabs>
                </DialogContent>
              </Dialog>

              {/* Pre-launch confirmation dialog */}
              <Dialog open={preLaunchOpen} onOpenChange={setPreLaunchOpen}>
                <DialogContent className="sm:max-w-md">
                  <DialogHeader>
                    <DialogTitle>Review Before Launch</DialogTitle>
                    <DialogDescription>
                      There are unresolved items that may affect performance.
                    </DialogDescription>
                  </DialogHeader>
                  <div className="space-y-4">
                    {recommendations.filter((r) => !recStatusMap[r.id]).length > 0 && (
                      <div className="space-y-2">
                        <h4 className="text-sm font-medium">Unaddressed Recommendations</h4>
                        {recommendations
                          .filter((r) => !recStatusMap[r.id])
                          .slice(0, 3)
                          .map((r) => (
                            <div key={r.id} className="rounded-lg border border-border p-2 text-sm">
                              <div className="flex gap-1.5 mb-1">
                                <Badge variant={r.severity === "error" ? "destructive" : "secondary"}>{r.severity}</Badge>
                              </div>
                              <span>{r.title}</span>
                            </div>
                          ))}
                      </div>
                    )}
                    {resourceHogs.length > 0 && (
                      <div className="space-y-2">
                        <h4 className="text-sm font-medium">Resource Hogs Detected</h4>
                        {resourceHogs.slice(0, 3).map((p) => (
                          <div key={p.pid} className="flex items-center justify-between text-sm rounded-lg border border-border p-2">
                            <span className="font-mono">{p.name}</span>
                            <span className="text-xs text-muted-foreground">
                              {p.cpu_percent.toFixed(0)}% CPU / {p.ram_mb.toFixed(0)} MB
                            </span>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                  <DialogFooter>
                    <Button
                      variant="secondary"
                      onClick={() => {
                        setPreLaunchOpen(false);
                      }}
                    >
                      Review Recommendations
                    </Button>
                    <Button
                      onClick={() => {
                        setPreLaunchOpen(false);
                        launchInstance();
                      }}
                    >
                      Launch Anyway
                    </Button>
                  </DialogFooter>
                </DialogContent>
              </Dialog>
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
