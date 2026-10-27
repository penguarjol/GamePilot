import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { useNavigate, useSearchParams } from "react-router-dom";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import type { Clip, KeyMoment } from "@/types";

const VOICE_PRESETS = [
  { id: "none", name: "Original" },
  { id: "deep", name: "Deep Voice" },
  { id: "chipmunk", name: "Chipmunk" },
  { id: "echo", name: "Echo" },
  { id: "reverb", name: "Reverb" },
  { id: "slowmo", name: "Slow-mo Voice" },
];

const CAPTION_STYLES = [
  { id: "bold", name: "Bold White" },
  { id: "bar", name: "Black Bar" },
  { id: "karaoke", name: "Karaoke" },
  { id: "hormozi", name: "One Word" },
];

const TRANSITIONS = [
  { id: "cut", name: "Hard Cut" },
  { id: "fade", name: "Crossfade" },
  { id: "wipe", name: "Wipe" },
  { id: "zoom", name: "Zoom Through" },
  { id: "glitch", name: "Glitch" },
];

function markerColor(type: string): string {
  if (type === "kill" || type === "multi_kill") return "bg-green-500";
  if (type === "death") return "bg-red-500";
  if (type === "reaction" || type === "audio_peak") return "bg-yellow-500";
  return "bg-blue-500";
}

