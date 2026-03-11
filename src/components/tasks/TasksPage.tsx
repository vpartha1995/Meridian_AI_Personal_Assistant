import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Plus, CheckCircle2, Circle, Trash2, Calendar } from "lucide-react";
import { useTaskStore } from "@/store/taskStore";
import { taskApi } from "@/lib/tauri";
import { PriorityBadge } from "@/components/shared/PriorityBadge";
import { TaskEditor } from "./TaskEditor";
import { formatRelativeTime, cn } from "@/lib/utils";
import type { Task } from "@/lib/types";

export function TasksPage() {
  const { tasks, updateTask, removeTask, addTask } = useTaskStore();
  const [editing,      setEditing]      = useState<Task | null>(null);
  const [showComplete, setShowComplete] = useState(false);
  const [creating,     setCreating]     = useState(false);

  const pending   = tasks.filter((t) => !t.completed);
  const completed = tasks.filter((t) => t.completed);

  async function toggleTask(t: Task) {
    const updated = { ...t, completed: !t.completed };
    await taskApi.update(updated);
    updateTask(updated);
  }

  async function deleteTask(id: string) {
    await taskApi.delete(id);
    removeTask(id);
  }

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="flex-shrink-0 flex items-center justify-between px-6 py-4 border-b border-border">
        <div>
          <h2 className="text-lg font-semibold text-foreground">Tasks</h2>
          <p className="text-xs text-muted-foreground">{pending.length} pending · {completed.length} completed</p>
        </div>
        <button
          onClick={() => setCreating(true)}
          className="flex items-center gap-2 px-4 py-2 bg-primary hover:bg-primary/90 text-primary-foreground rounded-lg text-sm font-medium transition-colors"
        >
          <Plus size={14} /> New Task
        </button>
      </div>

      {/* Body */}
      <div className="flex-1 overflow-y-auto p-6">
        <div className="max-w-2xl space-y-6">
          {/* Pending */}
          <section className="space-y-2">
            <h3 className="text-xs font-semibold text-muted-foreground uppercase tracking-widest">Pending</h3>
            <AnimatePresence>
              {pending.map((t) => (
                <TaskRow key={t.id} task={t} onToggle={toggleTask} onEdit={setEditing} onDelete={deleteTask} />
              ))}
            </AnimatePresence>
            {pending.length === 0 && (
              <p className="text-sm text-muted-foreground italic text-center py-8">
                All done! 🎉 You have no pending tasks.
              </p>
            )}
          </section>

          {/* Completed */}
          {completed.length > 0 && (
            <section className="space-y-2">
              <button
                onClick={() => setShowComplete(!showComplete)}
                className="text-xs font-semibold text-muted-foreground uppercase tracking-widest hover:text-foreground transition-colors"
              >
                Completed ({completed.length}) {showComplete ? "▲" : "▼"}
              </button>
              {showComplete && (
                <AnimatePresence>
                  {completed.map((t) => (
                    <TaskRow key={t.id} task={t} onToggle={toggleTask} onEdit={setEditing} onDelete={deleteTask} />
                  ))}
                </AnimatePresence>
              )}
            </section>
          )}
        </div>
      </div>

      {/* Task editor dialog */}
      {(editing || creating) && (
        <TaskEditor
          task={editing ?? undefined}
          onSave={async (data) => {
            if (editing) {
              const updated: Task = {
                ...editing,
                title: data.title,
                description: data.description ?? editing.description,
                due_at: data.due_at ?? editing.due_at,
                priority: data.priority,
              };
              await taskApi.update(updated);
              updateTask(updated);
            } else {
              const task = await taskApi.create({ ...data, description: data.description ?? null, due_at: data.due_at ?? null, source_url: null });
              addTask(task);
            }
            setEditing(null);
            setCreating(false);
          }}
          onClose={() => { setEditing(null); setCreating(false); }}
        />
      )}
    </div>
  );
}

function TaskRow({ task, onToggle, onEdit, onDelete }: {
  task:     Task;
  onToggle: (t: Task) => void;
  onEdit:   (t: Task) => void;
  onDelete: (id: string) => void;
}) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 6 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, height: 0 }}
      className="flex items-center gap-3 p-3 rounded-xl border border-border hover:border-primary/20 bg-card hover:bg-card/80 group transition-all"
    >
      <button onClick={() => onToggle(task)} className="flex-shrink-0 text-muted-foreground hover:text-primary transition-colors">
        {task.completed ? <CheckCircle2 size={16} className="text-green-400" /> : <Circle size={16} />}
      </button>

      <div className="flex-1 min-w-0 cursor-pointer" onClick={() => onEdit(task)}>
        <p className={cn("text-sm text-foreground truncate", task.completed && "line-through text-muted-foreground")}>
          {task.title}
        </p>
        {task.due_at && (
          <div className="flex items-center gap-1 mt-0.5">
            <Calendar size={10} className="text-muted-foreground" />
            <p className="text-xs text-muted-foreground">{formatRelativeTime(task.due_at)}</p>
          </div>
        )}
      </div>

      <PriorityBadge priority={task.priority as any} label />

      <button
        onClick={() => onDelete(task.id)}
        className="opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-destructive transition-all"
      >
        <Trash2 size={14} />
      </button>
    </motion.div>
  );
}
