import type { AppInfo } from "../lib/api";

export function Home({ info }: { info: AppInfo }) {
  return (
    <>
      <h1>Início</h1>
      <p className="sub">Instale uma vez. Organize. Configure. Jogue de um só lugar.</p>
      <div className="grid">
        <section className="card"><h2>Jogados recentemente</h2><div className="empty">Nenhum jogo ainda. A biblioteca chega no Marco 2.</div></section>
        <section className="card"><h2>Favoritos</h2><div className="empty">Nenhum favorito.</div></section>
        <section className="card"><h2>Emuladores e cores</h2><div className="empty">Nenhum componente instalado.</div></section>
        <section className="card"><h2>Downloads ativos</h2><div className="empty">Nenhum download.</div></section>
      </div>
      <section className="card">
        <h2>Armazenamento</h2>
        <div className="row"><span className={`badge ${info.mode === "portable" ? "warn" : "ok"}`}>modo {info.mode === "portable" ? "portátil" : "instalado"}</span>
          <span className="badge">banco v{info.schemaVersion}</span></div>
        <p className="sub" style={{ margin: "10px 0 4px" }}>Pasta de dados:</p>
        <div className="path">{info.dataRoot}</div>
      </section>
    </>
  );
}
