import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { useInvoke } from "@/hooks/useInvoke";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import type { Recommendation, SavedInstance, ConfigAnalysis, ConfigRecommendation, RecommendationOutcome } from "@/types";

type FilterStatus = "all" | "new" | "applied" | "ignored" | "deferred";

export function Recommendations() {
  const saved = useInvoke<SavedInstance[]>("get_saved_instances");
  const [recommendations, setRecommendations] = useState<Recommendation[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [filterStatus, setFilterStatus] = useState<FilterStatus>("all");
  const [statusMap, setStatusMap] = useState<Record<string, string>>({});
  const [configRecs, setConfigRecs] = useState<ConfigRecommendation[]>([]);
  const [selectedInstanceId, setSelectedInstanceId] = useState<string | null>(null);
  const [outcomeMap, setOutcomeMap] = useState<Record<string, RecommendationOutcome[]>>({});

  useEffect(() => {
    saved.execute();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (saved.data && saved.data.length > 0) {
      setSelectedInstanceId(saved.data[0].id);
      loadRecommendations(saved.data[0]);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [saved.data]);

  const loadRecommendations = async (instance: SavedInstance) => {
    setLoading(true);
    setError(null);
    try {
      const recs = await invoke<Recommendation[]>("get_recommendations_for_path", {
        instancePath: instance.path,
        launcher: instance.launcher ?? "Custom",
      });
      setRecommendations(recs);

      try {
        const statuses = await invoke<Record<string, string>>("get_recommendation_statuses", {
          instanceId: instance.id,
        });
        setStatusMap(statuses);
      } catch {
        setStatusMap({});
      }

      try {
        const config = await invoke<ConfigAnalysis>("analyze_configs", {
          instancePath: instance.path,
          modCount: instance.mod_count ?? 0,
        });
        setConfigRecs(config.recommendations);
      } catch {
        setConfigRecs([]);
      }

      try {
        const outcomes = await invoke<RecommendationOutcome[]>("get_recommendation_outcomes", {
          instanceId: instance.id,
        });
        const grouped: Record<string, RecommendationOutcome[]> = {};
        for (const o of outcomes) {
          (grouped[o.recommendation_id] ??= []).push(o);
        }
        setOutcomeMap(grouped);
      } catch {
        setOutcomeMap({});
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const updateStatus = async (recId: string, status: string) => {
    try {
      await invoke("update_recommendation_status", { recommendationId: recId, status });
      setStatusMap((prev) => ({ ...prev, [recId]: status }));
      toast.success(`Marked as ${status.replace("_", " ")}`);
    } catch (err) {
      toast.error(`Failed to update: ${err}`);
    }
  };

  const getRecStatus = (recId: string): string => {
    return statusMap[recId] ?? "new";
  };

  const filtered = recommendations.filter((r) => {
    if (filterStatus === "all") return true;
    const s = getRecStatus(r.id);
    if (filterStatus === "ignored") return s === "ignored_once" || s === "ignored_always";
    return s === filterStatus;
  });

  const summarizeOutcome = (recId: string): { label: string; className: string } | null => {
    const outcomes = outcomeMap[recId];
    if (!outcomes || outcomes.length === 0) return null;
    const positives = outcomes.filter((o) => o.outcome === "positive");
    const negatives = outcomes.filter((o) => o.outcome === "negative");
    if (positives.length > 0 && negatives.length === 0) {
      const best = positives.reduce((a, b) =>
        (a.improvement_percent ?? 0) > (b.improvement_percent ?? 0) ? a : b
      );
      return {
        label: `Verified: +${Math.round(best.improvement_percent ?? 0)}% improvement`,
        className: "bg-emerald-500/10 text-emerald-600 border-emerald-500/20",
      };
    }
    if (negatives.length > 0) {
      return {
        label: "Mixed results",
        className: "bg-amber-500/10 text-amber-600 border-amber-500/20",
      };
    }
    return null;
  };

  const handleInstanceChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const inst = saved.data?.find((s) => s.id === e.target.value);
    if (inst) {
      setSelectedInstanceId(inst.id);
      loadRecommendations(inst);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold tracking-tight">Recommendations</h1>
        {saved.data && saved.data.length > 0 && (
          <select
            className="rounded-lg border border-border bg-card px-3 py-1.5 text-sm text-foreground"
            value={selectedInstanceId ?? ""}
            onChange={handleInstanceChange}
          >
            {saved.data.map((s) => (
              <option key={s.id} value={s.id}>{s.name}</option>
            ))}
          </select>
        )}
      </div>

      <Tabs defaultValue="all" onValueChange={(v) => setFilterStatus(v as FilterStatus)}>
        <TabsList>
          <TabsTrigger value="all">All ({recommendations.length})</TabsTrigger>
          <TabsTrigger value="new">New</TabsTrigger>
          <TabsTrigger value="applied">Applied</TabsTrigger>
          <TabsTrigger value="ignored">Ignored</TabsTrigger>
          <TabsTrigger value="deferred">Deferred</TabsTrigger>
        </TabsList>

        <TabsContent value={filterStatus} className="mt-4">
          {error && <p className="text-sm text-destructive mb-4">{error}</p>}

          {loading ? (
            <div className="space-y-3">
              {Array.from({ length: 3 }).map((_, i) => (
                <div key={i} className="h-32 animate-pulse rounded-xl bg-muted" />
              ))}
            </div>
          ) : filtered.length > 0 ? (
            <div className="space-y-4">
              {filtered.map((r) => {
                const status = getRecStatus(r.id);
                const outcomeBadge = summarizeOutcome(r.id);
                return (
                  <Card key={r.id} className={`transition-opacity duration-300 ${status !== "new" ? "opacity-60" : ""}`}>
                    <CardContent className="pt-4 space-y-3">
                      <div className="flex flex-wrap gap-1.5">
                        <Badge variant={r.severity === "error" ? "destructive" : "secondary"}>{r.severity}</Badge>
                        <Badge variant="outline">{r.confidence}</Badge>
                        <Badge variant="outline">risk: {r.risk_level}</Badge>
                        <Badge variant="secondary">{r.category}</Badge>
                        {status !== "new" && (
                          <Badge variant="default">{status.replace("_", " ")}</Badge>
                        )}
                        {outcomeBadge && (
                          <Badge variant="outline" className={outcomeBadge.className}>
                            {outcomeBadge.label}
                          </Badge>
                        )}
                      </div>
                      <h3 className="font-medium">{r.title}</h3>
                      <p className="text-sm text-muted-foreground">{r.description}</p>
                      <div className="text-xs text-muted-foreground space-y-1">
                        <p><strong>Evidence:</strong> {r.evidence}</p>
                        <p><strong>Impact:</strong> {r.expected_impact}</p>
                      </div>
                      <div className="flex gap-2 pt-1">
                        {status === "new" && (
                          <>
                            {r.action_type === "open_link" && r.action_data ? (
                              <a href={r.action_data} target="_blank" rel="noreferrer">
                                <Button size="sm">Open Link</Button>
                              </a>
                            ) : (
                              <Button size="sm" onClick={() => updateStatus(r.id, "applied")}>
                                Mark Reviewed
                              </Button>
                            )}
                            <Button size="sm" variant="secondary" onClick={() => updateStatus(r.id, "ignored_once")}>
                              Ignore Once
                            </Button>
                            <Button size="sm" variant="secondary" onClick={() => {
                              updateStatus(r.id, "ignored_always");
                              invoke("add_ignore_rule", { ruleType: "recommendation", pattern: r.title });
                            }}>
                              Ignore Always
                            </Button>
                            <Button size="sm" variant="secondary" onClick={() => updateStatus(r.id, "deferred")}>
                              Defer
                            </Button>
                          </>
                        )}
                      </div>
                    </CardContent>
                  </Card>
                );
              })}
            </div>
          ) : (
            <div className="flex flex-col items-center justify-center h-40 text-muted-foreground">
              <span className="text-3xl mb-2">{"\u2691"}</span>
              <p className="text-sm">
                {recommendations.length === 0
                  ? "No recommendations yet. Add and analyze a Minecraft instance first."
                  : "No recommendations match the current filter."}
              </p>
            </div>
          )}
        </TabsContent>
      </Tabs>

      {/* Config Recommendations */}
      {configRecs.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>Config Recommendations</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            {configRecs.map((cr, i) => (
              <div key={i} className="rounded-lg border border-border p-3 space-y-1">
                <div className="flex items-center justify-between">
                  <span className="font-mono text-xs">{cr.file}</span>
                  <Badge variant="outline">{cr.confidence}</Badge>
                </div>
                <div className="text-sm">
                  <span className="text-muted-foreground">{cr.key}: </span>
                  <span className="line-through text-muted-foreground">{cr.current_value}</span>
                  <span className="mx-1.5 text-muted-foreground">{"\u2192"}</span>
                  <span className="text-primary font-medium">{cr.recommended_value}</span>
                </div>
                <p className="text-xs text-muted-foreground">{cr.reason}</p>
                <p className="text-xs text-muted-foreground">Impact: {cr.impact}</p>
              </div>
            ))}
          </CardContent>
        </Card>
      )}
    </div>
  );
}
