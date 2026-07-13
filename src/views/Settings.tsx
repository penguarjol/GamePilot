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
  const ignoreRules = useInvoke<IgnoreRule[]>("get_ignore_rules");
  const history = useInvoke<OptimizationAction[]>("get_optimization_history");

  useEffect(() => {
    ignoreRules.execute();
    history.execute();
    initTheme();
    initDiscordWebhook();
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
