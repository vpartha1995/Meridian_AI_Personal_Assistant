import { create } from "zustand";
import type { AppSettings } from "@/lib/types";
import { settingsApi } from "@/lib/tauri";

interface SettingsState {
  settings: AppSettings | null;
  theme:    "dark" | "light";
  load:     () => Promise<void>;
  save:     (s: AppSettings) => Promise<void>;
  setTheme: (t: "dark" | "light") => void;
}

const defaultSettings: AppSettings = {
  theme:               "dark",
  scan_interval_mins:  60,
  working_hours_start: 9,
  working_hours_end:   18,
  health_nudges:       true,
  notifications:       true,
  news_topics:         ["technology", "ai"],
  user_name:           "User",
  avatar_url:          null,
};

export const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: null,
  theme:    "dark",

  load: async () => {
    try {
      const [settings, theme] = await Promise.all([
        settingsApi.get(),
        settingsApi.getTheme(),
      ]);
      set({
        settings: settings ?? defaultSettings,
        theme: (theme as "dark" | "light") ?? "dark",
      });
    } catch {
      set({ settings: defaultSettings });
    }
  },

  save: async (settings) => {
    await settingsApi.save(settings);
    set({ settings });
  },

  setTheme: async (theme) => {
    set({ theme });
    // Update HTML class
    document.documentElement.classList.toggle("dark", theme === "dark");
    await settingsApi.setTheme(theme);
  },
}));
