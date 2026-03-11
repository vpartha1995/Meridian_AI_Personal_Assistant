import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import type {
  DailySummary,
  IntegrationInfo,
  ConnectionStatus,
  Task,
  NewTask,
  Reminder,
  NewReminder,
  AppSettings,
  OllamaStatus,
} from "./types";

// Typed invoke wrapper — all IPC calls go through here
async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return tauriInvoke<T>(cmd, args);
}

// ── Integrations ──────────────────────────────────────────────────────────────
export const integrationApi = {
  list:       ()           => invoke<IntegrationInfo[]>("list_integrations"),
  connect:    (id: string) => invoke<void>("connect_integration", { id }),
  disconnect: (id: string) => invoke<void>("disconnect_integration", { id }),
  status:     (id: string) => invoke<ConnectionStatus>("get_integration_status", { id }),
  sync:       (id?: string)=> invoke<number>("trigger_sync", { id: id ?? null }),
};

// ── Summary ───────────────────────────────────────────────────────────────────
export const summaryApi = {
  getToday:  ()              => invoke<DailySummary | null>("get_todays_summary"),
  generate:  ()              => invoke<DailySummary>("generate_daily_summary"),
  history:   (limit?: number)=> invoke<DailySummary[]>("get_summary_history", { limit: limit ?? 7 }),
};

// ── Tasks ─────────────────────────────────────────────────────────────────────
export const taskApi = {
  list:         (includeCompleted = false) =>
    invoke<Task[]>("list_tasks", { includeCompleted }),
  create:       (req: NewTask)   => invoke<Task>("create_task",  { req }),
  update:       (task: Task)     => invoke<void>("update_task",  { task }),
  delete:       (id: string)     => invoke<void>("delete_task",  { id }),
  complete:     (id: string)     => invoke<void>("complete_task",{ id }),
  listReminders:()               => invoke<Reminder[]>("list_reminders"),
  createReminder:(req: NewReminder) => invoke<Reminder>("create_reminder", { req }),
  deleteReminder:(id: string)    => invoke<void>("delete_reminder", { id }),
};

// ── AI ────────────────────────────────────────────────────────────────────────
export const aiApi = {
  status:       ()                             => invoke<OllamaStatus>("check_ollama_status"),
  draftEmail:   (context: string, instructions: string) =>
    invoke<string>("draft_email",    { context, instructions }),
  rephrase:     (text: string, tone: string)   =>
    invoke<string>("rephrase_text",  { text, tone }),
  ask:          (question: string, context: string) =>
    invoke<string>("ask_assistant",  { question, context }),
  summarize:    (text: string)                 =>
    invoke<string>("summarize_text", { text }),
};

// ── Settings ──────────────────────────────────────────────────────────────────
export const settingsApi = {
  get:            ()                   => invoke<AppSettings>("get_settings"),
  save:           (settings: AppSettings) => invoke<void>("save_settings", { settings }),
  getTheme:       ()                   => invoke<string>("get_theme"),
  setTheme:       (theme: string)      => invoke<void>("set_theme", { theme }),
  logHealthAction:(action: string)     => invoke<void>("log_health_action", { action }),
};

// ── Window ────────────────────────────────────────────────────────────────────
export const windowApi = {
  showOverlay: () => invoke<void>("show_overlay"),
  hideOverlay: () => invoke<void>("hide_overlay"),
  openUrl:     (url: string) => invoke<void>("open_url", { url }),
};
