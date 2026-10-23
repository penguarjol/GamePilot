import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Separator } from "@/components/ui/separator";
import type { Clip } from "@/types";

const MOOD_KEYS = ["skill", "funny", "montage", "viral"] as const;
type MoodKey = (typeof MOOD_KEYS)[number];

function scoreTone(score: number): string {
  if (score >= 90) return "text-green-400";
  if (score >= 75) return "text-blue-400";
  if (score >= 60) return "text-yellow-400";
  return "text-red-400";
}

function ConfidenceBar({ value, threshold = 0.8 }: { value: number; threshold?: number }) {
  const pct = value * 100;
  const threshPct = threshold * 100;
  return (
    <div className="space-y-1">
      <div className="relative h-3 rounded-full bg-muted overflow-hidden">
        <div
          className={`absolute left-0 top-0 h-full rounded-full ${pct >= threshPct ? "bg-green-500" : "bg-yellow-500"}`}
          style={{ width: `${pct}%` }}
        />
        <div
          className="absolute top-0 h-full w-0.5 bg-muted-foreground/60"
          style={{ left: `${threshPct}%` }}
        />
      </div>
      <div className="flex justify-between text-xs text-muted-foreground">
        <span>{pct.toFixed(0)}% confidence</span>
        <span>{pct >= threshPct ? "Auto-accepted by Decima" : "Escalated to Qwen3"}</span>
      </div>
    </div>
  );
}

function SignalBar({ label, value }: { label: string; value: number }) {
  return (
    <div className="flex items-center gap-2">
      <span className="text-xs text-muted-foreground w-16 text-right">{label}</span>
      <div className="flex-1 h-2 rounded-full bg-muted overflow-hidden">
        <div className="h-full bg-primary rounded-full" style={{ width: `${value * 100}%` }} />
      </div>
      <span className="text-xs w-8 text-right">{(value * 100).toFixed(0)}%</span>
    </div>
  );
}

interface RationaleData {
  what_happened: string;
  why_scored: string;
  mood_fit: Record<string, string>;
  edit_suggestion: string;
  trending_fit: string;
}

function parseRationale(raw: string | null): RationaleData | null {
  if (!raw) return null;
  try { return JSON.parse(raw); } catch { return null; }
}

