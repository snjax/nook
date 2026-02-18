import * as api from "../api/tauri";
import type { Settings } from "../types";

const defaultSettings: Settings = {
  exposeProtocols: ["http", "https", "postgres", "mysql", "redis", "mongodb"],
  notExposeFilters: [{ protocol: "dns" }, { port: 22 }],
  portProtocols: {},
  terminal: "",
  statsInterval: 2000,
  portsScanInterval: 3000,
  processScanInterval: 5000,
  dockerSocketPath: "",
  onboardingComplete: false,
  logLevel: "info",
  portAction: "prompt",
};

let settings = $state<Settings>({ ...defaultSettings });
let loaded = $state(false);

export function getSettings(): Settings {
  return settings;
}

export function isLoaded(): boolean {
  return loaded;
}

export async function loadSettings(): Promise<void> {
  try {
    settings = await api.getSettings();
    loaded = true;
  } catch (e) {
    console.error("Failed to load settings:", e);
    settings = { ...defaultSettings };
    loaded = true;
  }
}

export async function saveSettings(newSettings: Settings): Promise<void> {
  try {
    await api.saveSettings(newSettings);
    settings = newSettings;
  } catch (e) {
    console.error("Failed to save settings:", e);
    throw e;
  }
}

export function updateSettings(partial: Partial<Settings>): void {
  settings = { ...settings, ...partial };
}
