import { useState } from "react";
import { Plus, CheckCircle2, Circle } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { PriorityBadge } from "@/components/shared/PriorityBadge";
import { useTaskStore } from "@/store/taskStore";
import { taskApi } from "@/lib/tauri";
import { cn, formatRelativeTime } from "@/lib/utils";
import type { Task } from "@/lib/types";

interface Props { tasks: Task[] }

export function TaskWidget({ tasks }: Props) {
  const { updateTask, removeTask } = useTaskStore();
  const [adding, setAdding] = useState(false);
  const [newTitle, setNewTitle] = useState("");

  async function toggleTask(t: Task) {
    const updated = { ...t, completed: !t.completed };
    await taskApi.update(updated);
    updateTask(updated);
  }

  async function quickAdd() {
    if (!newTitle.trim()) return;
    const task = await taskApi.create({ title: newTitle.trim(), description: null, due_at: null, priority: "medium", source_url: null });
    useTaskStore.getState().addTask(task);
    setNewTitle("");
    setAdding(false);
  }

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center justify-between">
        <h3 className="text-xs font-semibold text-muted-foreground uppercase tracking-widest">
          Today's Tasks
        </h3>
        <button
          onClick={() => setAdding(true)}
          className="text-muted-foreground hover:text-primary transition-colors"
          title="Add task"
        >
          <Plus size={14} />
        </button>
      </div>

      {/* Quick add */}
      <AnimatePresence>
        {adding && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            className="overflow-hidden"
          >
            <input
              autoFocus
              value={newTitle}
              onChange={(e) => setNewTitle(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") quickAdd();
                if (e.key === "Escape") setAdding(false);
              }}
              placeholder="Task title…"
              className="w-full bg-input border border-border rounded-lg px-3 py-2 text-sm outline-none focus:ring-1 focus:ring-primary text-foreground placeholder:text-muted-foreground"
            />
            <p className="text-xs text-muted-foreground mt-1">Enter to save, Esc to cancel</p>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Task list */}
      <div className="space-y-1">
        <AnimatePresence>
          {tasks.slice(0, 8).map((t) => (
            <motion.div
              key={t.id}
              initial={{ opacity: 0, x: 10 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: -10 }}
              className="flex items-start gap-2 p-2 rounded-lg hover:bg-muted/50 group cursor-default transition-colors"
            >
              <button
                onClick={() => toggleTask(t)}
                className="mt-0.5 flex-shrink-0 text-muted-foreground hover:text-primary transition-colors"
              >
                <Circle size={14} />
              </button>
              <div className="flex-1 min-w-0">
                <p className="text-xs text-foreground leading-snug truncate">{t.title}</p>
                {t.due_at && (
                  <p className="text-xs text-muted-foreground mt-0.5">
                    {formatRelativeTime(t.due_at)}
                  </p>
                )}
              </div>
              <PriorityBadge priority={t.priority as any} />
            </motion.div>
          ))}
        </AnimatePresence>
        {tasks.length === 0 && (
          <p className="text-xs text-muted-foreground italic text-center py-4">
            No pending tasks 🎉
          </p>
        )}
      </div>
    </div>
  );
}
