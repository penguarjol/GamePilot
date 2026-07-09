import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { useInvoke } from "@/hooks/useInvoke";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { Separator } from "@/components/ui/separator";
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
import type { IgnoreRule } from "@/types";

export function Settings() {
  const [theme, setTheme] = useState<"dark" | "light">("dark");
  const ignoreRules = useInvoke<IgnoreRule[]>("get_ignore_rules");

  useEffect(() => {
    ignoreRules.execute();
    initTheme();
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
      toast.success("All data deleted");
    } catch (err) {
      toast.error(String(err));
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
            <dd>0.1.0</dd>
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
