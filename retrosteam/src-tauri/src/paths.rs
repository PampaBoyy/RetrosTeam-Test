//! Safe path handling. Every path built from a relative string goes through `safe_join`.
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PathError {
    #[error("path is empty")]
    Empty,
    #[error("path must be relative: {0}")]
    Absolute(String),
    #[error("path escapes its root: {0}")]
    Escapes(String),
}

/// Directories created under the data root (see docs/ARCHITECTURE.md).
pub const DATA_LAYOUT: &[&str] = &[
    "programs/emulators", "programs/tools", "cores",
    "roms/PS1", "roms/PS2", "roms/PSP", "roms/SNES", "roms/GBA",
    "games/native", "games/ports", "games/recompilations",
    "profiles/global", "profiles/platforms", "profiles/games",
    "mods", "userdata/saves", "userdata/states", "userdata/screenshots",
    "metadata", "cache", "downloads", "backups", "database", "logs",
];

/// Join `rel` onto `root`, rejecting absolute paths, drive prefixes and `..`.
/// Purely lexical: does not touch the filesystem (symlink checks come with extraction code).
pub fn safe_join(root: &Path, rel: &str) -> Result<PathBuf, PathError> {
    if rel.trim().is_empty() {
        return Err(PathError::Empty);
    }
    let p = Path::new(rel);
    if p.is_absolute() || p.has_root() {
        return Err(PathError::Absolute(rel.to_string()));
    }
    let mut out = root.to_path_buf();
    for c in p.components() {
        match c {
            Component::Normal(s) => out.push(s),
            Component::CurDir => {}
            _ => return Err(PathError::Escapes(rel.to_string())),
        }
    }
    Ok(out)
}

pub fn ensure_layout(root: &Path) -> std::io::Result<()> {
    for rel in DATA_LAYOUT {
        // DATA_LAYOUT is a trusted constant, so safe_join cannot fail here.
        let dir = safe_join(root, rel).expect("static layout path is valid");
        fs::create_dir_all(dir)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_normal_paths_with_spaces_and_unicode() {
        let r = Path::new("root");
        assert_eq!(safe_join(r, "roms/Super Nintendo/Pokémon ção.sfc").unwrap(),
                   r.join("roms").join("Super Nintendo").join("Pokémon ção.sfc"));
    }
    #[test]
    fn rejects_traversal() {
        assert!(matches!(safe_join(Path::new("r"), "../x"), Err(PathError::Escapes(_))));
        assert!(matches!(safe_join(Path::new("r"), "a/../../x"), Err(PathError::Escapes(_))));
    }
    #[test]
    fn rejects_absolute_and_empty() {
        assert!(matches!(safe_join(Path::new("r"), "/etc/passwd"), Err(PathError::Absolute(_))));
        assert_eq!(safe_join(Path::new("r"), "  "), Err(PathError::Empty));
    }
    #[cfg(windows)]
    #[test]
    fn rejects_windows_drive_and_unc() {
        assert!(safe_join(Path::new("r"), r"C:\Windows").is_err());
        assert!(safe_join(Path::new("r"), r"C:foo").is_err());
        assert!(safe_join(Path::new("r"), r"\\server\share").is_err());
    }
    #[test]
    fn layout_has_no_unsafe_entries() {
        for rel in DATA_LAYOUT { assert!(safe_join(Path::new("r"), rel).is_ok()); }
    }
}
