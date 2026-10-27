//! Download → verify → safe extract → install → register. One component at a time per id.
//! Pause/resume is NOT implemented; cancel works during the download phase only.
use crate::{catalog::{url_allowed, Entry, Source}, error::AppError, paths::safe_join};
use futures_util::StreamExt;
use regex::Regex;
use reqwest::{redirect, Client, Url};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter};

const MAX_DOWNLOAD: u64 = 4 << 30; // 4 GiB
const MAX_UNPACKED: u64 = 16 << 30; // 16 GiB
const MAX_ENTRIES: usize = 200_000;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub id: String,
    pub stage: &'static str,
    pub received: u64,
    pub total: Option<u64>,
    pub speed_bps: u64,
    pub message: Option<String>,
}

fn emit(app: &AppHandle, id: &str, stage: &'static str, received: u64, total: Option<u64>, speed_bps: u64, message: Option<String>) {
    let _ = app.emit("install-progress", Progress { id: id.to_string(), stage, received, total, speed_bps, message });
}

fn net(e: reqwest::Error) -> AppError {
    AppError::Invalid(format!("Falha de rede: {e}. Verifique a conexão e tente de novo."))
}

pub fn unix_ts() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

struct Resolved { url: Url, file_name: String, version: String, expected_sha256: Option<String>, size: Option<u64> }

#[derive(Deserialize)]
struct GhAsset { name: String, browser_download_url: String, size: Option<u64>, digest: Option<String> }
#[derive(Deserialize)]
struct GhRelease { tag_name: String, assets: Vec<GhAsset> }

fn client() -> Result<Client, AppError> {
    let policy = redirect::Policy::custom(|a| {
        if a.previous().len() >= 5 { return a.error("redirecionamentos demais"); }
        if url_allowed(a.url()) { a.follow() } else { a.error("redirecionamento para host não aprovado") }
    });
    Client::builder()
        .user_agent(concat!("RetrosTeam/", env!("CARGO_PKG_VERSION")))
        .redirect(policy)
        .connect_timeout(Duration::from_secs(20))
        .build()
        .map_err(net)
}

fn parse_approved(url: &str) -> Result<Url, AppError> {
    let u = Url::parse(url).map_err(|e| AppError::Invalid(format!("URL inválida: {e}")))?;
    if !url_allowed(&u) { return Err(AppError::Invalid(format!("URL fora das fontes aprovadas: {u}"))); }
    Ok(u)
}

async fn resolve(client: &Client, e: &Entry) -> Result<Resolved, AppError> {
    match &e.source {
        Source::LibretroCore { file } => {
            let name = format!("{file}.dll.zip");
            let url = parse_approved(&format!("https://buildbot.libretro.com/nightly/windows/x86_64/latest/{name}"))?;
            Ok(Resolved { url, file_name: name, version: "nightly".into(), expected_sha256: None, size: None })
        }
        Source::DirectUrl { url } => {
            let url = parse_approved(url)?;
            let file_name = url.path_segments().and_then(|s| s.last()).unwrap_or("download.bin").to_string();
            Ok(Resolved { url, file_name, version: "nightly".into(), expected_sha256: None, size: None })
        }
        Source::GithubRelease { repo, asset_pattern } => {
            let api = format!("https://api.github.com/repos/{repo}/releases/latest");
            let resp = client.get(&api).header("Accept", "application/vnd.github+json").send().await.map_err(net)?;
            if !resp.status().is_success() {
                return Err(AppError::Invalid(format!(
                    "O GitHub respondeu HTTP {} para {repo} (limite de requisições por hora ou release inexistente). Tente mais tarde.", resp.status())));
            }
            let rel: GhRelease = resp.json().await.map_err(net)?;
            let re = Regex::new(asset_pattern).map_err(|e| AppError::Invalid(e.to_string()))?;
            let asset = rel.assets.iter().find(|a| re.is_match(&a.name)).ok_or_else(|| AppError::Invalid(format!(
                "Nenhum arquivo da release {} corresponde ao pacote esperado para Windows x64. Instale manualmente pela página oficial.", rel.tag_name)))?;
            let sha = asset.digest.as_deref().and_then(|d| d.strip_prefix("sha256:")).map(|s| s.to_lowercase());
            Ok(Resolved { url: parse_approved(&asset.browser_download_url)?, file_name: asset.name.clone(), version: rel.tag_name.clone(), expected_sha256: sha, size: asset.size })
        }
        Source::Manual { .. } => Err(AppError::Invalid("Este componente não tem download automático aprovado.".into())),
    }
}

