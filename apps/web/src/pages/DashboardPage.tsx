import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { analysisApi, problemsApi, submissionsApi, knowledgeApi } from "@/lib/api";

export default function DashboardPage() {
  const summary = useQuery({ queryKey: ["summary"], queryFn: () => analysisApi.summary() });
  const problems = useQuery({ queryKey: ["problems"], queryFn: () => problemsApi.list() });
  const submissions = useQuery({
    queryKey: ["submissions"],
    queryFn: () => submissionsApi.list(),
  });
  const knowledge = useQuery({ queryKey: ["knowledge"], queryFn: () => knowledgeApi.list() });

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold">Dashboard</h1>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          title="Problems"
          value={problems.data?.length ?? 0}
          to="/problems"
          loading={problems.isLoading}
        />
        <StatCard
          title="Submissions"
          value={submissions.data?.length ?? 0}
          to="/submissions"
          loading={submissions.isLoading}
        />
        <StatCard
          title="Knowledge"
          value={knowledge.data?.length ?? 0}
          to="/knowledge"
          loading={knowledge.isLoading}
        />
        <StatCard
          title="AC rate"
          value={summary.data ? `${(summary.data.ac_rate * 100).toFixed(0)}%` : "—"}
          to="/analysis"
          loading={summary.isLoading}
        />
      </div>
    </div>
  );
}

function StatCard({
  title,
  value,
  to,
  loading,
}: {
  title: string;
  value: string | number;
  to: string;
  loading: boolean;
}) {
  return (
    <Link to={to}>
      <Card className="hover:bg-[var(--color-accent)] transition-colors">
        <CardHeader>
          <CardTitle className="text-sm text-[var(--color-muted-foreground)]">{title}</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-3xl font-bold">{loading ? "…" : value}</p>
        </CardContent>
      </Card>
    </Link>
  );
}
