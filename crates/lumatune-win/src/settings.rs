use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemeMode {
    pub fn index(self) -> i32 {
        match self {
            ThemeMode::System => 0,
            ThemeMode::Light => 1,
            ThemeMode::Dark => 2,
        }
    }

    pub fn from_index(value: i32) -> Self {
        match value {
            1 => ThemeMode::Light,
            2 => ThemeMode::Dark,
            _ => ThemeMode::System,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub theme: ThemeMode,
}

fn base() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}

fn root_in(base: &Path) -> PathBuf {
    base.join("LumaTune")
}

fn file_in(root: &Path) -> PathBuf {
    root.join("settings.json")
}

pub fn path() -> PathBuf {
    file_in(&root_in(&base()))
}

pub fn load() -> Settings {
    load_in(&root_in(&base()))
}

pub fn save(settings: &Settings) -> io::Result<()> {
    save_in(&root_in(&base()), settings)
}

fn load_in(root: &Path) -> Settings {
    fs::read_to_string(file_in(root))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn save_in(root: &Path, settings: &Settings) -> io::Result<()> {
    fs::create_dir_all(root)?;
    let text = serde_json::to_string_pretty(settings)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    fs::write(file_in(root), text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("lumatune-settings-{tag}"));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn missing_file_yields_defaults() {
        let root = temp_root("missing");
        assert_eq!(load_in(&root).theme, ThemeMode::System);
    }

    #[test]
    fn roundtrip_preserves_theme() {
        let root = temp_root("roundtrip");
        let settings = Settings {
            theme: ThemeMode::Dark,
        };
        save_in(&root, &settings).unwrap();
        assert_eq!(load_in(&root).theme, ThemeMode::Dark);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn corrupt_file_falls_back_to_defaults() {
        let root = temp_root("corrupt");
        fs::create_dir_all(&root).unwrap();
        fs::write(file_in(&root), "{ not json").unwrap();
        assert_eq!(load_in(&root).theme, ThemeMode::System);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn index_mapping_is_stable() {
        assert_eq!(ThemeMode::System.index(), 0);
        assert_eq!(ThemeMode::Light.index(), 1);
        assert_eq!(ThemeMode::Dark.index(), 2);
        assert_eq!(ThemeMode::from_index(2), ThemeMode::Dark);
        assert_eq!(ThemeMode::from_index(9), ThemeMode::System);
    }
}
