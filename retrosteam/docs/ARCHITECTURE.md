# Arquitetura (Marco 1)

**Avaliação da stack.** Tauri 2 é adequado: binário pequeno, Rust para processos/arquivos/downloads, WebView2 nativo no Windows.
Risco conhecido: o host Libretro (vídeo/áudio/entrada) é o maior desafio e ficará em um módulo/thread isolado (Marco 5),
possivelmente com janela própria; a UI web não renderiza frames de emulação diretamente.

## Fronteira de segurança
O frontend só chama comandos explícitos de `src-tauri/src/commands.rs` (`get_app_info`, `get_settings`, `save_settings`, `set_data_root`).
Não há plugin de shell nem de fs habilitado (ver `capabilities/default.json`); só `dialog:allow-open` para escolher pastas.
CSP restritiva em `tauri.conf.json`.

## Módulos Rust
- `paths` — `safe_join` (rejeita absolutos, `..`, prefixos de drive/UNC) e `DATA_LAYOUT`.
- `db` — migrações versionadas, cada uma em transação.
- `settings` — JSON validado, gravação atômica (temp + rename).
- `state` — resolve modo (portátil/instalado), pasta de dados, abre o banco.
- `error` — `AppError` serializável como texto para a UI.

## Pastas
Gerenciadas pelo app (podem ser recriadas): `programs`, `cores`, `downloads`, `cache`, `database`, `logs`.
Do usuário (nunca apagadas em updates): `roms`, `games`, `userdata`, `backups`, `profiles`, `mods`.

## Precedência de configuração (a implementar no Marco 6)
Jogo > plataforma > global > padrão do componente.

## Limitações atuais
- Alterar a pasta de dados registra o novo caminho, mas **não migra** arquivos existentes; vale após reiniciar.
- Idioma é só preferência salva; ainda não há traduções.
- `safe_join` é léxico; verificação de symlinks virá junto da extração de arquivos (Marco 3).
- Rede: nenhuma operação do Marco 1 usa rede; tudo funciona offline.
