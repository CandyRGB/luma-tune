use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use lumatune_core::Ramp;
use tempfile::NamedTempFile;
use windows::Win32::Graphics::Gdi::DeleteDC;

use crate::display::Display;
use crate::ramp;

fn dir() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("LumaTune")
}

pub fn claim() -> io::Result<fs::File> {
    let root = dir();
    fs::create_dir_all(&root)?;
    let file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(root.join("display.lock"))?;
    file.try_lock()?;
    Ok(file)
}

fn journal() -> io::Result<BTreeMap<String, Vec<u16>>> {
    match fs::read(dir().join("display-recovery.json")) {
        Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(BTreeMap::new()),
        Err(error) => Err(error),
    }
}

fn store(entries: &BTreeMap<String, Vec<u16>>) -> io::Result<()> {
    let root = dir();
    fs::create_dir_all(&root)?;
    let mut pending = NamedTempFile::new_in(&root)?;
    serde_json::to_writer(&mut pending, entries)?;
    pending.flush()?;
    pending.persist(root.join("display-recovery.json"))?;
    Ok(())
}

pub(crate) fn mark_active(display: &str, original: &Ramp) -> bool {
    (|| -> io::Result<()> {
        let mut entries = journal()?;
        entries.insert(display.to_owned(), original.to_vec());
        store(&entries)
    })()
    .is_ok()
}

pub(crate) fn clear_active(display: &str) {
    if let Ok(mut entries) = journal()
        && entries.remove(display).is_some()
    {
        let _ = store(&entries);
    }
}

pub fn heal(display: &Display) -> bool {
    let Some(id) = &display.identity else {
        return false;
    };
    let Ok(mut entries) = journal() else {
        return false;
    };
    if let Some(saved) = entries.get(id) {
        let Ok(original) = <&Ramp>::try_from(saved.as_slice()) else {
            return false;
        };
        if restore(display, original) {
            entries.remove(id);
            let _ = store(&entries);
            return true;
        }
        return false;
    }

    // Before the journal existed, only one device name was recorded.
    let legacy = dir().join("active.flag");
    if fs::read_to_string(&legacy).is_ok_and(|device| device.trim() == display.device)
        && restore(display, &ramp::identity())
    {
        let _ = fs::remove_file(legacy);
        return true;
    }
    false
}

pub(crate) fn has_pending(display: &Display) -> bool {
    journal().map_or(true, |entries| entries.contains_key(display.key()))
}

fn restore(display: &Display, original: &Ramp) -> bool {
    let hdc = display.hdc();
    if hdc.0.is_null() {
        return false;
    }
    let restored = ramp::write_verified(hdc, original);
    unsafe {
        let _ = DeleteDC(hdc);
    }
    restored
}
