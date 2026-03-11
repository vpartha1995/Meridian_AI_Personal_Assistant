import { useState } from "react";
import { ArrowLeft, Check } from "lucide-react";
import { taskApi } from "@/lib/tauri";
import { useTaskStore } from "@/store/taskStore";

interface Props {
  onBack: () => void;
  type?:  "note" | "reminder";
}

export function QuickCapture({ onBack, type = "note" }: Props) {
  const [text,    setText]    = useState("");
  const [fireAt,  setFireAt]  = useState("");
  const [saved,   setSaved]   = useState(false);
  const { addTask } = useTaskStore();

  async function handleSave() {
    if (!text.trim()) return;
    if (type === "reminder") {
      await taskApi.createReminder({ task_id: null, fire_at: fireAt || new Date().toISOString(), message: text });
    } else {
      const task = await taskApi.create({ title: text, description: null, due_at: null, priority: "medium", source_url: null });
      addTask(task);
    }
    setSaved(true);
    setTimeout(onBack, 1000);
  }

  const label = type === "reminder" ? "Set Reminder" : "Capture Note";

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-2">
        <button onClick={onBack} className="text-white/40 hover:text-white/80 transition-colors">
          <ArrowLeft size={14} />
        </button>
        <p className="text-sm font-medium text-white">{label}</p>
      </div>

      <textarea
        autoFocus
        value={text}
        onChange={(e) => setText(e.target.value)}
        placeholder={type === "reminder" ? "What do you want to be reminded of?" : "What's on your mind?"}
        rows={4}
        className="w-full bg-white/5 border border-white/10 rounded-xl px-4 py-3 text-sm text-white placeholder:text-white/30 outline-none focus:border-indigo-500/50 resize-none"
      />

      {type === "reminder" && (
        <input
          type="datetime-local"
          value={fireAt}
          onChange={(e) => setFireAt(e.target.value)}
          className="w-full bg-white/5 border border-white/10 rounded-xl px-4 py-2.5 text-sm text-white outline-none focus:border-indigo-500/50"
        />
      )}

      <button
        onClick={handleSave}
        disabled={!text.trim() || saved}
        className="w-full flex items-center justify-center gap-2 py-2.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-40 text-white text-sm font-medium transition-colors"
      >
        {saved ? <><Check size={14} /> Saved!</> : label}
      </button>
    </div>
  );
}
