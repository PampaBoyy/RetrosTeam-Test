use crate::{db, error::AppError, paths, settings::Settings};
use rusqlite::Connection;
use serde::Serialize;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, Arc, Mutex},
};
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
    pub warning: Option<String>,
    pub settings: Mutex<Settings>,
    /// Cancel flags of installs currently running, by component id.
    pub active: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

fn is_writable(dir: &Path) -> bool {
    let probe = dir.join(".write-test");
    let ok = fs::write(&probe, b"").is_ok();
    let _ = fs::remove_file(probe);
    ok
}

impl AppState {
    pub fn init(app: &AppHandle) -> Result<Self, AppError> {
        let exe_dir = std::env::current_exe()?
            .parent()
            .ok_or_else(|| AppError::Invalid("cannot resolve executable directory".into()))?
            .to_path_buf();

        // Portable is the DEFAULT: everything lives in RetrosTeamData next to the exe.
        // Falls back to the per-user folder if that location is read-only (e.g. Program Files),
        // or when an `installed.flag` file sits next to the exe.
        let portable_root = exe_dir.join("RetrosTeamData");
        let want_portable = !exe_dir.join("installed.flag").exists();
        let portable_ok = want_portable && fs::create_dir_all(&portable_root).is_ok() && is_writable(&portable_root);

        let mut warning = None;
        let (mode, config_dir, default_root) = if portable_ok {
            (Mode::Portable, portable_root.join("config"), portable_root)
        } else {
            if want_portable {
                warning = Some("A pasta do executável não é gravável; usando a pasta do usuário. Mova o RetrosTeam para uma pasta com permissão de escrita para o modo portátil.".to_string());
            }
            let cfg = app.path().app_config_dir().map_err(|e| AppError::Invalid(e.to_string()))?;
            let data = app.path().app_data_dir().map_err(|e| AppError::Invalid(e.to_string()))?;
            (Mode::Installed, cfg, data.join("RetrosTeam"))
        };

        fs::create_dir_all(&config_dir)?;
        let settings = Settings::load(&config_dir)?;
        // A custom data root that no longer exists (e.g. the folder was moved to another PC/drive)
        // is ignored instead of failing, so the app always starts.
        let data_root = match &settings.data_root {
            Some(p) if Path::new(p).is_dir() => PathBuf::from(p),
            Some(p) => {
                warning = Some(format!("A pasta de dados configurada não foi encontrada ({p}); usando a pasta padrão."));
                default_root
            }
            None => default_root,
        };
        paths::ensure_layout(&data_root)?;

        let db_path = paths::safe_join(&data_root, "database/retrosteam.db")?;
        let mut conn = Connection::open(&db_path)?;
        let schema_version = db::migrate(&mut conn)?;

        Ok(Self {
            mode, config_dir, data_root, db_path, schema_version, warning,
            settings: Mutex::new(settings),
            active: Mutex::new(HashMap::new()),
        })
    }
}
