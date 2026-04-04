import { writable } from "svelte/store";

export interface AppSettings {
  vaultPath: string | null;
  theme: "light" | "dark";
  editorFontSize: number;
}

const DEFAULT_SETTINGS: AppSettings = {
  vaultPath: null,
  theme: "dark",
  editorFontSize: 16,
};

function loadSettings(): AppSettings {
  try {
    const stored = localStorage.getItem("vaultmind-settings");
    if (stored) {
      return { ...DEFAULT_SETTINGS, ...JSON.parse(stored) };
    }
  } catch {
    // ignore
  }
  return { ...DEFAULT_SETTINGS };
}

function saveSettings(settings: AppSettings) {
  try {
    localStorage.setItem("vaultmind-settings", JSON.stringify(settings));
  } catch {
    // ignore
  }
}

const initial = loadSettings();
export const settings = writable<AppSettings>(initial);

settings.subscribe((value) => {
  saveSettings(value);
  applyTheme(value.theme);
  applyEditorFontSize(value.editorFontSize);
});

function applyTheme(theme: "light" | "dark") {
  document.documentElement.setAttribute("data-theme", theme);
}

function applyEditorFontSize(size: number) {
  document.documentElement.style.setProperty("--editor-font-size", `${size}px`);
}

applyTheme(initial.theme);
applyEditorFontSize(initial.editorFontSize);
