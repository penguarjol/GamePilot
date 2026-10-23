import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Switch } from "@/components/ui/switch";
import { Separator } from "@/components/ui/separator";

interface Platform {
  id: string;
  name: string;
  connected: boolean;
  posts: number;
  format: string;
}

const PLATFORMS: Platform[] = [
  { id: "tiktok", name: "TikTok", connected: true, posts: 47, format: "9:16, 60s" },
  { id: "youtube", name: "YouTube Shorts", connected: true, posts: 31, format: "9:16, 60s" },
  { id: "instagram", name: "Instagram Reels", connected: true, posts: 22, format: "9:16, 90s" },
  { id: "x", name: "X / Twitter", connected: false, posts: 0, format: "16:9, 140s" },
  { id: "discord", name: "Discord", connected: true, posts: 85, format: "Any" },
  { id: "reddit", name: "Reddit", connected: false, posts: 0, format: "16:9, 15min" },
];

const SCHEDULE = [
  { day: "Mon", time: "12:00 PM", platform: "TikTok", clip: "Triple Kill - Valorant" },
  { day: "Mon", time: "6:00 PM", platform: "YouTube Shorts", clip: "Ace - CS2" },
  { day: "Tue", time: "12:00 PM", platform: "TikTok", clip: "Pentakill - League" },
  { day: "Wed", time: "12:00 PM", platform: "TikTok", clip: "Squad Wipe - Apex" },
  { day: "Thu", time: "12:00 PM", platform: "TikTok", clip: "Auto-selected" },
  { day: "Fri", time: "12:00 PM", platform: "TikTok", clip: "Auto-selected" },
];

const TRENDING_TAGS = ["#gaming", "#valorantclips", "#clutch", "#ace", "#highlights", "#fyp", "#gamingmoments"];

export function DistributionHub() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold tracking-tight">Posting</h2>
        <p className="text-muted-foreground text-sm">Approved clips go out on schedule. Connect platforms and set the cadence.</p>
      </div>

      <div className="grid grid-cols-3 gap-4">
        <Card><CardContent className="pt-4"><div className="text-2xl font-bold">12.4k</div><div className="text-xs text-muted-foreground">Views this week</div></CardContent></Card>
        <Card><CardContent className="pt-4"><div className="text-2xl font-bold text-green-400">843</div><div className="text-xs text-muted-foreground">Likes this week</div></CardContent></Card>
        <Card><CardContent className="pt-4"><div className="text-2xl font-bold text-blue-400">3.2%</div><div className="text-xs text-muted-foreground">Avg engagement</div></CardContent></Card>
      </div>

      <div>
        <h3 className="text-sm font-semibold text-muted-foreground mb-3">Connected Platforms</h3>
        <div className="grid grid-cols-3 gap-3">
          {PLATFORMS.map(p => (
            <Card key={p.id}>
              <CardContent className="py-3">
                <div className="flex items-center justify-between mb-2">
                  <span className="font-medium text-sm">{p.name}</span>
                  <Switch checked={p.connected} />
                </div>
                <div className="space-y-1 text-xs text-muted-foreground">
                  <div className="flex justify-between"><span>Posts</span><span className="text-foreground">{p.posts}</span></div>
                  <div className="flex justify-between"><span>Format</span><span>{p.format}</span></div>
                </div>
                {!p.connected && <Button size="sm" className="w-full mt-2 text-xs">Connect via Postiz</Button>}
              </CardContent>
            </Card>
          ))}
        </div>
      </div>

      <Separator />

      <div>
        <h3 className="text-sm font-semibold text-muted-foreground mb-3">Weekly Schedule</h3>
        <div className="rounded-lg border overflow-hidden">
          <table className="w-full text-sm">
            <thead className="bg-muted/50">
              <tr><th className="text-left px-3 py-2 text-xs font-medium">Day</th><th className="text-left px-3 py-2 text-xs font-medium">Time</th><th className="text-left px-3 py-2 text-xs font-medium">Platform</th><th className="text-left px-3 py-2 text-xs font-medium">Clip</th></tr>
            </thead>
            <tbody>
              {SCHEDULE.map((s, i) => (
                <tr key={i} className="border-t border-border/50">
                  <td className="px-3 py-2 text-xs">{s.day}</td>
                  <td className="px-3 py-2 text-xs">{s.time}</td>
                  <td className="px-3 py-2 text-xs">{s.platform}</td>
                  <td className="px-3 py-2 text-xs text-muted-foreground">{s.clip}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      <Card className="bg-muted/20">
        <CardHeader className="py-2"><CardTitle className="text-sm">Trending Now</CardTitle></CardHeader>
        <CardContent className="pb-3">
          <div className="flex gap-1.5 flex-wrap">
            {TRENDING_TAGS.map(tag => (
              <Badge key={tag} variant="outline" className="text-xs">{tag}</Badge>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
