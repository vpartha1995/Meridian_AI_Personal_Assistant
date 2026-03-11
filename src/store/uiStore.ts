import { create } from "zustand";

type Page = "dashboard" | "tasks" | "settings" | "history";

interface UiState {
  activePage:    Page;
  overlayOpen:   boolean;
  sidebarOpen:   boolean;
  setPage:       (p: Page) => void;
  setOverlay:    (v: boolean) => void;
  toggleSidebar: () => void;
}

export const useUiStore = create<UiState>((set) => ({
  activePage:  "dashboard",
  overlayOpen: false,
  sidebarOpen: true,
  setPage:       (activePage)  => set({ activePage }),
  setOverlay:    (overlayOpen) => set({ overlayOpen }),
  toggleSidebar: ()            => set((s) => ({ sidebarOpen: !s.sidebarOpen })),
}));
