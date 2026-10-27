import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useNavigate } from "react-router-dom";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";

interface Job {
  id: string;
  source_video: string;
  game_id: string | null;
  status: string;
  progress: number;
  clips_found: number;
  clips_approved: number;
  error: string | null;
  created_at: string;
}

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

export function ProcessingQueue() {
  const navigate = useNavigate();
  const [jobs, setJobs] = useState<Job[]>([]);

  const loadJobs = useCallback(async () => {
    try {
      const result = await invoke<Job[]>("get_processing_jobs");
      setJobs(result);
    } catch { setJobs([]); }
  }, []);

  useEffect(() => {
    loadJobs();
    const i = setInterval(loadJobs, 5000);
    return () => clearInterval(i);
  }, [loadJobs]);

  const processFile = async () => {
    try {
      const file = await open({ multiple: false, filters: [{ name: "Video", extensions: ["mp4", "mkv", "mov", "avi", "webm"] }] });
      if (!file) return;
      const name = typeof file === "string" ? file.split(/[/\\]/).pop() : String(file);
      toast.info(`Processing: ${name}`);
      const result = await invoke<Record<string, unknown>>("process_video_file", { videoPath: file, gameId: null });
      toast.success(`Done. Found ${result.candidates ?? 0} clip candidates.`);
      loadJobs();
    } catch (e) {
      toast.error(`Failed: ${e}`);
    }
  };

  const deleteJob = async (jobId: string) => {
    try {
      await invoke("delete_processing_job", { jobId });
      toast.success("Job removed");
      loadJobs();
    } catch (e) { toast.error(`Delete failed: ${e}`); }
  };

  const clearCompleted = async () => {
    try {
      const count = await invoke<number>("clear_completed_jobs");
      toast.success(`Cleared ${count} job${count !== 1 ? "s" : ""}`);
      loadJobs();
    } catch (e) { toast.error(`Clear failed: ${e}`); }
  };

  const retryJob = async (job: Job) => {
    try {
      toast.info("Retrying...");
      await invoke("process_video_file", { videoPath: job.source_video, gameId: job.game_id });
      toast.success("Retry complete");
      loadJobs();
    } catch (e) { toast.error(`Retry failed: ${e}`); }
  };

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
          <Button onClick={processFile}>Process Video File</Button>
          <Button variant="outline" onClick={() => navigate("/settings")}>Watch Folders</Button>
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
                    {job.game_id && <Badge variant="outline">{job.game_id}</Badge>}
                    <span className="font-medium text-sm">{job.source_video.split(/[/\\]/).pop()}</span>
                  </div>
                  <span className="text-xs text-muted-foreground">{job.clips_found} candidate{job.clips_found !== 1 ? "s" : ""}</span>
                </div>
                <Progress value={job.progress * 100} />
                <div className="flex items-center justify-between text-xs">
                  <span className="text-blue-400">{stageLabel(job.status)}</span>
                  <span className="text-muted-foreground">{Math.round(job.progress * 100)}%</span>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {activeJobs.length === 0 && completedJobs.length === 0 && failedJobs.length === 0 && (
        <Card>
          <CardContent className="py-12 text-center text-muted-foreground">
            No processing jobs. Start a gaming session or drop a recording to process.
          </CardContent>
        </Card>
      )}

      {completedJobs.length > 0 && (
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-semibold text-muted-foreground">Completed</h3>
            <Button variant="ghost" size="sm" onClick={clearCompleted}>Clear All</Button>
          </div>
          {completedJobs.map(job => (
            <Card key={job.id} className="bg-muted/30">
              <CardContent className="py-3">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    {job.game_id && <Badge variant="outline">{job.game_id}</Badge>}
                    <span className="text-sm">{job.source_video.split(/[/\\]/).pop()}</span>
                  </div>
                  <div className="flex items-center gap-2 text-xs">
                    <span className="text-green-400">{job.clips_found} clips</span>
                    <Button variant="ghost" size="sm" onClick={() => navigate("/clips")}>View Clips</Button>
                    <Button variant="ghost" size="sm" onClick={() => deleteJob(job.id)}>Remove</Button>
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
                  <div>
                    <span className="text-sm">{job.source_video.split(/[/\\]/).pop()}</span>
                    <p className="text-xs text-red-400 mt-0.5">{job.error ?? "Unknown error"}</p>
                  </div>
                  <div className="flex gap-1">
                    <Button variant="outline" size="sm" onClick={() => retryJob(job)}>Retry</Button>
                    <Button variant="ghost" size="sm" onClick={() => deleteJob(job.id)}>Remove</Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
