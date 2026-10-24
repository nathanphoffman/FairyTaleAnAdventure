//! Mapping Amiga file names to real paths.

use std::path::{Path, PathBuf};

const DATA_DIR_VAR: &str = "FTA_DATA";
const DEFAULT_DATA_DIR: &str = "faery-tale-amiga/game";
const SAVES_DIR_VAR: &str = "FTA_SAVES";
const DEFAULT_SAVES_DIR: &str = "saves";
const SAVE_EXTENSION: &str = ".faery";
/// A disk device name is "df" plus a unit digit, like "df1".
const DISK_DEVICE_PREFIX: &str = "df";
const DISK_DEVICE_NAME_LEN: usize = 3;

fn data_dir() -> PathBuf {
    std::env::var_os(DATA_DIR_VAR)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_DATA_DIR))
}

fn save_dir() -> PathBuf {
    PathBuf::from(std::env::var_os(SAVES_DIR_VAR).unwrap_or_else(|| DEFAULT_SAVES_DIR.into()))
}

/// Finds `name` under `base`, matching each path component case-insensitively.
fn find_ignore_case(base: &Path, name: &str) -> Option<PathBuf> {
    let mut cur = base.to_path_buf();
    for part in name.split('/').filter(|p| !p.is_empty()) {
        let exact = cur.join(part);
        if exact.exists() {
            cur = exact;
            continue;
        }
        let found = std::fs::read_dir(&cur)
            .ok()?
            .flatten()
            .find(|e| e.file_name().to_string_lossy().eq_ignore_ascii_case(part))?;
        cur = found.path();
    }
    Some(cur)
}

/// Finds (or, for writing, makes room for) `name` in the saves directory.
fn resolve_in_saves(name: &str, for_write: bool) -> Option<PathBuf> {
    let dir = save_dir();
    if for_write {
        if let Err(e) = std::fs::create_dir_all(&dir) {
            eprintln!("Could not create {}: {e}", dir.display());
        }
        return Some(dir.join(name));
    }
    find_ignore_case(&dir, name)
}

/// The part after "df1:" if `name` starts with a disk device.
fn strip_disk_device(name: &str) -> Option<&str> {
    let (dev, rest) = name.split_once(':')?;
    let is_disk = dev.len() == DISK_DEVICE_NAME_LEN
        && dev[..DISK_DEVICE_PREFIX.len()].eq_ignore_ascii_case(DISK_DEVICE_PREFIX);
    is_disk.then_some(rest)
}

/// Maps an Amiga file name to a real path. `for_write` allows names that do not exist yet.
pub fn resolve(name: &str, for_write: bool) -> Option<PathBuf> {
    if let Some(rest) = strip_disk_device(name) {
        if rest.is_empty() {
            return Some(save_dir());
        }
        return resolve_in_saves(rest, for_write);
    }
    // game saves (A.faery ...) go to the saves directory, not the data directory
    if name.to_ascii_lowercase().ends_with(SAVE_EXTENSION) {
        return resolve_in_saves(name, for_write);
    }
    find_ignore_case(&data_dir(), name).or_else(|| for_write.then(|| data_dir().join(name)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disk_device_prefix_is_removed() {
        assert_eq!(strip_disk_device("df1:A.faery"), Some("A.faery"));
        assert_eq!(strip_disk_device("DF0:"), Some(""));
        assert_eq!(strip_disk_device("image"), None);
        assert_eq!(strip_disk_device("dh0:file"), None);
    }

    #[test]
    fn finds_files_ignoring_case() {
        let dir = std::env::temp_dir().join(format!("fta_paths_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("Fonts")).unwrap();
        std::fs::write(dir.join("Fonts").join("Amber"), b"x").unwrap();
        let found = find_ignore_case(&dir, "fonts/amber").unwrap();
        assert_eq!(found, dir.join("Fonts").join("Amber"));
        assert!(find_ignore_case(&dir, "fonts/missing").is_none());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
