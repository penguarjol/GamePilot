import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import type { ProcessingJob } from "@/types";

function stageLabel(status: string): string {
  switch (status) {
    case "queued": return "Queued";
    case "detecting": return "Detecting highlights (audio + scene + YOLO)";
    case "scoring": return "Scoring candidates (Decima + Qwen3)";
    case "processing": return "Extracting clips (FFmpeg)";
    case "done": return "Complete";
    case "failed": return "Failed";
    default: return status;
  }
}

function statusColor(status: string): string {
  switch (status) {
    case "done": return "text-green-400";
    case "failed": return "text-red-400";
    case "detecting": case "scoring": case "processing": return "text-blue-400";
    default: return "text-muted-foreground";
  }
}

export function ProcessingQueue() {
  const [jobs] = useState<ProcessingJob[]>([]);

  const activeJobs = jobs.filter(j => !["done", "failed"].includes(j.status));
  const completedJobs = jobs.filter(j => j.status === "done");
  const failedJobs = jobs.filter(j => j.status === "failed");

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Processing</h2>
          <p className="text-muted-foreground text-sm">
            Recordings are analyzed after each session. Clips appear in Review once scored.
          </p>
        </div>
        <div className="flex gap-2">
          <Button>Process Video File</Button>
          <Button variant="outline">Watch Folder</Button>
        </div>
      </div>

      {activeJobs.length > 0 && (
        <div className="space-y-3">
          <h3 className="text-sm font-semibold text-muted-foreground">Active</h3>
          {activeJobs.map(job => (
            <Card key={job.id}>
              <CardContent className="py-4 space-y-3">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Badge variant="outline">{job.game_id ?? "Unknown"}</Badge>
                    <span className="font-medium text-sm">{job.source_video.split(/[/\\]/).pop()}</span>
                  </div>
                  <span className="text-xs text-muted-foreground">
                    {job.clips_found} candidate{job.clips_found !== 1 ? "s" : ""} found
                  </span>
                </div>
                <Progress value={job.progress * 100} />
                <div className="flex items-center justify-between text-xs">
                  <span className={statusColor(job.status)}>{stageLabel(job.status)}</span>
                  <span className="text-muted-foreground">{Math.round(job.progress * 100)}%</span>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {activeJobs.length === 0 && completedJobs.length === 0 && (
        <Card>
          <CardContent className="py-12 text-center text-muted-foreground">
            No processing jobs. Start a gaming session or drop a recording to process.
          </CardContent>
        </Card>
      )}

      {completedJobs.length > 0 && (
        <div className="space-y-3">
          <h3 className="text-sm font-semibold text-muted-foreground">Completed</h3>
          {completedJobs.map(job => (
            <Card key={job.id} className="bg-muted/30">
              <CardContent className="py-3">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Badge variant="outline">{job.game_id ?? "Unknown"}</Badge>
                    <span className="text-sm">{job.source_video.split(/[/\\]/).pop()}</span>
                  </div>
                  <div className="flex items-center gap-3 text-xs">
                    <span className="text-green-400">{job.clips_found} clips</span>
                    <span className="text-muted-foreground">{job.clips_approved} approved</span>
                    <Button variant="ghost" size="sm">View Clips</Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {failedJobs.length > 0 && (
        <div className="space-y-3">
          <h3 className="text-sm font-semibold text-red-400">Failed</h3>
          {failedJobs.map(job => (
            <Card key={job.id} className="border-red-500/20">
              <CardContent className="py-3">
                <div className="flex items-center justify-between">
                  <span className="text-sm">{job.source_video.split(/[/\\]/).pop()}</span>
                  <div className="flex items-center gap-2">
                    <span className="text-xs text-red-400">{job.error ?? "Unknown error"}</span>
                    <Button variant="outline" size="sm">Retry</Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      <Card className="bg-muted/20 border-dashed">
        <CardContent className="py-4">
          <h4 className="text-sm font-medium mb-2">How detection works</h4>
          <div className="grid grid-cols-3 gap-3 text-xs text-muted-foreground">
            <div><span className="text-foreground font-medium">Audio peaks</span> — loud moments, reactions (librosa)</div>
            <div><span className="text-foreground font-medium">Scene changes</span> — transitions, cuts (PySceneDetect)</div>
            <div><span className="text-foreground font-medium">Speech</span> — words + timestamps (faster-whisper)</div>
            <div><span className="text-foreground font-medium">Game events</span> — kills, deaths, HUD (YOLO v11)</div>
            <div><span className="text-foreground font-medium">Screen text</span> — filters killcams (EasyOCR)</div>
            <div><span className="text-foreground font-medium">Scene understanding</span> — what happened (Qwen2.5-VL)</div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
