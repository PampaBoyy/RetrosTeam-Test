use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub schema_version: u32,
    pub theme: String,
    pub language: String,
    /// User-chosen data root. `None` means the mode's default location.
    pub data_root: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self { schema_version: 1, theme: "dark".into(), language: "pt-BR".into(), data_root: None }
    }
}

impl Settings {
    pub fn validate(&self) -> Result<(), AppError> {
        if !["dark", "light"].contains(&self.theme.as_str()) {
            return Err(AppError::Invalid(format!("unknown theme '{}'", self.theme)));
        }
        if !["pt-BR", "en"].contains(&self.language.as_str()) {
            return Err(AppError::Invalid(format!("unsupported language '{}'", self.language)));
        }
        Ok(())
    }

    pub fn load(config_dir: &Path) -> Result<Self, AppError> {
        let file = config_dir.join("settings.json");
        if !file.exists() {
            return Ok(Self::default());
        }
        Ok(serde_json::from_str(&fs::read_to_string(file)?)?)
    }

    /// Atomic save: write a temp file, then rename over the target.
    pub fn save(&self, config_dir: &Path) -> Result<(), AppError> {
        self.validate()?;
        fs::create_dir_all(config_dir)?;
        let tmp = config_dir.join("settings.json.tmp");
        fs::write(&tmp, serde_json::to_string_pretty(self)?)?;
        fs::rename(tmp, config_dir.join("settings.json"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip_and_defaults() {
        let d = std::env::temp_dir().join(format!("rt-settings-{}", std::process::id()));
        assert_eq!(Settings::load(&d).unwrap(), Settings::default());
        let mut s = Settings::default();
        s.theme = "light".into();
        s.save(&d).unwrap();
        assert_eq!(Settings::load(&d).unwrap().theme, "light");
        let _ = fs::remove_dir_all(d);
    }
    #[test]
    fn rejects_bad_values() {
        let mut s = Settings::default();
        s.theme = "neon".into();
        assert!(s.validate().is_err());
    }
}
