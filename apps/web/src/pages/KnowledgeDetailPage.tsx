import { useParams, useNavigate } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import Markdown from "react-markdown";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import { knowledgeApi, problemsApi, tagsApi } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Link } from "react-router-dom";

const KIND_LABELS: Record<string, string> = {
  template: "模板",
  technique: "技巧",
  note: "笔记",
  snippet: "代码片段",
};

export default function KnowledgeDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const knowledge = useQuery({
    queryKey: ["knowledge", Number(id)],
    queryFn: () => knowledgeApi.get(Number(id!)),
    enabled: !!id,
  });

  const tags = useQuery({ queryKey: ["tags"], queryFn: () => tagsApi.list() });

  const problems = useQuery({
    queryKey: ["problems"],
    queryFn: () => problemsApi.list(),
  });

  if (knowledge.isLoading) {
    return <p className="text-muted-foreground">加载中…</p>;
  }

  if (!knowledge.data) {
    return <p className="text-muted-foreground">知识条目不存在。</p>;
  }

  const k = knowledge.data;
  const tagNames = k.tag_ids
    .map((tid) => tags.data?.find((t) => t.id === tid)?.name)
    .filter(Boolean) as string[];
  const linked = k.problem_id
    ? problems.data?.find((p) => p.id === k.problem_id)
    : null;

  return (
    <div className="space-y-6 max-w-4xl">
      {/* Header */}
      <div className="flex items-start justify-between gap-4">
        <div>
          <div className="flex items-center gap-2">
            <h1 className="text-2xl font-bold">{k.title}</h1>
            <Badge variant="outline">{KIND_LABELS[k.kind] ?? k.kind}</Badge>
          </div>
          <p className="text-sm text-muted-foreground mt-1">
            更新于 {new Date(k.updated_at).toLocaleString()}
          </p>
        </div>
        <Button variant="outline" onClick={() => navigate(`/knowledge/${id}/edit`)}>
          编辑
        </Button>
      </div>

      {/* Tags */}
      {tagNames.length > 0 && (
        <div className="flex flex-wrap gap-1">
          {tagNames.map((n) => (
            <Badge key={n} variant="secondary">
              {n}
            </Badge>
          ))}
        </div>
      )}

      {/* Linked problem */}
      {linked && (
        <Card>
          <CardContent className="py-3">
            <span className="text-sm text-muted-foreground">关联题目: </span>
            <Link
              to={`/problems/${linked.id}`}
              className="text-sm text-primary hover:underline"
            >
              {linked.title}
            </Link>
          </CardContent>
        </Card>
      )}

      {/* Content */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">内容</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="knowledge-content prose prose-invert max-w-none">
            <Markdown remarkPlugins={[remarkMath]} rehypePlugins={[rehypeKatex]}>
              {k.content}
            </Markdown>
          </div>
        </CardContent>
      </Card>

      {/* Back link */}
      <Button variant="ghost" onClick={() => navigate("/knowledge")}>
        ← 返回知识库
      </Button>
    </div>
  );
}
