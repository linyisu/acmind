import { useState, useEffect } from "react";
import { useParams, useNavigate, useLocation } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import Markdown from "react-markdown";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import vscDarkPlus from "react-syntax-highlighter/dist/esm/styles/prism/vsc-dark-plus";
import { problemsApi, submissionsApi, tagsApi, aiApi, templatesApi, tasksApi } from "@/lib/api";
import type { AnalysisResp, Task } from "@/lib/api";
import { TEMPLATE_CATEGORIES } from "@acmind/shared";
import { toast } from "@/lib/stores/toast";
import { useTaskStore } from "@/lib/stores/task";
import type { Submission } from "@acmind/shared";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { ExternalLink, Copy, Check, Loader2, Search, Wrench, AlertTriangle, BookOpen, Database } from "lucide-react";

const AGENT_ICONS: Record<string, React.ReactNode> = {
  classifier: <Search className="h-3.5 w-3.5" />,
  template:   <Wrench className="h-3.5 w-3.5" />,
  error:      <AlertTriangle className="h-3.5 w-3.5" />,
  knowledge:  <BookOpen className="h-3.5 w-3.5" />,
  saver:      <Database className="h-3.5 w-3.5" />,
};

const LANG_MAP: Record<string, string> = {
  "C++": "cpp",
  "C++17": "cpp",
  "C++20": "cpp",
  C: "c",
  Python3: "python",
  Python: "python",
  Java: "java",
  Rust: "rust",
};

function verdictColor(v: string): string {
  if (v === "AC") return "bg-green-600 text-white";
  if (v === "WA") return "bg-red-600 text-white";
  if (v === "TLE" || v === "MLE") return "bg-yellow-600 text-white";
  if (v === "RE") return "bg-orange-600 text-white";
  if (v === "CE") return "bg-gray-500 text-white";
  return "bg-secondary text-secondary-foreground";
}

