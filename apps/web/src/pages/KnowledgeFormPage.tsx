import { useState, useEffect } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { knowledgeApi, problemsApi, tagsApi } from "@/lib/api";
import type { KnowledgeKind } from "@acmind/shared";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { ApiError } from "@/lib/api/client";

const KINDS: KnowledgeKind[] = ["template", "technique", "note", "snippet"];

export default function KnowledgeFormPage() {
  const { id } = useParams<{ id?: string }>();
  const isEdit = id !== undefined;
  const navigate = useNavigate();
  const qc = useQueryClient();

  const [title, setTitle] = useState("");
  const [kind, setKind] = useState<KnowledgeKind>("note");
  const [content, setContent] = useState("");
  const [problemId, setProblemId] = useState<string>("none");
  const [tagInput, setTagInput] = useState("");
  const [tagIds, setTagIds] = useState<number[]>([]);
  const [error, setError] = useState<string | null>(null);

  const tags = useQuery({ queryKey: ["tags"], queryFn: () => tagsApi.list() });
  const problems = useQuery({ queryKey: ["problems"], queryFn: () => problemsApi.list() });
  const knowledge = useQuery({
    queryKey: ["knowledge", Number(id)],
    queryFn: () => knowledgeApi.get(Number(id!)),
    enabled: isEdit,
  });

  useEffect(() => {
    if (knowledge.data) {
      const k = knowledge.data;
      setTitle(k.title);
      setKind(k.kind);
      setContent(k.content);
      setProblemId(k.problem_id ? String(k.problem_id) : "none");
      setTagIds(k.tag_ids);
    }
  }, [knowledge.data]);

  const create = useMutation({
    mutationFn: () =>
      knowledgeApi.create({
        title,
        kind,
        content,
        problem_id: problemId === "none" ? undefined : Number(problemId),
        tag_ids: tagIds,
      }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["knowledge"] });
      navigate("/knowledge");
    },
    onError: (e) =>
      setError(e instanceof ApiError ? e.message : "Failed to create entry"),
  });

  const update = useMutation({
    mutationFn: () =>
      knowledgeApi.update(Number(id), {
        title,
        kind,
        content,
        problem_id: problemId === "none" ? undefined : Number(problemId),
        tag_ids: tagIds,
      }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["knowledge"] });
      navigate(`/knowledge/${id}`);
    },
    onError: (e) =>
      setError(e instanceof ApiError ? e.message : "更新失败"),
  });

  function addTag() {
    const name = tagInput.trim();
    if (!name) return;
    const existing = tags.data?.find((t) => t.name === name);
    if (existing) {
      if (!tagIds.includes(existing.id)) setTagIds([...tagIds, existing.id]);
    } else {
      tagsApi
        .create({ name })
        .then((t) => {
          qc.invalidateQueries({ queryKey: ["tags"] });
          setTagIds([...tagIds, t.id]);
        })
        .catch((e) => setError(e instanceof ApiError ? e.message : "创建标签失败"));
    }
    setTagInput("");
  }

  function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    if (isEdit) update.mutate();
    else create.mutate();
  }

  const busy = create.isPending || update.isPending;

  return (
    <Card className="max-w-3xl">
      <CardHeader>
        <CardTitle>{isEdit ? "编辑知识" : "新建知识"}</CardTitle>
      </CardHeader>
      <CardContent>
        <form onSubmit={onSubmit} className="space-y-4">
          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-1.5">
              <Label>标题</Label>
              <Input value={title} onChange={(e) => setTitle(e.target.value)} required />
            </div>
            <div className="space-y-1.5">
              <Label>类型</Label>
              <Select value={kind} onValueChange={(v) => setKind(v as KnowledgeKind)}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {KINDS.map((k) => (
                    <SelectItem key={k} value={k}>
                      {k}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
          <div className="space-y-1.5">
            <Label>关联题目（可选）</Label>
            <Select value={problemId} onValueChange={setProblemId}>
              <SelectTrigger>
                <SelectValue placeholder="无" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="none">— 无 —</SelectItem>
                {problems.data?.map((p) => (
                  <SelectItem key={p.id} value={String(p.id)}>
                    {p.title}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-1.5">
            <Label>内容（支持 Markdown）</Label>
            <textarea
              className="flex min-h-48 w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring font-mono"
              value={content}
              onChange={(e) => setContent(e.target.value)}
              required
            />
          </div>
          <div className="space-y-1.5">
            <Label>标签</Label>
            <div className="flex gap-2">
              <Input
                value={tagInput}
                onChange={(e) => setTagInput(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    addTag();
                  }
                }}
                placeholder="输入标签名后按回车"
              />
              <Button type="button" variant="outline" onClick={addTag}>
                添加
              </Button>
            </div>
            <div className="flex flex-wrap gap-1 mt-2">
              {tagIds.map((tid) => {
                const t = tags.data?.find((x) => x.id === tid);
                return (
                  <span
                    key={tid}
                    className="inline-flex items-center rounded-md bg-accent px-2 py-0.5 text-xs"
                  >
                    {t?.name ?? tid}
                    <button
                      type="button"
                      className="ml-1 opacity-70 hover:opacity-100"
                      onClick={() => setTagIds(tagIds.filter((x) => x !== tid))}
                    >
                      ×
                    </button>
                  </span>
                );
              })}
            </div>
          </div>
          {error && <p className="text-sm text-destructive">{error}</p>}
          <div className="flex justify-end gap-2">
            <Button
              type="button"
              variant="ghost"
              onClick={() => navigate(isEdit ? `/knowledge/${id}` : "/knowledge")}
            >
              取消
            </Button>
            <Button type="submit" disabled={busy}>
              {busy ? "保存中…" : isEdit ? "保存" : "创建"}
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}
