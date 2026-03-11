import { create } from "zustand";
import type { DailySummary } from "@/lib/types";

interface SummaryState {
  summary:      DailySummary | null;
  isGenerating: boolean;
  error:        string | null;
  setSummary:      (s: DailySummary | null) => void;
  setIsGenerating: (v: boolean) => void;
  setError:        (e: string | null) => void;
}

export const useSummaryStore = create<SummaryState>((set) => ({
  summary:      null,
  isGenerating: false,
  error:        null,
  setSummary:      (summary)      => set({ summary }),
  setIsGenerating: (isGenerating) => set({ isGenerating }),
  setError:        (error)        => set({ error }),
}));
