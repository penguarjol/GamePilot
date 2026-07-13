import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { useInvoke } from "@/hooks/useInvoke";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import type { GameInfo, SteamGameInstance } from "@/types";

function Skeleton({ className = "" }: { className?: string }) {
  return <div className={`animate-pulse rounded-md bg-muted ${className}`} />;
}

export function GameLibrary() {
  const navigate = useNavigate();
  const games = useInvoke<GameInfo[]>("discover_all_games");
  const steamGames = useInvoke<SteamGameInstance[]>("discover_steam_games");

  useEffect(() => {
    games.execute();
    steamGames.execute();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const totalCount =
    (games.data?.length ?? 0) + (steamGames.data?.length ?? 0);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Game Library</h1>
          {!games.loading && !steamGames.loading && (
            <p className="text-sm text-muted-foreground mt-1">
              {totalCount} game{totalCount !== 1 ? "s" : ""} detected
            </p>
          )}
        </div>
        <Button
          variant="outline"
          onClick={() => {
            games.execute();
            steamGames.execute();
          }}
          disabled={games.loading || steamGames.loading}
        >
          {games.loading || steamGames.loading ? "Scanning..." : "Refresh"}
        </Button>
      </div>

      <section className="space-y-3">
        <h2 className="text-lg font-semibold tracking-tight">Minecraft</h2>
        {games.loading ? (
          <Skeleton className="h-28 w-full" />
        ) : (
          <MinecraftCard
            game={games.data?.find((g) => g.name === "Minecraft") ?? null}
            onNavigate={() => navigate("/minecraft")}
          />
        )}
      </section>

      <section className="space-y-3">
        <h2 className="text-lg font-semibold tracking-tight">Steam</h2>
        {steamGames.loading ? (
          <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4 gap-4">
            {Array.from({ length: 4 }).map((_, i) => (
              <Skeleton key={i} className="h-28 w-full" />
            ))}
          </div>
        ) : steamGames.error ? (
          <Card>
            <CardContent className="py-8 text-center">
              <p className="text-sm text-muted-foreground">
                No Steam installation found
              </p>
            </CardContent>
          </Card>
        ) : steamGames.data && steamGames.data.length > 0 ? (
          <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4 gap-4">
            {steamGames.data.map((sg) => (
              <SteamGameCard key={sg.id} game={sg} />
            ))}
          </div>
        ) : (
          <Card>
            <CardContent className="py-8 text-center">
              <p className="text-sm text-muted-foreground">
                No Steam games discovered
              </p>
            </CardContent>
          </Card>
        )}
      </section>
    </div>
  );
}

function MinecraftCard({
  game,
  onNavigate,
}: {
  game: GameInfo | null;
  onNavigate: () => void;
}) {
  return (
    <Card className="hover:border-primary/50 transition-colors cursor-pointer" onClick={onNavigate}>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-base">Minecraft</CardTitle>
        <Badge variant={game?.installed ? "default" : "secondary"}>
          {game?.installed ? "Installed" : "Not Found"}
        </Badge>
      </CardHeader>
      <CardContent className="flex items-center justify-between">
        <div className="text-sm text-muted-foreground">
          {game?.install_path ?? "No installation path detected"}
        </div>
        <Button size="sm" variant="outline" onClick={(e) => { e.stopPropagation(); onNavigate(); }}>
          View Details
        </Button>
      </CardContent>
    </Card>
  );
}

function SteamGameCard({ game }: { game: SteamGameInstance }) {
  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-medium truncate">
          {game.name}
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-2">
        <p className="text-xs text-muted-foreground truncate" title={game.path}>
          {game.path}
        </p>
        <div className="flex items-center gap-2">
          {game.version && (
            <Badge variant="outline" className="text-xs">
              {game.version}
            </Badge>
          )}
          {game.last_played && (
            <span className="text-xs text-muted-foreground">
              Played {new Date(game.last_played).toLocaleDateString()}
            </span>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
