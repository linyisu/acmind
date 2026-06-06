import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
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
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

const KINDS: KnowledgeKind[] = ["template", "technique", "note", "snippet"];

export default function KnowledgeListPage() {
  const qc = useQueryClient();
  const knowledge = useQuery({ queryKey: ["knowledge"], queryFn: () => knowledgeApi.list() });
  const problems = useQuery({ queryKey: ["problems"], queryFn: () => problemsApi.list() });
  const tags = useQuery({ queryKey: ["tags"], queryFn: () => tagsApi.list() });

  const [open, setOpen] = useState(false);
  const [title, setTitle] = useState("");
  const [kind, setKind] = useState<KnowledgeKind>("note");
  const [content, setContent] = useState("");
  const [problemId, setProblemId] = useState<string>("none");
  const [tagInput, setTagInput] = useState("");
  const [tagIds, setTagIds] = useState<number[]>([]);

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
      setOpen(false);
      setTitle("");
      setContent("");
      setProblemId("none");
      setTagInput("");
      setTagIds([]);
    },
  });

  const remove = useMutation({
    mutationFn: (id: number) => knowledgeApi.delete(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["knowledge"] }),
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
        .catch(() => {});
    }
    setTagInput("");
  }

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle>Knowledge</CardTitle>
        <Dialog open={open} onOpenChange={setOpen}>
          <DialogTrigger asChild>
            <Button>New</Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>New knowledge</DialogTitle>
              <DialogDescription>Templates, techniques, notes, snippets.</DialogDescription>
            </DialogHeader>
            <form
              onSubmit={(e) => {
                e.preventDefault();
                if (title && content) create.mutate();
              }}
              className="space-y-3"
            >
              <div className="space-y-1.5">
                <Label>Title</Label>
                <Input value={title} onChange={(e) => setTitle(e.target.value)} required />
              </div>
              <div className="space-y-1.5">
                <Label>Kind</Label>
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
              <div className="space-y-1.5">
                <Label>Linked problem (optional)</Label>
                <Select value={problemId} onValueChange={setProblemId}>
                  <SelectTrigger>
                    <SelectValue placeholder="None" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="none">— none —</SelectItem>
                    {problems.data?.map((p) => (
                      <SelectItem key={p.id} value={String(p.id)}>
                        {p.title}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-1.5">
                <Label>Content</Label>
                <textarea
                  className="flex min-h-32 w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  value={content}
                  onChange={(e) => setContent(e.target.value)}
                  required
                />
              </div>
              <div className="space-y-1.5">
                <Label>Tags</Label>
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
                    placeholder="Type and press Enter"
                  />
                  <Button type="button" variant="outline" onClick={addTag}>
                    Add
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
              <DialogFooter>
                <Button type="button" variant="ghost" onClick={() => setOpen(false)}>
                  Cancel
                </Button>
                <Button type="submit" disabled={create.isPending || !title || !content}>
                  {create.isPending ? "Creating…" : "Create"}
                </Button>
              </DialogFooter>
            </form>
          </DialogContent>
        </Dialog>
      </CardHeader>
      <CardContent>
        {knowledge.isLoading ? (
          <p>Loading…</p>
        ) : knowledge.data && knowledge.data.length > 0 ? (
          <div className="space-y-2">
            {knowledge.data.map((k) => (
              <KnowledgeRow
                key={k.id}
                k={k}
                tags={tags.data ?? []}
                problems={problems.data ?? []}
                onDelete={() => {
                  if (confirm(`Delete "${k.title}"?`)) remove.mutate(k.id);
                }}
              />
            ))}
          </div>
        ) : (
          <p className="text-muted-foreground">No knowledge entries yet.</p>
        )}
      </CardContent>
    </Card>
  );
}

function KnowledgeRow({
  k,
  tags,
  problems,
  onDelete,
}: {
  k: Knowledge;
  tags: Tag[];
  problems: { id: number; title: string }[];
  onDelete: () => void;
}) {
  const tagNames = k.tag_ids.map((id) => tags.find((t) => t.id === id)?.name).filter(Boolean) as string[];
  const linked = k.problem_id ? problems.find((p) => p.id === k.problem_id)?.title : null;
  return (
    <div className="rounded-md border border-border p-4 space-y-2">
      <div className="flex items-start justify-between gap-2">
        <div>
          <h3 className="font-medium">{k.title}</h3>
          <p className="text-xs text-muted-foreground">
            <Badge variant="outline" className="mr-2">
              {k.kind}
            </Badge>
            {linked && <span>↳ {linked}</span>}
          </p>
        </div>
        <Button size="sm" variant="destructive" onClick={onDelete}>
          Delete
        </Button>
      </div>
      <p className="text-sm whitespace-pre-wrap">{k.content}</p>
      {tagNames.length > 0 && (
        <div className="flex flex-wrap gap-1">
          {tagNames.map((n) => (
            <Badge key={n} variant="secondary">
              {n}
            </Badge>
          ))}
        </div>
      )}
    </div>
  );
}
