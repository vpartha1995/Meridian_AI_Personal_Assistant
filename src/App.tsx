import { useEffect, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { useUiStore } from "@/store/uiStore";
import { useSettingsStore } from "@/store/settingsStore";
import { useTaskStore } from "@/store/taskStore";
import { DashboardPage } from "@/components/dashboard/DashboardPage";
import { SettingsPage } from "@/components/settings/SettingsPage";
import { TasksPage } from "@/components/tasks/TasksPage";
import { Sidebar } from "@/components/shared/Sidebar";
import { OnboardingPage } from "@/components/onboarding/OnboardingPage";
import { OverlayWindow } from "@/components/overlay/OverlayWindow";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

const isOverlayWindow = window.location.pathname === "/overlay";

export default function App() {
  const { activePage } = useUiStore();
  const { load: loadSettings, settings } = useSettingsStore();
  const { load: loadTasks }              = useTaskStore();
  const [initialized, setInitialized]   = useState(false);

  useEffect(() => {
    if (isOverlayWindow) {
      setInitialized(true);
      return;
    }
    Promise.all([loadSettings(), loadTasks()]).finally(() => {
      setInitialized(true);
      // Apply theme
      document.documentElement.classList.toggle(
        "dark",
        (settings?.theme ?? "dark") === "dark"
      );
    });
  }, []);

  if (!initialized) return null;

  // Overlay window — shown as floating assistant
  if (isOverlayWindow) return <OverlayWindow />;

  // First-run onboarding — if no user name set
  if (settings && settings.user_name === "User" && !localStorage.getItem("onboarded")) {
    return <OnboardingPage onComplete={() => { localStorage.setItem("onboarded", "1"); }} />;
  }

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-background text-foreground">
      <Sidebar />

      <main className="flex-1 overflow-hidden">
        <AnimatePresence mode="wait">
          <motion.div
            key={activePage}
            initial={{ opacity: 0, x: 10 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -10 }}
            transition={{ duration: 0.2 }}
            className="h-full"
          >
            {activePage === "dashboard" && <DashboardPage />}
            {activePage === "tasks"     && <TasksPage />}
            {activePage === "settings"  && <SettingsPage />}
          </motion.div>
        </AnimatePresence>
      </main>
    </div>
  );
}
