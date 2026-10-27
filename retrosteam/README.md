# RetrosTeam

Launcher universal open-source (Windows 10/11 x64 primeiro) para jogos, emulação, Libretro, ports, recompilações e mods.
Stack: **Tauri 2 + React/TypeScript (Vite) + Rust + SQLite (rusqlite, bundled)**.

## Status — Marco 1 (Fundação)

| Item | Estado |
|---|---|
| Layout, navegação, tema escuro/claro | Implementado; renderização verificada em Chromium headless |
| Persistência de configurações (JSON, gravação atômica) | Código escrito + testes Rust; **ainda não compilado** |
| Pasta de dados configurável, modo portátil/instalado, `safe_join` | Código escrito + testes Rust; **ainda não compilado** |
| Migrações SQLite versionadas (`PRAGMA user_version`) | SQL validado em SQLite real; código Rust **ainda não compilado** |
| Biblioteca, downloads, emuladores, Libretro, mods | **Não implementado** (telas marcadas "Planejado") |

Nada acima de "Fundação" funciona ainda. Telas planejadas não têm ações falsas.

## Pré-requisitos (Windows)
1. Node.js 20+ e npm.
2. Rust estável (rustup, alvo `x86_64-pc-windows-msvc`) + *Visual Studio Build Tools* (workload "Desenvolvimento para desktop com C++").
3. WebView2 Runtime (já vem no Windows 11; no 10, instale o Evergreen).

## Rodar
```
npm install
npx tauri icon src-tauri/icons/source.png   # gera icon.ico e PNGs exigidos pelo build
npm run tauri dev
```
## Testar
```
cd src-tauri && cargo test      # paths, migrações, settings
npm run build                   # typecheck + bundle do frontend
```
## Gerar instalador
`npm run tauri build` (NSIS, sem exigir administrador por padrão).

## Modos
- **Instalado**: dados em `%APPDATA%\org.retrosteam.app\RetrosTeam`.
- **Portátil**: crie um arquivo vazio `portable.flag` ao lado do `.exe`; tudo fica em `RetrosTeamData\`.

Documentação de arquitetura: `docs/ARCHITECTURE.md`.
