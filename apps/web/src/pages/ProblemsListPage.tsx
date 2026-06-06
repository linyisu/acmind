import { useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { problemsApi, tagsApi } from "@/lib/api";
import type { Problem, Tag } from "@acmind/shared";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

const ALL = "__all__";

export default function ProblemsListPage() {
  const [filter, setFilter] = useState<string>(ALL);
  const navigate = useNavigate();
  const qc = useQueryClient();
  const tagId = filter === ALL ? undefined : Number(filter);
  const problems = useQuery({
    queryKey: ["problems", tagId],
    queryFn: () => problemsApi.list(tagId),
  });
  const tags = useQuery({ queryKey: ["tags"], queryFn: () => tagsApi.list() });
  const remove = useMutation({
    mutationFn: (id: number) => problemsApi.delete(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["problems"] }),
  });

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle>Problems</CardTitle>
        <div className="flex items-center gap-2">
          <Select value={filter} onValueChange={setFilter}>
            <SelectTrigger className="w-40">
              <SelectValue placeholder="Filter by tag" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={ALL}>All tags</SelectItem>
              {tags.data?.map((t) => (
                <SelectItem key={t.id} value={String(t.id)}>
                  {t.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Button onClick={() => navigate("/problems/new")}>New</Button>
        </div>
      </CardHeader>
      <CardContent>
        {problems.isLoading ? (
          <p>Loading…</p>
        ) : problems.data && problems.data.length > 0 ? (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Title</TableHead>
                <TableHead>Source</TableHead>
                <TableHead>Difficulty</TableHead>
                <TableHead>Tags</TableHead>
                <TableHead className="w-24">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {problems.data.map((p) => (
                <ProblemRow
                  key={p.id}
                  problem={p}
                  tags={tags.data ?? []}
                  onDelete={() => remove.mutate(p.id)}
                />
              ))}
            </TableBody>
          </Table>
        ) : (
          <p className="text-muted-foreground">No problems yet.</p>
        )}
      </CardContent>
    </Card>
  );
}

function ProblemRow({
  problem,
  tags,
  onDelete,
}: {
  problem: Problem;
  tags: Tag[];
  onDelete: () => void;
}) {
  const tagNames = problem.tag_ids
    .map((id) => tags.find((t) => t.id === id)?.name)
    .filter(Boolean) as string[];
  return (
    <TableRow>
      <TableCell>
        <Link to={`/problems/${problem.id}`} className="underline font-medium">
          {problem.title}
        </Link>
      </TableCell>
      <TableCell>{problem.source}</TableCell>
      <TableCell>{problem.difficulty ?? "—"}</TableCell>
      <TableCell>
        <div className="flex flex-wrap gap-1">
          {tagNames.map((n) => (
            <Badge key={n} variant="secondary">
              {n}
            </Badge>
          ))}
        </div>
      </TableCell>
      <TableCell>
        <Button
          size="sm"
          variant="destructive"
          onClick={() => {
            if (confirm(`Delete "${problem.title}"?`)) onDelete();
          }}
        >
          Delete
        </Button>
      </TableCell>
    </TableRow>
  );
}
