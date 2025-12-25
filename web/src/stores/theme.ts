import { defineStore } from "pinia";

const STORAGE_KEY = "nai-ui-theme";
export type ThemeMode = "light" | "dark";

function detectInitialTheme(): ThemeMode {
  if (typeof window === "undefined") return "light";
  const saved = window.localStorage.getItem(STORAGE_KEY);
  if (saved === "light" || saved === "dark") return saved;
  const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
  return prefersDark ? "dark" : "light";
}

function applyDomTheme(mode: ThemeMode) {
  if (typeof document === "undefined") return;
  document.documentElement.dataset.theme = mode;
  document.documentElement.style.colorScheme = mode;
}

export const useThemeStore = defineStore("theme", {
  state: () => ({
    theme: detectInitialTheme() as ThemeMode,
  }),
  actions: {
    setTheme(mode: ThemeMode) {
      this.theme = mode;
      if (typeof window !== "undefined") {
        window.localStorage.setItem(STORAGE_KEY, mode);
      }
      applyDomTheme(mode);
    },
    toggle() {
      this.setTheme(this.theme === "dark" ? "light" : "dark");
    },
    applyTheme() {
      applyDomTheme(this.theme);
    },
  },
});