/// Keep only a plain file name (no separators) taken from remote metadata.
fn plain_name(s: &str) -> Result<String, AppError> {
    let n = s.rsplit(['/', '\\']).next().unwrap_or("");
    if n.is_empty() || n == "." || n == ".." { return Err(AppError::Invalid(format!("nome de arquivo inválido: {s}"))); }
    Ok(n.to_string())
}

async fn download(app: &AppHandle, id: &str, client: &Client, r: &Resolved, dir: &Path, cancel: &AtomicBool)
    -> Result<(PathBuf, String, Option<String>), AppError> {
    fs::create_dir_all(dir)?;
    let name = plain_name(&r.file_name)?;
    let final_path = safe_join(dir, &name)?;
    let part = safe_join(dir, &format!("{name}.part"))?;

    let resp = client.get(r.url.clone()).send().await.map_err(net)?;
    if !resp.status().is_success() { return Err(AppError::Invalid(format!("O servidor respondeu HTTP {}.", resp.status()))); }
    if !url_allowed(resp.url()) { return Err(AppError::Invalid("o download terminou em um host não aprovado".into())); }
    let total = resp.content_length().or(r.size);
    if total.map_or(false, |t| t > MAX_DOWNLOAD) { return Err(AppError::Invalid("arquivo grande demais para o limite de segurança".into())); }
    let last_modified = resp.headers().get(reqwest::header::LAST_MODIFIED).and_then(|v| v.to_str().ok()).map(String::from);

    let mut file = fs::File::create(&part)?;
    let mut hasher = Sha256::new();
    let mut received = 0u64;
    let mut stream = resp.bytes_stream();
    let mut tick = Instant::now();
    let mut tick_bytes = 0u64;
    while let Some(chunk) = stream.next().await {
        if cancel.load(Ordering::Relaxed) { drop(file); let _ = fs::remove_file(&part); return Err(AppError::Cancelled); }
        let chunk = chunk.map_err(net)?;
        received += chunk.len() as u64;
        if received > MAX_DOWNLOAD { return Err(AppError::Invalid("download excedeu o limite de segurança".into())); }
        hasher.update(&chunk);
        file.write_all(&chunk).map_err(|e| AppError::Invalid(format!("Falha ao gravar no disco (espaço insuficiente?): {e}")))?;
        let dt = tick.elapsed();
        if dt >= Duration::from_millis(250) {
            let speed = ((received - tick_bytes) as f64 / dt.as_secs_f64()) as u64;
            emit(app, id, "downloading", received, total, speed, None);
            tick = Instant::now();
            tick_bytes = received;
        }
    }
    file.flush()?;
    drop(file);
    if let Some(t) = total { if t != received { return Err(AppError::Invalid(format!("Download incompleto ({received} de {t} bytes)."))); } }
    fs::rename(&part, &final_path)?;
    Ok((final_path, format!("{:x}", hasher.finalize()), last_modified))
}

fn verify(sha: &str, expected: &Option<String>) -> Result<&'static str, AppError> {
    match expected {
        Some(e) if e.eq_ignore_ascii_case(sha) => Ok("verified"),
        Some(e) => Err(AppError::Invalid(format!("SHA-256 não confere (esperado {e}, obtido {sha}). Arquivo descartado."))),
        None => Ok("unverified"),
    }
}

// ---------- safe extraction ----------
fn extract(archive: &Path, dest: &Path) -> Result<(), AppError> {
    fs::create_dir_all(dest)?;
    let name = archive.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
    if name.ends_with(".zip") { extract_zip(archive, dest) }
    else if name.ends_with(".7z") { extract_7z(archive, dest) }
    else { Err(AppError::Invalid("Formato de pacote não suportado (somente .zip e .7z).".into())) }
}

