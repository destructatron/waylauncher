use std::collections::HashMap;
use std::path::PathBuf;

use freedesktop_desktop_entry::{self as fde, DesktopEntry, Iter as DesktopIter};
use log::warn;

use crate::desktop_entry::DesktopEntryObject;

fn parse_entry(path: PathBuf, locales: &[String]) -> Option<DesktopEntryObject> {
    let data = std::fs::read_to_string(&path).ok()?;
    let entry = DesktopEntry::from_str(&path, &data, Some(locales.as_ref())).ok()?;

    // Filter: must be Type=Application
    let type_val = entry.type_().unwrap_or("");
    if type_val != "Application" {
        return None;
    }

    // Filter: skip NoDisplay=true and Hidden=true
    if entry.no_display() || entry.hidden() {
        return None;
    }

    let name = entry.name(locales)?.to_string();
    let exec = entry.exec()?.to_string();

    // Name and Exec are required
    if name.is_empty() || exec.is_empty() {
        return None;
    }

    let generic_name = entry
        .generic_name(locales)
        .map(|s| s.to_string())
        .unwrap_or_default();
    let comment = entry
        .comment(locales)
        .map(|s| s.to_string())
        .unwrap_or_default();
    let icon = entry.icon().unwrap_or("").to_string();
    let categories = entry
        .categories()
        .map(|cats| cats.join(";"))
        .unwrap_or_default();
    let terminal = entry.terminal();
    let desktop_file_path = path.to_string_lossy().to_string();

    Some(DesktopEntryObject::new(
        &name,
        &generic_name,
        &comment,
        &icon,
        &exec,
        &desktop_file_path,
        &categories,
        terminal,
    ))
}

fn get_locales() -> Vec<String> {
    fde::get_languages_from_env()
}

/// Scan all XDG application directories for .desktop files.
/// Deduplicates by appid with user dirs taking priority (they appear first).
pub fn scan_applications() -> Vec<DesktopEntryObject> {
    let locales = get_locales();
    let mut seen: HashMap<String, DesktopEntryObject> = HashMap::new();

    for path in DesktopIter::new(fde::default_paths()) {
        // Derive appid from filename (without .desktop extension)
        let appid = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        if appid.is_empty() {
            continue;
        }

        // User dirs appear first in default_paths(), so skip duplicates
        if seen.contains_key(&appid) {
            continue;
        }

        if let Some(entry) = parse_entry(path, &locales) {
            seen.insert(appid, entry);
        }
    }

    let mut entries: Vec<DesktopEntryObject> = seen.into_values().collect();
    entries.sort_by(|a, b| a.name().to_lowercase().cmp(&b.name().to_lowercase()));
    entries
}

/// Scan ~/Desktop for .desktop files.
pub fn scan_desktop_directory() -> Vec<DesktopEntryObject> {
    let locales = get_locales();
    let desktop_dir = match dirs::home_dir() {
        Some(home) => home.join("Desktop"),
        None => {
            warn!("Could not determine home directory");
            return Vec::new();
        }
    };

    if !desktop_dir.is_dir() {
        warn!("Desktop directory not found: {}", desktop_dir.display());
        return Vec::new();
    }

    let mut entries = Vec::new();

    let read_dir = match std::fs::read_dir(&desktop_dir) {
        Ok(rd) => rd,
        Err(e) => {
            warn!("Could not read desktop directory: {}", e);
            return Vec::new();
        }
    };

    for dir_entry in read_dir.flatten() {
        let path = dir_entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
            continue;
        }
        if let Some(entry) = parse_entry(path, &locales) {
            entries.push(entry);
        }
    }

    entries.sort_by(|a, b| a.name().to_lowercase().cmp(&b.name().to_lowercase()));
    entries
}
