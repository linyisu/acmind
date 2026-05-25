import { useQuery } from "@tanstack/react-query";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { BookOpen, Hash } from "lucide-react";

interface KnowledgePoint {
  id: string;
  name: string;
  category: string;
  parent_id?: string;
}

async function api<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<T>(cmd, args);
  } catch {
    return [] as unknown as T;
  }
}

const categoryColors: Record<string, "success" | "warning" | "error" | "secondary" | "default"> = {
  DP: "success",
  Graph: "warning",
  Math: "error",
  DS: "secondary",
  String: "default",
  Greedy: "success",
  Geometry: "error",
  Search: "warning",
  Other: "secondary",
};

export function KnowledgePage() {
  const { data: points, isLoading } = useQuery({
    queryKey: ["knowledge-points"],
    queryFn: () => api<KnowledgePoint[]>("list_knowledge_points"),
  });

  // Group by category
  const grouped = (points ?? []).reduce(
    (acc, p) => {
      if (!acc[p.category]) acc[p.category] = [];
      acc[p.category].push(p);
      return acc;
    },
    {} as Record<string, KnowledgePoint[]>,
  );

  const categoryOrder = [
    "DP",
    "Graph",
    "Math",
    "DS",
    "String",
    "Greedy",
    "Geometry",
    "Search",
    "Other",
  ];

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Knowledge Map</h1>
        <p className="text-muted-foreground">
          Algorithm and data structure knowledge points tracked by ACMind
        </p>
      </div>

      {isLoading ? (
        <div className="grid gap-4 md:grid-cols-2">
          {Array.from({ length: 6 }).map((_, i) => (
            <Skeleton key={i} className="h-32 w-full" />
          ))}
        </div>
      ) : (
        <div className="grid gap-4 md:grid-cols-2">
          {categoryOrder.map((cat) => {
            const items = grouped[cat] ?? [];
            return (
              <Card key={cat}>
                <CardHeader>
                  <CardTitle className="text-sm flex items-center gap-2">
                    <BookOpen className="h-4 w-4" />
                    <Badge variant={categoryColors[cat] ?? "secondary"}>
                      {cat}
                    </Badge>
                    <span className="text-muted-foreground font-normal text-xs">
                      {items.length} topics
                    </span>
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  {items.length > 0 ? (
                    <div className="flex flex-wrap gap-1.5">
                      {items.map((kp) => (
                        <span
                          key={kp.id}
                          className="inline-flex items-center gap-1 rounded-md border px-2 py-1 text-xs"
                        >
                          <Hash className="h-3 w-3 text-muted-foreground" />
                          {kp.name}
                        </span>
                      ))}
                    </div>
                  ) : (
                    <p className="text-xs text-muted-foreground">
                      No topics yet in this category
                    </p>
                  )}
                </CardContent>
              </Card>
            );
          })}
        </div>
      )}

      {/* Info */}
      <Card className="border-dashed">
        <CardHeader>
          <CardTitle className="text-sm">About Knowledge Points</CardTitle>
        </CardHeader>
        <CardContent className="text-sm text-muted-foreground space-y-2">
          <p>
            Knowledge points are automatically assigned to problems through AI
            analysis. They help you track which algorithm patterns you've
            practiced and where you need more work.
          </p>
          <p>
            <strong>Tip:</strong> Use AI analysis on your problems to
            automatically extract relevant knowledge points from your
            submissions.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