fn extract_zip(archive: &Path, dest: &Path) -> Result<(), AppError> {
    let mut z = zip::ZipArchive::new(fs::File::open(archive)?)
        .map_err(|e| AppError::Invalid(format!("Pacote zip corrompido: {e}")))?;
    if z.len() > MAX_ENTRIES { return Err(AppError::Invalid("pacote com arquivos demais".into())); }
    let mut total = 0u64;
    for i in 0..z.len() {
        let mut entry = z.by_index(i).map_err(|e| AppError::Invalid(format!("Pacote zip corrompido: {e}")))?;
        let rel = entry.enclosed_name().ok_or_else(|| AppError::Invalid(format!("Entrada insegura no pacote: {}", entry.name())))?;
        if entry.unix_mode().map_or(false, |m| m & 0o170000 == 0o120000) {
            return Err(AppError::Invalid(format!("Link simbólico no pacote recusado: {}", entry.name())));
        }
        let out = safe_join(dest, &rel.to_string_lossy().replace('\\', "/"))?;
        if entry.is_dir() { fs::create_dir_all(&out)?; continue; }
        if let Some(p) = out.parent() { fs::create_dir_all(p)?; }
        let mut w = fs::File::create(&out)?;
        total += io::copy(&mut (&mut entry).take(MAX_UNPACKED - total), &mut w)?;
        if total >= MAX_UNPACKED { return Err(AppError::Invalid("conteúdo extraído excede o limite de segurança".into())); }
    }
    Ok(())
}

fn extract_7z(archive: &Path, dest: &Path) -> Result<(), AppError> {
    let mut sz = sevenz_rust::SevenZReader::open(archive, sevenz_rust::Password::empty())
        .map_err(|e| AppError::Invalid(format!("Pacote 7z inválido: {e}")))?;
    let mut total = 0u64;
    let mut count = 0usize;
    let mut failure: Option<AppError> = None;
    let res = sz.for_each_entries(&mut |entry, reader| {
        count += 1;
        if count > MAX_ENTRIES { failure = Some(AppError::Invalid("pacote com arquivos demais".into())); return Ok(false); }
        let out = match safe_join(dest, &entry.name().replace('\\', "/")) {
            Ok(p) => p,
            Err(e) => { failure = Some(e.into()); return Ok(false); }
        };
        let step = (|| -> Result<(), AppError> {
            if entry.is_directory() { fs::create_dir_all(&out)?; return Ok(()); }
            if let Some(p) = out.parent() { fs::create_dir_all(p)?; }
            let mut w = fs::File::create(&out)?;
            total += io::copy(&mut reader.take(MAX_UNPACKED - total), &mut w)?;
            if total >= MAX_UNPACKED { return Err(AppError::Invalid("conteúdo extraído excede o limite de segurança".into())); }
            Ok(())
        })();
        if let Err(e) = step { failure = Some(e); return Ok(false); }
        Ok(true)
    });
    if let Some(f) = failure { return Err(f); }
    res.map_err(|e| AppError::Invalid(format!("Falha ao extrair 7z: {e}")))?;
    Ok(())
}

// ---------- placement ----------
fn content_root(staging: &Path) -> Result<PathBuf, AppError> {
    let entries: Vec<fs::DirEntry> = fs::read_dir(staging)?.collect::<Result<_, _>>()?;
    if entries.len() == 1 && entries[0].path().is_dir() { Ok(entries[0].path()) } else { Ok(staging.to_path_buf()) }
}

fn find_file(dir: &Path, pat: &Regex, depth: u32) -> Option<PathBuf> {
    let mut subdirs = vec![];
    for e in fs::read_dir(dir).ok()?.flatten() {
        let p = e.path();
        if p.is_dir() { subdirs.push(p); }
        else if p.file_name().and_then(|n| n.to_str()).map_or(false, |n| pat.is_match(n)) { return Some(p); }
    }
    if depth == 0 { return None; }
    subdirs.into_iter().find_map(|s| find_file(&s, pat, depth - 1))
}

