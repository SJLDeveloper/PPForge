//! Saving/loading characters and app settings using platform-appropriate directories.

use crate::model::Character;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const EXTENSION: &str = "ppchar";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub dark_mode: bool,
    pub ui_scale: f32,
    pub recent: Vec<PathBuf>,
    pub library_dir: Option<PathBuf>,
    pub default_ruleset: crate::rules::Ruleset,
    pub reopen_last: bool,
    pub last_open: Vec<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            dark_mode: true,
            ui_scale: 1.0,
            recent: Vec::new(),
            library_dir: None,
            default_ruleset: crate::rules::Ruleset::Errata,
            reopen_last: true,
            last_open: Vec::new(),
        }
    }
}

fn project_dirs() -> Option<directories::ProjectDirs> {
    directories::ProjectDirs::from("org", "PPForge", "PP Forge")
}

pub fn settings_path() -> Option<PathBuf> {
    project_dirs().map(|d| d.config_dir().join("settings.json"))
}

impl Settings {
    pub fn load() -> Settings {
        settings_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }
    pub fn save(&self) {
        if let Some(p) = settings_path() {
            if let Some(dir) = p.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            if let Ok(s) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(p, s);
            }
        }
    }
    pub fn push_recent(&mut self, p: &Path) {
        self.recent.retain(|r| r != p);
        self.recent.insert(0, p.to_path_buf());
        self.recent.truncate(12);
    }
    /// Default folder for character files: <Documents>/PP Forge, falling back to the data dir.
    pub fn library(&self) -> PathBuf {
        if let Some(d) = &self.library_dir {
            return d.clone();
        }
        if let Some(u) = directories::UserDirs::new()
            && let Some(docs) = u.document_dir() {
                return docs.join("PP Forge");
            }
        project_dirs().map(|d| d.data_dir().to_path_buf()).unwrap_or_else(|| PathBuf::from("."))
    }
}

pub fn save_character(c: &Character, path: &Path) -> Result<(), String> {
    let json = c.to_json().map_err(|e| e.to_string())?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    // Write atomically: temp file then rename, so a crash never leaves a half-written save.
    let tmp = path.with_extension(format!("{EXTENSION}.tmp"));
    std::fs::write(&tmp, json).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}

pub fn load_character(path: &Path) -> Result<Character, String> {
    let s = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let c = Character::from_json(&s).map_err(|e| format!("Not a valid character file: {e}"))?;
    if c.format_version > crate::model::FORMAT_VERSION {
        return Err("This file was saved by a newer version of PP Forge".into());
    }
    Ok(c)
}

/// List character files in the library folder, newest first.
pub fn list_library(dir: &Path) -> Vec<(PathBuf, String)> {
    let mut out: Vec<(PathBuf, String, std::time::SystemTime)> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) == Some(EXTENSION) {
                let name = p.file_stem().and_then(|s| s.to_str()).unwrap_or("?").to_string();
                let t = e.metadata().and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH);
                out.push((p, name, t));
            }
        }
    }
    out.sort_by_key(|x| std::cmp::Reverse(x.2));
    out.into_iter().map(|(p, n, _)| (p, n)).collect()
}

/// A filesystem-safe file name for a character.
pub fn safe_file_name(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let s = s.trim().to_string();
    if s.is_empty() { "character".into() } else { s }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_and_load() {
        let dir = std::env::temp_dir().join(format!("ppforge-test-{}", std::process::id()));
        let path = dir.join("hero.ppchar");
        let mut c = Character::new(crate::rules::Ruleset::Core);
        c.story.hero_name = "Saved".into();
        save_character(&c, &path).unwrap();
        let back = load_character(&path).unwrap();
        assert_eq!(back.story.hero_name, "Saved");
        assert_eq!(list_library(&dir).len(), 1);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn file_names_are_safe() {
        assert_eq!(safe_file_name("Dr. Doom/Victor?"), "Dr_ Doom_Victor_");
        assert_eq!(safe_file_name("  "), "character");
    }
}
