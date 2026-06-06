import { useState, useEffect } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { problemsApi, tagsApi } from "@/lib/api";
import type { Problem, Tag } from "@acmind/shared";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { ApiError } from "@/lib/api/client";

export default function ProblemFormPage() {
  const { id } = useParams<{ id?: string }>();
  const isEdit = id !== undefined;
  const navigate = useNavigate();
  const qc = useQueryClient();
  const [title, setTitle] = useState("");
  const [source, setSource] = useState("codeforces");
  const [externalId, setExternalId] = useState("");
  const [url, setUrl] = useState("");
  const [difficulty, setDifficulty] = useState("");
  const [statement, setStatement] = useState("");
  const [tagInput, setTagInput] = useState("");
  const [tagIds, setTagIds] = useState<number[]>([]);
  const [error, setError] = useState<string | null>(null);

  const tags = useQuery({ queryKey: ["tags"], queryFn: () => tagsApi.list() });
  const problem = useQuery({
    queryKey: ["problems", Number(id)],
    queryFn: () => problemsApi.get(Number(id!)),
    enabled: isEdit,
  });

  useEffect(() => {
    if (problem.data) {
      const p = problem.data;
      setTitle(p.title);
      setSource(p.source);
      setExternalId(p.external_id ?? "");
      setUrl(p.url ?? "");
      setDifficulty(p.difficulty?.toString() ?? "");
      setStatement(p.statement ?? "");
      setTagIds(p.tag_ids);
    }
  }, [problem.data]);

  const create = useMutation({
    mutationFn: () =>
      problemsApi.create({
        source,
        external_id: externalId || undefined,
        title,
        url: url || undefined,
        difficulty: difficulty ? Number(difficulty) : undefined,
        statement: statement || undefined,
        tag_ids: tagIds,
      }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["problems"] });
      navigate("/problems");
    },
    onError: (e) =>
      setError(e instanceof ApiError ? e.message : "Failed to create problem"),
  });

  const update = useMutation({
    mutationFn: () =>
      problemsApi.update(Number(id), {
        source,
        external_id: externalId || undefined,
        title,
        url: url || undefined,
        difficulty: difficulty ? Number(difficulty) : undefined,
        statement: statement || undefined,
        tag_ids: tagIds,
      }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["problems"] });
      navigate("/problems");
    },
    onError: (e) =>
      setError(e instanceof ApiError ? e.message : "Failed to update problem"),
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
        .catch((e) => setError(e instanceof ApiError ? e.message : "Failed to create tag"));
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
    <Card className="max-w-2xl">
      <CardHeader>
        <CardTitle>{isEdit ? "Edit problem" : "New problem"}</CardTitle>
      </CardHeader>
      <CardContent>
        <form onSubmit={onSubmit} className="space-y-4">
          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-1.5">
              <Label>Source</Label>
              <Input value={source} onChange={(e) => setSource(e.target.value)} required />
            </div>
            <div className="space-y-1.5">
              <Label>External ID</Label>
              <Input value={externalId} onChange={(e) => setExternalId(e.target.value)} />
            </div>
          </div>
          <div className="space-y-1.5">
            <Label>Title</Label>
            <Input value={title} onChange={(e) => setTitle(e.target.value)} required />
          </div>
          <div className="space-y-1.5">
            <Label>URL</Label>
            <Input value={url} onChange={(e) => setUrl(e.target.value)} />
          </div>
          <div className="space-y-1.5">
            <Label>Difficulty</Label>
            <Input
              type="number"
              value={difficulty}
              onChange={(e) => setDifficulty(e.target.value)}
            />
          </div>
          <div className="space-y-1.5">
            <Label>Statement</Label>
            <textarea
              className="flex min-h-24 w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              value={statement}
              onChange={(e) => setStatement(e.target.value)}
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
                placeholder="Type a tag and press Enter"
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
          {error && <p className="text-sm text-destructive">{error}</p>}
          <div className="flex justify-end gap-2">
            <Button type="button" variant="ghost" onClick={() => navigate("/problems")}>
              Cancel
            </Button>
            <Button type="submit" disabled={busy}>
              {busy ? "Saving…" : isEdit ? "Save" : "Create"}
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}