export default function ProblemDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const location = useLocation();
  const [selectedSubmission, setSelectedSubmission] = useState<Submission | null>(null);
  const [copied, setCopied] = useState(false);
  const [aiResult, setAiResult] = useState<AnalysisResp | null>(null);
  const [resultTask, setResultTask] = useState<Task | null>(null);
  const qc = useQueryClient();

  const { tasks, startPolling, setActiveTask, init } = useTaskStore();

  // Track the active analysis task
  const [activeTaskId, setActiveTaskIdLocal] = useState<number | null>(null);
  const activeTask = activeTaskId ? tasks.find((t) => t.id === activeTaskId) : null;

  // Derive isAnalyzing from actual task status, not just the ID
  // If the task is no longer running/pending, the analysis is done (success or fail).
  // If the task is not in the store yet (just created), treat as running.
  const activeTaskStatus = activeTask?.status;
  const isAnalyzing = activeTaskId !== null && (activeTaskStatus === undefined || activeTaskStatus === "running" || activeTaskStatus === "pending");

  // On mount: load tasks
  useEffect(() => {
    init();
  }, [init]);

  // If navigated from TaskIndicator with a taskId, show that task's result
  useEffect(() => {
    const state = location.state as { taskId?: number } | null;
    if (state?.taskId) {
      const task = tasks.find((t) => t.id === state.taskId);
      if (task?.status === "completed" && task.result) {
        setResultTask(task);
        navigate(location.pathname, { replace: true });
      }
    }
  }, [location.state, tasks, navigate, location.pathname]);

  // Find past completed analysis for this problem
  const pastAnalysis = tasks.find(
    (t) =>
      t.target_type === "problem" &&
      t.target_id === Number(id) &&
      t.status === "completed" &&
      t.result
  );

  // When task finishes, show result or toast + clear active ID
  useEffect(() => {
    if (!activeTask || !activeTaskId) return;

    if (activeTask.status === "completed" && activeTask.result) {
      setResultTask(activeTask);
      setActiveTaskIdLocal(null);
      qc.invalidateQueries({ queryKey: ["knowledge"] });
      qc.invalidateQueries({ queryKey: ["ai-analyses"] });
      qc.invalidateQueries({ queryKey: ["templates"] });
      toast.success("AI 全量分析完成！");
    } else if (activeTask.status === "failed") {
      setActiveTaskIdLocal(null);
      const err = activeTask.error?.replace(/^internal error:\s*/i, "") || "未知错误";
      toast.error(`分析失败: ${err}`);
    } else if (activeTask.status === "cancelled") {
      setActiveTaskIdLocal(null);
      toast("分析已取消");
    }
  }, [activeTaskId, activeTask, activeTask?.status, qc]);

  const analyzeMut = useMutation({
    mutationFn: (submissionId: number) => aiApi.analyze(submissionId),
    onSuccess: (data) => {
      setAiResult(data);
      qc.invalidateQueries({ queryKey: ["ai-analyses"] });
    },
  });

  const fullAnalyzeMut = useMutation({
    mutationFn: (problemId: number) => aiApi.analyzeProblem(problemId),
    onSuccess: (task: Task) => {
      setActiveTaskIdLocal(task.id);
      setActiveTask(task.id);
      startPolling();
      toast("分析任务已创建，正在后台运行...");
    },
    onError: (e: unknown) => {
      const msg = e instanceof Error ? e.message : String(e);
      toast.error(`创建分析任务失败: ${msg}`);
    },
  });

  const problem = useQuery({
    queryKey: ["problems", Number(id)],
    queryFn: () => problemsApi.get(Number(id!)),
    enabled: !!id,
  });

  const submissions = useQuery({
    queryKey: ["submissions", Number(id)],
    queryFn: () => submissionsApi.list(Number(id!)),
    enabled: !!id,
  });

  const tags = useQuery({ queryKey: ["tags"], queryFn: () => tagsApi.list() });

  const linkedTemplates = useQuery({
    queryKey: ["templates", "problem", Number(id)],
    queryFn: () => templatesApi.list({ problem_id: Number(id) }),
    enabled: !!id,
  });

  if (problem.isLoading) {
    return <p className="text-muted-foreground">Loading…</p>;
  }

  if (!problem.data) {
    return <p className="text-muted-foreground">Problem not found.</p>;
  }

  const p = problem.data;
  const tagNames = p.tag_ids
    .map((tid) => tags.data?.find((t) => t.id === tid)?.name)
    .filter(Boolean) as string[];

  function copyCode() {
    if (selectedSubmission?.code) {
      navigator.clipboard.writeText(selectedSubmission.code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  }

  function getLanguageHint(lang: string): string {
    for (const [key, value] of Object.entries(LANG_MAP)) {
      if (lang.toLowerCase().includes(key.toLowerCase())) return value;
    }
    return "plaintext";
  }

  return (
    <div className="space-y-6 max-w-5xl">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold">{p.title}</h1>
          <div className="flex items-center gap-2 mt-1 text-sm text-muted-foreground">
            <span>{p.source}</span>
            {p.external_id && <span>· {p.external_id}</span>}
            {p.difficulty != null && (
              <>
                <span>·</span>
                <Badge variant="outline">难度 {p.difficulty}</Badge>
              </>
            )}
          </div>
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            onClick={() => id && fullAnalyzeMut.mutate(Number(id))}
            disabled={isAnalyzing || fullAnalyzeMut.isPending}
          >
            {isAnalyzing ? (
              <span className="flex items-center gap-1.5">
                <Loader2 className="h-3.5 w-3.5 animate-spin" />
                分析中…
              </span>
            ) : (
              "AI 全量分析"
            )}
          </Button>
          {isAnalyzing && activeTaskId && (
            <Button
              variant="destructive"
              size="sm"
              onClick={async () => {
                try {
                  await tasksApi.cancel(activeTaskId);
                  setActiveTaskIdLocal(null);
                  toast.success("分析已取消");
                  qc.invalidateQueries({ queryKey: ["tasks"] });
                } catch {
                  toast.error("取消失败");
                }
              }}
            >
              取消
            </Button>
          )}
          {pastAnalysis && (
            <Button
              variant="outline"
              onClick={() => setResultTask(pastAnalysis)}
            >
              查看上次分析
            </Button>
          )}
          <Button variant="outline" onClick={() => navigate(`/problems/${id}/edit`)}>
            编辑
          </Button>
        </div>
      </div>

      {/* Active task progress */}
      {activeTask && activeTask.status === "running" && (
        <Card className="border-blue-800 bg-blue-950/30">
          <CardContent className="py-3">
            <div className="flex items-center gap-2 mb-2">
              <Loader2 className="h-4 w-4 animate-spin text-blue-400" />
              <span className="text-sm font-medium text-blue-300">
                AI 全量分析进行中
              </span>
            </div>
            <div className="space-y-1">
              {activeTask.progress.agents?.map((agent) => (
                <div key={agent.id} className="flex items-center gap-2 text-xs">
                  {AGENT_ICONS[agent.id] ?? <span className="w-3.5" />}
                  {agent.status === "completed" && <Check className="h-3 w-3 text-green-400" />}
                  {agent.status === "running" && <Loader2 className="h-3 w-3 animate-spin text-blue-400" />}
                  {agent.status === "pending" && <div className="h-3 w-3 rounded-full border border-muted-foreground/30" />}
                  {agent.status === "failed" && <span className="text-red-400">✗</span>}
                  <span className={
                    agent.status === "completed" ? "text-muted-foreground" :
                    agent.status === "running" ? "text-blue-300" : "text-muted-foreground/50"
                  }>
                    {agent.name}
                    {agent.message && agent.status !== "pending" ? `: ${agent.message}` : ""}
                  </span>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {tagNames.length > 0 && (
        <div className="flex flex-wrap gap-1">
          {tagNames.map((n) => (
            <Badge key={n} variant="secondary">{n}</Badge>
          ))}
        </div>
      )}

      {p.url && (
        <a href={p.url} target="_blank" rel="noopener noreferrer" className="inline-block mb-2">
          <Button variant="outline" size="sm" className="gap-1.5">
            <ExternalLink className="h-3.5 w-3.5" />
            在 {p.source} 查看
          </Button>
        </a>
      )}

      {p.statement && (
        <Card>
          <CardHeader>
            <CardTitle className="text-base">题面</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="statement-content">
              <Markdown remarkPlugins={[remarkMath]} rehypePlugins={[rehypeKatex]}>
                {p.statement}
              </Markdown>
            </div>
          </CardContent>
        </Card>
      )}

      <Card>
        <CardHeader>
          <CardTitle className="text-base">提交记录</CardTitle>
        </CardHeader>
        <CardContent>
          {submissions.isLoading ? (
            <p className="text-muted-foreground">加载中…</p>
          ) : submissions.data && submissions.data.length > 0 ? (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>时间</TableHead>
                  <TableHead>语言</TableHead>
                  <TableHead>结果</TableHead>
                  <TableHead>运行时间</TableHead>
                  <TableHead>内存</TableHead>
                  <TableHead className="w-12"></TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {submissions.data.map((s) => (
                  <TableRow
                    key={s.id}
                    className="cursor-pointer hover:bg-accent/50"
                    onClick={() => setSelectedSubmission(s)}
                  >
                    <TableCell className="text-xs text-muted-foreground">
                      {new Date(s.submitted_at).toLocaleString()}
                    </TableCell>
                    <TableCell>{s.language}</TableCell>
                    <TableCell>
                      <span
                        className={`inline-flex items-center rounded-md px-2 py-0.5 text-xs font-medium ${verdictColor(s.verdict)}`}
                      >
                        {s.verdict}
                      </span>
                    </TableCell>
                    <TableCell>{s.runtime_ms ?? "—"} ms</TableCell>
                    <TableCell>{s.memory_kb ?? "—"} KB</TableCell>
                    <TableCell>
                      <button
                        className="rounded-md p-1 hover:bg-accent transition-colors"
                        title="AI Analyze"
                        disabled={analyzeMut.isPending}
                        onClick={(e) => {
                          e.stopPropagation();
                          analyzeMut.mutate(s.id);
                        }}
                      >
                        <Search className="h-3.5 w-3.5" />
                      </button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          ) : (
            <p className="text-muted-foreground">还没有提交记录。</p>
          )}
        </CardContent>
      </Card>

      {/* Linked Templates */}
      {linkedTemplates.data && linkedTemplates.data.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="text-base">
              🔗 关联模板 ({linkedTemplates.data.length})
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid gap-2 sm:grid-cols-2">
              {linkedTemplates.data.map((t) => {
                const catLabel =
                  TEMPLATE_CATEGORIES.find((c) => c.value === t.category)?.label ?? t.category;
                return (
                  <button
                    key={t.id}
                    onClick={() => navigate(`/templates/${t.id}`)}
                    className="text-left rounded-md border border-border p-3 hover:bg-accent/30 transition-colors"
                  >
                    <div className="flex items-center gap-2 mb-1">
                      <span className="font-medium text-sm">{t.title}</span>
                      <Badge variant="outline" className="text-xs">
                        {catLabel}
                      </Badge>
                      <Badge variant="secondary" className="text-xs font-mono">
                        {t.language}
                      </Badge>
                    </div>
                    {t.time_complexity && (
                      <span className="text-xs text-muted-foreground font-mono">
                        {t.time_complexity}
                      </span>
                    )}
                  </button>
                );
              })}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Submission code dialog */}
      <Dialog
        open={!!selectedSubmission}
        onOpenChange={(open) => {
          if (!open) setSelectedSubmission(null);
        }}
      >
        <DialogContent className="sm:max-w-none w-[95vw] max-w-[1400px] h-[90vh] p-0 gap-0 overflow-hidden flex flex-col">
          <div className="flex items-center gap-2 px-4 py-2.5 border-b bg-muted/30 shrink-0 pr-12">
            <span className="font-semibold text-sm">#{selectedSubmission?.id}</span>
            {selectedSubmission && (
              <span
                className={`inline-flex items-center rounded-md px-2 py-0.5 text-xs font-medium ${verdictColor(selectedSubmission.verdict)}`}
              >
                {selectedSubmission.verdict}
              </span>
            )}
            {selectedSubmission && (
              <span className="text-xs text-muted-foreground">
                {selectedSubmission.language} · {selectedSubmission.runtime_ms ?? "—"}ms ·{" "}
                {selectedSubmission.memory_kb ?? "—"}KB
              </span>
            )}
          </div>
          <div className="flex-1 overflow-auto relative">
            <button
              onClick={copyCode}
              title={copied ? "已复制" : "复制"}
              className="absolute top-2 right-3 z-10 rounded-md bg-zinc-700 hover:bg-zinc-600 px-2 py-1 text-xs text-zinc-300 hover:text-white transition-colors"
            >
              {copied ? <Check className="size-3.5" /> : <Copy className="size-3.5" />}
            </button>
            {selectedSubmission?.code ? (
              <SyntaxHighlighter
                language={getLanguageHint(selectedSubmission.language)}
                style={vscDarkPlus}
                customStyle={{
                  margin: 0,
                  borderRadius: 0,
                  fontSize: "0.82rem",
                  lineHeight: "1.6",
                  minHeight: "100%",
                  background: "#1e1e1e",
                }}
                showLineNumbers
              >
                {selectedSubmission.code}
              </SyntaxHighlighter>
            ) : (
              <div className="flex items-center justify-center h-full text-muted-foreground">
                该提交没有源代码。
              </div>
            )}
          </div>
        </DialogContent>
      </Dialog>

      {/* AI Analysis result dialog (single submission) */}
      <Dialog
        open={!!aiResult}
        onOpenChange={(open) => {
          if (!open) setAiResult(null);
        }}
      >
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              AI 分析结果
              {aiResult && <Badge variant="outline">{aiResult.result.algorithm_type}</Badge>}
            </DialogTitle>
          </DialogHeader>
          {aiResult && (
            <div className="space-y-4">
              <div>
                <h4 className="text-sm font-medium mb-1">算法类型</h4>
                <p className="text-sm text-muted-foreground">
                  {aiResult.result.algorithm_type} / {aiResult.result.sub_type}
                </p>
              </div>
              {aiResult.result.tags.length > 0 && (
                <div>
                  <h4 className="text-sm font-medium mb-1">标签</h4>
                  <div className="flex flex-wrap gap-1">
                    {aiResult.result.tags.map((t) => (
                      <Badge key={t} variant="secondary">
                        {t}
                      </Badge>
                    ))}
                  </div>
                </div>
              )}
              <div>
                <h4 className="text-sm font-medium mb-1">分析摘要</h4>
                <div className="text-sm text-muted-foreground knowledge-content">
                  <Markdown remarkPlugins={[remarkMath]} rehypePlugins={[rehypeKatex]}>
                    {aiResult.result.summary}
                  </Markdown>
                </div>
              </div>
              {aiResult.result.template_snippet && (
                <div>
                  <h4 className="text-sm font-medium mb-1">模板代码</h4>
                  <SyntaxHighlighter
                    language="cpp"
                    style={vscDarkPlus}
                    customStyle={{ margin: 0, borderRadius: "0.375rem", fontSize: "0.8rem" }}
                  >
                    {aiResult.result.template_snippet}
                  </SyntaxHighlighter>
                </div>
              )}
              {aiResult.result.error_analysis && (
                <div>
                  <h4 className="text-sm font-medium mb-1">错误分析</h4>
                  <p className="text-sm text-destructive">{aiResult.result.error_analysis}</p>
                </div>
              )}
            </div>
          )}
        </DialogContent>
      </Dialog>

      {/* Full Problem Analysis result dialog (from task) */}
      <Dialog
        open={!!resultTask}
        onOpenChange={(open) => {
          if (!open) setResultTask(null);
        }}
      >
        <DialogContent className="max-w-3xl max-h-[85vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              AI 全量分析
              {resultTask?.result && "algorithm_type" in resultTask.result && (
                <Badge variant="outline">{String(resultTask.result.algorithm_type)}</Badge>
              )}
            </DialogTitle>
          </DialogHeader>
          {resultTask?.result && (
            <div className="space-y-4">
              {/* Stats */}
              <div className="grid grid-cols-3 gap-3">
                <div className="rounded-md border p-3 text-center">
                  <p className="text-2xl font-bold">
                    {String(resultTask.result.submissions_analyzed ?? 0)}
                  </p>
                  <p className="text-xs text-muted-foreground">分析提交数</p>
                </div>
                <div className="rounded-md border p-3 text-center">
                  <p className="text-2xl font-bold">
                    {Number(resultTask.result.extracted_templates ?? 0) +
                      Number(resultTask.result.extracted_errors ?? 0) +
                      Number(resultTask.result.extracted_knowledge ?? 0)}
                  </p>
                  <p className="text-xs text-muted-foreground">提取条目数</p>
                </div>
                <div className="rounded-md border p-3 text-center">
                  <p className="text-2xl font-bold">
                    {String(resultTask.result.saved ?? 0)}
                  </p>
                  <p className="text-xs text-muted-foreground">保存知识条目</p>
                </div>
              </div>

              {/* Algorithm & Tags */}
              <div>
                <h4 className="text-sm font-medium mb-1">算法分类</h4>
                <p className="text-sm text-muted-foreground">
                  {String(resultTask.result.algorithm_type)} /{" "}
                  {String(resultTask.result.sub_type)}
                </p>
                {Array.isArray(resultTask.result.tags) && (
                  <div className="flex flex-wrap gap-1 mt-1">
                    {(resultTask.result.tags as string[]).map((t) => (
                      <Badge key={t} variant="secondary">
                        {t}
                      </Badge>
                    ))}
                  </div>
                )}
              </div>

              {/* Summary */}
              <div>
                <h4 className="text-sm font-medium mb-1">综合分析</h4>
                <div className="text-sm text-muted-foreground knowledge-content">
                  <Markdown remarkPlugins={[remarkMath]} rehypePlugins={[rehypeKatex]}>
                    {String(resultTask.result.summary)}
                  </Markdown>
                </div>
              </div>

              {/* Extracted items summary */}
              <div className="border-t pt-3">
                <h4 className="text-sm font-medium mb-2">已自动保存</h4>
                <div className="flex gap-4 text-sm text-muted-foreground">
                  <span className="flex items-center gap-1"><Wrench className="h-3.5 w-3.5" /> 模板 ×{String(resultTask.result.extracted_templates ?? 0)}</span>
                  <span className="flex items-center gap-1"><AlertTriangle className="h-3.5 w-3.5" /> 错误模式 ×{String(resultTask.result.extracted_errors ?? 0)}</span>
                  <span className="flex items-center gap-1"><BookOpen className="h-3.5 w-3.5" /> 知识点 ×{String(resultTask.result.extracted_knowledge ?? 0)}</span>
                </div>
                <p className="text-xs text-muted-foreground mt-1">
                  模板已保存到模板库，错误和知识点已保存到知识库
                </p>
              </div>
            </div>
          )}
        </DialogContent>
      </Dialog>

      {/* AI loading indicator (single submission) */}
      {analyzeMut.isPending && (
        <div className="fixed bottom-4 right-4 bg-primary text-primary-foreground px-4 py-2 rounded-lg shadow-lg text-sm animate-pulse">
          分析中…
        </div>
      )}
      {analyzeMut.isError && (
        <div className="fixed bottom-4 right-4 bg-destructive text-destructive-foreground px-4 py-2 rounded-lg shadow-lg text-sm">
          ❌ 分析失败
        </div>
      )}
    </div>
  );
}
