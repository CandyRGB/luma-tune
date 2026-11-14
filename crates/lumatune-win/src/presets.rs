use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use lumatune_core::Preset;

fn base() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}

fn root_in(base: &Path) -> PathBuf {
    base.join("LumaTune").join("presets")
}

pub fn dir() -> PathBuf {
    root_in(&base())
}

fn sanitize(name: &str) -> String {
    let mut out: String = name
        .trim()
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    while out.ends_with('.') || out.ends_with(' ') {
        out.pop();
    }
    if out.is_empty() {
        out.push_str("未命名");
    }
    out
}

fn path_in(root: &Path, name: &str) -> PathBuf {
    root.join(format!("{}.json", sanitize(name)))
}

pub fn path_for(name: &str) -> PathBuf {
    path_in(&dir(), name)
}

fn list_in(root: &Path) -> Vec<Preset> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        if let Ok(text) = fs::read_to_string(&path) {
            if let Ok(preset) = serde_json::from_str::<Preset>(&text) {
                if preset.is_compatible() {
                    out.push(preset);
                }
            }
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

fn save_in(root: &Path, preset: &Preset) -> io::Result<PathBuf> {
    fs::create_dir_all(root)?;
    let path = path_in(root, &preset.name);
    let text = serde_json::to_string_pretty(preset).map_err(io::Error::other)?;
    fs::write(&path, text)?;
    Ok(path)
}

fn remove_in(root: &Path, name: &str) -> io::Result<()> {
    fs::remove_file(path_in(root, name))
}

pub fn list() -> Vec<Preset> {
    list_in(&dir())
}

pub fn save(preset: &Preset) -> io::Result<PathBuf> {
    save_in(&dir(), preset)
}

pub fn remove(name: &str) -> io::Result<()> {
    remove_in(&dir(), name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumatune_core::FilterParams;

    struct Scratch(PathBuf);

    impl Scratch {
        fn new(tag: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "lumatune-presets-{}-{}",
                tag,
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn root(&self) -> PathBuf {
            root_in(&self.0)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn save_list_remove_roundtrip() {
        let scratch = Scratch::new("roundtrip");
        let root = scratch.root();

        let mut params = FilterParams::default();
        params.gamma = 2.2;
        let preset = Preset::new("暗部增强", params);

        let path = save_in(&root, &preset).unwrap();
        assert!(path.exists());

        let loaded = list_in(&root);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "暗部增强");
        assert!((loaded[0].params.gamma - 2.2).abs() < 1e-6);

        remove_in(&root, &preset.name).unwrap();
        assert!(list_in(&root).is_empty());
    }

    #[test]
    fn saving_same_name_overwrites_instead_of_duplicating() {
        let scratch = Scratch::new("overwrite");
        let root = scratch.root();

        let mut params = FilterParams::default();
        params.contrast = 1.1;
        save_in(&root, &Preset::new("高对比", params.clone())).unwrap();
        params.contrast = 1.4;
        save_in(&root, &Preset::new("高对比", params)).unwrap();

        let loaded = list_in(&root);
        assert_eq!(loaded.len(), 1);
        assert!((loaded[0].params.contrast - 1.4).abs() < 1e-6);
    }

    #[test]
    fn broken_file_is_skipped() {
        let scratch = Scratch::new("broken");
        let root = scratch.root();
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("bad.json"), "{ not json").unwrap();
        save_in(&root, &Preset::new("好预设", FilterParams::default())).unwrap();

        let loaded = list_in(&root);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "好预设");
    }

    #[test]
    fn illegal_characters_are_replaced_in_filename() {
        assert_eq!(sanitize("a/b:c*d"), "a_b_c_d");
        assert_eq!(sanitize("  "), "未命名");
        assert_eq!(sanitize("trailing."), "trailing");
    }
}