fn rel_string(root: &Path, p: &Path) -> String {
    p.strip_prefix(root).unwrap_or(p).components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned()).collect::<Vec<_>>().join("/")
}

/// Moves `src` (file or dir) into backups/<bucket>/<id>-<ts>/ instead of deleting it.
pub fn move_to_backup(data_root: &Path, bucket: &str, id: &str, src: &Path) -> Result<PathBuf, AppError> {
    let dest = safe_join(data_root, &format!("backups/{bucket}/{id}-{}", unix_ts()))?;
    if src.is_dir() {
        if let Some(p) = dest.parent() { fs::create_dir_all(p)?; }
        fs::rename(src, &dest)?;
    } else {
        fs::create_dir_all(&dest)?;
        fs::rename(src, dest.join(src.file_name().unwrap_or_default()))?;
    }
    Ok(dest)
}

/// Returns (install path relative to data root, exe path relative to data root).
fn place(e: &Entry, data_root: &Path, content: &Path) -> Result<(String, Option<String>), AppError> {
    if e.kind == "core" {
        let dll = find_file(content, &Regex::new(r"(?i)\.dll$").unwrap(), 1)
            .ok_or_else(|| AppError::Invalid("O pacote do core não contém uma DLL.".into()))?;
        let name = plain_name(&dll.file_name().unwrap_or_default().to_string_lossy())?;
        let target = safe_join(data_root, &format!("cores/{name}"))?;
        if target.exists() { move_to_backup(data_root, "components", &e.id, &target)?; }
        fs::create_dir_all(target.parent().unwrap())?;
        fs::rename(&dll, &target)?;
        return Ok((rel_string(data_root, &target), None));
    }
    let target = safe_join(data_root, &format!("programs/emulators/{}", e.id))?;
    if target.exists() { move_to_backup(data_root, "components", &e.id, &target)?; } // keeps old settings/saves recoverable
    fs::create_dir_all(target.parent().unwrap())?;
    fs::rename(content, &target)?;
    if let Some(m) = &e.portable_marker { fs::write(safe_join(&target, m)?, b"")?; }
    let exe = match &e.exe_pattern {
        Some(p) => find_file(&target, &Regex::new(p).map_err(|e| AppError::Invalid(e.to_string()))?, 2).map(|f| rel_string(data_root, &f)),
        None => None,
    };
    Ok((rel_string(data_root, &target), exe))
}

fn register(db_path: &Path, e: &Entry, version: &str, install: &str, exe: &Option<String>, integrity: &str, sha: &str, url: &str) -> Result<(), AppError> {
    let conn = Connection::open(db_path)?;
    conn.execute(
        "INSERT INTO components (id, display_name, kind, installed_version, install_path, status, integrity, source_url, license, exe_path, sha256, installed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'installed', ?6, ?7, ?8, ?9, ?10, datetime('now'))
         ON CONFLICT(id) DO UPDATE SET display_name=excluded.display_name, installed_version=excluded.installed_version,
           install_path=excluded.install_path, status='installed', integrity=excluded.integrity, source_url=excluded.source_url,
           license=excluded.license, exe_path=excluded.exe_path, sha256=excluded.sha256, installed_at=excluded.installed_at",
        params![e.id, e.name, e.kind, version, install, integrity, url, e.license, exe, sha],
    )?;
    Ok(())
}

async fn run(app: &AppHandle, data_root: &Path, db_path: &Path, e: &Entry, cancel: &AtomicBool) -> Result<(), AppError> {
    let id = e.id.as_str();
    emit(app, id, "resolving", 0, None, 0, None);
    let client = client()?;
    let r = resolve(&client, e).await?;

    let dl_dir = safe_join(data_root, &format!("downloads/{id}"))?;
    let (archive, sha, last_modified) = download(app, id, &client, &r, &dl_dir, cancel).await?;

    emit(app, id, "verifying", 0, None, 0, None);
    let integrity = verify(&sha, &r.expected_sha256)?;

    emit(app, id, "extracting", 0, None, 0, None);
    let staging = safe_join(data_root, &format!("cache/staging/{id}"))?;
    let (st2, ar2) = (staging.clone(), archive.clone());
    tauri::async_runtime::spawn_blocking(move || extract(&ar2, &st2))
        .await.map_err(|e| AppError::Invalid(e.to_string()))??;

    emit(app, id, "installing", 0, None, 0, None);
    let (root2, e2, st3) = (data_root.to_path_buf(), e.clone(), staging.clone());
    let (install, exe) = tauri::async_runtime::spawn_blocking(move || {
        let content = content_root(&st3)?;
        place(&e2, &root2, &content)
    }).await.map_err(|e| AppError::Invalid(e.to_string()))??;

    let version = if r.version == "nightly" { last_modified.unwrap_or_else(|| "nightly".into()) } else { r.version.clone() };
    register(db_path, e, &version, &install, &exe, integrity, &sha, r.url.as_str())?;
    Ok(())
}

