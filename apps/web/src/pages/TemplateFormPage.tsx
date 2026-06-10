import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { templatesApi, tagsApi, problemsApi } from "@/lib/api";
import type {
  TemplateCategory,
  CreateTemplateRequest,
  UpdateTemplateRequest,
} from "@acmind/shared";
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
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import { toast } from "@/lib/stores/toast";
import { ArrowLeft, X, Search } from "lucide-react";

const DIFFICULTY_OPTIONS = [
  { value: 1, label: "★ 基础" },
  { value: 2, label: "★★ 简单" },
  { value: 3, label: "★★★ 中等" },
  { value: 4, label: "★★★★ 困难" },
  { value: 5, label: "★★★★★ 极难" },
];

export default function TemplateFormPage() {
  const { id } = useParams<{ id: string }>();
  const isEdit = !!id;
  const tid = Number(id);
  const navigate = useNavigate();
  const qc = useQueryClient();

  const [title, setTitle] = useState("");
  const [category, setCategory] = useState<TemplateCategory>("other");
  const [language, setLanguage] = useState("cpp");
  const [code, setCode] = useState("");
  const [description, setDescription] = useState("");
  const [timeComplexity, setTimeComplexity] = useState("");
  const [spaceComplexity, setSpaceComplexity] = useState("");
  const [difficulty, setDifficulty] = useState<number | undefined>(undefined);
  const [selectedTagIds, setSelectedTagIds] = useState<number[]>([]);
  const [selectedProblemIds, setSelectedProblemIds] = useState<number[]>([]);
  const [tagSearch, setTagSearch] = useState("");

  // Load existing template for edit
  const existing = useQuery({
    queryKey: ["templates", tid],
    queryFn: () => templatesApi.get(tid),
    enabled: isEdit,
  });

  useEffect(() => {
    if (isEdit && existing.data) {
      const t = existing.data;
      setTitle(t.title);
      setCategory(t.category);
      setLanguage(t.language);
      setCode(t.code);
      setDescription(t.description);
      setTimeComplexity(t.time_complexity ?? "");
      setSpaceComplexity(t.space_complexity ?? "");
      setDifficulty(t.difficulty ?? undefined);
      setSelectedTagIds(t.tag_ids);
      setSelectedProblemIds(t.problem_ids);
    }
  }, [isEdit, existing.data]);

  const tags = useQuery({
    queryKey: ["tags"],
    queryFn: () => tagsApi.list(),
  });

  const problems = useQuery({
    queryKey: ["problems"],
    queryFn: () => problemsApi.list(),
  });

  const createMut = useMutation({
    mutationFn: (req: CreateTemplateRequest) => templatesApi.create(req),
    onSuccess: (t) => {
      qc.invalidateQueries({ queryKey: ["templates"] });
      toast.success("模板已创建");
      navigate(`/templates/${t.id}`);
    },
    onError: () => toast.error("创建失败"),
  });

  const updateMut = useMutation({
    mutationFn: (req: UpdateTemplateRequest) => templatesApi.update(tid, req),
    onSuccess: (t) => {
      qc.invalidateQueries({ queryKey: ["templates"] });
      toast.success("模板已更新");
      navigate(`/templates/${t.id}`);
    },
    onError: () => toast.error("更新失败"),
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim()) {
      toast.error("请填写模板名称");
      return;
    }
    if (!code.trim()) {
      toast.error("请填写模板代码");
      return;
    }

    if (isEdit) {
      updateMut.mutate({
        title,
        category,
        language,
        code,
        description,
        time_complexity: timeComplexity || undefined,
        space_complexity: spaceComplexity || undefined,
        difficulty,
        tag_ids: selectedTagIds,
      });
    } else {
      createMut.mutate({
        title,
        category,
        language,
        code,
        description,
        time_complexity: timeComplexity || undefined,
        space_complexity: spaceComplexity || undefined,
        difficulty,
        tag_ids: selectedTagIds,
        problem_ids: selectedProblemIds,
      });
    }
  };

  const toggleTag = (tagId: number) => {
    setSelectedTagIds((prev) =>
      prev.includes(tagId)
        ? prev.filter((id) => id !== tagId)
        : [...prev, tagId]
    );
  };

  const toggleProblem = (problemId: number) => {
    setSelectedProblemIds((prev) =>
      prev.includes(problemId)
        ? prev.filter((id) => id !== problemId)
        : [...prev, problemId]
    );
  };

  const filteredTags =
    tagSearch.trim()
      ? (tags.data ?? []).filter((t) =>
          t.name.toLowerCase().includes(tagSearch.toLowerCase())
        )
      : tags.data ?? [];

  const isLoading = isEdit && existing.isLoading;

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => navigate("/templates")}
          >
            <ArrowLeft className="h-4 w-4" />
          </Button>
          <CardTitle>{isEdit ? "编辑模板" : "新建模板"}</CardTitle>
        </div>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <p>Loading...</p>
        ) : (
          <form onSubmit={handleSubmit} className="space-y-4">
            {/* Title */}
            <div className="space-y-1.5">
              <Label htmlFor="title">模板名称 *</Label>
              <Input
                id="title"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                placeholder="如: Segment Tree, KMP, Dijkstra"
              />
            </div>

            {/* Category + Language + Difficulty */}
            <div className="grid grid-cols-3 gap-3">
              <div className="space-y-1.5">
                <Label>算法分类</Label>
                <Select
                  value={category}
                  onValueChange={(v) => setCategory(v as TemplateCategory)}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {TEMPLATE_CATEGORIES.map((c) => (
                      <SelectItem key={c.value} value={c.value}>
                        {c.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-1.5">
                <Label>编程语言</Label>
                <Select value={language} onValueChange={setLanguage}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {TEMPLATE_LANGUAGES.map((l) => (
                      <SelectItem key={l} value={l}>
                        {l}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-1.5">
                <Label>难度</Label>
                <Select
                  value={difficulty?.toString() ?? "none"}
                  onValueChange={(v) =>
                    setDifficulty(v === "none" ? undefined : Number(v))
                  }
                >
                  <SelectTrigger>
                    <SelectValue placeholder="选择难度" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="none">未设置</SelectItem>
                    {DIFFICULTY_OPTIONS.map((d) => (
                      <SelectItem
                        key={d.value}
                        value={String(d.value)}
                      >
                        {d.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>

            {/* Complexity */}
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1.5">
                <Label htmlFor="tc">时间复杂度</Label>
                <Input
                  id="tc"
                  value={timeComplexity}
                  onChange={(e) => setTimeComplexity(e.target.value)}
                  placeholder="如: O(n log n)"
                  className="font-mono"
                />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="sc">空间复杂度</Label>
                <Input
                  id="sc"
                  value={spaceComplexity}
                  onChange={(e) => setSpaceComplexity(e.target.value)}
                  placeholder="如: O(n)"
                  className="font-mono"
                />
              </div>
            </div>

            {/* Description */}
            <div className="space-y-1.5">
              <Label htmlFor="desc">使用说明</Label>
              <textarea
                id="desc"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="描述模板的适用场景、使用方法等..."
                className="w-full min-h-24 rounded-md border border-input bg-background px-3 py-2 text-sm resize-y"
              />
            </div>

            {/* Code */}
            <div className="space-y-1.5">
              <Label htmlFor="code">模板代码 *</Label>
              <textarea
                id="code"
                value={code}
                onChange={(e) => setCode(e.target.value)}
                placeholder="粘贴模板代码..."
                className="w-full min-h-48 rounded-md border border-input bg-background px-3 py-2 text-sm font-mono resize-y"
              />
            </div>

            {/* Tags */}
            <div className="space-y-1.5">
              <Label>标签</Label>
              <div className="relative">
                <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
                <Input
                  placeholder="搜索标签..."
                  value={tagSearch}
                  onChange={(e) => setTagSearch(e.target.value)}
                  className="pl-8 mb-2"
                />
              </div>
              <div className="flex flex-wrap gap-1.5 max-h-32 overflow-y-auto">
                {filteredTags.map((tag) => (
                  <button
                    key={tag.id}
                    type="button"
                    onClick={() => toggleTag(tag.id)}
                  >
                    <Badge
                      variant={
                        selectedTagIds.includes(tag.id)
                          ? "default"
                          : "outline"
                      }
                      className="cursor-pointer"
                    >
                      {tag.name}
                      {selectedTagIds.includes(tag.id) && (
                        <X className="h-3 w-3 ml-1" />
                      )}
                    </Badge>
                  </button>
                ))}
                {filteredTags.length === 0 && (
                  <span className="text-xs text-muted-foreground">
                    没有匹配的标签
                  </span>
                )}
              </div>
            </div>

            {/* Problems (create only) */}
            {!isEdit && (
              <div className="space-y-1.5">
                <Label>关联题目</Label>
                <div className="flex flex-wrap gap-1.5 max-h-32 overflow-y-auto border rounded-md p-2">
                  {(problems.data ?? []).map((p) => (
                    <button
                      key={p.id}
                      type="button"
                      onClick={() => toggleProblem(p.id)}
                    >
                      <Badge
                        variant={
                          selectedProblemIds.includes(p.id)
                            ? "default"
                            : "outline"
                        }
                        className="cursor-pointer text-xs"
                      >
                        {p.title}
                        {selectedProblemIds.includes(p.id) && (
                          <X className="h-3 w-3 ml-1" />
                        )}
                      </Badge>
                    </button>
                  ))}
                  {(!problems.data || problems.data.length === 0) && (
                    <span className="text-xs text-muted-foreground">
                      还没有题目
                    </span>
                  )}
                </div>
              </div>
            )}

            {/* Submit */}
            <div className="flex justify-end gap-2">
              <Button
                type="button"
                variant="outline"
                onClick={() => navigate("/templates")}
              >
                取消
              </Button>
              <Button
                type="submit"
                disabled={createMut.isPending || updateMut.isPending}
              >
                {createMut.isPending || updateMut.isPending
                  ? "保存中..."
                  : isEdit
                    ? "保存修改"
                    : "创建模板"}
              </Button>
            </div>
          </form>
        )}
      </CardContent>
    </Card>
  );
}