export function ClipEditor() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const clipId = searchParams.get("clipId");

  const [clip, setClip] = useState<Clip | null>(null);
  const [moments, setMoments] = useState<KeyMoment[]>([]);
  const [format, setFormat] = useState<"16:9" | "9:16">("16:9");
  const [captionsOn, setCaptionsOn] = useState(true);
  const [trackSubject, setTrackSubject] = useState(true);
  const [voicePreset, setVoicePreset] = useState("none");
  const [captionStyle, setCaptionStyle] = useState("bold");
  const [transition, setTransition] = useState("cut");
  const [editorTab, setEditorTab] = useState("effects");
  const [exporting, setExporting] = useState(false);
  const videoRef = useRef<HTMLVideoElement>(null);

  useEffect(() => {
    if (!clipId) return;
    invoke<Clip>("get_clip", { clipId }).then(setClip).catch(() => {});
    invoke<Array<Record<string, unknown>>>("get_clip_signals", { clipId }).then(signals => {
      setMoments(signals.map(s => ({
        timestamp_ms: (s.timestamp_ms as number) ?? 0,
        type: ((s.signal_type as string) ?? "event") as KeyMoment["type"],
        label: (s.signal_data as string) ?? "",
        confidence: (s.signal_value as number) ?? 0,
        signals: [s.signal_type as string],
      })));
    }).catch(() => {});
  }, [clipId]);

  const handleExport = async () => {
    if (!clip?.source_video) return;
    try {
      const outPath = await save({
        defaultPath: `${clip.title ?? "clip"}_export.mp4`,
        filters: [{ name: "Video", extensions: ["mp4"] }],
      });
      if (!outPath) return;
      setExporting(true);
      toast.info("Exporting clip via FFmpeg...");
      await invoke("export_clip", {
        videoPath: clip.source_video,
        startMs: clip.start_time_ms,
        endMs: clip.end_time_ms,
        outputPath: outPath,
        verticalCrop: format === "9:16",
        burnCaptions: captionsOn,
      });
      toast.success(`Exported to ${outPath}`);
    } catch (e) {
      toast.error(`Export failed: ${e}`);
    } finally {
      setExporting(false);
    }
  };

  const handleSaveDraft = async () => {
    if (!clipId) return;
    try {
      const composition = JSON.stringify({ format, captionsOn, trackSubject, voicePreset, captionStyle, transition, moments });
      await invoke("save_composition", { clipId, compositionJson: composition, recipeId: null });
      toast.success("Draft saved");
    } catch (e) {
      toast.error(`Save failed: ${e}`);
    }
  };

  const handleDiscard = () => {
    navigate(-1);
  };

  const handlePreviewVoice = async () => {
    if (!clip?.source_video || voicePreset === "none") return;
    try {
      toast.info("Rendering voice preview...");
      const result = await invoke<Record<string, string>>("preview_effect", {
        videoPath: clip.source_video,
        effectType: "voice_mod",
        preset: voicePreset,
      });
      if (result.preview_path && videoRef.current) {
        videoRef.current.src = `file://${result.preview_path}`;
        videoRef.current.play();
      }
    } catch (e) {
      toast.error(`Preview failed: ${e}`);
    }
  };

  const videoSrc = clip?.clip_path ? `file://${clip.clip_path}` : clip?.source_video ? `file://${clip.source_video}` : undefined;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold tracking-tight">
          {clip ? `Edit: ${clip.title ?? "Untitled Clip"}` : "Editor"}
        </h2>
        <div className="flex gap-2">
          <Button variant="ghost" onClick={handleDiscard}>Back</Button>
          <Button variant="outline" onClick={handleSaveDraft}>Save Draft</Button>
          <Button onClick={handleExport} disabled={exporting || !clip}>
            {exporting ? "Exporting..." : "Export"}
          </Button>
        </div>
      </div>

      {!clipId && (
        <Card>
          <CardContent className="py-12 text-center text-muted-foreground">
            No clip selected. Go to the Clips tab and click a clip to open it here.
          </CardContent>
        </Card>
      )}

      {clipId && (
        <div className="grid grid-cols-[1fr_260px] gap-4">
          <div className="space-y-3">
            <div className="flex gap-2">
              <Button variant={format === "16:9" ? "default" : "outline"} size="sm" onClick={() => setFormat("16:9")}>Landscape</Button>
              <Button variant={format === "9:16" ? "default" : "outline"} size="sm" onClick={() => setFormat("9:16")}>Vertical</Button>
            </div>

            <Card>
              <CardContent className="pt-4">
                {videoSrc ? (
                  <video
                    ref={videoRef}
                    src={videoSrc}
                    controls
                    className="w-full rounded-lg"
                    style={{
                      maxHeight: format === "16:9" ? 300 : 400,
                      aspectRatio: format === "16:9" ? "16/9" : "9/16",
                      objectFit: "contain",
                      background: "#000",
                      margin: "0 auto",
                      display: "block",
                      width: format === "9:16" ? 200 : "100%",
                    }}
                  />
                ) : (
                  <div className="aspect-video bg-muted rounded-lg flex items-center justify-center">
                    <span className="text-muted-foreground text-sm">No video loaded</span>
                  </div>
                )}
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="py-2"><CardTitle className="text-sm">Timeline</CardTitle></CardHeader>
              <CardContent>
                <div className="space-y-1">
                  {clip && (
                    <div className="flex justify-between text-xs text-muted-foreground">
                      <span>{((clip.start_time_ms) / 1000).toFixed(1)}s</span>
                      <span>{clip.duration_ms ? `${(clip.duration_ms / 1000).toFixed(1)}s duration` : ""}</span>
                      <span>{((clip.end_time_ms) / 1000).toFixed(1)}s</span>
                    </div>
                  )}
                  <div className="relative h-8 bg-muted rounded overflow-hidden">
                    {moments.map((m, i) => {
                      const clipDur = clip ? clip.end_time_ms - clip.start_time_ms : 1;
                      const pos = clip ? ((m.timestamp_ms - clip.start_time_ms) / clipDur) * 100 : 0;
                      return (
                        <div
                          key={i}
                          className={`absolute top-1 w-2 h-2 rounded-full ${markerColor(m.type)} cursor-pointer`}
                          style={{ left: `${Math.max(0, Math.min(98, pos))}%` }}
                          title={`${m.label || m.type} @ ${(m.timestamp_ms / 1000).toFixed(1)}s`}
                          onClick={() => {
                            if (videoRef.current) videoRef.current.currentTime = m.timestamp_ms / 1000;
                          }}
                        />
                      );
                    })}
                  </div>
                  {moments.length === 0 && (
                    <p className="text-xs text-muted-foreground">No key moments detected for this clip.</p>
                  )}
                  {moments.length > 0 && (
                    <p className="text-xs text-muted-foreground">{moments.length} key moments (green=kill, red=death, yellow=reaction, blue=scene)</p>
                  )}
                </div>
              </CardContent>
            </Card>
          </div>

          <div className="space-y-3">
            <Tabs value={editorTab} onValueChange={setEditorTab}>
              <TabsList className="w-full">
                <TabsTrigger value="effects" className="flex-1 text-xs">Effects</TabsTrigger>
                <TabsTrigger value="audio" className="flex-1 text-xs">Audio</TabsTrigger>
                <TabsTrigger value="export" className="flex-1 text-xs">Export</TabsTrigger>
              </TabsList>

              <TabsContent value="effects" className="space-y-3 mt-3">
                <div>
                  <label className="text-xs font-medium">Voice Mod</label>
                  <select className="w-full mt-1 text-xs bg-muted border border-border rounded px-2 py-1.5" value={voicePreset} onChange={e => setVoicePreset(e.target.value)}>
                    {VOICE_PRESETS.map(v => <option key={v.id} value={v.id}>{v.name}</option>)}
                  </select>
                  {voicePreset !== "none" && (
                    <Button variant="ghost" size="sm" className="text-xs mt-1" onClick={handlePreviewVoice}>Preview</Button>
                  )}
                </div>
                <div>
                  <label className="text-xs font-medium">Captions</label>
                  <div className="flex items-center justify-between mt-1">
                    <span className="text-xs text-muted-foreground">Burn-in captions</span>
                    <Switch checked={captionsOn} onCheckedChange={setCaptionsOn} />
                  </div>
                  {captionsOn && (
                    <select className="w-full mt-1 text-xs bg-muted border border-border rounded px-2 py-1.5" value={captionStyle} onChange={e => setCaptionStyle(e.target.value)}>
                      {CAPTION_STYLES.map(c => <option key={c.id} value={c.id}>{c.name}</option>)}
                    </select>
                  )}
                </div>
                <div>
                  <label className="text-xs font-medium">Transitions</label>
                  <select className="w-full mt-1 text-xs bg-muted border border-border rounded px-2 py-1.5" value={transition} onChange={e => setTransition(e.target.value)}>
                    {TRANSITIONS.map(t => <option key={t.id} value={t.id}>{t.name}</option>)}
                  </select>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-xs">Track subject (9:16)</span>
                  <Switch checked={trackSubject} onCheckedChange={setTrackSubject} />
                </div>
              </TabsContent>

              <TabsContent value="audio" className="space-y-3 mt-3">
                <p className="text-xs text-muted-foreground">Add trending sounds or sound effects to your clip.</p>
                <Button variant="outline" size="sm" className="text-xs w-full" onClick={async () => {
                  const key = await invoke<string | null>("get_preference", { key: "scrapebadger_api_key" }).catch(() => null);
                  if (!key) { toast.error("Set ScrapeBadger API key in Settings > Clipper first"); return; }
                  toast.info("Trending sounds browser: configure in Settings > Clipper. Full browser panel coming soon.");
                }}>Browse Trending Sounds</Button>
                <Button variant="outline" size="sm" className="text-xs w-full" onClick={() => {
                  toast.info("Sound effects are bundled in sidecar/sfx/. Drag files onto the timeline.");
                }}>Sound Effects Library</Button>
              </TabsContent>

              <TabsContent value="export" className="space-y-3 mt-3">
                <div>
                  <label className="text-xs font-medium">Format</label>
                  <select className="w-full mt-1 text-xs bg-muted border border-border rounded px-2 py-1.5">
                    <option>MP4 (H.264)</option>
                    <option>MP4 (HEVC)</option>
                  </select>
                </div>
                <div>
                  <label className="text-xs font-medium">Quality</label>
                  <select className="w-full mt-1 text-xs bg-muted border border-border rounded px-2 py-1.5">
                    <option>High (1080p)</option>
                    <option>Ultra (4K)</option>
                  </select>
                </div>
                <Separator />
                <div className="text-xs font-medium">Quick Export</div>
                <div className="flex flex-wrap gap-1">
                  {["TikTok", "YouTube Shorts", "Reels", "X"].map(p => (
                    <Badge key={p} variant="outline" className="text-xs cursor-pointer hover:bg-muted" onClick={() => {
                      if (p === "TikTok" || p === "YouTube Shorts" || p === "Reels") setFormat("9:16");
                      else setFormat("16:9");
                      toast.info(`Format set for ${p}`);
                    }}>{p}</Badge>
                  ))}
                </div>
              </TabsContent>
            </Tabs>
          </div>
        </div>
      )}
    </div>
  );
}
