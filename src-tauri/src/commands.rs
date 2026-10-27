//! The only surface the frontend can call. Each command validates its input.
use crate::{catalog::{self, Source}, error::AppError, installer, paths, settings::Settings, state::{AppState, Mode}};
use rusqlite::Connection;
use serde::Serialize;
use std::{collections::HashMap, path::PathBuf, sync::{atomic::{AtomicBool, Ordering}, Arc}};
use tauri::State;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutEntry { path: String, exists: bool }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    version: String,
    mode: Mode,
    data_root: String,
    config_dir: String,
    db_path: String,
    schema_version: i64,
    warning: Option<String>,
    layout: Vec<LayoutEntry>,
}

#[tauri::command]
pub fn get_app_info(state: State<AppState>) -> Result<AppInfo, AppError> {
    let layout = paths::DATA_LAYOUT.iter().map(|rel| {
        let exists = paths::safe_join(&state.data_root, rel).map(|p| p.is_dir()).unwrap_or(false);
        LayoutEntry { path: (*rel).to_string(), exists }
    }).collect();
    Ok(AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        mode: state.mode,
        data_root: state.data_root.display().to_string(),
        config_dir: state.config_dir.display().to_string(),
        db_path: state.db_path.display().to_string(),
        schema_version: state.schema_version,
        warning: state.warning.clone(),
        layout,
    })
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<Settings, AppError> {
    Ok(state.settings.lock().unwrap().clone())
}

#[tauri::command]
pub fn save_settings(state: State<AppState>, settings: Settings) -> Result<Settings, AppError> {
    // Data root is changed only through set_data_root, never through this command.
    let mut cur = state.settings.lock().unwrap();
    let mut next = settings;
    next.data_root = cur.data_root.clone();
    next.save(&state.config_dir)?;
    *cur = next.clone();
    Ok(next)
}

/// Records a new data root. Existing files are NOT moved: the new location takes effect
/// after restart and starts with a fresh layout/database (documented limitation).
#[tauri::command]
pub fn set_data_root(state: State<AppState>, path: String) -> Result<Settings, AppError> {
    let p = PathBuf::from(&path);
    if !p.is_absolute() {
        return Err(AppError::Invalid("the data root must be an absolute path".into()));
    }
    std::fs::create_dir_all(&p)?;
    paths::ensure_layout(&p)?; // also proves the folder is writable
    let mut cur = state.settings.lock().unwrap();
    let mut next = cur.clone();
    next.data_root = Some(p.display().to_string());
    next.save(&state.config_dir)?;
    *cur = next.clone();
    Ok(next)
}

// ---------------- component manager ----------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentView {
    id: String,
    name: String,
    kind: String,
    platforms: Vec<String>,
    license: String,
    homepage: String,
    notes: Option<String>,
    source_type: &'static str,
    auto_install: bool,
    installed: bool,
    installed_version: Option<String>,
    integrity: Option<String>,
    install_path: Option<String>,
    exe_path: Option<String>,
}

type Row = (Option<String>, Option<String>, Option<String>, Option<String>);

fn installed_rows(state: &AppState) -> Result<HashMap<String, Row>, AppError> {
    let conn = Connection::open(&state.db_path)?;
    let mut stmt = conn.prepare("SELECT id, installed_version, install_path, integrity, exe_path FROM components WHERE status = 'installed'")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, (r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))))?;
    Ok(rows.collect::<Result<HashMap<_, _>, _>>()?)
}

#[tauri::command]
pub fn list_components(state: State<AppState>) -> Result<Vec<ComponentView>, AppError> {
    let rows = installed_rows(&state)?;
    Ok(catalog::load().components.into_iter().map(|e| {
        let row = rows.get(&e.id);
        let abs = |rel: &Option<String>| rel.as_ref().and_then(|r| paths::safe_join(&state.data_root, r).ok());
        let install_abs = row.and_then(|r| abs(&r.1));
        // "installed" only if the files are really still on disk.
        let installed = install_abs.as_ref().map_or(false, |p| p.exists());
        ComponentView {
            source_type: e.source.kind_name(),
            auto_install: !matches!(e.source, Source::Manual { .. }),
            installed_version: row.and_then(|r| r.0.clone()).filter(|_| installed),
            integrity: row.and_then(|r| r.2.clone()).filter(|_| installed),
            install_path: install_abs.filter(|_| installed).map(|p| p.display().to_string()),
            exe_path: row.and_then(|r| abs(&r.3)).filter(|p| p.exists()).map(|p| p.display().to_string()),
            installed, id: e.id, name: e.name, kind: e.kind, platforms: e.platforms,
            license: e.license, homepage: e.homepage, notes: e.notes,
        }
    }).collect())
}

#[tauri::command]
pub async fn install_component(app: tauri::AppHandle, state: State<'_, AppState>, id: String) -> Result<(), AppError> {
    let entry = catalog::load().components.into_iter().find(|c| c.id == id)
        .ok_or_else(|| AppError::Invalid(format!("componente desconhecido: {id}")))?;
    if matches!(entry.source, Source::Manual { .. }) {
        return Err(AppError::Invalid("Este componente não tem download automático aprovado.".into()));
    }
    let cancel = Arc::new(AtomicBool::new(false));
    {
        let mut active = state.active.lock().unwrap();
        if active.contains_key(&id) { return Err(AppError::Invalid("Este componente já está sendo instalado.".into())); }
        active.insert(id.clone(), cancel.clone());
    }
    let result = installer::install(app, state.data_root.clone(), state.db_path.clone(), entry, cancel).await;
    state.active.lock().unwrap().remove(&id);
    result
}

#[tauri::command]
pub fn cancel_install(state: State<AppState>, id: String) -> bool {
    match state.active.lock().unwrap().get(&id) {
        Some(flag) => { flag.store(true, Ordering::Relaxed); true }
        None => false,
    }
}

/// Never deletes: the install is moved to backups/removed/ so saves kept inside it stay recoverable.
#[tauri::command]
pub fn uninstall_component(state: State<AppState>, id: String) -> Result<String, AppError> {
    if state.active.lock().unwrap().contains_key(&id) {
        return Err(AppError::Invalid("Aguarde ou cancele a instalação em andamento.".into()));
    }
    let conn = Connection::open(&state.db_path)?;
    let rel: Option<String> = conn.query_row("SELECT install_path FROM components WHERE id = ?1", [&id], |r| r.get(0))
        .map_err(|_| AppError::Invalid("Componente não está instalado.".into()))?;
    let rel = rel.ok_or_else(|| AppError::Invalid("Componente sem caminho registrado.".into()))?;
    let src = paths::safe_join(&state.data_root, &rel)?;
    let dest = if src.exists() { Some(installer::move_to_backup(&state.data_root, "removed", &id, &src)?) } else { None };
    conn.execute("UPDATE components SET status = 'not_installed', installed_version = NULL, install_path = NULL, exe_path = NULL WHERE id = ?1", [&id])?;
    Ok(dest.map(|d| d.display().to_string()).unwrap_or_default())
}
