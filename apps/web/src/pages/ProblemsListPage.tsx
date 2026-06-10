import { useState, useMemo } from "react";
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

const ALL = "__all__";

export default function ProblemsListPage() {
  const [tagFilter, setTagFilter] = useState<string>(ALL);
  const [search, setSearch] = useState("");
  const navigate = useNavigate();
  const qc = useQueryClient();
  const problems = useQuery({
    queryKey: ["problems"],
    queryFn: () => problemsApi.list(),
  });
  const tags = useQuery({ queryKey: ["tags"], queryFn: () => tagsApi.list() });
  const remove = useMutation({
    mutationFn: (id: number) => problemsApi.delete(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["problems"] });
      toast.success("题目已删除");
    },
    onError: () => toast.error("删除失败"),
  });

  const filtered = useMemo(() => {
    if (!problems.data) return [];
    let list = problems.data;

    if (tagFilter !== ALL) {
      const tid = Number(tagFilter);
      list = list.filter((p) => p.tag_ids.includes(tid));
    }

    if (search.trim()) {
      const q = search.trim().toLowerCase();
      list = list.filter(
        (p) =>
          p.title.toLowerCase().includes(q) ||
          p.source.toLowerCase().includes(q) ||
          (p.external_id && p.external_id.toLowerCase().includes(q)),
      );
    }

    return list;
  }, [problems.data, tagFilter, search]);

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle>题目列表</CardTitle>
        <Button onClick={() => navigate("/problems/new")}>新建题目</Button>
      </CardHeader>
      <CardContent>
        {/* Filters */}
        <div className="flex flex-wrap gap-3 mb-4">
          <div className="relative flex-1 min-w-48">
            <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="搜索题目名称、来源或 ID…"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="pl-8"
            />
          </div>
          <Select value={tagFilter} onValueChange={setTagFilter}>
            <SelectTrigger className="w-40">
              <SelectValue placeholder="按标签筛选" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={ALL}>全部标签</SelectItem>
              {tags.data?.map((t) => (
                <SelectItem key={t.id} value={String(t.id)}>
                  {t.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        {/* Results count */}
        {problems.data && problems.data.length > 0 && (
          <p className="text-xs text-muted-foreground mb-3">
            显示 {filtered.length} / {problems.data.length} 道题目
          </p>
        )}

        {problems.isLoading ? (
          <p>加载中…</p>
        ) : filtered.length > 0 ? (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>标题</TableHead>
                <TableHead>来源</TableHead>
                <TableHead>难度</TableHead>
                <TableHead>标签</TableHead>
                <TableHead className="w-24">操作</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {filtered.map((p) => (
                <ProblemRow
                  key={p.id}
                  problem={p}
                  tags={tags.data ?? []}
                  onDelete={() => {
                    if (confirm(`确认删除「${p.title}」？`)) remove.mutate(p.id);
                  }}
                />
              ))}
            </TableBody>
          </Table>
        ) : problems.data && problems.data.length > 0 ? (
          <p className="text-muted-foreground">没有匹配的题目。</p>
        ) : (
          <p className="text-muted-foreground">还没有题目，点击「新建题目」开始。</p>
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
          onClick={onDelete}
        >
          Delete
        </Button>
      </TableCell>
    </TableRow>
  );
}