pub async fn install(app: AppHandle, data_root: PathBuf, db_path: PathBuf, e: Entry, cancel: Arc<AtomicBool>) -> Result<(), AppError> {
    let id = e.id.clone();
    let staging = safe_join(&data_root, &format!("cache/staging/{id}"))?;
    let _ = fs::remove_dir_all(&staging);
    let res = run(&app, &data_root, &db_path, &e, &cancel).await;
    let _ = fs::remove_dir_all(safe_join(&data_root, &format!("downloads/{id}"))?);
    let _ = fs::remove_dir_all(&staging);
    match &res {
        Ok(_) => emit(&app, &id, "done", 0, None, 0, None),
        Err(AppError::Cancelled) => emit(&app, &id, "cancelled", 0, None, 0, None),
        Err(err) => emit(&app, &id, "error", 0, None, 0, Some(err.to_string())),
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("rt-{name}-{}-{}", std::process::id(), unix_ts()));
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn verify_logic() {
        assert_eq!(verify("ab", &Some("AB".into())).unwrap(), "verified");
        assert_eq!(verify("ab", &None).unwrap(), "unverified");
        assert!(verify("ab", &Some("cd".into())).is_err());
    }
    #[test]
    fn plain_name_strips_paths() {
        assert_eq!(plain_name("../../evil.exe").unwrap(), "evil.exe");
        assert_eq!(plain_name("a\\b\\c.zip").unwrap(), "c.zip");
        assert!(plain_name("..").is_err());
    }
    #[test]
    fn zip_extraction_rejects_traversal() {
        let d = tmp("zip");
        let zp = d.join("bad.zip");
        {
            let mut w = zip::ZipWriter::new(fs::File::create(&zp).unwrap());
            w.start_file("../escape.txt", zip::write::SimpleFileOptions::default()).unwrap();
            w.write_all(b"x").unwrap();
            w.finish().unwrap();
        }
        assert!(extract(&zp, &d.join("out")).is_err());
        assert!(!d.join("escape.txt").exists());
        let _ = fs::remove_dir_all(d);
    }
    #[test]
    fn zip_extraction_and_flatten_work() {
        let d = tmp("zip-ok");
        let zp = d.join("ok.zip");
        {
            let mut w = zip::ZipWriter::new(fs::File::create(&zp).unwrap());
            w.start_file("Emu/emu.exe", zip::write::SimpleFileOptions::default()).unwrap();
            w.write_all(b"MZ").unwrap();
            w.finish().unwrap();
        }
        let out = d.join("out");
        extract(&zp, &out).unwrap();
        let root = content_root(&out).unwrap();
        assert!(find_file(&root, &Regex::new("(?i)^emu\\.exe$").unwrap(), 2).is_some());
        let _ = fs::remove_dir_all(d);
    }
    #[test]
    fn replacing_install_keeps_old_copy_in_backups() {
        let root = tmp("place");
        let old = root.join("programs/emulators/emu-x");
        fs::create_dir_all(&old).unwrap();
        fs::write(old.join("saves.bin"), b"precious").unwrap();
        let moved = move_to_backup(&root, "components", "emu-x", &old).unwrap();
        assert!(!old.exists());
        assert_eq!(fs::read(moved.join("saves.bin")).unwrap(), b"precious");
        let _ = fs::remove_dir_all(root);
    }
}
