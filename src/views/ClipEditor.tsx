import { useState } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import type { KeyMoment } from "@/types";

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

function FilmstripTimeline({ moments }: { moments: KeyMoment[] }) {
  return (
    <div className="space-y-1">
      <div className="flex justify-between text-xs text-muted-foreground">
        <span>00:03</span>
        <span className="text-foreground text-xs">Trim: 00:03 - 00:31</span>
        <span>00:34</span>
      </div>
      <div className="relative h-10 bg-muted rounded overflow-hidden">
        <div className="absolute left-[8%] right-[8%] top-0 h-full bg-muted-foreground/10 rounded" />
        <div className="absolute left-[8%] top-0 w-0.5 h-full bg-primary rounded" />
        <div className="absolute right-[8%] top-0 w-0.5 h-full bg-primary rounded" />
        {moments.map((m, i) => {
          const pos = `${10 + (i * 20)}%`;
          const color =
            m.type === "kill" || m.type === "multi_kill" ? "bg-green-500" :
            m.type === "death" ? "bg-red-500" :
            m.type === "reaction" || m.type === "audio_peak" ? "bg-yellow-500" :
            "bg-blue-500";
          return (
            <div
              key={i}
              className={`absolute top-1 w-1.5 h-1.5 rounded-full ${color}`}
              style={{ left: pos }}
              title={`${m.label} (${m.type})`}
            />
          );
        })}
      </div>
      <div className="relative h-6 bg-muted/50 rounded overflow-hidden">
        {Array.from({ length: 40 }, (_, i) => (
          <div
            key={i}
            className="absolute bottom-0 w-0.5 bg-muted-foreground/30 rounded"
            style={{ left: `${(i / 40) * 100}%`, height: `${4 + Math.random() * 16}px` }}
          />
        ))}
      </div>
      <p className="text-xs text-muted-foreground">
        Colored dots: detection events (green=kill, red=death, yellow=reaction, blue=scene)
      </p>
    </div>
  );
}

