use std::fs;
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};

use lumatune_core::{FilterParams, Preset};
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

fn base() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}

fn root_in(base: &Path) -> PathBuf {
    base.join("LumaTune").join("presets")
}

fn dir() -> PathBuf {
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

pub struct StoredPreset {
    path: PathBuf,
    pub preset: Preset,
    pub defaults: Option<FilterParams>,
}

impl StoredPreset {
    pub fn id(&self) -> String {
        let file = self
            .path
            .file_name()
            .expect("preset file")
            .to_string_lossy();
        if self.defaults.is_some() {
            format!("built-in/{file}")
        } else {
            file.into_owned()
        }
    }

    pub fn save(&mut self, preset: Preset) -> io::Result<()> {
        write(&self.path, &preset, true)?;
        self.preset = preset;
        Ok(())
    }

    pub fn delete(&self) -> io::Result<()> {
        if self.defaults.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "初始预设不能删除",
            ));
        }
        fs::remove_file(&self.path)
    }
}

#[derive(Deserialize, Serialize)]
struct BindingMove {
    executable: String,
    from: String,
    to: String,
}

pub fn recover_binding_move(presets: &mut [StoredPreset]) -> io::Result<()> {
    recover_binding_move_in(&dir(), presets)
}

fn recover_binding_move_in(root: &Path, presets: &mut [StoredPreset]) -> io::Result<()> {
    let path = root.join(".binding-move");
    let pending: BindingMove = match fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    // Removal from the source is the commit point. Roll back the target while
    // the source still owns the binding.
    if presets
        .iter()
        .any(|entry| entry.id() == pending.from && entry.preset.matches_exe(&pending.executable))
        && let Some(target) = presets.iter_mut().find(|entry| entry.id() == pending.to)
    {
        target.preset.unbind(&pending.executable);
        write(&target.path, &target.preset, true)?;
    }
    fs::remove_file(path)
}

pub fn rebind(from: &mut StoredPreset, to: &mut StoredPreset, name: &str) -> io::Result<()> {
    rebind_in(&dir(), from, to, name)
}

fn rebind_in(
    root: &Path,
    from: &mut StoredPreset,
    to: &mut StoredPreset,
    name: &str,
) -> io::Result<()> {
    if !from.preset.matches_exe(name) || to.preset.matches_exe(name) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "该应用的绑定已发生变化",
        ));
    }
    let marker = root.join(".binding-move");
    let pending = BindingMove {
        executable: name.to_owned(),
        from: from.id(),
        to: to.id(),
    };
    write(&marker, &pending, false)?;

    let mut next = to.preset.clone();
    next.bind_to(name);
    if let Err(error) = write(&to.path, &next, true) {
        let _ = fs::remove_file(marker);
        return Err(error);
    }
    let mut previous = from.preset.clone();
    previous.unbind(name);
    if let Err(error) = from.save(previous) {
        if write(&to.path, &to.preset, true).is_ok() {
            let _ = fs::remove_file(marker);
        }
        return Err(error);
    }
    to.preset = next;
    let _ = fs::remove_file(marker);
    Ok(())
}

fn read(path: &Path) -> Option<Preset> {
    let text = fs::read_to_string(path).ok()?;
    let preset: Preset = serde_json::from_str(&text).ok()?;
    preset.is_compatible().then_some(preset)
}

fn list_in(root: &Path) -> Vec<StoredPreset> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        if let Some(preset) = read(&path) {
            out.push(StoredPreset {
                path,
                preset,
                defaults: None,
            });
        }
    }
    out.sort_by_key(|entry| entry.preset.name.to_lowercase());
    out
}

fn write(path: &Path, value: &impl Serialize, replace: bool) -> io::Result<()> {
    let parent = path.parent().expect("preset path has a parent directory");
    fs::create_dir_all(parent)?;
    let text = serde_json::to_string_pretty(value).map_err(io::Error::other)?;
    let mut pending = NamedTempFile::new_in(parent)?;
    pending.write_all(text.as_bytes())?;
    if replace {
        pending.persist(path)?;
    } else {
        pending.persist_noclobber(path)?;
    }
    Ok(())
}

