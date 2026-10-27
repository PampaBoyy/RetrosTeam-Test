import { useEffect, useState } from "react";
import { api, type AppInfo, type ComponentView, type InstallProgress } from "../lib/api";
import { ACTIVE } from "../hooks/useInstalls";

export function Home({ info, progress }: { info: AppInfo; progress: Record<string, InstallProgress> }) {
  const [comps, setComps] = useState<ComponentView[]>([]);
  const finished = Object.values(progress).filter((p) => p.stage === "done").length;
  useEffect(() => { api.listComponents().then(setComps).catch(() => {}); }, [finished]);
  const installed = comps.filter((c) => c.installed);
  const active = Object.values(progress).filter((p) => ACTIVE.includes(p.stage));
  const name = (id: string) => comps.find((c) => c.id === id)?.name ?? id;
  return (
    <>
      <h1>Início</h1>
      <p className="sub">Instale uma vez. Organize. Configure. Jogue de um só lugar.</p>
      <div className="grid">
        <section className="card"><h2>Jogados recentemente</h2><div className="empty">Nenhum jogo ainda. A biblioteca chega no Marco 2.</div></section>
        <section className="card"><h2>Favoritos</h2><div className="empty">Nenhum favorito.</div></section>
        <section className="card"><h2>Emuladores e cores</h2>
          {installed.length === 0 ? <div className="empty">Nenhum componente instalado.</div> :
            <div className="row" style={{ gap: 6 }}>{installed.map((c) => <span key={c.id} className="badge ok">{c.name}</span>)}</div>}</section>
        <section className="card"><h2>Downloads ativos</h2>
          {active.length === 0 ? <div className="empty">Nenhum download.</div> :
            active.map((p) => <div key={p.id} className="row"><span className="badge">{name(p.id)}</span> {p.stage}</div>)}</section>
      </div>
      <section className="card">
        <h2>Armazenamento</h2>
        {info.warning && <div className="msg err" role="alert">{info.warning}</div>}
        <div className="row"><span className={`badge ${info.mode === "portable" ? "warn" : "ok"}`}>modo {info.mode === "portable" ? "portátil" : "instalado"}</span>
          <span className="badge">banco v{info.schemaVersion}</span></div>
        <p className="sub" style={{ margin: "10px 0 4px" }}>Pasta de dados:</p>
        <div className="path">{info.dataRoot}</div>
      </section>
    </>
  );
}
