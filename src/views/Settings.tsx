import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import { toast } from "sonner";
import { useInvoke } from "@/hooks/useInvoke";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { Separator } from "@/components/ui/separator";
import { Badge } from "@/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  DialogFooter,
  DialogClose,
} from "@/components/ui/dialog";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type { IgnoreRule, OptimizationAction } from "@/types";

export function Settings() {
  const [theme, setTheme] = useState<"dark" | "light">("dark");
  const [appVersion, setAppVersion] = useState<string>("...");
  const [exporting, setExporting] = useState(false);
  const [discordWebhookUrl, setDiscordWebhookUrl] = useState("");
  const [discordSaving, setDiscordSaving] = useState(false);
  const [discordTesting, setDiscordTesting] = useState(false);
  const [privacyProcessScanning, setPrivacyProcessScanning] = useState(true);
  const [privacySessionTracking, setPrivacySessionTracking] = useState(true);
  const [privacyModAnalysis, setPrivacyModAnalysis] = useState(true);
  const [privacyScreenCapture, setPrivacyScreenCapture] = useState(false);
  const ignoreRules = useInvoke<IgnoreRule[]>("get_ignore_rules");
  const history = useInvoke<OptimizationAction[]>("get_optimization_history");

  useEffect(() => {
    ignoreRules.execute();
    history.execute();
    initTheme();
    initDiscordWebhook();
    initPrivacyPrefs();
    getVersion().then(setAppVersion).catch(() => setAppVersion("unknown"));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const initTheme = async () => {
    try {
      const savedTheme = await invoke<string | null>("get_preference", { key: "theme" });
      if (savedTheme === "light" || savedTheme === "dark") {
        setTheme(savedTheme);
      }
    } catch { /* preferences may not be initialized yet */ }
  };

  const toggleTheme = async () => {
    const next = theme === "dark" ? "light" : "dark";
    setTheme(next);
    document.documentElement.classList.toggle("dark", next === "dark");
    await invoke("set_preference", { key: "theme", value: next });
  };

  const initDiscordWebhook = async () => {
    try {
      const saved = await invoke<string | null>("get_preference", { key: "discord_webhook_url" });
      if (saved) setDiscordWebhookUrl(saved);
    } catch { /* preference may not exist yet */ }
  };

  const initPrivacyPrefs = async () => {
    try {
      const ps = await invoke<string | null>("get_preference", { key: "privacy_process_scanning" });
      if (ps !== null) setPrivacyProcessScanning(ps !== "false");
      const st = await invoke<string | null>("get_preference", { key: "privacy_session_tracking" });
      if (st !== null) setPrivacySessionTracking(st !== "false");
      const ma = await invoke<string | null>("get_preference", { key: "privacy_mod_analysis" });
      if (ma !== null) setPrivacyModAnalysis(ma !== "false");
      const sc = await invoke<string | null>("get_preference", { key: "privacy_screen_capture" });
      if (sc !== null) setPrivacyScreenCapture(sc === "true");
    } catch { /* preferences may not be initialized yet */ }
  };

  const setPrivacyPref = async (key: string, value: boolean, setter: (v: boolean) => void) => {
    setter(value);
    try {
      await invoke("set_preference", { key, value: String(value) });
    } catch (err) {
      toast.error(`Failed to save preference: ${err}`);
      setter(!value);
    }
  };

  const saveDiscordWebhook = async () => {
    setDiscordSaving(true);
    try {
      await invoke("set_preference", { key: "discord_webhook_url", value: discordWebhookUrl });
      toast.success("Discord webhook URL saved");
    } catch (err) {
      toast.error(String(err));
    } finally {
      setDiscordSaving(false);
    }
  };

  const testDiscordWebhook = async () => {
    if (!discordWebhookUrl.trim()) return;
    setDiscordTesting(true);
    try {
      await invoke("test_discord_webhook", { webhookUrl: discordWebhookUrl.trim() });
      toast.success("Test message sent to Discord");
    } catch (err) {
      toast.error(`Test failed: ${err}`);
    } finally {
      setDiscordTesting(false);
    }
  };

  const removeRule = async (ruleId: string) => {
    try {
      await invoke("remove_ignore_rule", { ruleId });
      await ignoreRules.execute();
      toast.success("Rule removed");
    } catch (err) {
      toast.error(String(err));
    }
  };

  const deleteAllData = async () => {
    try {
      await invoke("delete_all_data");
      await history.execute();
      toast.success("All data deleted");
    } catch (err) {
      toast.error(String(err));
    }
  };

  const exportData = async () => {
    setExporting(true);
    try {
      const data = await invoke<Record<string, unknown>>("export_user_data");
      const filePath = await save({
        defaultPath: `gamepilot-export-${new Date().toISOString().slice(0, 10)}.json`,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (filePath) {
        await writeTextFile(filePath, JSON.stringify(data, null, 2));
        toast.success("Data exported");
      }
    } catch (err) {
      toast.error(String(err));
    } finally {
      setExporting(false);
    }
  };

  const formatDate = (iso: string) => {
    try {
      return new Date(iso).toLocaleString(undefined, {
        dateStyle: "medium",
        timeStyle: "short",
      });
    } catch {
      return iso;
    }
  };

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold tracking-tight">Settings</h1>

      {/* Clipper */}
      <Card>
        <CardHeader>
          <CardTitle>Clipper</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium">Sidecar Status</p>
              <p className="text-xs text-muted-foreground">
                The ML processing engine that detects highlights
              </p>
            </div>
            <Button variant="outline" size="sm" onClick={async () => {
              try {
                const status = await invoke<Record<string, unknown>>("get_sidecar_status");
                if (status.running) {
                  toast.success(`Sidecar running on port ${status.port} (PID ${status.pid})`);
                } else {
                  toast.error("Sidecar is not running. Check the log file for errors.");
                }
              } catch (e) {
                toast.error(`Could not check sidecar: ${e}`);
              }
            }}>Check Status</Button>
          </div>

          <Separator />

          <div>
            <p className="text-sm font-medium">Ollama URL</p>
            <p className="text-xs text-muted-foreground mb-2">
              Local Ollama instance for AI scoring rationale (Qwen3-8B)
            </p>
            <div className="flex gap-2">
              <input
                type="text"
                className="flex-1 text-sm bg-muted border border-border rounded px-3 py-1.5"
                defaultValue="http://localhost:11434"
                onBlur={async (e) => {
                  try {
                    await invoke("set_clipper_config", { key: "ollama_url", value: e.target.value });
                    toast.success("Ollama URL saved");
                  } catch (err) {
                    toast.error(`Failed to save: ${err}`);
                  }
                }}
              />
              <Button variant="outline" size="sm" onClick={async () => {
                try {
                  const resp = await fetch("http://localhost:11434/api/tags");
                  if (resp.ok) {
                    const data = await resp.json();
                    const models = (data.models || []).map((m: Record<string, string>) => m.name).join(", ");
                    toast.success(`Ollama connected. Models: ${models || "none"}`);
                  } else {
                    toast.error("Ollama responded but returned an error");
                  }
                } catch {
                  toast.error("Cannot reach Ollama at localhost:11434. Is it running?");
                }
              }}>Test Connection</Button>
            </div>
            <p className="text-xs text-muted-foreground mt-1.5">
              Install Ollama from ollama.com, then run: ollama pull qwen3:8b
            </p>
          </div>

          <Separator />

          <div>
            <p className="text-sm font-medium">Watch Folders</p>
            <p className="text-xs text-muted-foreground mb-2">
              Folders monitored for new recordings after each session
            </p>
            <div className="space-y-1.5 text-sm">
              <div className="flex items-center justify-between bg-muted/50 rounded px-3 py-1.5">
                <span className="text-muted-foreground">C:\Medal\</span>
                <Badge variant="outline" className="text-xs">Medal.tv</Badge>
              </div>
            </div>
            <Button variant="outline" size="sm" className="mt-2" onClick={async () => {
              try {
                const folder = await (await import("@tauri-apps/plugin-dialog")).open({ directory: true });
                if (folder) {
                  toast.success(`Watch folder added: ${folder}`);
                  await invoke("set_clipper_config", { key: "watch_folders", value: JSON.stringify([folder]) });
                }
              } catch (e) {
                toast.error(`Failed: ${e}`);
              }
            }}>Add Folder</Button>
          </div>

          <Separator />

          <div>
            <p className="text-sm font-medium">Postiz URL</p>
            <p className="text-xs text-muted-foreground mb-2">
              Self-hosted social media scheduler for auto-posting clips
            </p>
            <div className="flex gap-2">
              <input
                type="text"
                className="flex-1 text-sm bg-muted border border-border rounded px-3 py-1.5"
                defaultValue="https://api.postiz.com"
                placeholder="https://your-postiz-instance.com"
                onBlur={async (e) => {
                  try {
                    await invoke("set_clipper_config", { key: "postiz_url", value: e.target.value });
                    toast.success("Postiz URL saved");
                  } catch (err) { toast.error(`Failed: ${err}`); }
                }}
              />
            </div>
          </div>

          <div>
            <p className="text-sm font-medium">Postiz API Key</p>
            <div className="flex gap-2 mt-1">
              <input
                type="password"
                className="flex-1 text-sm bg-muted border border-border rounded px-3 py-1.5"
                placeholder="Your Postiz API key"
                onBlur={async (e) => {
                  if (!e.target.value) return;
                  try {
                    await invoke("set_clipper_config", { key: "postiz_api_key", value: e.target.value });
                    toast.success("Postiz API key saved");
                  } catch (err) { toast.error(`Failed: ${err}`); }
                }}
              />
            </div>
          </div>

          <Separator />

          <div>
            <p className="text-sm font-medium">GIPHY API Key</p>
            <p className="text-xs text-muted-foreground mb-2">For GIF/sticker browser in the editor (free at developers.giphy.com)</p>
            <input
              type="text"
              className="w-full text-sm bg-muted border border-border rounded px-3 py-1.5"
              placeholder="Your GIPHY beta API key"
              onBlur={async (e) => {
                if (!e.target.value) return;
                try {
                  await invoke("set_clipper_config", { key: "giphy_api_key", value: e.target.value });
                  toast.success("GIPHY API key saved");
                } catch (err) { toast.error(`Failed: ${err}`); }
              }}
            />
          </div>

          <div>
            <p className="text-sm font-medium">ScrapeBadger API Key</p>
            <p className="text-xs text-muted-foreground mb-2">For TikTok trending sounds (scrapebadger.com)</p>
            <input
              type="text"
              className="w-full text-sm bg-muted border border-border rounded px-3 py-1.5"
              placeholder="Your ScrapeBadger API key"
              onBlur={async (e) => {
                if (!e.target.value) return;
                try {
                  await invoke("set_clipper_config", { key: "scrapebadger_api_key", value: e.target.value });
                  toast.success("ScrapeBadger API key saved");
                } catch (err) { toast.error(`Failed: ${err}`); }
              }}
            />
          </div>

          <Separator />

          <div>
            <p className="text-sm font-medium">ML Models</p>
            <p className="text-xs text-muted-foreground mb-2">Download models for detection and scoring</p>
            <Button variant="outline" size="sm" onClick={async () => {
              try {
                const result = await invoke<Record<string, unknown>>("get_model_status");
                const models = (result.models as Array<Record<string, string>>) || [];
                const summary = models.map(m => `${m.name}: ${m.status} (${m.size})`).join("\n");
                toast.success(`Models:\n${summary || "None loaded"}`);
              } catch (e) {
                toast.error(`Cannot check models (sidecar may not be running): ${e}`);
              }
            }}>Check Model Status</Button>
            <Button variant="outline" size="sm" className="ml-2" onClick={async () => {
              try {
                toast.info("Downloading Whisper large-v3 (this may take several minutes)...");
                await invoke("download_model", { modelName: "faster-whisper-large-v3" });
                toast.success("Whisper model downloaded");
              } catch (e) {
                toast.error(`Download failed: ${e}`);
              }
            }}>Download Whisper</Button>
          </div>

          <div>
            <p className="text-sm font-medium">Restart Sidecar</p>
            <p className="text-xs text-muted-foreground mb-2">Manually restart the ML processing engine</p>
            <Button variant="outline" size="sm" onClick={async () => {
              try {
                toast.info("Restarting sidecar...");
                await invoke("restart_sidecar_cmd");
                toast.success("Sidecar restarted");
              } catch (e) {
                toast.error(`Restart failed: ${e}`);
              }
            }}>Restart Sidecar</Button>
          </div>
        </CardContent>
      </Card>

      {/* Appearance */}
      <Card>
        <CardHeader>
          <CardTitle>Appearance</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium">Dark Mode</p>
              <p className="text-xs text-muted-foreground">Toggle between dark and light themes</p>
            </div>
            <Switch
              checked={theme === "dark"}
              onCheckedChange={toggleTheme}
            />
          </div>
        </CardContent>
      </Card>

      {/* Discord Integration */}
      <Card>
        <CardHeader>
          <CardTitle>Discord Integration</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <p className="text-sm font-medium">Webhook URL</p>
            <p className="text-xs text-muted-foreground">
              Paste a Discord webhook URL to share optimization profiles directly to a channel.
              Create one in Discord via Server Settings &gt; Integrations &gt; Webhooks.
            </p>
            <input
              type="text"
              value={discordWebhookUrl}
              onChange={(e) => setDiscordWebhookUrl(e.target.value)}
              placeholder="https://discord.com/api/webhooks/..."
              className="w-full h-8 rounded-lg border border-border bg-background px-3 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
            />
            <div className="flex gap-2">
              <Button
                variant="secondary"
                size="sm"
                onClick={saveDiscordWebhook}
                disabled={discordSaving}
              >
                {discordSaving ? "Saving..." : "Save"}
              </Button>
              <Button
                variant="secondary"
                size="sm"
                onClick={testDiscordWebhook}
                disabled={discordTesting || !discordWebhookUrl.trim()}
              >
                {discordTesting ? "Testing..." : "Test"}
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Privacy */}
      <Card>
        <CardHeader>
          <CardTitle>Privacy</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium">Process Scanning</p>
              <p className="text-xs text-muted-foreground">Scan running processes to detect games and resource hogs</p>
            </div>
            <Switch
              checked={privacyProcessScanning}
              onCheckedChange={(v) => setPrivacyPref("privacy_process_scanning", v, setPrivacyProcessScanning)}
            />
          </div>
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium">Session Tracking</p>
              <p className="text-xs text-muted-foreground">Record play sessions with telemetry data for performance insights</p>
            </div>
            <Switch
              checked={privacySessionTracking}
              onCheckedChange={(v) => setPrivacyPref("privacy_session_tracking", v, setPrivacySessionTracking)}
            />
          </div>
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium">Mod Analysis</p>
              <p className="text-xs text-muted-foreground">Analyze installed mods to generate performance recommendations</p>
            </div>
            <Switch
              checked={privacyModAnalysis}
              onCheckedChange={(v) => setPrivacyPref("privacy_mod_analysis", v, setPrivacyModAnalysis)}
            />
          </div>
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium">Screen Capture</p>
              <p className="text-xs text-muted-foreground">Enable vision-based analysis via periodic screen captures</p>
            </div>
            <Switch
              checked={privacyScreenCapture}
              onCheckedChange={(v) => setPrivacyPref("privacy_screen_capture", v, setPrivacyScreenCapture)}
            />
          </div>
        </CardContent>
      </Card>

      {/* Optimization History */}
      <Card>
        <CardHeader>
          <CardTitle>
            Optimization History
            {history.data && (
              <span className="text-muted-foreground font-normal ml-2">({history.data.length})</span>
            )}
          </CardTitle>
        </CardHeader>
        <CardContent>
          {history.data && history.data.length > 0 ? (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Date</TableHead>
                  <TableHead>Type</TableHead>
                  <TableHead>Description</TableHead>
                  <TableHead>File</TableHead>
                  <TableHead>Status</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {history.data.map((action) => (
                  <TableRow key={action.id}>
                    <TableCell className="whitespace-nowrap text-xs">
                      {formatDate(action.applied_at)}
                    </TableCell>
                    <TableCell className="text-xs">{action.action_type}</TableCell>
                    <TableCell className="text-xs max-w-[300px] truncate">
                      {action.description}
                    </TableCell>
                    <TableCell className="font-mono text-xs max-w-[200px] truncate">
                      {action.file_path
                        ? action.file_path.split("/").pop() ?? action.file_path
                        : "-"}
                    </TableCell>
                    <TableCell>
                      {action.status === "applied" ? (
                        <Badge variant="default" className="bg-green-600/20 text-green-400 text-xs">
                          Applied
                        </Badge>
                      ) : (
                        <Badge variant="secondary" className="text-xs">
                          Rolled back
                        </Badge>
                      )}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          ) : (
            <p className="text-sm text-muted-foreground">No optimization actions recorded yet</p>
          )}
        </CardContent>
      </Card>

      {/* Ignore Rules */}
      <Card>
        <CardHeader>
          <CardTitle>
            Ignore Rules
            {ignoreRules.data && (
              <span className="text-muted-foreground font-normal ml-2">({ignoreRules.data.length})</span>
            )}
          </CardTitle>
        </CardHeader>
        <CardContent>
          {ignoreRules.data && ignoreRules.data.length > 0 ? (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Type</TableHead>
                  <TableHead>Pattern</TableHead>
                  <TableHead>Reason</TableHead>
                  <TableHead>Created</TableHead>
                  <TableHead>Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {ignoreRules.data.map((rule) => (
                  <TableRow key={rule.id}>
                    <TableCell>{rule.rule_type}</TableCell>
                    <TableCell className="font-mono text-xs">{rule.pattern}</TableCell>
                    <TableCell>{rule.reason ?? "-"}</TableCell>
                    <TableCell>{new Date(rule.created_at).toLocaleDateString()}</TableCell>
                    <TableCell>
                      <Button
                        variant="destructive"
                        size="xs"
                        onClick={() => removeRule(rule.id)}
                      >
                        Remove
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          ) : (
            <p className="text-sm text-muted-foreground">No ignore rules configured</p>
          )}
        </CardContent>
      </Card>

      <Separator />

      {/* Data Management */}
      <Card>
        <CardHeader>
          <CardTitle>Data Management</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium">Export Data</p>
              <p className="text-xs text-muted-foreground">
                Download all your data as a JSON file
              </p>
            </div>
            <Button variant="secondary" onClick={exportData} disabled={exporting}>
              {exporting ? "Exporting..." : "Export Data"}
            </Button>
          </div>
          <Separator />
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium">Application Log</p>
              <p className="text-xs text-muted-foreground">
                Open the log file for debugging and error reporting
              </p>
            </div>
            <Button variant="outline" onClick={async () => {
              try {
                const path = await invoke<string>("get_log_path");
                await invoke("show_overlay"); // just to verify invoke works
                toast.success(`Log file: ${path}`);
                // Use the opener plugin to open the folder
                const dir = path.substring(0, path.lastIndexOf("\\") || path.lastIndexOf("/"));
                await invoke("open_path", { path: dir }).catch(() => {
                  // fallback: just show the path
                  navigator.clipboard.writeText(path);
                  toast.info("Log path copied to clipboard");
                });
              } catch {
                toast.error("Could not locate log file");
              }
            }}>
              Open Log Folder
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Danger Zone */}
      <Card className="border-destructive/30">
        <CardHeader>
          <CardTitle className="text-destructive">Danger Zone</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium">Delete All Data</p>
              <p className="text-xs text-muted-foreground">
                Permanently remove all sessions, recommendations, and preferences
              </p>
            </div>
            <Dialog>
              <DialogTrigger render={<Button variant="destructive" />}>
                Delete All Data
              </DialogTrigger>
              <DialogContent>
                <DialogHeader>
                  <DialogTitle>Confirm Deletion</DialogTitle>
                </DialogHeader>
                <p className="text-sm text-muted-foreground">
                  This will permanently delete all stored data including sessions, recommendations,
                  and preferences. This action cannot be undone.
                </p>
                <DialogFooter>
                  <DialogClose render={<Button variant="secondary" />}>Cancel</DialogClose>
                  <DialogClose render={<Button variant="destructive" onClick={deleteAllData} />}>
                    Delete Everything
                  </DialogClose>
                </DialogFooter>
              </DialogContent>
            </Dialog>
          </div>
        </CardContent>
      </Card>

      {/* About */}
      <Card>
        <CardHeader>
          <CardTitle>About</CardTitle>
        </CardHeader>
        <CardContent>
          <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-sm">
            <dt className="text-muted-foreground">Application</dt>
            <dd>GamePilot</dd>
            <dt className="text-muted-foreground">Version</dt>
            <dd>{appVersion}</dd>
            <dt className="text-muted-foreground">Runtime</dt>
            <dd>Tauri 2 + React 19</dd>
            <dt className="text-muted-foreground">Purpose</dt>
            <dd>
              Minecraft performance analysis and optimization. Scans instances,
              analyzes mods, provides JVM tuning recommendations, and tracks gaming sessions.
            </dd>
          </dl>
        </CardContent>
      </Card>
    </div>
  );
}