fn create_in(root: &Path, preset: Preset) -> io::Result<StoredPreset> {
    let mut path = path_in(root, &preset.name);
    let mut suffix = 2;
    loop {
        match write(&path, &preset, false) {
            Ok(()) => {
                return Ok(StoredPreset {
                    path,
                    preset,
                    defaults: None,
                });
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                if read(&path).is_some_and(|existing| existing.name == preset.name) {
                    return Err(error);
                }
                // An older file can retain this name after the preset is renamed.
                path = root.join(format!("{}-{suffix}.json", sanitize(&preset.name)));
                suffix += 1;
            }
            Err(error) => return Err(error),
        }
    }
}

pub fn create(preset: Preset) -> io::Result<StoredPreset> {
    create_in(&dir(), preset)
}

fn built_ins(root: &Path) -> impl Iterator<Item = StoredPreset> {
    let root = root.join("built-in");
    [
        ("daily", "日常", FilterParams::default()),
        (
            "reading",
            "夜间阅读",
            FilterParams {
                temperature: 4800,
                brightness: -0.02,
                ..FilterParams::default()
            },
        ),
        ("photo", "照片校色", FilterParams::default()),
        (
            "game",
            "游戏",
            FilterParams {
                temperature: 7200,
                shadows: 0.08,
                ..FilterParams::default()
            },
        ),
        (
            "movie",
            "观影",
            FilterParams {
                temperature: 6000,
                contrast: 1.12,
                ..FilterParams::default()
            },
        ),
        (
            "office",
            "办公",
            FilterParams {
                temperature: 5800,
                ..FilterParams::default()
            },
        ),
    ]
    .into_iter()
    .map(move |(id, name, defaults)| {
        let path = root.join(format!("{id}.json"));
        let preset = read(&path).unwrap_or_else(|| Preset::new(name, defaults.clone()));
        StoredPreset {
            path,
            preset,
            defaults: Some(defaults),
        }
    })
}

pub fn list() -> Vec<StoredPreset> {
    let root = dir();
    // Keep the previous precedence when a legacy preset already binds an executable.
    list_in(&root).into_iter().chain(built_ins(&root)).collect()
}

pub fn cover_path(name: &str) -> Option<PathBuf> {
    let mut parts = Path::new(name).components();
    if !matches!(parts.next(), Some(Component::Normal(_))) || parts.next().is_some() {
        return None;
    }
    // Only accept an app-managed file name, including on Windows ADS paths.
    if !name.starts_with("cover-")
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
    {
        return None;
    }
    Some(dir().join("covers").join(name))
}

