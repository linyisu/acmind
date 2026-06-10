import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { motion } from "motion/react";
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

  const cards = [
    { title: "题目", value: problems.data?.length ?? 0, to: "/problems", loading: problems.isLoading },
    { title: "提交", value: submissions.data?.length ?? 0, to: "/problems", loading: submissions.isLoading },
    { title: "知识条目", value: knowledge.data?.length ?? 0, to: "/knowledge", loading: knowledge.isLoading },
    {
      title: "AC 率",
      value: summary.data ? `${(summary.data.ac_rate * 100).toFixed(0)}%` : "—",
      to: "/analysis",
      loading: summary.isLoading,
    },
  ];

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold">仪表盘</h1>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {cards.map((c, i) => (
          <StatCard key={c.title} index={i} {...c} />
        ))}
      </div>
    </div>
  );
}

function StatCard({
  title,
  value,
  to,
  loading,
  index,
}: {
  title: string;
  value: string | number;
  to: string;
  loading: boolean;
  index: number;
}) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.28, delay: index * 0.05, ease: "easeOut" }}
      whileHover={{ y: -2 }}
      whileTap={{ scale: 0.98 }}
    >
      <Link to={to}>
        <Card className="hover:bg-accent transition-colors">
          <CardHeader>
            <CardTitle className="text-sm text-muted-foreground">{title}</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-3xl font-bold">{loading ? "…" : value}</p>
          </CardContent>
        </Card>
      </Link>
    </motion.div>
  );
}
