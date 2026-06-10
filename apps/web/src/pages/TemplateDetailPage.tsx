import { useNavigate, useParams } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import Markdown from "react-markdown";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import { templatesApi, tagsApi, problemsApi } from "@/lib/api";
import { TEMPLATE_CATEGORIES } from "@acmind/shared";
import {
  Card,
  CardContent,
  CardHeader,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { toast } from "@/lib/stores/toast";
import {
  ArrowLeft,
  Star,
  Copy,
  ExternalLink,
  Clock,
  Cpu,
  Database,
} from "lucide-react";

const CATEGORY_LABELS: Record<string, string> = Object.fromEntries(
  TEMPLATE_CATEGORIES.map((c) => [c.value, c.label])
);

const SOURCE_LABELS: Record<string, string> = {
  manual: "手动创建",
  ai_extracted: "AI 提取",
};

export default function TemplateDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const qc = useQueryClient();
  const tid = Number(id);

  const template = useQuery({
    queryKey: ["templates", tid],
    queryFn: () => templatesApi.get(tid),
    enabled: !!id,
  });

  const tags = useQuery({
    queryKey: ["tags"],
    queryFn: () => tagsApi.list(),
  });

  const problems = useQuery({
    queryKey: ["problems"],
    queryFn: () => problemsApi.list(),
  });

  const remove = useMutation({
    mutationFn: () => templatesApi.delete(tid),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["templates"] });
      toast.success("模板已删除");
      navigate("/templates");
    },
    onError: () => toast.error("删除失败"),
  });

  const t = template.data;
  const tagNames =
    t?.tag_ids
      .map((id) => tags.data?.find((tg) => tg.id === id)?.name)
      .filter(Boolean) ?? [];

  const linkedProblems =
    t?.problem_ids
      .map((id) => problems.data?.find((p) => p.id === id))
      .filter(Boolean) ?? [];

  const handleCopyCode = () => {
    if (t) {
      navigator.clipboard.writeText(t.code);
      toast.success("代码已复制到剪贴板");
    }
  };

  if (template.isLoading) {
    return (
      <Card>
        <CardContent className="p-6">
          <p>Loading...</p>
        </CardContent>
      </Card>
    );
  }

  if (!t) {
    return (
      <Card>
        <CardContent className="p-6">
          <p className="text-muted-foreground">模板不存在。</p>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-4">
      {/* Header */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => navigate("/templates")}
              >
                <ArrowLeft className="h-4 w-4" />
              </Button>
              <h2 className="text-lg font-semibold">{t.title}</h2>
            </div>
            <div className="flex gap-2">
              <Button
                variant="outline"
                onClick={() => navigate(`/templates/${t.id}/edit`)}
              >
                编辑
              </Button>
              <Button
                variant="destructive"
                onClick={() => {
                  if (confirm(`确认删除模板 "${t.title}"？`))
                    remove.mutate();
                }}
              >
                删除
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap items-center gap-2 mb-3">
            <Badge variant="outline">
              {CATEGORY_LABELS[t.category] ?? t.category}
            </Badge>
            <Badge variant="secondary" className="font-mono">
              {t.language}
            </Badge>
            {t.difficulty != null && (
              <span className="text-amber-500 flex items-center gap-0.5">
                {Array.from({ length: t.difficulty }).map((_, i) => (
                  <Star key={i} className="h-4 w-4 fill-current" />
                ))}
              </span>
            )}
            <span className="text-xs text-muted-foreground">
              来源: {SOURCE_LABELS[t.source] ?? t.source}
            </span>
          </div>

          {/* Meta row */}
          <div className="flex flex-wrap gap-4 text-sm text-muted-foreground mb-3">
            {t.time_complexity && (
              <span className="flex items-center gap-1">
                <Clock className="h-3.5 w-3.5" />
                时间: <span className="inline-block"><Markdown remarkPlugins={[remarkMath]} rehypePlugins={[rehypeKatex]}>{t.time_complexity}</Markdown></span>
              </span>
            )}
            {t.space_complexity && (
              <span className="flex items-center gap-1">
                <Cpu className="h-3.5 w-3.5" />
                空间: <span className="inline-block"><Markdown remarkPlugins={[remarkMath]} rehypePlugins={[rehypeKatex]}>{t.space_complexity}</Markdown></span>
              </span>
            )}
            {t.usage_count > 0 && (
              <span className="flex items-center gap-1">
                <Database className="h-3.5 w-3.5" />
                使用 {t.usage_count} 次
              </span>
            )}
          </div>

          {/* Tags */}
          {tagNames.length > 0 && (
            <div className="flex flex-wrap gap-1">
              {tagNames.map((n) => (
                <Badge key={n} variant="secondary" className="text-xs">
                  {n}
                </Badge>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Description */}
      {t.description && (
        <Card>
          <CardContent className="p-4">
            <h3 className="text-sm font-medium mb-2">📝 使用说明</h3>
            <div className="text-sm text-muted-foreground knowledge-content">
              <Markdown remarkPlugins={[remarkMath]} rehypePlugins={[rehypeKatex]}>
                {t.description}
              </Markdown>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Code */}
      <Card>
        <CardContent className="p-4">
          <div className="flex items-center justify-between mb-2">
            <h3 className="text-sm font-medium">💻 代码</h3>
            <Button variant="outline" size="sm" onClick={handleCopyCode}>
              <Copy className="h-3.5 w-3.5 mr-1" />
              复制代码
            </Button>
          </div>
          <pre className="bg-muted rounded-md p-4 overflow-x-auto text-sm">
            <code>{t.code}</code>
          </pre>
        </CardContent>
      </Card>

      {/* Linked Problems */}
      <Card>
        <CardContent className="p-4">
          <h3 className="text-sm font-medium mb-2">
            🔗 关联题目 ({linkedProblems.length})
          </h3>
          {linkedProblems.length > 0 ? (
            <ul className="space-y-1">
              {linkedProblems.map(
                (p) =>
                  p && (
                    <li key={p.id}>
                      <button
                        onClick={() => navigate(`/problems/${p.id}`)}
                        className="text-sm text-blue-600 hover:underline flex items-center gap-1"
                      >
                        <ExternalLink className="h-3 w-3" />
                        {p.title}
                        {p.source && (
                          <span className="text-muted-foreground">
                            ({p.source})
                          </span>
                        )}
                      </button>
                    </li>
                  )
              )}
            </ul>
          ) : (
            <p className="text-sm text-muted-foreground">暂无关联题目。</p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