export function ReviewQueue() {
  const [clips, setClips] = useState<Clip[]>([]);
  const [currentIdx, setCurrentIdx] = useState(0);
  const [mood, setMood] = useState<MoodKey>("viral");
  const [feedbackOpen, setFeedbackOpen] = useState(false);
  const [feedbackText, setFeedbackText] = useState("");

  const loadPending = useCallback(async () => {
    try {
      const result = await invoke<Clip[]>("list_clips", { status: "scored", limit: 50 });
      setClips(result);
    } catch {
      setClips([]);
    }
  }, []);

  useEffect(() => { loadPending(); }, [loadPending]);

  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (feedbackOpen) return;
      switch (e.key.toLowerCase()) {
        case "j": handleAction("reject"); break;
        case "l": handleAction("approve"); break;
        case "k": handleAction("skip"); break;
        case "f": handleAction("favorite"); break;
      }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [currentIdx, clips, feedbackOpen]);

  const clip = clips[currentIdx];

  const handleAction = async (action: "approve" | "reject" | "skip" | "favorite") => {
    if (!clip) return;

    if (action !== "skip") {
      const newStatus = action === "reject" ? "rejected" : "approved";
      try {
        await invoke("update_clip_status", { clipId: clip.id, newStatus });
        await invoke("submit_clip_feedback", {
          feedbackJson: JSON.stringify({
            id: crypto.randomUUID(),
            clip_id: clip.id,
            action,
            rating: null,
            rationale_agreed: feedbackOpen ? false : null,
            written_reason: feedbackText || null,
          }),
        });
      } catch (e) {
        console.error("Feedback failed:", e);
      }
    }

    setFeedbackOpen(false);
    setFeedbackText("");
    setCurrentIdx(i => Math.min(i + 1, clips.length - 1));
  };

  if (clips.length === 0) {
    return (
      <div className="space-y-6">
        <h2 className="text-2xl font-bold tracking-tight">Review</h2>
        <Card>
          <CardContent className="py-16 text-center">
            <p className="text-muted-foreground text-lg">Nothing to review. Go play.</p>
            <p className="text-muted-foreground text-sm mt-2">Clips appear here after a session is processed.</p>
          </CardContent>
        </Card>
      </div>
    );
  }

  if (!clip) {
    return (
      <div className="space-y-6">
        <h2 className="text-2xl font-bold tracking-tight">Review</h2>
        <Card>
          <CardContent className="py-16 text-center">
            <p className="text-lg">All clips reviewed.</p>
            <Button className="mt-4" onClick={() => setCurrentIdx(0)}>Start Over</Button>
          </CardContent>
        </Card>
      </div>
    );
  }

  const rationale = parseRationale(clip.rationale);
  const moodScore = clip.mood_scores?.[mood] ?? clip.composite_score ?? 0;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Review</h2>
          <p className="text-muted-foreground text-sm">{currentIdx + 1} of {clips.length} waiting</p>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-xs text-muted-foreground">Mood:</span>
          <Tabs value={mood} onValueChange={v => setMood(v as MoodKey)}>
            <TabsList>
              {MOOD_KEYS.map(m => (
                <TabsTrigger key={m} value={m} className="capitalize">{m}</TabsTrigger>
              ))}
            </TabsList>
          </Tabs>
        </div>
      </div>

      <div className="grid grid-cols-[1fr_320px] gap-4">
        <div className="space-y-4">
          <Card>
            <CardContent className="pt-4">
              <div className="aspect-video bg-muted rounded-lg flex items-center justify-center mb-4">
                <span className="text-muted-foreground text-sm">
                  {clip.game_id ?? "Unknown"} - {clip.title ?? "Untitled"} ({clip.duration_ms ? `${(clip.duration_ms / 1000).toFixed(0)}s` : ""})
                </span>
              </div>

              <div className="flex gap-2 justify-center">
                <Button variant="outline" onClick={() => handleAction("reject")}>Reject (J)</Button>
                <Button onClick={() => handleAction("approve")}>Approve (L)</Button>
                <Button variant="secondary" onClick={() => handleAction("skip")}>Skip (K)</Button>
                <Button variant="ghost" onClick={() => handleAction("favorite")}>Favorite (F)</Button>
              </div>
            </CardContent>
          </Card>
        </div>

        <div className="space-y-3">
          <div className="space-y-1">
            <div className="flex items-center gap-2">
              <Badge variant="outline">{clip.game_id ?? "Unknown"}</Badge>
              <span className="font-medium text-sm">{clip.title ?? "Untitled Clip"}</span>
            </div>
            {clip.transcript && (
              <p className="text-xs text-muted-foreground line-clamp-2">{clip.transcript}</p>
            )}
          </div>

          <Separator />

          <div className="grid grid-cols-2 gap-3">
            <div className="text-center">
              <span className={`text-2xl font-bold ${scoreTone(moodScore)}`}>{moodScore}</span>
              <div className="text-xs text-muted-foreground capitalize">{mood}</div>
            </div>
            <div className="text-center">
              <span className="text-2xl font-bold">{((clip.decima_confidence ?? 0) * 100).toFixed(0)}%</span>
              <div className="text-xs text-muted-foreground">Confidence</div>
            </div>
          </div>

          <ConfidenceBar value={clip.decima_confidence ?? 0} />

          {rationale && (
            <Card>
              <CardHeader className="py-2 px-3">
                <CardTitle className="text-xs">AI Rationale</CardTitle>
              </CardHeader>
              <CardContent className="px-3 pb-3 space-y-2 text-xs">
                <div>
                  <span className="font-medium">What happened:</span>{" "}
                  <span className="text-muted-foreground">{rationale.what_happened}</span>
                </div>
                <div>
                  <span className="font-medium">Why scored well:</span>{" "}
                  <span className="text-muted-foreground">{rationale.why_scored}</span>
                </div>
                {rationale.mood_fit && (
                  <div>
                    <span className="font-medium">Mood fit:</span>{" "}
                    <span className="text-muted-foreground">{rationale.mood_fit[mood] ?? "N/A"}</span>
                  </div>
                )}
                {rationale.edit_suggestion && (
                  <div>
                    <span className="font-medium">Edit suggestion:</span>{" "}
                    <span className="text-muted-foreground">{rationale.edit_suggestion}</span>
                  </div>
                )}
                {rationale.trending_fit && (
                  <div>
                    <span className="font-medium">Trending fit:</span>{" "}
                    <span className="text-muted-foreground">{rationale.trending_fit}</span>
                  </div>
                )}
                <Separator />
                <div className="flex gap-2">
                  <Button variant="ghost" size="sm" className="text-xs h-6" onClick={() => setFeedbackOpen(!feedbackOpen)}>
                    I disagree
                  </Button>
                  <Button variant="ghost" size="sm" className="text-xs h-6">Agree</Button>
                </div>
                {feedbackOpen && (
                  <div className="space-y-2">
                    <textarea
                      className="w-full text-xs bg-muted rounded p-2 resize-none h-16"
                      placeholder="What did the AI get wrong?"
                      value={feedbackText}
                      onChange={e => setFeedbackText(e.target.value)}
                    />
                    <div className="flex gap-1">
                      <Button size="sm" className="text-xs h-6" onClick={() => handleAction("reject")}>Submit</Button>
                      <Button variant="ghost" size="sm" className="text-xs h-6" onClick={() => setFeedbackOpen(false)}>Cancel</Button>
                    </div>
                  </div>
                )}
              </CardContent>
            </Card>
          )}

          <Card>
            <CardHeader className="py-2 px-3">
              <CardTitle className="text-xs">Signal Breakdown</CardTitle>
            </CardHeader>
            <CardContent className="px-3 pb-3 space-y-1.5">
              <SignalBar label="YOLO" value={0.35} />
              <SignalBar label="VLM" value={0.30} />
              <SignalBar label="Audio" value={0.15} />
              <SignalBar label="Scene" value={0.10} />
              <SignalBar label="Whisper" value={0.05} />
              <SignalBar label="Trend" value={0.05} />
            </CardContent>
          </Card>

          {clip.transcript && (
            <Card>
              <CardHeader className="py-2 px-3">
                <CardTitle className="text-xs">Transcript</CardTitle>
              </CardHeader>
              <CardContent className="px-3 pb-3">
                <p className="text-xs text-muted-foreground">{clip.transcript}</p>
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </div>
  );
}
