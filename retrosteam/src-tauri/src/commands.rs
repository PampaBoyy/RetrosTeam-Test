//! The only surface the frontend can call. Each command validates its input.
use crate::{error::AppError, paths, settings::Settings, state::{AppState, Mode}};
use serde::Serialize;
use std::path::PathBuf;
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
