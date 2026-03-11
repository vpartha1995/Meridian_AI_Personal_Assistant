// ── Integration types ─────────────────────────────────────────────────────────

export type ConnectionStatus =
  | { status: "connected"; account: string }
  | { status: "disconnected" }
  | { status: "error"; message: string };

export interface IntegrationInfo {
  id:     string;
  name:   string;
  icon:   string;
  status: ConnectionStatus;
}

// ── Summary types ─────────────────────────────────────────────────────────────

export interface PriorityItem {
  title:    string;
  source:   string;
  url:      string | null;
  priority: "high" | "medium" | "low";
  action:   string;
}

export interface DailySummary {
  generated_at:    string;
  email_summary:   string;
  slack_summary:   string;
  jira_summary:    string;
  zoom_summary:    string;
  gchat_summary:   string;
  top_priorities:  PriorityItem[];
  motivational:    string;
  raw_event_count: number;
}

// ── Task types ────────────────────────────────────────────────────────────────

export type Priority = "high" | "medium" | "low";

export interface Task {
  id:              string;
  title:           string;
  description:     string | null;
  due_at:          string | null;
  completed:       boolean;
  priority:        Priority;
  recurrence_rule: string | null;
  source_url:      string | null;
  created_at:      string;
  updated_at:      string;
}

export interface NewTask {
  title:       string;
  description: string | null;
  due_at:      string | null;
  priority:    Priority | null;
  source_url:  string | null;
}

export interface Reminder {
  id:      string;
  task_id: string | null;
  fire_at: string;
  message: string;
  fired:   boolean;
}

export interface NewReminder {
  task_id: string | null;
  fire_at: string;
  message: string;
}

// ── Settings types ────────────────────────────────────────────────────────────

export interface AppSettings {
  theme:               "dark" | "light";
  scan_interval_mins:  number;
  working_hours_start: number;
  working_hours_end:   number;
  health_nudges:       boolean;
  notifications:       boolean;
  news_topics:         string[];
  user_name:           string;
  avatar_url:          string | null;
}

// ── AI types ──────────────────────────────────────────────────────────────────

export interface OllamaStatus {
  available:   boolean;
  model_ready: boolean;
  model_name:  string;
}
