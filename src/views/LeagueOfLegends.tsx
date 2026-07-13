import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
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
import type { HardwareInfo, ProcessInfo } from "@/types";

interface LiveGameData {
  game_mode: string;
  game_time: number;
  players: LivePlayer[];
  events: GameEvent[];
}

interface LivePlayer {
  summoner_name: string;
  champion: string;
  team: string;
  level: number;
  kills: number;
  deaths: number;
  assists: number;
  cs: number;
}

interface GameEvent {
  timestamp: number;
  event_type: string;
  description: string;
}

export function LeagueOfLegends() {
  const hardware = useInvoke<HardwareInfo>("get_hardware_info");
  const processes = useInvoke<ProcessInfo[]>("get_process_info");
  const [gameActive, setGameActive] = useState<boolean | null>(null);
  const [liveData, setLiveData] = useState<LiveGameData | null>(null);
  const [checking, setChecking] = useState(false);
  const [polling, setPolling] = useState(false);

  useEffect(() => {
    hardware.execute();
    processes.execute();
    checkGameStatus();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (!polling || !gameActive) return;
    const interval = setInterval(fetchLiveData, 5000);
    return () => clearInterval(interval);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [polling, gameActive]);

  const checkGameStatus = async () => {
    setChecking(true);
    try {
      const active = await invoke<boolean>("check_league_game_active");
      setGameActive(active);
      if (active) {
        await fetchLiveData();
        setPolling(true);
      } else {
        setLiveData(null);
        setPolling(false);
      }
    } catch (err) {
      toast.error(`Failed to check game status: ${err}`);
      setGameActive(false);
    } finally {
      setChecking(false);
    }
  };

  const fetchLiveData = async () => {
    try {
      const data = await invoke<LiveGameData>("get_league_live_data");
      setLiveData(data);
    } catch (err) {
      toast.error(`Failed to fetch live data: ${err}`);
      setPolling(false);
    }
  };

  const formatGameTime = (seconds: number) => {
    const m = Math.floor(seconds / 60);
    const s = seconds % 60;
    return `${m}:${s.toString().padStart(2, "0")}`;
  };

  const resourceHogs = processes.data?.filter((p) => p.is_resource_hog) ?? [];

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">
            League of Legends
          </h1>
          <p className="text-sm text-muted-foreground mt-1">
            Live game tracking and system performance
          </p>
        </div>
        <Button
          variant="outline"
          onClick={checkGameStatus}
          disabled={checking}
        >
          {checking ? "Checking..." : "Refresh Status"}
        </Button>
      </div>

      {/* Game Status */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-base">Game Status</CardTitle>
          <Badge variant={gameActive ? "default" : "secondary"}>
            {gameActive === null
              ? "Checking..."
              : gameActive
                ? "In Game"
                : "Not In Game"}
          </Badge>
        </CardHeader>
        <CardContent>
          {gameActive && liveData ? (
            <div className="space-y-1 text-sm">
              <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1">
                <dt className="text-muted-foreground">Mode</dt>
                <dd>{liveData.game_mode}</dd>
                <dt className="text-muted-foreground">Time</dt>
                <dd className="font-mono">
                  {formatGameTime(liveData.game_time)}
                </dd>
              </dl>
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">
              League is not currently in a game. Start a match and click
              Refresh Status to begin tracking.
            </p>
          )}
        </CardContent>
      </Card>

      {/* Live Player Data */}
      {gameActive && liveData && liveData.players.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Players</CardTitle>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Summoner</TableHead>
                  <TableHead>Champion</TableHead>
                  <TableHead>Team</TableHead>
                  <TableHead className="text-right">Lvl</TableHead>
                  <TableHead className="text-right">K</TableHead>
                  <TableHead className="text-right">D</TableHead>
                  <TableHead className="text-right">A</TableHead>
                  <TableHead className="text-right">CS</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {liveData.players.map((p) => (
                  <TableRow key={p.summoner_name}>
                    <TableCell className="font-medium">
                      {p.summoner_name}
                    </TableCell>
                    <TableCell>{p.champion}</TableCell>
                    <TableCell>
                      <Badge
                        variant="outline"
                        className={
                          p.team === "ORDER"
                            ? "border-blue-500 text-blue-400"
                            : "border-red-500 text-red-400"
                        }
                      >
                        {p.team}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-right">{p.level}</TableCell>
                    <TableCell className="text-right">{p.kills}</TableCell>
                    <TableCell className="text-right">{p.deaths}</TableCell>
                    <TableCell className="text-right">{p.assists}</TableCell>
                    <TableCell className="text-right">{p.cs}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      )}

      {/* Events */}
      {gameActive && liveData && liveData.events.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Recent Events</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2 max-h-64 overflow-y-auto">
              {liveData.events
                .slice()
                .reverse()
                .slice(0, 20)
                .map((e, i) => (
                  <div
                    key={i}
                    className="flex items-center gap-3 text-sm rounded-lg border border-border p-2"
                  >
                    <span className="font-mono text-xs text-muted-foreground shrink-0">
                      {formatGameTime(e.timestamp)}
                    </span>
                    <Badge variant="outline" className="shrink-0">
                      {e.event_type}
                    </Badge>
                    <span className="text-muted-foreground truncate">
                      {e.description}
                    </span>
                  </div>
                ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* System Performance */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">System Performance</CardTitle>
        </CardHeader>
        <CardContent>
          {hardware.data ? (
            <div className="grid grid-cols-2 gap-6 text-sm">
              <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1">
                <dt className="text-muted-foreground">CPU</dt>
                <dd>{hardware.data.cpu_model}</dd>
                <dt className="text-muted-foreground">Usage</dt>
                <dd>{hardware.data.cpu_usage_percent.toFixed(1)}%</dd>
                <dt className="text-muted-foreground">GPU</dt>
                <dd>{hardware.data.gpu_model}</dd>
              </dl>
              <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1">
                <dt className="text-muted-foreground">RAM</dt>
                <dd>
                  {(hardware.data.ram_used_mb / 1024).toFixed(1)} /{" "}
                  {(hardware.data.ram_total_mb / 1024).toFixed(1)} GB
                </dd>
                <dt className="text-muted-foreground">Available</dt>
                <dd>
                  {(hardware.data.ram_available_mb / 1024).toFixed(1)} GB
                </dd>
              </dl>
            </div>
          ) : hardware.loading ? (
            <p className="text-sm text-muted-foreground">
              Loading hardware info...
            </p>
          ) : (
            <p className="text-sm text-muted-foreground">
              Unable to load hardware info
            </p>
          )}

          {resourceHogs.length > 0 && (
            <div className="mt-4 space-y-2">
              <h4 className="text-sm font-medium">Resource Hogs</h4>
              {resourceHogs.slice(0, 5).map((p) => (
                <div
                  key={p.pid}
                  className="flex items-center justify-between text-sm rounded-lg border border-border p-2"
                >
                  <span className="font-mono">{p.name}</span>
                  <span className="text-xs text-muted-foreground">
                    {p.cpu_percent.toFixed(0)}% CPU / {p.ram_mb.toFixed(0)} MB
                  </span>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
