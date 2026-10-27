use crate::{db, error::AppError, paths, settings::Settings};
use rusqlite::Connection;
use serde::Serialize;
use std::{fs, path::PathBuf, sync::Mutex};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode { Portable, Installed }

pub struct AppState {
    pub mode: Mode,
    pub config_dir: PathBuf,
    pub data_root: PathBuf,
    pub db_path: PathBuf,
    pub schema_version: i64,
    pub settings: Mutex<Settings>,
}

impl AppState {
    pub fn init(app: &AppHandle) -> Result<Self, AppError> {
        let exe_dir = std::env::current_exe()?
            .parent()
            .ok_or_else(|| AppError::Invalid("cannot resolve executable directory".into()))?
            .to_path_buf();

        // Portable mode: a `portable.flag` file next to the executable.
        let (mode, config_dir, default_root) = if exe_dir.join("portable.flag").exists() {
            let root = exe_dir.join("RetrosTeamData");
            (Mode::Portable, root.join("config"), root)
        } else {
            let cfg = app.path().app_config_dir().map_err(|e| AppError::Invalid(e.to_string()))?;
            let data = app.path().app_data_dir().map_err(|e| AppError::Invalid(e.to_string()))?;
            (Mode::Installed, cfg, data.join("RetrosTeam"))
        };

        fs::create_dir_all(&config_dir)?;
        let settings = Settings::load(&config_dir)?;
        let data_root = match &settings.data_root {
            Some(p) => PathBuf::from(p),
            None => default_root,
        };
        paths::ensure_layout(&data_root)?;

        let db_path = paths::safe_join(&data_root, "database/retrosteam.db")?;
        let mut conn = Connection::open(&db_path)?;
        let schema_version = db::migrate(&mut conn)?;

        Ok(Self { mode, config_dir, data_root, db_path, schema_version, settings: Mutex::new(settings) })
    }
}
