import { create } from "zustand";
import { tasksApi } from "../api";
import type { Task } from "../api";

interface TaskState {
  tasks: Task[];
  activeTaskId: number | null;
  polling: boolean;
  pollInterval: ReturnType<typeof setInterval> | null;
  initialized: boolean;

  fetchTasks: () => Promise<void>;
  init: () => void;
  setActiveTask: (id: number | null) => void;
  startPolling: () => void;
  stopPolling: () => void;
}

export const useTaskStore = create<TaskState>((set, get) => ({
  tasks: [],
  activeTaskId: null,
  polling: false,
  pollInterval: null,
  initialized: false,

  /** Load tasks on first call (e.g., when TaskIndicator mounts). */
  init: () => {
    if (get().initialized) return;
    set({ initialized: true });
    get().fetchTasks();
  },

  fetchTasks: async () => {
    try {
      const tasks = await tasksApi.list();
      set({ tasks });

      // Auto-stop polling if no active tasks
      const hasActive = tasks.some(
        (t) => t.status === "pending" || t.status === "running"
      );
      if (!hasActive && get().polling) {
        get().stopPolling();
      }
    } catch {
      // Ignore fetch errors during polling
    }
  },

  setActiveTask: (id) => set({ activeTaskId: id }),

  startPolling: () => {
    if (get().polling) return;
    set({ polling: true });
    get().fetchTasks();
    const interval = setInterval(() => get().fetchTasks(), 2000);
    set({ pollInterval: interval });
  },

  stopPolling: () => {
    const { pollInterval } = get();
    if (pollInterval) clearInterval(pollInterval);
    set({ polling: false, pollInterval: null });
  },
}));