export function ClipEditor() {
  const [format, setFormat] = useState<"16:9" | "9:16">("16:9");
  const [captionsOn, setCaptionsOn] = useState(true);
  const [trackSubject, setTrackSubject] = useState(true);
  const [voicePreset, setVoicePreset] = useState("none");
  const [captionStyle, setCaptionStyle] = useState("bold");
  const [transition, setTransition] = useState("cut");
  const [editorTab, setEditorTab] = useState("effects");

  const mockMoments: KeyMoment[] = [
    { timestamp_ms: 3000, type: "scene_change", label: "Scene", confidence: 0.8, signals: ["scene"] },
    { timestamp_ms: 12000, type: "kill", label: "Headshot", confidence: 0.95, signals: ["yolo"] },
    { timestamp_ms: 18000, type: "reaction", label: "Reaction", confidence: 0.7, signals: ["audio", "whisper"] },
    { timestamp_ms: 25000, type: "multi_kill", label: "Triple Kill", confidence: 0.98, signals: ["yolo"] },
  ];

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold tracking-tight">Edit Clip</h2>
        <div className="flex gap-2">
          <Button variant="ghost" onClick={() => toast.info("Changes discarded")}>Discard</Button>
          <Button variant="outline" onClick={() => toast.success("Draft saved")}>Save Draft</Button>
          <Button onClick={() => toast.info("Export will process the composition via FFmpeg. Coming in the next update.")}>Export</Button>
        </div>
      </div>

      <div className="grid grid-cols-[1fr_260px] gap-4">
        <div className="space-y-3">
          <div className="flex gap-2">
            <Button variant={format === "16:9" ? "default" : "outline"} size="sm" onClick={() => setFormat("16:9")}>Landscape</Button>
            <Button variant={format === "9:16" ? "default" : "outline"} size="sm" onClick={() => setFormat("9:16")}>Vertical</Button>
          </div>

          <Card>
            <CardContent className="pt-4">
              <div
                className="bg-muted rounded-lg flex items-center justify-center mx-auto"
                style={{
                  aspectRatio: format === "16:9" ? "16/9" : "9/16",
                  maxHeight: format === "16:9" ? 280 : 400,
                  width: format === "9:16" ? 200 : "100%",
                }}
              >
                <span className="text-muted-foreground text-sm">{format} preview</span>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="py-2">
              <CardTitle className="text-sm">Timeline</CardTitle>
            </CardHeader>
            <CardContent>
              <FilmstripTimeline moments={mockMoments} />
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
                <select
                  className="w-full mt-1 text-xs bg-muted border border-border rounded px-2 py-1.5"
                  value={voicePreset}
                  onChange={e => setVoicePreset(e.target.value)}
                >
                  {VOICE_PRESETS.map(v => <option key={v.id} value={v.id}>{v.name}</option>)}
                </select>
              </div>

              <div>
                <label className="text-xs font-medium">Captions</label>
                <div className="flex items-center justify-between mt-1">
                  <span className="text-xs text-muted-foreground">Burn-in captions</span>
                  <Switch checked={captionsOn} onCheckedChange={setCaptionsOn} />
                </div>
                {captionsOn && (
                  <select
                    className="w-full mt-1 text-xs bg-muted border border-border rounded px-2 py-1.5"
                    value={captionStyle}
                    onChange={e => setCaptionStyle(e.target.value)}
                  >
                    {CAPTION_STYLES.map(c => <option key={c.id} value={c.id}>{c.name}</option>)}
                  </select>
                )}
              </div>

              <div>
                <label className="text-xs font-medium">Transitions</label>
                <select
                  className="w-full mt-1 text-xs bg-muted border border-border rounded px-2 py-1.5"
                  value={transition}
                  onChange={e => setTransition(e.target.value)}
                >
                  {TRANSITIONS.map(t => <option key={t.id} value={t.id}>{t.name}</option>)}
                </select>
              </div>

              <div className="flex items-center justify-between">
                <span className="text-xs">Track subject (9:16)</span>
                <Switch checked={trackSubject} onCheckedChange={setTrackSubject} />
              </div>
            </TabsContent>

            <TabsContent value="audio" className="space-y-3 mt-3">
              <div className="text-xs text-muted-foreground">Trending sounds and SFX library will appear here.</div>
              <Button variant="outline" size="sm" className="text-xs" onClick={() => toast.info("Trending sounds browser coming in next update. Configure ScrapeBadger API key in Settings > Clipper.")}>Browse Trending Sounds</Button>
              <Button variant="outline" size="sm" className="text-xs" onClick={() => toast.info("Sound effects library coming in next update.")}>Sound Effects Library</Button>
            </TabsContent>

            <TabsContent value="export" className="space-y-3 mt-3">
              <div>
                <label className="text-xs font-medium">Format</label>
                <select className="w-full mt-1 text-xs bg-muted border border-border rounded px-2 py-1.5">
                  <option>MP4 (H.264)</option>
                  <option>MP4 (HEVC)</option>
                  <option>WebM (VP9)</option>
                </select>
              </div>
              <div>
                <label className="text-xs font-medium">Quality</label>
                <select className="w-full mt-1 text-xs bg-muted border border-border rounded px-2 py-1.5">
                  <option>Ultra (4K)</option>
                  <option>High (1080p)</option>
                  <option>Medium (720p)</option>
                </select>
              </div>
              <Separator />
              <div className="text-xs font-medium">Quick Export</div>
              <div className="flex flex-wrap gap-1">
                {["TikTok (9:16, 60s)", "YouTube Shorts", "Reels (9:16, 90s)", "X (16:9, 140s)"].map(p => (
                  <Badge key={p} variant="outline" className="text-xs cursor-pointer hover:bg-muted">{p}</Badge>
                ))}
              </div>
            </TabsContent>
          </Tabs>
        </div>
      </div>
    </div>
  );
}