pub fn import_cover(source: &Path) -> io::Result<String> {
    let ext = source
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp" | "bmp") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "不支持的图片格式",
        ));
    }
    let covers = dir().join("covers");
    fs::create_dir_all(&covers)?;
    let mut pending = tempfile::Builder::new()
        .prefix("cover-")
        .suffix(&format!(".{ext}"))
        .tempfile_in(covers)?;
    io::copy(&mut fs::File::open(source)?, &mut pending)?;
    let (_, path) = pending.keep()?;
    Ok(path
        .file_name()
        .expect("generated file name")
        .to_string_lossy()
        .into_owned())
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

        let entry = create_in(&root, preset).unwrap();
        assert!(entry.path.exists());

        let loaded = list_in(&root);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].preset.name, "暗部增强");
        assert!((loaded[0].preset.params.gamma - 2.2).abs() < 1e-6);

        entry.delete().unwrap();
        assert!(list_in(&root).is_empty());
    }

    #[test]
    fn binding_move_recovers_when_only_the_target_was_written() {
        let scratch = Scratch::new("binding-recovery");
        let root = scratch.root();
        let mut original = Preset::new("Original", FilterParams::default());
        original.bind_to("viewer.exe");
        let source = create_in(&root, original).unwrap();
        let mut target = Preset::new("Target", FilterParams::default());
        target.bind_to("viewer.exe");
        let target = create_in(&root, target).unwrap();
        let marker = root.join(".binding-move");
        write(
            &marker,
            &BindingMove {
                executable: "viewer.exe".into(),
                from: source.id(),
                to: target.id(),
            },
            false,
        )
        .unwrap();

        let mut loaded = list_in(&root);
        recover_binding_move_in(&root, &mut loaded).unwrap();
        assert!(!marker.exists());
        assert_eq!(
            list_in(&root)
                .iter()
                .filter(|entry| entry.preset.matches_exe("viewer.exe"))
                .map(StoredPreset::id)
                .collect::<Vec<_>>(),
            [source.id()]
        );
    }

    #[test]
    fn binding_move_rolls_back_if_the_source_write_fails() {
        let scratch = Scratch::new("binding-rollback");
        let root = scratch.root();
        let mut original = Preset::new("Original", FilterParams::default());
        original.bind_to("viewer.exe");
        let mut source = create_in(&root, original).unwrap();
        let mut target = create_in(&root, Preset::new("Target", FilterParams::default())).unwrap();
        fs::remove_file(&source.path).unwrap();
        fs::create_dir(&source.path).unwrap();

        assert!(rebind_in(&root, &mut source, &mut target, "viewer.exe").is_err());
        assert!(source.preset.matches_exe("viewer.exe"));
        assert!(!read(&target.path).unwrap().matches_exe("viewer.exe"));
        assert!(!root.join(".binding-move").exists());
    }

    #[test]
    fn saving_an_existing_record_preserves_its_file_and_metadata() {
        let scratch = Scratch::new("overwrite");
        let root = scratch.root();

        let mut params = FilterParams::default();
        params.contrast = 1.1;
        let mut original = Preset::new("高对比", params.clone());
        original.bind_to("viewer.exe");
        original.favorite = true;
        let mut entry = create_in(&root, original.clone()).unwrap();
        assert!(create_in(&root, original).is_err());
        params.contrast = 1.4;
        let mut updated = entry.preset.clone();
        updated.name = "高对比 2".into();
        updated.params = params;
        entry.save(updated.clone()).unwrap();

        let loaded = list_in(&root);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].path, entry.path);
        assert_eq!(loaded[0].preset, updated);
        let separate = create_in(&root, Preset::new("高对比", FilterParams::default())).unwrap();
        assert_ne!(separate.path, entry.path);
    }

    #[test]
    fn built_in_defaults_are_independent_of_saved_edits() {
        let scratch = Scratch::new("built-in");
        let root = scratch.root();
        let mut entry = built_ins(&root).nth(1).unwrap();
        let mut edited = entry.preset.clone();
        edited.params.temperature = 8200;
        entry.save(edited).unwrap();
        let loaded = built_ins(&root).nth(1).unwrap();
        assert_eq!(loaded.preset.params.temperature, 8200);
        assert_eq!(loaded.defaults.as_ref().unwrap().temperature, 4800);
        assert!(loaded.delete().is_err());
    }

    #[test]
    fn broken_file_is_skipped() {
        let scratch = Scratch::new("broken");
        let root = scratch.root();
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("bad.json"), "{ not json").unwrap();
        create_in(&root, Preset::new("好预设", FilterParams::default())).unwrap();

        let loaded = list_in(&root);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].preset.name, "好预设");
    }

    #[test]
    fn illegal_characters_are_replaced_in_filename() {
        assert_eq!(sanitize("a/b:c*d"), "a_b_c_d");
        assert_eq!(sanitize("  "), "未命名");
        assert_eq!(sanitize("trailing."), "trailing");
    }
}
