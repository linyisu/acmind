import { useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { useTaskStore } from "@/lib/stores/task";
import type { Task, AgentProgress } from "@/lib/api";
import { Loader2, CheckCircle2, XCircle, Clock, Bell, Search, Wrench, AlertTriangle, BookOpen, Database } from "lucide-react";

const AGENT_ICONS: Record<string, React.ReactNode> = {
  classifier: <Search className="h-3 w-3" />,
  template:   <Wrench className="h-3 w-3" />,
  error:      <AlertTriangle className="h-3 w-3" />,
  knowledge:  <BookOpen className="h-3 w-3" />,
  saver:      <Database className="h-3 w-3" />,
};

const STATUS_ICONS: Record<string, React.ReactNode> = {
  completed: <CheckCircle2 className="h-3 w-3 text-green-500" />,
  running: <Loader2 className="h-3 w-3 text-blue-400 animate-spin" />,
  failed: <XCircle className="h-3 w-3 text-red-500" />,
  cancelled: <XCircle className="h-3 w-3 text-orange-400" />,
  pending: <Clock className="h-3 w-3 text-muted-foreground/40" />,
};

const TASK_KIND_LABELS: Record<string, string> = {
  ai_full_analysis: "AI 全量分析",
};

function formatElapsed(task: Task): string {
  if (!task.started_at) return "";
  const start = new Date(task.started_at).getTime();
  const end = task.completed_at
    ? new Date(task.completed_at).getTime()
    : Date.now();
  const secs = Math.floor((end - start) / 1000);
  if (secs < 60) return `${secs}秒`;
  return `${Math.floor(secs / 60)}分${secs % 60}秒`;
}

export function TaskIndicator() {
  const tasks = useTaskStore((s) => s.tasks);
  const polling = useTaskStore((s) => s.polling);
  const init = useTaskStore((s) => s.init);
  const navigate = useNavigate();
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    init();
  }, [init]);

  useEffect(() => {
    function onClick(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener("mousedown", onClick);
    return () => document.removeEventListener("mousedown", onClick);
  }, []);

  const activeTasks = tasks.filter(
    (t) => t.status === "pending" || t.status === "running"
  );
  const recentTasks = tasks.slice(0, 10);

  if (!polling && tasks.length === 0) return null;

  return (
    <div className="relative" ref={ref}>
      <button
        onClick={() => setOpen(!open)}
        className="flex items-center gap-1.5 px-2 py-1.5 rounded-md text-sm hover:bg-accent transition-colors"
      >
        {activeTasks.length > 0 ? (
          <>
            <Loader2 className="h-4 w-4 animate-spin text-blue-400" />
            <span className="text-blue-400 font-medium">{activeTasks.length}</span>
          </>
        ) : (
          <>
            <Bell className="h-4 w-4 text-muted-foreground" />
            <span className="text-muted-foreground">{tasks.length}</span>
          </>
        )}
      </button>

      {open && (
        <div className="absolute right-0 top-full mt-1 w-80 max-h-96 overflow-y-auto rounded-lg border border-border bg-card shadow-lg z-50">
          <div className="p-2 border-b border-border flex items-center justify-between">
            <span className="text-xs font-medium text-muted-foreground">任务</span>
            <button
              onClick={() => { navigate("/tasks"); setOpen(false); }}
              className="text-xs text-blue-400 hover:underline"
            >
              查看全部
            </button>
          </div>
          {recentTasks.length > 0 ? (
            recentTasks.map((t) => (
              <TaskRow
                key={t.id}
                task={t}
                onClick={
                  t.target_type === "problem"
                    ? () => {
                        navigate(`/problems/${t.target_id}`, { state: { taskId: t.id } });
                        setOpen(false);
                      }
                    : undefined
                }
              />
            ))
          ) : (
            <p className="p-3 text-xs text-muted-foreground text-center">暂无任务</p>
          )}
        </div>
      )}
    </div>
  );
}

function TaskRow({ task, onClick }: { task: Task; onClick?: () => void }) {
  const kindLabel = TASK_KIND_LABELS[task.kind] ?? task.kind;
  const isActive = task.status === "running" || task.status === "pending";
  const runningAgent = isActive
    ? task.progress.agents?.find((a: AgentProgress) => a.status === "running")
    : undefined;

  return (
    <button
      className="w-full text-left px-3 py-2 hover:bg-accent/50 transition-colors border-b border-border last:border-0"
      onClick={onClick}
    >
      <div className="flex items-center gap-2">
        {STATUS_ICONS[task.status] ?? STATUS_ICONS.pending}
        <span className="text-sm font-medium truncate">{kindLabel}</span>
        <span className="text-xs text-muted-foreground ml-auto">{formatElapsed(task)}</span>
      </div>
      {runningAgent && (
        <p className="text-xs text-blue-300 mt-0.5 ml-5 truncate flex items-center gap-1">
          {AGENT_ICONS[runningAgent.id] ?? null}
          {runningAgent.name}: {runningAgent.message}
        </p>
      )}
      {task.status === "completed" && task.result && (
        <p className="text-xs text-muted-foreground mt-0.5 ml-5">
          分析完成
          {task.result.extracted_templates != null && ` · ${task.result.extracted_templates} 模板`}
          {task.result.extracted_errors != null && ` · ${task.result.extracted_errors} 错误`}
          {task.result.extracted_knowledge != null && ` · ${task.result.extracted_knowledge} 知识`}
        </p>
      )}
      {task.status === "failed" && task.error && (
        <p className="text-xs text-red-400 mt-0.5 ml-5 truncate">
          {task.error.replace(/^internal error:\s*/i, "")}
        </p>
      )}
    </button>
  );
}
