# RetrosTeam

Launcher universal open-source (Windows 10/11 x64 primeiro) para jogos, emulação, Libretro, ports, recompilações e mods.
Stack: **Tauri 2 + React/TypeScript (Vite) + Rust + SQLite (rusqlite, bundled)**.

## Status

| Item | Estado |
|---|---|
| Layout, navegação, tema, páginas Início/Emuladores/Downloads/Configurações | Interface renderizada e testada em Chromium headless (modo prévia) |
| Modo **portátil por padrão** (tudo em `RetrosTeamData` ao lado do exe) | Código escrito; **não compilado** |
| Catálogo (11 emuladores automáticos, 2 manuais, 14 cores Libretro) | JSON validado (ids, regex, hosts); **downloads nunca testados contra a rede real** |
| Download, SHA-256, extração segura (.zip/.7z), instalação, registro e remoção com backup | Código + testes Rust escritos; **não compilados nem executados** |
| Rodar jogos (RetroArch/cores/emuladores), biblioteca, mods | **Não implementado** |

Nada acima de "Fundação + gerenciador de componentes" funciona ainda.

## Portabilidade
Coloque a pasta `RetrosTeam` em qualquer lugar (pen drive, HD externo). Tudo — configurações, banco, emuladores, cores, saves — fica em `RetrosTeamData` ao lado do `.exe`. Caminhos no banco são relativos. Se a pasta não for gravável (ex.: Program Files) o app usa a pasta do usuário e avisa. Um arquivo `installed.flag` ao lado do exe força esse modo.

## Segurança dos downloads
Só HTTPS e só hosts aprovados (`libretro.com`, `github.com`, `githubusercontent.com`), inclusive em redirecionamentos. Hash SHA-256: conferido com o `digest` da API do GitHub quando existe; o buildbot da Libretro não publica hashes, então o app mostra "calculado, sem verificação do publicador". Extração recusa `..`, caminhos absolutos e links simbólicos, com limites de tamanho. Reinstalar/remover move a versão antiga para `backups/` em vez de apagar.

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
Use o workflow do GitHub (`Build Windows exe`): gera `RetrosTeam-portable.zip` com a pasta `RetrosTeam` pronta.


Documentação de arquitetura: `docs/ARCHITECTURE.md`.
