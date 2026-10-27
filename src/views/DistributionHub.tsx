import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

interface DistEntry {
  id: string;
  clip_id: string;
  platform: string;
  status: string;
  scheduled_at: string | null;
  posted_at: string | null;
  engagement_data: string | null;
  clip_title: string | null;
}

export function DistributionHub() {
  const navigate = useNavigate();
  const [history, setHistory] = useState<DistEntry[]>([]);
  const [postizConfigured, setPostizConfigured] = useState(false);

  const loadData = useCallback(async () => {
    try {
      const entries = await invoke<DistEntry[]>("get_distribution_history");
      setHistory(entries);
    } catch { setHistory([]); }

    try {
      const config = await invoke<Record<string, unknown>>("get_clipper_config");
      const url = config?.postiz_url as string || "";
      setPostizConfigured(url.length > 0 && url !== "https://api.postiz.com");
    } catch {}
  }, []);

  useEffect(() => { loadData(); }, [loadData]);

  const scheduled = history.filter(h => h.status === "scheduled");
  const posted = history.filter(h => h.status === "posted");

  const totalViews = posted.reduce((sum, p) => {
    try { const d = JSON.parse(p.engagement_data || "{}"); return sum + (d.views || 0); } catch { return sum; }
  }, 0);
  const totalLikes = posted.reduce((sum, p) => {
    try { const d = JSON.parse(p.engagement_data || "{}"); return sum + (d.likes || 0); } catch { return sum; }
  }, 0);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Posting</h2>
          <p className="text-muted-foreground text-sm">
            {postizConfigured
              ? "Connected to Postiz. Approved clips post on schedule."
              : "Configure Postiz in Settings > Clipper to enable auto-posting."}
          </p>
        </div>
        <Button variant="outline" onClick={() => navigate("/settings")}>Configure Postiz</Button>
      </div>

      {posted.length > 0 && (
        <div className="grid grid-cols-3 gap-4">
          <Card><CardContent className="pt-4"><div className="text-2xl font-bold">{totalViews.toLocaleString()}</div><div className="text-xs text-muted-foreground">Total Views</div></CardContent></Card>
          <Card><CardContent className="pt-4"><div className="text-2xl font-bold text-green-400">{totalLikes.toLocaleString()}</div><div className="text-xs text-muted-foreground">Total Likes</div></CardContent></Card>
          <Card><CardContent className="pt-4"><div className="text-2xl font-bold text-blue-400">{posted.length}</div><div className="text-xs text-muted-foreground">Posts Published</div></CardContent></Card>
        </div>
      )}

      {!postizConfigured && history.length === 0 && (
        <Card>
          <CardContent className="py-12 text-center">
            <p className="text-muted-foreground text-lg mb-2">No posts yet</p>
            <p className="text-muted-foreground text-sm mb-4">
              Connect Postiz to start auto-posting your approved clips to TikTok, YouTube Shorts, Instagram Reels, and more.
            </p>
            <Button onClick={() => navigate("/settings")}>Set Up Postiz</Button>
          </CardContent>
        </Card>
      )}

      {scheduled.length > 0 && (
        <div className="space-y-3">
          <h3 className="text-sm font-semibold text-muted-foreground">Scheduled</h3>
          {scheduled.map(entry => (
            <Card key={entry.id}>
              <CardContent className="py-3">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Badge variant="outline">{entry.platform}</Badge>
                    <span className="text-sm">{entry.clip_title ?? entry.clip_id}</span>
                  </div>
                  <span className="text-xs text-muted-foreground">{entry.scheduled_at ?? "Pending"}</span>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {posted.length > 0 && (
        <div className="space-y-3">
          <h3 className="text-sm font-semibold text-muted-foreground">Post History</h3>
          <div className="rounded-lg border overflow-hidden">
            <table className="w-full text-sm">
              <thead className="bg-muted/50">
                <tr>
                  <th className="text-left px-3 py-2 text-xs font-medium">Clip</th>
                  <th className="text-left px-3 py-2 text-xs font-medium">Platform</th>
                  <th className="text-left px-3 py-2 text-xs font-medium">Posted</th>
                  <th className="text-right px-3 py-2 text-xs font-medium">Views</th>
                  <th className="text-right px-3 py-2 text-xs font-medium">Likes</th>
                </tr>
              </thead>
              <tbody>
                {posted.map(entry => {
                  let views = 0, likes = 0;
                  try { const d = JSON.parse(entry.engagement_data || "{}"); views = d.views || 0; likes = d.likes || 0; } catch {}
                  return (
                    <tr key={entry.id} className="border-t border-border/50">
                      <td className="px-3 py-2 text-xs">{entry.clip_title ?? "Untitled"}</td>
                      <td className="px-3 py-2 text-xs"><Badge variant="outline" className="text-xs">{entry.platform}</Badge></td>
                      <td className="px-3 py-2 text-xs text-muted-foreground">{entry.posted_at ?? "-"}</td>
                      <td className="px-3 py-2 text-xs text-right">{views.toLocaleString()}</td>
                      <td className="px-3 py-2 text-xs text-right">{likes.toLocaleString()}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}
