import { create } from "zustand";
import type { Task, Reminder } from "@/lib/types";
import { taskApi } from "@/lib/tauri";

interface TaskState {
  tasks:     Task[];
  reminders: Reminder[];
  loading:   boolean;
  load:      ()                => Promise<void>;
  addTask:   (t: Task)         => void;
  updateTask:(t: Task)         => void;
  removeTask:(id: string)      => void;
  setReminders:(r: Reminder[]) => void;
}

export const useTaskStore = create<TaskState>((set, get) => ({
  tasks:     [],
  reminders: [],
  loading:   false,

  load: async () => {
    set({ loading: true });
    try {
      const [tasks, reminders] = await Promise.all([
        taskApi.list(),
        taskApi.listReminders(),
      ]);
      set({ tasks, reminders, loading: false });
    } catch (e) {
      console.error("Failed to load tasks", e);
      set({ loading: false });
    }
  },

  addTask:   (t) => set((s) => ({ tasks: [t, ...s.tasks] })),
  updateTask:(t) => set((s) => ({ tasks: s.tasks.map((x) => (x.id === t.id ? t : x)) })),
  removeTask:(id)=> set((s) => ({ tasks: s.tasks.filter((x) => x.id !== id) })),
  setReminders:(reminders) => set({ reminders }),
}));
