import { useEffect, useState } from "react";
import { api, type ComponentView, type InstallProgress } from "../lib/api";
import { ACTIVE } from "../hooks/useInstalls";
import { fmtBytes, STAGE_LABEL } from "../lib/format";

export function Downloads({ progress }: { progress: Record<string, InstallProgress> }) {
  const [names, setNames] = useState<Record<string, ComponentView>>({});
  useEffect(() => { api.listComponents().then((l) => setNames(Object.fromEntries(l.map((c) => [c.id, c])))); }, []);
  const items = Object.values(progress);
  return (
    <>
      <h1>Downloads</h1>
      <p className="sub">Instalações desta sessão. Pausar/retomar ainda não é suportado.</p>
      {items.length === 0 ? <div className="empty">Nenhum download nesta sessão. Comece em Emuladores e Cores.</div> :
        items.map((p) => (
          <div className="card" key={p.id}>
            <div className="row" style={{ justifyContent: "space-between" }}>
              <strong>{names[p.id]?.name ?? p.id}</strong>
              <span className={`badge ${p.stage === "done" ? "ok" : p.stage === "error" ? "warn" : ""}`}>{STAGE_LABEL[p.stage]}</span>
            </div>
            {ACTIVE.includes(p.stage) && p.stage === "downloading" && <div className="sub" style={{ margin: "6px 0 0" }}>{fmtBytes(p.received)}{p.total ? ` / ${fmtBytes(p.total)}` : ""} · {fmtBytes(p.speedBps)}/s</div>}
            {p.message && <div className="msg err">{p.message}</div>}
          </div>
        ))}
    </>
  );
}
