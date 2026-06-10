import { useState, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import Markdown from "react-markdown";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import { templatesApi, tagsApi } from "@/lib/api";
import type { Template, TemplateCategory } from "@acmind/shared";
import {
  TEMPLATE_CATEGORIES,
  TEMPLATE_LANGUAGES,
} from "@acmind/shared";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { toast } from "@/lib/stores/toast";
import { Search, Code2, Star } from "lucide-react";

const CATEGORY_LABELS: Record<string, string> = Object.fromEntries(
  TEMPLATE_CATEGORIES.map((c) => [c.value, c.label])
);

export default function TemplatesListPage() {
  const qc = useQueryClient();
  const navigate = useNavigate();

  const [search, setSearch] = useState("");
  const [categoryFilter, setCategoryFilter] = useState("all");
  const [langFilter, setLangFilter] = useState("all");
  const [sortBy, setSortBy] = useState("created");

  const templates = useQuery({
    queryKey: ["templates", categoryFilter, langFilter, sortBy],
    queryFn: () =>
      templatesApi.list({
        category:
          categoryFilter !== "all"
            ? (categoryFilter as TemplateCategory)
            : undefined,
        language: langFilter !== "all" ? langFilter : undefined,
        sort: sortBy,
      }),
  });

  const tags = useQuery({
    queryKey: ["tags"],
    queryFn: () => tagsApi.list(),
  });

  const remove = useMutation({
    mutationFn: (id: number) => templatesApi.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["templates"] });
      toast.success("模板已删除");
    },
    onError: () => toast.error("删除失败"),
  });

  const filtered = useMemo(() => {
    if (!templates.data) return [];
    if (!search.trim()) return templates.data;
    const q = search.trim().toLowerCase();
    return templates.data.filter(
      (t) =>
        t.title.toLowerCase().includes(q) ||
        t.description.toLowerCase().includes(q)
    );
  }, [templates.data, search]);

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <div className="flex items-center gap-2">
          <Code2 className="h-5 w-5" />
          <CardTitle>模板库</CardTitle>
        </div>
        <Button onClick={() => navigate("/templates/new")}>新建模板</Button>
      </CardHeader>
      <CardContent>
        {/* Filters */}
        <div className="flex flex-wrap gap-3 mb-4">
          <div className="relative flex-1 min-w-48">
            <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="搜索模板名称或描述..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="pl-8"
            />
          </div>
          <Select value={categoryFilter} onValueChange={setCategoryFilter}>
            <SelectTrigger className="w-32">
              <SelectValue placeholder="分类" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部分类</SelectItem>
              {TEMPLATE_CATEGORIES.map((c) => (
                <SelectItem key={c.value} value={c.value}>
                  {c.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Select value={langFilter} onValueChange={setLangFilter}>
            <SelectTrigger className="w-28">
              <SelectValue placeholder="语言" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部语言</SelectItem>
              {TEMPLATE_LANGUAGES.map((l) => (
                <SelectItem key={l} value={l}>
                  {l}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Select value={sortBy} onValueChange={setSortBy}>
            <SelectTrigger className="w-28">
              <SelectValue placeholder="排序" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="created">最新创建</SelectItem>
              <SelectItem value="usage">使用最多</SelectItem>
              <SelectItem value="title">按名称</SelectItem>
            </SelectContent>
          </Select>
        </div>

        {/* Stats count */}
        {templates.data && templates.data.length > 0 && (
          <p className="text-xs text-muted-foreground mb-3">
            共 {filtered.length} 个模板
            {search.trim() && `（筛选自 ${templates.data.length} 个）`}
          </p>
        )}

        {templates.isLoading ? (
          <p>Loading...</p>
        ) : filtered.length > 0 ? (
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
            {filtered.map((t) => (
              <TemplateCard
                key={t.id}
                t={t}
                tags={tags.data ?? []}
                onView={() => navigate(`/templates/${t.id}`)}
                onEdit={() => navigate(`/templates/${t.id}/edit`)}
                onDelete={() => {
                  if (confirm(`确认删除模板 "${t.title}"？`))
                    remove.mutate(t.id);
                }}
              />
            ))}
          </div>
        ) : templates.data && templates.data.length > 0 ? (
          <p className="text-muted-foreground">没有匹配的模板。</p>
        ) : (
          <p className="text-muted-foreground">还没有模板，点击"新建模板"开始。</p>
        )}
      </CardContent>
    </Card>
  );
}

function TemplateCard({
  t,
  tags: allTags,
  onView,
  onEdit,
  onDelete,
}: {
  t: Template;
  tags: { id: number; name: string }[];
  onView: () => void;
  onEdit: () => void;
  onDelete: () => void;
}) {
  const tagNames = t.tag_ids
    .map((id) => allTags.find((tg) => tg.id === id)?.name)
    .filter(Boolean) as string[];

  return (
    <div
      className="rounded-md border border-border p-4 space-y-2 cursor-pointer hover:bg-accent/30 transition-colors"
      onClick={onView}
    >
      <div className="flex items-start justify-between gap-2">
        <div className="min-w-0 flex-1">
          <h3 className="font-medium truncate">{t.title}</h3>
          <div className="flex items-center gap-1.5 mt-0.5 flex-wrap">
            <Badge variant="outline" className="text-xs">
              {CATEGORY_LABELS[t.category] ?? t.category}
            </Badge>
            <Badge variant="secondary" className="text-xs font-mono">
              {t.language}
            </Badge>
            {t.time_complexity && (
              <span className="text-xs text-muted-foreground inline-block">
                <Markdown remarkPlugins={[remarkMath]} rehypePlugins={[rehypeKatex]}>
                  {t.time_complexity}
                </Markdown>
              </span>
            )}
            {t.difficulty != null && (
              <span className="text-xs text-amber-500 flex items-center gap-0.5">
                {Array.from({ length: t.difficulty }).map((_, i) => (
                  <Star key={i} className="h-3 w-3 fill-current" />
                ))}
              </span>
            )}
          </div>
        </div>
        <div
          className="flex gap-1 shrink-0"
          onClick={(e) => e.stopPropagation()}
        >
          <Button size="sm" variant="outline" onClick={onEdit}>
            编辑
          </Button>
          <Button size="sm" variant="destructive" onClick={onDelete}>
            删除
          </Button>
        </div>
      </div>
      {t.description && (
        <div className="text-sm text-muted-foreground line-clamp-2 knowledge-content">
          <Markdown remarkPlugins={[remarkMath]} rehypePlugins={[rehypeKatex]}>
            {t.description}
          </Markdown>
        </div>
      )}
      <div className="flex items-center justify-between">
        <div className="flex flex-wrap gap-1">
          {tagNames.slice(0, 3).map((n) => (
            <Badge key={n} variant="secondary" className="text-xs">
              {n}
            </Badge>
          ))}
        </div>
        <span className="text-xs text-muted-foreground">
          {t.usage_count > 0 && `使用 ${t.usage_count} 次`}
        </span>
      </div>
    </div>
  );
}
