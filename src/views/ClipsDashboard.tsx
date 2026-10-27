import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { toast } from "sonner";
import type { Clip } from "@/types";

const MOOD_KEYS = ["skill", "funny", "montage", "viral"] as const;
type MoodKey = (typeof MOOD_KEYS)[number];

const GAMES = ["All", "Valorant", "CS2", "Fortnite", "Apex", "League", "Overwatch", "Minecraft"];

function scoreTone(score: number): string {
  if (score >= 90) return "text-green-400";
  if (score >= 75) return "text-blue-400";
  if (score >= 60) return "text-yellow-400";
  return "text-red-400";
}

function scoreLabel(score: number): string {
  if (score >= 90) return "Viral";
  if (score >= 75) return "Strong";
  if (score >= 60) return "Decent";
  return "Weak";
}

function statusBadge(status: string) {
  const variant =
    status === "approved" ? "default" :
    status === "rejected" ? "destructive" :
    status === "posted" ? "secondary" :
    "outline";
  return <Badge variant={variant}>{status}</Badge>;
}

function ConfidenceBar({ value, threshold = 0.8 }: { value: number; threshold?: number }) {
  const pct = value * 100;
  const threshPct = threshold * 100;
  return (
    <div className="relative h-1.5 rounded-full bg-muted overflow-hidden">
      <div
        className={`absolute left-0 top-0 h-full rounded-full ${pct >= threshPct ? "bg-green-500" : "bg-yellow-500"}`}
        style={{ width: `${pct}%` }}
      />
      <div
        className="absolute top-0 h-full w-0.5 bg-muted-foreground/50"
        style={{ left: `${threshPct}%` }}
      />
    </div>
  );
}

export function ClipsDashboard() {
  const navigate = useNavigate();
  const [clips, setClips] = useState<Clip[]>([]);
  const [gameFilter, setGameFilter] = useState("All");
  const [statusFilter, setStatusFilter] = useState("all");
  const [moodSort, setMoodSort] = useState<MoodKey>("viral");

  const loadClips = useCallback(async () => {
    try {
      const result = await invoke<Clip[]>("list_clips", {
        gameId: gameFilter === "All" ? null : gameFilter.toLowerCase(),
        status: statusFilter === "all" ? null : statusFilter,
        limit: 50,
      });
      setClips(result);
    } catch {
      setClips([]);
    }
  }, [gameFilter, statusFilter]);

  useEffect(() => { loadClips(); }, [loadClips]);

  const sortedClips = [...clips].sort((a, b) => {
    const aScore = a.mood_scores?.[moodSort] ?? a.composite_score ?? 0;
    const bScore = b.mood_scores?.[moodSort] ?? b.composite_score ?? 0;
    return bScore - aScore;
  });

  const approved = clips.filter(c => c.status === "approved").length;
  const pending = clips.filter(c => c.status === "detected" || c.status === "scored").length;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Your Clips</h2>
          <p className="text-muted-foreground text-sm">
            {clips.length} clips found. {pending} waiting for review.
          </p>
        </div>
        <Button onClick={async () => {
          try {
            const file = await open({
              multiple: false,
              filters: [{ name: "Video", extensions: ["mp4", "mkv", "mov", "avi", "webm"] }],
            });
            if (!file) return;
            toast.info(`Processing: ${typeof file === "string" ? file.split(/[/\\]/).pop() : file}`);
            const result = await invoke("process_video_file", { videoPath: file, gameId: null });
            toast.success(`Done! Found ${(result as Record<string, unknown>).candidates ?? 0} clip candidates.`);
            loadClips();
          } catch (e) {
            toast.error(`Processing failed: ${e}`);
          }
        }}>Process New Video</Button>
      </div>

      <div className="grid grid-cols-4 gap-4">
        <Card>
          <CardContent className="pt-4">
            <div className="text-2xl font-bold">{clips.length}</div>
            <div className="text-xs text-muted-foreground">Total Clips</div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-4">
            <div className="text-2xl font-bold text-green-400">{approved}</div>
            <div className="text-xs text-muted-foreground">Approved</div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-4">
            <div className="text-2xl font-bold text-yellow-400">{pending}</div>
            <div className="text-xs text-muted-foreground">Pending Review</div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-4">
            <div className="text-2xl font-bold text-blue-400">0</div>
            <div className="text-xs text-muted-foreground">Scheduled Posts</div>
          </CardContent>
        </Card>
      </div>

      <div className="flex items-center gap-3 flex-wrap">
        <span className="text-xs font-medium text-muted-foreground">Sort by:</span>
        <Tabs value={moodSort} onValueChange={(v) => setMoodSort(v as MoodKey)}>
          <TabsList>
            {MOOD_KEYS.map(m => (
              <TabsTrigger key={m} value={m} className="capitalize">{m}</TabsTrigger>
            ))}
          </TabsList>
        </Tabs>
        <div className="ml-auto flex gap-2">
          {GAMES.map(g => (
            <Button
              key={g}
              variant={gameFilter === g ? "default" : "outline"}
              size="sm"
              onClick={() => setGameFilter(g)}
            >
              {g}
            </Button>
          ))}
        </div>
      </div>

      <div className="flex gap-2">
        {["all", "detected", "scored", "approved", "rejected", "posted"].map(s => (
          <Button
            key={s}
            variant={statusFilter === s ? "default" : "ghost"}
            size="sm"
            onClick={() => setStatusFilter(s)}
            className="capitalize"
          >
            {s}
          </Button>
        ))}
      </div>

      <div className="space-y-3">
        {sortedClips.length === 0 && (
          <Card>
            <CardContent className="py-12 text-center text-muted-foreground">
              No clips yet. Process a recording or wait for your next session to end.
            </CardContent>
          </Card>
        )}
        {sortedClips.map(clip => (
          <Card key={clip.id} className="cursor-pointer hover:bg-muted/50 transition-colors" onClick={() => navigate(`/editor?clipId=${clip.id}`)}>
            <CardContent className="py-4">
              <div className="flex items-start gap-4">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-1">
                    <Badge variant="outline" className="text-xs">{clip.game_id ?? "Unknown"}</Badge>
                    <span className="font-medium">{clip.title ?? "Untitled Clip"}</span>
                    <span className="text-xs text-muted-foreground ml-auto">
                      {clip.duration_ms ? `${(clip.duration_ms / 1000).toFixed(0)}s` : ""}
                    </span>
                    {statusBadge(clip.status)}
                  </div>
                  {clip.transcript && (
                    <p className="text-sm text-muted-foreground truncate">{clip.transcript}</p>
                  )}
                  <div className="mt-2 w-48">
                    <ConfidenceBar value={clip.decima_confidence ?? 0} />
                  </div>
                </div>
                <div className="grid grid-cols-2 gap-x-4 gap-y-1 text-right shrink-0">
                  {clip.mood_scores && MOOD_KEYS.map(m => (
                    <div key={m} className="text-xs">
                      <span className="text-muted-foreground capitalize">{m}</span>{" "}
                      <span className={`font-semibold ${scoreTone(clip.mood_scores![m])}`}>
                        {clip.mood_scores![m]}
                      </span>
                    </div>
                  ))}
                  {!clip.mood_scores && clip.composite_score != null && (
                    <div>
                      <span className={`text-lg font-bold ${scoreTone(clip.composite_score)}`}>
                        {clip.composite_score}
                      </span>
                      <div className="text-xs text-muted-foreground">
                        {scoreLabel(clip.composite_score)}
                      </div>
                    </div>
                  )}
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
