import { motion } from "framer-motion";
import { LayoutDashboard, CheckSquare, Settings, History, RefreshCw } from "lucide-react";
import { useUiStore } from "@/store/uiStore";
import { useSettingsStore } from "@/store/settingsStore";
import { cn } from "@/lib/utils";

const navItems = [
  { id: "dashboard", label: "Dashboard", icon: LayoutDashboard },
  { id: "tasks",     label: "Tasks",     icon: CheckSquare },
  { id: "history",   label: "History",   icon: History },
  { id: "settings",  label: "Settings",  icon: Settings },
] as const;

export function Sidebar() {
  const { activePage, setPage } = useUiStore();
  const { settings }            = useSettingsStore();

  return (
    <aside className="w-16 flex-shrink-0 bg-surface-raised border-r border-border flex flex-col items-center py-4 gap-2">
      {/* Logo */}
      <div className="w-9 h-9 rounded-xl bg-gradient-to-br from-indigo-600 to-violet-600 flex items-center justify-center mb-4 shadow-glow-primary">
        <span className="text-base font-bold text-white">M</span>
      </div>

      {/* Navigation */}
      <nav className="flex flex-col gap-1 flex-1">
        {navItems.map((item) => {
          const Icon = item.icon;
          const active = activePage === item.id;
          return (
            <button
              key={item.id}
              onClick={() => setPage(item.id as any)}
              title={item.label}
              className={cn(
                "relative w-10 h-10 rounded-xl flex items-center justify-center transition-all",
                "hover:bg-accent/50 group",
                active && "bg-primary/10 text-primary",
                !active && "text-muted-foreground hover:text-foreground"
              )}
            >
              {active && (
                <motion.div
                  layoutId="sidebar-indicator"
                  className="absolute inset-0 bg-primary/10 rounded-xl"
                  transition={{ type: "spring", stiffness: 400, damping: 30 }}
                />
              )}
              <Icon size={18} className="relative z-10" />
              {/* Tooltip */}
              <span className="absolute left-14 px-2 py-1 bg-popover border border-border text-xs rounded-lg
                               opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none
                               whitespace-nowrap shadow-lg z-50">
                {item.label}
              </span>
            </button>
          );
        })}
      </nav>

      {/* Avatar */}
      <div className="mt-auto">
        <div className="w-8 h-8 rounded-full bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center text-xs font-semibold text-white cursor-pointer hover:ring-2 hover:ring-primary transition-all"
             title={settings?.user_name ?? "Profile"}>
          {(settings?.user_name ?? "U")[0].toUpperCase()}
        </div>
      </div>
    </aside>
  );
}
