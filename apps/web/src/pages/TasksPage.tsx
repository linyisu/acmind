import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { useTaskStore } from "@/lib/stores/task";
import { tasksApi } from "@/lib/api";
import type { AgentProgress, Task } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { toast } from "@/lib/stores/toast";
import {
  Loader2,
  CheckCircle2,
  XCircle,
  Clock,
  Activity,
  X,
  Search,
  Wrench,
  AlertTriangle,
  BookOpen,
  Database,
} from "lucide-react";

const STATUS_ICONS: Record<string, React.ReactNode> = {
  completed: <CheckCircle2 className="h-4 w-4 text-green-500" />,
  running: <Loader2 className="h-4 w-4 text-blue-400 animate-spin" />,
  failed: <XCircle className="h-4 w-4 text-red-500" />,
  cancelled: <XCircle className="h-4 w-4 text-orange-400" />,
  pending: <Clock className="h-4 w-4 text-muted-foreground/40" />,
};

/** Agent role icons (Lucide), keyed by agent id. */
const AGENT_ICONS: Record<string, React.ReactNode> = {
  classifier: <Search className="h-4 w-4" />,
  template:   <Wrench className="h-4 w-4" />,
  error:      <AlertTriangle className="h-4 w-4" />,
  knowledge:  <BookOpen className="h-4 w-4" />,
  saver:      <Database className="h-4 w-4" />,
};

function agentIcon(agentId: string): React.ReactNode {
  return AGENT_ICONS[agentId] ?? <Activity className="h-4 w-4" />;
}

const TASK_KIND_LABELS: Record<string, string> = {
  ai_full_analysis: "全量分析",
};

function formatElapsed(task: Task): string {
  if (!task.started_at) return "";
  const start = new Date(task.started_at).getTime();
  const end = task.completed_at
    ? new Date(task.completed_at).getTime()
    : Date.now();
  const secs = Math.floor((end - start) / 1000);
  if (secs < 60) return `${secs}秒`;
  const mins = Math.floor(secs / 60);
  const remainSecs = secs % 60;
  return `${mins}分${remainSecs}秒`;
}

function timeAgo(dateStr: string): string {
  const now = Date.now();
  const then = new Date(dateStr).getTime();
  const diffSecs = Math.floor((now - then) / 1000);
  if (diffSecs < 60) return "刚刚";
  if (diffSecs < 3600) return `${Math.floor(diffSecs / 60)} 分钟前`;
  if (diffSecs < 86400) return `${Math.floor(diffSecs / 3600)} 小时前`;
  return `${Math.floor(diffSecs / 86400)} 天前`;
}

function statusBadge(status: string) {
  switch (status) {
    case "completed":
      return <Badge className="bg-green-600 text-white">已完成</Badge>;
    case "running":
      return <Badge className="bg-blue-600 text-white">运行中</Badge>;
    case "failed":
      return <Badge variant="destructive">失败</Badge>;
    case "cancelled":
      return <Badge className="bg-orange-500 text-white">已取消</Badge>;
    default:
      return <Badge variant="outline">等待中</Badge>;
  }
}

