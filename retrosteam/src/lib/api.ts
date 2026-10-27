// Single gateway to the Rust backend. Only explicit commands are exposed.
export interface LayoutEntry { path: string; exists: boolean }
export interface AppInfo {
  version: string; mode: "portable" | "installed"; dataRoot: string; configDir: string;
  dbPath: string; schemaVersion: number; layout: LayoutEntry[];
}
export interface Settings { schemaVersion: number; theme: "dark" | "light"; language: "pt-BR" | "en"; dataRoot: string | null }

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
  configDir: "(preview)", dbPath: "(preview)", schemaVersion: 1,
  layout: ["programs/emulators", "cores", "roms/SNES", "userdata/saves", "database", "logs"].map((path) => ({ path, exists: true })),
};

export const api = {
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
