import { useQuery } from "@tanstack/react-query";
import {
  Bar,
  BarChart,
  CartesianGrid,
  Legend,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { analysisApi } from "@/lib/api";

export default function AnalysisPage() {
  const summary = useQuery({ queryKey: ["summary"], queryFn: () => analysisApi.summary() });
  const timeline = useQuery({ queryKey: ["timeline"], queryFn: () => analysisApi.timeline() });
  const diff = useQuery({
    queryKey: ["difficulty-distribution"],
    queryFn: () => analysisApi.difficultyDistribution(),
  });

  const verdictData = summary.data
    ? Object.entries(summary.data.by_verdict).map(([name, value]) => ({ name, value }))
    : [];

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold">数据分析</h1>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card>
          <CardHeader>
            <CardDescription>提交总数</CardDescription>
            <CardTitle className="text-3xl">
              {summary.isLoading ? "…" : summary.data?.total ?? 0}
            </CardTitle>
          </CardHeader>
        </Card>
        <Card>
          <CardHeader>
            <CardDescription>AC 率</CardDescription>
            <CardTitle className="text-3xl">
              {summary.isLoading
                ? "…"
                : summary.data
                ? `${(summary.data.ac_rate * 100).toFixed(1)}%`
                : "—"}
            </CardTitle>
          </CardHeader>
        </Card>
        <Card>
          <CardHeader>
            <CardDescription>判题结果种类</CardDescription>
            <CardTitle className="text-3xl">
              {summary.data ? Object.keys(summary.data.by_verdict).length : 0}
            </CardTitle>
          </CardHeader>
        </Card>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <Card>
          <CardHeader>
            <CardTitle>判题结果分布</CardTitle>
            <CardDescription>按结果类型分组统计</CardDescription>
          </CardHeader>
          <CardContent>
            {verdictData.length === 0 ? (
              <p className="text-muted-foreground">暂无数据。</p>
            ) : (
              <ResponsiveContainer width="100%" height={280}>
                <BarChart data={verdictData}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis dataKey="name" />
                  <YAxis allowDecimals={false} />
                  <Tooltip />
                  <Legend />
                  <Bar dataKey="value" fill="var(--primary)" />
                </BarChart>
              </ResponsiveContainer>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>提交时间线</CardTitle>
            <CardDescription>每日提交数量</CardDescription>
          </CardHeader>
          <CardContent>
            {timeline.isLoading ? (
              <p>Loading…</p>
            ) : timeline.data && timeline.data.length > 0 ? (
              <ResponsiveContainer width="100%" height={280}>
                <LineChart data={timeline.data}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis dataKey="date" />
                  <YAxis allowDecimals={false} />
                  <Tooltip />
                  <Legend />
                  <Line type="monotone" dataKey="count" stroke="var(--primary)" />
                  <Line type="monotone" dataKey="ac_count" stroke="#22c55e" />
                </LineChart>
              </ResponsiveContainer>
            ) : (
              <p className="text-muted-foreground">暂无数据。</p>
            )}
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>难度分布</CardTitle>
          <CardDescription>按题目难度分组统计</CardDescription>
        </CardHeader>
        <CardContent>
          {diff.isLoading ? (
            <p>加载中…</p>
          ) : diff.data && diff.data.length > 0 ? (
            <ResponsiveContainer width="100%" height={280}>
              <BarChart data={diff.data}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="difficulty" />
                <YAxis allowDecimals={false} />
                <Tooltip />
                <Legend />
                <Bar dataKey="count" fill="var(--primary)" />
                <Bar dataKey="ac_count" fill="#22c55e" />
              </BarChart>
            </ResponsiveContainer>
          ) : (
            <p className="text-muted-foreground">No data yet.</p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