export default function TasksPage() {
  const navigate = useNavigate();
  const { tasks, polling, init, fetchTasks } = useTaskStore();

  useEffect(() => {
    init();
  }, [init]);

  // Refresh on focus
  useEffect(() => {
    const handler = () => fetchTasks();
    window.addEventListener("focus", handler);
    return () => window.removeEventListener("focus", handler);
  }, [fetchTasks]);

  const running = tasks.filter((t) => t.status === "running" || t.status === "pending");
  const completed = tasks.filter((t) => t.status === "completed");
  const failed = tasks.filter((t) => t.status === "failed");

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <div className="flex items-center gap-2">
            <Activity className="h-5 w-5" />
            <CardTitle>任务中心</CardTitle>
          </div>
          <div className="flex gap-3 text-sm text-muted-foreground">
            {running.length > 0 && (
              <span className="flex items-center gap-1">
                <Loader2 className="h-3.5 w-3.5 animate-spin text-blue-400" />
                运行中 {running.length}
              </span>
            )}
            <span>已完成 {completed.length}</span>
            {failed.length > 0 && (
              <span className="text-red-400">失败 {failed.length}</span>
            )}
          </div>
        </CardHeader>
        <CardContent>
          {polling && (
            <p className="text-xs text-blue-400 mb-3 flex items-center gap-1">
              <Loader2 className="h-3 w-3 animate-spin" />
              自动刷新中（每 2 秒）
            </p>
          )}

          {tasks.length === 0 ? (
            <p className="text-muted-foreground text-sm">还没有任务。在题目详情页点击「AI 全量分析」开始。</p>
          ) : (
            <div className="space-y-3">
              {tasks.map((task) => (
                <TaskCard
                  key={task.id}
                  task={task}
                  navigate={navigate}
                  onCancel={
                    task.status === "running" || task.status === "pending"
                      ? async () => {
                          try {
                            await tasksApi.cancel(task.id);
                            toast.success("取消指令已发送");
                            fetchTasks();
                          } catch {
                            toast.error("取消失败");
                          }
                        }
                      : undefined
                  }
                />
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function TaskCard({
  task,
  navigate,
  onCancel,
}: {
  task: Task;
  navigate: (path: string) => void;
  onCancel?: () => void;
}) {
  const kindLabel = TASK_KIND_LABELS[task.kind] ?? task.kind;
  const isRunning = task.status === "running" || task.status === "pending";

  return (
    <div className="rounded-md border border-border p-4 space-y-3">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          {STATUS_ICONS[task.status] ?? STATUS_ICONS.pending}
          <div>
            <span className="font-medium text-sm">
              #{task.id} {kindLabel}
            </span>
            {task.target_type === "problem" && (
              <button
                onClick={() => navigate(`/problems/${task.target_id}`)}
                className="ml-2 text-xs text-blue-500 hover:underline"
              >
                查看题目
              </button>
            )}
          </div>
        </div>
        <div className="flex items-center gap-2">
          {statusBadge(task.status)}
          <span className="text-xs text-muted-foreground">
            {timeAgo(task.created_at)}
          </span>
          {task.started_at && (
            <span className="text-xs text-muted-foreground">
              用时 {formatElapsed(task)}
            </span>
          )}
          {isRunning && onCancel && (
            <Button
              variant="ghost"
              size="sm"
              className="h-6 w-6 p-0 text-muted-foreground hover:text-red-400"
              title="取消任务"
              onClick={(e) => {
                e.stopPropagation();
                onCancel();
              }}
            >
              <X className="h-3.5 w-3.5" />
            </Button>
          )}
        </div>
      </div>

      {/* Agents */}
      {task.progress.agents && task.progress.agents.length > 0 && (
        <div className="space-y-2 pl-2">
          {task.progress.agents.map((agent) => (
            <AgentRow key={agent.id} agent={agent} taskStatus={task.status} />
          ))}
        </div>
      )}

      {/* Error */}
      {task.status === "failed" && task.error && (
        <p className="text-sm text-red-400 pl-2">{task.error}</p>
      )}

      {/* Result summary */}
      {task.status === "completed" && task.result && (
        <div className="flex gap-3 text-xs text-muted-foreground pl-2">
          {typeof task.result.extracted_templates === "number" && (
            <span className="flex items-center gap-1"><Wrench className="h-3 w-3" /> 模板 ×{task.result.extracted_templates}</span>
          )}
          {typeof task.result.matched_templates === "number" && task.result.matched_templates > 0 && (
            <span className="flex items-center gap-1"><CheckCircle2 className="h-3 w-3" /> 复用 ×{task.result.matched_templates}</span>
          )}
          {typeof task.result.extracted_errors === "number" && (
            <span className="flex items-center gap-1"><AlertTriangle className="h-3 w-3" /> 错误 ×{task.result.extracted_errors}</span>
          )}
          {typeof task.result.extracted_knowledge === "number" && (
            <span className="flex items-center gap-1"><BookOpen className="h-3 w-3" /> 知识 ×{task.result.extracted_knowledge}</span>
          )}
          {typeof task.result.submissions_analyzed === "number" && (
            <span className="flex items-center gap-1"><Activity className="h-3 w-3" /> 分析了 {task.result.submissions_analyzed} 条提交</span>
          )}
        </div>
      )}
    </div>
  );
}

function AgentRow({ agent, taskStatus }: { agent: AgentProgress; taskStatus?: string }) {
  const hasSteps = agent.steps.length > 0;
  // When the parent task is done (failed/cancelled/completed), no agent should spin.
  const isEffectivelyDone = taskStatus === "failed" || taskStatus === "cancelled" || taskStatus === "completed";
  const displayStatus = isEffectivelyDone && agent.status === "running" ? "failed" : agent.status;

  return (
    <div className="space-y-1">
      <div className="flex items-center gap-2 text-sm">
        {agentIcon(agent.id)}
        {STATUS_ICONS[displayStatus] ?? STATUS_ICONS.pending}
        <span className={displayStatus === "pending" ? "text-muted-foreground/50" : ""}>
          {agent.name}
        </span>
        {agent.message && (
          <span className="text-xs text-muted-foreground">
            {agent.message}
          </span>
        )}
      </div>

      {/* Steps */}
      {hasSteps && displayStatus !== "pending" && (
        <div className="pl-9 space-y-0.5">
          {agent.steps.map((step, i) => {
            // Also stop step spinners when task is done
            const stepStatus = isEffectivelyDone && step.status === "running" ? "failed" : step.status;
            return (
            <div key={i} className="flex items-center gap-1.5 text-xs">
              {stepStatus === "completed" ? (
                <CheckCircle2 className="h-3 w-3 text-green-500" />
              ) : stepStatus === "running" ? (
                <Loader2 className="h-3 w-3 animate-spin text-blue-400" />
              ) : stepStatus === "failed" ? (
                <XCircle className="h-3 w-3 text-red-400" />
              ) : (
                <Clock className="h-3 w-3 text-muted-foreground/30" />
              )}
              <span className={
                stepStatus === "completed" ? "text-muted-foreground" :
                stepStatus === "running" ? "text-blue-300" :
                stepStatus === "failed" ? "text-red-400" : "text-muted-foreground/40"
              }>
                {step.label}
              </span>
              {step.detail && (
                <span className="text-muted-foreground/60">({step.detail})</span>
              )}
            </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
