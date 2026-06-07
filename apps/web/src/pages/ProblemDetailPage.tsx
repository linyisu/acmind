import { useParams, useNavigate } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { problemsApi, submissionsApi, tagsApi } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

const VERDICT_COLORS: Record<string, "default" | "destructive" | "secondary" | "outline"> = {
  AC: "default",
  WA: "destructive",
  TLE: "destructive",
  RE: "destructive",
  CE: "destructive",
  PENDING: "secondary",
};

export default function ProblemDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const problem = useQuery({
    queryKey: ["problems", Number(id)],
    queryFn: () => problemsApi.get(Number(id!)),
    enabled: !!id,
  });

  const submissions = useQuery({
    queryKey: ["submissions", Number(id)],
    queryFn: () => submissionsApi.list(Number(id!)),
    enabled: !!id,
  });

  const tags = useQuery({ queryKey: ["tags"], queryFn: () => tagsApi.list() });

  if (problem.isLoading) {
    return <p className="text-muted-foreground">Loading…</p>;
  }

  if (!problem.data) {
    return <p className="text-muted-foreground">Problem not found.</p>;
  }

  const p = problem.data;
  const tagNames = p.tag_ids
    .map((tid) => tags.data?.find((t) => t.id === tid)?.name)
    .filter(Boolean) as string[];

  return (
    <div className="space-y-6 max-w-3xl">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold">{p.title}</h1>
          <div className="flex items-center gap-2 mt-1 text-sm text-muted-foreground">
            <span>{p.source}</span>
            {p.external_id && <span>· {p.external_id}</span>}
            {p.difficulty != null && (
              <>
                <span>·</span>
                <Badge variant="outline">Difficulty {p.difficulty}</Badge>
              </>
            )}
          </div>
        </div>
        <Button variant="outline" onClick={() => navigate(`/problems/${id}/edit`)}>
          Edit
        </Button>
      </div>

      {tagNames.length > 0 && (
        <div className="flex flex-wrap gap-1">
          {tagNames.map((n) => (
            <Badge key={n} variant="secondary">
              {n}
            </Badge>
          ))}
        </div>
      )}

      {p.url && (
        <a
          href={p.url}
          target="_blank"
          rel="noopener noreferrer"
          className="text-sm text-primary underline"
        >
          View on {p.source} →
        </a>
      )}

      {p.statement && (
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Statement</CardTitle>
          </CardHeader>
          <CardContent>
            <pre className="whitespace-pre-wrap text-sm leading-relaxed">{p.statement}</pre>
          </CardContent>
        </Card>
      )}

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Submissions</CardTitle>
        </CardHeader>
        <CardContent>
          {submissions.isLoading ? (
            <p className="text-muted-foreground">Loading…</p>
          ) : submissions.data && submissions.data.length > 0 ? (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>When</TableHead>
                  <TableHead>Language</TableHead>
                  <TableHead>Verdict</TableHead>
                  <TableHead>Runtime</TableHead>
                  <TableHead>Memory</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {submissions.data.map((s) => (
                  <TableRow key={s.id}>
                    <TableCell className="text-xs text-muted-foreground">
                      {new Date(s.submitted_at).toLocaleString()}
                    </TableCell>
                    <TableCell>{s.language}</TableCell>
                    <TableCell>
                      <Badge variant={VERDICT_COLORS[s.verdict] ?? "outline"}>{s.verdict}</Badge>
                    </TableCell>
                    <TableCell>{s.runtime_ms ?? "—"} ms</TableCell>
                    <TableCell>{s.memory_kb ?? "—"} KB</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          ) : (
            <p className="text-muted-foreground">No submissions yet.</p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
