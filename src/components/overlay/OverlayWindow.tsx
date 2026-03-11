import { useState } from "react";
import { motion } from "framer-motion";
import { X, Mail, FileText, Bell, Wand2, Search } from "lucide-react";
import { windowApi } from "@/lib/tauri";
import { QuickCapture } from "./QuickCapture";
import { EmailAssistant } from "./EmailAssistant";
import { cn } from "@/lib/utils";

type Mode = "home" | "capture" | "email" | "remind";

export function OverlayWindow() {
  const [mode, setMode] = useState<Mode>("home");
  const [query, setQuery] = useState("");

  return (
    <div className="w-full h-full glass-dark rounded-2xl overflow-hidden flex flex-col select-none">
      {/* Header */}
      <div
        data-tauri-drag-region
        className="flex items-center justify-between px-4 py-3 border-b border-white/10"
      >
        <div className="flex items-center gap-2">
          <div className="w-6 h-6 rounded-lg bg-gradient-to-br from-indigo-600 to-violet-600 flex items-center justify-center">
            <span className="text-xs font-bold text-white">M</span>
          </div>
          <span className="text-sm font-semibold text-white">Meridian</span>
        </div>
        <button
          onClick={() => windowApi.hideOverlay()}
          className="text-white/40 hover:text-white/80 transition-colors"
        >
          <X size={14} />
        </button>
      </div>

      {/* Search / ask bar */}
      {mode === "home" && (
        <div className="px-4 py-3 border-b border-white/10">
          <input
            autoFocus
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Ask anything or capture a note…"
            className="w-full bg-white/5 border border-white/10 rounded-xl px-4 py-2.5 text-sm text-white placeholder:text-white/30 outline-none focus:border-indigo-500/50 focus:ring-1 focus:ring-indigo-500/30 transition-all"
          />
        </div>
      )}

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-4">
        {mode === "home" && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            className="space-y-4"
          >
            <div>
              <p className="text-xs text-white/40 uppercase tracking-widest mb-2">Quick Actions</p>
              <div className="grid grid-cols-2 gap-2">
                {[
                  { mode: "capture" as Mode, icon: FileText, label: "Capture Note",  color: "from-blue-600 to-blue-500"   },
                  { mode: "email"   as Mode, icon: Mail,     label: "Draft Email",   color: "from-violet-600 to-violet-500"},
                  { mode: "remind"  as Mode, icon: Bell,     label: "Set Reminder",  color: "from-orange-600 to-orange-500"},
                  { mode: "email"   as Mode, icon: Wand2,    label: "Rephrase Text", color: "from-pink-600 to-pink-500"   },
                ].map((item) => (
                  <button
                    key={item.label}
                    onClick={() => setMode(item.mode)}
                    className="flex items-center gap-2.5 p-3 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-left transition-all group"
                  >
                    <div className={cn("w-8 h-8 rounded-lg bg-gradient-to-br flex items-center justify-center flex-shrink-0", item.color)}>
                      <item.icon size={14} className="text-white" />
                    </div>
                    <span className="text-xs text-white/70 group-hover:text-white transition-colors">{item.label}</span>
                  </button>
                ))}
              </div>
            </div>
          </motion.div>
        )}

        {mode === "capture" && (
          <QuickCapture onBack={() => setMode("home")} />
        )}
        {mode === "email" && (
          <EmailAssistant onBack={() => setMode("home")} />
        )}
        {mode === "remind" && (
          <QuickCapture onBack={() => setMode("home")} type="reminder" />
        )}
      </div>
    </div>
  );
}
