import { useState } from "react";
import { api, type AppInfo, type Settings } from "../lib/api";

interface Props { info: AppInfo; settings: Settings; onSettings: (s: Settings) => void }

export function SettingsPage({ info, settings, onSettings }: Props) {
  const [msg, setMsg] = useState<{ kind: "ok" | "err"; text: string } | null>(null);

  const save = async (next: Settings) => {
    try { onSettings(await api.saveSettings(next)); setMsg({ kind: "ok", text: "Configurações salvas." }); }
    catch (e) { setMsg({ kind: "err", text: String(e) }); }
  };

  const changeRoot = async () => {
    try {
      const folder = await api.pickFolder();
      if (!folder) return;
      onSettings(await api.setDataRoot(folder));
      setMsg({ kind: "ok", text: "Nova pasta registrada. Reinicie o RetrosTeam para usá-la. Os arquivos existentes NÃO foram movidos." });
    } catch (e) { setMsg({ kind: "err", text: String(e) }); }
  };

  return (
    <>
      <h1>Configurações</h1>
      <p className="sub">Valores globais. Perfis por plataforma e por jogo chegam no Marco 6.</p>

      <section className="card">
        <h2>Aparência e idioma</h2>
        <label className="f" htmlFor="theme">Tema</label>
        <select id="theme" value={settings.theme} onChange={(e) => save({ ...settings, theme: e.target.value as Settings["theme"] })}>
          <option value="dark">Escuro</option><option value="light">Claro</option>
        </select>
        <label className="f" htmlFor="lang">Idioma (preferência salva; traduções da interface ainda não implementadas)</label>
        <select id="lang" value={settings.language} onChange={(e) => save({ ...settings, language: e.target.value as Settings["language"] })}>
          <option value="pt-BR">Português (Brasil)</option><option value="en">English</option>
        </select>
      </section>

      <section className="card">
        <h2>Pasta de dados</h2>
        <div className="row">
          <div className="path">{info.dataRoot}</div>
          <button className="btn secondary" onClick={changeRoot}>Alterar…</button>
        </div>
        <p className="sub" style={{ margin: "10px 0 0" }}>
          Gerenciado pelo RetrosTeam: <code>programs, cores, downloads, cache, database, logs</code>.
          Seus arquivos: <code>roms, games, userdata, backups</code> — nunca são apagados em atualizações.
        </p>
        <ul className="layout">
          {info.layout.map((l) => <li key={l.path}>{l.path} {l.exists ? "✓" : "✗ ausente"}</li>)}
        </ul>
      </section>

      <section className="card">
        <h2>Diagnóstico</h2>
        <p className="sub" style={{ margin: 0 }}>Config: <code>{info.configDir}</code><br />Banco: <code>{info.dbPath}</code> (versão {info.schemaVersion})</p>
      </section>
      {msg && <div className={`msg ${msg.kind}`} role="status">{msg.text}</div>}
    </>
  );
}
