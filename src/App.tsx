import { useEffect, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { useUiStore } from "@/store/uiStore";
import { useSettingsStore } from "@/store/settingsStore";
import { useTaskStore } from "@/store/taskStore";
import { DashboardPage } from "@/components/dashboard/DashboardPage";
import { SettingsPage } from "@/components/settings/SettingsPage";
import { TasksPage } from "@/components/tasks/TasksPage";
import { HistoryPage } from "@/components/history/HistoryPage";
import { Sidebar } from "@/components/shared/Sidebar";
import { OnboardingPage } from "@/components/onboarding/OnboardingPage";
import { OverlayWindow } from "@/components/overlay/OverlayWindow";
import { isPermissionGranted, requestPermission } from "@tauri-apps/plugin-notification";

const isOverlayWindow = window.location.pathname === "/overlay";

async function ensureNotificationPermission() {
  try {
    const granted = await isPermissionGranted();
    if (!granted) await requestPermission();
  } catch {
    // Notification permission API not available in dev browser environment
  }
}

export default function App() {
  const { activePage } = useUiStore();
  const { load: loadSettings, settings } = useSettingsStore();
  const { load: loadTasks }              = useTaskStore();
  const [initialized, setInitialized]   = useState(false);
  // Track onboarding completion in React state so the page re-renders immediately
  // when onComplete fires. Using only localStorage doesn't trigger a re-render.
  const [onboarded, setOnboarded]        = useState(() => !!localStorage.getItem("onboarded"));

  useEffect(() => {
    if (isOverlayWindow) {
      setInitialized(true);
      return;
    }
    Promise.all([loadSettings(), loadTasks()]).finally(() => {
      setInitialized(true);
      // Apply saved theme class
      document.documentElement.classList.toggle(
        "dark",
        (settings?.theme ?? "dark") === "dark"
      );
      // Request OS notification permission so reminders and nudges can fire
      ensureNotificationPermission();
    });
  }, []);

  if (!initialized) return null;

  // Overlay window — rendered as floating assistant (no sidebar)
  if (isOverlayWindow) return <OverlayWindow />;

  // First-run onboarding — shown only if name was never set AND not yet completed
  if (settings && settings.user_name === "User" && !onboarded) {
    return (
      <OnboardingPage
        onComplete={() => {
          localStorage.setItem("onboarded", "1");
          setOnboarded(true); // triggers re-render → main app shows immediately
        }}
      />
    );
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
            {activePage === "history"   && <HistoryPage />}
            {activePage === "settings"  && <SettingsPage />}
          </motion.div>
        </AnimatePresence>
      </main>
    </div>
  );
}
