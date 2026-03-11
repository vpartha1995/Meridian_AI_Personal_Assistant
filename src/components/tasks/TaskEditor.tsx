import { useState } from "react";
import { X } from "lucide-react";
import type { Task, Priority } from "@/lib/types";
import { cn } from "@/lib/utils";

interface Props {
  task?:    Task;
  onSave:  (data: { title: string; description?: string; due_at?: string; priority: Priority }) => Promise<void>;
  onClose: () => void;
}

export function TaskEditor({ task, onSave, onClose }: Props) {
  const [title,    setTitle]    = useState(task?.title       ?? "");
  const [desc,     setDesc]     = useState(task?.description ?? "");
  const [dueAt,    setDueAt]    = useState(task?.due_at?.slice(0, 16) ?? "");
  const [priority, setPriority] = useState<Priority>(task?.priority ?? "medium");
  const [saving,   setSaving]   = useState(false);

  async function handleSave() {
    if (!title.trim()) return;
    setSaving(true);
    try {
      await onSave({ title: title.trim(), description: desc || undefined, due_at: dueAt || undefined, priority });
      onClose();
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <div className="bg-card border border-border rounded-2xl w-full max-w-md shadow-2xl">
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-4 border-b border-border">
          <h3 className="font-semibold text-foreground">{task ? "Edit Task" : "New Task"}</h3>
          <button onClick={onClose} className="text-muted-foreground hover:text-foreground transition-colors">
            <X size={16} />
          </button>
        </div>

        {/* Body */}
        <div className="p-5 space-y-4">
          <div className="space-y-1.5">
            <label className="text-xs font-medium text-muted-foreground">Title *</label>
            <input
              autoFocus
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="Task title…"
              className="w-full bg-input border border-border rounded-lg px-3 py-2.5 text-sm outline-none focus:ring-1 focus:ring-ring text-foreground placeholder:text-muted-foreground"
            />
          </div>

          <div className="space-y-1.5">
            <label className="text-xs font-medium text-muted-foreground">Description</label>
            <textarea
              value={desc}
              onChange={(e) => setDesc(e.target.value)}
              rows={3}
              placeholder="Add more detail…"
              className="w-full bg-input border border-border rounded-lg px-3 py-2.5 text-sm outline-none focus:ring-1 focus:ring-ring text-foreground placeholder:text-muted-foreground resize-none"
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1.5">
              <label className="text-xs font-medium text-muted-foreground">Due Date</label>
              <input
                type="datetime-local"
                value={dueAt}
                onChange={(e) => setDueAt(e.target.value)}
                className="w-full bg-input border border-border rounded-lg px-3 py-2 text-sm outline-none focus:ring-1 focus:ring-ring text-foreground"
              />
            </div>

            <div className="space-y-1.5">
              <label className="text-xs font-medium text-muted-foreground">Priority</label>
              <select
                value={priority}
                onChange={(e) => setPriority(e.target.value as Priority)}
                className="w-full bg-input border border-border rounded-lg px-3 py-2 text-sm outline-none focus:ring-1 focus:ring-ring text-foreground"
              >
                <option value="high">High</option>
                <option value="medium">Medium</option>
                <option value="low">Low</option>
              </select>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3 px-5 py-4 border-t border-border">
          <button onClick={onClose} className="px-4 py-2 text-sm text-muted-foreground hover:text-foreground transition-colors">
            Cancel
          </button>
          <button
            onClick={handleSave}
            disabled={!title.trim() || saving}
            className="px-4 py-2 bg-primary hover:bg-primary/90 text-primary-foreground rounded-lg text-sm font-medium transition-colors disabled:opacity-50"
          >
            {saving ? "Saving…" : task ? "Save Changes" : "Create Task"}
          </button>
        </div>
      </div>
    </div>
  );
}
