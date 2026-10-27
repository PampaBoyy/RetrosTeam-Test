import { useCallback, useEffect, useState } from "react";
import { api, type ComponentView, type InstallProgress } from "../lib/api";
import { ACTIVE } from "../hooks/useInstalls";
import { fmtBytes, fmtEta, STAGE_LABEL } from "../lib/format";

interface Props { progress: Record<string, InstallProgress>; dismiss: (id: string) => void }

function Bar({ p }: { p: InstallProgress }) {
  const pct = p.total ? Math.min(100, (p.received / p.total) * 100) : null;
  const eta = p.total ? fmtEta(p.total - p.received, p.speedBps) : null;
  return (
    <div style={{ marginTop: 10 }}>
      <div className="bar" role="progressbar" aria-valuenow={pct ?? undefined} aria-valuemin={0} aria-valuemax={100}>
        <div className={pct === null ? "fill indet" : "fill"} style={pct === null ? undefined : { width: `${pct}%` }} />
      </div>
      <div className="sub" style={{ margin: "6px 0 0", fontSize: 12 }}>
        {STAGE_LABEL[p.stage]}
        {p.stage === "downloading" && <> · {fmtBytes(p.received)}{p.total ? ` / ${fmtBytes(p.total)}` : ""} · {fmtBytes(p.speedBps)}/s{eta ? ` · ${eta}` : ""}</>}
      </div>
    </div>
  );
}

function Card({ c, p, onChange, dismiss }: { c: ComponentView; p?: InstallProgress; onChange: () => void; dismiss: () => void }) {
  const [err, setErr] = useState<string | null>(null);
  const busy = !!p && ACTIVE.includes(p.stage);
  const act = async (fn: () => Promise<unknown>) => { setErr(null); try { await fn(); } catch (e) { setErr(String(e)); } onChange(); };
  const failed = p?.stage === "error" ? p.message : null;

  return (
    <article className="card" style={{ marginBottom: 0 }} aria-label={c.name}>
      <div className="row" style={{ justifyContent: "space-between", alignItems: "flex-start" }}>
        <h2 style={{ margin: 0 }}>{c.name}</h2>
        {c.installed ? <span className="badge ok">Instalado{c.installedVersion ? ` · ${c.installedVersion.slice(0, 22)}` : ""}</span> : null}
      </div>
      <div className="row" style={{ gap: 6, margin: "8px 0" }}>
        {c.platforms.map((pl) => <span key={pl} className="badge">{pl}</span>)}
      </div>
      <div className="sub" style={{ margin: 0, fontSize: 12 }}>Licença: {c.license}</div>
      {c.notes && <div className="sub" style={{ margin: "6px 0 0", fontSize: 12 }}>{c.notes}</div>}
      {c.installed && (
        <div style={{ marginTop: 8 }}>
          <span className={`badge ${c.integrity === "verified" ? "ok" : "warn"}`}>
            {c.integrity === "verified" ? "SHA-256 conferido com o publicador" : "SHA-256 calculado; publicador não fornece hash"}
          </span>
          {c.installPath && <div className="path" style={{ marginTop: 8 }}>{c.installPath}</div>}
        </div>
      )}
      {p && p.stage !== "done" && p.stage !== "error" && p.stage !== "cancelled" && <Bar p={p} />}
      {(failed || err) && <div className="msg err" role="alert">{failed || err}</div>}
      {p?.stage === "cancelled" && <div className="msg">Download cancelado.</div>}
      <div className="row" style={{ marginTop: 12 }}>
        {!c.autoInstall ? (
          <div style={{ fontSize: 12 }}><span className="badge warn">Instalação manual</span>{" "}
            <span className="sub">Site oficial: {c.homepage}</span></div>
        ) : busy ? (
          <button className="btn secondary" disabled={p!.stage !== "downloading" && p!.stage !== "resolving"} onClick={() => api.cancelInstall(c.id)}>Cancelar</button>
        ) : (
          <>
            <button className="btn" onClick={() => { dismiss(); act(() => api.installComponent(c.id)); }}>
              {failed ? "Tentar de novo" : c.installed ? "Reinstalar / atualizar" : "Baixar e instalar"}
            </button>
            {c.installed && <button className="btn secondary" onClick={() => act(async () => { await api.uninstallComponent(c.id); })}>Remover (mantém backup)</button>}
          </>
        )}
      </div>
    </article>
  );
}

export function Emulators({ progress, dismiss }: Props) {
  const [list, setList] = useState<ComponentView[] | null>(null);
  const [q, setQ] = useState("");
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(() => { api.listComponents().then(setList).catch((e) => setError(String(e))); }, []);
  useEffect(refresh, [refresh]);
  // reload the list whenever an install finishes
  const finished = Object.values(progress).filter((p) => p.stage === "done").length;
  useEffect(() => { if (finished) refresh(); }, [finished, refresh]);

  const match = (c: ComponentView) => !q || (c.name + " " + c.platforms.join(" ")).toLowerCase().includes(q.toLowerCase());
  const emus = (list ?? []).filter((c) => c.kind === "emulator" && match(c));
  const cores = (list ?? []).filter((c) => c.kind === "core" && match(c));

  const grid = (items: ComponentView[]) => (
    items.length ? <div className="grid">{items.map((c) => (
      <Card key={c.id} c={c} p={progress[c.id]} onChange={refresh} dismiss={() => dismiss(c.id)} />
    ))}</div> : <div className="empty">Nada encontrado.</div>
  );

  return (
    <>
      <h1>Emuladores e Cores</h1>
      <p className="sub">Downloads vêm só das fontes oficiais (GitHub Releases e buildbot da Libretro) e tudo é instalado dentro da pasta do RetrosTeam.</p>
      <div className="row" style={{ marginBottom: 18 }}>
        <input className="search" type="search" placeholder="Buscar por nome ou plataforma…" aria-label="Buscar componentes" value={q} onChange={(e) => setQ(e.target.value)} />
      </div>
      {error && <div className="card msg err" role="alert">{error}</div>}
      {!list && !error && <p className="sub">Carregando…</p>}
      {list && (<>
        <h2 className="section">Emuladores independentes</h2>
        {grid(emus)}
        <h2 className="section">Cores Libretro</h2>
        <p className="sub" style={{ marginTop: -6 }}>Os cores são baixados para a pasta <code>cores</code>. Rodar jogos com eles dentro do app chega no Marco 5; até lá, use o RetroArch acima.</p>
        {grid(cores)}
      </>)}
    </>
  );
}
