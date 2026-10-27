import { useEffect, useState } from "react";
import { api, isDesktop, type AppInfo, type Settings } from "./lib/api";
import { Home } from "./pages/Home";
import { Library } from "./pages/Library";
import { Planned } from "./pages/Planned";
import { SettingsPage } from "./pages/SettingsPage";

const NAV = [
  { id: "home", label: "Início" },
  { id: "library", label: "Biblioteca" },
  { id: "emulators", label: "Emuladores e Cores" },
  { id: "downloads", label: "Downloads" },
  { id: "ports", label: "Jogos e Ports" },
  { id: "community", label: "Comunidade" },
  { id: "settings", label: "Configurações" },
] as const;
type PageId = (typeof NAV)[number]["id"];

const PLANNED: Record<string, { title: string; milestone: string; text: string }> = {
  emulators: { title: "Emuladores e Cores", milestone: "Marcos 3–5", text: "Registro de componentes, adaptadores de emuladores e host Libretro." },
  downloads: { title: "Downloads", milestone: "Marco 3", text: "Gerenciador de downloads com verificação de integridade." },
  ports: { title: "Jogos e Ports", milestone: "Marco 7", text: "Manifestos de projetos, recompilações e mods." },
  community: { title: "Comunidade", milestone: "Pós-Marco 8", text: "Integrações opcionais (ex.: RetroAchievements), somente após o app offline estar estável." },
};

function Logo() {
  return (
    <svg viewBox="0 0 32 32" aria-hidden="true">
      <rect width="32" height="32" rx="8" fill="var(--accent)" />
      <path d="M9 23V9h8a4.5 4.5 0 0 1 0 9h-3m3 0 5 5" fill="none" stroke="var(--accent-ink)" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

export default function App() {
  const [page, setPage] = useState<PageId>("home");
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [settings, setSettings] = useState<Settings | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([api.appInfo(), api.getSettings()])
      .then(([i, s]) => { setInfo(i); setSettings(s); })
      .catch((e) => setError(String(e)));
  }, []);

  useEffect(() => {
    if (settings) document.documentElement.dataset.theme = settings.theme;
  }, [settings]);

  return (
    <>
      {!isDesktop() && <div className="preview-banner">Prévia de navegador — dados de exemplo, nada é salvo</div>}
      <div className="app" style={!isDesktop() ? { height: "calc(100vh - 24px)" } : undefined}>
        <nav className="sidebar" aria-label="Navegação principal">
          <div className="brand"><Logo /> RetrosTeam</div>
          {NAV.map((n) => (
            <button key={n.id} className="nav" aria-current={page === n.id ? "page" : undefined} onClick={() => setPage(n.id)}>
              {n.label}
            </button>
          ))}
          <div className="spacer" />
          <span className="badge" style={{ alignSelf: "flex-start" }}>v{info?.version ?? "…"}</span>
        </nav>
        <main className="main">
          {error && <div className="card msg err" role="alert">Falha ao iniciar: {error}</div>}
          {!error && (!info || !settings) && <p className="sub">Carregando…</p>}
          {info && settings && page === "home" && <Home info={info} />}
          {info && settings && page === "library" && <Library />}
          {info && settings && page === "settings" && (
            <SettingsPage info={info} settings={settings} onSettings={setSettings} />
          )}
          {info && settings && PLANNED[page] && <Planned {...PLANNED[page]} />}
        </main>
      </div>
    </>
  );
}
