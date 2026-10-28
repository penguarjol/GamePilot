import { useState, useEffect, useCallback } from "react";
import { Outlet } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { Sidebar } from "@/components/Sidebar";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import type { HardwareInfo, DiscoveredInstance } from "@/types";

function App() {
  const [onboarding, setOnboarding] = useState<number | null>(null);
  const [hwInfo, setHwInfo] = useState<HardwareInfo | null>(null);
  const [instances, setInstances] = useState<DiscoveredInstance[]>([]);
  const [scanning, setScanning] = useState(false);
  const [selectedPath, setSelectedPath] = useState<string | null>(null);

  useEffect(() => {
    async function init() {
      try {
        const theme = await invoke<string | null>("get_preference", { key: "theme" });
        if (theme === "light") {
          document.documentElement.classList.remove("dark");
        }
      } catch { /* non-fatal */ }

      try {
        const done = await invoke<string | null>("get_preference", { key: "onboarding_complete" });
        if (done === null) setOnboarding(0);
      } catch { /* non-fatal */ }
    }
    init();
  }, []);

  const runScan = useCallback(async () => {
    setScanning(true);
    try {
      // Hardware info first (fast)
      const hw = await invoke<HardwareInfo>("get_hardware_info");
      setHwInfo(hw);

      // Instance discovery can be slow — run with a timeout so the wizard doesn't freeze
      const discoverWithTimeout = Promise.race([
        invoke<DiscoveredInstance[]>("discover_all_instances"),
        new Promise<DiscoveredInstance[]>((resolve) => setTimeout(() => resolve([]), 10000)),
      ]);
      const disc = await discoverWithTimeout;
      setInstances(disc);
    } catch {
      // Scan errors are non-fatal during onboarding
    } finally {
      setScanning(false);
    }
  }, []);

  useEffect(() => {
    if (onboarding === 1) runScan();
  }, [onboarding, runScan]);

  const finishOnboarding = async () => {
    if (selectedPath) {
      const inst = instances.find((i) => i.path === selectedPath);
      if (inst) {
        try {
          const scanned = await invoke("scan_instance", {
            path: inst.path,
            launcher: inst.launcher,
          });
          await invoke("save_instance", { instanceJson: JSON.stringify(scanned) });
        } catch { /* non-fatal */ }
      }
    }
    await invoke("set_preference", { key: "onboarding_complete", value: "true" }).catch(() => {});
    setOnboarding(null);
  };

  const skipOnboarding = async () => {
    await invoke("set_preference", { key: "onboarding_complete", value: "true" }).catch(() => {});
    setOnboarding(null);
  };

  return (
    <>
      {onboarding !== null && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm">
          <Card className="w-full max-w-lg">
            <CardHeader>
              <CardTitle className="text-lg">
                {onboarding === 0 && "Welcome to GamePilot"}
                {onboarding === 1 && "Scanning Your System"}
                {onboarding === 2 && "Select an Instance"}
                {onboarding === 3 && "Setup Complete"}
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              {onboarding === 0 && (
                <>
                  <p className="text-muted-foreground">
                    GamePilot optimizes your games, auto-clips your best moments, and posts them to social. It detects your hardware, finds performance issues, and processes your recordings with local AI. All data stays on your machine.
                  </p>
                  <div className="flex gap-2 justify-end">
                    <Button variant="ghost" onClick={skipOnboarding}>Skip</Button>
                    <Button onClick={() => setOnboarding(1)}>Next</Button>
                  </div>
                </>
              )}

              {onboarding === 1 && (
                <>
                  {scanning ? (
                    <div className="space-y-3">
                      <p className="text-muted-foreground">Detecting your hardware and installed games...</p>
                      <Progress value={null} />
                    </div>
                  ) : (
                    <div className="space-y-3">
                      {hwInfo && (
                        <div className="rounded-lg bg-muted/50 p-3 text-sm space-y-1">
                          <div className="flex justify-between"><span className="text-muted-foreground">CPU</span><span>{hwInfo.cpu_model}</span></div>
                          <div className="flex justify-between"><span className="text-muted-foreground">RAM</span><span>{hwInfo.ram_total_mb} MB</span></div>
                          <div className="flex justify-between"><span className="text-muted-foreground">GPU</span><span>{hwInfo.gpu_model}</span></div>
                        </div>
                      )}
                      <p className="text-sm text-muted-foreground">
                        Found {instances.length} instance{instances.length !== 1 ? "s" : ""}
                      </p>
                    </div>
                  )}
                  <div className="flex gap-2 justify-end">
                    <Button variant="ghost" onClick={skipOnboarding}>Skip</Button>
                    <Button onClick={() => setOnboarding(2)} disabled={scanning}>Next</Button>
                  </div>
                </>
              )}

              {onboarding === 2 && (
                <>
                  {instances.length > 0 ? (
                    <div className="max-h-56 overflow-y-auto space-y-2">
                      {instances.map((inst) => (
                        <button
                          key={inst.path}
                          onClick={() => setSelectedPath(inst.path)}
                          className={`w-full text-left rounded-lg border p-3 transition-colors ${
                            selectedPath === inst.path
                              ? "border-primary bg-primary/10"
                              : "border-border hover:bg-muted/50"
                          }`}
                        >
                          <div className="font-medium text-sm">{inst.name}</div>
                          <div className="text-xs text-muted-foreground mt-0.5">
                            {inst.launcher} {inst.minecraft_version ? `/ ${inst.minecraft_version}` : ""} {inst.mod_count > 0 ? `/ ${inst.mod_count} mods` : ""}
                          </div>
                        </button>
                      ))}
                    </div>
                  ) : (
                    <p className="text-muted-foreground text-sm">
                      No instances detected. You can add one manually from the Minecraft tab.
                    </p>
                  )}
                  <div className="flex gap-2 justify-end">
                    <Button variant="ghost" onClick={skipOnboarding}>Skip</Button>
                    <Button onClick={() => setOnboarding(3)}>
                      {selectedPath ? "Next" : "Skip Selection"}
                    </Button>
                  </div>
                </>
              )}

              {onboarding === 3 && (
                <>
                  <p className="text-muted-foreground">
                    {selectedPath
                      ? "Your instance is ready to analyze. Check the Clips tab after your next session for auto-detected highlights."
                      : "Setup complete. GamePilot will detect your games automatically. Check the Clips tab after a gaming session."}
                  </p>
                  <div className="flex gap-2 justify-end">
                    <Button onClick={finishOnboarding}>Get Started</Button>
                  </div>
                </>
              )}
            </CardContent>
          </Card>
        </div>
      )}

      <div className="flex h-screen overflow-hidden bg-background">
        <Sidebar />
        <main className="flex-1 overflow-y-auto p-6">
          <Outlet />
        </main>
      </div>
    </>
  );
}

export default App;
