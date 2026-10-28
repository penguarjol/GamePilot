import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

function ClipProcessingStatus() {
  const [status, setStatus] = useState<"checking" | "active" | "unavailable">("checking");

  const check = useCallback(async () => {
    try {
      const s = await invoke<Record<string, unknown>>("get_sidecar_status");
      setStatus(s.running ? "active" : "unavailable");
    } catch {
      setStatus("unavailable");
    }
  }, []);

  useEffect(() => { check(); const i = setInterval(check, 15000); return () => clearInterval(i); }, [check]);

  return (
    <div className="flex items-center justify-between">
      <div>
        <p className="text-sm font-medium">Clip Processing Engine</p>
        <p className="text-xs text-muted-foreground">Detects highlights, scores clips, and renders exports</p>
      </div>
      <div className="flex items-center gap-2">
        <div className={`w-2 h-2 rounded-full ${status === "active" ? "bg-green-500" : status === "unavailable" ? "bg-red-500" : "bg-yellow-500"}`} />
        <span className="text-xs text-muted-foreground">
          {status === "active" ? "Active" : status === "unavailable" ? "Unavailable" : "Checking..."}
        </span>
        {status === "unavailable" && (
          <button className="text-xs text-primary hover:underline" onClick={async () => {
            try {
              setStatus("checking");
              await invoke("restart_sidecar_cmd");
              setTimeout(check, 2000);
            } catch { setStatus("unavailable"); }
          }}>Retry</button>
        )}
      </div>
    </div>
  );
}
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
          <CardTitle>Clip Processing</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <ClipProcessingStatus />

          <Separator />

          <div>
            <p className="text-sm font-medium">Ollama (AI Scoring)</p>
            <p className="text-xs text-muted-foreground mb-2">
              Provides written rationale for why clips scored the way they did.
              Install from ollama.com, then run: ollama pull qwen3:8b
            </p>
            <div className="flex gap-2">
              <input type="text" className="flex-1 text-sm bg-muted border border-border rounded px-3 py-1.5" defaultValue="http://localhost:11434"
                onBlur={async (e) => { try { await invoke("set_clipper_config", { key: "ollama_url", value: e.target.value }); } catch {} }} />
              <Button variant="outline" size="sm" onClick={async () => {
                try {
                  const result = await invoke<Record<string, unknown>>("test_ollama", { url: "http://localhost:11434" });
                  const models = ((result.models || []) as Array<Record<string, string>>).map(m => m.name).join(", ");
                  toast.success(`Ollama connected. Models: ${models || "none installed"}`);
                } catch (e) { toast.error(`Cannot reach Ollama: ${e}`); }
              }}>Test</Button>
            </div>
          </div>

          <Separator />

          <div>
            <p className="text-sm font-medium">Watch Folders</p>
            <p className="text-xs text-muted-foreground mb-2">Folders monitored for new recordings after each session</p>
            <Button variant="outline" size="sm" onClick={async () => {
              try {
                const folder = await (await import("@tauri-apps/plugin-dialog")).open({ directory: true });
                if (folder) { await invoke("set_clipper_config", { key: "watch_folders", value: JSON.stringify([folder]) }); toast.success(`Watch folder added: ${folder}`); }
              } catch (e) { toast.error(`Failed: ${e}`); }
            }}>Add Folder</Button>
          </div>

          <Separator />

          <div>
            <p className="text-sm font-medium">Social Posting (Postiz)</p>
            <p className="text-xs text-muted-foreground mb-2">Auto-post clips to TikTok, YouTube Shorts, Instagram, and more</p>
            <div className="space-y-2">
              <input type="text" className="w-full text-sm bg-muted border border-border rounded px-3 py-1.5" placeholder="Postiz instance URL"
                defaultValue="https://api.postiz.com" onBlur={async (e) => { try { await invoke("set_clipper_config", { key: "postiz_url", value: e.target.value }); toast.success("Saved"); } catch {} }} />
              <input type="password" className="w-full text-sm bg-muted border border-border rounded px-3 py-1.5" placeholder="Postiz API key"
                onBlur={async (e) => { if (!e.target.value) return; try { await invoke("set_clipper_config", { key: "postiz_api_key", value: e.target.value }); toast.success("Saved"); } catch {} }} />
            </div>
          </div>

          <Separator />

          <div>
            <p className="text-sm font-medium">API Keys (Optional)</p>
            <p className="text-xs text-muted-foreground mb-2">For trending sounds and GIF browser in the editor</p>
            <div className="space-y-2">
              <input type="text" className="w-full text-sm bg-muted border border-border rounded px-3 py-1.5" placeholder="GIPHY API key (free at developers.giphy.com)"
                onBlur={async (e) => { if (!e.target.value) return; try { await invoke("set_clipper_config", { key: "giphy_api_key", value: e.target.value }); toast.success("Saved"); } catch {} }} />
              <input type="text" className="w-full text-sm bg-muted border border-border rounded px-3 py-1.5" placeholder="ScrapeBadger API key (for TikTok trending sounds)"
                onBlur={async (e) => { if (!e.target.value) return; try { await invoke("set_clipper_config", { key: "scrapebadger_api_key", value: e.target.value }); toast.success("Saved"); } catch {} }} />
            </div>
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
