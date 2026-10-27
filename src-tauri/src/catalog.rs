//! Curated component catalog (catalog.json, embedded at build time).
//! No URL here is guessed: GitHub entries are resolved at install time through the official
//! Releases API; Libretro cores come from the official Libretro buildbot.
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    pub catalog_version: u32,
    pub components: Vec<Entry>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub kind: String, // "emulator" | "core"
    pub platforms: Vec<String>,
    pub license: String,
    pub homepage: String,
    #[serde(default)]
    pub notes: Option<String>,
    pub source: Source,
    /// Regex matched against file names to locate the main executable after extraction.
    #[serde(default)]
    pub exe_pattern: Option<String>,
    /// Empty file created in the install folder so the emulator keeps its data next to itself.
    #[serde(default)]
    pub portable_marker: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Source {
    LibretroCore { file: String },
    GithubRelease { repo: String, #[serde(rename = "assetPattern")] asset_pattern: String },
    DirectUrl { url: String },
    Manual { url: String },
}

impl Source {
    pub fn kind_name(&self) -> &'static str {
        match self {
            Source::LibretroCore { .. } => "libretro_core",
            Source::GithubRelease { .. } => "github_release",
            Source::DirectUrl { .. } => "direct_url",
            Source::Manual { .. } => "manual",
        }
    }
}

pub fn load() -> Catalog {
    serde_json::from_str(include_str!("../catalog.json")).expect("catalog.json is valid")
}

const ALLOWED_HOSTS: &[&str] = &["libretro.com", "github.com", "githubusercontent.com"];

pub fn url_allowed(u: &reqwest::Url) -> bool {
    u.scheme() == "https"
        && u.host_str().map_or(false, |h| {
            ALLOWED_HOSTS.iter().any(|d| h == *d || h.ends_with(&format!(".{d}")))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn catalog_is_valid() {
        let c = load();
        assert_eq!(c.catalog_version, 1);
        let mut ids = std::collections::HashSet::new();
        for e in &c.components {
            assert!(ids.insert(e.id.clone()), "duplicate id {}", e.id);
            assert!(e.kind == "emulator" || e.kind == "core");
            if let Some(p) = &e.exe_pattern { Regex::new(p).unwrap(); }
            if let Source::GithubRelease { asset_pattern, .. } = &e.source { Regex::new(asset_pattern).unwrap(); }
        }
    }
    #[test]
    fn host_allowlist() {
        let ok = |s: &str| url_allowed(&reqwest::Url::parse(s).unwrap());
        assert!(ok("https://buildbot.libretro.com/x"));
        assert!(ok("https://api.github.com/repos/a/b"));
        assert!(ok("https://release-assets.githubusercontent.com/x"));
        assert!(!ok("http://buildbot.libretro.com/x"));
        assert!(!ok("https://evilgithub.com/x"));
        assert!(!ok("https://github.com.evil.io/x"));
    }
}
