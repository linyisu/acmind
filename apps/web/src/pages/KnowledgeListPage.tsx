import { useState, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import Markdown from "react-markdown";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import { knowledgeApi, problemsApi, tagsApi } from "@/lib/api";
import type { Knowledge, KnowledgeKind, Tag } from "@acmind/shared";
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
import { Search } from "lucide-react";

const KINDS: KnowledgeKind[] = ["template", "technique", "note", "snippet"];
const KIND_LABELS: Record<string, string> = {
  template: "模板",
  technique: "技巧",
  note: "笔记",
  snippet: "代码片段",
};

export default function KnowledgeListPage() {
  const qc = useQueryClient();
  const navigate = useNavigate();
  const knowledge = useQuery({ queryKey: ["knowledge"], queryFn: () => knowledgeApi.list() });
  const tags = useQuery({ queryKey: ["tags"], queryFn: () => tagsApi.list() });
  const problems = useQuery({ queryKey: ["problems"], queryFn: () => problemsApi.list() });

  const [search, setSearch] = useState("");
  const [kindFilter, setKindFilter] = useState<string>("all");
  const [tagFilter, setTagFilter] = useState<string>("all");

  const remove = useMutation({
    mutationFn: (id: number) => knowledgeApi.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["knowledge"] });
      toast.success("知识条目已删除");
    },
    onError: () => toast.error("删除失败"),
  });

  const filtered = useMemo(() => {
    if (!knowledge.data) return [];
    let list = knowledge.data;

    if (kindFilter !== "all") {
      list = list.filter((k) => k.kind === kindFilter);
    }

    if (tagFilter !== "all") {
      const tid = Number(tagFilter);
      list = list.filter((k) => k.tag_ids.includes(tid));
    }

    if (search.trim()) {
      const q = search.trim().toLowerCase();
      list = list.filter(
        (k) =>
          k.title.toLowerCase().includes(q) ||
          k.content.toLowerCase().includes(q),
      );
    }

    return list;
  }, [knowledge.data, kindFilter, tagFilter, search]);

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle>知识库</CardTitle>
        <Button onClick={() => navigate("/knowledge/new")}>新建</Button>
      </CardHeader>
      <CardContent>
        {/* Filters */}
        <div className="flex flex-wrap gap-3 mb-4">
          <div className="relative flex-1 min-w-48">
            <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="搜索标题或内容…"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="pl-8"
            />
          </div>
          <Select value={kindFilter} onValueChange={setKindFilter}>
            <SelectTrigger className="w-32">
              <SelectValue placeholder="Kind" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部类型</SelectItem>
              {KINDS.map((k) => (
                <SelectItem key={k} value={k}>
                  {KIND_LABELS[k] ?? k}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Select value={tagFilter} onValueChange={setTagFilter}>
            <SelectTrigger className="w-36">
              <SelectValue placeholder="Tag" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部标签</SelectItem>
              {tags.data?.map((t) => (
                <SelectItem key={t.id} value={String(t.id)}>
                  {t.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        {/* Results count */}
        {knowledge.data && knowledge.data.length > 0 && (
          <p className="text-xs text-muted-foreground mb-3">
            显示 {filtered.length} / {knowledge.data.length} 条
          </p>
        )}

        {knowledge.isLoading ? (
          <p>加载中…</p>
        ) : filtered.length > 0 ? (
          <div className="space-y-2">
            {filtered.map((k) => (
              <KnowledgeRow
                key={k.id}
                k={k}
                tags={tags.data ?? []}
                problems={problems.data ?? []}
                onView={() => navigate(`/knowledge/${k.id}`)}
                onEdit={() => navigate(`/knowledge/${k.id}/edit`)}
                onDelete={() => {
                  if (confirm(`确认删除「${k.title}」？`)) remove.mutate(k.id);
                }}
              />
            ))}
          </div>
        ) : knowledge.data && knowledge.data.length > 0 ? (
          <p className="text-muted-foreground">没有匹配的知识条目。</p>
        ) : (
          <p className="text-muted-foreground">还没有知识条目，点击「新建」开始。</p>
        )}
      </CardContent>
    </Card>
  );
}

function KnowledgeRow({
  k,
  tags,
  problems,
  onView,
  onEdit,
  onDelete,
}: {
  k: Knowledge;
  tags: Tag[];
  problems: { id: number; title: string }[];
  onView: () => void;
  onEdit: () => void;
  onDelete: () => void;
}) {
  const tagNames = k.tag_ids
    .map((id) => tags.find((t) => t.id === id)?.name)
    .filter(Boolean) as string[];
  const linked = k.problem_id
    ? problems.find((p) => p.id === k.problem_id)?.title
    : null;

  // No truncation here — render full content as Markdown with line-clamp via CSS

  return (
    <div
      className="rounded-md border border-border p-4 space-y-2 cursor-pointer hover:bg-accent/30 transition-colors"
      onClick={onView}
    >
      <div className="flex items-start justify-between gap-2">
        <div className="min-w-0">
          <h3 className="font-medium">{k.title}</h3>
          <div className="flex items-center gap-2 mt-0.5">
            <Badge variant="outline" className="text-xs">
              {KIND_LABELS[k.kind] ?? k.kind}
            </Badge>
            {linked && (
              <span className="text-xs text-muted-foreground">↳ {linked}</span>
            )}
            <span className="text-xs text-muted-foreground">
              {new Date(k.updated_at).toLocaleDateString()}
            </span>
          </div>
        </div>
        <div className="flex gap-1 shrink-0" onClick={(e) => e.stopPropagation()}>
          <Button size="sm" variant="outline" onClick={onEdit}>
            Edit
          </Button>
          <Button size="sm" variant="destructive" onClick={onDelete}>
            Delete
          </Button>
        </div>
      </div>
      <div className="text-sm text-muted-foreground line-clamp-2 knowledge-content">
        <Markdown remarkPlugins={[remarkMath]} rehypePlugins={[rehypeKatex]}>
          {k.content}
        </Markdown>
      </div>
      {tagNames.length > 0 && (
        <div className="flex flex-wrap gap-1">
          {tagNames.map((n) => (
            <Badge key={n} variant="secondary" className="text-xs">
              {n}
            </Badge>
          ))}
        </div>
      )}
    </div>
  );
}
