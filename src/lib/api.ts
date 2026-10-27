import catalogJson from "../../src-tauri/catalog.json";

// Single gateway to the Rust backend. Only explicit commands are exposed.
export interface LayoutEntry { path: string; exists: boolean }
export interface AppInfo {
  version: string; mode: "portable" | "installed"; dataRoot: string; configDir: string;
  dbPath: string; schemaVersion: number; warning: string | null; layout: LayoutEntry[];
}
export interface Settings { schemaVersion: number; theme: "dark" | "light"; language: "pt-BR" | "en"; dataRoot: string | null }

export interface ComponentView {
  id: string; name: string; kind: "emulator" | "core"; platforms: string[]; license: string; homepage: string;
  notes?: string | null; sourceType: string; autoInstall: boolean; installed: boolean;
  installedVersion: string | null; integrity: string | null; installPath: string | null; exePath: string | null;
}
export type Stage = "resolving" | "downloading" | "verifying" | "extracting" | "installing" | "done" | "error" | "cancelled";
export interface InstallProgress { id: string; stage: Stage; received: number; total: number | null; speedBps: number; message: string | null }

export const isDesktop = () => typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(cmd, args);
}

// BROWSER PREVIEW ONLY: static sample so the UI can be reviewed outside Tauri (npm run dev in a browser).
// It is never used inside the desktop app and nothing here is persisted.
const previewSettings: Settings = { schemaVersion: 1, theme: "dark", language: "pt-BR", dataRoot: null };
const previewInfo: AppInfo = {
  version: "0.1.0", mode: "installed", dataRoot: "(preview) C:\\Users\\Você\\AppData\\Roaming\\RetrosTeam",
  configDir: "(preview)", dbPath: "(preview)", schemaVersion: 2, warning: null,
  layout: ["programs/emulators", "cores", "roms/SNES", "userdata/saves", "database", "logs"].map((path) => ({ path, exists: true })),
};

// BROWSER PREVIEW ONLY: built from the real catalog, but nothing is installed.
const previewComponents = (): ComponentView[] =>
  (catalogJson.components as any[]).map((c) => ({
    ...c, sourceType: c.source.type, autoInstall: c.source.type !== "manual", installed: false,
    installedVersion: null, integrity: null, installPath: null, exePath: null,
  }));

export const api = {
  listComponents: () => (isDesktop() ? call<ComponentView[]>("list_components") : Promise.resolve(previewComponents())),
  installComponent: (id: string) =>
    isDesktop() ? call<void>("install_component", { id }) : Promise.reject("Downloads só funcionam no app desktop"),
  cancelInstall: (id: string) => (isDesktop() ? call<boolean>("cancel_install", { id }) : Promise.resolve(false)),
  uninstallComponent: (id: string) =>
    isDesktop() ? call<string>("uninstall_component", { id }) : Promise.reject("Indisponível na prévia"),
  onProgress: async (cb: (p: InstallProgress) => void): Promise<() => void> => {
    if (!isDesktop()) return () => {};
    const { listen } = await import("@tauri-apps/api/event");
    return listen<InstallProgress>("install-progress", (e) => cb(e.payload));
  },
  appInfo: () => (isDesktop() ? call<AppInfo>("get_app_info") : Promise.resolve(previewInfo)),
  getSettings: () => (isDesktop() ? call<Settings>("get_settings") : Promise.resolve({ ...previewSettings })),
  saveSettings: (settings: Settings) =>
    isDesktop() ? call<Settings>("save_settings", { settings }) : Promise.resolve(settings),
  setDataRoot: (path: string) =>
    isDesktop() ? call<Settings>("set_data_root", { path }) : Promise.reject("Indisponível na prévia de navegador"),
  pickFolder: async (): Promise<string | null> => {
    if (!isDesktop()) return null;
    const { open } = await import("@tauri-apps/plugin-dialog");
    const r = await open({ directory: true, multiple: false });
    return typeof r === "string" ? r : null;
  },
};
